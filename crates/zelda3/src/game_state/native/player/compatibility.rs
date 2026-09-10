//! Cartridge-layout import and publication for the native player components.
//!
//! Projection order and partial writes are source contracts. Keep them visible
//! here rather than hiding RAM publication inside native movement/action logic.

use super::*;
use crate::game_state::native::ram_target::RamTarget;

impl FollowerLinkState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            input: PlayerInputState {
                button_mask_b_y: ram_byte(ram, BUTTON_MASK_B_Y),
                filtered_joypad_h: ram_byte(ram, FILTERED_JOYPAD_H),
                filtered_joypad_l: ram_byte(ram, FILTERED_JOYPAD_L),
                joypad1h_last: ram_byte(ram, JOYPAD1H_LAST),
                joypad1l_last: ram_byte(ram, JOYPAD1L_LAST),
                joypad1h_last2: ram_byte(ram, JOYPAD1H_LAST2),
                joypad1l_last2: ram_byte(ram, JOYPAD1L_LAST2),
                button_b_frames: ram_byte(ram, BUTTON_B_FRAMES),
            },
            presentation: PlayerPresentationState {
                oam_x_offset: ram_byte(ram, PLAYER_OAM_X_OFFSET),
                oam_y_offset: ram_byte(ram, PLAYER_OAM_Y_OFFSET),
                visibility_status: ram_byte(ram, LINK_VISIBILITY_STATUS),
                blink_countdown: ram_byte(ram, COUNTDOWN_FOR_BLINK),
                spin_animation_step_counter: ram_byte(ram, STEP_COUNTER_FOR_SPIN_ATTACK),
                animation_step: ram_byte(ram, LINK_ANIMATION_STEPS),
                opening_pose: ram_byte(ram, LINK_POSE_DURING_OPENING),
                water_ripple_or_grass_state: ram_byte(ram, DRAW_WATER_RIPPLES_OR_GRASS),
                primary_water_grass_timer: ram_byte(ram, PRIMARY_WATER_GRASS_TIMER),
                secondary_water_grass_timer: ram_byte(ram, SECONDARY_WATER_GRASS_TIMER),
                frame_change_counter: ram_byte(ram, LINK_FRAME_CHANGE_COUNTER),
                sprite_oam_state_timer: ram_byte(ram, LINK_SPRITE_OAM_STATE_TIMER),
                throw_oam_state_index: ram_byte(ram, LINK_THROW_OAM_STATE_INDEX),
                item_hold_pose: ram_byte(ram, LINK_POSE_FOR_ITEM),
                force_hold_sword_up: ram_byte(ram, LINK_FORCE_HOLD_SWORD_UP),
                dash_noise_requested: ram_byte(ram, LINK_WANT_MAKE_NOISE_WHEN_DASHED),
                faint_animation_active: ram_byte(ram, LINK_FAINT_ANIMATION_ACTIVE),
                index_of_dashing_sfx: ram_byte(ram, INDEX_OF_DASHING_SFX),
                spin_offsets: ram_byte(ram, LINK_SPIN_OFFSETS),
                link_dma_graphics_index: read_le_u16(ram, LINK_DMA_GRAPHICS_INDEX),
                link_dma_left_sprite_bank: read_le_u16(ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX),
                link_dma_right_sprite_bank: read_le_u16(ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX),
                sword_dma_graphics_index: ram_byte(ram, LINK_DMA_SWORD_GRAPHICS_INDEX),
                shield_dma_graphics_index: ram_byte(ram, LINK_DMA_SHIELD_GRAPHICS_INDEX),
                link_dma_staging_index: ram_byte(ram, LINK_DMA_STAGING_INDEX),
                palette_bits_of_oam: read_le_u16(ram, LINK_PALETTE_BITS_OF_OAM),
                player_pose_draw_counter: ram_byte(ram, PLAYER_POSE_DRAW_COUNTER),
                player_special_draw_flag: ram_byte(ram, PLAYER_SPECIAL_DRAW_FLAG),
                sleep_in_bed_state: ram_byte(ram, PLAYER_SLEEP_IN_BED_STATE),
            },
            movement: PlayerMovementState {
                x: read_le_u16(ram, LINK_X_COORD),
                y: read_le_u16(ram, LINK_Y_COORD),
                z: read_le_u16(ram, LINK_Z_COORD),
                z_mirror: read_le_u16(ram, LINK_Z_COORD_MIRROR),
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
                cached_lower_level_mirror_state: ram_byte(
                    ram,
                    LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED,
                ),
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
                running: ram_byte(ram, LINK_IS_RUNNING),
                doorway_state: ram_byte(ram, IS_STANDING_IN_DOORWAY),
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
                whirlpool_trigger: ram_byte(ram, LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE),
                prevent_movement: ram_byte(ram, LINK_PREVENT_FROM_MOVING),
                cached_tile_action_index: ram_byte(ram, CACHED_TILE_ACTION_INDEX),
                force_move_any_direction: read_le_u16(ram, FORCE_MOVE_ANY_DIRECTION),
                position_mode: ram_byte(ram, LINK_POSITION_MODE),
                near_moveable_statue_flag: ram_byte(ram, LINK_IS_NEAR_MOVEABLE_STATUE),
                grabbing_wall: ram_byte(ram, LINK_GRABBING_WALL),
                somaria_platform_state: ram_byte(ram, PLAYER_ON_SOMARIA_PLATFORM),
                near_pit_state: ram_byte(ram, PLAYER_NEAR_PIT_STATE),
                pit_data_index: ram_byte(ram, PLAYER_PIT_DATA_INDEX),
                pit_correction_timer: ram_byte(ram, PIT_CORRECTION_TIMER),
                pit_correction_active: ram_byte(ram, PIT_CORRECTION_ACTIVE_FLAG),
                moving_against_diag_deadlocked: ram_byte(ram, MOVING_AGAINST_DIAG_DEADLOCKED),
                incapacitated_camera_timer: ram_byte(ram, LINK_INCAPACITATED_CAMERA_TIMER),
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
                cheat_walk_through_walls: ram_byte(ram, CHEAT_WALK_THROUGH_WALLS),
                x_page_movement_delta: ram_byte(ram, LINK_X_PAGE_MOVEMENT_DELTA),
                y_page_movement_delta: ram_byte(ram, LINK_Y_PAGE_MOVEMENT_DELTA),
                moving_floor_x: read_le_u16(ram, RELATED_TO_MOVING_FLOOR_X),
                moving_floor_y: read_le_u16(ram, RELATED_TO_MOVING_FLOOR_Y),
                drag_player_x: read_le_u16(ram, DRAG_PLAYER_X),
                drag_player_y: read_le_u16(ram, DRAG_PLAYER_Y),
            },
            actions: PlayerActionsState {
                menu_block_flag: ram_byte(ram, FLAG_BLOCK_LINK_MENU),
                handler_state: ram_byte(ram, LINK_HANDLER_STATE),
                immobilized: ram_byte(ram, FLAG_IS_LINK_IMMOBILIZED),
                action_state_bits: ram_byte(ram, LINK_STATE_BITS),
                auxiliary_state: ram_byte(ram, LINK_AUXILIARY_STATE),
                picking_throw_state: ram_byte(ram, LINK_PICKING_THROW_STATE),
                spin_attack_delay_timer: ram_byte(ram, LINK_DELAY_TIMER_SPIN_ATTACK),
                spin_attack_step_counter: ram_byte(ram, LINK_SPIN_ATTACK_STEP_COUNTER),
                spin_attack_state: ram_byte(ram, STATE_FOR_SPIN_ATTACK),
                spin_attack_sound_latch: ram_byte(ram, SPIN_ATTACK_SOUND_LATCH),
                incapacitated_timer: ram_byte(ram, LINK_INCAPACITATED_TIMER),
                y_button_action_flags: ram_byte(ram, Y_BUTTON_ACTION_FLAGS),
                y_button_action_step: ram_byte(ram, Y_BUTTON_ACTION_STEP),
                y_button_action_timer: ram_byte(ram, Y_BUTTON_ACTION_TIMER),
                defense_flags: ram_byte(ram, PLAYER_DEFENSE_FLAGS),
                item_receipt_method: ram_byte(ram, ITEM_RECEIPT_METHOD),
                action_handler_timer: ram_byte(ram, PLAYER_HANDLER_TIMER),
                bunny_transform_timer: ram_byte(ram, LINK_BUNNY_TRANSFORM_TIMER),
                bunny_state: ram_byte(ram, LINK_IS_BUNNY),
                bunny_mirror: ram_byte(ram, LINK_IS_BUNNY_MIRROR),
                temp_bunny_timer: read_le_u16(ram, LINK_TIMER_TEMPBUNNY),
                transform_poof_needed: ram_byte(ram, LINK_NEED_FOR_POOF_FOR_TRANSFORM),
                magic_spell_player_lock: ram_byte(ram, MAGIC_SPELL_PLAYER_LOCK_FLAG),
                item_holding_timer: ram_byte(ram, LINK_ITEM_HOLDING_TIMER),
                ancilla_interactive_reset_flag: ram_byte(ram, ANCILLA_INTERACTIVE_RESET_FLAG),
                item_action_step: ram_byte(ram, LINK_ITEM_ACTION_STEP),
                item_action_debug_value_2: ram_byte(ram, LINK_DEBUG_VALUE_2),
                item_debug_value_1: ram_byte(ram, LINK_DEBUG_VALUE_1),
                given_damage: ram_byte(ram, LINK_GIVE_DAMAGE),
                pull_action_state: ram_byte(ram, LINK_PULL_ACTION_STATE),
                current_item_y: ram_byte(ram, LINK_CURRENT_ITEM_Y),
                current_item_active: ram_byte(ram, LINK_CURRENT_ITEM_ACTIVE),
                receive_item_index: ram_byte(ram, LINK_RECEIVE_ITEM_INDEX),
                item_in_hand: ram_byte(ram, LINK_ITEM_IN_HAND),
                item_pickup_in_progress: ram_byte(ram, ITEM_PICKUP_IN_PROGRESS_FLAG),
                selected_rod: ram_byte(ram, EQ_SELECTED_ROD),
                ancilla_pickup_flag: ram_byte(ram, FLAG_IS_ANCILLA_TO_PICK_UP),
                sprite_pickup_flag: ram_byte(ram, FLAG_IS_SPRITE_TO_PICK_UP),
                sprite_pickup_flag_cached: ram_byte(ram, FLAG_IS_SPRITE_TO_PICK_UP_CACHED),
                pull_for_rupees_sprite_needed: ram_byte(ram, LINK_NEED_FOR_PULLFORRUPEES_SPRITE),
                hookshot_interlock: ram_byte(ram, RELATED_TO_HOOKSHOT),
                hookshot_grave_latch: ram_byte(ram, LINK_SOMETHING_WITH_HOOKSHOT),
                electrocute_on_touch: ram_byte(ram, LINK_ELECTROCUTE_ON_TOUCH),
                cape_mode: ram_byte(ram, LINK_CAPE_MODE),
                cape_decrement_counter: ram_byte(ram, CAPE_DECREMENT_COUNTER),
                sword_delay_timer: ram_byte(ram, LINK_SWORD_DELAY_TIMER),
                transforming: ram_byte(ram, LINK_IS_TRANSFORMING),
                flute_countdown: ram_byte(ram, FLUTE_COUNTDOWN),
                hookshot_bg_check_off_timer: ram_byte(ram, HOOKSHOT_BG_CHECK_OFF_TIMER),
                sprite_damage_disabled: ram_byte(ram, LINK_DISABLE_SPRITE_DAMAGE),
            },
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(LINK_X_COORD, self.movement.x);
        ram.write_word(LINK_Y_COORD, self.movement.y);
        ram.write_word(LINK_Z_COORD, self.movement.z);
        ram.write_word(LINK_Z_COORD_MIRROR, self.movement.z_mirror);
        ram.write_byte(PLAYER_OAM_X_OFFSET, self.presentation.oam_x_offset);
        ram.write_byte(PLAYER_OAM_Y_OFFSET, self.presentation.oam_y_offset);
        ram.write_byte(LINK_X_SUBPIXEL, self.movement.x_subpixel);
        ram.write_byte(LINK_Y_SUBPIXEL, self.movement.y_subpixel);
        ram.write_byte(LINK_X_VELOCITY, self.movement.x_velocity);
        ram.write_byte(LINK_Y_VELOCITY, self.movement.y_velocity);
        ram.write_byte(LINK_ACTUAL_X_VELOCITY, self.movement.actual_x_velocity);
        ram.write_byte(LINK_ACTUAL_Y_VELOCITY, self.movement.actual_y_velocity);
        ram.write_byte(LINK_Z_VELOCITY, self.movement.z_velocity);
        ram.write_byte(LINK_Z_VELOCITY_COPY, self.movement.z_velocity_copy);
        ram.write_byte(LINK_Z_VELOCITY_MIRROR, self.movement.z_velocity_mirror);
        ram.write_byte(
            LINK_Z_VELOCITY_COPY_MIRROR,
            self.movement.z_velocity_copy_mirror,
        );
        ram.write_byte(
            LINK_RECOIL_Z_VELOCITY_DUNGEON,
            self.movement.recoil_z_velocity_for_dungeon_reset,
        );
        ram.write_byte(LINK_RECOIL_TIMER, self.movement.recoil_timer);
        ram.write_byte(LINK_IS_ON_LOWER_LEVEL, self.movement.floor);
        ram.write_byte(
            LINK_IS_ON_LOWER_LEVEL_MIRROR,
            self.movement.lower_level_mirror_state,
        );
        ram.write_byte(
            LINK_IS_ON_LOWER_LEVEL_CACHED,
            self.movement.cached_lower_level_state,
        );
        ram.write_byte(
            LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED,
            self.movement.cached_lower_level_mirror_state,
        );
        ram.write_byte(LINK_DIRECTION, self.movement.direction);
        ram.write_byte(LINK_CANT_CHANGE_DIRECTION, self.movement.direction_lock);
        ram.write_byte(LINK_DIRECTION_MASK_A, self.movement.direction_mask_a);
        ram.write_byte(LINK_DIRECTION_MASK_B, self.movement.direction_mask_b);
        ram.write_byte(LINK_LAST_DIRECTION, self.movement.last_direction);
        ram.write_byte(
            LINK_LAST_DIRECTION_MOVED_TOWARDS,
            self.movement.last_direction_moved_towards,
        );
        ram.write_byte(
            LINK_MOVING_AGAINST_DIAG_TILE,
            self.movement.moving_against_diag_tile,
        );
        ram.write_byte(LINK_FLAG_MOVING, self.movement.movement_flag);
        ram.write_byte(LINK_QUADRANT_X, self.movement.quadrant_x);
        ram.write_byte(LINK_QUADRANT_Y, self.movement.quadrant_y);
        ram.write_byte(LINK_QUADRANT_X_CACHED, self.movement.cached_quadrant_x);
        ram.write_byte(LINK_QUADRANT_Y_CACHED, self.movement.cached_quadrant_y);
        ram.write_byte(
            LINK_NUM_ORTHOGONAL_DIRECTIONS,
            self.movement.num_orthogonal_directions,
        );
        ram.write_byte(
            SWIM_PLAYER_DIRECTION_FLAGS,
            self.movement.swim_direction_flags,
        );
        ram.write_byte(LINK_FACING, self.movement.facing);
        ram.write_byte(LINK_FACING_MIRROR, self.movement.facing_mirror);
        ram.write_byte(LINK_FACING_CACHED, self.movement.cached_facing);
        ram.write_byte(LINK_SPEED_SETTING, self.movement.speed_setting);
        ram.write_byte(LINK_SPEED_MODIFIER, self.movement.speed_modifier);
        ram.write_byte(LINK_DASH_COUNTER, self.movement.dash_counter);
        ram.write_byte(LINK_COUNTDOWN_FOR_DASH, self.movement.dash_countdown);
        ram.write_byte(LINK_TIMER_JUMP_LEDGE, self.movement.jump_ledge_timer);
        ram.write_byte(
            ABOUT_TO_JUMP_OFF_LEDGE,
            self.movement.about_to_jump_off_ledge,
        );
        ram.write_byte(LINK_TIMER_PUSH_GET_TIRED, self.movement.push_fatigue_timer);
        ram.write_byte(
            GRAVESTONE_PUSH_TIMEOUT,
            self.movement.gravestone_push_timeout,
        );
        ram.write_byte(FLAG_BLOCK_LINK_MENU, self.actions.menu_block_flag);
        ram.write_byte(LINK_HANDLER_STATE, self.actions.handler_state);
        ram.write_byte(FLAG_IS_LINK_IMMOBILIZED, self.actions.immobilized);
        ram.write_byte(LINK_STATE_BITS, self.actions.action_state_bits);
        ram.write_byte(LINK_AUXILIARY_STATE, self.actions.auxiliary_state);
        ram.write_byte(LINK_IS_RUNNING, self.movement.running);
        ram.write_byte(LINK_PICKING_THROW_STATE, self.actions.picking_throw_state);
        ram.write_byte(BUTTON_MASK_B_Y, self.input.button_mask_b_y);
        ram.write_byte(FILTERED_JOYPAD_H, self.input.filtered_joypad_h);
        ram.write_byte(FILTERED_JOYPAD_L, self.input.filtered_joypad_l);
        ram.write_byte(JOYPAD1H_LAST, self.input.joypad1h_last);
        ram.write_byte(JOYPAD1L_LAST, self.input.joypad1l_last);
        ram.write_byte(JOYPAD1H_LAST2, self.input.joypad1h_last2);
        ram.write_byte(JOYPAD1L_LAST2, self.input.joypad1l_last2);
        ram.write_byte(
            LINK_DELAY_TIMER_SPIN_ATTACK,
            self.actions.spin_attack_delay_timer,
        );
        ram.write_byte(
            LINK_SPIN_ATTACK_STEP_COUNTER,
            self.actions.spin_attack_step_counter,
        );
        ram.write_byte(STATE_FOR_SPIN_ATTACK, self.actions.spin_attack_state);
        ram.write_byte(
            SPIN_ATTACK_SOUND_LATCH,
            self.actions.spin_attack_sound_latch,
        );
        ram.write_byte(LINK_INCAPACITATED_TIMER, self.actions.incapacitated_timer);
        ram.write_byte(LINK_VISIBILITY_STATUS, self.presentation.visibility_status);
        ram.write_byte(Y_BUTTON_ACTION_FLAGS, self.actions.y_button_action_flags);
        ram.write_byte(Y_BUTTON_ACTION_STEP, self.actions.y_button_action_step);
        ram.write_byte(Y_BUTTON_ACTION_TIMER, self.actions.y_button_action_timer);
        ram.write_byte(PLAYER_DEFENSE_FLAGS, self.actions.defense_flags);
        ram.write_byte(ITEM_RECEIPT_METHOD, self.actions.item_receipt_method);
        ram.write_byte(PLAYER_HANDLER_TIMER, self.actions.action_handler_timer);
        ram.write_byte(IS_STANDING_IN_DOORWAY, self.movement.doorway_state);
        ram.write_byte(COUNTDOWN_FOR_BLINK, self.presentation.blink_countdown);
        ram.write_byte(
            LINK_BUNNY_TRANSFORM_TIMER,
            self.actions.bunny_transform_timer,
        );
        ram.write_byte(LINK_IS_BUNNY, self.actions.bunny_state);
        ram.write_byte(LINK_IS_BUNNY_MIRROR, self.actions.bunny_mirror);
        ram.write_word(LINK_TIMER_TEMPBUNNY, self.actions.temp_bunny_timer);
        ram.write_byte(
            LINK_NEED_FOR_POOF_FOR_TRANSFORM,
            self.actions.transform_poof_needed,
        );
        ram.write_byte(
            STEP_COUNTER_FOR_SPIN_ATTACK,
            self.presentation.spin_animation_step_counter,
        );
        ram.write_byte(BUTTON_B_FRAMES, self.input.button_b_frames);
        ram.write_byte(LINK_ANIMATION_STEPS, self.presentation.animation_step);
        ram.write_byte(LINK_POSE_DURING_OPENING, self.presentation.opening_pose);
        ram.write_byte(
            DRAW_WATER_RIPPLES_OR_GRASS,
            self.presentation.water_ripple_or_grass_state,
        );
        ram.write_byte(
            PRIMARY_WATER_GRASS_TIMER,
            self.presentation.primary_water_grass_timer,
        );
        ram.write_byte(
            SECONDARY_WATER_GRASS_TIMER,
            self.presentation.secondary_water_grass_timer,
        );
        ram.write_byte(LINK_IS_IN_DEEP_WATER, self.movement.deep_water_state);
        ram.write_byte(LINK_MAYBE_SWIM_FASTER, self.movement.swim_fast_state);
        ram.write_byte(LINK_SWIM_HARD_STROKE, self.movement.hard_swim_stroke);
        ram.write_word(
            SWIM_STROKE_FRAME_COUNTER,
            self.movement.swim_stroke_frame_counters[0],
        );
        ram.write_word(
            SWIM_STROKE_FRAME_COUNTER + 2,
            self.movement.swim_stroke_frame_counters[1],
        );
        ram.write_byte(SWIM_STROKE_ANIM_STEP, self.movement.swim_stroke_anim_step);
        ram.write_byte(SWIMMING_COUNTDOWN, self.movement.swimming_countdown);
        ram.write_byte(LINK_ON_CONVEYOR_BELT, self.movement.conveyor_belt_state);
        ram.write_byte(LINK_TILE_BELOW, self.movement.tile_below);
        ram.write_byte(TILE_ACTION_INDEX, self.movement.tile_action_index);
        ram.write_byte(TILE_COLL_FLAG, self.movement.tile_collision_flag);
        ram.write_byte(
            LINK_FRAME_CHANGE_COUNTER,
            self.presentation.frame_change_counter,
        );
        ram.write_byte(
            LINK_SPRITE_OAM_STATE_TIMER,
            self.presentation.sprite_oam_state_timer,
        );
        ram.write_byte(
            LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE,
            self.movement.whirlpool_trigger,
        );
        ram.write_byte(LINK_PREVENT_FROM_MOVING, self.movement.prevent_movement);
        ram.write_byte(
            MAGIC_SPELL_PLAYER_LOCK_FLAG,
            self.actions.magic_spell_player_lock,
        );
        ram.write_byte(LINK_ITEM_HOLDING_TIMER, self.actions.item_holding_timer);
        ram.write_byte(
            CACHED_TILE_ACTION_INDEX,
            self.movement.cached_tile_action_index,
        );
        ram.write_byte(
            ANCILLA_INTERACTIVE_RESET_FLAG,
            self.actions.ancilla_interactive_reset_flag,
        );
        ram.write_word(
            FORCE_MOVE_ANY_DIRECTION,
            self.movement.force_move_any_direction,
        );
        ram.write_byte(LINK_ITEM_ACTION_STEP, self.actions.item_action_step);
        ram.write_byte(
            LINK_THROW_OAM_STATE_INDEX,
            self.presentation.throw_oam_state_index,
        );
        ram.write_byte(LINK_DEBUG_VALUE_2, self.actions.item_action_debug_value_2);
        ram.write_byte(LINK_DEBUG_VALUE_1, self.actions.item_debug_value_1);
        ram.write_byte(LINK_GIVE_DAMAGE, self.actions.given_damage);
        ram.write_byte(LINK_PULL_ACTION_STATE, self.actions.pull_action_state);
        ram.write_byte(LINK_CURRENT_ITEM_Y, self.actions.current_item_y);
        ram.write_byte(LINK_CURRENT_ITEM_ACTIVE, self.actions.current_item_active);
        ram.write_byte(LINK_RECEIVE_ITEM_INDEX, self.actions.receive_item_index);
        ram.write_byte(LINK_ITEM_IN_HAND, self.actions.item_in_hand);
        ram.write_byte(
            ITEM_PICKUP_IN_PROGRESS_FLAG,
            self.actions.item_pickup_in_progress,
        );
        ram.write_byte(LINK_POSITION_MODE, self.movement.position_mode);
        ram.write_byte(EQ_SELECTED_ROD, self.actions.selected_rod);
        // LINK_MAGIC_CONSUMPTION (0xf37b) is solely owned by PlayerResourcesState, which
        // holds its only writer (Sprite_MagicShopKeeper granting the 1/2-magic upgrade,
        // C sprite_main.c `link_magic_consumption = 1`). C reads one live byte from every
        // consumer; a second projected copy here re-stamped the frame-start value.
        ram.write_byte(FLAG_IS_ANCILLA_TO_PICK_UP, self.actions.ancilla_pickup_flag);
        ram.write_byte(FLAG_IS_SPRITE_TO_PICK_UP, self.actions.sprite_pickup_flag);
        ram.write_byte(
            FLAG_IS_SPRITE_TO_PICK_UP_CACHED,
            self.actions.sprite_pickup_flag_cached,
        );
        ram.write_byte(
            LINK_NEED_FOR_PULLFORRUPEES_SPRITE,
            self.actions.pull_for_rupees_sprite_needed,
        );
        ram.write_byte(
            LINK_IS_NEAR_MOVEABLE_STATUE,
            self.movement.near_moveable_statue_flag,
        );
        ram.write_byte(RELATED_TO_HOOKSHOT, self.actions.hookshot_interlock);
        ram.write_byte(LINK_GRABBING_WALL, self.movement.grabbing_wall);
        ram.write_byte(
            LINK_SOMETHING_WITH_HOOKSHOT,
            self.actions.hookshot_grave_latch,
        );
        ram.write_byte(LINK_ELECTROCUTE_ON_TOUCH, self.actions.electrocute_on_touch);
        ram.write_byte(LINK_CAPE_MODE, self.actions.cape_mode);
        ram.write_byte(CAPE_DECREMENT_COUNTER, self.actions.cape_decrement_counter);
        ram.write_byte(LINK_POSE_FOR_ITEM, self.presentation.item_hold_pose);
        ram.write_byte(
            LINK_FORCE_HOLD_SWORD_UP,
            self.presentation.force_hold_sword_up,
        );
        ram.write_byte(LINK_SWORD_DELAY_TIMER, self.actions.sword_delay_timer);
        ram.write_byte(
            LINK_WANT_MAKE_NOISE_WHEN_DASHED,
            self.presentation.dash_noise_requested,
        );
        ram.write_byte(
            LINK_FAINT_ANIMATION_ACTIVE,
            self.presentation.faint_animation_active,
        );
        ram.write_byte(LINK_IS_TRANSFORMING, self.actions.transforming);
        ram.write_byte(FLUTE_COUNTDOWN, self.actions.flute_countdown);
        ram.write_byte(
            HOOKSHOT_BG_CHECK_OFF_TIMER,
            self.actions.hookshot_bg_check_off_timer,
        );
        ram.write_byte(INDEX_OF_DASHING_SFX, self.presentation.index_of_dashing_sfx);
        ram.write_byte(LINK_SPIN_OFFSETS, self.presentation.spin_offsets);
        ram.write_byte(
            PLAYER_ON_SOMARIA_PLATFORM,
            self.movement.somaria_platform_state,
        );
        ram.write_byte(PLAYER_NEAR_PIT_STATE, self.movement.near_pit_state);
        ram.write_byte(PLAYER_PIT_DATA_INDEX, self.movement.pit_data_index);
        ram.write_byte(PIT_CORRECTION_TIMER, self.movement.pit_correction_timer);
        ram.write_byte(
            PIT_CORRECTION_ACTIVE_FLAG,
            self.movement.pit_correction_active,
        );
        ram.write_byte(
            MOVING_AGAINST_DIAG_DEADLOCKED,
            self.movement.moving_against_diag_deadlocked,
        );
        ram.write_byte(
            LINK_INCAPACITATED_CAMERA_TIMER,
            self.movement.incapacitated_camera_timer,
        );
        ram.write_byte(
            LINK_DISABLE_SPRITE_DAMAGE,
            self.actions.sprite_damage_disabled,
        );
        ram.write_word(
            LINK_DMA_GRAPHICS_INDEX,
            self.presentation.link_dma_graphics_index,
        );
        ram.write_word(
            LINK_DMA_LEFT_SPRITE_BANK_INDEX,
            self.presentation.link_dma_left_sprite_bank,
        );
        ram.write_word(
            LINK_DMA_RIGHT_SPRITE_BANK_INDEX,
            self.presentation.link_dma_right_sprite_bank,
        );
        ram.write_byte(
            LINK_DMA_SWORD_GRAPHICS_INDEX,
            self.presentation.sword_dma_graphics_index,
        );
        ram.write_byte(
            LINK_DMA_SHIELD_GRAPHICS_INDEX,
            self.presentation.shield_dma_graphics_index,
        );
        ram.write_byte(
            LINK_DMA_STAGING_INDEX,
            self.presentation.link_dma_staging_index,
        );
        // LINK_DMA_SOURCE_OFFSET/TILE_OFFSET/COUNTDOWN (0xc00f/0xc015/0xc013) are
        // deliberately NOT modeled: C's Graphics_IncrementalVRAMUpload reads and
        // advances them raw in WRAM, and other systems reuse them during the
        // attract/text sequence. The RAM player view owns the raw read-modify-write.
        ram.write_word(
            LINK_PALETTE_BITS_OF_OAM,
            self.presentation.palette_bits_of_oam,
        );
        // SCRATCH_1 (0x74) is C's `scratch_1`, a call-local scratch register written and
        // consumed inside a single routine (tile_detect, the overworld bush-poof spawn,
        // ancilla, player_oam). TileDetectionState is the sole native model; a second copy
        // here clobbered it 10866 times over the recorded route, on both bytes of the word.
        // Link's sprite-index scratch has no reader at all -- the setter writes RAM
        // directly, exactly as C's `scratch_1 = j` does.
        ram.write_word(LINK_Y_COORD_ORIGINAL, self.movement.hop_origin_coord);
        ram.write_word(LINK_X_COORD_CACHED, self.movement.cached_x);
        ram.write_word(LINK_Y_COORD_CACHED, self.movement.cached_y);
        ram.write_word(LINK_X_COORD_COPY, self.movement.copied_x);
        ram.write_word(LINK_Y_COORD_COPY, self.movement.copied_y);
        ram.write_word(LINK_X_COORD_PREV, self.movement.previous_x);
        ram.write_word(LINK_Y_COORD_PREV, self.movement.previous_y);
        ram.write_byte(
            LINK_X_COORD_SAFE_RETURN_LO,
            self.movement.safe_return_x as u8,
        );
        ram.write_byte(
            LINK_X_COORD_SAFE_RETURN_HI,
            (self.movement.safe_return_x >> 8) as u8,
        );
        ram.write_byte(
            LINK_Y_COORD_SAFE_RETURN_LO,
            self.movement.safe_return_y as u8,
        );
        ram.write_byte(
            LINK_Y_COORD_SAFE_RETURN_HI,
            (self.movement.safe_return_y >> 8) as u8,
        );
        ram.write_word(BIT9_OF_XCOORD, self.movement.bit9_of_xcoord);
        ram.write_byte(
            PLAYER_POSE_DRAW_COUNTER,
            self.presentation.player_pose_draw_counter,
        );
        ram.write_byte(
            PLAYER_SPECIAL_DRAW_FLAG,
            self.presentation.player_special_draw_flag,
        );
        ram.write_byte(
            PLAYER_SLEEP_IN_BED_STATE,
            self.presentation.sleep_in_bed_state,
        );
        ram.write_byte(
            CHEAT_WALK_THROUGH_WALLS,
            self.movement.cheat_walk_through_walls,
        );
        ram.write_byte(
            LINK_X_PAGE_MOVEMENT_DELTA,
            self.movement.x_page_movement_delta,
        );
        ram.write_byte(
            LINK_Y_PAGE_MOVEMENT_DELTA,
            self.movement.y_page_movement_delta,
        );
        ram.write_word(RELATED_TO_MOVING_FLOOR_X, self.movement.moving_floor_x);
        ram.write_word(RELATED_TO_MOVING_FLOOR_Y, self.movement.moving_floor_y);
        ram.write_word(DRAG_PLAYER_X, self.movement.drag_player_x);
        ram.write_word(DRAG_PLAYER_Y, self.movement.drag_player_y);
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
        self.state.movement.set_speed_setting(value);
        self.ram[LINK_SPEED_SETTING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_speed_setting(&mut self) -> u8 {
        let value = self.state.movement.decrement_speed_setting();
        self.ram[LINK_SPEED_SETTING] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_speed_modifier(&mut self) {
        self.state.movement.clear_speed_modifier();
        self.ram[LINK_SPEED_MODIFIER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_speed_modifier(&mut self, value: u8) {
        self.state.movement.set_speed_modifier(value);
        self.ram[LINK_SPEED_MODIFIER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_lower_level(&mut self) {
        self.state.movement.mark_lower_level();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_lower_level_mirror(&mut self) {
        self.state.movement.mark_lower_level_mirror();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lower_level_state(&mut self, value: u8) {
        self.state.movement.set_lower_level_state(value);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lower_level_mirror_state(&mut self, value: u8) {
        self.state.movement.set_lower_level_mirror_state(value);
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lower_level_states(&mut self, state: u8, mirror: u8) {
        self.state.movement.set_lower_level_states(state, mirror);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = state;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = mirror;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lower_level(&mut self) {
        self.state.movement.clear_lower_level();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lower_level_states(&mut self) {
        self.state.movement.clear_lower_level_states();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn toggle_lower_level_state(&mut self) {
        self.state.movement.toggle_lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = self.state.lower_level_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn toggle_lower_level_mirror_state(&mut self) {
        self.state.movement.toggle_lower_level_mirror_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.state.lower_level_mirror_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mirror_lower_level_state(&mut self) {
        self.state.movement.mirror_lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.state.lower_level_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_lower_level_states(&mut self) {
        self.state.movement.cache_lower_level_states();
        self.ram[LINK_IS_ON_LOWER_LEVEL_CACHED] = self.state.cached_lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED] =
            self.state.cached_lower_level_mirror_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_lower_level_state_from_cached(&mut self) {
        self.state.movement.restore_lower_level_state_from_cached();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = self.state.lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.state.lower_level_mirror_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn arm_stair_speed_modifier(&mut self) {
        self.state.movement.arm_stair_speed_modifier();
        self.ram[LINK_SPEED_SETTING] = 2;
        self.ram[LINK_SPEED_MODIFIER] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn resolve_dash_speed_setting(&mut self) {
        self.state.movement.resolve_dash_speed_setting();
        self.ram[LINK_SPEED_SETTING] = self.state.speed_setting();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn promote_pending_speed_modifier(&mut self) {
        self.state.movement.promote_pending_speed_modifier();
        self.ram[LINK_SPEED_MODIFIER] = self.state.speed_modifier();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increase_near_pit_speed_modifier(&mut self) {
        self.state.movement.increase_near_pit_speed_modifier();
        self.ram[LINK_SPEED_MODIFIER] = self.state.speed_modifier();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_dash_deceleration(&mut self) {
        self.state.movement.advance_dash_deceleration();
        self.ram[LINK_SPEED_MODIFIER] = self.state.speed_modifier();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_handler_state(&mut self, value: u8) {
        self.state.actions.set_handler_state(value);
        self.ram[LINK_HANDLER_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_handler_state(&mut self) {
        self.state.actions.clear_handler_state();
        self.ram[LINK_HANDLER_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_facing(&mut self, value: u8) {
        self.state.movement.set_facing(value);
        self.ram[LINK_FACING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_facing_from_cached(&mut self) {
        self.state.movement.restore_facing_from_cached();
        self.ram[LINK_FACING] = self.state.facing();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_facing_mirror(&mut self, value: u8) {
        self.state.movement.set_facing_mirror(value);
        self.ram[LINK_FACING_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_facing_to_mirror(&mut self) {
        self.state.movement.cache_facing_to_mirror();
        self.ram[LINK_FACING_MIRROR] = self.state.facing();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_facing(&mut self) {
        self.state.movement.cache_facing();
        self.ram[LINK_FACING_CACHED] = self.state.facing();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_moving_against_diag_tile(&mut self, value: u8) {
        self.state.movement.set_moving_against_diag_tile(value);
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_moving_against_diag_tile_flags(&mut self, value: u8) {
        self.state
            .movement
            .add_moving_against_diag_tile_flags(value);
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = self.state.moving_against_diag_tile();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_moving_against_diag_tile(&mut self) {
        self.state.movement.clear_moving_against_diag_tile();
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_flag_moving(&mut self, value: u8) {
        self.state.movement.set_flag_moving(value);
        self.ram[LINK_FLAG_MOVING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_quadrants_from_packed_nibbles(&mut self, value: u8) {
        self.state.movement.set_quadrants_from_packed_nibbles(value);
        self.ram[LINK_QUADRANT_X] = self.state.quadrant_x();
        self.ram[LINK_QUADRANT_Y] = self.state.quadrant_y();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_quadrants(&mut self, x: u8, y: u8) {
        self.state.movement.set_quadrants(x, y);
        self.ram[LINK_QUADRANT_X] = x;
        self.ram[LINK_QUADRANT_Y] = y;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn toggle_quadrant_x(&mut self) -> u8 {
        let value = self.state.movement.toggle_quadrant_x();
        self.ram[LINK_QUADRANT_X] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn toggle_quadrant_y(&mut self) -> u8 {
        let value = self.state.movement.toggle_quadrant_y();
        self.ram[LINK_QUADRANT_Y] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn reset_direction_limits(&mut self) {
        self.state.movement.reset_direction_limits();
        self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
        self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_direction_masks(&mut self) {
        self.state.movement.reset_direction_masks();
        self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
        self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_orthogonal_direction_count(&mut self) {
        self.state.movement.increment_orthogonal_direction_count();
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = self.state.num_orthogonal_directions();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_orthogonal_direction_count(&mut self) {
        self.state.movement.clear_orthogonal_direction_count();
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_moved_towards(&mut self, value: u8) {
        self.state.movement.set_last_direction_moved_towards(value);
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_from_current_direction(&mut self) {
        self.state
            .movement
            .set_last_direction_from_current_direction();
        self.ram[LINK_LAST_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction(&mut self, value: u8) {
        self.state.movement.set_last_direction(value);
        self.ram[LINK_LAST_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mask_last_direction(&mut self, mask: u8) {
        self.state.movement.mask_last_direction(mask);
        self.ram[LINK_LAST_DIRECTION] = self.state.last_direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_from_swim_flags(&mut self) {
        self.state.movement.set_last_direction_from_swim_flags();
        self.ram[LINK_LAST_DIRECTION] = self.state.swim_direction_flags();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_swim_flags_from_last_direction(&mut self) {
        self.state.movement.set_swim_flags_from_last_direction();
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.state.last_direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.state.movement.set_direction(value);
        self.ram[LINK_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_and_last_direction(&mut self, value: u8) {
        self.state.movement.set_direction_and_last_direction(value);
        self.ram[LINK_DIRECTION] = value;
        self.ram[LINK_LAST_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_and_swim_flags(&mut self, value: u8) {
        self.state.movement.set_direction_and_swim_flags(value);
        self.ram[LINK_DIRECTION] = value;
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mask_direction(&mut self, mask: u8) {
        self.state.movement.mask_direction(mask);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_direction_flags(&mut self, flags: u8) {
        self.state.movement.add_direction_flags(flags);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_flags(&mut self, flags: u8) {
        self.state.movement.clear_direction_flags(flags);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_lock(&mut self) {
        self.state.movement.clear_direction_lock();
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_lock_bits(&mut self, mask: u8) {
        self.state.movement.set_direction_lock_bits(mask);
        self.ram[LINK_CANT_CHANGE_DIRECTION] = self.state.direction_lock();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_lock_bits(&mut self, mask: u8) {
        self.state.movement.clear_direction_lock_bits(mask);
        self.ram[LINK_CANT_CHANGE_DIRECTION] = self.state.direction_lock();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_mask_a(&mut self, value: u8) {
        self.state.movement.set_direction_mask_a(value);
        self.ram[LINK_DIRECTION_MASK_A] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_mask_b(&mut self, value: u8) {
        self.state.movement.set_direction_mask_b(value);
        self.ram[LINK_DIRECTION_MASK_B] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn apply_direction_masks(&mut self) {
        self.state.movement.apply_direction_masks();
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn force_direction_from_diag_tile_if_needed(&mut self) {
        self.state
            .movement
            .force_direction_from_diag_tile_if_needed();
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn resolve_orthogonal_direction_count_from_facing(&mut self) {
        self.state
            .movement
            .resolve_orthogonal_direction_count_from_facing();
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = self.state.num_orthogonal_directions();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_moving_floor_direction(&mut self, floor_y: u16, floor_x: u16) {
        self.state
            .movement
            .mark_moving_floor_direction(floor_y, floor_x);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_moved_towards_from_facing(&mut self) {
        self.state
            .movement
            .set_last_direction_moved_towards_from_facing();
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = self.state.last_direction_moved_towards();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_swim_direction_flags(&mut self, direction: u8) {
        self.state.movement.set_swim_direction_flags(direction);
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = direction;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.movement.set_y(value);
        write_le_u16(self.ram, LINK_Y_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.movement.set_x(value);
        write_le_u16(self.ram, LINK_X_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.movement.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.state.movement.set_y_low(value);
        self.ram[LINK_Y_COORD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.state.movement.set_x_low(value);
        self.ram[LINK_X_COORD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_cached(&mut self) {
        let x = self.state.movement.cached_x;
        let y = self.state.movement.cached_y;
        self.state.movement.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_current_position(&mut self) {
        self.state.movement.cached_x = self.state.x();
        self.state.movement.cached_y = self.state.y();
        write_le_u16(self.ram, LINK_Y_COORD_CACHED, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_CACHED, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_copied_position_from_current(&mut self) {
        self.state.movement.copied_x = self.state.x();
        self.state.movement.copied_y = self.state.y();
        write_le_u16(self.ram, LINK_Y_COORD_COPY, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_COPY, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_y_from_previous_position(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_PREV);
        self.state.movement.set_y(y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_previous(&mut self) {
        let x = read_le_u16(self.ram, LINK_X_COORD_PREV);
        let y = read_le_u16(self.ram, LINK_Y_COORD_PREV);
        self.state.movement.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_previous_position_from_current(&mut self) {
        self.state.movement.cache_previous_position_from_current();
        write_le_u16(self.ram, LINK_Y_COORD_PREV, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_PREV, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_previous_position_from_current_xy_order(&mut self) {
        self.state.movement.cache_previous_position_from_current();
        write_le_u16(self.ram, LINK_X_COORD_PREV, self.state.x());
        write_le_u16(self.ram, LINK_Y_COORD_PREV, self.state.y());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_previous_position(&mut self, x: u16, y: u16) {
        self.state.movement.set_previous_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD_PREV, x);
        write_le_u16(self.ram, LINK_Y_COORD_PREV, y);
        self.debug_assert_matches_ram();
    }

    fn axis_offsets(axis: PlayerAxis) -> (usize, usize) {
        match axis {
            PlayerAxis::X => (LINK_X_SUBPIXEL, LINK_X_COORD),
            PlayerAxis::Y => (LINK_Y_SUBPIXEL, LINK_Y_COORD),
            PlayerAxis::Z => (LINK_Z_SUBPIXEL, LINK_Z_COORD),
        }
    }

    fn axis_position(&self, axis: PlayerAxis) -> PlayerPosition {
        // $2c is also the attract throne-fade timer. Keep its shared storage;
        // X/Y coordinates and fractions already belong to native movement.
        let shared_z_subpixel = if axis == PlayerAxis::Z {
            self.ram[LINK_Z_SUBPIXEL]
        } else {
            0
        };
        self.state.movement.position(axis, shared_z_subpixel)
    }

    fn move_axis_by_subpixel_delta(&mut self, axis: PlayerAxis, delta: i16) -> u16 {
        let mut position = self.axis_position(axis);
        position.advance(delta);
        self.state.movement.set_axis_position(axis, position);
        let (subpixel_offset, coord_offset) = Self::axis_offsets(axis);
        self.ram[subpixel_offset] = position.subpixel;
        write_le_u16(self.ram, coord_offset, position.coordinate);
        self.debug_assert_matches_ram();
        position.coordinate
    }

    pub(crate) fn move_axis_by_velocity(&mut self, axis: PlayerAxis, velocity: u8) -> u16 {
        self.move_axis_by_subpixel_delta(axis, PlayerPosition::velocity_delta(velocity))
    }

    pub(crate) fn move_x_by_velocity(&mut self, velocity: u8) -> u16 {
        self.move_axis_by_velocity(PlayerAxis::X, velocity)
    }

    pub(crate) fn move_y_by_velocity(&mut self, velocity: u8) -> u16 {
        self.move_axis_by_velocity(PlayerAxis::Y, velocity)
    }

    pub(crate) fn move_x_by_subpixel_delta(&mut self, delta: u16) -> u16 {
        self.move_axis_by_subpixel_delta(PlayerAxis::X, delta as i16)
    }

    pub(crate) fn move_y_by_subpixel_delta(&mut self, delta: u16) -> u16 {
        self.move_axis_by_subpixel_delta(PlayerAxis::Y, delta as i16)
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
        self.state.movement.set_y(y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_overworld_exit(&mut self) {
        let x = read_le_u16(self.ram, LINK_X_COORD_EXIT_OVERWORLD);
        let y = read_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD);
        self.state.movement.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_safe_return(&mut self) {
        let x = self.state.safe_return_x();
        let y = self.state.safe_return_y();
        self.state.movement.set_position(x, y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_safe_return_position(&mut self, x: u16, y: u16) {
        self.state.movement.store_safe_return_position(x, y);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = x as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (x >> 8) as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_safe_return_low_from_current(&mut self) {
        self.state.movement.store_safe_return_low_from_current();
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.state.safe_return_y() as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.state.safe_return_x() as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_safe_return_y(&mut self, y: u16) {
        self.state.movement.store_safe_return_y(y);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_safe_return_y_low(&mut self, value: u8) {
        self.state.movement.set_safe_return_y_low(value);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_safe_return_position_from_current(&mut self) {
        self.state
            .movement
            .cache_safe_return_position_from_current();
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.state.safe_return_x() as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.state.safe_return_x_high();
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.state.safe_return_y_low();
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.state.safe_return_y_high();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_safe_return_high_from_current(&mut self) {
        self.state.movement.cache_safe_return_high_from_current();
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.state.safe_return_x_high();
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.state.safe_return_y_high();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_page_movement_deltas(&mut self) {
        self.state.movement.clear_page_movement_deltas();
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_page_movement_deltas(&mut self, y_delta: u8, x_delta: u8) {
        self.state
            .movement
            .set_page_movement_deltas(y_delta, x_delta);
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = y_delta;
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = x_delta;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.state
            .movement
            .set_y_page_movement_delta_from_high_position(high);
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = self.state.y_page_movement_delta();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.state
            .movement
            .set_x_page_movement_delta_from_high_position(high);
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = self.state.x_page_movement_delta();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_y_from_hop_origin(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_ORIGINAL);
        self.state.movement.set_y(y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_link_state_block_for_ending(&mut self) {
        self.ram[LINK_Y_COORD..LINK_Y_COORD + 0x70].fill(0);
        self.sync_from_ram();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_oam_x_offset(&mut self, value: u8) {
        self.state.presentation.set_oam_x_offset(value);
        self.ram[PLAYER_OAM_X_OFFSET] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_oam_y_offset(&mut self, value: u8) {
        self.state.presentation.set_oam_y_offset(value);
        self.ram[PLAYER_OAM_Y_OFFSET] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn disable_oam_offsets(&mut self) {
        self.state.presentation.disable_oam_offsets();
        self.ram[PLAYER_OAM_Y_OFFSET] = 0x80;
        self.ram[PLAYER_OAM_X_OFFSET] = 0x80;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.state.movement.set_x_velocity(value);
        self.ram[LINK_X_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.state.movement.set_y_velocity(value);
        self.ram[LINK_Y_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_movement_velocity_from_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.state
            .movement
            .set_movement_velocity_from_delta(x_delta, y_delta);
        self.ram[LINK_X_VELOCITY] = x_delta as u8;
        self.ram[LINK_Y_VELOCITY] = y_delta as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn subtract_axis_velocity_delta(&mut self, horizontal: bool, delta: u8) {
        self.state
            .movement
            .subtract_axis_velocity_delta(horizontal, delta);
        if horizontal {
            self.ram[LINK_X_VELOCITY] = self.state.x_velocity();
        } else {
            self.ram[LINK_Y_VELOCITY] = self.state.y_velocity();
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_movement_velocity_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.state
            .movement
            .add_movement_velocity_delta(x_delta, y_delta);
        self.ram[LINK_X_VELOCITY] = self.state.x_velocity();
        self.ram[LINK_Y_VELOCITY] = self.state.y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_y_velocity_delta(&mut self, y_delta: u8) {
        self.state.movement.add_y_velocity_delta(y_delta);
        self.ram[LINK_Y_VELOCITY] = self.state.y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_movement_velocity(&mut self) {
        self.state.movement.clear_movement_velocity();
        self.ram[LINK_X_VELOCITY] = 0;
        self.ram[LINK_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_movement_subpixels(&mut self) {
        self.state.movement.clear_movement_subpixels();
        self.ram[LINK_X_SUBPIXEL] = 0;
        self.ram[LINK_Y_SUBPIXEL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_movement_velocity_and_direction(&mut self) {
        self.clear_movement_velocity();
        self.state.movement.set_direction(0);
        self.ram[LINK_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_velocity_from_safe_return_delta_unless_ledge_hopping(&mut self) {
        if self.ram[LINK_HANDLER_STATE] != 11 {
            let value = self.state.y_low_delta_from_safe_return();
            self.state.movement.set_y_velocity(value);
            self.ram[LINK_Y_VELOCITY] = value;
            self.debug_assert_matches_ram();
        }
    }

    pub(crate) fn set_x_velocity_from_safe_return_delta(&mut self) {
        let value = self
            .state
            .x_low()
            .wrapping_sub(self.state.safe_return_x() as u8);
        self.state.movement.set_x_velocity(value);
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
            self.state.movement.set_direction(direction);
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
            self.state.movement.set_direction(direction);
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
        self.state.movement.set_actual_x_velocity(value);
        self.ram[LINK_ACTUAL_X_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_y_velocity(&mut self, value: u8) {
        self.state.movement.set_actual_y_velocity(value);
        self.ram[LINK_ACTUAL_Y_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_x_velocity(&mut self) {
        self.state.movement.clear_actual_x_velocity();
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_y_velocity(&mut self) {
        self.state.movement.clear_actual_y_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_velocity_xy(&mut self, x: u8, y: u8) {
        self.state.movement.set_actual_velocity_xy(x, y);
        self.ram[LINK_ACTUAL_X_VELOCITY] = x;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = y;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_velocity_xy(&mut self) {
        self.state.movement.clear_actual_velocity_xy();
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn invert_actual_velocity_xy(&mut self) {
        self.state.movement.invert_actual_velocity_xy();
        self.ram[LINK_ACTUAL_X_VELOCITY] = self.state.actual_x_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = self.state.actual_y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn xor_actual_velocity_xy(&mut self, mask: u8) {
        self.state.movement.xor_actual_velocity_xy(mask);
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
        self.state.movement.set_direction(direction);
        self.ram[LINK_DIRECTION] = direction;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_velocity_from_direction(&mut self, direction: u8, velocity: u8) {
        self.state
            .movement
            .set_actual_velocity_from_direction(direction, velocity);
        self.ram[LINK_ACTUAL_X_VELOCITY] = self.state.actual_x_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = self.state.actual_y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z(&mut self, value: u16) {
        self.state.movement.set_z(value);
        write_le_u16(self.ram, LINK_Z_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z_low(&mut self, value: u8) {
        self.state.movement.set_z_low(value);
        self.ram[LINK_Z_COORD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_z_high(&mut self) {
        self.state.movement.clear_z_high();
        self.ram[LINK_Z_COORD + 1] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_z_low_from_mirror(&mut self) {
        self.state.movement.restore_z_low_from_mirror();
        self.ram[LINK_Z_COORD] = self.ram[LINK_Z_COORD_MIRROR];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_z_from_mirror(&mut self) {
        self.state.movement.restore_z_from_mirror();
        let value = read_le_u16(self.ram, LINK_Z_COORD_MIRROR);
        write_le_u16(self.ram, LINK_Z_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_z_low_to_mirror(&mut self) {
        self.state.movement.cache_z_low_to_mirror();
        self.ram[LINK_Z_COORD_MIRROR] = self.ram[LINK_Z_COORD];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_z_to_mirror(&mut self) {
        self.state.movement.cache_z_to_mirror();
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, self.state.z());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z_mirror(&mut self, value: u16) {
        self.state.movement.set_z_mirror(value);
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_z_mirror_low(&mut self) {
        self.state.movement.clear_z_mirror_low();
        self.ram[LINK_Z_COORD_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_z_mirror_word_low(&mut self) {
        self.state.movement.clear_z_mirror_word_low();
        let value = read_le_u16(self.ram, LINK_Z_COORD_MIRROR) & !0x00ff;
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn force_z_mirror_low_ff(&mut self) {
        self.state.movement.force_z_mirror_low_ff();
        let value = read_le_u16(self.ram, LINK_Z_COORD_MIRROR) | 0x00ff;
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z_and_mirror(&mut self, value: u16) {
        self.state.movement.set_z_and_mirror(value);
        write_le_u16(self.ram, LINK_Z_COORD, value);
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    /// Publish the fraction only; the coordinate delta remains on the source stack.
    pub(crate) fn move_axis_subpixel_only_by_velocity(
        &mut self,
        axis: PlayerAxis,
        velocity: u8,
    ) -> u16 {
        let mut position = self.axis_position(axis);
        let delta = position.advance_subpixel(PlayerPosition::velocity_delta(velocity));
        self.state.movement.set_axis_position(axis, position);
        self.ram[Self::axis_offsets(axis).0] = position.subpixel;
        self.debug_assert_matches_ram();
        delta
    }

    /// Complete the pending coordinate addition without touching the fraction.
    pub(crate) fn apply_axis_pixel_delta(&mut self, axis: PlayerAxis, delta: u16) -> u16 {
        let mut position = self.axis_position(axis);
        position.apply_pixel_delta(delta);
        self.state.movement.set_axis_position(axis, position);
        write_le_u16(self.ram, Self::axis_offsets(axis).1, position.coordinate);
        self.debug_assert_matches_ram();
        position.coordinate
    }

    /// Publish only the low coordinate byte, matching `STA $20,x` before
    /// `STA $21,x`. Return the computed high byte for the suspended store.
    pub(crate) fn apply_axis_pixel_delta_low(&mut self, axis: PlayerAxis, delta: u16) -> u8 {
        let mut position = self.axis_position(axis);
        let high = position.apply_pixel_delta_low(delta);
        self.state.movement.set_axis_position(axis, position);
        self.ram[Self::axis_offsets(axis).1] = position.coordinate as u8;
        self.debug_assert_matches_ram();
        high
    }

    /// Publish the retained high byte, preserving the low byte observed on reentry.
    pub(crate) fn apply_axis_coordinate_high(&mut self, axis: PlayerAxis, high: u8) -> u16 {
        let mut position = self.axis_position(axis);
        position.apply_coordinate_high(high);
        self.state.movement.set_axis_position(axis, position);
        self.ram[Self::axis_offsets(axis).1 + 1] = high;
        self.debug_assert_matches_ram();
        position.coordinate
    }

    pub(crate) fn move_z_by_velocity(&mut self, velocity: u8) -> u16 {
        self.move_axis_by_velocity(PlayerAxis::Z, velocity)
    }

    pub(crate) fn set_actual_z_velocity(&mut self, value: u8) {
        self.state.movement.set_actual_z_velocity(value);
        self.ram[LINK_Z_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_z_velocity_and_copy(&mut self, value: u8) {
        self.state.movement.set_actual_z_velocity_and_copy(value);
        self.ram[LINK_Z_VELOCITY] = value;
        self.ram[LINK_Z_VELOCITY_COPY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_z_velocity_mirror_and_copy(&mut self, value: u8) {
        self.state
            .movement
            .set_actual_z_velocity_mirror_and_copy(value);
        self.ram[LINK_Z_VELOCITY_MIRROR] = value;
        self.ram[LINK_Z_VELOCITY_COPY_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_actual_z_velocity_from_mirror(&mut self) {
        self.state.movement.restore_actual_z_velocity_from_mirror();
        self.ram[LINK_Z_VELOCITY] = self.ram[LINK_Z_VELOCITY_MIRROR];
        self.ram[LINK_Z_VELOCITY_COPY] = self.ram[LINK_Z_VELOCITY_COPY_MIRROR];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_actual_z_velocity_to_mirror(&mut self) {
        self.state.movement.cache_actual_z_velocity_to_mirror();
        self.ram[LINK_Z_VELOCITY_MIRROR] = self.ram[LINK_Z_VELOCITY];
        self.ram[LINK_Z_VELOCITY_COPY_MIRROR] = self.ram[LINK_Z_VELOCITY_COPY];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn prime_airborne_z_velocity(&mut self) {
        self.state.movement.prime_airborne_z_velocity();
        self.ram[LINK_Z_VELOCITY] = 0xff;
        write_le_u16(self.ram, LINK_Z_COORD, 0xffff);
        self.ram[LINK_Z_SUBPIXEL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_actual_z_velocity(&mut self, delta: u8) {
        self.state.movement.decrement_actual_z_velocity(delta);
        self.ram[LINK_Z_VELOCITY] = self.ram[LINK_Z_VELOCITY].wrapping_sub(delta);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ground_state(&mut self) {
        self.state.actions.set_ground_state();
        self.ram[LINK_HANDLER_STATE] = PLAYER_HANDLER_STATE_GROUND;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_running(&mut self) {
        self.state.movement.clear_running();
        self.ram[LINK_IS_RUNNING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn start_running(&mut self) {
        self.state.movement.start_running();
        self.ram[LINK_IS_RUNNING] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_running_state(&mut self, value: u8) {
        self.state.movement.set_running_state(value);
        self.ram[LINK_IS_RUNNING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dash_countdown(&mut self, value: u8) {
        self.state.movement.set_dash_countdown(value);
        self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_dash_countdown(&mut self) -> u8 {
        let value = self.state.movement.increment_dash_countdown();
        self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn decrement_dash_countdown(&mut self) -> u8 {
        let value = self.state.movement.decrement_dash_countdown();
        self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_dash_counter(&mut self, value: u8) {
        self.state.movement.set_dash_counter(value);
        self.ram[LINK_DASH_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn prime_dash_counter(&mut self) {
        self.state.movement.prime_dash_counter();
        self.ram[LINK_DASH_COUNTER] = 64;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_dash_counter_clamped_to_minimum(&mut self, minimum: u8) {
        self.state
            .movement
            .decrement_dash_counter_clamped_to_minimum(minimum);
        self.ram[LINK_DASH_COUNTER] = self.state.dash_counter();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cancel_dash_state(&mut self) {
        self.state.movement.cancel_dash_state();
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[LINK_IS_RUNNING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn immobilize(&mut self) {
        self.state.actions.immobilize();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_immobilized(&mut self) {
        self.state.actions.clear_immobilized();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_button_mask_b_y(&mut self, value: u8) {
        self.state.input.set_button_mask_b_y(value);
        self.ram[BUTTON_MASK_B_Y] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_menu_block_flag(&mut self, value: u8) {
        self.state.actions.set_menu_block_flag(value);
        self.ram[FLAG_BLOCK_LINK_MENU] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_menu_block(&mut self) {
        self.state.actions.clear_menu_block();
        self.ram[FLAG_BLOCK_LINK_MENU] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_menu_block_flag(&mut self) -> u8 {
        let value = self.state.actions.increment_menu_block_flag();
        self.ram[FLAG_BLOCK_LINK_MENU] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn add_button_mask_b_y_bits(&mut self, bits: u8) {
        self.state.input.add_button_mask_b_y_bits(bits);
        self.ram[BUTTON_MASK_B_Y] |= bits;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pull_action_state(&mut self, value: u8) {
        self.state.actions.set_pull_action_state(value);
        self.ram[LINK_PULL_ACTION_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_button_mask_b_y_bits(&mut self, mask: u8) {
        self.state.input.clear_button_mask_b_y_bits(mask);
        self.ram[BUTTON_MASK_B_Y] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_filtered_joypad_h(&mut self, value: u8) {
        self.state.input.set_filtered_joypad_h(value);
        self.ram[FILTERED_JOYPAD_H] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_filtered_joypad_l(&mut self, value: u8) {
        self.state.input.set_filtered_joypad_l(value);
        self.ram[FILTERED_JOYPAD_L] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_filtered_joypad_l_bits(&mut self, bits: u8) {
        self.state.input.clear_filtered_joypad_l_bits(bits);
        self.ram[FILTERED_JOYPAD_L] &= !bits;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1h_last(&mut self, value: u8) {
        self.state.input.set_joypad1h_last(value);
        self.ram[JOYPAD1H_LAST] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1l_last(&mut self, value: u8) {
        self.state.input.set_joypad1l_last(value);
        self.ram[JOYPAD1L_LAST] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1h_last2(&mut self, value: u8) {
        self.state.input.set_joypad1h_last2(value);
        self.ram[JOYPAD1H_LAST2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1l_last2(&mut self, value: u8) {
        self.state.input.set_joypad1l_last2(value);
        self.ram[JOYPAD1L_LAST2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_spin_attack_delay_timer(&mut self, value: u8) {
        self.state.actions.set_spin_attack_delay_timer(value);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_spin_attack_delay_timer(&mut self) -> u8 {
        let value = self.state.actions.decrement_spin_attack_delay_timer();
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_incapacitated_timer(&mut self, value: u8) {
        self.state.actions.set_incapacitated_timer(value);
        self.ram[LINK_INCAPACITATED_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_incapacitated_timer(&mut self) -> u8 {
        let value = self.state.actions.decrement_incapacitated_timer();
        self.ram[LINK_INCAPACITATED_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn reset_elapsed_incapacitated_timer(&mut self) {
        self.state.actions.reset_elapsed_incapacitated_timer();
        self.ram[LINK_INCAPACITATED_TIMER] = self.state.incapacitated_timer();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_visibility_status(&mut self, value: u8) {
        self.state.presentation.set_visibility_status(value);
        self.ram[LINK_VISIBILITY_STATUS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_button_action_flags(&mut self, value: u8) {
        self.state.actions.set_y_button_action_flags(value);
        self.ram[Y_BUTTON_ACTION_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_y_button_action_flag_bits(&mut self, bits: u8) {
        self.state.actions.add_y_button_action_flag_bits(bits);
        self.ram[Y_BUTTON_ACTION_FLAGS] |= bits;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_button_action_step(&mut self, value: u8) {
        self.state.actions.set_y_button_action_step(value);
        self.ram[Y_BUTTON_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_button_action_timer(&mut self, value: u8) {
        self.state.actions.set_y_button_action_timer(value);
        self.ram[Y_BUTTON_ACTION_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_y_button_action_timer(&mut self) -> u8 {
        let value = self.state.actions.decrement_y_button_action_timer();
        self.ram[Y_BUTTON_ACTION_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_defense_flags(&mut self) {
        self.state.actions.clear_defense_flags();
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
        self.state.actions.set_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn or_defense_flags(&mut self, value: u8) {
        self.state.actions.or_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] |= value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn and_defense_flags(&mut self, value: u8) {
        self.state.actions.and_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] &= value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_receipt_method(&mut self, value: u8) {
        self.state.actions.set_item_receipt_method(value);
        self.ram[ITEM_RECEIPT_METHOD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_below(&mut self, value: u8) {
        self.state.movement.set_tile_below(value);
        self.ram[LINK_TILE_BELOW] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_action_index(&mut self, value: u8) {
        self.state.movement.set_tile_action_index(value);
        self.ram[TILE_ACTION_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_coll_flag(&mut self, value: u8) {
        self.state.movement.set_tile_coll_flag(value);
        self.ram[TILE_COLL_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_force_move_any_direction(&mut self, value: u16) {
        self.state.movement.set_force_move_any_direction(value);
        write_le_u16(self.ram, FORCE_MOVE_ANY_DIRECTION, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_recoil_timer(&mut self, value: u8) {
        self.state.movement.set_recoil_timer(value);
        self.ram[LINK_RECOIL_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_recoil_timer(&mut self) -> u8 {
        let value = self.state.movement.increment_recoil_timer();
        self.ram[LINK_RECOIL_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn tick_jump_ledge_timer_or_reset(&mut self) -> bool {
        let reset = self.state.movement.tick_jump_ledge_timer_or_reset();
        self.ram[LINK_TIMER_JUMP_LEDGE] = self.state.jump_ledge_timer();
        self.debug_assert_matches_ram();
        reset
    }

    pub(crate) fn reset_jump_ledge_timer(&mut self) {
        self.state.movement.reset_jump_ledge_timer();
        self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_about_to_jump_off_ledge(&mut self) {
        self.state.movement.clear_about_to_jump_off_ledge();
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_about_to_jump_off_ledge(&mut self) {
        self.state.movement.increment_about_to_jump_off_ledge();
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = self.state.about_to_jump_off_ledge();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_push_fatigue_timer(&mut self) -> u8 {
        let value = self.state.movement.decrement_push_fatigue_timer();
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_push_fatigue_timer(&mut self, value: u8) {
        self.state.movement.set_push_fatigue_timer(value);
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_push_fatigue_timer(&mut self) {
        self.state.movement.reset_push_fatigue_timer();
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_near_moveable_statue(&mut self) {
        self.state.movement.clear_near_moveable_statue();
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_near_moveable_statue(&mut self) {
        self.state.movement.mark_near_moveable_statue();
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pull_for_rupees_sprite_need(&mut self) {
        self.state.actions.clear_pull_for_rupees_sprite_need();
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pull_for_rupees_sprite_need(&mut self) {
        self.state.actions.set_pull_for_rupees_sprite_need();
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pit_correction(&mut self) {
        self.state.movement.clear_pit_correction();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pit_correction_active(&mut self) {
        self.state.movement.set_pit_correction_active();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pit_correction_timer(&mut self, value: u8) {
        self.state.movement.set_pit_correction_timer(value);
        self.ram[PIT_CORRECTION_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_pit_correction_timer(&mut self) {
        self.state.movement.increment_pit_correction_timer();
        self.ram[PIT_CORRECTION_TIMER] = self.state.pit_correction_timer();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_moving_against_diag_deadlocked(&mut self, value: u8) {
        self.state
            .movement
            .set_moving_against_diag_deadlocked(value);
        self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_electrocute_on_touch(&mut self) {
        self.state.actions.clear_electrocute_on_touch();
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_electrocute_on_touch(&mut self, value: u8) {
        self.state.actions.set_electrocute_on_touch(value);
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_faint_animation_active(&mut self) {
        self.state.presentation.clear_faint_animation_active();
        self.ram[LINK_FAINT_ANIMATION_ACTIVE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_faint_animation_active(&mut self, value: u8) {
        self.state.presentation.set_faint_animation_active(value);
        self.ram[LINK_FAINT_ANIMATION_ACTIVE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_debug_value_1(&mut self) {
        self.state.actions.clear_item_debug_value_1();
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_hookshot_grave_latch(&mut self) {
        self.state.actions.clear_hookshot_grave_latch();
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hookshot_grave_latch(&mut self) {
        self.state.actions.set_hookshot_grave_latch();
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dash_noise_request(&mut self) {
        self.state.presentation.set_dash_noise_request();
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_dash_noise_request(&mut self) {
        self.state.presentation.clear_dash_noise_request();
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_cape_mode(&mut self) {
        self.state.actions.clear_cape_mode();
        self.ram[LINK_CAPE_MODE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cape_mode(&mut self, value: u8) {
        self.state.actions.set_cape_mode(value);
        self.ram[LINK_CAPE_MODE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cape_decrement_counter(&mut self, value: u8) {
        self.state.actions.set_cape_decrement_counter(value);
        self.ram[CAPE_DECREMENT_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_cape_decrement_counter(&mut self) {
        self.state.actions.decrement_cape_decrement_counter();
        self.ram[CAPE_DECREMENT_COUNTER] = self.state.cape_decrement_counter();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_transforming(&mut self) {
        self.state.actions.clear_transforming();
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_transforming(&mut self) {
        self.state.actions.set_transforming();
        self.ram[LINK_IS_TRANSFORMING] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sword_delay_timer(&mut self) {
        self.state.actions.clear_sword_delay_timer();
        self.ram[LINK_SWORD_DELAY_TIMER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sword_delay_timer(&mut self, value: u8) {
        self.state.actions.set_sword_delay_timer(value);
        self.ram[LINK_SWORD_DELAY_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_sword_delay_timer(&mut self) -> u8 {
        let value = self.state.actions.decrement_sword_delay_timer();
        self.ram[LINK_SWORD_DELAY_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_spin_offsets(&mut self, value: u8) {
        self.state.presentation.set_spin_offsets(value);
        self.ram[LINK_SPIN_OFFSETS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_somaria_platform_state(&mut self) {
        self.state.movement.clear_somaria_platform_state();
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_somaria_platform_state(&mut self, value: u8) {
        self.state.movement.set_somaria_platform_state(value);
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_spin_attack_step_counter(&mut self) {
        self.state.actions.clear_spin_attack_step_counter();
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_spin_attack_step_counter(&mut self) -> u8 {
        let value = self.state.actions.increment_spin_attack_step_counter();
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_spin_attack_sound_latch(&mut self, value: u8) {
        self.state.actions.set_spin_attack_sound_latch(value);
        self.ram[SPIN_ATTACK_SOUND_LATCH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_spin_attack_sound_latch(&mut self) {
        self.state.actions.clear_spin_attack_sound_latch();
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_state_for_spin_attack(&mut self, value: u8) {
        self.state.actions.set_state_for_spin_attack(value);
        self.ram[STATE_FOR_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_state_for_spin_attack(&mut self) {
        self.state.actions.clear_state_for_spin_attack();
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_immobilized_flag(&mut self) -> u8 {
        let value = self.state.actions.increment_immobilized_flag();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_immobilized_flag(&mut self, value: u8) {
        self.state.actions.set_immobilized_flag(value);
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
        self.state.actions.clear_action_handler_timer();
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_action_handler_timer(&mut self, value: u8) {
        self.state.actions.set_action_handler_timer(value);
        self.ram[PLAYER_HANDLER_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_action_handler_timer(&mut self) -> u8 {
        let value = self.state.actions.increment_action_handler_timer();
        self.ram[PLAYER_HANDLER_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_doorway_state(&mut self) {
        self.state.movement.clear_doorway_state();
        self.ram[IS_STANDING_IN_DOORWAY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_doorway_state(&mut self, value: u8) {
        self.state.movement.set_doorway_state(value);
        self.ram[IS_STANDING_IN_DOORWAY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_blink_countdown(&mut self) {
        self.state.presentation.clear_blink_countdown();
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_blink_countdown(&mut self, value: u8) {
        self.state.presentation.set_blink_countdown(value);
        self.ram[COUNTDOWN_FOR_BLINK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_blink_countdown(&mut self) -> u8 {
        let value = self.state.presentation.decrement_blink_countdown();
        self.ram[COUNTDOWN_FOR_BLINK] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_spin_animation_step_counter(&mut self, value: u8) {
        self.state
            .presentation
            .set_spin_animation_step_counter(value);
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_spin_animation_step_counter(&mut self) -> u8 {
        let value = self
            .state
            .presentation
            .increment_spin_animation_step_counter();
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_spin_animation_step_counter(&mut self) {
        self.state.presentation.clear_spin_animation_step_counter();
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_button_b_frames(&mut self) {
        self.state.input.clear_button_b_frames();
        self.ram[BUTTON_B_FRAMES] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_button_b_frames(&mut self, value: u8) {
        self.state.input.set_button_b_frames(value);
        self.ram[BUTTON_B_FRAMES] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_button_b_frames_word(&mut self, value: u16) {
        self.state.set_button_b_frames_word(value);
        write_le_u16(self.ram, BUTTON_B_FRAMES, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_button_b_frames(&mut self) -> u8 {
        let value = self.state.input.increment_button_b_frames();
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
        self.state.presentation.clear_animation_step();
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_animation_step(&mut self, value: u8) {
        self.state.presentation.set_animation_step(value);
        self.ram[LINK_ANIMATION_STEPS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_opening_pose(&mut self) {
        self.state.presentation.increment_opening_pose();
        self.ram[LINK_POSE_DURING_OPENING] = self.state.opening_pose();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_animation_step(&mut self, wrap_at: u8, wrap_to: u8) {
        self.state
            .presentation
            .advance_animation_step(wrap_at, wrap_to);
        self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
        if self.ram[LINK_ANIMATION_STEPS] == wrap_at {
            self.ram[LINK_ANIMATION_STEPS] = wrap_to;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_animation_step_at_least(&mut self, wrap_at: u8, wrap_to: u8) {
        self.state
            .presentation
            .advance_animation_step_at_least(wrap_at, wrap_to);
        self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
        if self.ram[LINK_ANIMATION_STEPS] >= wrap_at {
            self.ram[LINK_ANIMATION_STEPS] = wrap_to;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_idle_swim_animation(&mut self) {
        self.state.presentation.animation_step &= 1;
        self.state.presentation.frame_change_counter =
            self.state.presentation.frame_change_counter.wrapping_add(1);
        if self.state.presentation.frame_change_counter >= 16 {
            self.state.presentation.frame_change_counter = 0;
            self.state.movement.swim_stroke_anim_step = 0;
            self.state.presentation.animation_step =
                (self.state.presentation.animation_step & 1) ^ 1;
        }
        self.ram[LINK_ANIMATION_STEPS] = self.state.presentation.animation_step;
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.state.presentation.frame_change_counter;
        self.ram[SWIM_STROKE_ANIM_STEP] = self.state.movement.swim_stroke_anim_step;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_active_swim_animation(&mut self, stroke_steps: &[u8; 4]) {
        self.state.presentation.frame_change_counter =
            self.state.presentation.frame_change_counter.wrapping_add(1);
        if self.state.presentation.frame_change_counter >= 8 {
            self.state.presentation.frame_change_counter = 0;
            self.state.presentation.animation_step =
                self.state.presentation.animation_step.wrapping_add(1) & 3;
            self.state.movement.swim_stroke_anim_step =
                stroke_steps[self.state.presentation.animation_step as usize];
        }
        self.ram[LINK_ANIMATION_STEPS] = self.state.presentation.animation_step;
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.state.presentation.frame_change_counter;
        self.ram[SWIM_STROKE_ANIM_STEP] = self.state.movement.swim_stroke_anim_step;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_animation_step_if_at_least(&mut self, threshold: u8) {
        self.state
            .presentation
            .clear_animation_step_if_at_least(threshold);
        self.ram[LINK_ANIMATION_STEPS] = self.state.animation_step();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn subtract_animation_step_if_at_least(&mut self, threshold: u8, delta: u8) {
        self.state
            .presentation
            .subtract_animation_step_if_at_least(threshold, delta);
        self.ram[LINK_ANIMATION_STEPS] = self.state.animation_step();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_water_ripple_or_grass_state(&mut self) {
        self.state.presentation.clear_water_ripple_or_grass_state();
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_water_ripple_or_grass_state(&mut self, value: u8) {
        self.state
            .presentation
            .set_water_ripple_or_grass_state(value);
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_secondary_water_grass_timer(&mut self, value: u8) {
        self.state
            .presentation
            .set_secondary_water_grass_timer(value);
        self.ram[SECONDARY_WATER_GRASS_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swim_fast_state(&mut self) {
        self.state.movement.clear_swim_fast_state();
        self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_swim_stroke_state(&mut self) {
        self.state.movement.reset_swim_stroke_state();
        self.ram[SWIMMING_COUNTDOWN] = 0;
        self.ram[LINK_SWIM_HARD_STROKE] = 0;
        self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn start_hard_swim_stroke(&mut self, hard_stroke: u8) {
        self.state.movement.start_hard_swim_stroke(hard_stroke);
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
        self.state.movement.tick_hard_swim_stroke(countdown);
        self.ram[LINK_MAYBE_SWIM_FASTER] = self.state.swim_fast_state();
        self.ram[LINK_SWIM_HARD_STROKE] = self.state.hard_swim_stroke();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_pickup_in_progress(&mut self, value: u8) {
        self.state.actions.set_item_pickup_in_progress(value);
        self.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hookshot_bg_check_off_timer(&mut self, value: u8) {
        self.state.actions.set_hookshot_bg_check_off_timer(value);
        self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_hookshot_bg_check_off_timer(&mut self) {
        self.state.actions.decrement_hookshot_bg_check_off_timer();
        self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = self.state.hookshot_bg_check_off_timer();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_selected_rod(&mut self, value: u8) {
        self.state.actions.set_selected_rod(value);
        self.ram[EQ_SELECTED_ROD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_flute_countdown(&mut self, value: u8) {
        self.state.actions.set_flute_countdown(value);
        self.ram[FLUTE_COUNTDOWN] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_flute_countdown(&mut self) {
        self.state.actions.decrement_flute_countdown();
        self.ram[FLUTE_COUNTDOWN] = self.state.flute_countdown();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_flute_countdown(&mut self) {
        self.state.actions.clear_flute_countdown();
        self.ram[FLUTE_COUNTDOWN] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_index_of_dashing_sfx(&mut self) {
        self.state.presentation.clear_index_of_dashing_sfx();
        self.ram[INDEX_OF_DASHING_SFX] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_index_of_dashing_sfx(&mut self) {
        self.state.presentation.decrement_index_of_dashing_sfx();
        self.ram[INDEX_OF_DASHING_SFX] = self.state.index_of_dashing_sfx();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_water_ripple_or_grass_state(&mut self) -> u8 {
        let value = self
            .state
            .presentation
            .increment_water_ripple_or_grass_state();
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_primary_water_grass_timer(&mut self, value: u8) {
        self.state.presentation.set_primary_water_grass_timer(value);
        self.ram[PRIMARY_WATER_GRASS_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enter_deep_water_state(&mut self) {
        self.state.movement.enter_deep_water_state();
        self.ram[LINK_IS_IN_DEEP_WATER] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_deep_water_state(&mut self) {
        self.state.movement.clear_deep_water_state();
        self.ram[LINK_IS_IN_DEEP_WATER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_conveyor_belt_state(&mut self) {
        self.state.movement.clear_conveyor_belt_state();
        self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_conveyor_belt_state(&mut self, value: u8) {
        self.state.movement.set_conveyor_belt_state(value);
        self.ram[LINK_ON_CONVEYOR_BELT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_misc_bugfix_movement_state(&mut self) {
        self.state.movement.clear_misc_bugfix_movement_state();
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        self.ram[LINK_FLAG_MOVING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_action_step_var(&mut self) {
        self.state.actions.clear_item_action_step_var();
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_current_quadrants(&mut self) {
        self.state.movement.cache_current_quadrants();
        self.ram[LINK_QUADRANT_X_CACHED] = self.ram[LINK_QUADRANT_X];
        self.ram[LINK_QUADRANT_Y_CACHED] = self.ram[LINK_QUADRANT_Y];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_quadrants_from_cached(&mut self) {
        self.state.movement.restore_quadrants_from_cached();
        self.ram[LINK_QUADRANT_X] = self.ram[LINK_QUADRANT_X_CACHED];
        self.ram[LINK_QUADRANT_Y] = self.ram[LINK_QUADRANT_Y_CACHED];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_frame_change_counter(&mut self, delay: u8) -> bool {
        let advanced = self.state.presentation.advance_frame_change_counter(delay);
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.state.presentation.frame_change_counter;
        self.debug_assert_matches_ram();
        advanced
    }

    pub(crate) fn clear_frame_change_counter(&mut self) {
        self.state.presentation.clear_frame_change_counter();
        self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_oam_state_timer(&mut self, value: u8) {
        self.state.presentation.set_sprite_oam_state_timer(value);
        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_recoil_z_velocity_for_dungeon_reset(&mut self, value: u8) {
        self.state
            .movement
            .set_recoil_z_velocity_for_dungeon_reset(value);
        self.ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_recoil_z_velocity(&mut self, value: u8) {
        self.state.movement.set_recoil_z_velocity(value);
        self.ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = value;
        self.ram[LINK_Z_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_sprite_oam_state_timer(&mut self) -> u8 {
        let value = self.state.presentation.decrement_sprite_oam_state_timer();
        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn mark_pit_landing_oam_state(&mut self) {
        self.state.presentation.mark_pit_landing_oam_state();
        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_whirlpool_trigger(&mut self) {
        self.state.movement.set_whirlpool_trigger();
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_whirlpool_trigger(&mut self) {
        self.state.movement.clear_whirlpool_trigger();
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn prevent_movement(&mut self) {
        self.state.movement.prevent_movement();
        self.ram[LINK_PREVENT_FROM_MOVING] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_prevent_movement(&mut self) {
        self.state.movement.clear_prevent_movement();
        self.ram[LINK_PREVENT_FROM_MOVING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hop_origin_delta_from_y(&mut self, y: u16) -> u16 {
        let value = self.state.movement.set_hop_origin_delta_from_y(y);
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
            .movement
            .set_movement_velocity_from_position_delta(x, y, old_x, old_y);
        self.ram[LINK_Y_VELOCITY] = self.state.movement.y_velocity;
        self.ram[LINK_X_VELOCITY] = self.state.movement.x_velocity;
        self.debug_assert_matches_ram();
    }

    /// Link_HandleVelocity's four STZ stores in source order ($27,$28,$68,$69).
    pub(crate) fn clear_velocity_selection_store(&mut self, store: u8) {
        match store {
            0 => {
                self.state.movement.actual_y_velocity = 0;
                self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
            }
            1 => {
                self.state.movement.actual_x_velocity = 0;
                self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
            }
            2 => {
                self.state.movement.y_page_movement_delta = 0;
                self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
            }
            3 => {
                self.state.movement.x_page_movement_delta = 0;
                self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
            }
            _ => panic!("velocity-selection clear store is outside the source sequence"),
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_velocity_and_page_movement_deltas(&mut self) {
        self.state
            .movement
            .clear_actual_velocity_and_page_movement_deltas();
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_moving_floor_position(&mut self, x: u16, y: u16) {
        self.state.movement.cache_moving_floor_position(x, y);
        write_le_u16(self.ram, RELATED_TO_MOVING_FLOOR_Y, y);
        write_le_u16(self.ram, RELATED_TO_MOVING_FLOOR_X, x);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_incapacitated_camera_timer(&mut self) -> u8 {
        let value = self.state.movement.decrement_incapacitated_camera_timer();
        self.ram[LINK_INCAPACITATED_CAMERA_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn increment_pull_action_state(&mut self) {
        self.state.actions.increment_pull_action_state();
        self.ram[LINK_PULL_ACTION_STATE] = self.state.actions.pull_action_state;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_holding_timer(&mut self, value: u8) {
        self.state.actions.set_item_holding_timer(value);
        self.ram[LINK_ITEM_HOLDING_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swim_movement_velocity(&mut self) {
        self.state.movement.clear_swim_movement_velocity();
        self.ram[LINK_Y_VELOCITY] = 0;
        self.ram[LINK_X_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_sleep_in_bed_state(&mut self) {
        self.state.presentation.increment_sleep_in_bed_state();
        self.ram[PLAYER_SLEEP_IN_BED_STATE] = self.state.presentation.sleep_in_bed_state;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cached_tile_action_index(&mut self, value: u8) {
        self.state.movement.set_cached_tile_action_index(value);
        self.ram[CACHED_TILE_ACTION_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swimming_countdown(&mut self) {
        self.state.movement.clear_swimming_countdown();
        self.ram[SWIMMING_COUNTDOWN] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_ancilla_interactive_reset_flag(&mut self) {
        self.state.actions.clear_ancilla_interactive_reset_flag();
        self.ram[ANCILLA_INTERACTIVE_RESET_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_force_move_high_byte(&mut self) {
        self.state.movement.clear_force_move_high_byte();
        write_le_u16(
            self.ram,
            FORCE_MOVE_ANY_DIRECTION,
            self.state.movement.force_move_any_direction,
        );
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_pickup_flag_cached(&mut self, value: u8) {
        self.state.actions.set_sprite_pickup_flag_cached(value);
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_swim_stroke_frame_counter(&mut self, offset: usize, value: u16) {
        self.state
            .movement
            .set_swim_stroke_frame_counter(offset, value);
        write_le_u16(self.ram, SWIM_STROKE_FRAME_COUNTER + offset, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_magic_spell_player_lock(&mut self) {
        self.state.actions.clear_magic_spell_player_lock();
        self.ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_player_pose_draw_counter(&mut self) {
        self.state.presentation.clear_player_pose_draw_counter();
        self.ram[PLAYER_POSE_DRAW_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_player_pose_draw_counter(&mut self) {
        self.state.presentation.increment_player_pose_draw_counter();
        self.ram[PLAYER_POSE_DRAW_COUNTER] = self.state.presentation.player_pose_draw_counter;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_player_special_draw_flag(&mut self) {
        self.state.presentation.clear_player_special_draw_flag();
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_player_special_draw_flag(&mut self, value: u8) {
        self.state.presentation.set_player_special_draw_flag(value);
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_bit9_of_xcoord_word(&mut self, value: u16) {
        self.state.movement.set_bit9_of_xcoord_word(value);
        write_le_u16(self.ram, BIT9_OF_XCOORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_item_action_step_var(&mut self) -> u8 {
        let value = self.state.actions.increment_item_action_step_var();
        self.ram[LINK_ITEM_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn advance_item_action_step_var_wrapping_7_to_1(&mut self) -> u8 {
        let value = self
            .state
            .actions
            .advance_item_action_step_var_wrapping_7_to_1();
        self.ram[LINK_ITEM_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_given_damage(&mut self) {
        self.state.actions.clear_given_damage();
        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.debug_assert_matches_ram();
    }

    #[track_caller]
    pub(crate) fn set_given_damage(&mut self, value: u8) {
        if crate::debug_env::var_os("ZELDA3_DEBUG_LINK_DAMAGE").is_some() {
            eprintln!(
                "[LINKDMG] given_damage={value:#x} caller={} cur_sprite={:#x}",
                std::panic::Location::caller(),
                self.ram[0x0fa0],
            );
        }
        self.state.actions.set_given_damage(value);
        self.ram[LINK_GIVE_DAMAGE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_in_hand(&mut self, value: u8) {
        self.state.actions.set_item_in_hand(value);
        self.ram[LINK_ITEM_IN_HAND] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_in_hand(&mut self) {
        self.state.actions.clear_item_in_hand();
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_in_hand_bits(&mut self, mask: u8) {
        self.state.actions.clear_item_in_hand_bits(mask);
        self.ram[LINK_ITEM_IN_HAND] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_position_mode(&mut self) {
        self.state.movement.clear_position_mode();
        self.ram[LINK_POSITION_MODE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_position_mode(&mut self, value: u8) {
        self.state.movement.set_position_mode(value);
        self.ram[LINK_POSITION_MODE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_position_mode_bits(&mut self, mask: u8) {
        self.state.movement.set_position_mode_bits(mask);
        self.ram[LINK_POSITION_MODE] |= mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_position_mode_bits(&mut self, mask: u8) {
        self.state.movement.clear_position_mode_bits(mask);
        self.ram[LINK_POSITION_MODE] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_action_step_var(&mut self, value: u8) {
        self.state.actions.set_item_action_step_var(value);
        self.ram[LINK_ITEM_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_throw_oam_state_index(&mut self, value: u8) {
        self.state.presentation.set_throw_oam_state_index(value);
        self.ram[LINK_THROW_OAM_STATE_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_action_debug_value_2(&mut self, value: u8) {
        self.state.actions.set_item_action_debug_value_2(value);
        self.ram[LINK_DEBUG_VALUE_2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_current_item_y(&mut self, value: u8) {
        self.state.actions.set_current_item_y(value);
        self.ram[LINK_CURRENT_ITEM_Y] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_current_item_active(&mut self, value: u8) {
        self.state.actions.set_current_item_active(value);
        self.ram[LINK_CURRENT_ITEM_ACTIVE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_receive_item_index(&mut self, value: u8) {
        self.state.actions.set_receive_item_index(value);
        self.ram[LINK_RECEIVE_ITEM_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_graphics_index_word(&mut self, value: u16) {
        self.state
            .presentation
            .set_link_dma_graphics_index_word(value);
        write_le_u16(self.ram, LINK_DMA_GRAPHICS_INDEX, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_left_sprite_bank_word(&mut self, value: u16) {
        self.state
            .presentation
            .set_link_dma_left_sprite_bank_word(value);
        write_le_u16(self.ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_right_sprite_bank_word(&mut self, value: u16) {
        self.state
            .presentation
            .set_link_dma_right_sprite_bank_word(value);
        write_le_u16(self.ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_link_dma_sprite_banks(&mut self) {
        self.state.presentation.clear_link_dma_sprite_banks();
        write_le_u16(self.ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX, 0);
        write_le_u16(self.ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_palette_bits_of_oam_word(&mut self, value: u16) {
        self.state.presentation.set_palette_bits_of_oam_word(value);
        write_le_u16(self.ram, LINK_PALETTE_BITS_OF_OAM, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_link_dma_animation_cycle(&mut self, countdown: u16) {
        // Write-through: the Link DMA animation words are RAM-resident (see the
        // note in FollowerLinkState::write_to_ram).
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, countdown);
        write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, 0);
        write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sword_dma_graphics_index(&mut self, value: u8) {
        self.state.presentation.set_sword_dma_graphics_index(value);
        self.ram[LINK_DMA_SWORD_GRAPHICS_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_shield_dma_graphics_index(&mut self, value: u8) {
        self.state.presentation.set_shield_dma_graphics_index(value);
        self.ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_staging_index(&mut self, value: u8) {
        self.state.presentation.set_link_dma_staging_index(value);
        self.ram[LINK_DMA_STAGING_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    /// C: `scratch_1 = j` in player_oam — a call-local scratch write with no native
    /// model behind it (see FollowerLinkState::write_to_ram). Write RAM, like C.
    pub(crate) fn set_link_sprite_index_scratch(&mut self, value: u16) {
        write_le_u16(self.ram, SCRATCH_1, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hop_origin_coord(&mut self, value: u16) {
        self.state.movement.set_hop_origin_coord(value);
        write_le_u16(self.ram, LINK_Y_COORD_ORIGINAL, value);
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
        self.state.actions.clear_ancilla_pickup_flag();
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ancilla_pickup_flag(&mut self, value: u8) {
        self.state.actions.set_ancilla_pickup_flag(value);
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sprite_pickup_flag(&mut self) {
        self.state.actions.clear_sprite_pickup_flag();
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_pickup_flag(&mut self, value: u8) {
        self.state.actions.set_sprite_pickup_flag(value);
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hookshot_interlock(&mut self, value: u8) {
        self.state.actions.set_hookshot_interlock(value);
        self.ram[RELATED_TO_HOOKSHOT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_hookshot_interlock(&mut self) {
        self.state.actions.clear_hookshot_interlock();
        self.ram[RELATED_TO_HOOKSHOT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn xor_hookshot_interlock(&mut self, mask: u8) {
        self.state.actions.xor_hookshot_interlock(mask);
        self.ram[RELATED_TO_HOOKSHOT] ^= mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_grabbing_wall(&mut self) {
        self.state.movement.clear_grabbing_wall();
        self.ram[LINK_GRABBING_WALL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_grabbing_wall(&mut self, value: u8) {
        self.state.movement.set_grabbing_wall(value);
        self.ram[LINK_GRABBING_WALL] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enable_cutscene_immunity(&mut self) {
        self.state.actions.enable_cutscene_immunity();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_damage_disable_timer(&mut self, value: u8) {
        self.state.actions.set_sprite_damage_disable_timer(value);
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sprite_damage_disable_timer(&mut self) {
        self.state.actions.clear_sprite_damage_disable_timer();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_sprite_damage_disable_timer(&mut self) {
        self.state.actions.increment_sprite_damage_disable_timer();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = self.ram[LINK_DISABLE_SPRITE_DAMAGE].wrapping_add(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_hold_pose(&mut self, value: u8) {
        self.state.presentation.set_item_hold_pose(value);
        self.ram[LINK_POSE_FOR_ITEM] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_hold_pose(&mut self) {
        self.state.presentation.clear_item_hold_pose();
        self.ram[LINK_POSE_FOR_ITEM] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn force_hold_sword_up(&mut self) {
        self.state.presentation.force_hold_sword_up();
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_force_hold_sword_up(&mut self) {
        self.state.presentation.clear_force_hold_sword_up();
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_near_pit_state(&mut self, value: u8) {
        self.state.movement.set_near_pit_state(value);
        self.ram[PLAYER_NEAR_PIT_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_near_pit_state(&mut self) {
        self.state.movement.clear_near_pit_state();
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pit_data_index(&mut self, value: u8) {
        self.state.movement.set_pit_data_index(value);
        self.ram[PLAYER_PIT_DATA_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pit_data_index(&mut self) {
        self.state.movement.clear_pit_data_index();
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_pit_data_index(&mut self) -> u8 {
        let value = self.state.movement.advance_pit_data_index();
        self.ram[PLAYER_PIT_DATA_INDEX] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn begin_pit_check(&mut self) {
        self.state.movement.begin_pit_check();
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cape_transform_timer(&mut self, value: u8) {
        self.state.actions.set_cape_transform_timer(value);
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn tick_cape_transform_timer(&mut self) -> u8 {
        let value = self.state.actions.tick_cape_transform_timer();
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_cape_transform_timer(&mut self) {
        self.state.actions.clear_cape_transform_timer();
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_mirror(&mut self) {
        self.state.actions.clear_bunny_mirror();
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_body_state(&mut self) {
        self.state.actions.clear_bunny_body_state();
        self.ram[LINK_IS_BUNNY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_bunny_state(&mut self, value: u8) {
        self.state.actions.set_bunny_state(value);
        self.ram[LINK_IS_BUNNY] = value;
        self.ram[LINK_IS_BUNNY_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_transform_flags(&mut self) {
        self.state.actions.clear_bunny_transform_flags();
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_transform_after_moon_pearl(&mut self) {
        self.state.actions.clear_bunny_transform_after_moon_pearl();
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        // C clears only the low byte here; preserve the high byte in both native
        // state and RAM projection.
        self.ram[LINK_TIMER_TEMPBUNNY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_transform_poof_need_and_temp_bunny_timer(&mut self) {
        self.state
            .actions
            .clear_transform_poof_need_and_temp_bunny_timer();
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_temp_bunny_timer(&mut self) {
        self.state.actions.clear_temp_bunny_timer();
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_temp_bunny_timer(&mut self, value: u16) {
        self.state.actions.set_temp_bunny_timer(value);
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_temp_bunny_timer(&mut self) -> u16 {
        let value = self.state.actions.decrement_temp_bunny_timer();
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
        self.state.actions.set_auxiliary_state(value);
        self.ram[LINK_AUXILIARY_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_auxiliary_state(&mut self) {
        self.state.actions.clear_auxiliary_state();
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_state_bits(&mut self, value: u8) {
        self.state.actions.set_state_bits(value);
        self.ram[LINK_STATE_BITS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_state_bits(&mut self) {
        self.state.actions.clear_state_bits();
        self.ram[LINK_STATE_BITS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lifting_or_carrying_state(&mut self) {
        self.state.actions.clear_lifting_or_carrying_state();
        self.ram[LINK_STATE_BITS] &= !0x80;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn keep_only_lifting_or_carrying_state(&mut self) {
        self.state.actions.keep_only_lifting_or_carrying_state();
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
        self.publish_cleared_state_item_and_grab_flags();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_picking_throw_state(&mut self) {
        self.state.actions.clear_picking_throw_state();
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_picking_throw_state(&mut self, value: u8) {
        self.state.actions.set_picking_throw_state(value);
        self.ram[LINK_PICKING_THROW_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn start_lift_throw_state(&mut self) {
        self.state.actions.start_lift_throw_state();
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

    fn publish_cleared_item_action_sequence(&mut self) {
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
    }

    fn publish_cleared_state_item_and_grab_flags(&mut self) {
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
    }

    fn publish_cancelled_sword_and_item_usage(&mut self) {
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x81;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = self.state.direction_lock();
    }

    pub(crate) fn cancel_sword_and_item_usage(&mut self) {
        self.state.cancel_sword_and_item_usage();
        self.publish_cancelled_sword_and_item_usage();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn finish_recoil_landing(&mut self) {
        self.state.finish_recoil_landing();
        write_le_u16(self.ram, LINK_Z_COORD, 0);
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    fn publish_quiet_cape_removal(&mut self) {
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 32;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[LINK_CAPE_MODE] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
    }

    pub(crate) fn unequip_cape_quietly(&mut self) {
        self.state.unequip_cape_quietly();
        self.publish_quiet_cape_removal();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swim_stroke_counters(&mut self) {
        self.state.clear_swim_stroke_counters();
        write_le_u16(self.ram, SWIM_STROKE_FRAME_COUNTER, 0);
        write_le_u16(self.ram, SWIM_STROKE_FRAME_COUNTER + 2, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn initialize_link_action_state(&mut self) {
        self.state.initialize_link_action_state();
        self.ram[LINK_FACING] = 2;
        self.ram[LINK_LAST_DIRECTION] = 0;
        self.publish_cleared_item_action_sequence();
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.publish_cleared_state_item_and_grab_flags();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_action_state(&mut self) {
        self.state.reset_action_state();
        self.ram[TILE_ACTION_INDEX] = 0;
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[TILE_COLL_FLAG] = 0;
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
        self.ram[LINK_SWORD_DELAY_TIMER] = 0;
        write_le_u16(self.ram, TILEDETECT_MISC_TILES, 0);
        self.publish_cleared_item_action_sequence();
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.publish_cleared_state_item_and_grab_flags();
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
        self.publish_cancelled_sword_and_item_usage();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn finish_initialization(&mut self) {
        self.state.finish_initialization();
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
        self.publish_quiet_cape_removal();
        self.publish_cancelled_sword_and_item_usage();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_movement_and_transformation_state(&mut self) {
        self.state.reset_movement_and_transformation_state();
        self.ram[LINK_LAST_DIRECTION] = 0;
        self.ram[LINK_DIRECTION] = 0;
        self.ram[LINK_FLAG_MOVING] = 0;
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.ram[PLAYER_RESET_ANCILLA_WORK_BYTE_24] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        // Single-byte store (low byte only), matching C — the high byte of the temp-bunny
        // timer is preserved (see reset_movement_and_transformation_state on the state).
        self.ram[LINK_TIMER_TEMPBUNNY] = 0;
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.ram[BIT9_OF_XCOORD] = 0;
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.ram[LINK_SPIN_OFFSETS] = 0;
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
        self.ram[ITEM_RECEIPT_METHOD] = 0;
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_platform_and_pit_state(&mut self) {
        self.state.reset_platform_and_pit_state();
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
        self.state.movement.set_drag_player_x(value);
        write_le_u16(self.ram, DRAG_PLAYER_X, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_drag_player_y(&mut self, value: u16) {
        self.state.movement.set_drag_player_y(value);
        write_le_u16(self.ram, DRAG_PLAYER_Y, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_drag_player_x(&mut self, delta: u16) {
        self.state.movement.add_drag_player_x(delta);
        write_le_u16(self.ram, DRAG_PLAYER_X, self.state.drag_player_x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_drag_player_y(&mut self, delta: u16) {
        self.state.movement.add_drag_player_y(delta);
        write_le_u16(self.ram, DRAG_PLAYER_Y, self.state.drag_player_y());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_gravestone_push_timeout(&mut self, value: u8) {
        self.state.movement.set_gravestone_push_timeout(value);
        self.ram[GRAVESTONE_PUSH_TIMEOUT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_gravestone_push_timeout(&mut self) {
        self.state.movement.decrement_gravestone_push_timeout();
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
        self.state
            .actions
            .land_after_splash_with_handler(handler_state);
        self.ram[LINK_HANDLER_STATE] = handler_state;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enter_water_hop_state(&mut self) {
        self.state.actions.enter_water_hop_state();
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
        self.state.actions.become_bunny_handler();
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

    pub(crate) fn recache_bunny_state(&mut self, has_moon_pearl: bool) {
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
        self.state.movement.enter_deep_water();
        self.ram[LINK_IS_IN_DEEP_WATER] = 1;
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.state.last_direction();
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.debug_assert_matches_ram();
    }
}
