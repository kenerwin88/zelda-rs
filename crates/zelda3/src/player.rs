// Methods ported from zelda3/src/player.c and included inside ZeldaState.

use super::sprite::SpriteSpawnInfo;
use super::*;
use crate::types::Point16U;

mod player_shared;
use player_shared::*;

const DOOR_ANIMATION_STEP_INDICATOR_PLAYER: usize = 0x0690;
const PUSH_BLOCK_DIRECTION_PLAYER: usize = 0x0474;
const SPRITE_C_PLAYER: usize = 0x0db0;
const DASH_FOLLOWER_SLOWDOWN_INDICATORS: [u8; 15] =
    [0xff, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const DASH_FOLLOWER_RELEASE_INDICATORS: [u8; 15] = [0xff, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn player_memory_location_to_give_item_to(item: u8) -> usize {
    PLAYER_MEMORY_LOCATION_TO_GIVE_ITEM_TO_MEMORY_LOCATIONS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
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
        let room = self.game_state.world.location.dungeon_room();
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        if let Some(frame) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME") {
            if self.game_state.frame.frame_counter as u16 != frame {
                return;
            }
        }
        if let Some(frame_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME_MIN") {
            if (self.game_state.frame.frame_counter as u16) < frame_min {
                return;
            }
        }
        if let Some(frame_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME_MAX") {
            if self.game_state.frame.frame_counter as u16 > frame_max {
                return;
            }
        }
        if let Some(expected_room) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_ROOM") {
            if room != expected_room {
                return;
            }
        }
        if let Some(expected_ow) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_OW") {
            if u16::from(self.game_state.world.location.overworld_screen_index()) != expected_ow {
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
            self.game_state.frame.frame_counter,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            self.game_state.player.follower_link.handler_state(),
            self.game_state.player.follower_link.auxiliary_state(),
            self.game_state.player.follower_link.incapacitated_timer(),
            self.game_state.player.follower_link.recoil_timer(),
            self.ram[SCRATCH_A],
            self.game_state.player.follower_link.z(),
            self.game_state.player.follower_link.actual_z_velocity(),
            self.game_state
                .player
                .follower_link
                .actual_z_velocity_copy(),
            self.game_state.player.follower_link.x_subpixel(),
            self.game_state.player.follower_link.y_subpixel(),
            self.game_state.player.follower_link.actual_x_velocity(),
            self.game_state.player.follower_link.actual_y_velocity(),
            self.game_state.player.follower_link.direction(),
            self.game_state.player.follower_link.last_direction(),
            self.game_state
                .player
                .follower_link
                .last_direction_moved_towards(),
            self.game_state.player.tile_detection.pit_tile(),
            self.game_state.player.follower_link.tile_below(),
            self.game_state.player.tile_detection.collision_bits(),
            self.game_state.player.tile_detection.normal_tiles(),
            self.game_state.player.follower_link.defense_flags(),
            self.game_state.player.follower_link.speed_setting(),
            self.game_state.player.follower_link.speed_modifier(),
        );
    }

    pub(super) fn replay_trace_drag_tail(&self, label: &str) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_SUB_FRAME").is_none() {
            return;
        }
        eprintln!(
            "drag-tail frame={} {label} r14=0x{:04x} tilecoll=0x{:02x} misc=0x{:04x} drag=0x{:02x} timer=0x{:02x} bframes=0x{:02x} lastmove=0x{:02x} face=0x{:02x}",
            self.game_state.frame.frame_counter,
            self.game_state.player.tile_detection.collision_bits(),
            self.game_state.player.follower_link.tile_coll_flag(),
            self.game_state.player.tile_detection.misc_tiles(),
            self.game_state.player.follower_link.defense_flags(),
            self.game_state.player.follower_link.push_fatigue_timer(),
            self.game_state.player.follower_link.button_b_frames(),
            self.game_state
                .player
                .follower_link
                .last_direction_moved_towards(),
            self.game_state.player.follower_link.facing(),
        );
    }

    pub(super) fn bit_sum4(value: u8) -> u8 {
        (value & 1) + ((value >> 1) & 1) + ((value >> 2) & 1) + ((value >> 3) & 1)
    }

    pub(super) fn dungeon_handle_layer_change(&mut self) {
        self.follower_link_state_mut().mark_lower_level_mirror();
        if self
            .game_state
            .dungeon
            .stair_movement
            .kind_of_in_room_staircase()
            == 0
        {
            self.increment_dungeon_room_index_by(16);
        }
        if self
            .game_state
            .dungeon
            .stair_movement
            .kind_of_in_room_staircase()
            != 2
        {
            self.follower_link_state_mut().mark_lower_level();
        }
        self.follower_link_state_mut()
            .clear_about_to_jump_off_ledge();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn check_ability_to_swim(&mut self) {
        self.replay_trace_submodule("check_ability_to_swim-entry");
        if !self.game_state.player.follower_link.is_bunny_mirror()
            && self.game_state.player.follower_link.has_flippers()
        {
            return;
        }
        if self.game_state.player.follower_link.has_moon_pearl() {
            self.follower_link_state_mut().clear_bunny_mirror();
        }
        self.follower_link_state_mut().set_visibility_status(0x0c);
        let submodule = if self.game_state.world.location.is_indoors() {
            20
        } else {
            42
        };
        self.set_submodule(submodule);
        self.replay_trace_submodule("check_ability_to_swim-exit");
    }

    pub(super) fn link_initialize(&mut self) {
        self.follower_link_state_mut()
            .initialize_link_action_state();
        self.link_reset_swimming_state();
        self.follower_link_state_mut()
            .finish_link_action_state_initialization();
        self.link_force_unequip_cape_quietly();
        self.link_reset_sword_and_item_usage();

        if self
            .game_state
            .enhanced_features
            .has(LINK_INITIALIZE_FEATURES0_MISC_BUG_FIXES)
        {
            self.follower_link_state_mut()
                .clear_misc_bugfix_movement_state();
            self.set_bg1_y_offset(0);
            self.set_bg1_x_offset(0);

            if !self.game_state.player.follower_link.has_moon_pearl()
                && self.game_state.inventory.save_progress.dark_world_state() != 0
            {
                self.follower_link_state_mut().become_bunny_handler();
                self.load_gear_palettes_bunny();
            }
        }
    }

    pub(super) fn link_reset_properties_a(&mut self) {
        self.follower_link_state_mut().reset_properties_a_fields();
        // C's Link_ResetProperties_A also zeroes is_archer_or_shovel_game,
        // tagalong_event_flags, and BYTE(tiledetect_tile_type); route those through
        // the native states that own them so no model goes stale.
        self.minigame_state_mut().clear_is_archer_or_shovel_game();
        self.follower_state_mut().clear_event_flags();
        self.tile_detect_position_mut().clear_tile_type_low();
        self.link_reset_swimming_state();
        self.link_reset_properties_b();
    }

    pub(super) fn link_reset_properties_b(&mut self) {
        self.follower_link_state_mut().reset_properties_b_fields();
        self.link_reset_properties_c();
    }

    pub(super) fn link_reset_properties_c(&mut self) {
        if self
            .game_state
            .enhanced_features
            .has(LINK_RESET_PROPERTIES_C_FEATURES0_MISC_BUG_FIXES)
        {
            self.clear_custom_spell_animation();
        }

        self.follower_link_state_mut().reset_properties_c_fields();
        self.link_reset_sword_and_item_usage();
    }

    pub(super) fn link_tuck_into_bed(&mut self) {
        self.follower_link_state_mut().set_y(0x215a);
        self.follower_link_state_mut().set_x(0x0940);
        self.follower_link_state_mut().setup_bed_pose();
        self.ancilla_add_blanket(0x20);
    }

    pub(super) fn link_reset_swimming_state(&mut self) {
        self.follower_link_state_mut().reset_swimming_state_fields();
        self.reset_all_acceleration();
    }

    pub(super) fn link_reset_state_after_damaging_pit(&mut self) {
        self.link_reset_swimming_state();
        self.follower_link_state_mut().reset_after_damaging_pit();
    }

    pub(super) fn link_state_bunny_recache(&mut self) {
        self.follower_link_state_mut().recache_bunny_state();
        self.link_reset_swimming_state();
        self.follower_link_state_mut().set_handler_state(2);
        if self.game_state.player.follower_link.has_moon_pearl() {
            self.follower_link_state_mut().clear_handler_state();
            self.load_actual_gear_palettes();
        }
    }

    pub(super) fn link_set_to_deep_water(&mut self) {
        self.follower_link_state_mut().enter_deep_water();
        self.link_reset_swimming_state();
    }

    pub(super) fn link_splash_upon_landing(&mut self) {
        if self.game_state.player.follower_link.is_bunny_mirror() {
            if self.game_state.player.follower_link.is_in_deep_water() {
                self.ancilla_add_splash(21, 0);
                self.link_state_bunny_recache();
                return;
            }
            self.follower_link_state_mut().land_after_splash();
        } else if self.game_state.player.follower_link.is_in_deep_water() {
            if self.game_state.player.follower_link.handler_state()
                != LINK_SPLASH_UPON_LANDING_PLAYER_HANDLER_STATE_RECOIL_OTHER
            {
                self.ancilla_add_splash(21, 0);
            }
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut().land_after_splash();
        } else {
            self.follower_link_state_mut().land_after_splash();
        }
    }

    pub(super) fn link_handle_swim_accels(&mut self) {
        let mut mask = 0x0c;
        for offset in [0, 2] {
            if self.game_state.player.follower_link.joypad1h_last() & mask != 0 {
                let acceleration = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration(offset);
                let max_speed = self.game_state.player.swim_acceleration.max_speed(offset);
                if acceleration != 0 && max_speed >= 384 {
                    let target = LINK_HANDLE_SWIM_ACCELS_SWIM_ACCELERATION_TARGETS
                        .iter()
                        .copied()
                        .find(|value| *value >= acceleration)
                        .unwrap_or(384);
                    self.swim_acceleration_mut().set_max_speed(offset, target);
                } else if max_speed != 0 {
                    let target = max_speed.wrapping_add(160).min(384);
                    self.swim_acceleration_mut().set_max_speed(offset, target);
                } else {
                    self.swim_acceleration_mut().set_acceleration(offset, 1);
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_flag_max_accels(&mut self) {
        if self.game_state.player.follower_link.flag_moving() == 0 {
            return;
        }

        for offset in [2, 0] {
            let acceleration = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset);
            if acceleration != 0 {
                self.swim_acceleration_mut()
                    .set_max_speed(offset, acceleration);
                self.swim_acceleration_mut().set_mode(offset, 1);
            }
        }
    }

    pub(super) fn link_set_ice_max_accel(&mut self) {
        if self.game_state.player.follower_link.flag_moving() == 0 {
            return;
        }
        self.swim_acceleration_mut().set_max_speed(0, 0x0180);
        self.swim_acceleration_mut().set_max_speed(2, 0x0180);
    }

    pub(super) fn link_set_momentum(&mut self) {
        let joy = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        let mut mask = 0x0c;
        let mut bit = 0x08;
        for offset in [0, 2] {
            if joy & mask != 0 {
                let flag_moving = self.game_state.player.follower_link.flag_moving();
                let stroke_timer = if flag_moving != 0 {
                    LINK_SET_MOMENTUM_SWIM_STROKE_TIMERS_BY_MOVING_FLAG[(flag_moving - 1) as usize]
                } else {
                    32
                };
                self.follower_link_state_mut()
                    .set_swim_stroke_frame_counter(offset, stroke_timer as u16);

                if (self.game_state.player.follower_link.swim_direction_flags()
                    | self.game_state.player.follower_link.direction())
                    & mask
                    == mask
                {
                    self.swim_acceleration_mut().set_mode(offset, 2);
                } else {
                    let direction = if joy & bit != 0 { 0 } else { 1 };
                    self.swim_acceleration_mut()
                        .set_acceleration_direction(offset, direction);
                    self.swim_acceleration_mut().set_mode(offset, 0);
                }

                if self.game_state.player.swim_acceleration.max_speed(offset) == 0 {
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub(super) fn player_handler_04_swimming(&mut self) {
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            self.follower_link_state_mut()
                .interrupt_swimming_for_auxiliary_state();
            self.reset_all_acceleration();
            self.link_state_recoil();
            return;
        }

        self.follower_link_state_mut().clear_swimming_action_state();
        if !self.game_state.player.follower_link.has_flippers() {
            return;
        }

        let has_swim_velocity = self.game_state.player.swim_acceleration.acceleration(0)
            | self.game_state.player.swim_acceleration.acceleration(2)
            != 0;
        if !has_swim_velocity {
            let swim = &self.game_state.player.swim_acceleration;
            if swim.mode_low(0) != 2 && swim.mode_low(1) != 2 {
                self.reset_all_acceleration();
            }
            self.follower_link_state_mut().advance_idle_swim_animation();
        } else {
            self.follower_link_state_mut()
                .advance_active_swim_animation(
                    &PLAYER_HANDLER_04_SWIMMING_ACTIVE_SWIM_ANIMATION_DELAYS,
                );
        }

        if self.game_state.player.follower_link.hard_swim_stroke() == 0 {
            let hard_stroke = ((self.game_state.player.follower_link.filtered_joypad_l() & 0x80)
                | self.game_state.player.follower_link.filtered_joypad_h())
                & 0xc0;
            if !has_swim_velocity || hard_stroke == 0 {
                self.link_handle_swim_movements();
                return;
            }
            self.follower_link_state_mut()
                .start_hard_swim_stroke(hard_stroke);
            self.ancilla_sfx2_near(37);
            self.link_handle_swim_accels();
        }

        self.follower_link_state_mut().tick_hard_swim_stroke();
        self.link_handle_swim_movements();
    }

    fn advance_link_animation_step(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .follower_link_state_mut()
            .advance_frame_change_counter(delay)
        {
            self.follower_link_state_mut()
                .advance_animation_step(wrap_at, wrap_to);
        }
    }

    fn advance_link_animation_step_at_least(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .follower_link_state_mut()
            .advance_frame_change_counter(delay)
        {
            self.follower_link_state_mut()
                .advance_animation_step_at_least(wrap_at, wrap_to);
        }
    }

    pub(super) fn link_handle_swim_movements(&mut self) {
        let mut direction = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if direction == 0 {
            direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        }

        if direction == 0 {
            self.follower_link_state_mut()
                .clear_swim_movement_velocity();
            self.link_flag_max_accels();
            if self.game_state.player.follower_link.flag_moving() != 0 {
                if self.game_state.player.follower_link.is_running() {
                    direction = self.game_state.player.follower_link.swim_direction_flags();
                } else {
                    if self.game_state.player.swim_acceleration.acceleration(0)
                        | self.game_state.player.swim_acceleration.acceleration(2)
                        == 0
                    {
                        self.follower_link_state_mut().clear_defense_flags();
                        self.link_reset_swimming_state();
                    }
                    self.finish_swim_movement_tail();
                    return;
                }
            } else {
                self.follower_link_state_mut()
                    .reset_idle_swim_animation_if_out_of_water();
                self.finish_swim_movement_tail();
                return;
            }
        }

        if direction != self.game_state.player.follower_link.swim_direction_flags() {
            self.follower_link_state_mut()
                .set_swim_direction_flags(direction);
            self.follower_link_state_mut()
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
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_set_the_max_accel(&mut self) {
        if self.game_state.player.follower_link.flag_moving() != 0
            || self.game_state.player.follower_link.hard_swim_stroke() != 0
        {
            return;
        }

        let mut mask = 0x0c;
        for offset in [0, 2] {
            let mode = self.game_state.player.swim_acceleration.mode(offset);
            if self.game_state.player.follower_link.joypad1h_last() & mask != 0 && mode != 2 {
                let speed_active = self
                    .game_state
                    .player
                    .swim_acceleration
                    .speed_active_flag(offset);
                let acceleration = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration(offset);
                let max_speed = self.game_state.player.swim_acceleration.max_speed(offset);
                if speed_active != 0 || (acceleration >= 240 && acceleration >= max_speed) {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    if acceleration >= 240 {
                        self.swim_acceleration_mut()
                            .set_speed_active_flag(offset, 1);
                        self.swim_acceleration_mut().set_mode(offset, 1);
                    } else {
                        self.swim_acceleration_mut().set_max_speed(offset, 240);
                        self.swim_acceleration_mut()
                            .set_speed_active_flag(offset, 0);
                    }
                }
            } else {
                self.swim_acceleration_mut().set_max_speed(offset, 240);
                self.swim_acceleration_mut()
                    .set_speed_active_flag(offset, 0);
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_handle_toss(&mut self) -> bool {
        if self.game_state.player.follower_link.y_button_action_flags() & 0x80 == 0
            || self.game_state.player.follower_link.filtered_joypad_l() & 0x80 == 0
            || self.game_state.player.follower_link.is_lift_throw_primed()
        {
            return false;
        }

        self.follower_link_state_mut()
            .clear_lift_throw_scratch_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        true
    }

    pub(super) fn link_cancel_dash(&mut self) {
        if !self.game_state.player.follower_link.is_running() {
            return;
        }
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x1e {
                self.ancilla_slot_view_mut(i).set_ancilla_type(0);
            }
        }
        self.follower_link_state_mut().cancel_dash_state();
        self.swim_acceleration_mut().clear_mode_low_axis();
    }

    pub(super) fn repel_dash(&mut self) {
        if self.game_state.player.follower_link.is_running()
            && self.game_state.player.follower_link.dash_counter() != 64
        {
            self.link_reset_swimming_state();
            self.ancilla_add_dash_tremor(29, 1);
            self.prepare_apply_rumble_to_sprites();
            if self.game_state.system_signals.sound_effect_2() & 0x3f != 27
                && self.game_state.system_signals.sound_effect_2() & 0x3f != 50
            {
                self.ancilla_sfx3_near(3);
            }
            self.link_apply_tile_rebound();
        }
    }

    pub(super) fn sprite_repel_dash(&mut self) {
        self.follower_link_state_mut()
            .set_last_direction_moved_towards_from_facing();
        self.repel_dash();
    }

    pub(super) fn link_apply_tile_rebound(&mut self) {
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards() as usize;
        self.follower_link_state_mut().set_actual_velocity_xy(
            LINK_APPLY_TILE_REBOUND_DASH_REBOUND_X_VELOCITY_BY_DIRECTION[dir],
            LINK_APPLY_TILE_REBOUND_DASH_REBOUND_Y_VELOCITY_BY_DIRECTION[dir],
        );
        self.follower_link_state_mut().set_incapacitated_timer(24);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(36);
        if self.game_state.player.follower_link.flag_moving() != 0 {
            self.follower_link_state_mut().set_direction_and_swim_flags(
                LINK_APPLY_TILE_REBOUND_DIRECTION_BITS_BY_FACING[dir],
            );
            self.swim_acceleration_mut().set_acceleration_direction(
                0,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_DIRECTIONS[dir] as u16,
            );
            self.swim_acceleration_mut().set_acceleration_direction(
                2,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_DIRECTIONS[dir] as u16,
            );
            let i = (self.game_state.player.follower_link.flag_moving() - 1) as usize * 4 + dir;
            self.swim_acceleration_mut().set_acceleration(
                0,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_ACCELERATIONS[i],
            );
            self.swim_acceleration_mut().set_acceleration(
                2,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_ACCELERATIONS[i],
            );
        }
        self.follower_link_state_mut().set_auxiliary_state(1);
        self.follower_link_state_mut().set_dash_noise_request();
        self.tile_detect_position_mut()
            .clear_interaction_scratch_x_low();
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 2
            != 0
        {
            self.follower_link_state_mut().set_y_velocity(0);
        } else {
            self.follower_link_state_mut().set_x_velocity(0);
        }
    }

    pub(super) fn link_handle_moving_animation_full_long_entry(&mut self) {
        if self.game_state.player.follower_link.handler_state() == 4 {
            self.link_handle_moving_animation_swimming();
            return;
        }

        let mut r0 = self.game_state.player.follower_link.last_direction();
        if r0 == 0 {
            return;
        }
        if self.game_state.player.follower_link.flag_moving() != 0 {
            r0 = self.game_state.player.follower_link.swim_direction_flags();
        }
        if self.game_state.player.follower_link.direction_lock() == 0 {
            let mut y;
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
            {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            } else if self.game_state.player.follower_link.doorway_state() != 0 {
                y = self
                    .game_state
                    .player
                    .follower_link
                    .doorway_state()
                    .wrapping_mul(2)
                    & !3;
            } else if r0
                & LINK_HANDLE_MOVING_ANIMATION_FULL_LONG_ENTRY_FACING_DIRECTION_BITS
                    [self.game_state.player.follower_link.facing_index()]
                != 0
            {
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
            self.follower_link_state_mut().set_facing(y);
        }
        self.link_handle_moving_animation_start_with_dash();
    }

    pub(super) fn link_handle_moving_animation_start_with_dash(&mut self) {
        if self.game_state.player.follower_link.is_running() {
            self.link_handle_moving_animation_dash();
            return;
        }
        let mut x = self.game_state.player.follower_link.facing() >> 1;
        if self.game_state.player.follower_link.speed_setting() == 6 {
            x = x.wrapping_add(4);
        } else if self.game_state.player.follower_link.flag_moving() != 0 {
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f == 0 {
                self.follower_link_state_mut().clear_animation_step();
                return;
            }
            x = x.wrapping_add(4);
        }

        if self.game_state.player.follower_link.handler_state() == 23
            || (self.game_state.enhanced_features.has(4096)
                && self.game_state.player.follower_link.handler_state() == 28)
        {
            if self.game_state.player.follower_link.animation_step() < 4
                && self.game_state.player.follower_link.on_somaria_platform() != 2
            {
                self.advance_link_animation_step(
                    LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS[x as usize],
                    4,
                    0,
                );
            } else {
                self.follower_link_state_mut().clear_animation_step();
            }
            return;
        }

        if self.game_state.frame.submodule == 18 || self.game_state.frame.submodule == 19 {
            x = 12;
        } else if self.game_state.frame.submodule != 14
            && !self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            if self.game_state.player.follower_link.defense_flags() & 0x8d != 0 {
                x = 12;
            } else if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 0
                && self.game_state.player.follower_link.button_b_frames() == 0
            {
                let mut idx = self.game_state.player.follower_link.animation_step();
                if self.game_state.player.follower_link.speed_setting() == 6 {
                    idx = idx.wrapping_add(8);
                }
                if self.game_state.player.follower_link.flag_moving() != 0 {
                    idx = idx.wrapping_add(8);
                }
                if self.game_state.player.follower_link.on_somaria_platform() != 2 {
                    self.advance_link_animation_step(
                        LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_WALK_ANIMATION_DELAYS
                            [idx as usize],
                        9,
                        1,
                    );
                }
                return;
            }
        }

        if self.game_state.player.follower_link.animation_step() < 6
            && self.game_state.player.follower_link.on_somaria_platform() != 2
        {
            self.advance_link_animation_step(
                LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS[x as usize],
                6,
                0,
            );
        } else {
            self.follower_link_state_mut().clear_animation_step();
        }
    }

    pub(super) fn link_handle_moving_animation_swimming(&mut self) {
        let r0 = self.game_state.player.follower_link.swim_direction_flags();
        if r0 == 0 || self.game_state.player.follower_link.direction_lock() != 0 {
            return;
        }
        let mut y;
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            != 0
        {
            if self.game_state.player.follower_link.doorway_state() != 0 {
                y = self
                    .game_state
                    .player
                    .follower_link
                    .doorway_state()
                    .wrapping_mul(2)
                    & !3;
            } else if r0
                & LINK_HANDLE_MOVING_ANIMATION_SWIMMING_FACING_DIRECTION_BITS
                    [self.game_state.player.follower_link.facing_index()]
                != 0
            {
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
        self.follower_link_state_mut().set_facing(y);
    }

    pub(super) fn link_handle_moving_animation_dash(&mut self) {
        let mut t = 6usize;
        while self.game_state.player.follower_link.dash_countdown()
            >= LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_THRESHOLDS[t]
            && t != 0
        {
            t -= 1;
        }
        if self.game_state.player.follower_link.button_b_frames() < 9
            && self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 0
        {
            self.advance_link_animation_step(
                LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_DELAYS[t * 8],
                9,
                1,
            );
        } else {
            self.advance_link_animation_step_at_least(
                LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_MIN_ANIM_STEPS[t],
                6,
                0,
            );
        }
    }

    pub(super) fn link_apply_moving_floor_velocity(&mut self) {
        self.follower_link_state_mut()
            .clear_orthogonal_direction_count();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_velocity());
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().set_x(x);
    }

    pub(super) fn link_apply_conveyor(&mut self) {
        if self.game_state.player.follower_link.conveyor_belt_state() == 0 {
            return;
        }
        if !self
            .game_state
            .player
            .follower_link
            .is_grounded_or_z_sentinel()
        {
            return;
        }
        if self.game_state.player.follower_link.grabbing_wall_has(1)
            || self.game_state.player.follower_link.handler_state() == 19
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return;
        }
        let j = self
            .game_state
            .player
            .follower_link
            .conveyor_belt_state()
            .wrapping_sub(1) as usize;
        if j >= LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG.len() {
            return;
        }
        if self.game_state.player.follower_link.is_running()
            && self.game_state.player.follower_link.dash_counter() == 32
            && self.game_state.player.follower_link.direction()
                & LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j]
                != 0
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_orthogonal_direction_count();
        self.follower_link_state_mut()
            .add_direction_flags(LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j]);
        let y = ((self.game_state.player.follower_link.y() as u32) << 8
            | self.game_state.player.bg1_movement_accumulator.y_subpixel() as u32)
            .wrapping_add(((LINK_APPLY_CONVEYOR_MOVING_BELT_Y[j] as i32) << 4) as u32);
        self.bg1_move_calc_mut().set_y_subpixel(y as u8);
        self.follower_link_state_mut().set_y((y >> 8) as u16);
        let x = ((self.game_state.player.follower_link.x() as u32) << 8
            | self.game_state.player.bg1_movement_accumulator.x_subpixel() as u32)
            .wrapping_add(((LINK_APPLY_CONVEYOR_MOVING_BELT_X[j] as i32) << 4) as u32);
        self.bg1_move_calc_mut().set_x_subpixel(x as u8);
        self.follower_link_state_mut().set_x((x >> 8) as u16);
    }

    pub(super) fn flag67_with_directions(&mut self) {
        self.follower_link_state_mut()
            .derive_direction_from_actual_velocity();
    }

    pub(super) fn link_add_in_velocity_y_falling(&mut self) {
        let adjust = i16::from(self.game_state.player.tile_detection.y_low() & 7)
            - if self
                .game_state
                .player
                .follower_link
                .y_velocity_signed()
                .is_negative()
            {
                8
            } else {
                0
            };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(adjust as u16);
        self.follower_link_state_mut().set_y(y);
    }

    pub(super) fn link_add_in_velocity_y(&mut self) {
        let y =
            self.game_state.player.follower_link.y().wrapping_sub(
                self.game_state.player.follower_link.y_velocity() as i8 as i16 as u16,
            );
        self.follower_link_state_mut().set_y(y);
    }

    pub(super) fn player_change_z(&mut self, z_delta: u8) {
        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.z_low() == 0 {
                return;
            }
            if (self.game_state.player.follower_link.z_low() as i8).is_negative() {
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                return;
            }
        }
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(z_delta);
    }

    pub(super) fn link_move_position(&mut self) {
        if let Some(position_return) = self.link_move_position_until_coordinates_integrated() {
            self.complete_link_move_position_after_coordinates(position_return);
        }
    }

    fn link_move_position_until_coordinates_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);

        if self.game_state.player.follower_link.handler_state() != 10
            && self.game_state.player.follower_link.on_somaria_platform() == 2
        {
            self.link_handle_velocity_and_sand_drag(x, y);
            return None;
        }

        let actual_x_velocity = self.game_state.player.follower_link.actual_x_velocity();
        let actual_y_velocity = self.game_state.player.follower_link.actual_y_velocity();
        self.follower_link_state_mut()
            .move_x_by_velocity(actual_x_velocity);
        self.follower_link_state_mut()
            .move_y_by_velocity(actual_y_velocity);
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            let actual_z_velocity = self.game_state.player.follower_link.actual_z_velocity();
            self.follower_link_state_mut()
                .move_z_by_velocity(actual_z_velocity);
        }

        Some(LinkMovePositionReturn { old_x: x, old_y: y })
    }

    pub(super) fn complete_link_move_position_after_coordinates(
        &mut self,
        position_return: LinkMovePositionReturn,
    ) {
        self.link_handle_moving_floor();
        self.link_apply_conveyor();
        self.link_handle_velocity_and_sand_drag(position_return.old_x, position_return.old_y);
    }

    pub(super) fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(self.game_state.player.follower_link.drag_player_y());
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(self.game_state.player.follower_link.drag_player_x());
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().set_x(x);
        self.follower_link_state_mut()
            .set_movement_velocity_from_position_delta(x, y, old_x, old_y);
    }

    pub(super) fn link_handle_moving_floor(&mut self) {
        if self.game_state.dungeon.room_load.header_collision() == 0 {
            return;
        }
        let z = self.game_state.player.follower_link.z_low();
        if z != 0 && z != 0xff {
            return;
        }
        if !self
            .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH)
        {
            return;
        }
        if self.game_state.player.follower_link.handler_state() == 19 {
            return;
        }

        let floor_y = self.game_state.dungeon.moving_floor.floor_y_velocity();
        let floor_x = self.game_state.dungeon.moving_floor.floor_x_velocity();
        self.follower_link_state_mut()
            .mark_moving_floor_direction(floor_y, floor_x);

        self.link_apply_moving_floor_velocity();
    }

    pub(super) fn check_if_room_needs_double_layer_check(&mut self) -> bool {
        if self.game_state.dungeon.room_load.header_collision() == 0
            || self.game_state.dungeon.room_load.header_collision() == 4
        {
            return false;
        }

        if self.game_state.dungeon.room_load.header_collision() >= 2 {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_v_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            self.follower_link_state_mut().set_y(y);

            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_h_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut()
                .cache_moving_floor_position(x, y);
        }
        self.follower_link_state_mut().mark_lower_level();
        true
    }

    pub(super) fn create_velocity_from_moving_background(&mut self) {
        if self.game_state.dungeon.room_load.header_collision() != 1 {
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(self.game_state.player.follower_link.moving_floor_x());
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.player.follower_link.moving_floor_y());
            let new_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_v_copy2());
            let new_x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2());
            self.follower_link_state_mut().set_y(new_y);
            self.follower_link_state_mut().set_x(new_x);
            if self.game_state.player.follower_link.direction() != 0 {
                self.follower_link_state_mut()
                    .add_movement_velocity_delta(x, y);
            }
        }
        self.follower_link_state_mut().clear_lower_level();
    }

    pub(super) fn calculate_snap_scratch_y(&mut self) {
        let mut y_vel = self.game_state.player.follower_link.y_velocity() as i8;
        if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            if y_vel >= 0 {
                y_vel = y_vel.wrapping_neg();
            }
        } else if y_vel < 0 {
            y_vel = y_vel.wrapping_neg();
        }

        let x = self.game_state.player.follower_link.x();
        let delta = if y_vel < 0 { -1i16 } else { 1i16 };
        self.follower_link_state_mut()
            .set_x(x.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_y(&mut self) {
        self.follower_link_state_mut().set_direction_lock_bits(2);
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let t = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if t & 7 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            return;
        }

        let mut t = self.game_state.player.follower_link.y_velocity();
        let dir = if self.game_state.player.follower_link.x_low() >= 0x80 {
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
        if !self.game_state.player.follower_link.direction_lock_has(1) {
            self.follower_link_state_mut().set_facing(dir);
        }
        let x = self.game_state.player.follower_link.x();
        self.follower_link_state_mut()
            .set_x(x.wrapping_add(vel as u16));
    }

    pub(super) fn snap_on_x(&mut self) {
        let x = self.game_state.player.follower_link.x();
        let adjust = (x & 7).wrapping_sub(
            if (self.game_state.player.follower_link.x_velocity() as i8).is_negative() {
                8
            } else {
                0
            },
        );
        self.follower_link_state_mut().set_x(x.wrapping_sub(adjust));
    }

    pub(super) fn calculate_snap_scratch_x(&mut self) {
        let mut x_vel = self.game_state.player.follower_link.x_velocity() as i8;
        if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
        } else if x_vel < 0 {
            x_vel = x_vel.wrapping_neg();
        }

        let y = self.game_state.player.follower_link.y();
        let delta = if x_vel < 0 { -1i16 } else { 1i16 };
        self.follower_link_state_mut()
            .set_y(y.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_x(&mut self) -> i8 {
        self.follower_link_state_mut().set_direction_lock_bits(2);
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let r0 = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if r0 & 7 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            return r0 as i8;
        }

        let mut x_vel = self.game_state.player.follower_link.x_velocity_signed();
        let dir = if self.game_state.player.follower_link.y_low() >= 0x80 {
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
        if !self.game_state.player.follower_link.direction_lock_has(1) {
            self.follower_link_state_mut().set_facing(dir);
        }
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
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
            + if self.game_state.player.follower_link.is_on_lower_level() {
                0x1000
            } else {
                0
            };
        self.game_state.dungeon.bg2_attributes.bg2_attr(offset)
    }

    pub(super) fn link_handle_change_in_z_velocity(&mut self) {
        self.player_change_z(
            if self.game_state.player.follower_link.handler_state() == 19 {
                1
            } else {
                2
            },
        );
    }

    pub(super) fn run_slope_collision_checks_vertical_first(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x20
            == 0
        {
            self.start_movement_collision_checks_y();
        }
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x10
            == 0
        {
            self.start_movement_collision_checks_x();
        }
    }

    pub(super) fn run_slope_collision_checks_horizontal_first(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x10
            == 0
        {
            self.start_movement_collision_checks_x();
        }
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x20
            == 0
        {
            self.start_movement_collision_checks_y();
        }
    }

    pub(super) fn link_hop_in_or_out_of_water_y(&mut self) {
        let ts = if self.game_state.world.location.is_outdoors() {
            2
        } else if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            0
        } else {
            self.game_state.display.sub_screen_layers
        };

        let mut vel = LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Y[ts as usize];
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            == 0
        {
            vel = 0u8.wrapping_sub(vel);
        }

        self.follower_link_state_mut()
            .set_actual_velocity_xy(0, vel);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Z[ts as usize],
            );
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().enter_water_hop_state();
    }

    pub(super) fn link_hop_in_or_out_of_water_x(&mut self) {
        let ts = if self.game_state.world.location.is_outdoors() {
            2
        } else if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            0
        } else {
            self.game_state.display.sub_screen_layers
        };

        let mut vel = LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_X[ts as usize];
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 1
            == 0
        {
            vel = 0u8.wrapping_sub(vel);
        }
        self.follower_link_state_mut()
            .set_actual_velocity_xy(vel, 0);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_Z[ts as usize],
            );
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().enter_water_hop_state();
    }

    pub(super) fn run_ledge_hop_timer(&mut self) -> bool {
        let mut rv = false;
        if !self
            .game_state
            .player
            .follower_link
            .is_in_auxiliary_state(1)
        {
            if !self.game_state.player.follower_link.is_running() {
                if self
                    .follower_link_state_mut()
                    .tick_jump_ledge_timer_or_reset()
                {
                    return true;
                }
            } else {
                rv = true;
            }
        }
        self.follower_link_state_mut()
            .restore_position_from_previous();
        self.follower_link_state_mut().clear_movement_subpixels();
        rv
    }

    /// Ledge-hop sound. C plays this via `Ancilla_Sfx2_Near(0x20)`, which routes through
    /// `PlaySfx_SetPanFrom`: it stamps RAW_SFX_PAN_VALUE (0xcf8) = 0x20 AND sets
    /// SOUND_EFFECT_1 = `0x20 | pan`. `ancilla_sfx2_near` mirrors that exactly.
    fn link_ledge_hop_sfx(&mut self) {
        self.ancilla_sfx2_near(0x20);
    }

    pub(super) fn flag_moving_into_slopes_y(&mut self) {
        let diag_state = self.game_state.player.tile_detection.diag_state() as usize;
        let x = self.game_state.player.follower_link.x().wrapping_sub(
            if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
                1
            } else {
                0
            },
        );
        let o = diag_state * 4 + (x & 7) as usize;
        let mut y = (self.game_state.player.tile_detection.y_low() & 7) as i8;

        if self.game_state.player.tile_detection.diagonal_tile() & 5 != 0 {
            let mut ym = (self.game_state.player.tile_detection.y_low() & 7) as i8;
            if self.game_state.enhanced_features.has(4096) {
                if diag_state & 2 != 0 {
                    ym = ym.wrapping_neg();
                } else {
                    ym = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(8 - ym);
                }
            } else {
                if diag_state & 2 == 0 {
                    ym = 8 - ym;
                } else {
                    ym += 8;
                }
                ym = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(ym);
            }

            let y_velocity = self.game_state.player.follower_link.y_velocity_signed();
            if y_velocity == 0 {
                return;
            }
            if y_velocity.is_negative() {
                ym = ym.wrapping_neg();
            }
            y = ym;
        } else {
            y = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(y);
        }

        if self
            .game_state
            .player
            .follower_link
            .y_velocity_signed()
            .is_negative()
        {
            if y <= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y as i16 as u16);
            self.follower_link_state_mut().set_y(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(8);
        } else {
            if y >= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y as i16 as u16);
            self.follower_link_state_mut().set_y(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(4);
        }

        let diag_flags = if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
            0x12
        } else {
            0x11
        };
        self.follower_link_state_mut()
            .add_moving_against_diag_tile_flags(diag_flags);
    }

    pub(super) fn flag_moving_into_slopes_x(&mut self) {
        let diag_state = self.game_state.player.tile_detection.diag_state();
        let mut x = (self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(if diag_state == 6 { 1 } else { 0 })
            & 7) as i8;
        let which_y_offset =
            if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
                2
            } else {
                0
            };
        let o = diag_state as usize * 4
            + (self
                .game_state
                .player
                .tile_detection
                .y_low_at(which_y_offset)
                & 7) as usize;

        if self.game_state.player.tile_detection.diagonal_tile() & 5 != 0 {
            let mut xm = (self.game_state.player.follower_link.x() & 7) as i8;
            if diag_state != 4 && diag_state != 6 {
                xm = xm.wrapping_neg();
            } else {
                xm = FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER[o].wrapping_sub(8 - xm);
            }
            let x_velocity = self.game_state.player.follower_link.x_velocity_signed();
            if x_velocity == 0 {
                return;
            }
            if x_velocity.is_negative() {
                xm = xm.wrapping_neg();
            }
            x = xm;
        } else {
            x = FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER[o].wrapping_sub(x);
        }

        if self
            .game_state
            .player
            .follower_link
            .x_velocity_signed()
            .is_negative()
        {
            if x <= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(x as i16 as u16);
            self.follower_link_state_mut().set_x(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(2);
        } else {
            if x >= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(x as i16 as u16);
            self.follower_link_state_mut().set_x(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(1);
        }

        self.follower_link_state_mut()
            .add_moving_against_diag_tile_flags(if diag_state & 2 != 0 { 0x28 } else { 0x24 });
    }

    pub(super) fn player_something_with_velocity_tired_or_swim(&mut self, xvel: u16, yvel: u16) {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(old_x, old_y);

        self.follower_link_state_mut()
            .move_x_by_subpixel_delta(xvel);
        let u = (xvel >> 8) as u8;
        let actual_x_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((xvel as u8) >> 4);

        self.follower_link_state_mut()
            .move_y_by_subpixel_delta(yvel);
        let u = (yvel >> 8) as u8;
        let actual_y_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((yvel as u8) >> 4);
        self.follower_link_state_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);

        if self.game_state.dungeon.room_load.header_collision() == 4 {
            self.link_apply_moving_floor_velocity();
        }
        self.follower_link_state_mut().clear_page_movement_deltas();
        self.link_handle_velocity_and_sand_drag(old_x, old_y);
    }

    pub(super) fn link_check_for_edge_screen_transition(&mut self) -> bool {
        if self
            .game_state
            .player
            .follower_link
            .is_edge_transition_blocked_by_handler_state()
            || self.game_state.player.follower_link.incapacitated_timer() == 0
        {
            return false;
        }
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_recoil_timer(3);
        self.follower_link_state_mut()
            .restore_position_from_previous();
        true
    }

    pub(super) fn player_limit_directions_inner(&mut self) {
        self.follower_link_state_mut().reset_direction_limits();

        let direction = self.game_state.player.follower_link.direction();
        if direction & 0x0c != 0 {
            self.follower_link_state_mut()
                .increment_orthogonal_direction_count();
            let last_direction_moved_towards = if direction & 8 != 0 { 0 } else { 1 };
            self.follower_link_state_mut()
                .set_last_direction_moved_towards(last_direction_moved_towards);
            self.tile_detect_movement_vertical_slopes(last_direction_moved_towards as u16);

            let r14 = self.game_state.player.tile_detection.collision_bits();
            if r14 & 0x30 != 0
                && self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 == 0
                && ((((r14 & 0x30) >> 4) as u8) & self.game_state.player.follower_link.direction())
                    == 0
                && self.game_state.player.follower_link.direction() & 3 != 0
            {
                let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[if self
                    .game_state
                    .player
                    .follower_link
                    .direction()
                    & 2
                    != 0
                {
                    2
                } else {
                    3
                }];
                self.follower_link_state_mut().set_direction_mask_a(mask);
            } else {
                let mut set_thingy = false;
                if self.game_state.dungeon.room_load.header_collision() == 0
                    && self.game_state.player.follower_link.has_auxiliary_state()
                    && self.game_state.player.tile_detection.slope_collision_bits() & 3 != 0
                {
                    set_thingy = true;
                }

                if r14 & 3 != 0 {
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    if self.game_state.player.follower_link.flag_moving() != 0
                        && self.game_state.player.tile_detection.spike_cactus_tiles() & 3 == 0
                        && self.game_state.player.follower_link.direction() & 3 != 0
                    {
                        self.swim_acceleration_mut().set_speed_active_flag(0, 0);
                        self.swim_acceleration_mut().set_mode(0, 0);
                        self.swim_acceleration_mut().set_acceleration(0, 0);
                        self.swim_acceleration_mut().set_max_speed(0, 0);
                    }
                    set_thingy = true;
                }

                if set_thingy {
                    self.follower_link_state_mut().set_pit_correction_active();
                    let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[self
                        .game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards()
                        as usize];
                    self.follower_link_state_mut().set_direction_mask_a(mask);
                }
            }
        }

        let direction = self.game_state.player.follower_link.direction();
        if direction & 0x0c != 0 && direction & 3 != 0 {
            self.follower_link_state_mut()
                .increment_orthogonal_direction_count();
            let last_direction_moved_towards = if direction & 2 != 0 { 2 } else { 3 };
            self.follower_link_state_mut()
                .set_last_direction_moved_towards(last_direction_moved_towards);
            self.tile_detect_movement_horizontal_slopes(last_direction_moved_towards as u16);

            let r14 = self.game_state.player.tile_detection.collision_bits();
            if r14 & 0x30 != 0
                && self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 != 0
                && ((((r14 & 0x30) >> 2) as u8) & self.game_state.player.follower_link.direction())
                    == 0
                && self.game_state.player.follower_link.direction() & 0x0c != 0
            {
                let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[if self
                    .game_state
                    .player
                    .follower_link
                    .direction()
                    & 8
                    != 0
                {
                    0
                } else {
                    1
                }];
                self.follower_link_state_mut().set_direction_mask_b(mask);
            } else {
                let mut set_thingy_b = false;
                if self.game_state.dungeon.room_load.header_collision() == 0
                    && self.game_state.player.follower_link.has_auxiliary_state()
                    && self.game_state.player.tile_detection.slope_collision_bits() & 3 != 0
                {
                    set_thingy_b = true;
                }

                if r14 & 3 != 0 {
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    if self.game_state.player.follower_link.flag_moving() != 0
                        && self.game_state.player.tile_detection.spike_cactus_tiles() & 3 == 0
                        && self.game_state.player.follower_link.direction() & 0x0c != 0
                    {
                        self.swim_acceleration_mut().set_speed_active_flag(2, 0);
                        self.swim_acceleration_mut().set_mode(2, 0);
                        self.swim_acceleration_mut().set_acceleration(2, 0);
                        self.swim_acceleration_mut().set_max_speed(2, 0);
                    }
                    set_thingy_b = true;
                }

                if set_thingy_b {
                    self.follower_link_state_mut().set_pit_correction_active();
                    let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[self
                        .game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards()
                        as usize];
                    self.follower_link_state_mut().set_direction_mask_b(mask);
                }
            }

            self.follower_link_state_mut().apply_direction_masks();
        }

        self.follower_link_state_mut()
            .force_direction_from_diag_tile_if_needed();
        self.follower_link_state_mut()
            .resolve_orthogonal_direction_count_from_facing();
    }

    pub(super) fn link_handle_velocity(&mut self) {
        if let Some(position_return) = self.link_handle_velocity_until_position_integrated() {
            self.complete_link_move_position_after_coordinates(position_return);
        }
    }

    pub(super) fn link_handle_velocity_until_position_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        self.link_move_position_until_coordinates_integrated()
    }

    /// `Link_HandleVelocity` through `Link_MovePosition`'s axis loop, stopping
    /// after the `pass` axis' subpixel store (the ROM's mid-loop interruption
    /// at route host 179586). `None` when the velocity handler took an early
    /// exit before `Link_MovePosition`.
    pub(super) fn link_handle_velocity_until_position_partial(
        &mut self,
        pass: u8,
    ) -> Option<LinkMovePositionPartialReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_partial_after_subpixel(pass))
    }

    /// `Link_HandleVelocity` through the low coordinate-byte store for
    /// `pass`, retaining the computed high byte and later movement suffix.
    pub(super) fn link_handle_velocity_until_position_after_coordinate_low(
        &mut self,
        pass: u8,
    ) -> Option<LinkMovePositionAfterCoordinateLowReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_after_coordinate_low(pass))
    }

    /// `Link_HandleVelocity` through both coordinate stores for `pass` in
    /// `Link_MovePosition`. `None` when the velocity handler returned before
    /// entering the movement loop.
    pub(super) fn link_handle_velocity_until_position_after_coordinates(
        &mut self,
        pass: u8,
    ) -> Option<LinkMovePositionAfterCoordinatesReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_after_coordinates(pass))
    }

    /// Run speed selection and the source-proven actual-velocity prefix,
    /// retaining whichever components have not yet been published.
    pub(super) fn link_handle_velocity_until_actual_checkpoint(
        &mut self,
        checkpoint: impl Into<LinkActualVelocityCheckpoint>,
    ) -> Option<LinkActualVelocityReturn> {
        let checkpoint = checkpoint.into();
        let completed_clear_stores = match checkpoint {
            LinkActualVelocityCheckpoint::Clearing { completed } => {
                assert!((1..4).contains(&completed));
                completed
            }
            _ => 4,
        };
        let speed_index = self.link_handle_velocity_before_velocity_clear()?;
        for store in 0..completed_clear_stores {
            self.follower_link_state_mut()
                .clear_velocity_selection_store(store);
        }
        if matches!(
            checkpoint,
            LinkActualVelocityCheckpoint::BeforeSelection
                | LinkActualVelocityCheckpoint::Clearing { .. }
        ) {
            return Some(LinkActualVelocityReturn {
                completed_clear_stores,
                pending_speed_index: Some(speed_index),
                pending_actual_x: None,
                pending_actual_y: None,
            });
        }
        let (direction, velocity) = self.link_handle_velocity_after_velocity_cleared(speed_index);
        let mut pending_actual_x = (direction & 0x03 != 0).then_some(if direction & 0x02 != 0 {
            0u8.wrapping_sub(velocity)
        } else {
            velocity
        });
        if let Some(actual_x) = pending_actual_x.filter(|_| {
            matches!(
                checkpoint,
                LinkActualVelocityCheckpoint::BeforeY | LinkActualVelocityCheckpoint::AfterBoth
            )
        }) {
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x);
            pending_actual_x = None;
        }
        let mut pending_actual_y = (direction & 0x0c != 0).then_some(if direction & 0x08 != 0 {
            0u8.wrapping_sub(velocity)
        } else {
            velocity
        });
        if checkpoint == LinkActualVelocityCheckpoint::AfterBoth {
            if let Some(actual_y) = pending_actual_y.take() {
                self.follower_link_state_mut()
                    .set_actual_y_velocity(actual_y);
            }
        }
        Some(LinkActualVelocityReturn {
            completed_clear_stores,
            pending_speed_index: None,
            pending_actual_x,
            pending_actual_y,
        })
    }

    /// Resume pending actual-velocity publication in X-then-Y order, airborne defaults,
    /// and `Link_MovePosition` without replaying speed/modifier selection.
    pub(super) fn link_handle_velocity_from_actual_checkpoint_until_position_integrated(
        &mut self,
        velocity_return: LinkActualVelocityReturn,
    ) -> Option<LinkMovePositionReturn> {
        for store in velocity_return.completed_clear_stores..4 {
            self.follower_link_state_mut()
                .clear_velocity_selection_store(store);
        }
        if let Some(speed_index) = velocity_return.pending_speed_index {
            let (direction, velocity) =
                self.link_handle_velocity_after_velocity_cleared(speed_index);
            self.follower_link_state_mut()
                .set_actual_velocity_from_direction(direction, velocity);
        }
        if let Some(actual_x) = velocity_return.pending_actual_x {
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x);
        }
        if let Some(actual_y) = velocity_return.pending_actual_y {
            self.follower_link_state_mut()
                .set_actual_y_velocity(actual_y);
        }
        self.follower_link_state_mut().prime_airborne_z_velocity();
        self.link_move_position_until_coordinates_integrated()
    }

    /// `Link_HandleVelocity` up to its `Link_MovePosition` call. `false` when
    /// the ROM returned earlier (safe-return/sand-drag, swim, or the
    /// all-blocked collision exit).
    fn link_handle_velocity_before_move_position(&mut self) -> bool {
        let Some((direction, velocity)) =
            self.link_handle_velocity_before_actual_velocity_resolution()
        else {
            return false;
        };
        self.follower_link_state_mut()
            .set_actual_velocity_from_direction(direction, velocity);
        self.follower_link_state_mut().prime_airborne_z_velocity();
        true
    }

    /// Run the stateful speed-selection prefix, stopping before either
    /// component of actual velocity is resolved.
    fn link_handle_velocity_before_actual_velocity_resolution(&mut self) -> Option<(u8, u8)> {
        let speed_index = self.link_handle_velocity_until_velocity_cleared()?;
        Some(self.link_handle_velocity_after_velocity_cleared(speed_index))
    }

    fn link_handle_velocity_until_velocity_cleared(&mut self) -> Option<u8> {
        let speed_index = self.link_handle_velocity_before_velocity_clear()?;
        self.follower_link_state_mut()
            .clear_actual_velocity_and_page_movement_deltas();
        Some(speed_index)
    }

    fn link_handle_velocity_before_velocity_clear(&mut self) -> Option<u8> {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();

        if (self.game_state.frame.submodule == 2 && self.game_state.frame.main_module == 14)
            || self
                .game_state
                .player
                .follower_link
                .is_prevented_from_moving()
        {
            self.store_link_safe_return_position(old_x, old_y);
            self.link_handle_velocity_and_sand_drag(old_x, old_y);
            return None;
        }

        if self.game_state.player.follower_link.handler_state() == 4 {
            self.handle_swim_stroke_and_subpixels();
            return None;
        }

        let speed_index = if self.game_state.player.follower_link.flag_moving() != 0 {
            if !self.game_state.player.follower_link.is_running() {
                self.handle_swim_stroke_and_subpixels();
                return None;
            }
            24
        } else {
            if self.game_state.player.follower_link.is_running() {
                self.follower_link_state_mut().clear_speed_modifier();
                assert!(self.game_state.player.follower_link.dash_counter() >= 32);
            }
            if (self
                .game_state
                .player
                .tile_detection
                .tile_collision_bits_primary()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .tile_collision_bits_secondary())
                == 0x0f
            {
                return None;
            }
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                != 0
            {
                match self.game_state.player.follower_link.speed_setting() {
                    16 => 22,
                    12 => 14,
                    _ => 12,
                }
            } else {
                self.game_state.player.follower_link.speed_setting()
            }
        };

        Some(speed_index)
    }

    fn link_handle_velocity_after_velocity_cleared(&mut self, mut speed_index: u8) -> (u8, u8) {
        let direction = self.game_state.player.follower_link.direction();
        if (direction & 0x0c) != 0 && (direction & 0x03) != 0 {
            speed_index = speed_index.wrapping_add(1);
        }

        if self.game_state.player.follower_link.is_near_pit() {
            if self.game_state.player.follower_link.near_pit_state_is(3) {
                self.follower_link_state_mut()
                    .increase_near_pit_speed_modifier();
            }
        } else if self.game_state.player.follower_link.speed_modifier() != 0 {
            speed_index =
                if self.game_state.frame.submodule == 8 || self.game_state.frame.submodule == 16 {
                    10
                } else {
                    2
                };
            let speed_modifier = self.game_state.player.follower_link.speed_modifier();
            if speed_modifier != 1 && speed_modifier < 16 {
                self.follower_link_state_mut().advance_dash_deceleration();
                speed_index = 26;
            } else if speed_modifier != 1 {
                self.follower_link_state_mut().clear_speed_modifier();
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }

        let velocity = self
            .game_state
            .player
            .follower_link
            .speed_modifier()
            .wrapping_add(LINK_HANDLE_VELOCITY_SPEED_MOD[speed_index as usize]);
        (self.game_state.player.follower_link.direction(), velocity)
    }

    /// `Link_MovePosition` up to and including the `pass` axis' subpixel
    /// store: the ROM's loop runs z (airborne only), then x, then y; earlier
    /// passes complete, the `pass` axis owes its coordinate delta.
    fn link_move_position_partial_after_subpixel(
        &mut self,
        pass: u8,
    ) -> LinkMovePositionPartialReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for candidate in self.link_move_position_passes() {
            let velocity = self.link_move_position_pass_velocity(candidate);
            if candidate == pass {
                let pending_pixel_delta = self
                    .follower_link_state_mut()
                    .move_axis_subpixel_only_by_velocity(candidate, velocity);
                return LinkMovePositionPartialReturn {
                    old_x: x,
                    old_y: y,
                    partial: LinkMovePositionPartial {
                        pass,
                        pending_pixel_delta,
                    },
                };
            }
            self.link_move_position_full_pass(candidate, velocity);
        }
        panic!("Link_MovePosition interruption named pass {pass} outside the ROM's axis loop");
    }

    /// `Link_MovePosition` through both coordinate stores for `pass`.
    fn link_move_position_after_coordinates(
        &mut self,
        pass: u8,
    ) -> LinkMovePositionAfterCoordinatesReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for candidate in self.link_move_position_passes() {
            let velocity = self.link_move_position_pass_velocity(candidate);
            self.link_move_position_full_pass(candidate, velocity);
            if candidate == pass {
                return LinkMovePositionAfterCoordinatesReturn {
                    old_x: x,
                    old_y: y,
                    pass,
                };
            }
        }
        panic!("Link_MovePosition interruption named pass {pass} outside the ROM's axis loop");
    }

    /// `Link_MovePosition` through the current axis' low coordinate-byte
    /// store, preserving its high byte and every later axis.
    fn link_move_position_after_coordinate_low(
        &mut self,
        pass: u8,
    ) -> LinkMovePositionAfterCoordinateLowReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for candidate in self.link_move_position_passes() {
            let velocity = self.link_move_position_pass_velocity(candidate);
            if candidate == pass {
                let pending_pixel_delta = self
                    .follower_link_state_mut()
                    .move_axis_subpixel_only_by_velocity(candidate, velocity);
                let pending_coordinate_high = self
                    .follower_link_state_mut()
                    .apply_axis_pixel_delta_low(candidate, pending_pixel_delta);
                return LinkMovePositionAfterCoordinateLowReturn {
                    old_x: x,
                    old_y: y,
                    pass,
                    pending_coordinate_high,
                };
            }
            self.link_move_position_full_pass(candidate, velocity);
        }
        panic!("Link_MovePosition interruption named pass {pass} outside the ROM's axis loop");
    }

    /// Resume `Link_MovePosition` after `link_move_position_partial_after_subpixel`.
    pub(super) fn complete_link_move_position_from_partial(
        &mut self,
        position_return: LinkMovePositionPartialReturn,
    ) {
        let LinkMovePositionPartial {
            pass,
            pending_pixel_delta,
        } = position_return.partial;
        self.follower_link_state_mut()
            .apply_axis_pixel_delta(pass, pending_pixel_delta);
        let passes = self.link_move_position_passes();
        let resume_at = passes
            .iter()
            .position(|candidate| *candidate == pass)
            .expect("a suspended Link_MovePosition pass must belong to the ROM's axis loop");
        for candidate in passes[resume_at + 1..].iter().copied() {
            let velocity = self.link_move_position_pass_velocity(candidate);
            self.link_move_position_full_pass(candidate, velocity);
        }
        self.complete_link_move_position_after_coordinates(LinkMovePositionReturn {
            old_x: position_return.old_x,
            old_y: position_return.old_y,
        });
    }

    /// Resume after the current axis' low coordinate byte was published.
    pub(super) fn complete_link_move_position_from_after_coordinate_low(
        &mut self,
        position_return: LinkMovePositionAfterCoordinateLowReturn,
    ) {
        self.follower_link_state_mut().apply_axis_coordinate_high(
            position_return.pass,
            position_return.pending_coordinate_high,
        );
        self.complete_link_move_position_from_after_coordinates(
            LinkMovePositionAfterCoordinatesReturn {
                old_x: position_return.old_x,
                old_y: position_return.old_y,
                pass: position_return.pass,
            },
        );
    }

    /// Resume after `link_move_position_after_coordinates`, beginning with
    /// the following axis and then completing the ordinary movement tail.
    pub(super) fn complete_link_move_position_from_after_coordinates(
        &mut self,
        position_return: LinkMovePositionAfterCoordinatesReturn,
    ) {
        let passes = self.link_move_position_passes();
        let resume_at = passes
            .iter()
            .position(|candidate| *candidate == position_return.pass)
            .expect("a suspended Link_MovePosition pass must belong to the ROM's axis loop");
        for candidate in passes[resume_at + 1..].iter().copied() {
            let velocity = self.link_move_position_pass_velocity(candidate);
            self.link_move_position_full_pass(candidate, velocity);
        }
        self.complete_link_move_position_after_coordinates(LinkMovePositionReturn {
            old_x: position_return.old_x,
            old_y: position_return.old_y,
        });
    }

    /// The ROM's `Link_MovePosition` loop passes (X register values) in order.
    fn link_move_position_passes(&self) -> Vec<u8> {
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            vec![4, 2, 0]
        } else {
            vec![2, 0]
        }
    }

    fn link_move_position_pass_velocity(&self, pass: u8) -> u8 {
        match pass {
            4 => self.game_state.player.follower_link.actual_z_velocity(),
            2 => self.game_state.player.follower_link.actual_x_velocity(),
            _ => self.game_state.player.follower_link.actual_y_velocity(),
        }
    }

    fn link_move_position_full_pass(&mut self, pass: u8, velocity: u8) {
        match pass {
            4 => {
                self.follower_link_state_mut().move_z_by_velocity(velocity);
            }
            2 => {
                self.follower_link_state_mut().move_x_by_velocity(velocity);
            }
            _ => {
                self.follower_link_state_mut().move_y_by_velocity(velocity);
            }
        }
    }

    pub(super) fn handle_swim_stroke_and_subpixels(&mut self) {
        self.follower_link_state_mut().clear_actual_velocity_xy();

        let mut stroke = [0u16; 2];

        for i in (0..=1).rev() {
            let offset = i * 2;
            let stroke_timer = self
                .game_state
                .player
                .follower_link
                .swim_stroke_frame_counter(offset)
                .wrapping_sub(1);
            self.follower_link_state_mut()
                .set_swim_stroke_frame_counter(offset, stroke_timer);
            if (stroke_timer as i16) < 0 {
                self.follower_link_state_mut()
                    .set_swim_stroke_frame_counter(offset, 0);
                self.swim_acceleration_mut().set_mode(offset, 1);
            }

            let mut table_index = self.game_state.player.swim_acceleration.mode(offset);
            if self.game_state.player.follower_link.flag_moving() != 0 {
                table_index = table_index.wrapping_add(
                    u16::from(self.game_state.player.follower_link.flag_moving()) * 4,
                );
            }

            let delta = HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_ACCELERATION_DELTAS
                [table_index as usize] as i16 as u16;
            let mut sum = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset)
                .wrapping_add(delta);
            if (sum as i16) <= 0 {
                self.follower_link_state_mut().mask_direction(
                    HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_AXIS_DIRECTION_CLEAR_MASKS[i],
                );
                self.follower_link_state_mut()
                    .set_last_direction_from_current_direction();
                if self.game_state.player.swim_acceleration.mode(offset) == 2 {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                    self.swim_acceleration_mut().set_acceleration(offset, 2);
                } else {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    self.swim_acceleration_mut().set_max_speed(offset, 0);
                    self.swim_acceleration_mut().set_acceleration(offset, 0);
                }
            } else {
                let dir_index = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration_direction(offset) as usize
                    + i * 2;
                self.follower_link_state_mut().add_direction_flags(
                    HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_DIRECTION_BITS_BY_AXIS[dir_index],
                );
                let max_sum = self.game_state.player.swim_acceleration.max_speed(offset);
                if sum >= max_sum {
                    sum = max_sum;
                }
                self.swim_acceleration_mut().set_acceleration(offset, sum);
            }

            stroke[i] = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset);
            if self.game_state.player.follower_link.has_swim_axis_drag() {
                stroke[i] = stroke[i].wrapping_sub(stroke[i] >> 2);
            }
            if self
                .game_state
                .player
                .swim_acceleration
                .acceleration_direction(offset)
                == 0
            {
                stroke[i] = 0u16.wrapping_sub(stroke[i]);
            }
        }

        self.player_something_with_velocity_tired_or_swim(stroke[1], stroke[0]);
    }

    pub(super) fn link_receive_item(&mut self, item: u8, chest_position: u16) {
        let _ = self.link_receive_item_from(item, chest_position, ItemReceiptCaller::AtomicCaller);
    }

    pub(super) fn link_receive_item_from(
        &mut self,
        item: u8,
        chest_position: u16,
        caller: ItemReceiptCaller,
    ) -> GameCallStatus {
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_blink_countdown();
            self.follower_link_state_mut().clear_state_bits();
        }
        self.follower_link_state_mut().set_receive_item_index(item);
        if item == 0x3e {
            self.ancilla_sfx3_near(0x2e);
        }
        self.follower_link_state_mut().set_item_holding_timer(0x60);
        if self.game_state.player.follower_link.item_receipt_method() == 0
            || self.game_state.player.follower_link.item_receipt_method() == 3
        {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_direction_lock();
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_handler_state(21);
            self.follower_link_state_mut().set_item_hold_pose(1);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            if item == 0x20 {
                self.follower_link_state_mut().set_item_hold_pose(2);
            }
        }
        let call_status = self.ancilla_add_item_receipt_from(0x22, 4, chest_position, caller);
        if call_status.is_suspended() {
            return call_status;
        }
        self.complete_link_receive_item(item);
        GameCallStatus::Returned
    }

    pub(super) fn complete_link_receive_item(&mut self, item: u8) {
        if item != 0x20 && item != 0x37 && item != 0x38 && item != 0x39 {
            self.hud_refresh_icon();
        }
        self.link_cancel_dash();
    }

    pub(super) fn handle_nudging(&mut self, arg_r0: i8) {
        let last_direction_moved_towards = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards();
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
        let o = (((if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            0
        } else {
            2
        }) + p)
            >> 1) as usize;

        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let y0 = link_y.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_Y0_OFFSETS[o] as u16) & mask;
        let x0 = (link_x.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_X0_OFFSETS[o] as u16) & mask) >> 3;
        let y1 = link_y.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_Y1_OFFSETS[o] as u16) & mask;
        let x1 = (link_x.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_X1_OFFSETS[o] as u16) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let blocked = (self.game_state.player.tile_detection.collision_bits()
            | self.game_state.player.tile_detection.horizontal_ledge() as u16)
            & 3
            != 0
            || (self.game_state.player.tile_detection.vertical_ledge()
                | self.game_state.player.tile_detection.diagonal_ledge_tiles())
                & 0x33
                != 0;
        if blocked {
            if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 2
                != 0
            {
                let y = self.game_state.player.follower_link.y();
                self.follower_link_state_mut()
                    .set_y(y.wrapping_sub(arg_r0 as i16 as u16));
            } else {
                let x = self.game_state.player.follower_link.x();
                self.follower_link_state_mut()
                    .set_x(x.wrapping_sub(arg_r0 as i16 as u16));
            }
        }
    }

    pub(super) fn handle_pushing_bonking_snaps_y(&mut self) {
        let r14 = self.game_state.player.tile_detection.collision_bits();
        if r14 & 7 == 0 {
            if self.game_state.player.follower_link.is_on_lower_level() {
                return;
            }
            self.follower_link_state_mut().and_defense_flags(!9);
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        let mut used_swim_axis_reprobe = false;
        if self.game_state.player.follower_link.handler_state() == 4 {
            if self.game_state.dungeon.moving_floor.floor_y_velocity_low() == 0 {
                self.reset_all_acceleration();
            }
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                != 0
            {
                self.link_add_in_velocity_y_falling();
                used_swim_axis_reprobe = true;
            }
        }

        if r14 & 2 != 0 || (r14 & 5) == 5 {
            self.replay_trace_drag_tail("snaps-y-before-first-bonk");
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            self.tile_detect_position_mut().set_collision_bits(bak);
            self.replay_trace_drag_tail("snaps-y-after-first-bonk");
        }

        self.follower_link_state_mut().set_pit_correction_active();

        if !used_swim_axis_reprobe {
            if r14 & 2 == 2 {
                self.replay_trace_drag_tail("snaps-y-before-add-vel");
                self.link_add_in_velocity_y_falling();
                self.replay_trace_drag_tail("snaps-y-after-add-vel");
            } else {
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 1
                {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
                self.link_add_in_velocity_y_falling();
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 2
                {
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
            let y_vel = self.game_state.player.follower_link.y_velocity();
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
            if self.game_state.player.follower_link.x() & 7 != 0 {
                let x = self.game_state.player.follower_link.x();
                self.follower_link_state_mut()
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
        let r14 = self.game_state.player.tile_detection.collision_bits();
        if r14 & 7 == 0 {
            if self.game_state.player.follower_link.is_on_lower_level() {
                return;
            }
            self.follower_link_state_mut().and_defense_flags(!9);
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        if self.game_state.player.follower_link.handler_state() == 4
            && self.game_state.dungeon.moving_floor.floor_x_velocity_low() == 0
        {
            self.reset_all_acceleration();
        }

        if r14 & 2 != 0 {
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            self.tile_detect_position_mut().set_collision_bits(bak);
        }

        self.follower_link_state_mut().set_pit_correction_active();

        if r14 & 7 == 7 {
            self.snap_on_x();
        } else {
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 2
            {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
            self.snap_on_x();
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 1
            {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
        }

        if (r14 & 5) == 5 {
            self.link_bonk_and_smash();
            self.repel_dash();
        } else if r14 & 2 == 0 {
            let x_vel = self.game_state.player.follower_link.x_velocity();
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
            if self.game_state.player.follower_link.y() & 7 != 0 {
                let y = self.game_state.player.follower_link.y();
                self.follower_link_state_mut()
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
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            .wrapping_mul(2)
            != self.game_state.player.follower_link.facing()
        {
            return false;
        }

        self.replay_trace_drag_tail("tail-match-entry");
        let drag_bits = (self.game_state.player.follower_link.tile_coll_flag() & 1) << 1;
        self.follower_link_state_mut().or_defense_flags(drag_bits);
        self.replay_trace_drag_tail("tail-after-lowbit");
        let push_fatigue_timer = self
            .follower_link_state_mut()
            .decrement_push_fatigue_timer();
        if self.game_state.player.follower_link.button_b_frames() == 0
            && !(push_fatigue_timer as i8).is_negative()
        {
            self.replay_trace_drag_tail("tail-return-timer");
            return true;
        }

        let tile_coll = self.game_state.player.follower_link.tile_coll_flag();
        let drag_bits = if self.game_state.player.tile_detection.misc_tiles() & 0x20 != 0 {
            tile_coll << 3
        } else {
            tile_coll
        };
        self.follower_link_state_mut().or_defense_flags(drag_bits);
        self.replay_trace_drag_tail("tail-after-fullbits");
        false
    }

    fn handle_pushing_bonking_snaps_return(&mut self) {
        self.follower_link_state_mut().reset_push_fatigue_timer();
        self.follower_link_state_mut().and_defense_flags(!2);
    }

    pub(super) fn push_block_attempt_to_push_the_block(&self, what: u8, x: u16, y: u16) -> bool {
        let idx = what as usize * 4
            + self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards() as usize;
        let mask = self.game_state.player.tile_detection.location_calc_mask();

        let x0 = (x
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_X_OFFSETS_0[idx] as i16 as u16)
            & mask)
            >> 3;
        let y0 = y
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_Y_OFFSETS_0[idx] as i16 as u16)
            & mask;
        let xt = self.push_block_get_target_tile_flag(x0, y0);
        if push_block_target_is_blocked(xt) {
            return true;
        }

        let x1 = (x
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_X_OFFSETS_1[idx] as i16 as u16)
            & mask)
            >> 3;
        let y1 = y
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_Y_OFFSETS_1[idx] as i16 as u16)
            & mask;
        push_block_target_is_blocked(self.push_block_get_target_tile_flag(x1, y1))
    }

    pub(super) fn link_find_valid_landing_tile_north(&mut self) {
        let y_coord_bak = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(y_coord_bak);
        loop {
            let y = self.game_state.player.follower_link.y().wrapping_sub(16);
            self.follower_link_state_mut().set_y(y);
            self.tile_detect_movement_y(
                self.game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards() as u16,
            );
            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath()
                | self.game_state.player.tile_detection.thick_grass()
                | self.game_state.player.tile_detection.deepwater();
            if terrain & 7 == 7 {
                break;
            }
        }

        if self.game_state.player.tile_detection.deepwater() & 7 != 0 {
            self.follower_link_state_mut().set_auxiliary_state(1);
            self.follower_link_state_mut().clear_electrocute_on_touch();
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
        }

        let y = self.game_state.player.follower_link.y().wrapping_sub(16);
        self.follower_link_state_mut().set_y(y);
        let diff = self
            .follower_link_state_mut()
            .set_hop_origin_delta_from_y(y);
        self.follower_link_state_mut().set_y(y_coord_bak);
        let o = ((diff as u8) >> 3) as usize;
        let dy = LINK_FIND_VALID_LANDING_TILE_NORTH_Y_DELTAS[o];
        let actual_y_velocity = if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            != 0
        {
            dy
        } else {
            0u8.wrapping_sub(dy)
        };
        self.follower_link_state_mut()
            .set_actual_velocity_xy(0, actual_y_velocity);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(LINK_FIND_VALID_LANDING_TILE_NORTH_Z_DELTAS[o]);
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut()
            .set_incapacitated_timer(LINK_FIND_VALID_LANDING_TILE_NORTH_TIMERS[o]);
        self.follower_link_state_mut().set_auxiliary_state(2);
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_handler_state(6);
    }

    pub(super) fn link_find_valid_landing_tile_diagonal_north(&mut self) {
        let y_safe = self.game_state.player.follower_link.safe_return_y_low();
        let x_bak = self.game_state.player.follower_link.x();
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards();

        let actual_x_velocity = if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            != 2
        {
            1
        } else {
            0xff
        };
        self.follower_link_state_mut()
            .set_actual_x_velocity(actual_x_velocity);
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(0);
        self.link_hop_find_landing_spot_diagonally_down();

        self.follower_link_state_mut().set_x(x_bak);
        self.follower_link_state_mut().set_safe_return_y_low(y_safe);

        let diff = self
            .game_state
            .player
            .follower_link
            .hop_origin_coord()
            .wrapping_sub(self.game_state.player.follower_link.y());
        let o = (diff >> 3) as usize;
        self.follower_link_state_mut().restore_y_from_hop_origin();

        let actual_y_velocity =
            0u8.wrapping_sub(LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_Y_DELTAS[o]);
        let actual_x_velocity = if dir != 2 {
            LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_X_DELTAS[o]
        } else {
            0u8.wrapping_sub(LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_X_DELTAS[o])
        };
        self.follower_link_state_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_Z_DELTAS[o],
            );
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().clear_z_mirror_word_low();
        self.follower_link_state_mut().set_auxiliary_state(2);
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_handler_state(13);
    }

    pub(super) fn tile_detect_main_handler(&mut self, item: u8) {
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let probe_base = if item == 8 {
            let spin = self
                .game_state
                .player
                .follower_link
                .state_for_spin_attack()
                .wrapping_sub(2);
            if spin >= 8 {
                return;
            }
            TILE_DETECT_MAIN_HANDLER_SPIN_OFFSETS[spin as usize] as u16 + 0x40
        } else {
            item as u16 * 8 + self.game_state.player.follower_link.facing() as u16
        };
        let offset = probe_base >> 1;

        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        let x = (link_x
            .wrapping_add(TILE_DETECT_MAIN_HANDLER_X_OFFSETS[offset as usize] as i16 as u16)
            & mask)
            >> 3;
        let y = link_y
            .wrapping_add(TILE_DETECT_MAIN_HANDLER_Y_OFFSETS[offset as usize] as i16 as u16)
            & mask;

        if matches!(item, 1 | 2 | 3 | 6 | 7 | 8) {
            self.tile_behavior_handle_item_and_execute(x, y);
            return;
        }

        self.tile_detection_execute(x, y, 1);
        if item == 5 {
            return;
        }

        if self.game_state.player.tile_detection.thick_grass() & 0x10 != 0 {
            let tx = self.game_state.player.follower_link.x() & 0x0f;
            let ty = self.game_state.player.follower_link.y().wrapping_add(8) & 0x0f;
            if !(4..11).contains(&ty)
                && !(4..12).contains(&tx)
                && self.game_state.player.follower_link.blink_countdown() == 0
                && !self.game_state.player.follower_link.has_auxiliary_state()
            {
                if self.game_state.world.location.is_indoors() {
                    self.Dungeon_FlagRoomData_Quadrants();
                    self.ancilla_sfx2_near(0x33);
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.set_submodule(21);
                    let prev_room = self.game_state.world.location.dungeon_room_index();
                    self.dungeon_room_tracking_mut()
                        .set_room_index_prev(prev_room);
                    let room = self.game_state.dungeon.header.travel_destination(0);
                    self.set_dungeon_room_index(room);
                    self.handle_layer_of_destination();
                } else if !self.game_state.player.follower_link.whirlpool_triggered() {
                    self.do_sword_interaction_with_tiles_mirror();
                }
            }
        } else {
            self.follower_link_state_mut().clear_whirlpool_trigger();
            if self.game_state.player.tile_detection.thick_grass() & 1 != 0 {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(2);
                if !self.link_permission_for_slosh_sounds()
                    && !self.game_state.player.follower_link.has_auxiliary_state()
                {
                    self.ancilla_sfx2_near(26);
                }
                return;
            }

            if self.game_state.player.tile_detection.shallow_water() & 1 != 0 {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(1);
                if self.game_state.world.location.is_outdoors()
                    && self.game_state.player.follower_link.is_in_deep_water()
                    && !self.game_state.player.follower_link.is_bunny_mirror()
                {
                    if self.game_state.player.follower_link.has_flippers() {
                        self.follower_link_state_mut().clear_deep_water_state();
                        self.follower_link_state_mut()
                            .set_last_direction_from_swim_flags();
                        self.follower_link_state_mut().clear_handler_state();
                    }
                } else if !self.link_permission_for_slosh_sounds() {
                    if self.game_state.world.location.overworld_screen_index() == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if !self.game_state.player.follower_link.has_auxiliary_state() {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }

            if self.game_state.world.location.is_outdoors()
                && !self.game_state.player.follower_link.is_in_deep_water()
                && self.game_state.player.tile_detection.deepwater() & 1 != 0
            {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(1);
                if !self.link_permission_for_slosh_sounds() {
                    if self.game_state.world.location.overworld_screen_index() == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if !self.game_state.player.follower_link.has_auxiliary_state() {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if self
            .game_state
            .player
            .tile_detection
            .spike_floor_and_triggers()
            & 1
            != 0
        {
            self.follower_link_state_mut()
                .set_item_pickup_in_progress(1);
            return;
        }
        self.follower_link_state_mut()
            .set_item_pickup_in_progress(0);

        if self
            .game_state
            .player
            .tile_detection
            .spike_floor_and_triggers()
            & 0x10
            != 0
        {
            self.follower_link_state_mut().clear_given_damage();
            if !self.game_state.player.follower_link.is_cape_active()
                && !self.search_for_byrna_spark()
                && self.game_state.player.follower_link.blink_countdown() == 0
            {
                self.follower_link_state_mut()
                    .clear_transform_poof_need_and_temp_bunny_timer();
                if self.game_state.player.follower_link.has_moon_pearl() {
                    self.follower_link_state_mut()
                        .clear_bunny_transform_after_moon_pearl();
                }
                self.follower_link_state_mut().set_given_damage(8);
                self.link_cancel_dash();
                return;
            }
        }

        if self.game_state.player.tile_detection.icy_floor() & 0x11 != 0 {
            if self.game_state.player.follower_link.flag_moving() != 0 {
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    != 0
                {
                    self.follower_link_state_mut()
                        .set_last_direction_from_swim_flags();
                }
            } else {
                if self.game_state.player.follower_link.direction() & 0x0c != 0 {
                    self.swim_acceleration_mut().set_acceleration(0, 0x0180);
                }
                if self.game_state.player.follower_link.direction() & 3 != 0 {
                    self.swim_acceleration_mut().set_acceleration(0, 0x0180);
                }
                let flag_moving = if self.game_state.player.tile_detection.icy_floor() & 1 != 0 {
                    1
                } else {
                    2
                };
                self.follower_link_state_mut().set_flag_moving(flag_moving);
                self.follower_link_state_mut()
                    .set_swim_flags_from_last_direction();
                self.link_reset_swimming_state();
            }
        } else {
            if self.game_state.player.follower_link.handler_state() != 4 {
                if self.game_state.player.follower_link.flag_moving() != 0 {
                    self.follower_link_state_mut()
                        .set_last_direction_from_swim_flags();
                }
                self.link_reset_swimming_state();
            }
            self.follower_link_state_mut().set_flag_moving(0);
        }

        if self.game_state.player.tile_detection.spike_cactus_tiles() & 0x10 != 0
            && self.game_state.player.follower_link.blink_countdown() == 0
        {
            self.follower_link_state_mut().set_blink_countdown(58);
        }
    }

    pub(super) fn start_movement_collision_checks_y(&mut self) {
        self.replay_trace_submodule("start-y-entry");
        if self.game_state.player.follower_link.y_velocity() == 0 {
            return;
        }
        let last_direction_moved_towards =
            if self.game_state.player.follower_link.doorway_state() == 1 {
                if self.game_state.player.follower_link.y_low() < 0x80 {
                    0
                } else {
                    1
                }
            } else if self
                .game_state
                .player
                .follower_link
                .y_velocity_signed()
                .is_negative()
            {
                0
            } else {
                1
            };
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(last_direction_moved_towards);
        self.tile_detect_movement_y(last_direction_moved_towards as u16);
        self.replay_trace_submodule("start-y-after-tiledetect");
        if self.game_state.world.location.is_indoors() {
            self.start_movement_collision_checks_y_handle_indoors();
        } else {
            self.start_movement_collision_checks_y_handle_outdoors();
        }
    }

    pub(super) fn start_movement_collision_checks_x(&mut self) {
        if self.game_state.player.follower_link.x_velocity() == 0 {
            return;
        }
        let last_direction_moved_towards =
            if self.game_state.player.follower_link.doorway_state() == 2 {
                if self.game_state.player.follower_link.x_low() < 0x80 {
                    2
                } else {
                    3
                }
            } else if self
                .game_state
                .player
                .follower_link
                .x_velocity_signed()
                .is_negative()
            {
                2
            } else {
                3
            };
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(last_direction_moved_towards);
        self.tile_detect_movement_x(last_direction_moved_towards as u16);
        if self.game_state.world.location.is_indoors() {
            self.start_movement_collision_checks_x_handle_indoors();
        } else {
            self.start_movement_collision_checks_x_handle_outdoors();
        }
    }

    pub(super) fn start_movement_collision_checks_y_handle_indoors(&mut self) {
        let mut r14 = self.game_state.player.tile_detection.collision_bits();
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
        {
            r14 |= r14 >> 4;
            self.tile_detect_position_mut().set_collision_bits(r14);
        } else {
            if self.game_state.player.follower_link.doorway_state() == 2 {
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 0
                {
                    if self.game_state.dungeon.room_load.header_collision() != 3
                        || !self.game_state.player.follower_link.is_on_lower_level()
                    {
                        self.link_add_in_velocity_y();
                        self.change_axis_of_perpendicular_door_movement_y();
                        return;
                    }
                } else if self.game_state.player.tile_detection.door_direction_flags() != 0 {
                    self.link_add_in_velocity_y();
                    self.finish_indoor_y_collision();
                    return;
                }
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if self
                        .game_state
                        .player
                        .follower_link
                        .y_velocity_signed()
                        .is_negative()
                    {
                        8
                    } else {
                        4
                    };
                    self.follower_link_state_mut()
                        .set_force_move_any_direction(force_move);
                }

                self.follower_link_state_mut().set_doorway_state(1);
                self.follower_link_state_mut().clear_conveyor_belt_state();
                if r14 & 0x70 != 0x70 {
                    if r14 & 5 != 0 {
                        self.follower_link_state_mut()
                            .clear_moving_against_diag_tile();
                        self.link_add_in_velocity_y_falling();
                        self.calculate_snap_scratch_y();
                        self.follower_link_state_mut().clear_doorway_state();
                        if r14 & 0x20 != 0
                            && r14 & 1 == 0
                            && self.game_state.player.follower_link.x() & 7 == 1
                        {
                            let x = self.game_state.player.follower_link.x() & !7;
                            self.follower_link_state_mut().set_x(x);
                        }
                        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                            self.follower_link_state_mut().clear_direction_lock_bits(2);
                        }
                        return;
                    }
                    if r14 & 0x20 != 0 {
                        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                            self.follower_link_state_mut().clear_direction_lock_bits(2);
                        }
                        return;
                    }
                } else {
                    if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                        self.follower_link_state_mut().clear_direction_lock_bits(2);
                    }
                    return;
                }
            }
        }

        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
        }
        self.finish_indoor_y_collision();
    }

    pub(super) fn start_movement_collision_checks_x_handle_indoors(&mut self) {
        let mut r14 = self.game_state.player.tile_detection.collision_bits();
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
        {
            r14 |= r14 >> 4;
            self.tile_detect_position_mut().set_collision_bits(r14);
        } else {
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
            {
                self.follower_link_state_mut().clear_speed_modifier();
            }

            if self.game_state.player.follower_link.doorway_state() == 1
                && self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 0
                && (self.game_state.dungeon.room_load.header_collision() != 3
                    || !self.game_state.player.follower_link.is_on_lower_level())
            {
                self.snap_on_x();
                let spd = self.change_axis_of_perpendicular_door_movement_x();
                self.handle_nudging_in_a_door(spd);
                return;
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if self
                        .game_state
                        .player
                        .follower_link
                        .x_velocity_signed()
                        .is_negative()
                    {
                        2
                    } else {
                        1
                    };
                    self.follower_link_state_mut()
                        .set_force_move_any_direction(force_move);
                }

                self.follower_link_state_mut().set_doorway_state(2);
                self.follower_link_state_mut().clear_conveyor_belt_state();
                if r14 & 0x70 != 0x70 {
                    if r14 & 7 != 0 {
                        self.follower_link_state_mut()
                            .clear_moving_against_diag_tile();
                        self.follower_link_state_mut().clear_doorway_state();
                        self.snap_on_x();
                        self.calculate_snap_scratch_x();
                        return;
                    }
                    self.follower_link_state_mut().clear_direction_lock_bits(2);
                    return;
                }

                self.follower_link_state_mut().clear_direction_lock_bits(2);
                return;
            }
        }

        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_direction_lock_bits(2);
            self.follower_link_state_mut().clear_doorway_state();
            self.set_room_transitioning_flags(0);
            self.follower_link_state_mut()
                .set_force_move_any_direction(0);
        }

        if self.game_state.player.tile_detection.collision_bits() & 2 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            self.follower_link_state_mut().clear_conveyor_belt_state();
            self.flag_moving_into_slopes_x();
            if self
                .game_state
                .player
                .follower_link
                .moving_against_diag_tile()
                & 0x0f
                != 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        self.finish_indoor_collision_common(false);
    }

    pub(super) fn start_movement_collision_checks_y_handle_outdoors(&mut self) {
        self.replay_trace_submodule("outdoor-y-entry");
        self.replay_trace_drag_tail("outdoor-y-drag-entry");
        self.follower_link_state_mut().resolve_dash_speed_setting();
        self.replay_trace_drag_tail("outdoor-y-after-speed-setting");

        if self.game_state.player.tile_detection.pit_tile() & 5 != 0
            && self.game_state.player.tile_detection.collision_bits() & 2 == 0
        {
            self.start_falling_into_hole();
            return;
        }

        let liftable_primary = if self.game_state.player.tile_detection.read_something() & 2 != 0 {
            self.game_state.player.tile_detection.liftable_tile_index() >> 1
        } else {
            0
        };
        self.tile_detect_position_mut()
            .set_liftable_action_index_primary(liftable_primary);

        if self.game_state.player.tile_detection.deepwater() & 2 != 0
            && !self.game_state.player.follower_link.is_in_deep_water()
            && !self.game_state.player.follower_link.has_auxiliary_state()
        {
            self.link_reset_sword_and_item_usage();
            self.link_cancel_dash();
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_reset_swimming_state();
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 1
                && {
                    self.link_force_unequip_cape_quietly();
                    self.game_state.player.follower_link.has_flippers()
                }
            {
                if !self.game_state.player.follower_link.is_bunny_mirror() {
                    self.follower_link_state_mut().set_handler_state(4);
                }
            } else {
                self.link_ledge_hop_sfx();
                self.restore_link_safe_return_position();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.link_hop_in_or_out_of_water_y();
            }
        }

        if self.game_state.player.follower_link.is_in_deep_water() {
            if self.game_state.player.tile_detection.vertical_ledge() & 7 != 0 {
                let r14 = (self.game_state.player.tile_detection.vertical_ledge() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            if (self.game_state.player.tile_detection.stair_tile() & 7) == 7
                || self.game_state.player.tile_detection.normal_tiles() & 7 == 7
            {
                self.link_cancel_dash();
                self.follower_link_state_mut().clear_deep_water_state();
                if self.game_state.player.follower_link.auxiliary_state() == 0 {
                    self.follower_link_state_mut()
                        .set_last_direction_from_swim_flags();
                    self.follower_link_state_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.ancilla_add_splash(0x15, 0);
                    self.link_hop_in_or_out_of_water_y();
                    return;
                }
            }
        }

        if self.game_state.player.tile_detection.horizontal_ledge() & 2 != 0
            || self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x22 != 0
        {
            self.tile_detect_position_mut().set_collision_bits(7);
            self.handle_pushing_bonking_snaps_y();
            return;
        }

        if self.game_state.player.tile_detection.vertical_ledge() & 0x70 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.set_allow_scroll_z(1);
            self.follower_link_state_mut().set_handler_state(11);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().set_z_mirror(0xffff);
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            let zvel = if self.game_state.player.follower_link.is_in_deep_water() {
                14
            } else {
                20
            };
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(zvel);
            let auxiliary_state = if self.game_state.player.follower_link.is_in_deep_water() {
                4
            } else {
                2
            };
            self.follower_link_state_mut()
                .set_auxiliary_state(auxiliary_state);
            return;
        }

        if self.game_state.player.tile_detection.vertical_ledge() & 7 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_cancel_dash();
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_find_valid_landing_tile_north();
            return;
        }

        if !self.game_state.player.follower_link.is_in_deep_water() {
            if self
                .game_state
                .player
                .tile_detection
                .ledges_down_leftright()
                & 7
                != 0
                && self.game_state.player.tile_detection.vertical_ledge() & 0x77 == 0
            {
                let xand = if self.game_state.player.tile_detection.interacting_tile() == 0x2f {
                    4
                } else {
                    1
                };
                if self
                    .game_state
                    .player
                    .tile_detection
                    .ledges_down_leftright()
                    & xand
                    != 0
                    && self.run_ledge_hop_timer()
                {
                    self.link_cancel_dash();
                    let actual_x_velocity = if self
                        .game_state
                        .player
                        .tile_detection
                        .ledges_down_leftright()
                        & 4
                        != 0
                    {
                        16
                    } else {
                        0u8.wrapping_sub(16)
                    };
                    self.follower_link_state_mut()
                        .set_actual_x_velocity(actual_x_velocity);
                    self.setup_horizontal_ledge_hop(14);
                    return;
                }
            }

            if self.game_state.player.tile_detection.horizontal_ledge() & 0x70 != 0
                && self.game_state.player.tile_detection.vertical_ledge() & 0x77 == 0
                && self.run_ledge_hop_timer()
            {
                self.link_cancel_dash();
                self.link_ledge_hop_sfx();
                let last_direction_moved_towards =
                    if self.game_state.player.tile_detection.horizontal_ledge() & 0x40 != 0 {
                        3
                    } else {
                        2
                    };
                self.follower_link_state_mut()
                    .set_last_direction_moved_towards(last_direction_moved_towards);
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().set_speed_setting(0);
                self.link_find_valid_landing_tile_diagonal_north();
                return;
            }
        }

        if (self.game_state.player.tile_detection.stair_tile() & 7) == 7 {
            if self.game_state.player.follower_link.incapacitated_timer() != 0 {
                let r14 = (self.game_state.player.tile_detection.stair_tile() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            } else if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 2
                == 0
            {
                self.follower_link_state_mut().arm_stair_speed_modifier();
                return;
            }
        }

        self.follower_link_state_mut().resolve_dash_speed_setting();
        self.replay_trace_drag_tail("outdoor-y-after-late-speed-setting");
        if self.game_state.player.follower_link.speed_modifier() == 1 {
            self.follower_link_state_mut()
                .promote_pending_speed_modifier();
        }
        self.replay_trace_drag_tail("outdoor-y-after-speed-modifier");

        if self.game_state.player.tile_detection.collision_bits() & 7 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            self.replay_trace_drag_tail("outdoor-y-before-slopes");
            self.flag_moving_into_slopes_y();
            self.replay_trace_drag_tail("outdoor-y-after-slopes");
            if self
                .game_state
                .player
                .follower_link
                .moving_against_diag_tile()
                & 0x0f
                != 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        self.replay_trace_drag_tail("outdoor-y-after-clear-diag");
        if self
            .game_state
            .player
            .tile_detection
            .key_lock_gravestones_low()
            & 2
            != 0
            && self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                == 0
        {
            let timeout = self
                .game_state
                .player
                .follower_link
                .gravestone_push_timeout()
                .wrapping_sub(1);
            self.follower_link_state_mut()
                .set_gravestone_push_timeout(timeout);
            if self.game_state.player.follower_link.is_running() || (timeout as i8).is_negative() {
                let bak = self.game_state.player.tile_detection.collision_bits();
                self.ancilla_add_grave_stone(0x24, 4);
                self.tile_detect_position_mut().set_collision_bits(bak);
                self.follower_link_state_mut()
                    .set_gravestone_push_timeout(52);
            }
        } else {
            self.follower_link_state_mut()
                .set_gravestone_push_timeout(52);
        }
        self.replay_trace_drag_tail("outdoor-y-after-gravestone");

        if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 != 0 {
            if (self.game_state.player.follower_link.incapacitated_timer()
                | self.game_state.player.follower_link.blink_countdown()
                | self.game_state.player.follower_link.is_cape_active() as u8)
                == 0
            {
                let should_damage = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    == 0
                {
                    self.game_state.player.follower_link.y() & 4 == 0
                } else {
                    self.game_state.player.follower_link.y() & 4 != 0
                };
                if should_damage {
                    self.follower_link_state_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.game_state.player.tile_detection.spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
            }
        }
        self.replay_trace_drag_tail("outdoor-y-before-snaps");
        self.handle_pushing_bonking_snaps_y();
        self.replay_trace_submodule("outdoor-y-after-snaps");
    }

    pub(super) fn start_movement_collision_checks_x_handle_outdoors(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            == 0
        {
            self.follower_link_state_mut().clear_speed_modifier();
            if self.game_state.player.follower_link.speed_setting() == 2 {
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }

        if self.game_state.player.tile_detection.pit_tile() & 5 != 0
            && self.game_state.player.tile_detection.collision_bits() & 2 == 0
        {
            self.start_falling_into_hole();
            return;
        }

        let liftable_secondary = if self.game_state.player.tile_detection.read_something() & 2 != 0
        {
            self.game_state.player.tile_detection.liftable_tile_index() >> 1
        } else {
            0
        };
        self.tile_detect_position_mut()
            .set_liftable_action_index_secondary(liftable_secondary);

        if self.game_state.player.tile_detection.deepwater() & 4 != 0
            && !self.game_state.player.follower_link.is_in_deep_water()
            && !self.game_state.player.follower_link.has_auxiliary_state()
        {
            self.link_cancel_dash();
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 1
                && {
                    self.link_force_unequip_cape_quietly();
                    self.game_state.player.follower_link.has_flippers()
                }
            {
                if !self.game_state.player.follower_link.is_bunny_mirror() {
                    self.follower_link_state_mut().set_handler_state(4);
                }
            } else {
                self.restore_link_safe_return_position();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.link_hop_in_or_out_of_water_x();
                self.link_ledge_hop_sfx();
            }
        }

        if if self.game_state.player.follower_link.is_in_deep_water() {
            self.game_state.player.tile_detection.horizontal_ledge() & 7 == 7
        } else {
            self.game_state.player.tile_detection.vertical_ledge() & 0x42 != 0
        } {
            self.tile_detect_position_mut().set_collision_bits(7);
            self.handle_pushing_bonking_snaps_x();
            return;
        }

        if self.game_state.player.tile_detection.normal_tiles() & 7 == 7
            && self.game_state.player.follower_link.is_in_deep_water()
        {
            self.link_cancel_dash();
            if self.game_state.player.follower_link.auxiliary_state() == 0 {
                self.follower_link_state_mut()
                    .set_last_direction_from_swim_flags();
                self.follower_link_state_mut().clear_deep_water_state();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.ancilla_add_splash(0x15, 0);
                self.link_hop_in_or_out_of_water_x();
                return;
            }
        }

        if self.game_state.player.tile_detection.horizontal_ledge() & 7 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            let actual_x_velocity = if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 1
                != 0
            {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(12);
            if self.game_state.world.location.is_outdoors() {
                self.follower_link_state_mut().set_lower_level_state(2);
            }
            let x_bak = self.game_state.player.follower_link.x();
            let rv = self.link_hopping_horizontally_find_tile_x(
                (self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & !2)
                    * 2,
            );
            self.follower_link_state_mut()
                .set_last_direction_moved_towards(1);
            if rv != 0xff {
                self.link_hopping_horizontally_find_tile_y();
            } else {
                self.link_hop_find_tile_to_land_on_south();
            }
            self.follower_link_state_mut().set_x(x_bak);
            return;
        }

        if self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x77 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            let handler_state = if self.game_state.system_signals.sound_effect_1() & 7 == 0 {
                16
            } else {
                15
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
            let actual_x_velocity = if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 1
                != 0
            {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(20);
            self.set_link_z_coord_mirror_low_ff();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.set_allow_scroll_z(1);
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            return;
        }

        if self.game_state.player.tile_detection.horizontal_ledge() & 0x70 != 0
            && self.game_state.player.tile_detection.horizontal_ledge() & 7 == 0
            && self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x77 == 0
            && self.game_state.player.follower_link.handler_state() != 13
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_find_valid_landing_tile_diagonal_north();
            return;
        }

        if self
            .game_state
            .player
            .tile_detection
            .ledges_down_leftright()
            & 7
            != 0
            && self.game_state.player.tile_detection.horizontal_ledge() & 7 == 0
            && self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x77 == 0
            && self.run_ledge_hop_timer()
        {
            let actual_x_velocity = if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 1
                != 0
            {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(14);
            return;
        }

        if self.game_state.player.tile_detection.collision_bits() & 2 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            let skip_check = self.game_state.player.follower_link.is_running()
                && self.game_state.player.follower_link.facing() & 4 == 0;
            if !skip_check
                || self
                    .game_state
                    .enhanced_features
                    .has(FINISH_INDOOR_COLLISION_COMMON_FEATURES0_MISC_BUG_FIXES)
            {
                self.flag_moving_into_slopes_x();
                if self
                    .game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile()
                    & 0x0f
                    != 0
                {
                    return;
                }
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 != 0 {
            if (self.game_state.player.follower_link.incapacitated_timer()
                | self.game_state.player.follower_link.blink_countdown()
                | self.game_state.player.follower_link.is_cape_active() as u8)
                == 0
            {
                let should_damage = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    == 2
                {
                    self.game_state.player.follower_link.x() & 4 == 0
                } else {
                    self.game_state.player.follower_link.x() & 4 != 0
                };
                if should_damage {
                    self.follower_link_state_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.game_state.player.tile_detection.spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn start_falling_into_hole(&mut self) {
        if self.game_state.player.follower_link.handler_state() != 5
            && self.game_state.player.follower_link.handler_state() != 2
        {
            self.follower_link_state_mut().set_sprite_oam_state_timer(9);
            self.follower_link_state_mut().begin_pit_check();
            self.follower_link_state_mut().set_handler_state(1);
        }
    }

    fn setup_horizontal_ledge_hop(&mut self, player_state: u8) {
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut().set_speed_setting(0);
        self.set_allow_scroll_z(1);
        self.follower_link_state_mut().set_auxiliary_state(2);
        self.follower_link_state_mut()
            .set_actual_z_velocity_mirror_and_copy(20);
        self.set_link_z_coord_mirror_low_ff();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut()
            .set_handler_state(player_state);
    }

    fn finish_indoor_y_collision(&mut self) {
        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            self.follower_link_state_mut().clear_direction_lock_bits(2);
            self.set_room_transitioning_flags(0);
            self.follower_link_state_mut()
                .set_force_move_any_direction(0);
        }

        if self.game_state.player.tile_detection.collision_bits() & 7 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            self.follower_link_state_mut().clear_conveyor_belt_state();
            self.flag_moving_into_slopes_y();
            if self
                .game_state
                .player
                .follower_link
                .moving_against_diag_tile()
                & 0x0f
                != 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self.game_state.player.tile_detection.key_lock_gravestones() & 0x20 != 0 {
            let bak = self.game_state.player.tile_detection.collision_bits();
            let mut chest_position = 0;
            let _ = self.OpenChestForItem(
                self.game_state.player.tile_detection.tile_type() as u8,
                &mut chest_position,
            );
            self.tile_detect_position_mut().clear_tile_type();
            self.tile_detect_position_mut().set_collision_bits(bak);
        }

        self.finish_indoor_collision_common(true);
    }

    fn finish_indoor_collision_common(&mut self, y_axis: bool) {
        let r14 = self.game_state.player.tile_detection.collision_bits();
        if !self.game_state.player.follower_link.is_on_lower_level() {
            if self.game_state.player.tile_detection.water_staircase() & 7 != 0 {
                self.set_player_layer_collision(
                    crate::game_state::constants::player::LAYER_COLLISION_BG1,
                    true,
                );
            } else if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 == 0
                && r14 & 2 == 0
            {
                self.set_player_layer_collision(
                    crate::game_state::constants::player::LAYER_COLLISION_BG1,
                    false,
                );
            }
        } else if self.game_state.player.tile_detection.moving_floor_tiles() & 7 != 0 {
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

        if self.game_state.player.tile_detection.misc_tiles() & 0x2200 != 0 {
            let dy = if self.game_state.player.tile_detection.misc_tiles() & 0x2000 != 0 {
                8
            } else {
                0
            };
            let dir = self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards() as usize;
            let rupees = self
                .game_state
                .inventory
                .player_resources
                .rupees_goal()
                .wrapping_add(5);
            self.player_resources_mut().set_rupees_goal(rupees);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(FINISH_INDOOR_COLLISION_COMMON_RUPEE_Y_OFFSETS[dir] as u16)
                .wrapping_sub(dy);
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(FINISH_INDOOR_COLLISION_COMMON_RUPEE_X_OFFSETS[dir] as u16);
            self.dungeon_delete_rupee_tile_for_player(x, y);
            self.ancilla_sfx3_near(10);
        }

        let moving_floor_flags = self
            .game_state
            .dungeon
            .environment
            .moving_floor_check_flags();
        if moving_floor_flags & 0x22 != 0 {
            self.follower_link_state_mut()
                .set_conveyor_belt_state(if moving_floor_flags & 0x20 != 0 { 2 } else { 1 });
        } else if moving_floor_flags & 0x2200 != 0 {
            self.follower_link_state_mut().set_conveyor_belt_state(
                if moving_floor_flags & 0x2000 != 0 {
                    4
                } else {
                    3
                },
            );
        } else if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 == 0
            && r14 & 2 == 0
        {
            self.follower_link_state_mut().clear_conveyor_belt_state();
        }

        if y_axis {
            self.finish_indoor_y_collision_tail();
        } else {
            self.finish_indoor_x_collision_tail();
        }
    }

    fn finish_indoor_y_collision_tail(&mut self) {
        if (self.game_state.player.tile_detection.vertical_ledge() & 7) == 7
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .increment_about_to_jump_off_ledge();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.ancilla_sfx2_near(0x20);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_y();
        } else if (self.game_state.player.tile_detection.deepwater() & 7) == 7
            && !self.game_state.player.follower_link.is_in_deep_water()
        {
            self.link_cancel_dash();
            if self.game_state.display.sub_screen_layers == 0 {
                self.dungeon_handle_layer_change();
            } else {
                self.follower_link_state_mut().enter_deep_water_state();
                self.follower_link_state_mut()
                    .set_swim_flags_from_last_direction();
                self.follower_link_state_mut()
                    .clear_state_item_and_grab_flags();
                self.follower_link_state_mut().set_speed_setting(0);
                self.link_reset_swimming_state();
                self.ancilla_sfx2_near(0x20);
            }
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_y();
        } else if self.game_state.player.tile_detection.normal_tiles() & 2 != 0
            && self.game_state.player.follower_link.is_in_deep_water()
        {
            if self.game_state.player.follower_link.has_auxiliary_state() {
                self.tile_detect_position_mut().set_collision_bits(7);
            } else {
                self.link_cancel_dash();
                self.follower_link_state_mut()
                    .set_last_direction_from_swim_flags();
                self.follower_link_state_mut().clear_deep_water_state();
                if self.ancilla_add_splash(0x15, 0) {
                    self.follower_link_state_mut().enter_deep_water_state();
                    self.tile_detect_position_mut().set_collision_bits(7);
                } else {
                    self.follower_link_state_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.link_hop_in_or_out_of_water_y();
                }
            }
        }

        if (self.game_state.player.tile_detection.stair_tile() & 7) == 7 {
            if self.game_state.player.follower_link.incapacitated_timer() != 0 {
                let stair_bits = (self.game_state.player.tile_detection.stair_tile() & 7) as u16;
                self.tile_detect_position_mut()
                    .set_collision_bits(stair_bits);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            let stairs = self.game_state.player.tile_detection.inroom_staircase();
            if stairs & 0x77 != 0 {
                let submodule = if stairs & 0x70 != 0 { 16 } else { 8 };
                self.set_submodule(submodule);
                self.set_main_module(7);
                self.link_cancel_dash();
            } else {
                if self
                    .game_state
                    .enhanced_features
                    .has(LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING)
                {
                    self.link_cancel_dash();
                }
            }
            if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 2
                == 0
            {
                self.follower_link_state_mut().arm_stair_speed_modifier();
                return;
            }
        }

        if self.finish_indoor_collision_shared_tail(true) {
            return;
        }
        self.handle_pushing_bonking_snaps_y();
    }

    fn finish_indoor_x_collision_tail(&mut self) {
        if self.game_state.player.tile_detection.horizontal_ledge() & 7 == 7
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .increment_about_to_jump_off_ledge();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_x();
            self.link_ledge_hop_sfx();
            return;
        }

        if self.finish_indoor_collision_shared_tail(false) {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            == 0
        {
            self.follower_link_state_mut().clear_speed_modifier();
            if self.game_state.player.follower_link.speed_setting() == 2 {
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn finish_indoor_collision_shared_tail(&mut self, y_axis: bool) -> bool {
        if y_axis {
            self.follower_link_state_mut().resolve_dash_speed_setting();
            self.follower_link_state_mut()
                .promote_pending_speed_modifier();
        }

        let r14 = self.game_state.player.tile_detection.collision_bits();
        if self.game_state.player.tile_detection.pit_tile() & 5 != 0 && r14 & 2 == 0 {
            self.start_falling_into_hole();
            return true;
        }

        if y_axis {
            self.follower_link_state_mut().clear_pit_data_index();
        } else {
            self.follower_link_state_mut().clear_near_pit_state();
        }
        if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 != 0 {
            if (self.game_state.player.follower_link.incapacitated_timer()
                | self.game_state.player.follower_link.blink_countdown()
                | self.game_state.player.follower_link.is_cape_active() as u8)
                == 0
            {
                let coord = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & 2
                    == 0
                {
                    self.game_state.player.follower_link.y()
                } else {
                    self.game_state.player.follower_link.x()
                };
                let low_phase = coord & 4 == 0;
                let damage = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & 1
                    == 0
                {
                    low_phase
                } else {
                    !low_phase
                };
                if damage {
                    self.follower_link_state_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return true;
                }
            } else {
                let spike_bits =
                    (self.game_state.player.tile_detection.spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_mut()
                    .set_collision_bits(spike_bits);
            }
        }

        if self.game_state.dungeon.room_load.header_collision() == 0
            || self.game_state.dungeon.room_load.header_collision() == 4
            || !self.game_state.player.follower_link.is_on_lower_level()
        {
            self.handle_indoor_pushblock_timeout(y_axis);
        }
        false
    }

    fn handle_indoor_pushblock_timeout(&mut self, y_axis: bool) {
        let block_flags = self.game_state.player.tile_detection.block_flags();
        if block_flags != 0
            && self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
        {
            self.tile_detect_position_mut()
                .set_staircase_cache(block_flags as u8);
            self.follower_link_state_mut()
                .decrement_gravestone_push_timeout();
            if !(self
                .game_state
                .player
                .follower_link
                .gravestone_push_timeout() as i8)
                .is_negative()
            {
                return;
            }
            let mut bits = block_flags;
            for i in (0..=15).rev() {
                if bits & 0x8000 != 0 {
                    let idx = self.find_free_moving_block_slot(i);
                    if idx != 0xff {
                        let slot = idx as usize;
                        self.tile_detect_position_mut()
                            .set_collision_bits(idx as u16);
                        if !self.initialize_push_block(idx, (i * 2) as u8) {
                            self.sprite_dungeon_draw_single_push_block(slot * 2);
                            self.tile_detect_position_mut().set_collision_bits(4);
                            let facing = self
                                .game_state
                                .player
                                .follower_link
                                .last_direction_moved_towards()
                                * 2;
                            self.pushed_block_mut().set_facing_player(slot, facing);
                            self.pushed_block_mut().set_push_direction(facing);
                            let target = if y_axis {
                                let y_lo = self.game_state.player.pushed_block.y_low(slot);
                                y_lo.wrapping_sub(u8::from(
                                    self.game_state
                                        .player
                                        .follower_link
                                        .last_direction_moved_towards()
                                        == 1,
                                ))
                            } else {
                                let x_lo = self.game_state.player.pushed_block.x_low(slot);
                                x_lo.wrapping_sub(u8::from(
                                    self.game_state
                                        .player
                                        .follower_link
                                        .last_direction_moved_towards()
                                        != 2,
                                ))
                            } & 0x0f;
                            self.pushed_block_mut().set_target_low(slot, target);
                        }
                    }
                }
                bits <<= 1;
            }
        }
        self.follower_link_state_mut()
            .set_gravestone_push_timeout(21);
    }

    fn dungeon_delete_rupee_tile_for_player(&mut self, x: u16, y: u16) {
        let pos = ((y & 0x01f8) * 8) | ((x & 0x01f8) >> 3);
        let dst = self.game_state.display.current_vram_upload_data_address();
        self.write_vram_upload_absolute_word(dst + 4, 0x190f);
        self.write_vram_upload_absolute_word(dst + 10, 0x190f);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos as usize, 0x190f);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile((pos + 64) as usize, 0x190f);
        let attr = u16::from(self.dungeon_tile_attribute(0x190f)) * 0x0101;
        let vram0 = self.Dungeon_MapVramAddr(pos);
        let vram1 = self.Dungeon_MapVramAddr(pos + 64);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_attr_word(pos as usize, attr);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_attr_word((pos + 64) as usize, attr);
        self.write_vram_upload_absolute_word(dst, vram0);
        self.write_vram_upload_absolute_word(dst + 6, vram1);
        self.write_vram_upload_absolute_word(dst + 2, 0x0100);
        self.write_vram_upload_absolute_word(dst + 8, 0x0100);
        self.write_vram_upload_absolute_word(dst + 12, 0xffff);
        self.advance_vram_upload_cursor_by(24);
        self.dungeon_savegame_state_mut()
            .set_savegame_state_high_bits(0x10);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn link_handle_liftables(&mut self) -> u8 {
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let facing = self.game_state.player.follower_link.facing_index();
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let y0 = link_y.wrapping_add(LINK_HANDLE_LIFTABLES_ACTION_Y[facing] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(20) & mask;
        let x0 =
            (link_x.wrapping_add(LINK_HANDLE_LIFTABLES_ACTION_X[facing] as i16 as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(8) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let mut action = if (self.game_state.player.tile_detection.collision_bits()
            | self.game_state.player.tile_detection.vertical_ledge() as u16)
            & 1
            != 0
        {
            3
        } else {
            2
        };

        if self.game_state.world.location.is_indoors() {
            let liftable = self.Dungeon_CheckForAndIDLiftableTile();
            if liftable != 0xffff {
                self.tile_detect_position_mut()
                    .set_liftable_action_index_primary(
                        LINK_HANDLE_LIFTABLES_ACTION_FOR_TILE[(liftable & 0x0f) as usize],
                    );
            } else {
                if self.game_state.player.tile_detection.read_something() & 1 != 0
                    && self.game_state.player.follower_link.facing() == 0
                    && self.game_state.player.tile_detection.liftable_tile_index() == 0
                {
                    action = 4;
                }
                if self.game_state.player.tile_detection.chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
        } else {
            if self.game_state.player.tile_detection.read_something() & 1 == 0 {
                if self.game_state.player.tile_detection.chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            if self.game_state.player.follower_link.facing() == 0
                && self.game_state.player.tile_detection.liftable_tile_index() == 0
            {
                action = 4;
                if self.game_state.player.tile_detection.chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            let liftable_primary = self.game_state.player.tile_detection.liftable_tile_index() >> 1;
            self.tile_detect_position_mut()
                .set_liftable_action_index_primary(liftable_primary);
        }

        let liftable_index = self
            .game_state
            .player
            .tile_detection
            .liftable_action_index_primary() as usize;
        if self.game_state.inventory.items.gloves()
            >= LINK_HANDLE_LIFTABLES_ACTION_FOR_GLOVES[liftable_index]
        {
            action = 1;
        }

        if self.game_state.player.tile_detection.chest() & 1 != 0 {
            action = 5;
        }
        action
    }

    pub(super) fn link_bonk_and_smash(&mut self) {
        if !self.game_state.player.follower_link.is_running()
            || self.game_state.player.follower_link.dash_counter() == 64
            || self.game_state.player.tile_detection.dashable_tiles() & 0x70 == 0
        {
            return;
        }

        for i in 0..2 {
            if let Some((j, x, y)) = self.overworld_smash_rock_pile_result(i != 0) {
                if let Some(k) = LINK_BONK_AND_SMASH_LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE
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
        let bak = self.game_state.player.follower_link.y();
        if down_one_tile {
            self.follower_link_state_mut().set_y(bak.wrapping_add(8));
        }
        let (pos, x, y) = self.overworld_get_link_map16_coords_result();
        self.follower_link_state_mut().set_y(bak);
        let a = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos);
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
        let dir = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(OVERWORLD_GET_LINK_MAP16_COORDS_RESULT_X_OFFSETS[dir] as u16)
            & !0x0f;
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(OVERWORLD_GET_LINK_MAP16_COORDS_RESULT_Y_OFFSETS[dir] as u16)
            & !0x0f;
        let pos = ((y.wrapping_sub(self.game_state.world.scroll.overworld_offset_base_y())
            & self.game_state.world.scroll.overworld_offset_mask_y())
            << 3)
            .wrapping_add(
                ((x >> 3).wrapping_sub(self.game_state.world.scroll.overworld_offset_base_x()))
                    & self.game_state.world.scroll.overworld_offset_mask_x(),
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
        let secret = self.overworld_reveal_secret(pos);
        if secret != 0 {
            tile = secret;
        }
        self.dungeon_room_tilemaps_mut()
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
        let pos = 2
            * ((pos >> 1).wrapping_add(
                SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_MAP16_QUADRANT_OFFSETS[quadrant] as u16,
            ));
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(pos);
        self.dungeon_doors_mut().set_door_open_counter(40);
        let secret = self.overworld_reveal_secret(pos);
        if secret == 0xffff {
            let screen =
                u16::from(self.game_state.world.location.overworld_screen_index()) as usize;
            self.set_overworld_event_bits(screen, 0x20);
            self.set_sound_effect_2(27);
            self.dungeon_doors_mut().set_door_open_counter(80);
        }
        let x = x.wrapping_add(
            (SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_QUADRANT_X_OFFSETS[quadrant] * 2) as u16,
        );
        let y = y.wrapping_add(
            (SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_QUADRANT_Y_OFFSETS[quadrant] * 2) as u16,
        );
        self.overworld_do_map_update32x32_b_for_smash();
        self.map16_quadrant_attr(a, x, y)
    }

    pub(super) fn overworld_memorize_map16_change_for_smash(&mut self, pos: u16, value: u16) {
        if value == 0x0dc5 || value == 0x0dc9 {
            return;
        }
        let x = self.game_state.memorized_tiles.count() as usize;
        self.memorized_tile_mut().set_entry_value(x, value);
        self.memorized_tile_mut().set_entry_addr(x, pos);
        self.memorized_tile_mut().set_count((x + 2) as u16);
    }

    pub(super) fn overworld_draw_map16_for_smash(&mut self, pos: u16, value: u16) {
        let vram_pos = self.overworld_find_map16_vram_address_for_smash(pos);
        let dst = self.game_state.display.current_vram_upload_data_address();
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
        self.dungeon_doors_mut().clear_door_open_counter_low();
    }

    fn overworld_do_map_update32x32_for_smash(&mut self) {
        let i = self.game_state.memorized_tiles.count() as usize;
        let j = (self.game_state.dungeon.doors.door_open_counter() >> 1) as usize;
        let base = self
            .game_state
            .dungeon
            .object_tracking
            .big_rock_starting_address();
        let entries = [
            (
                base,
                OVERWORLD_DO_MAP_UPDATE32X32_FOR_SMASH_DOOR_ANIM_TILES[j],
            ),
            (
                base.wrapping_add(2),
                OVERWORLD_DO_MAP_UPDATE32X32_FOR_SMASH_DOOR_ANIM_TILES[j + 1],
            ),
            (
                base.wrapping_add(0x80),
                OVERWORLD_DO_MAP_UPDATE32X32_FOR_SMASH_DOOR_ANIM_TILES[j + 2],
            ),
            (
                base.wrapping_add(0x82),
                OVERWORLD_DO_MAP_UPDATE32X32_FOR_SMASH_DOOR_ANIM_TILES[j + 3],
            ),
        ];
        for (n, (pos, tile)) in entries.into_iter().enumerate() {
            self.memorized_tile_mut().set_entry_addr(i + n * 2, pos);
            self.memorized_tile_mut().set_entry_value(i + n * 2, tile);
            self.overworld_draw_map16_persist_for_smash(pos, tile);
        }
        let upload = self.game_state.display.vram_upload_cursor_usize();
        self.write_vram_upload_buffer_word(upload, 0xffff);
        self.memorized_tile_mut().set_count((i + 8) as u16);
        let step = self
            .game_state
            .dungeon
            .doors
            .door_animation_step()
            .wrapping_add(if self.game_state.dungeon.doors.door_open_counter() == 32 {
                2
            } else {
                1
            });
        self.dungeon_doors_mut().set_door_animation_step(step);
        self.set_bg_vram_load_mode(1);
        self.dungeon_doors_mut().increment_door_open_counter_low();
    }

    fn overworld_draw_map16_persist_for_smash(&mut self, pos: u16, value: u16) {
        self.dungeon_room_tilemaps_mut()
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
        self.follower_link_state_mut().mask_direction(0x0f);
        self.player_limit_directions_inner();
    }

    pub(super) fn link_handle_diagonal_kickback(&mut self) {
        if self.game_state.player.follower_link.x_velocity() == 0
            || self.game_state.player.follower_link.y_velocity() == 0
        {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        self.follower_link_state_mut()
            .cache_copied_position_from_current();

        self.tile_detect_movement_x(
            if self
                .game_state
                .player
                .follower_link
                .x_velocity_signed()
                .is_negative()
            {
                2
            } else {
                3
            },
        );
        if self.game_state.player.tile_detection.slope_collision_bits() & 5 == 0 {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }
        self.flag_moving_into_slopes_x();
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x0f
            == 0
        {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        let xd = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.player.follower_link.copied_x()) as u8;
        let copied_x = self.game_state.player.follower_link.copied_x();
        self.follower_link_state_mut().set_x(copied_x);
        self.follower_link_state_mut().set_x_velocity(xd);

        self.tile_detect_movement_y(
            if self
                .game_state
                .player
                .follower_link
                .y_velocity_signed()
                .is_negative()
            {
                0
            } else {
                1
            },
        );
        if self.game_state.player.tile_detection.slope_collision_bits() & 5 == 0 {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }
        self.flag_moving_into_slopes_y();
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x0f
            == 0
        {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        let diag_tile = self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile();
        self.follower_link_state_mut()
            .set_moving_against_diag_deadlocked(diag_tile);
        let yd = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.player.follower_link.copied_y()) as u8;
        self.follower_link_state_mut().set_y_velocity(yd);

        let x_vel = self.game_state.player.follower_link.x_velocity_signed();
        let x_idx = x_vel.unsigned_abs() as usize;
        let x_delta = if x_vel < 0 {
            LINK_HANDLE_DIAGONAL_KICKBACK_X_OFFSETS_1[x_idx]
        } else {
            LINK_HANDLE_DIAGONAL_KICKBACK_X_OFFSETS_0[x_idx]
        };
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(x_delta as i16 as u16);
        self.follower_link_state_mut().set_x(x);

        let y_vel = self.game_state.player.follower_link.y_velocity_signed();
        let y_idx = y_vel.unsigned_abs() as usize;
        let y_delta = if y_vel < 0 {
            LINK_HANDLE_DIAGONAL_KICKBACK_Y_OFFSETS_1[y_idx]
        } else {
            LINK_HANDLE_DIAGONAL_KICKBACK_Y_OFFSETS_0[y_idx]
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y_delta as i16 as u16);
        self.follower_link_state_mut().set_y(y);

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
    }

    pub(super) fn link_handle_cardinal_collision(&mut self) {
        self.replay_trace_submodule("cardinal-entry");
        self.tile_detect_position_mut().clear_diag_state();
        self.tile_detect_position_mut().clear_diagonal_tile();

        let can_double_layer = if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x30
            != 0
        {
            true
        } else {
            self.link_handle_diagonal_kickback();
            self.game_state
                .player
                .follower_link
                .moving_against_diag_deadlocked()
                == 0
        };

        if can_double_layer && self.check_if_room_needs_double_layer_check() {
            if self.game_state.dungeon.room_load.header_collision() >= 2
                && self.game_state.dungeon.room_load.header_collision() != 3
            {
                self.follower_link_state_mut().set_tile_coll_flag(2);
                self.player_tile_detect_nearby();
                let collision_bits = self.game_state.player.tile_detection.collision_bits() as u8;
                self.tile_detect_position_mut()
                    .set_tile_collision_bits_primary(collision_bits);
                if self
                    .game_state
                    .player
                    .tile_detection
                    .tile_collision_bits_primary()
                    != 0
                {
                    let floor_x_velocity =
                        self.game_state.dungeon.moving_floor.floor_x_velocity_low() as u16;
                    let floor_y_velocity =
                        self.game_state.dungeon.moving_floor.floor_y_velocity_low() as u16;
                    self.follower_link_state_mut()
                        .add_movement_velocity_delta(floor_x_velocity, floor_y_velocity);

                    let a = self.game_state.player.tile_detection.collision_bits() as u8;
                    let horizontal_first = if a == 12 || a == 3 {
                        false
                    } else if a == 10 || a == 5 {
                        true
                    } else if (a & 0x0c) == 0 && (a & 3) == 0 {
                        false
                    } else if self.game_state.player.follower_link.y_velocity() != 0 {
                        true
                    } else if self.game_state.player.follower_link.x_velocity() == 0 {
                        false
                    } else {
                        (self.game_state.dungeon.moving_floor.floor_y_velocity_low() as i8) >= 0
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

        let collision = self.game_state.dungeon.room_load.header_collision();
        let moved = (self.game_state.player.follower_link.x_velocity()
            | self.game_state.player.follower_link.y_velocity())
            != 0;
        if collision == 2 {
            self.player_tile_detect_nearby();
            if (self.game_state.player.tile_detection.collision_bits() as u8
                | self
                    .game_state
                    .player
                    .tile_detection
                    .tile_collision_bits_primary())
                == 0x0f
            {
                if self.game_state.player.follower_link.blink_countdown() == 0 {
                    self.follower_link_state_mut().set_blink_countdown(58);
                }
                if self.game_state.player.follower_link.direction() == 0 {
                    if self.game_state.dungeon.moving_floor.floor_y_velocity_low() != 0 {
                        let y_velocity = self.game_state.player.follower_link.y_velocity();
                        self.follower_link_state_mut()
                            .set_y_velocity((0u8).wrapping_sub(y_velocity));
                    }
                    if self.game_state.dungeon.moving_floor.floor_x_velocity_low() != 0 {
                        let x_velocity = self.game_state.player.follower_link.x_velocity();
                        self.follower_link_state_mut()
                            .set_x_velocity((0u8).wrapping_sub(x_velocity));
                    }
                }
            }
            self.follower_link_state_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks_vertical_first();
        } else if collision == 3 {
            self.follower_link_state_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks_horizontal_first();
        } else if collision == 4 || moved {
            self.follower_link_state_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks_vertical_first();
        } else if !self
            .game_state
            .player
            .follower_link
            .is_edge_transition_blocked_by_handler_state()
            && self.game_state.player.follower_link.handler_state() != 19
        {
            self.player_tile_detect_nearby();
            if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0 {
                self.follower_link_state_mut().set_handler_state(1);
                if !self.game_state.player.follower_link.is_running() {
                    self.follower_link_state_mut().set_speed_setting(4);
                }
            }
        }

        self.tile_detect_main_handler(0);
        self.replay_trace_submodule("cardinal-after-tile-main");
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            != 0
        {
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
        }

        self.follower_link_state_mut()
            .refresh_direction_from_safe_return_delta();
        self.replay_trace_submodule("cardinal-after-dir-vel");

        if self.game_state.world.location.is_outdoors()
            || self.game_state.dungeon.room_load.header_collision() != 4
            || self.game_state.player.follower_link.handler_state() != 4
        {
            return;
        }

        if self.game_state.dungeon.moving_floor.floor_y_velocity_low() != 0
            && self
                .game_state
                .player
                .follower_link
                .y_velocity()
                .wrapping_sub(self.game_state.dungeon.moving_floor.floor_y_velocity_low())
                == 0
        {
            if (self.game_state.dungeon.moving_floor.floor_y_velocity_low() as i8).is_negative() {
                self.follower_link_state_mut().clear_direction_flags(8);
            } else {
                self.follower_link_state_mut().clear_direction_flags(4);
            }
        }
        if self.game_state.dungeon.moving_floor.floor_x_velocity_low() != 0
            && self
                .game_state
                .player
                .follower_link
                .x_velocity()
                .wrapping_sub(self.game_state.dungeon.moving_floor.floor_x_velocity_low())
                == 0
        {
            if (self.game_state.dungeon.moving_floor.floor_x_velocity_low() as i8).is_negative() {
                self.follower_link_state_mut().clear_direction_flags(2);
            } else {
                self.follower_link_state_mut().clear_direction_flags(1);
            }
        }
    }

    pub(super) fn link_state_recoil(&mut self) {
        self.replay_trace_player_state("recoil-entry");
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();
        self.store_link_safe_return_position(old_x, old_y);

        self.link_handle_change_in_z_velocity();
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();

        if self.game_state.player.follower_link.is_z_low_negative()
            && (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative()
        {
            self.tile_detect_main_handler(5);
            if self.game_state.player.tile_detection.deepwater() & 1 != 0 {
                self.follower_link_state_mut().set_handler_state(4);
                self.link_set_to_deep_water();
                self.link_reset_sword_and_item_usage();
                self.ancilla_add_splash(21, 0);
                self.link_handle_recoil_and_timer(true);
            } else {
                let recoil_timer = self.follower_link_state_mut().increment_recoil_timer();
                if recoil_timer != 4 {
                    let mut z = self
                        .game_state
                        .player
                        .follower_link
                        .actual_z_velocity_copy();
                    let mut s = recoil_timer;
                    loop {
                        z >>= 1;
                        s = s.wrapping_sub(1);
                        if s != 0 {
                            break;
                        }
                    }
                    self.follower_link_state_mut().set_actual_z_velocity(z);
                } else {
                    self.follower_link_state_mut().set_recoil_timer(3);
                }
                self.link_handle_recoil_and_timer(false);
            }
        } else {
            self.link_handle_recoil_and_timer(false);
        }
        self.follower_link_state_mut().clear_z_high();
        self.replay_trace_player_state("recoil-exit");
    }

    pub(super) fn link_state_sleeping(&mut self) {
        match self.game_state.player.follower_link.sleep_in_bed_state() {
            0 => {
                if self.game_state.frame.frame_counter & 0x1f == 0 {
                    self.ancilla_add_snoring(0x21, 1);
                }
            }
            1 => {
                if self.game_state.frame.submodule == 0 {
                    if (self.follower_link_state_mut().decrement_dash_countdown() as i8)
                        .is_negative()
                    {
                        self.follower_link_state_mut().set_dash_countdown(0);
                        let input = (self.game_state.player.follower_link.filtered_joypad_h()
                            & 0xe0)
                            | (self.game_state.player.follower_link.filtered_joypad_h() << 4)
                            | self.game_state.player.follower_link.filtered_joypad_l();
                        if input & 0xf0 != 0 {
                            self.follower_link_state_mut().increment_opening_pose();
                            self.follower_link_state_mut().set_facing(6);
                            self.follower_link_state_mut()
                                .increment_sleep_in_bed_state();
                            self.follower_link_state_mut().set_dash_countdown(4);
                        }
                    }
                }
            }
            2 => {
                if (self.follower_link_state_mut().decrement_dash_countdown() as i8).is_negative() {
                    self.follower_link_state_mut().set_actual_velocity_xy(21, 4);
                    self.follower_link_state_mut()
                        .set_actual_z_velocity_and_copy(24);
                    self.follower_link_state_mut().set_incapacitated_timer(16);
                    self.follower_link_state_mut().set_auxiliary_state(2);
                    self.follower_link_state_mut().set_handler_state(6);
                }
            }
            _ => {}
        }
    }

    pub(super) fn link_state_zapped(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.LinkZap_HandleMosaic();

        let delay = self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer();
        if !(delay as i8).is_negative() {
            return;
        }

        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(2);
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        if self.game_state.player.follower_link.action_handler_timer() & 1 != 0 {
            self.palette_electro_themed_gear();
        } else {
            self.load_actual_gear_palettes();
        }
        if self.game_state.player.follower_link.action_handler_timer() == 8 {
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().clear_electrocute_on_touch();
            self.follower_link_state_mut().clear_auxiliary_state();
            self.Player_SetCustomMosaicLevel(0);
        }
    }

    pub(super) fn link_state_exiting_dash(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.game_state.player.follower_link.joypad1h_last() & 0x0f != 0
            || self.game_state.player.follower_link.dash_countdown() >= 16
        {
            self.follower_link_state_mut().set_dash_countdown(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut().clear_running();
            self.swim_acceleration_mut().set_mode(0, 0);
            if self.game_state.player.follower_link.button_b_frames() < 9 {
                self.follower_link_state_mut().clear_direction_lock();
            }
        } else {
            self.follower_link_state_mut().increment_dash_countdown();
        }
        self.link_handle_moving_animation_full_long_entry();
    }

    pub(super) fn reset_all_acceleration(&mut self) {
        self.swim_acceleration_mut().clear_axis_motion(0);
        self.swim_acceleration_mut().clear_axis_motion(2);
        for offset in [0, 2] {
            self.follower_link_state_mut()
                .set_swim_stroke_frame_counter(offset, 0);
        }
    }

    pub(super) fn link_force_unequip_cape_quietly(&mut self) {
        self.follower_link_state_mut().set_cape_transform_timer(32);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.follower_link_state_mut().set_cape_mode(0);
        self.follower_link_state_mut().clear_electrocute_on_touch();
    }

    pub(super) fn link_force_unequip_cape(&mut self) {
        self.ancilla_add_cape_poof(35, 4);
        self.ancilla_sfx2_near(21);
        self.link_force_unequip_cape_quietly();
    }

    pub(super) fn halt_link_when_using_items(&mut self) {
        if self.game_state.dungeon.room_load.header_collision_2() == 2
            && self.has_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BOTH,
            )
        {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().clear_movement_subpixels();
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
        }
        if self
            .game_state
            .player
            .follower_link
            .has_somaria_platform_state()
        {
            self.follower_link_state_mut().set_direction(0);
        }
    }

    pub(super) fn link_handle_cape_passive_lift_check(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || (self
                .game_state
                .enhanced_features
                .has(LINK_HANDLE_CAPE_PASSIVE_LIFT_CHECK_FEATURES0_MISC_BUG_FIXES)
                && self
                    .game_state
                    .player
                    .follower_link
                    .has_grabbing_wall_state())
        {
            self.player_check_handle_cape_stuff();
        }
    }

    pub(super) fn player_check_handle_cape_stuff(&mut self) {
        if !self.game_state.player.follower_link.is_cape_active()
            || self.game_state.player.follower_link.current_item_active() != 19
        {
            return;
        }
        if self.game_state.player.follower_link.current_item_active()
            == self.game_state.player.follower_link.current_item_y()
        {
            self.follower_link_state_mut()
                .decrement_cape_decrement_counter();
            if self
                .game_state
                .player
                .follower_link
                .cape_decrement_counter()
                != 0
            {
                return;
            }
            let cape_timer = PLAYER_CHECK_HANDLE_CAPE_STUFF_CAPE_DEPLETION_TIMERS
                [self.magic_consumption_level_live() as usize];
            self.follower_link_state_mut()
                .set_cape_decrement_counter(cape_timer);
            if self.game_state.player.follower_link.magic_power() == 0 {
                return;
            }
            if self.follower_link_state_mut().decrement_magic_power() != 0 {
                return;
            }
        }
        self.link_force_unequip_cape();
    }

    pub(super) fn check_y_button_press(&mut self) -> bool {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.game_state.player.follower_link.filtered_joypad_h() & 0x40 == 0
        {
            return false;
        }
        self.follower_link_state_mut()
            .add_button_mask_b_y_bits(0x40);
        true
    }

    pub(super) fn link_check_magic_cost(&mut self, item: u8) -> bool {
        let idx = item as usize * 3 + self.magic_consumption_level_live() as usize;
        let cost = LINK_CHECK_MAGIC_COST_LINK_ITEM_MAGIC_COSTS[idx];
        if self.follower_link_state_mut().spend_magic(cost) {
            return true;
        }
        if item != 3 {
            self.ancilla_sfx2_near(60);
            self.dialogue_message_index_mut().set_value(123);
            self.main_show_text_message();
        }
        false
    }

    pub(super) fn refund_magic(&mut self, item: u8) {
        let idx = item as usize * 3 + self.magic_consumption_level_live() as usize;
        let cost = REFUND_MAGIC_LINK_ITEM_MAGIC_COSTS[idx];

        let clamp_full = self
            .game_state
            .enhanced_features
            .has(REFUND_MAGIC_FEATURES0_MISC_BUG_FIXES);
        self.follower_link_state_mut()
            .refund_magic(cost, clamp_full);
    }

    pub(super) fn link_item_reset_from_overworld_things(&mut self) {
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut()
            .clear_state_item_and_grab_flags();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_item_cape(&mut self) {
        if !self.game_state.player.follower_link.is_cape_active() {
            if (self.follower_link_state_mut().tick_cape_transform_timer() as i8) >= 0 {
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.halt_link_when_using_items();
                return;
            }

            self.follower_link_state_mut().clear_cape_transform_timer();
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            if self.game_state.player.follower_link.magic_power() == 0 {
                self.ancilla_sfx2_near(60);
                self.dialogue_message_index_mut().set_value(123);
                self.main_show_text_message();
                return;
            }

            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_cape_mode(1);
            let cape_timer =
                LINK_ITEM_CAPE_CAPE_DEPLETION_TIMERS[self.magic_consumption_level_live() as usize];
            self.follower_link_state_mut()
                .set_cape_decrement_counter(cape_timer);
            self.follower_link_state_mut().set_cape_transform_timer(20);
            self.ancilla_add_cape_poof(35, 4);
            self.ancilla_sfx2_near(20);
            return;
        }

        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.follower_link_state_mut()
            .decrement_cape_decrement_counter();
        if self
            .game_state
            .player
            .follower_link
            .cape_decrement_counter()
            == 0
        {
            let cape_timer =
                LINK_ITEM_CAPE_CAPE_DEPLETION_TIMERS[self.magic_consumption_level_live() as usize];
            self.follower_link_state_mut()
                .set_cape_decrement_counter(cape_timer);
            if self.game_state.player.follower_link.magic_power() == 0
                && self
                    .game_state
                    .enhanced_features
                    .has(LINK_ITEM_CAPE_FEATURES0_MISC_BUG_FIXES)
            {
                self.link_force_unequip_cape();
                return;
            }
            if self.follower_link_state_mut().decrement_magic_power() == 0 {
                self.link_force_unequip_cape();
                return;
            }
        }

        if (self.follower_link_state_mut().tick_cape_transform_timer() as i8) < 0 {
            self.follower_link_state_mut().clear_cape_transform_timer();
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x40 != 0 {
                self.link_force_unequip_cape();
            }
        }
    }

    pub(super) fn link_item_rod(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            if !self.link_check_magic_cost(0) {
                self.follower_link_state_mut()
                    .clear_button_mask_b_y_bits(0x40);
                return;
            }
            self.follower_link_state_mut()
                .set_item_action_debug_value_2(1);
            if self.game_state.player.follower_link.selected_rod() == 1 {
                self.ancilla_add_fire_rod_shot(2, 1);
            } else {
                self.ancilla_add_ice_rod_shot(11, 1);
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_ROD_ROD_ANIM_DELAYS[0]);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_item_in_hand(1);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        if step < LINK_ITEM_ROD_ROD_ANIM_DELAYS.len() {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_ROD_ROD_ANIM_DELAYS[step]);
            return;
        }
        self.follower_link_state_mut()
            .set_item_action_debug_value_2(0);
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().clear_item_in_hand_bits(1);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_item_hammer(&mut self) {
        if self.game_state.player.follower_link.item_in_hand_has(0x10) {
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || self.game_state.player.follower_link.filtered_joypad_h() & 0x40 == 0
            {
                return;
            }
            self.follower_link_state_mut()
                .add_button_mask_b_y_bits(0x40);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS[0]);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_item_in_hand(2);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        self.follower_link_state_mut().set_spin_attack_delay_timer(
            LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS
                [step.min(LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS.len() - 1)],
        );
        if self.game_state.player.follower_link.action_handler_timer() == 1 {
            self.tile_detect_main_handler(3);
            self.ancilla_add_hit_stars(22, 0);
            if self.game_state.system_signals.sound_effect_1() == 0 {
                self.ancilla_sfx2_near(16);
                self.spawn_hammer_water_splash();
            }
        } else if self.game_state.player.follower_link.action_handler_timer() == 3 {
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().clear_item_in_hand_bits(2);
        }
    }

    pub(super) fn link_item_bow(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_BOW_BOW_DELAYS[0]);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_item_in_hand(16);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        if step < LINK_ITEM_BOW_BOW_DELAYS.len() {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_BOW_BOW_DELAYS[step]);
            return;
        }

        let k = self.ancilla_add_arrow(
            9,
            self.game_state.player.follower_link.facing(),
            2,
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
        );
        if k >= 0 {
            let k = k as usize;
            if self.game_state.archery_game.arrows_left() != 0 {
                self.archery_game_mut().decrement_arrows_left();
                self.player_resources_mut().increment_arrows_by(2);
            }
            if self.game_state.archery_game.out_of_arrows() == 0
                && self.game_state.inventory.player_resources.arrows() != 0
            {
                if self.player_resources_mut().decrement_arrows() == 0 {
                    self.hud_refresh_icon();
                }
            } else {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
                self.ancilla_sfx2_near(60);
            }
        }

        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_item_in_hand_bits(0x10);
        if self.game_state.player.follower_link.button_b_frames() >= 9 {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
    }

    pub(super) fn link_item_boomerang(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
                || self.game_state.minigame.flag_boomerang_in_place() != 0
            {
                return;
            }
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().set_item_in_hand(0x80);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(7);
            let s0 = self.ancilla_add_boomerang(5, 0);
            if self.game_state.player.follower_link.button_b_frames() >= 9 {
                self.link_reset_boomerang_y_stuff();
                return;
            }
            if s0 == 0 {
                let last_direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                self.follower_link_state_mut()
                    .set_last_direction(last_direction);
            } else {
                self.follower_link_state_mut().set_direction_lock_bits(1);
            }
        } else {
            self.follower_link_state_mut().set_direction_lock_bits(1);
        }

        if self.game_state.player.follower_link.has_item_in_hand() {
            self.halt_link_when_using_items();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(5);
            self.follower_link_state_mut()
                .increment_action_handler_timer();
            if self.game_state.player.follower_link.action_handler_timer() != 2 {
                return;
            }
        }
        self.link_reset_boomerang_y_stuff();
    }

    pub(super) fn link_reset_boomerang_y_stuff(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
            self.follower_link_state_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_handle_a_press(&mut self) {
        self.follower_link_state_mut()
            .set_sprite_pickup_flag_cached(0);
        if self.game_state.player.follower_link.has_item_in_hand()
            || self.game_state.player.follower_link.position_mode_has(0x1f)
            || self
                .game_state
                .player
                .follower_link
                .player_pose_draw_counter()
                != 0
        {
            return;
        }
        if self.game_state.player.follower_link.button_b_frames() < 9
            && (self.game_state.player.follower_link.button_mask_b_y() & 0x80) != 0
        {
            return;
        }

        let mut action = self.game_state.player.follower_link.tile_action_index();
        if !self.game_state.player.follower_link.has_action_state()
            && !self
                .game_state
                .player
                .follower_link
                .has_grabbing_wall_state()
        {
            if !self.link_check_new_a_press() {
                self.follower_link_state_mut().set_y_button_action_flags(0);
                return;
            }
            if self
                .game_state
                .player
                .follower_link
                .needs_pull_for_rupees_sprite()
                && self.game_state.player.follower_link.facing() == 0
            {
                action = 7;
            } else if self
                .game_state
                .player
                .follower_link
                .is_near_moveable_statue()
            {
                action = 6;
            } else {
                let mut attempt_action = false;
                if self.game_state.player.follower_link.ancilla_pickup_flag() == 0 {
                    if self.game_state.player.follower_link.sprite_pickup_flag() == 0 {
                        action = self.link_handle_liftables();
                        attempt_action = true;
                    } else {
                        let pickup_flag = self.game_state.player.follower_link.sprite_pickup_flag();
                        self.follower_link_state_mut()
                            .set_sprite_pickup_flag_cached(pickup_flag);
                    }
                }
                if !attempt_action {
                    if self.game_state.player.follower_link.button_b_frames() != 0 {
                        self.link_reset_sword_and_item_usage();
                    }
                    if self
                        .game_state
                        .player
                        .follower_link
                        .has_item_or_position_mode()
                    {
                        self.follower_link_state_mut().clear_item_in_hand();
                        self.follower_link_state_mut().clear_position_mode();
                        self.link_reset_boomerang_y_stuff();
                        self.minigame_state_mut().clear_flag_boomerang_in_place();
                        if self.ancilla_slot_view(0).ancilla_type() == 5 {
                            self.ancilla_slot_view_mut(0).set_ancilla_type(0);
                        }
                    }
                    action = 1;
                }
            }

            if action as usize >= LINK_APRESS_BASIC_ABILITY_BITMASKS.len()
                || (LINK_APRESS_BASIC_ABILITY_BITMASKS[action as usize]
                    & self.game_state.inventory.player_resources.ability_flags())
                    == 0
            {
                self.follower_link_state_mut().set_y_button_action_flags(0);
                return;
            }
            self.follower_link_state_mut().set_tile_action_index(action);
            self.link_a_press_perform_basic(action.wrapping_mul(2));
        }

        let action_index = self.game_state.player.follower_link.tile_action_index();
        self.follower_link_state_mut()
            .set_cached_tile_action_index(action_index);
        match self.game_state.player.follower_link.tile_action_index() {
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
        if self.game_state.player.follower_link.y_button_action_flags() & 0x80 != 0
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.game_state.player.follower_link.filtered_joypad_l() & 0x80 == 0
        {
            return false;
        }
        self.follower_link_state_mut()
            .add_y_button_action_flag_bits(0x80);
        true
    }

    pub(super) fn link_perform_dash(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .has_somaria_platform_state()
            || (self.game_state.player.follower_link.sprite_pickup_flag()
                | self.game_state.player.follower_link.ancilla_pickup_flag())
                != 0
            || self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            return;
        }
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().set_dash_countdown(29);
        self.follower_link_state_mut().set_dash_counter(64);
        self.follower_link_state_mut().set_handler_state(17);
        self.follower_link_state_mut().start_running();
        let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
        self.follower_link_state_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.follower_link_state_mut().clear_state_bits();
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();

        let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
        if self.game_state.sprites.follower_runtime.indicator()
            == DASH_FOLLOWER_SLOWDOWN_INDICATORS[follower]
        {
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_state_mut().set_reacquire_timer(64);
        }
    }

    pub(super) fn link_perform_grab(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 != 0
            && self.game_state.player.follower_link.button_b_frames() >= 9
        {
            return;
        }
        self.follower_link_state_mut().set_grabbing_wall(1);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(0);
        self.follower_link_state_mut().clear_item_action_step_var();
    }

    pub(super) fn link_perform_read(&mut self) {
        let message = if self.game_state.world.location.is_indoors() {
            self.Dungeon_GetTeleMsg(self.game_state.world.location.dungeon_room() as usize)
        } else if self.game_state.inventory.save_progress.progress_indicator() < 2 {
            0x003a
        } else {
            self.asset_u16(
                110,
                u16::from(self.game_state.world.location.overworld_screen_index()) as usize,
            )
        };
        self.dialogue_message_index_mut().set_value(message);
        self.main_show_text_message();
        self.follower_link_state_mut().set_y_button_action_flags(0);
    }

    pub(super) fn link_perform_open_chest(&mut self) {
        if self.game_state.player.follower_link.facing() != 0
            || self.game_state.player.follower_link.item_receipt_method() != 0
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return;
        }

        self.follower_link_state_mut().set_y_button_action_flags(0);
        let Some((mut item, chest_position)) = self
            .OpenChestForItemResult(self.game_state.player.tile_detection.interacting_tile() as u8)
        else {
            self.follower_link_state_mut().set_item_receipt_method(0);
            return;
        };

        self.follower_link_state_mut().set_item_receipt_method(1);

        if let Some(&alternate) = LINK_PERFORM_OPEN_CHEST_RECEIVE_ITEM_ALTERNATES.get(item as usize)
        {
            if alternate != 0xff {
                let ram_addr = player_memory_location_to_give_item_to(item);
                if self.item_memory_value(ram_addr) != 0 {
                    item = alternate;
                }
            }
        }

        let caller = if self.ground_apress_defers_atomic_item_receipt {
            ItemReceiptCaller::GroundApress
        } else {
            ItemReceiptCaller::AtomicCaller
        };
        let _ = self.link_receive_item_from(item, chest_position, caller);
    }

    pub(super) fn link_perform_statue_drag(&mut self) {
        self.follower_link_state_mut().set_grabbing_wall(2);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(0);
        self.follower_link_state_mut().clear_item_action_step_var();
    }

    pub(super) fn link_perform_rupee_pull(&mut self) {
        if self.game_state.player.follower_link.facing() != 0 {
            return;
        }
        self.link_reset_properties_a();
        self.follower_link_state_mut().set_grabbing_wall(2);
        self.follower_link_state_mut().set_direction_lock_bits(2);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(0);
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().set_handler_state(29);
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_button_mask_b_y(0);
    }

    pub(super) fn search_for_byrna_spark(&self) -> bool {
        if self.game_state.player.follower_link.position_mode_has(8) {
            return false;
        }
        (0..=4)
            .rev()
            .any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x31)
    }

    pub(super) fn link_permission_for_slosh_sounds(&self) -> bool {
        if self.game_state.player.follower_link.direction() & 0x0f == 0 {
            return true;
        }
        if self.game_state.player.follower_link.handler_state() != 17 {
            self.game_state.frame.frame_counter & 0x0f != 0
        } else {
            self.game_state.frame.frame_counter & 0x07 != 0
        }
    }

    pub(super) fn link_a_press_lift_carry_throw(&mut self) {
        if !self.game_state.player.follower_link.has_action_state() {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .picking_throw_state_has(2)
            && self.game_state.player.follower_link.y_button_action_timer() >= 5
        {
            self.follower_link_state_mut().set_y_button_action_timer(5);
        }
        if self
            .game_state
            .player
            .follower_link
            .has_picking_throw_state()
        {
            self.halt_link_when_using_items();
        }
        if self.game_state.player.follower_link.is_lift_throw_primed() {
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_frame_change_counter();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
        }
        self.follower_link_state_mut()
            .decrement_y_button_action_timer();
        if self.game_state.player.follower_link.y_button_action_timer() != 0 {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .picking_throw_state_has(2)
        {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            if self.game_state.player.follower_link.handler_state() == 24 {
                self.follower_link_state_mut().clear_handler_state();
            }
        } else if self.game_state.player.follower_link.action_handler_timer() != 0 {
            if self
                .game_state
                .player
                .follower_link
                .action_handler_timer()
                .wrapping_add(1)
                != 9
            {
                self.follower_link_state_mut()
                    .increment_action_handler_timer();
                let timer = self.game_state.player.follower_link.action_handler_timer() as usize;
                self.follower_link_state_mut().set_y_button_action_timer(
                    LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_ACTION_TIMERS[timer],
                );
                self.follower_link_state_mut().set_y_button_action_step(
                    LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_ACTION_STEPS[timer],
                );
                if self.game_state.player.follower_link.action_handler_timer() == 6 {
                    self.dungeon_secret_scratch_mut().clear_pending_kind();
                    let (what, x, y) = if self.game_state.world.location.is_indoors() {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.Dungeon_LiftAndReplaceLiftable(&mut pt);
                        (what, pt.x, pt.y)
                    } else {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.Overworld_HandleLiftableTiles(&mut pt);
                        (what, pt.x, pt.y)
                    };
                    self.follower_link_state_mut().set_handler_state(24);
                    self.follower_link_state_mut().set_sprite_pickup_flag(1);
                    self.sprite_spawn_throwable_terrain((what & 0x0f).wrapping_add(1), x, y);
                    self.follower_link_state_mut()
                        .clear_filtered_joypad_l_bits(0x80);
                }
                return;
            }
        } else {
            if self.game_state.player.follower_link.y_button_action_step() as usize
                >= LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_SEQUENCE_TIMERS.len() - 1
            {
                return;
            }
            let y_button_action_step = self
                .game_state
                .player
                .follower_link
                .y_button_action_step()
                .wrapping_add(1);
            self.follower_link_state_mut()
                .set_y_button_action_step(y_button_action_step);
            self.follower_link_state_mut().set_y_button_action_timer(
                LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_SEQUENCE_TIMERS
                    [y_button_action_step as usize],
            );
            if self.game_state.player.follower_link.y_button_action_step() != 3 {
                return;
            }
        }
        self.follower_link_state_mut().clear_picking_throw_state();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_a_press_pull_object(&mut self) {
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        let facing = self.game_state.player.follower_link.facing_index();
        if LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_DIRS[facing]
            & self.game_state.player.follower_link.joypad1h_last()
            == 0
        {
            self.follower_link_state_mut().clear_item_action_step_var();
            let step = self.game_state.player.follower_link.item_action_step_var() as usize;
            self.follower_link_state_mut()
                .set_y_button_action_step(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_STEPS[step]);
            self.follower_link_state_mut()
                .set_y_button_action_timer(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_TIMER[step]);
        } else {
            self.follower_link_state_mut()
                .decrement_y_button_action_timer();
            if (self.game_state.player.follower_link.y_button_action_timer() as i8) < 0 {
                let step = self
                    .follower_link_state_mut()
                    .advance_item_action_step_var_wrapping_7_to_1()
                    as usize;
                self.follower_link_state_mut()
                    .set_y_button_action_step(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_STEPS[step]);
                self.follower_link_state_mut()
                    .set_y_button_action_timer(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_TIMER[step]);
            }
        }
        if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_a_press_statue_drag(&mut self) {
        self.follower_link_state_mut().set_speed_setting(20);
        let j = self.game_state.player.follower_link.joypad1h_last()
            & LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_DIRS
                [self.game_state.player.follower_link.facing_index()];
        if j == 0 {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_item_action_step_var();
        } else {
            self.follower_link_state_mut().set_direction(j);
            self.follower_link_state_mut()
                .decrement_y_button_action_timer();
            if (self.game_state.player.follower_link.y_button_action_timer() as i8) >= 0 {
                if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                    self.link_a_press_statue_drag_release();
                }
                return;
            }
            self.follower_link_state_mut()
                .advance_item_action_step_var_wrapping_7_to_1();
        }
        let step = self.game_state.player.follower_link.item_action_step_var() as usize;
        self.follower_link_state_mut()
            .set_y_button_action_step(LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_ANIM_STEPS[step]);
        self.follower_link_state_mut()
            .set_y_button_action_timer(LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_ANIM_TIMER[step]);
        if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
            self.link_a_press_statue_drag_release();
        }
    }

    fn link_a_press_statue_drag_release(&mut self) {
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_near_moveable_statue();
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().clear_grabbing_wall();
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_item_bombs(&mut self) {
        if self.game_state.player.follower_link.doorway_state() != 0
            || self.game_state.sprites.follower_runtime.indicator() == 13
            || !self.check_y_button_press()
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        let limit = if self
            .game_state
            .enhanced_features
            .has(LINK_ITEM_BOMBS_FEATURES0_MORE_ACTIVE_BOMBS)
        {
            3
        } else {
            1
        };
        self.ancilla_add_bomb(7, limit);
        self.follower_link_state_mut().clear_item_in_hand();
    }

    pub(super) fn link_item_book(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0
            || self.game_state.player.follower_link.doorway_state() != 0
            || !self.check_y_button_press()
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        if self
            .game_state
            .player
            .follower_link
            .item_pickup_in_progress()
        {
            self.link_perform_desert_prayer();
        } else {
            self.ancilla_sfx2_near(60);
        }
    }

    pub(super) fn link_item_bottle(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        let btidx = self
            .game_state
            .inventory
            .player_resources
            .equipped_bottle_index()
            .wrapping_sub(1) as usize;
        if btidx >= 4 {
            return;
        }
        let bottle = self.game_state.inventory.items.bottle(btidx);
        if bottle == 0 {
            return;
        }
        if bottle < 3 {
            self.ancilla_sfx2_near(60);
        } else if bottle == 3 {
            if self.game_state.inventory.player_resources.health_capacity()
                == self.game_state.inventory.player_resources.current_health()
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.follower_link_state_mut().clear_item_in_hand();
            let main_module = self.game_state.frame.main_module;
            self.set_submodule(4);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 4 {
            if self.game_state.player.follower_link.magic_power() == 128 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.follower_link_state_mut().clear_item_in_hand();
            let main_module = self.game_state.frame.main_module;
            self.set_submodule(8);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 5 {
            if self.game_state.inventory.player_resources.health_capacity()
                == self.game_state.inventory.player_resources.current_health()
                && self.game_state.player.follower_link.magic_power() == 128
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.follower_link_state_mut().clear_item_in_hand();
            let main_module = self.game_state.frame.main_module;
            self.set_submodule(9);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 6 {
            self.follower_link_state_mut().clear_item_in_hand();
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
        let main_module = self.game_state.frame.main_module;
        self.set_submodule(5);
        self.set_saved_module_for_menu(main_module);
        self.set_main_module(14);
        self.set_modal_pause_flag(1);
        self.follower_link_state_mut().set_y_button_action_timer(22);
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_state_bits(2);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.set_ambient_sound_effect(17);
        self.set_music_control(242);
    }

    pub(super) fn link_item_lamp(&mut self) {
        if self.game_state.player.follower_link.doorway_state() != 0 || !self.check_y_button_press()
        {
            return;
        }
        if self.game_state.inventory.items.torch() != 0 && self.link_check_magic_cost(6) {
            self.ancilla_add_magic_powder(0x1a, 0);
            self.dungeon_light_torch();
            self.ancilla_add_lamp_flame(0x2f, 2);
        }
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().set_button_b_frames(0);
        self.follower_link_state_mut().clear_direction_lock();
    }

    pub(super) fn link_item_powder(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            if self.game_state.inventory.items.mushroom() != 2 {
                self.ancilla_sfx2_near(60);
                self.finish_powder_item();
                return;
            }
            if !self.link_check_magic_cost(2) {
                self.finish_powder_item();
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_POWDER_MUSHROOM_TIMER[0]);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
            self.follower_link_state_mut().set_item_in_hand(0x40);
        }

        self.follower_link_state_mut()
            .clear_movement_velocity_and_direction();
        self.follower_link_state_mut().clear_movement_subpixels();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_ITEM_POWDER_MUSHROOM_TIMER[step]);
        if self.game_state.player.follower_link.action_handler_timer() == 4 {
            self.ancilla_add_magic_powder(26, 0);
        }
        if self.game_state.player.follower_link.action_handler_timer() == 9 {
            if self.game_state.frame.submodule == 0 {
                self.tile_detect_main_handler(1);
            }
            self.finish_powder_item();
        }
    }

    pub(super) fn link_item_shovel_and_flute(&mut self) {
        if self.game_state.inventory.items.flute() == 1 {
            self.link_item_shovel();
        } else if self.game_state.inventory.items.flute() != 0 {
            self.link_item_flute();
        }
    }

    pub(super) fn link_item_shovel(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY[0]);
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_position_mode(1);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        let step = self
            .follower_link_state_mut()
            .increment_item_action_step_var() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY[step]);
        self.follower_link_state_mut()
            .set_action_handler_timer(LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY2[step]);

        if self.game_state.player.follower_link.action_handler_timer() == 1 {
            self.tile_detect_main_handler(2);
            if self.game_state.world.transient.overworld_hole_tilemap_pos() != 0 {
                self.ancilla_sfx3_near(27);
                self.ancilla_add_dug_up_flute(54, 0);
            }
            if (self.game_state.player.tile_detection.thick_grass()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath())
                & 1
                == 0
            {
                self.ancilla_add_hit_stars(22, 0);
                self.ancilla_sfx2_near(5);
            } else {
                self.ancilla_add_shovel_dirt(23, 0);
                if self.game_state.minigame.is_archer_or_shovel_game() != 0 {
                    self.digging_game_guy_attempt_prize_spawn();
                }
                self.ancilla_sfx2_near(18);
            }
        }

        if self.game_state.player.follower_link.item_action_step_var() == 3 {
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
            self.follower_link_state_mut()
                .set_button_mask_b_y(button_mask_b_y);
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_item_flute(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0 {
            self.follower_link_state_mut().decrement_flute_countdown();
            if self.game_state.player.follower_link.flute_countdown() != 0 {
                return;
            }
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
        }
        if !self.check_y_button_press() {
            return;
        }
        self.follower_link_state_mut().set_flute_countdown(128);
        self.ancilla_sfx2_near(19);
        if self.game_state.world.location.is_indoors()
            || u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 != 0
            || self.game_state.frame.main_module == 11
        {
            return;
        }
        if (0..5).any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x27) {
            return;
        }
        if self.game_state.inventory.items.flute() == 2 {
            let screen = u16::from(self.game_state.world.location.overworld_screen_index());
            let y = self.game_state.player.follower_link.y();
            let x = self.game_state.player.follower_link.x();
            if screen == 0x18 && (0x760..0x7e0).contains(&y) && (0x1cf..0x230).contains(&x) {
                self.set_submodule(45);
                self.ancilla_add_exploding_weather_vane(55, 0);
            }
        } else {
            self.ancilla_add_duck_take_off(39, 4);
            self.follower_link_state_mut()
                .clear_pull_for_rupees_sprite_need();
        }
    }

    pub(super) fn link_handle_y_item(&mut self) {
        if self.game_state.player.follower_link.button_b_frames() != 0
            && self.game_state.player.follower_link.button_b_frames() < 9
        {
            return;
        }

        let mut item = self.game_state.player.follower_link.current_item_y();
        if self.game_state.player.follower_link.is_bunny_mirror() && item != 11 && item != 20 {
            return;
        }

        if self.game_state.minigame.is_archer_or_shovel_game() != 0
            && !self.game_state.player.follower_link.is_bunny_mirror()
        {
            if self.game_state.minigame.is_archer_or_shovel_game() == 2 {
                self.link_item_bow();
            } else {
                self.link_item_shovel();
            }
            return;
        }

        let old_down = self.game_state.player.follower_link.joypad1h_last();
        let old_pressed = self.game_state.player.follower_link.filtered_joypad_h();
        let old_bottle = self
            .game_state
            .inventory
            .player_resources
            .equipped_bottle_index();
        if !self.game_state.player.follower_link.has_item_in_hand()
            && !self.game_state.player.follower_link.has_position_mode()
            && old_down & 0x40 == 0
        {
            let btn_index = self.get_current_item_button_index();
            if btn_index != 0 {
                let hud_item = self
                    .game_state
                    .inventory
                    .save_progress
                    .hud_current_item_slot(btn_index);
                if hud_item != 0 {
                    if hud_item >= 21 {
                        self.player_resources_mut()
                            .set_equipped_bottle_index(hud_item - 20);
                    }
                    item = self.hud_lookup_inventory_item(hud_item);
                    self.follower_link_state_mut()
                        .set_joypad1h_last(old_down | 0x40);
                    if self.game_state.player.follower_link.filtered_joypad_l()
                        & LINK_ITEM_Y_BUTTON_BUTTON_INDEX_KEYS[btn_index]
                        != 0
                    {
                        self.follower_link_state_mut()
                            .set_filtered_joypad_h(old_pressed | 0x40);
                    }
                }
            }
        }

        if item != self.game_state.player.follower_link.current_item_active() {
            if self.game_state.player.follower_link.current_item_active() == 8
                && self.game_state.inventory.items.flute() & 2 != 0
            {
                self.follower_link_state_mut()
                    .clear_button_mask_b_y_bits(0x40);
            }
            if self.game_state.player.follower_link.current_item_active() == 19
                && self.game_state.player.follower_link.is_cape_active()
            {
                self.link_force_unequip_cape();
            }
        }

        if !self.game_state.player.follower_link.has_item_in_hand()
            && !self.game_state.player.follower_link.has_position_mode()
        {
            self.follower_link_state_mut().set_current_item_active(item);
        }
        if matches!(
            self.game_state.player.follower_link.current_item_active(),
            5 | 6
        ) {
            let rod = self.game_state.player.follower_link.current_item_active() - 4;
            self.follower_link_state_mut().set_selected_rod(rod);
        }

        match self.game_state.player.follower_link.current_item_active() {
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
                self.game_state.player.follower_link.current_item_active()
            ),
        }

        self.follower_link_state_mut().set_joypad1h_last(old_down);
        self.follower_link_state_mut()
            .set_filtered_joypad_h(old_pressed);
        self.player_resources_mut()
            .set_equipped_bottle_index(old_bottle);
    }

    pub(super) fn link_item_ether(&mut self) {
        self.start_medallion_item(
            8,
            LINK_ITEM_ETHER_ETHER_ANIM_DELAYS[0],
            LINK_ITEM_ETHER_ETHER_ANIM_STATES[0],
            None,
        );
    }

    pub(super) fn link_item_bombos(&mut self) {
        self.start_medallion_item(
            9,
            LINK_ITEM_BOMBOS_BOMBOS_ANIM_DELAYS[0],
            LINK_ITEM_BOMBOS_BOMBOS_ANIM_STATES[0],
            None,
        );
    }

    pub(super) fn link_item_quake(&mut self) {
        self.start_medallion_item(
            10,
            LINK_ITEM_QUAKE_QUAKE_ANIM_DELAYS[0],
            LINK_ITEM_QUAKE_QUAKE_ANIM_STATES[0],
            Some(()),
        );
    }

    pub(super) fn link_state_using_ether(&mut self) {
        self.increment_modal_pause_flag();
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 4
        {
            self.ancilla_sfx3_near(35);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 9
        {
            self.ancilla_sfx2_near(44);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 12
        {
            self.follower_link_state_mut()
                .set_spin_animation_step_counter(10);
        }

        let step = self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter() as usize;
        let delays = if self
            .game_state
            .enhanced_features
            .has(LINK_STATE_USING_ETHER_FEATURES0_DIM_FLASHES)
        {
            LINK_STATE_USING_ETHER_ETHER_ANIM_DELAYS_NO_FLASH
        } else {
            LINK_STATE_USING_ETHER_ETHER_ANIM_DELAYS
        };
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(delays[step]);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(LINK_STATE_USING_ETHER_ETHER_ANIM_STATES[step]);
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_sound_latch()
            == 0
            && self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 10
        {
            self.follower_link_state_mut()
                .set_spin_attack_sound_latch(1);
            self.ancilla_add_ether_spell(24, 0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_state_using_bombos(&mut self) {
        self.increment_modal_pause_flag();
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 4
        {
            self.ancilla_sfx3_near(35);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 10
        {
            self.ancilla_sfx2_near(44);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 20
        {
            self.follower_link_state_mut()
                .set_spin_animation_step_counter(19);
        }
        let step = self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_STATE_USING_BOMBOS_BOMBOS_ANIM_DELAYS[step]);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(LINK_STATE_USING_BOMBOS_BOMBOS_ANIM_STATES[step]);
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_sound_latch()
            == 0
            && self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 19
        {
            self.follower_link_state_mut()
                .set_spin_attack_sound_latch(1);
            self.ancilla_add_bombos_spell(25, 0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_state_using_quake(&mut self) {
        self.increment_modal_pause_flag();
        self.follower_link_state_mut().clear_actual_velocity_xy();

        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 10
        {
            self.follower_link_state_mut()
                .restore_actual_z_velocity_from_mirror();
            self.follower_link_state_mut().restore_z_low_from_mirror();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.player_change_z(2);
            self.link_move_position();
            self.follower_link_state_mut()
                .cache_actual_z_velocity_to_mirror();
            self.follower_link_state_mut().cache_z_low_to_mirror();
            if !self.game_state.player.follower_link.is_z_low_negative() {
                let spin_state =
                    if (self.game_state.player.follower_link.actual_z_velocity() as i8) < 0 {
                        21
                    } else {
                        20
                    };
                self.follower_link_state_mut()
                    .set_state_for_spin_attack(spin_state);
                return;
            }
        } else {
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 4
        {
            self.ancilla_sfx3_near(35);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 10
        {
            self.ancilla_sfx2_near(44);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 11
        {
            self.ancilla_sfx2_near(12);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 12
        {
            self.follower_link_state_mut()
                .set_spin_animation_step_counter(11);
        }
        let step = self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_STATE_USING_QUAKE_QUAKE_ANIM_DELAYS[step]);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(LINK_STATE_USING_QUAKE_QUAKE_ANIM_STATES[step]);
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_sound_latch()
            == 0
            && self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 11
        {
            self.follower_link_state_mut()
                .set_spin_attack_sound_latch(1);
            self.ancilla_add_quake_spell(28, 0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_item_mirror(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if !self.check_y_button_press() {
                return;
            }
            if self.game_state.sprites.follower_runtime.indicator() == 10 {
                self.dialogue_message_index_mut().set_value(289);
                self.main_show_text_message();
                return;
            }
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);

        if self.game_state.player.follower_link.doorway_state() != 0
            || (self
                .game_state
                .player
                .follower_link
                .cheat_walk_through_walls()
                == 0
                && !self
                    .game_state
                    .enhanced_features
                    .has(LINK_ITEM_MIRROR_FEATURES0_MIRROR_TO_DARKWORLD)
                && self.game_state.world.location.is_outdoors()
                && u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 == 0)
        {
            self.ancilla_sfx2_near(60);
            return;
        }

        self.do_sword_interaction_with_tiles_mirror();
    }

    pub(super) fn do_sword_interaction_with_tiles_mirror(&mut self) {
        if self.game_state.world.location.is_indoors() {
            if self.game_state.player.follower_link.is_menu_blocked() {
                return;
            }
            self.Mirror_SaveRoomData();
            if self.game_state.system_signals.sound_effect_1() != 60 {
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(0);
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(1);
            }
            return;
        }
        if self.game_state.frame.main_module == 11 {
            return;
        }
        let screen = u16::from(self.game_state.world.location.overworld_screen_index());
        self.world_palette_theme_mut()
            .set_last_light_vs_dark_world((screen & 0x40) as u8);
        if self
            .game_state
            .world
            .palette_theme
            .last_light_vs_dark_world()
            != 0
        {
            let y = self.game_state.player.follower_link.y();
            let x = self.game_state.player.follower_link.x();
            self.set_bird_travel_destination(15, x, y);
        }
        self.set_submodule(35);
        self.follower_link_state_mut()
            .clear_pull_for_rupees_sprite_need();
        self.follower_link_state_mut().set_whirlpool_trigger();
        self.set_subsubmodule(0);
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_handler_state(20);
    }

    pub(super) fn link_state_crossing_worlds(&mut self) {
        self.link_reset_properties_b();
        self.tile_check_for_mirror_bonk();
        let world_changed =
            (u16::from(self.game_state.world.location.overworld_screen_index()) as u8 & 0x40)
                != self
                    .game_state
                    .world
                    .palette_theme
                    .last_light_vs_dark_world();
        let bonk_bits = self.game_state.player.tile_detection.bonk_bits_low();
        if world_changed && bonk_bits & 0x0c != 0 && Self::bit_sum4(bonk_bits) >= 2 {
            self.start_mirror_transition(44);
            return;
        }

        if Self::bit_sum4(self.game_state.player.tile_detection.deepwater() as u8) >= 2 {
            if self.game_state.player.follower_link.has_flippers() {
                self.link_set_to_deep_water();
                self.follower_link_state_mut().set_handler_state(4);
                self.link_force_unequip_cape_quietly();
                return;
            }
            if world_changed {
                self.start_mirror_transition(44);
                return;
            }
            self.check_ability_to_swim();
        }

        if self.game_state.player.follower_link.is_in_deep_water() {
            self.follower_link_state_mut().clear_deep_water_state();
            self.follower_link_state_mut()
                .set_last_direction_from_swim_flags();
        }
        self.follower_link_state_mut().set_dash_countdown(0);
        self.follower_link_state_mut().clear_running();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().set_button_b_frames(0);
        self.follower_link_state_mut().clear_direction_lock();
        self.swim_acceleration_mut().set_mode(0, 0);
        self.follower_link_state_mut().set_actual_y_velocity(0);
        if world_changed {
            self.memorized_tile_mut().set_count(0);
        }
        let handler_state = if self.game_state.player.follower_link.has_moon_pearl()
            || u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 == 0
        {
            0
        } else {
            23
        };
        self.follower_link_state_mut()
            .set_handler_state(handler_state);
    }

    pub(super) fn handle_followers_after_mirroring(&mut self) {
        self.tile_detect_main_handler(0);
        self.follower_link_state_mut().clear_animation_step();
        match self.game_state.sprites.follower_runtime.indicator() {
            12 | 13 => {
                if self.game_state.sprites.follower_runtime.indicator() == 13 {
                    self.set_super_bomb_indicator_timer(0xfe);
                    self.set_super_bomb_indicator_counter(0);
                }
                if self.game_state.sprites.follower_runtime.dropped() != 0 {
                    self.follower_state_mut().set_dropped(0);
                    self.follower_state_mut().set_indicator(0);
                }
            }
            9 | 10 => self.follower_state_mut().set_indicator(0),
            7 | 8 => {
                self.follower_state_mut().xor_indicator(7 ^ 8);
                self.load_follower_graphics();
                self.ancilla_add_dwarf_poof(0x40, 4);
            }
            _ => {}
        }

        if !self.game_state.player.follower_link.has_moon_pearl() {
            self.ancilla_add_bunny_poof(0x23, 4);
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut().clear_cape_transform_timer();
        } else if self.game_state.player.follower_link.is_cape_active() {
            self.link_force_unequip_cape();
            self.follower_link_state_mut().clear_cape_transform_timer();
        }
    }

    pub(super) fn link_item_hookshot(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0
            || self.game_state.player.follower_link.doorway_state() != 0
            || self.game_state.player.follower_link.defense_flags() & 2 != 0
            || !self.check_y_button_press()
        {
            return;
        }

        self.reset_all_acceleration();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(7);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.follower_link_state_mut().set_position_mode(4);
        self.follower_link_state_mut().set_handler_state(19);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.ancilla_add_hookshot(0x1f, 3);
    }

    pub(super) fn link_state_hookshotting(&mut self) {
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        let hookshot = (0..=4)
            .rev()
            .find(|&i| self.ancilla_slot_view(i).ancilla_type() == 0x1f);
        let Some(_k) = hookshot else {
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
            self.finish_hookshot_state_after_missing_ancilla();
            return;
        };

        if self
            .game_state
            .player
            .follower_link
            .spin_attack_delay_timer()
            != 0
        {
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                < 0
            {
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(0);
            }
        }

        if !self
            .game_state
            .player
            .follower_link
            .has_hookshot_interlock()
        {
            self.follower_link_state_mut()
                .store_safe_return_low_from_current();
            self.follower_link_state_mut().set_y_velocity(0);
            self.follower_link_state_mut().set_x_velocity(0);
            self.link_handle_cardinal_collision();
            return;
        }

        self.follower_link_state_mut()
            .clear_somaria_platform_state();

        let hei = self.game_state.messaging.runtime.effect_index() as usize;
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
            let target_y = y
                .wrapping_add(LINK_STATE_HOOKSHOTTING_HOOKSHOT_TARGET_Y_OFFSETS[dir] as i16 as u16);
            let target_x = x
                .wrapping_add(LINK_STATE_HOOKSHOTTING_HOOKSHOT_TARGET_X_OFFSETS[dir] as i16 as u16);
            let yd = target_y.wrapping_sub(self.game_state.player.follower_link.y()) as i16;
            let mut actual_y_velocity = 0;
            if yd.wrapping_abs() >= 2 {
                actual_y_velocity = LINK_STATE_HOOKSHOTTING_HOOKSHOT_PULL_Y_VELOCITIES[dir];
            }
            let xd = target_x.wrapping_sub(self.game_state.player.follower_link.x()) as i16;
            let mut actual_x_velocity = 0;
            if xd.wrapping_abs() >= 2 {
                actual_x_velocity = LINK_STATE_HOOKSHOTTING_HOOKSHOT_PULL_X_VELOCITIES[dir];
            }
            self.follower_link_state_mut()
                .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
            if (actual_x_velocity | actual_y_velocity) != 0 {
                self.continue_hookshot_drag();
                return;
            }
        }

        self.ancilla_slot_view_mut(hei).set_ancilla_type(0);
        self.follower_state_mut()
            .set_hookshot_release_tail_index_from_tail_write_index();
        self.finish_hookshot_state_without_button_clamp();

        if self.ancilla_slot_view(hei).work_byte_1() != 0 {
            self.follower_link_state_mut()
                .toggle_lower_level_mirror_state();
            self.dungeon_stair_movement_mut().decrement_current_floor();
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                == 0
            {
                let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
                self.dungeon_room_tracking_mut()
                    .set_room_index2(dungeon_room_index);
                self.increment_dungeon_room_index_by(0x10);
            }
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().toggle_lower_level_state();
            }
            self.Dungeon_FlagRoomData_Quadrants();
        }

        self.player_tile_detect_nearby();
        if self.game_state.player.tile_detection.deepwater() & 0x0f != 0
            && !self.game_state.player.follower_link.is_in_deep_water()
        {
            self.link_set_to_deep_water();
            self.ancilla_add_splash(21, 0);
            self.follower_link_state_mut().set_handler_state(4);
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            if self.game_state.world.location.is_indoors() {
                self.follower_link_state_mut().mark_lower_level();
            }
            if self.game_state.player.follower_link.button_b_frames() >= 9 {
                self.follower_link_state_mut().set_button_b_frames(9);
            }
        } else if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0 {
            self.follower_link_state_mut().set_sprite_oam_state_timer(9);
            self.follower_link_state_mut().begin_pit_check();
            self.follower_link_state_mut().set_handler_state(1);
            if self.game_state.player.follower_link.button_b_frames() >= 9 {
                self.follower_link_state_mut().set_button_b_frames(9);
            }
        } else {
            let y = self.game_state.player.follower_link.y();
            let x = self.game_state.player.follower_link.x();
            self.follower_link_state_mut()
                .store_safe_return_position(x, y);
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
        }
    }

    fn continue_hookshot_drag(&mut self) {
        self.link_move_position();
        self.tile_detect_main_handler(5);
        if self.game_state.world.location.is_indoors() {
            let x = (self.game_state.player.tile_detection.vertical_ledge() >> 4)
                | self.game_state.player.tile_detection.vertical_ledge()
                | self.game_state.player.tile_detection.horizontal_ledge();
            if x & 1 != 0 {
                self.follower_link_state_mut()
                    .decrement_hookshot_bg_check_off_timer();
                if (self
                    .game_state
                    .player
                    .follower_link
                    .hookshot_bg_check_off_timer() as i8)
                    < 0
                {
                    self.follower_link_state_mut()
                        .set_hookshot_bg_check_off_timer(3);
                    self.follower_link_state_mut().xor_hookshot_interlock(2);
                }
            }
        }
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if !self
            .game_state
            .player
            .follower_link
            .hookshot_interlock_has(2)
        {
            if self.game_state.player.tile_detection.thick_grass_low() & 1 != 0 {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(2);
                if !self.link_permission_for_slosh_sounds() {
                    self.ancilla_sfx2_near(26);
                }
            } else if (self.game_state.player.tile_detection.shallow_water_low()
                | self.game_state.player.tile_detection.deepwater() as u8)
                & 1
                != 0
            {
                self.follower_link_state_mut()
                    .increment_water_ripple_or_grass_state();
                self.ancilla_sfx2_near(
                    if self.game_state.world.location.overworld_screen_index() == 0x70 {
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
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self
                .game_state
                .player
                .follower_link
                .has_somaria_platform_state()
                || self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }

            let mut did_charge_magic = false;
            if !(0..5).any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x2c) {
                if !self.link_check_magic_cost(4) {
                    if self
                        .game_state
                        .enhanced_features
                        .has(LINK_ITEM_CANE_OF_SOMARIA_FEATURES0_MISC_BUG_FIXES)
                    {
                        self.follower_link_state_mut()
                            .clear_button_mask_b_y_bits(0x40);
                    }
                    return;
                }
                did_charge_magic = true;
            }

            self.follower_link_state_mut()
                .set_item_action_debug_value_2(1);
            if self.ancilla_add_somaria_block(0x2c, 1).is_none() {
                if did_charge_magic
                    || !self
                        .game_state
                        .enhanced_features
                        .has(LINK_ITEM_CANE_OF_SOMARIA_FEATURES0_MISC_BUG_FIXES)
                {
                    self.refund_magic(4);
                }
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS[0]);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().set_position_mode_bits(8);
        }

        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        if step < LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS.len() {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS[step]);
            return;
        }
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut()
            .set_item_action_debug_value_2(0);
        self.follower_link_state_mut().clear_position_mode_bits(8);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_item_cane_of_byrna(&mut self) {
        if self.search_for_byrna_spark() {
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            if !self.link_check_magic_cost(8) {
                self.finish_byrna_item();
                return;
            }
            self.ancilla_add_cane_of_byrna_init_spark(0x30, 0);
            self.follower_link_state_mut()
                .clear_spin_attack_step_counter();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_BYRNA_BYRNA_DELAYS[0]);
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_position_mode(8);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
        }

        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_BYRNA_BYRNA_DELAYS[step]);
        if self.game_state.player.follower_link.action_handler_timer() == 1 {
            self.ancilla_sfx3_near(42);
        } else if self.game_state.player.follower_link.action_handler_timer() == 3 {
            self.finish_byrna_item();
        }
    }

    pub(super) fn link_item_net(&mut self) {
        let base = self.game_state.player.follower_link.facing_index() * 10;
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut()
                .set_action_handler_timer(LINK_ITEM_NET_BUG_NET_TIMERS[base]);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(3);
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_position_mode(16);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
            self.ancilla_sfx2_near(50);
        }

        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_item_action_step_var();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(3);
        if self.game_state.player.follower_link.item_action_step_var() == 10 {
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
            self.follower_link_state_mut()
                .set_button_mask_b_y(button_mask_b_y);
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().disable_oam_offsets();
            return;
        }

        let index = base + self.game_state.player.follower_link.item_action_step_var() as usize;
        self.follower_link_state_mut()
            .set_action_handler_timer(LINK_ITEM_NET_BUG_NET_TIMERS[index]);
    }

    pub(super) fn ancilla_add_dug_up_flute(&mut self, ty: u8, limit: u8) {
        let Some(k) = self.ancilla_add_simple(ty, limit) else {
            return;
        };
        let x_velocity = if self.game_state.player.follower_link.facing() == 4 {
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
            self.follower_link_state_mut()
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
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
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
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);

        let sword_ok = self.game_state.inventory.items.sword_type().wrapping_add(1) & !1 != 0;
        let blocked = self.game_state.player.follower_link.doorway_state() != 0
            || self.game_state.player.follower_link.is_menu_blocked()
            || self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0
            || !sword_ok
            || (self.game_state.sprites.follower_runtime.dropped() != 0
                && self.game_state.sprites.follower_runtime.indicator() == 13);
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

        self.follower_link_state_mut()
            .set_handler_state(player_state);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(delay);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(spin_state);
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut()
            .set_spin_attack_sound_latch(0);
        if quake.is_some() {
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(40);
            self.follower_link_state_mut().clear_z_mirror_low();
        }
        self.ancilla_sfx3_near(35);
    }

    fn start_mirror_transition(&mut self, submodule: u8) {
        self.set_submodule(submodule);
        self.follower_link_state_mut()
            .clear_pull_for_rupees_sprite_need();
        self.follower_link_state_mut().set_whirlpool_trigger();
        self.set_subsubmodule(0);
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_handler_state(20);
    }

    pub(super) fn ancilla_add_hookshot(&mut self, a: u8, y: u8) {
        self.ancilla_add_hookshot_inner(a, y);
    }

    fn ancilla_add_hookshot_inner(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_simple(a, y)?;
        self.follower_link_state_mut().clear_hookshot_interlock();
        self.messaging_state_mut().set_effect_index(k as u8);
        let dir = self.game_state.player.follower_link.facing() >> 1;
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
            hookshot.set_x_velocity(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_X_VEL[dir as usize]);
            hookshot.set_y_velocity(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_Y_VEL[dir as usize]);
        }
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_X_DELTA[dir as usize] as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_Y_DELTA[dir as usize] as u16);
        self.ancilla_set_xy(k, x, y);
        Some(k)
    }

    fn finish_hookshot_state(&mut self) {
        self.finish_hookshot_state_without_button_clamp();
        if self.game_state.player.follower_link.button_b_frames() >= 9 {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
    }

    fn finish_hookshot_state_after_missing_ancilla(&mut self) {
        self.follower_link_state_mut().clear_handler_state();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_position_mode_bits(4);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        if self.game_state.player.follower_link.button_b_frames() >= 9 {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
    }

    fn finish_hookshot_state_without_button_clamp(&mut self) {
        self.follower_link_state_mut().clear_handler_state();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().clear_hookshot_interlock();
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_position_mode_bits(4);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
    }

    fn finish_byrna_item(&mut self) {
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().clear_action_handler_timer();
        let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
        self.follower_link_state_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.follower_link_state_mut().clear_position_mode();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    fn finish_powder_item(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_reset_sword_and_item_usage(&mut self) {
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().and_defense_flags(!9);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_button_b_frames(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x81);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }
}

impl ZeldaState {
    pub(super) fn cache_camera_properties(&mut self) {
        let bg2_x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2_y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.cache_bg2_live_scroll_from(bg2_x, bg2_y);
        self.follower_link_state_mut().cache_current_position();
        let y_start = self.game_state.world.room_bounds.y_bound(0);
        let y_end = self.game_state.world.room_bounds.y_bound(2);
        let x_start = self.game_state.world.room_bounds.x_bound(0);
        let x_end = self.game_state.world.room_bounds.x_bound(2);
        self.set_cached_room_bounds(y_start, y_end, x_start, x_end);
        self.cache_scroll_targets();
        self.cache_camera_scroll();
        self.cache_quadrant_fullsize_state();
        self.follower_link_state_mut().cache_current_quadrants();
        self.follower_link_state_mut().cache_facing();
        self.follower_link_state_mut().cache_lower_level_states();
        let doorway_state = self.game_state.player.follower_link.doorway_state();
        self.set_standing_in_doorway_cached(doorway_state);
        self.dungeon_stair_movement_mut().cache_current_floor();
    }

    pub(super) fn link_main(&mut self) {
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        self.clear_modal_pause_flag();
        if !self.game_state.player.follower_link.is_immobilized() {
            self.link_control_handler();
        }
        self.handle_somaria_and_graves();
    }

    pub(super) fn link_control_handler(&mut self) {
        if self.game_state.player.follower_link.given_damage() != 0 {
            if self.game_state.player.follower_link.is_cape_active() {
                self.follower_link_state_mut().clear_given_damage();
                self.follower_link_state_mut().clear_auxiliary_state();
                self.follower_link_state_mut().set_incapacitated_timer(0);
            } else if self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
                == 0
            {
                let dmg = self.game_state.player.follower_link.given_damage();
                self.follower_link_state_mut().clear_given_damage();
                if self.ancilla_slot_view(0).ancilla_type() == 5
                    && self.game_state.player.follower_link.action_handler_timer() == 0
                    && self
                        .game_state
                        .player
                        .follower_link
                        .spin_attack_delay_timer()
                        != 0
                {
                    self.ancilla_slot_view_mut(0).set_ancilla_type(0);
                    self.minigame_state_mut().clear_flag_boomerang_in_place();
                }
                if self.game_state.player.follower_link.blink_countdown() == 0 {
                    self.follower_link_state_mut().set_blink_countdown(58);
                }
                self.ancilla_sfx2_near(38);
                self.sprite_battle_mut().increment_times_hurt_by_sprites();
                let new_dmg = self
                    .game_state
                    .inventory
                    .player_resources
                    .current_health()
                    .wrapping_sub(dmg);
                let new_dmg = if new_dmg == 0 || new_dmg >= 0xa8 {
                    let main_layers = self.game_state.display.main_screen_layers;
                    let sub_layers = self.game_state.display.sub_screen_layers;
                    self.set_mapbak_tm(main_layers);
                    self.set_mapbak_ts(sub_layers);
                    let main_module = self.game_state.frame.main_module;
                    self.set_saved_module_for_menu(main_module);
                    self.set_main_module(18);
                    self.set_submodule(1);
                    self.follower_link_state_mut().clear_blink_countdown();
                    self.player_resources_mut().set_heart_filler(0);
                    0
                } else {
                    new_dmg
                };
                self.player_resources_mut().set_current_health(new_dmg);
            }
        }
        if self.game_state.player.follower_link.handler_state() != 0 {
            self.player_check_handle_cape_stuff();
        }
        match self.game_state.player.follower_link.handler_state() {
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
            if self.game_state.player.follower_link.handler_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        self.follower_link_state_mut().set_pit_correction_timer(0);
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.handle_link_from1_d();
        } else {
            self.player_handler_00_ground_3();
        }
    }

    pub(super) fn handle_link_from1_d(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_position_mode();
        self.follower_link_state_mut().clear_action_scratch_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut()
            .clear_state_item_and_grab_flags();
        self.follower_link_state_mut().clear_defense_flags();
        self.link_reset_swimming_state();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_z_high();
        if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
            if self.game_state.player.follower_link.is_cape_active() {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(2);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
            self.ancilla_sfx3_near(43);
            self.follower_link_state_mut().set_handler_state(7);
            self.link_state_zapped();
        } else {
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.follower_link_state_mut().set_handler_state(2);
            self.link_state_recoil();
        }
    }

    pub(super) fn link_state_0_f(&mut self) {
        // LinkState_0F is an assert-only unreachable state in the C port.
        panic!("LinkState_0F reached");
    }

    pub(super) fn player_handler_15_hold_item(&mut self) {}

    pub(super) fn link_handle_bunny_transformation(&mut self) -> bool {
        if self.game_state.player.follower_link.temp_bunny_timer() == 0 {
            return false;
        }

        if !self.game_state.player.follower_link.needs_transform_poof() {
            if matches!(
                self.game_state.player.follower_link.handler_state(),
                23 | 28
            ) {
                self.follower_link_state_mut().clear_temp_bunny_timer();
                return false;
            }
            if self
                .game_state
                .player
                .follower_link
                .picking_throw_state_has(2)
            {
                self.follower_link_state_mut().clear_state_bits();
            }
            let preserved_lift_bit = self.game_state.player.follower_link.state_bits() & 0x80;
            self.link_reset_properties_a();
            self.follower_link_state_mut()
                .set_state_bits(preserved_lift_bit);

            for i in 0..5 {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 0x30 | 0x31) {
                    self.ancilla_slot_view_mut(i).set_ancilla_type(0);
                }
            }
            self.link_cancel_dash();
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x14);
            self.follower_link_state_mut().set_cape_transform_timer(20);
            self.follower_link_state_mut().start_bunny_transform_poof();
        }

        if (self.follower_link_state_mut().tick_cape_transform_timer() as i8).is_negative() {
            self.follower_link_state_mut().set_handler_state(28);
            self.follower_link_state_mut().finish_bunny_transform_poof();
            self.load_gear_palettes_bunny();
        }
        true
    }

    pub(super) fn link_state_temporary_bunny(&mut self) {
        let timer = self.game_state.player.follower_link.temp_bunny_timer();
        if timer == 0 {
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x15);
            self.follower_link_state_mut().set_cape_transform_timer(32);
            self.follower_link_state_mut().clear_handler_state();
            self.link_reset_properties_c();
            self.follower_link_state_mut().clear_bunny_transform_flags();
            self.load_actual_gear_palettes();
            self.link_state_default();
        } else {
            self.follower_link_state_mut().decrement_temp_bunny_timer();
            self.player_handler_17_bunny();
        }
    }

    pub(super) fn player_handler_17_bunny(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.follower_link_state_mut().set_pit_correction_timer(0);
        if !self.game_state.player.follower_link.is_in_deep_water() {
            if !self.game_state.player.follower_link.has_auxiliary_state() {
                self.link_temp_bunny_func2();
                return;
            }
            if self.game_state.player.follower_link.has_moon_pearl() {
                self.follower_link_state_mut().clear_bunny_mirror();
            }
        }
        self.link_state_bunny_recache();
    }

    pub(super) fn link_temp_bunny_func2(&mut self) {
        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.follower_link_state_mut().set_z(0xffff);
        self.follower_link_state_mut().set_actual_z_velocity(0xff);
        self.follower_link_state_mut().set_recoil_timer(0);
        if self.game_state.player.follower_link.flag_moving() != 0 {
            self.swim_acceleration_mut().set_max_speed(0, 0x0180);
            self.swim_acceleration_mut().set_max_speed(2, 0x0180);
            self.link_handle_swim_movements();
            return;
        }

        self.reset_all_acceleration();
        self.link_handle_y_item();
        let mut dir = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if dir == 0 {
            dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        }
        if dir == 0 {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().set_last_direction(0);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().and_defense_flags(!9);
            self.follower_link_state_mut().reset_push_fatigue_timer();
            self.follower_link_state_mut().reset_jump_ledge_timer();
        } else {
            self.follower_link_state_mut().set_direction(dir);
            if dir != self.game_state.player.follower_link.last_direction() {
                self.follower_link_state_mut().set_last_direction(dir);
                self.follower_link_state_mut().clear_movement_subpixels();
                self.follower_link_state_mut()
                    .clear_moving_against_diag_tile();
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().reset_push_fatigue_timer();
                self.follower_link_state_mut().reset_jump_ledge_timer();
            }
        }
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_state_holding_big_rock(&mut self) {
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_action_scratch_state();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().set_z_low(0);
            if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
                self.link_reset_sword_and_item_usage();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(2);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.follower_link_state_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.follower_link_state_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        self.follower_link_state_mut().set_z(0xffff);
        self.follower_link_state_mut().set_actual_z_velocity(0xff);
        self.follower_link_state_mut().set_recoil_timer(0);
        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.follower_link_state_mut()
                .clear_lift_throw_scratch_state();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        self.link_handle_a_press();
        let dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        if dir == 0 {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().set_last_direction(0);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().and_defense_flags(!9);
            self.follower_link_state_mut().reset_push_fatigue_timer();
            self.follower_link_state_mut().reset_jump_ledge_timer();
        } else {
            self.follower_link_state_mut().set_direction(dir);
            if dir != self.game_state.player.follower_link.last_direction() {
                self.follower_link_state_mut().set_last_direction(dir);
                self.follower_link_state_mut().clear_movement_subpixels();
                self.follower_link_state_mut()
                    .clear_moving_against_diag_tile();
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().reset_push_fatigue_timer();
                self.follower_link_state_mut().reset_jump_ledge_timer();
            }
        }
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn handle_somaria_and_graves(&mut self) {
        if self.game_state.world.location.is_outdoors()
            && self.game_state.player.follower_link.hookshot_grave_latch()
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
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_given_damage();
        let frames = self
            .follower_link_state_mut()
            .decrement_button_b_frames_word();
        if (frames as i16).is_negative() {
            self.follower_link_state_mut().set_button_b_frames_word(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
        } else if frames == 0xbf {
            self.follower_link_state_mut().force_hold_sword_up();
        } else if frames == 160 {
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut().set_x(0x06b0);
            self.follower_link_state_mut().set_y(0x0037);
            self.ancilla_add_ether_spell(0x18, 0);
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 0, 4);
            self.follower_link_state_mut().immobilize();
            self.follower_link_state_mut().clear_menu_block();
        }
    }

    pub(super) fn link_state_receiving_bombos(&mut self) {
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_given_damage();
        let frames = self
            .follower_link_state_mut()
            .decrement_button_b_frames_word();
        if (frames as i16).is_negative() {
            self.follower_link_state_mut().set_button_b_frames_word(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
        } else if frames == 223 {
            self.follower_link_state_mut().force_hold_sword_up();
        } else if frames == 160 {
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut().set_x(0x0378);
            self.follower_link_state_mut().set_y(0x0eb0);
            self.ancilla_add_bombos_spell(0x19, 0);
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 5, 4);
            self.follower_link_state_mut().immobilize();
        }
    }

    pub(super) fn ether_tablet_start_cutscene(&mut self) {
        self.follower_link_state_mut()
            .set_button_b_frames_word(0x00c0);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_handler_state(25);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.follower_link_state_mut().set_menu_block_flag(1);
    }

    pub(super) fn bombos_tablet_start_cutscene(&mut self) {
        self.follower_link_state_mut()
            .set_button_b_frames_word(0x00e0);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_handler_state(26);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.set_custom_spell_animation_active();
    }

    pub(super) fn link_state_reading_desert_tablet(&mut self) {
        let button_b_frames = self
            .game_state
            .player
            .follower_link
            .button_b_frames()
            .wrapping_sub(1);
        self.follower_link_state_mut()
            .set_button_b_frames(button_b_frames);
        if self.game_state.player.follower_link.button_b_frames() == 0 {
            self.follower_link_state_mut().clear_handler_state();
            self.link_perform_desert_prayer();
        }
    }

    pub(super) fn link_state_pits(&mut self) {
        self.follower_link_state_mut().set_direction(0);
        if self.game_state.player.follower_link.pit_correction_active() && {
            self.follower_link_state_mut()
                .increment_pit_correction_timer();
            self.game_state.player.follower_link.pit_correction_timer() == 0x20
        } {
            self.follower_link_state_mut().set_pit_correction_timer(31);
        } else {
            if !self.game_state.player.follower_link.is_running() {
                if !self
                    .game_state
                    .player
                    .follower_link
                    .is_in_auxiliary_state(1)
                {
                    let direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                    self.follower_link_state_mut().set_direction(direction);
                }
                self.link_state_pits_after_aux_state();
                return;
            }
            if self.game_state.player.follower_link.dash_countdown() != 0
                && (!self
                    .game_state
                    .enhanced_features
                    .has(LINK_STATE_PITS_FEATURES0_MISC_BUG_FIXES)
                    || self.game_state.player.follower_link.joypad1l_last() & 0x80 != 0)
            {
                self.link_state_dashing();
                return;
            }
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f != 0
                && (self.game_state.player.follower_link.joypad1h_last()
                    & 0x0f
                    & self.game_state.player.follower_link.direction())
                    == 0
            {
                self.link_cancel_dash();
                if !self
                    .game_state
                    .player
                    .follower_link
                    .is_in_auxiliary_state(1)
                {
                    let direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                    self.follower_link_state_mut().set_direction(direction);
                }
            }
        }

        self.link_state_pits_after_aux_state();
    }

    pub(super) fn handle_dungeon_landing_from_pit(&mut self) {
        self.link_oam_main();
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        if self.game_state.frame.submodule == 7 {
            self.follower_link_state_mut().set_visibility_status(0);
        }
        if self.game_state.frame.frame_counter & 3 == 0 {
            self.follower_link_state_mut().advance_pit_data_index();
            if self.game_state.player.follower_link.pit_data_index() == 10 {
                self.follower_link_state_mut().set_pit_data_index(6);
            }
        }
        self.follower_link_state_mut().set_direction(4);
        self.link_handle_velocity();

        let link_y = self.game_state.player.follower_link.y();
        let target_y = self.game_state.player.tile_detection.y();
        if (link_y as i16).is_negative() && !(target_y as i16).is_negative() {
            if (!link_y).wrapping_add(1).wrapping_add(target_y) < 0x8000 {
                return;
            }
        } else if target_y >= link_y {
            return;
        }

        if self
            .game_state
            .enhanced_features
            .has(HANDLE_DUNGEON_LANDING_FROM_PIT_FEATURES0_MISC_BUG_FIXES)
        {
            self.follower_link_state_mut()
                .clear_about_to_jump_off_ledge();
        }
        self.follower_link_state_mut().set_y(target_y);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().clear_speed_modifier();
        self.follower_link_state_mut().clear_pit_data_index();
        self.follower_link_state_mut().clear_near_pit_state();
        self.follower_link_state_mut().set_speed_setting(0);
        self.set_subsubmodule(0);
        self.set_submodule(0);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        if self.game_state.sprites.follower_runtime.indicator() != 0
            && self.game_state.sprites.follower_runtime.indicator() != 3
        {
            self.follower_state_mut().set_appearance_none_flag(0);
            if self.game_state.sprites.follower_runtime.indicator() == 13 {
                self.follower_state_mut().set_indicator(0);
                self.set_super_bomb_indicator_timer(0);
                self.set_super_bomb_indicator_counter(0);
                self.follower_state_mut().set_dropped(0);
            } else {
                self.follower_initialize();
            }
        }
        self.tile_detect_main_handler(0);
        if self.game_state.player.tile_detection.shallow_water() & 1 != 0 {
            self.ancilla_sfx2_near(0x24);
        }
        self.player_tile_detect_nearby();
        if self.game_state.system_signals.sound_effect_1() & 0x3f != 0x24 {
            self.ancilla_sfx2_near(0x21);
        }

        if self.game_state.dungeon.room_load.header_collision_2() == 2
            && self.game_state.player.tile_detection.water_staircase() & 0x0f != 0
        {
            self.set_player_layer_collision_flags(
                crate::game_state::constants::player::LAYER_COLLISION_BOTH,
            );
        }
        if self.game_state.player.tile_detection.deepwater() & 0x0f == 0x0f {
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().mark_lower_level();
            self.ancilla_add_splash(0x15, 1);
            self.follower_link_state_mut().set_handler_state(4);
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().set_speed_setting(0);
        } else {
            let handler_state = if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0 {
                1
            } else {
                0
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
        }
    }

    pub(super) fn link_state_spin_attack(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.game_state.player.follower_link.has_auxiliary_state() {
            for i in (0..5).rev() {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 0x2a | 0x2b) {
                    self.ancilla_slot_view_mut(i).set_ancilla_type(0);
                }
            }
            self.follower_link_state_mut().clear_z_high();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut().set_state_for_spin_attack(0);
            self.follower_link_state_mut()
                .clear_spin_animation_step_counter();
            self.follower_link_state_mut().set_speed_setting(0);
            if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
                if self.game_state.player.follower_link.is_cape_active() {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(2);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.follower_link_state_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.follower_link_state_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.link_handle_recoil_and_timer(false);
        } else {
            self.follower_link_state_mut().set_direction(0);
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.follower_link_state_mut().set_handler_state(3);
            self.follower_link_state_mut().clear_pit_correction();
            self.handle_indoor_camera_and_doors();
        }

        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 2
        {
            self.ancilla_sfx3_near(35);
        }
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 12
        {
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().set_state_for_spin_attack(0);
            self.follower_link_state_mut()
                .clear_spin_animation_step_counter();
            if self.game_state.player.follower_link.handler_state() != 30 {
                let button_mask_b_y = if self.game_state.player.follower_link.button_b_frames() != 0
                {
                    self.game_state.player.follower_link.joypad1h_last() & 0x80
                } else {
                    0
                };
                self.follower_link_state_mut()
                    .set_button_mask_b_y(button_mask_b_y);
            }
            self.follower_link_state_mut().clear_handler_state();
        } else {
            let idx = self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                .wrapping_add(self.game_state.player.follower_link.spin_offsets())
                as usize;
            self.follower_link_state_mut()
                .set_state_for_spin_attack(LINK_SPIN_GRAPHICS_BY_DIR[idx]);
            let delay = LINK_SPIN_DELAYS[self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter() as usize];
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(delay);
            self.tile_detect_main_handler(8);
        }
    }

    pub(super) fn link_hop_hopping_south_ow(&mut self) {
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(1);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if self.game_state.player.follower_link.incapacitated_timer() == 0
            && self
                .game_state
                .player
                .follower_link
                .actual_z_velocity_mirror()
                == 0
        {
            self.ancilla_sfx2_near(32);
            self.link_hop_find_tile_to_land_on_south();
            if self.game_state.world.location.is_outdoors() {
                self.follower_link_state_mut().set_lower_level_state(2);
            }
        }

        self.follower_link_state_mut().restore_z_from_mirror();
        self.follower_link_state_mut()
            .restore_actual_z_velocity_from_mirror();
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(2);
        self.link_move_position();

        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.actual_z_velocity() < 0xa0 {
                self.follower_link_state_mut().set_actual_z_velocity(0xa0);
            }
            if self
                .game_state
                .player
                .follower_link
                .is_landing_at_or_above_ground()
            {
                self.follower_link_state_mut().set_z(0);
                self.link_splash_upon_landing();
                if self.game_state.player.follower_link.is_near_pit() {
                    self.follower_link_state_mut().set_handler_state(1);
                }
                if self.game_state.player.follower_link.handler_state() != 4
                    && self.game_state.player.follower_link.handler_state() != 1
                    && !self.game_state.player.follower_link.is_in_deep_water()
                {
                    self.ancilla_sfx2_near(33);
                }
                self.follower_link_state_mut()
                    .clear_sprite_damage_disable_timer();
                self.set_allow_scroll_z(0);
                self.follower_link_state_mut().clear_auxiliary_state();
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_incapacitated_timer(0);
                if self.game_state.world.location.is_outdoors() {
                    self.follower_link_state_mut().clear_lower_level();
                }
            } else {
                let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
                self.follower_link_state_mut().set_y_velocity(y_velocity);
            }
        } else {
            let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
            self.follower_link_state_mut().set_y_velocity(y_velocity);
        }
        self.follower_link_state_mut()
            .cache_actual_z_velocity_to_mirror();
        self.follower_link_state_mut().cache_z_to_mirror();
    }

    pub(super) fn link_hop_find_tile_to_land_on_south(&mut self) {
        let original_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self
            .game_state
            .player
            .follower_link
            .y_low_delta_from_safe_return();
        self.follower_link_state_mut().set_y_velocity(y_velocity);
        loop {
            let dir = self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards_index();
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
            self.follower_link_state_mut().set_y(y);
            self.tile_detect_movement_y(
                self.game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    .into(),
            );
            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self.game_state.player.tile_detection.pit_tile_word()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath()
                | self.game_state.player.tile_detection.thick_grass()
                | self.game_state.player.tile_detection.deepwater();
            if terrain & 7 == 7 {
                break;
            }
        }
        if self.game_state.player.tile_detection.deepwater() & 7 != 0 {
            self.follower_link_state_mut().enter_deep_water_state();
            if !self
                .game_state
                .player
                .follower_link
                .is_in_auxiliary_state(4)
            {
                self.follower_link_state_mut().set_auxiliary_state(2);
            }
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
        }
        if self.game_state.player.tile_detection.pit_tile_word() & 7 != 0 {
            self.follower_link_state_mut().mark_pit_landing_oam_state();
            self.follower_link_state_mut().begin_pit_check();
        }
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards_index();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().store_safe_return_y(y);
        self.follower_link_state_mut().set_incapacitated_timer(1);
        let mut z = self.game_state.player.follower_link.z_low();
        if z >= 0xf0 {
            z = 0;
        }
        let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
        self.follower_link_state_mut().set_z_and_mirror(z);
    }

    pub(super) fn link_state_hopping_horizontally_ow(&mut self) {
        let direction = if self
            .game_state
            .player
            .follower_link
            .actual_x_velocity_signed()
            .is_negative()
        {
            6
        } else {
            5
        };
        self.follower_link_state_mut().set_direction(direction);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_actual_y_velocity(0);
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.link_state_handling_jump();
    }

    pub(super) fn link_hopping_horizontally_find_tile_y(&mut self) {
        let original_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self
            .game_state
            .player
            .follower_link
            .y_low_delta_from_safe_return();
        self.follower_link_state_mut().set_y_velocity(y_velocity);

        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards_index();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
        self.follower_link_state_mut().set_y(y);
        self.tile_detect_movement_y(
            self.game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                .into(),
        );

        let terrain = self.game_state.player.tile_detection.normal_tiles()
            | self
                .game_state
                .player
                .tile_detection
                .destruction_aftermath()
            | self.game_state.player.tile_detection.thick_grass()
            | self.game_state.player.tile_detection.deepwater();

        if terrain & 7 != 7 {
            self.follower_link_state_mut().set_y(original_y);
            self.follower_link_state_mut().set_incapacitated_timer(1);

            let org_velx = self.game_state.player.follower_link.actual_x_velocity();
            let mut velx = org_velx as i8;
            if velx < 0 {
                velx = velx.wrapping_neg();
            }
            let idx = ((velx as u8) >> 4) as usize;
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(HOP_HORIZ_VEL_Z[idx]);
            let mut xt = HOP_HORIZ_VEL_X[idx];
            if (org_velx as i8) < 0 {
                xt = 0u8.wrapping_sub(xt);
            }
            self.follower_link_state_mut().set_actual_x_velocity(xt);
        } else {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
            self.follower_link_state_mut().set_y(y);
            self.follower_link_state_mut().store_safe_return_y(y);
            self.follower_link_state_mut().set_incapacitated_timer(1);
            let mut z = self.game_state.player.follower_link.z_low();
            if z == 255 {
                z = 0;
            }
            let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
            self.follower_link_state_mut().set_z_and_mirror(z);
        }

        if self.game_state.player.tile_detection.deepwater() & 7 != 0 {
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.link_set_to_deep_water();
        }
    }

    pub(super) fn link_hopping_horizontally_find_tile_x(&mut self, o: u8) -> u8 {
        assert!(o == 0 || o == 2);
        let original_x = self.game_state.player.follower_link.x();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_x);
        let table_idx = (o >> 1) as usize;
        let mut i: i16 = 7;
        loop {
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(HOP_HORIZ_X_STEP[table_idx] as i16 as u16);
            self.follower_link_state_mut().set_x(x);
            self.tile_detect_movement_x(
                self.game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    .into(),
            );

            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath()
                | self.game_state.player.tile_detection.thick_grass()
                | self.game_state.player.tile_detection.deepwater()
                | self.game_state.player.tile_detection.pit_tile_word();

            if terrain & 7 == 7 {
                if self.game_state.player.tile_detection.deepwater() & 7 == 7 {
                    self.follower_link_state_mut().enter_deep_water_state();
                    self.follower_link_state_mut().set_auxiliary_state(2);
                    self.follower_link_state_mut()
                        .set_swim_flags_from_last_direction();
                    self.follower_link_state_mut().clear_swimming_countdown();
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.follower_link_state_mut().clear_grabbing_wall();
                    self.reset_all_acceleration();
                }
                break;
            }
            i -= 1;
            if i < 0 {
                let x = original_x.wrapping_add(HOP_HORIZ_X_FALLBACK[table_idx] as i16 as u16);
                self.follower_link_state_mut().set_x(x);
                break;
            }
        }

        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(HOP_HORIZ_X_FINAL[table_idx] as i16 as u16);
        self.follower_link_state_mut().set_x(x);
        let distance = original_x.wrapping_sub(x) as i16;
        let distance = if distance < 0 { -distance } else { distance };
        let idx = (distance as u16 >> 3) as usize;
        let mut velx = HOP_HORIZ_X_VEL[idx];
        if o != 2 {
            velx = 0u8.wrapping_sub(velx);
        }
        self.follower_link_state_mut().set_actual_x_velocity(velx);
        self.follower_link_state_mut()
            .set_actual_z_velocity_mirror_and_copy(HOP_HORIZ_Z_VEL[idx]);
        i as u8
    }

    pub(super) fn link_state_hopping_diagonally_up_ow(&mut self) {
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.player_change_z(2);
        self.link_move_position();
        if self.game_state.player.follower_link.is_z_low_negative() {
            self.link_splash_upon_landing();
            if self.game_state.player.follower_link.handler_state() != 4
                && !self.game_state.player.follower_link.is_in_deep_water()
            {
                self.ancilla_sfx2_near(33);
            }
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_actual_z_velocity(0xff);
            self.follower_link_state_mut().set_z(0xffff);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_direction_lock();
        }
    }

    pub(super) fn link_state_hopping_diagonally_down_ow(&mut self) {
        let dir = if self
            .game_state
            .player
            .follower_link
            .actual_x_velocity_signed()
            .is_negative()
        {
            2
        } else {
            3
        };
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(dir);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_actual_y_velocity(0);
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if self.game_state.player.follower_link.incapacitated_timer() == 0
            && self
                .game_state
                .player
                .follower_link
                .actual_z_velocity_mirror()
                == 0
        {
            self.follower_link_state_mut()
                .set_last_direction_moved_towards(1);
            let old_x = self.game_state.player.follower_link.x();
            self.ancilla_sfx2_near(32);
            self.link_hop_find_landing_spot_diagonally_down();
            self.follower_link_state_mut().set_x(old_x);

            let distance = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.player.follower_link.hop_origin_coord());
            let idx = ((distance >> 3) as usize).min(23);
            let mut velx = LEDGE_DOWN_X_VEL[idx];
            if dir == 2 {
                velx = 0u8.wrapping_sub(velx);
            }
            self.follower_link_state_mut().set_actual_x_velocity(velx);
            if self.game_state.world.location.is_outdoors() {
                self.follower_link_state_mut().set_lower_level_state(2);
            }
        }
        self.link_state_handling_jump();
    }

    pub(super) fn link_hop_find_landing_spot_diagonally_down(&mut self) {
        let original_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self
            .game_state
            .player
            .follower_link
            .y_low_delta_from_safe_return();
        self.follower_link_state_mut().set_y_velocity(y_velocity);

        let scratch = loop {
            let o = if self
                .game_state
                .player
                .follower_link
                .actual_x_velocity_signed()
                .is_negative()
            {
                0
            } else {
                1
            };
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(LEDGE_DIAG_DX[o] as i16 as u16);
            self.follower_link_state_mut().set_x(x);
            let dir = self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards_index();
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(LEDGE_DIAG_DY[dir] as i16 as u16);
            self.follower_link_state_mut().set_y(y);
            self.tile_detect_movement_y(
                self.game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    .into(),
            );
            let scratch = LEDGE_DIAG_BITS[o];
            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath_low() as u16
                | self.game_state.player.tile_detection.thick_grass_low() as u16
                | self.game_state.player.tile_detection.deepwater();
            if terrain & scratch as u16 == scratch as u16 {
                break scratch;
            }
        };

        if self.game_state.player.tile_detection.deepwater() & scratch as u16 != 0 {
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_grabbing_wall();
        }

        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards_index();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(LEDGE_DIAG_DY2[dir] as i16 as u16);
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().store_safe_return_y(y);
        self.follower_link_state_mut().set_incapacitated_timer(1);
        let z = y
            .wrapping_sub(original_y)
            .wrapping_add(self.game_state.player.follower_link.z_low() as u16);
        self.follower_link_state_mut().set_z_and_mirror(z);
    }

    pub(super) fn link_state_handling_jump(&mut self) {
        self.follower_link_state_mut()
            .restore_actual_z_velocity_from_mirror();
        self.follower_link_state_mut().restore_z_low_from_mirror();
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(2);
        self.link_move_position();
        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.actual_z_velocity() < 0xa0 {
                self.follower_link_state_mut().set_actual_z_velocity(0xa0);
            }
            if self
                .game_state
                .player
                .follower_link
                .is_low_z_landing_at_or_above_ground()
            {
                self.follower_link_state_mut().set_z(0);
                let mut falling_into_pit = false;
                if matches!(
                    self.game_state.player.follower_link.handler_state(),
                    12 | 14
                ) {
                    self.tile_detect_main_handler(0);
                    if self.game_state.player.tile_detection.deepwater() as u8 & 1 != 0 {
                        self.follower_link_state_mut().set_handler_state(4);
                        self.link_set_to_deep_water();
                        self.link_reset_sword_and_item_usage();
                        self.ancilla_add_splash(21, 0);
                    } else if self.game_state.player.tile_detection.pit_tile() & 1 != 0 {
                        self.follower_link_state_mut().mark_pit_landing_oam_state();
                        self.follower_link_state_mut().begin_pit_check();
                        self.follower_link_state_mut().set_handler_state(1);
                        falling_into_pit = true;
                    }
                }
                if !falling_into_pit {
                    self.link_splash_upon_landing();
                    if self.game_state.player.follower_link.handler_state() != 4
                        && !self.game_state.player.follower_link.is_in_deep_water()
                    {
                        self.ancilla_sfx2_near(33);
                    }
                }
                if self.game_state.player.follower_link.handler_state() != 4
                    || !self.game_state.player.follower_link.is_bunny_mirror()
                {
                    self.follower_link_state_mut()
                        .clear_sprite_damage_disable_timer();
                }
                self.set_allow_scroll_z(0);
                self.follower_link_state_mut().clear_auxiliary_state();
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_incapacitated_timer(0);
                if self.game_state.world.location.is_outdoors() {
                    self.follower_link_state_mut().clear_lower_level();
                }
            } else {
                let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
                self.follower_link_state_mut().set_y_velocity(y_velocity);
            }
        } else {
            let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
            self.follower_link_state_mut().set_y_velocity(y_velocity);
        }
        self.follower_link_state_mut()
            .cache_actual_z_velocity_to_mirror();
        self.follower_link_state_mut().cache_z_low_to_mirror();
    }

    pub(super) fn link_state_dashing(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.game_state.player.follower_link.handler_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        if !self.game_state.player.follower_link.is_running() {
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().set_dash_countdown(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut().clear_direction_lock();
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 != 0
            && self.game_state.player.follower_link.button_b_frames() >= 9
        {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
        self.follower_link_state_mut().set_pit_correction_timer(0);

        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().set_dash_countdown(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_direction_lock();
            self.follower_link_state_mut().clear_running();
            self.follower_link_state_mut().clear_defense_flags();
            if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
                if self.game_state.player.follower_link.is_cape_active() {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(2);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.follower_link_state_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.follower_link_state_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        let mut a = self.game_state.player.follower_link.dash_countdown();
        if a == 0 {
            a = self.game_state.player.follower_link.index_of_dashing_sfx();
            self.follower_link_state_mut()
                .decrement_index_of_dashing_sfx();
        }
        if LINK_STATE_DASHING_DASH_SFX_TRIGGER_MASKS
            [(self.game_state.player.follower_link.dash_countdown() >> 4) as usize]
            & a
            == 0
        {
            self.ancilla_sfx2_near(35);
        }
        if (self.follower_link_state_mut().decrement_dash_countdown() as i8).is_negative() {
            self.follower_link_state_mut().set_dash_countdown(0);
            let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
            if self.game_state.sprites.follower_runtime.indicator()
                == DASH_FOLLOWER_SLOWDOWN_INDICATORS[follower]
            {
                self.follower_state_mut()
                    .set_indicator(DASH_FOLLOWER_RELEASE_INDICATORS[follower]);
            }
        } else {
            self.follower_link_state_mut().clear_index_of_dashing_sfx();
            if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().set_dash_countdown(0);
                self.follower_link_state_mut().set_speed_setting(0);
                self.follower_link_state_mut().clear_handler_state();
                self.follower_link_state_mut().clear_running();
                if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                    self.follower_link_state_mut().clear_direction_lock();
                }
                return;
            }
            self.ancilla_add_dash_dust_charging(30, 0);
            self.follower_link_state_mut().clear_movement_velocity();
            self.follower_link_state_mut().prime_dash_counter();
            self.follower_link_state_mut().set_speed_setting(16);
            let mut dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 != 0
                || self.game_state.player.follower_link.doorway_state() != 0
                || dir == 0
            {
                dir = LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING
                    [self.game_state.player.follower_link.facing_index()];
            }
            self.follower_link_state_mut()
                .set_direction_and_last_direction(dir);
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.link_handle_moving_animation_full_long_entry();
            let org_x = self.game_state.player.follower_link.x();
            let org_y = self.game_state.player.follower_link.y();
            self.store_link_safe_return_position(org_x, org_y);
            self.link_handle_moving_floor();
            self.link_apply_conveyor();
            if self
                .game_state
                .player
                .follower_link
                .has_somaria_platform_state()
            {
                self.link_handle_velocity_and_sand_drag(org_x, org_y);
            }
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut()
                .set_movement_velocity_from_position_delta(x, y, org_x, org_y);
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
            return;
        }

        self.follower_link_state_mut()
            .clear_animation_step_if_at_least(6);
        self.follower_link_state_mut()
            .decrement_dash_counter_clamped_to_minimum(32);
        self.ancilla_add_dash_dust(30, 0);
        self.follower_link_state_mut()
            .clear_spin_attack_step_counter();
        if self.game_state.inventory.items.sword_type().wrapping_add(1) & 0xfe != 0 {
            self.tile_detect_main_handler(7);
        }
        if self.game_state.inventory.save_progress.progress_indicator() != 0 {
            self.follower_link_state_mut()
                .add_button_mask_b_y_bits(0x80);
            self.follower_link_state_mut().set_button_b_frames(9);
        }
        self.follower_link_state_mut().set_incapacitated_timer(0);

        let mut want_stop_dash = false;

        if self
            .game_state
            .enhanced_features
            .has(LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING)
        {
            if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                self.follower_link_state_mut().set_dash_countdown(0x11);
                want_stop_dash = true;
            } else {
                let t = LINK_STATE_DASHING_DASH_CONTROLS_TO_DIRECTION
                    [(self.game_state.player.follower_link.joypad1h_last() & 0x0f) as usize];
                if t != 0 && t != self.game_state.player.follower_link.last_direction() {
                    self.follower_link_state_mut()
                        .set_direction_and_last_direction(t);
                    self.follower_link_state_mut()
                        .set_swim_flags_from_last_direction();
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
        } else {
            let dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
            want_stop_dash = dir != 0
                && dir
                    != LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING
                        [self.game_state.player.follower_link.facing_index()];
        }
        if want_stop_dash {
            self.follower_link_state_mut().set_handler_state(18);
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x80);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.link_state_exiting_dash();
            return;
        }

        if self.game_state.player.follower_link.speed_setting() == 0
            && self
                .game_state
                .enhanced_features
                .has(LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING)
        {
            self.follower_link_state_mut().set_speed_setting(16);
        }
        let mut dir = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if dir == 0 {
            dir = LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING
                [self.game_state.player.follower_link.facing_index()];
        }
        self.follower_link_state_mut()
            .set_direction_and_last_direction(dir);
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_handle_sword_cooldown(&mut self) {
        if (self.follower_link_state_mut().decrement_sword_delay_timer() as i8) >= 0 {
            return;
        }
        self.follower_link_state_mut().clear_sword_delay_timer();
        if self
            .game_state
            .player
            .follower_link
            .has_item_or_position_mode()
        {
            return;
        }
        if self.game_state.player.follower_link.button_b_frames() < 9 {
            if !self.game_state.player.follower_link.is_running() {
                self.link_check_for_sword_swing();
            }
        } else {
            self.handle_sword_controls();
        }
    }

    pub(super) fn handle_sword_sfx_and_beam(&mut self) {
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.follower_link_state_mut().clear_button_b_frames();
        self.follower_link_state_mut()
            .clear_spin_attack_step_counter();

        let health = self
            .game_state
            .inventory
            .player_resources
            .health_capacity()
            .wrapping_sub(4);
        let sword = self.game_state.inventory.items.sword_type();
        if health < self.game_state.inventory.player_resources.current_health()
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
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(1);
    }

    pub(super) fn link_check_for_sword_swing(&mut self) {
        if self.game_state.player.follower_link.y_button_action_flags() & 0x10 != 0 {
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x80 == 0 {
                return;
            }
            if self.game_state.player.follower_link.doorway_state() != 0 {
                self.tile_detect_sword_swing_deep_in_door(
                    self.game_state.player.follower_link.doorway_state(),
                );
                if self.game_state.player.tile_detection.collision_bits_low() & 0x30 == 0x30 {
                    return;
                }
            }
            self.follower_link_state_mut()
                .add_button_mask_b_y_bits(0x80);
            self.handle_sword_sfx_and_beam();
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
        }

        if self.game_state.player.follower_link.joypad1h_last() & 0x80 == 0 {
            self.follower_link_state_mut().add_button_mask_b_y_bits(1);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let frames = self.follower_link_state_mut().increment_button_b_frames();
            if frames >= 9 {
                self.handle_sword_controls();
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(spin_attack_delay_for_frames(frames));
            let sword = self.game_state.inventory.items.sword_type();
            if frames == 5 {
                if sword != 0 && sword != 1 && sword != 0xff {
                    self.ancilla_add_sword_swing_sparkle(0x26, 4);
                }
                if sword != 0 && sword != 0xff {
                    self.tile_detect_main_handler(if sword == 1 { 1 } else { 6 });
                }
            } else if frames >= 4
                && self.game_state.player.follower_link.button_mask_b_y() & 1 != 0
                && self.game_state.player.follower_link.joypad1h_last() & 0x80 != 0
            {
                self.follower_link_state_mut().clear_button_mask_b_y_bits(1);
                self.handle_sword_sfx_and_beam();
                return;
            }
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn handle_sword_controls(&mut self) {
        if self.game_state.player.follower_link.joypad1h_last() & 0x80 != 0 {
            self.player_sword_spin_attack_jerks_hold_down();
        } else if self
            .game_state
            .player
            .follower_link
            .spin_attack_step_counter()
            < 48
        {
            self.link_reset_sword_and_item_usage();
        } else {
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut()
                .clear_spin_attack_step_counter();
            self.link_activate_spin_attack();
        }
    }

    pub(super) fn player_sword_spin_attack_jerks_hold_down(&mut self) {
        if self.game_state.player.follower_link.defense_flags() & 0x80 != 0
            || self.game_state.player.follower_link.defense_flags() & 9 == 0
        {
            if self.game_state.sprite_battle.damaging_enemies_timer() == 0 {
                self.follower_link_state_mut().set_button_b_frames(9);
                self.follower_link_state_mut().set_direction_lock_bits(1);
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(0);
                if self.game_state.player.follower_link.speed_setting() != 4
                    && self.game_state.player.follower_link.speed_setting() != 16
                {
                    self.follower_link_state_mut().set_speed_setting(12);
                    if self.game_state.inventory.items.sword_type().wrapping_add(1) & !1 == 0 {
                        return;
                    }
                    if (0..5)
                        .rev()
                        .any(|i| matches!(self.ancilla_slot_view(i).ancilla_type(), 0x30 | 0x31))
                    {
                        return;
                    }
                    if self
                        .game_state
                        .player
                        .follower_link
                        .spin_attack_step_counter()
                        >= 6
                        && self.game_state.frame.frame_counter & 3 == 0
                    {
                        self.ancilla_spawn_sword_charge_sparkle();
                    }
                    if self
                        .game_state
                        .player
                        .follower_link
                        .spin_attack_step_counter()
                        < 64
                    {
                        if self
                            .follower_link_state_mut()
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
            } else if self.game_state.sprite_battle.damaging_enemies_timer() == 1 {
                self.link_reset_sword_and_item_usage();
                return;
            }
        }
        if self.game_state.player.follower_link.button_b_frames() == 9 {
            self.follower_link_state_mut().set_button_b_frames(10);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(SPIN_ATTACK_DELAYS[10]);
        }
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let mut frames = self
                .game_state
                .player
                .follower_link
                .button_b_frames()
                .wrapping_add(1);
            if frames == 13 {
                if self.game_state.inventory.items.sword_type().wrapping_add(1) & !1 != 0
                    && self.game_state.player.follower_link.defense_flags() & 9 != 0
                {
                    self.ancilla_add_wall_tap_spark(27, 1);
                    self.ancilla_sfx2_near(
                        if self.game_state.player.follower_link.defense_flags() & 8 != 0 {
                            6
                        } else {
                            5
                        },
                    );
                    self.tile_detect_main_handler(1);
                }
                frames = 10;
            }
            self.follower_link_state_mut().set_button_b_frames(frames);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(spin_attack_delay_for_frames(frames));
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn link_activate_spin_attack(&mut self) {
        self.ancilla_add_spin_attack_init_spark(42, 0, 0);
        self.link_animate_victory_spin();
    }

    pub(super) fn link_animate_victory_spin(&mut self) {
        self.follower_link_state_mut().set_handler_state(3);
        let spin_offsets = (self.game_state.player.follower_link.facing() >> 1) * 12;
        self.follower_link_state_mut()
            .set_spin_offsets(spin_offsets);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(3);
        let spin_state =
            LINK_SPIN_GRAPHICS_BY_DIR[self.game_state.player.follower_link.spin_offsets() as usize];
        self.follower_link_state_mut()
            .set_state_for_spin_attack(spin_state);
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut().set_button_b_frames(144);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().set_button_mask_b_y(0x80);
        self.link_state_spin_attack();
    }

    pub(super) fn link_state_tree_pull(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.handle_link_from1_d();
            return;
        }

        if self
            .game_state
            .player
            .follower_link
            .has_grabbing_wall_state()
        {
            if self.game_state.player.follower_link.button_mask_b_y() == 0 {
                if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                    self.follower_link_state_mut().clear_grabbing_wall();
                    self.follower_link_state_mut().clear_item_action_step_var();
                    self.follower_link_state_mut().set_y_button_action_timer(2);
                    self.follower_link_state_mut().set_y_button_action_step(0);
                    self.follower_link_state_mut().clear_direction_lock();
                    self.follower_link_state_mut().clear_handler_state();
                    self.link_state_default();
                    return;
                }
                if self.game_state.player.follower_link.joypad1h_last() & 4 == 0 {
                    self.link_state_tree_pull_tail();
                    return;
                }
                self.follower_link_state_mut().set_button_mask_b_y(4);
                self.ancilla_sfx2_near(0x22);
            }

            self.follower_link_state_mut()
                .decrement_y_button_action_timer();
            if (self.game_state.player.follower_link.y_button_action_timer() as i8) >= 0 {
                self.link_state_tree_pull_tail();
                return;
            }
            let j = self
                .follower_link_state_mut()
                .increment_item_action_step_var() as usize;
            self.follower_link_state_mut()
                .set_y_button_action_step(*GRAB_WALL_ANIM_STEPS.get(j).unwrap_or(&0));
            self.follower_link_state_mut()
                .set_y_button_action_timer(*GRAB_WALL_ANIM_TIMER.get(j).unwrap_or(&0));
            if j != 7 {
                self.link_state_tree_pull_tail();
                return;
            }

            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_y_button_action_timer(2);
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_state_bits(1);
            self.follower_link_state_mut().clear_picking_throw_state();
        }

        if self.game_state.player.follower_link.defense_flags() & 9 != 0 {
            self.link_state_tree_pull_reset_to_normal();
            return;
        }
        if self.game_state.player.follower_link.item_action_step_var() == 9 {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x0f == 0 {
                self.link_handle_cardinal_collision();
                self.handle_indoor_camera_and_doors();
                return;
            }
            self.follower_link_state_mut().clear_handler_state();
            self.link_state_default();
            return;
        }
        self.ancilla_add_dash_dust_charging(0x1e, 0);
        self.follower_link_state_mut()
            .decrement_y_button_action_timer();
        if (self.game_state.player.follower_link.y_button_action_timer() as i8) < 0 {
            let j = self
                .follower_link_state_mut()
                .increment_item_action_step_var() as usize;
            self.follower_link_state_mut()
                .set_y_button_action_step(GRAB_WALL_ANIM_STEPS2[j]);
            self.follower_link_state_mut().set_y_button_action_timer(2);
            self.follower_link_state_mut().set_actual_y_velocity(48);
            if j == 9 {
                self.link_state_tree_pull_reset_to_normal();
                return;
            }
        }
        self.flag67_with_directions();
        if self.game_state.player.follower_link.direction() & 3 == 0 {
            self.follower_link_state_mut().clear_actual_x_velocity();
        }
        if self.game_state.player.follower_link.direction() & 0x0c == 0 {
            self.follower_link_state_mut().clear_actual_y_velocity();
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
            self.follower_link_state_mut().clear_page_movement_deltas();
            self.follower_link_state_mut()
                .clear_orthogonal_direction_count();
            self.link_handle_recoiling();
            if self
                .follower_link_state_mut()
                .decrement_incapacitated_timer()
                == 0
            {
                self.follower_link_state_mut()
                    .reset_elapsed_incapacitated_timer();
                if self
                    .game_state
                    .player
                    .follower_link
                    .is_recoil_landing_z_window()
                    && (self.game_state.player.follower_link.actual_z_velocity() as i8) < 0
                {
                    if self.game_state.player.follower_link.has_auxiliary_state() {
                        self.follower_link_state_mut()
                            .clear_sprite_damage_disable_timer();
                        let old_state = self.game_state.player.follower_link.handler_state();
                        self.tile_detect_position_mut()
                            .set_interaction_scratch_y(old_state as u16);
                        if self.game_state.player.follower_link.handler_state() != 6 {
                            self.follower_link_state_mut().clear_button_b_frames();
                            self.follower_link_state_mut().set_button_mask_b_y(0);
                            self.follower_link_state_mut()
                                .set_spin_attack_delay_timer(0);
                            self.follower_link_state_mut()
                                .clear_spin_attack_step_counter();
                        }
                        self.link_splash_upon_landing();
                        if !self.game_state.player.follower_link.is_bunny_mirror()
                            || !self.game_state.player.follower_link.is_in_deep_water()
                        {
                            if self.game_state.player.follower_link.dash_noise_requested() {
                                self.follower_link_state_mut().clear_dash_noise_request();
                                self.ancilla_sfx2_near(33);
                            } else if old_state != 2
                                && self.game_state.player.follower_link.handler_state() != 4
                            {
                                self.ancilla_sfx2_near(33);
                            }
                            if self.game_state.player.follower_link.handler_state() == 4 {
                                self.link_force_unequip_cape_quietly();
                                if self.game_state.world.location.is_indoors()
                                    && old_state != 2
                                    && self.game_state.player.follower_link.has_flippers()
                                {
                                    self.follower_link_state_mut().mark_lower_level();
                                }
                                self.ancilla_add_splash(21, 0);
                            }
                            self.tile_detect_main_handler(0);
                            if self.game_state.player.tile_detection.thick_grass_low() & 1 != 0 {
                                self.ancilla_sfx2_near(26);
                            }
                            if self.game_state.player.tile_detection.shallow_water_low() & 1 != 0
                                && self.game_state.system_signals.sound_effect_1() != 36
                            {
                                self.ancilla_sfx2_near(28);
                            }
                            if self.game_state.player.tile_detection.deepwater() & 1 != 0 {
                                self.follower_link_state_mut().set_handler_state(4);
                                self.link_set_to_deep_water();
                                self.link_reset_sword_and_item_usage();
                                self.ancilla_add_splash(21, 0);
                            }
                        }
                        self.finish_recoil_landing();
                    }
                    self.follower_link_state_mut().clear_animation_step();
                    self.follower_link_state_mut().set_incapacitated_timer(0);
                }
            }
        } else {
            self.finish_recoil_landing();
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }

        if self.game_state.player.follower_link.handler_state() != 5
            && self.game_state.player.follower_link.incapacitated_timer() >= 33
        {
            if (self
                .follower_link_state_mut()
                .decrement_incapacitated_camera_timer() as i8)
                >= 0
            {
                self.handle_indoor_camera_and_doors();
                self.follower_link_state_mut().clear_z_high();
                return;
            }
            self.follower_link_state_mut()
                .reset_incapacitated_camera_timer_from_incapacitated();
        }

        self.flag67_with_directions();
        if self.game_state.player.follower_link.handler_state() != 6 {
            self.link_handle_diagonal_collision();
            if self.game_state.player.follower_link.direction() & 3 == 0 {
                self.follower_link_state_mut().clear_actual_x_velocity();
            }
            if self.game_state.player.follower_link.direction() & 0x0c == 0 {
                self.follower_link_state_mut().clear_actual_y_velocity();
            }
        }
        self.link_move_position();

        if self.game_state.player.follower_link.handler_state() != 6 {
            self.link_handle_cardinal_collision();
            self.follower_link_state_mut().clear_pit_correction();
        }
        self.handle_indoor_camera_and_doors();
        if self
            .game_state
            .player
            .follower_link
            .should_probe_recoil_landing_tile()
        {
            self.player_tile_detect_nearby();
            self.replay_trace_player_state("recoil-timer-after-nearby");
            if self.game_state.player.tile_detection.pit_tile() & 0x0f == 0x0f {
                self.follower_link_state_mut().set_handler_state(1);
                self.follower_link_state_mut().set_speed_setting(4);
                self.replay_trace_player_state("recoil-timer-set-pit");
            }
        }
        self.follower_link_state_mut().clear_z_high();
        self.replay_trace_player_state("recoil-timer-exit");
    }

    pub(super) fn gravestone_move(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
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
        self.follower_link_state_mut().clear_hookshot_grave_latch();
        self.follower_link_state_mut().and_defense_flags(!4);
        let debris_y = self.game_state.effects.door_debris.y(k);
        let debris_x = self.game_state.effects.door_debris.x(k);
        self.tile_detect_position_mut()
            .set_interaction_scratch_y_bytes(debris_y, debris_x);
        let big_rock = self
            .game_state
            .player
            .tile_detection
            .interaction_scratch_y();
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(big_rock);
        let counter = match big_rock {
            0x0532 => 0x48,
            0x0488 => 0x60,
            _ => 0x40,
        };
        self.dungeon_doors_mut().set_door_open_counter(counter);
        self.overworld_do_map_update32x32_b_for_smash();
    }

    pub(super) fn somaria_block_handle_player_interaction(&mut self, k: usize) {
        self.sprite_system_mut().set_cur_object_index(k as u8);
        if self.ancilla_slot_view(k).g() != 0 {
            return;
        }

        if self.ancilla_slot_view(k).h() == 0 {
            if self.game_state.player.follower_link.has_auxiliary_state()
                || self.game_state.player.follower_link.state_bits_has(1)
                || {
                    let z = self.ancilla_slot_view(k).z();
                    z != 0 && z != 0xff
                }
                || self.ancilla_slot_view(k).k() != 0
                || self.ancilla_slot_view(k).l() != 0
            {
                return;
            }
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f == 0 {
                self.ancilla_slot_view_mut(k).set_work_byte_3(0);
                self.follower_link_state_mut().clear_defense_flags();
                self.ancilla_slot_view_mut(k).set_a(255);
                if !self.game_state.player.follower_link.is_running() {
                    self.follower_link_state_mut().set_speed_setting(0);
                    return;
                }
            } else if self.game_state.player.follower_link.joypad1h_last() & 0x0f
                == self.ancilla_slot_view(k).work_byte_3()
            {
                if self.game_state.player.follower_link.speed_setting() == 18 {
                    self.follower_link_state_mut().or_defense_flags(0x81);
                }
            } else {
                let last_direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                self.ancilla_slot_view_mut(k)
                    .set_work_byte_3(last_direction);
                self.follower_link_state_mut().set_speed_setting(0);
            }

            if !self.ancilla_check_link_collision(k, 4)
                || self.ancilla_slot_view(k).floor()
                    != self.game_state.player.follower_link.lower_level_state()
            {
                return;
            }

            if !self.game_state.player.follower_link.is_running()
                || self.game_state.player.follower_link.dash_counter() == 64
            {
                let t = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
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
                if self.game_state.player.follower_link.actual_y_velocity() == 0
                    || self.game_state.player.follower_link.actual_x_velocity() == 0
                {
                    if !self.ancilla_check_tile_collision_class2(k) {
                        self.ancilla_move_y(k);
                        self.ancilla_move_x(k);
                        let movement_ticks = self.ancilla_slot_view_mut(k).advance_a();
                        if !self
                            .game_state
                            .player
                            .follower_link
                            .is_lifting_or_carrying()
                            && movement_ticks & 7 == 0
                        {
                            self.ancilla_sfx2_pan(k, 0x22);
                        }
                    }
                    self.follower_link_state_mut().set_defense_flags(0x81);
                    self.follower_link_state_mut().set_speed_setting(0x12);
                }
                self.sprite_nullify_hookshot_drag();
                return;
            }

            if self.game_state.player.follower_link.ancilla_pickup_flag() == k as u8 + 1 {
                self.follower_link_state_mut().clear_ancilla_pickup_flag();
            }
            self.link_cancel_dash();
            self.ancilla_sfx3_pan(k, 0x32);
            let j = self.game_state.player.follower_link.facing_index();
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_direction(j as u8);
                ancilla.set_y_velocity(LINK_ITEM_CANE_OF_SOMARIA_BLOCK_Y_VELOCITIES[j]);
                ancilla.set_x_velocity(LINK_ITEM_CANE_OF_SOMARIA_BLOCK_X_VELOCITIES[j]);
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
            let mut ancilla = self.ancilla_slot_view_mut(k);
            let y_velocity = ((ancilla.y_velocity() as i8) / 2) as u8;
            let x_velocity = ((ancilla.x_velocity() as i8) / 2) as u8;
            ancilla.set_z_velocity(
                LINK_ITEM_CANE_OF_SOMARIA_BLOCK_Z_VELOCITIES[j.wrapping_sub(1) as usize],
            );
            ancilla.set_y_velocity(y_velocity);
            ancilla.set_x_velocity(x_velocity);
        }
    }

    pub(super) fn gravestone_act_as_barrier(&mut self, k: usize) {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        let r4 = y.wrapping_add(0x18);
        let r6 = x.wrapping_add(0x20);
        let lx = self.game_state.player.follower_link.x().wrapping_add(8);
        let ly = self.game_state.player.follower_link.y().wrapping_add(8);
        if ly >= y && ly < r4 && lx >= x && lx < r6 {
            let r10 = ly.abs_diff(r4);
            let new_y = self.game_state.player.follower_link.y().wrapping_add(r10);
            self.follower_link_state_mut().set_y(new_y);
            self.follower_link_state_mut()
                .add_y_velocity_delta(r10 as u8);
            self.follower_link_state_mut().or_defense_flags(4);
        }
        if self.game_state.player.follower_link.has_facing() {
            let facing = self.game_state.player.follower_link.facing() & !4;
            self.follower_link_state_mut().set_facing(facing);
        }
    }

    pub(super) fn link_handle_recoiling(&mut self) {
        self.follower_link_state_mut().set_direction(0);
        if self.game_state.player.follower_link.actual_y_velocity() != 0 {
            let direction = if self
                .game_state
                .player
                .follower_link
                .actual_y_velocity_signed()
                .is_negative()
            {
                8
            } else {
                4
            };
            self.follower_link_state_mut()
                .add_direction_flags(direction);
            self.follower_link_state_mut()
                .set_last_direction_from_current_direction();
            self.player_handle_incapacitated_inner2();
        }
        if self.game_state.player.follower_link.actual_x_velocity() != 0 {
            let direction = if self
                .game_state
                .player
                .follower_link
                .actual_x_velocity_signed()
                .is_negative()
            {
                2
            } else {
                1
            };
            self.follower_link_state_mut()
                .add_direction_flags(direction);
            self.follower_link_state_mut()
                .set_last_direction_from_current_direction();
        }
        self.player_handle_incapacitated_inner2();
    }

    pub(super) fn player_handle_incapacitated_inner2(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .is_moving_against_diag_tile_on_both_axes()
            && self.game_state.player.follower_link.handler_state() == 2
        {
            self.follower_link_state_mut().invert_actual_velocity_xy();
        }
        if self.game_state.player.follower_link.doorway_state() == 1 {
            self.follower_link_state_mut().mask_last_direction(0x0c);
            self.follower_link_state_mut().mask_direction(0x0c);
            self.follower_link_state_mut().clear_actual_x_velocity();
        } else if self.game_state.player.follower_link.doorway_state() == 2 {
            self.follower_link_state_mut().mask_last_direction(3);
            self.follower_link_state_mut().mask_direction(3);
            self.follower_link_state_mut().clear_actual_y_velocity();
        }
    }

    pub(super) fn find_free_moving_block_slot(&mut self, x: u8) -> u8 {
        if self
            .game_state
            .dungeon
            .object_tracking
            .changeable_object_index(1)
            == 0
        {
            self.dungeon_object_tracking_mut()
                .set_changeable_object_index(1, x.wrapping_add(1));
            return 1;
        }
        if self
            .game_state
            .dungeon
            .object_tracking
            .changeable_object_index(0)
            == 0
        {
            self.dungeon_object_tracking_mut()
                .set_changeable_object_index(0, x.wrapping_add(1));
            return 0;
        }
        0xff
    }

    pub(super) fn initialize_push_block(&mut self, r14: u8, idx: u8) -> bool {
        let slot = r14 as usize;
        let idx_word = (idx >> 1) as usize;
        let pos = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(idx_word);
        let mut x = (pos & 0x007e) << 2;
        let mut y = (pos & 0x1f80) >> 4;
        x = x.wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h() & 0xff00);
        y = y.wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v() & 0xff00);

        self.pushed_block_mut().init_slot(slot, x, y);

        if self.game_state.dungeon.header.primary_header_tag() != 38
            && self
                .game_state
                .dungeon
                .object_tracking
                .replacement_tile_state(idx_word)
                == 0
        {
            if !self.push_block_attempt_to_push_the_block(0, x, y) {
                self.ancilla_sfx2_near(0x22);
                self.dungeon_object_tracking_mut()
                    .set_replacement_tile_state(idx_word, 1);
                return false;
            }
        }

        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(slot);
        true
    }

    pub(super) fn sprite_dungeon_draw_single_push_block(&mut self, mut j: usize) {
        j >>= 1;
        self.oam_allocate_from_region_b(4);
        let y = self
            .game_state
            .player
            .pushed_block
            .y(j)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(1);
        let x = self
            .game_state
            .player
            .pushed_block
            .x(j)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let ch = SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_CHARS
            [SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_PUSH_BLOCK_CHAR_INDEX_BY_MODE
                [self.game_state.player.pushed_block.animation_mode() as usize]
                .min(SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_CHARS.len() - 1)];
        if ch != 0xff {
            let oam = self.game_state.oam.current_pointer_usize();
            self.oam_state_mut()
                .write_entry(oam, x as u8, y as u8, ch, 0x20);
            let ext = self.game_state.oam.current_extended_pointer_usize();
            self.oam_state_mut().set_extended_byte_at(ext, 2);
        }
    }

    pub(super) fn handle_layer_of_destination(&mut self) {
        let hole_teleporter_plane = self.game_state.dungeon.header.hole_teleporter_plane(0);
        self.follower_link_state_mut().set_lower_level_states(
            u8::from(hole_teleporter_plane >= 2),
            u8::from(hole_teleporter_plane >= 1),
        );
    }

    pub(super) fn dungeon_pit_do_damage(&mut self) {
        self.set_submodule(20);
        if self.player_resources_mut().decrement_current_health_by(8) >= 0xa8 {
            self.player_resources_mut().set_current_health(0);
        }
    }

    pub(super) fn reset_some_things_after_death(&mut self, speed_setting: u8) {
        self.follower_link_state_mut().clear_deep_water_state();
        self.follower_link_state_mut()
            .set_speed_setting(speed_setting);
        self.follower_link_state_mut().clear_conveyor_belt_state();
        self.set_player_layer_collision_flags(0);
        self.follower_link_state_mut().clear_immobilized();
        self.follower_state_mut().clear_palette_swap_flag();
        self.follower_link_state_mut()
            .clear_faint_animation_active();
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_actual_z_velocity(0);
        // C writes `link_z_coord = 0` as a single byte (LINK_Z_COORD low only); the
        // high byte 0x25 is left stale. Use set_z_low, not the 16-bit set_z, so we
        // don't clobber 0x25 (mode-reused; e.g. carries Link-Z high into the intro).
        self.follower_link_state_mut().set_z_low(0);
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.tile_detect_position_mut()
            .set_tile_collision_bits_primary(0);
        self.follower_link_state_mut().clear_blink_countdown();
        self.follower_link_state_mut().clear_handler_state();
        self.follower_link_state_mut().set_visibility_status(0);
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }

    pub(super) fn player_handler_00_ground_3(&mut self) {
        self.apply_links_movement_to_camera_called = false;
        self.follower_link_state_mut().set_z(0xffff);
        self.follower_link_state_mut().set_actual_z_velocity(0xff);
        self.follower_link_state_mut().set_recoil_timer(0);

        if !self.link_handle_toss() {
            self.ground_apress_defers_atomic_item_receipt = true;
            self.link_handle_a_press();
            self.ground_apress_defers_atomic_item_receipt = false;
            if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                        ground_apress_tail: Some(_),
                        ..
                    },
                })
            ) {
                // The chest receipt scheduled by this A-press blocks the ROM
                // iteration inside its decompression; the remainder of this
                // handler runs at the receipt's completion slice with the
                // joypad latch it holds now.
                return;
            }
            self.player_handler_00_ground_3_after_a_press(true);
        } else {
            self.player_handler_00_ground_3_after_a_press(false);
        }
    }

    /// Everything `HandleLink_From1D` runs after its A-press dispatch. Split
    /// out so a chest receipt's completion slice can resume it after the
    /// decompression the ROM blocks on (see the ground_apress_tail receipt).
    pub(super) fn player_handler_00_ground_3_after_a_press(&mut self, ran_a_press: bool) {
        let mut clear_vel_after = false;
        if ran_a_press {
            if !self.game_state.player.follower_link.has_action_state()
                && !self
                    .game_state
                    .player
                    .follower_link
                    .has_grabbing_wall_state()
                && !self.game_state.player.follower_link.has_pull_action_state()
                && self.game_state.player.follower_link.handler_state() != 17
            {
                self.link_handle_y_item();
                if self
                    .game_state
                    .enhanced_features
                    .has(PLAYER_HANDLER_00_GROUND_3_FEATURES0_MISC_BUG_FIXES)
                    && ((self.game_state.frame.main_module == 14
                        && self.game_state.frame.submodule != 2)
                        || matches!(
                            self.game_state.player.follower_link.handler_state(),
                            8 | 9 | 10
                        ))
                {
                    self.finish_ground_movement_clear_vel_tail();
                    return;
                }
                if self.game_state.inventory.save_progress.progress_indicator() != 0 {
                    self.link_handle_sword_cooldown();
                    if self.game_state.player.follower_link.handler_state() == 3 {
                        self.finish_ground_movement_clear_vel_tail();
                        return;
                    }
                }
            }
        }

        let _ = &mut clear_vel_after;
        self.link_handle_cape_passive_lift_check();
        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.follower_link_state_mut()
                .clear_lift_throw_scratch_state();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        if self.game_state.player.follower_link.has_pull_action_state() {
            self.follower_link_state_mut().set_direction(0);
            clear_vel_after = true;
        } else if !self.game_state.player.follower_link.is_transforming()
            && self
                .game_state
                .player
                .follower_link
                .is_ready_to_start_ground_movement()
            && (self.game_state.player.follower_link.button_b_frames() >= 9
                || (self.game_state.player.follower_link.button_mask_b_y() & 0x20) != 0
                || (self.game_state.player.follower_link.button_mask_b_y() & 0x80) == 0)
        {
            if self.game_state.player.follower_link.flag_moving() != 0 {
                self.swim_acceleration_mut().set_max_speed(0, 0x0180);
                self.swim_acceleration_mut().set_max_speed(2, 0x0180);
                self.link_handle_swim_movements();
                return;
            }

            self.reset_all_acceleration();
            let mut dir = (self
                .game_state
                .player
                .follower_link
                .force_move_any_direction() as u8)
                & 0x0f;
            if dir == 0 {
                if self.game_state.player.follower_link.grabbing_wall_has(2) {
                    self.finish_ground_movement_tail(clear_vel_after);
                    return;
                }
                dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
            }
            if dir == 0 {
                self.follower_link_state_mut()
                    .clear_movement_velocity_and_direction();
                self.follower_link_state_mut().set_last_direction(0);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().and_defense_flags(!0x0f);
                self.follower_link_state_mut().reset_push_fatigue_timer();
                self.follower_link_state_mut().reset_jump_ledge_timer();
            } else {
                self.follower_link_state_mut().set_direction(dir);
                if dir != self.game_state.player.follower_link.last_direction() {
                    self.follower_link_state_mut().set_last_direction(dir);
                    self.follower_link_state_mut().clear_movement_subpixels();
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    self.follower_link_state_mut().clear_defense_flags();
                    self.follower_link_state_mut().reset_push_fatigue_timer();
                    self.follower_link_state_mut().reset_jump_ledge_timer();
                }
            }
        }

        self.finish_ground_movement_tail(clear_vel_after);
    }

    pub(super) fn link_perform_throw(&mut self) {
        if (self.game_state.player.follower_link.sprite_pickup_flag()
            | self.game_state.player.follower_link.ancilla_pickup_flag())
            == 0
        {
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut().set_y_button_action_flags(0);
            let mut i = 15i8;
            while self.sprite_slot_view(i as usize).state() != 0 {
                i -= 1;
                if i < 0 {
                    return;
                }
            }

            if matches!(
                self.game_state
                    .player
                    .tile_detection
                    .liftable_action_index_primary(),
                5 | 6
            ) {
                self.follower_link_state_mut().set_action_handler_timer(1);
            } else {
                let (attr, x, y) = if self.game_state.world.location.is_indoors() {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.Dungeon_LiftAndReplaceLiftable(&mut pt);
                    (attr, pt.x, pt.y)
                } else {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.Overworld_HandleLiftableTiles(&mut pt);
                    (attr, pt.x, pt.y)
                };
                let Some(idx) = LINK_PERFORM_THROW_LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE
                    .iter()
                    .rposition(|&value| value == attr)
                else {
                    return;
                };
                self.follower_link_state_mut().set_sprite_pickup_flag(1);
                self.sprite_spawn_throwable_terrain(idx as u8, x, y);
                self.follower_link_state_mut()
                    .clear_filtered_joypad_l_bits(0x80);
                self.follower_link_state_mut().clear_action_handler_timer();
            }
        } else {
            self.follower_link_state_mut().clear_action_handler_timer();
        }

        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().set_y_button_action_timer(6);
        self.follower_link_state_mut().start_lift_throw_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_speed_setting(12);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().mask_direction(0xf0);
        self.follower_link_state_mut().set_direction_lock_bits(1);
    }

    pub(super) fn spawn_hammer_water_splash(&mut self) {
        if (self.game_state.frame.submodule
            | self.game_state.player.follower_link.immobilized_flag()
            | self.game_state.frame.modal_pause_flag)
            != 0
        {
            return;
        }
        let i = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(SPAWN_HAMMER_WATER_SPLASH_HAMMER_WATER_X[i] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(SPAWN_HAMMER_WATER_SPLASH_HAMMER_WATER_Y[i] as i16 as u16);
        let tiletype = if self.game_state.world.location.is_indoors() {
            let mut t = if self.game_state.player.follower_link.lower_level_state() >= 1 {
                0x1000
            } else {
                0
            };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.game_state.dungeon.bg2_attributes.bg2_attr(t)
        } else {
            self.overworld_get_tile_attribute_at_location(x >> 3, y)
        };

        if matches!(tiletype, 8 | 9) {
            let j = self.sprite_spawn_small_splash(0);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, x.wrapping_sub(8));
                self.sprite_set_y(j, y.wrapping_sub(16));
                let floor = self.game_state.player.follower_link.lower_level_state();
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_floor(floor);
                sprite.set_z(0);
            }
        }
    }

    pub(super) fn digging_game_guy_attempt_prize_spawn(&mut self) {
        self.digging_game_prize_mut().increment_attempts();
        if self.game_state.player.follower_link.y() >= 0x0b18 {
            return;
        }
        let j = self.get_random_number() & 7;
        let item_to_spawn = match j {
            0..=3 => DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_ITEMS[j as usize],
            4 => {
                if self.game_state.effects.digging_game_prize.attempts() < 25
                    || self.game_state.effects.digging_game_prize.spawned_marker() != 0
                    || self.get_random_number() & 3 != 0
                {
                    return;
                }
                self.digging_game_prize_mut().mark_spawned();
                0xeb
            }
            _ => return,
        };

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(4, item_to_spawn, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = usize::from(self.game_state.player.follower_link.facing() != 4);
            {
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_x_velocity(DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_XVEL[i]);
                sprite.set_y_velocity(0);
                sprite.set_z_velocity(24);
                sprite.set_stunned(255);
                sprite.set_delay_aux4(48);
            }
            let x =
                self.game_state.player.follower_link.x().wrapping_add(
                    DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_X[i] as i16 as u16,
                ) & !0x0f;
            let y = self.game_state.player.follower_link.y().wrapping_add(22) & !0x0f;
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
        if self.game_state.world.location.is_outdoors() {
            return;
        }
        if self.game_state.player.follower_link.doorway_state() != 0 {
            self.handle_door_transitions();
        } else {
            self.apply_links_movement_to_camera();
        }
    }

    pub(super) fn cache_camera_properties_if_outdoors(&mut self) {
        if self.game_state.world.location.is_outdoors() {
            self.cache_camera_properties_for_player();
        }
    }

    pub(super) fn handle_door_transitions(&mut self) {
        self.follower_link_state_mut().clear_page_movement_deltas();

        if self
            .game_state
            .enhanced_features
            .has(HANDLE_DOOR_TRANSITIONS_FEATURES0_MISC_BUG_FIXES)
            && !(self.game_state.frame.main_module == 7 && self.game_state.frame.submodule == 0)
        {
            return;
        }

        if self.game_state.player.follower_link.last_direction() & 0x0c != 0
            && self.game_state.player.follower_link.doorway_state() == 1
        {
            if self.game_state.player.follower_link.last_direction() & 4 != 0 {
                let t = self.game_state.player.follower_link.y().wrapping_add(28);
                if t & 0x00fc == 0 {
                    self.follower_link_state_mut()
                        .set_y_page_movement_delta_from_high_position((t >> 8) as u8);
                }
            } else {
                let t = self.game_state.player.follower_link.y().wrapping_sub(18);
                self.follower_link_state_mut()
                    .set_y_page_movement_delta_from_high_position((t >> 8) as u8);
            }
        }

        if self.game_state.player.follower_link.last_direction() & 3 != 0
            && self.game_state.player.follower_link.doorway_state() == 2
        {
            if self.game_state.player.follower_link.last_direction() & 1 != 0 {
                let t = self.game_state.player.follower_link.x().wrapping_add(21);
                if t & 0x00fc == 0 {
                    self.follower_link_state_mut()
                        .set_x_page_movement_delta_from_high_position((t >> 8) as u8);
                }
            } else {
                let t = self.game_state.player.follower_link.x().wrapping_sub(8);
                self.follower_link_state_mut()
                    .set_x_page_movement_delta_from_high_position((t >> 8) as u8);
            }
        }

        if self.game_state.player.follower_link.x_page_movement_delta() != 0 {
            self.follower_link_state_mut().set_y_button_action_timer(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self
                .game_state
                .player
                .follower_link
                .x_page_movement_delta_signed()
                .is_negative()
            {
                self.Dung_StartInterRoomTrans_Left_Plus();
            } else {
                self.HandleEdgeTransitionMovementEast_RightBy8();
            }
        } else if self.game_state.player.follower_link.y_page_movement_delta() != 0 {
            self.follower_link_state_mut().set_y_button_action_timer(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self
                .game_state
                .player
                .follower_link
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
            (
                self.game_state
                    .player
                    .follower_link
                    .y_high()
                    .wrapping_sub(self.game_state.player.follower_link.safe_return_y_high()),
                self.game_state
                    .player
                    .follower_link
                    .x_high()
                    .wrapping_sub(self.game_state.player.follower_link.safe_return_x_high()),
            )
        };
        self.follower_link_state_mut()
            .set_page_movement_deltas(y_delta, x_delta);

        if self.game_state.player.follower_link.x_page_movement_delta() != 0 {
            if self
                .game_state
                .player
                .follower_link
                .x_page_movement_delta_signed()
                .is_negative()
            {
                self.AdjustQuadrantAndCamera_left();
            } else {
                self.AdjustQuadrantAndCamera_right();
            }
        }
        if self.game_state.player.follower_link.y_page_movement_delta() != 0 {
            if self
                .game_state
                .player
                .follower_link
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

#[cfg(test)]
#[path = "player_tests.rs"]
mod tests;
