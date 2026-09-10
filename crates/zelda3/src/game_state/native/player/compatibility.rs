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

    fn sync(&mut self) {
        self.state
            .write_to_ram(&mut crate::game_state::native::ram_target::DiffTarget::new(
                self.ram,
            ));
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_cached(&mut self) {
        let x = self.state.movement.cached_x;
        let y = self.state.movement.cached_y;
        self.state.movement.set_position(x, y);
        self.sync();
    }

    pub(crate) fn cache_current_position(&mut self) {
        self.state.movement.cached_x = self.state.x();
        self.state.movement.cached_y = self.state.y();
        self.sync();
    }

    pub(crate) fn cache_copied_position_from_current(&mut self) {
        self.state.movement.copied_x = self.state.x();
        self.state.movement.copied_y = self.state.y();
        self.sync();
    }

    pub(crate) fn restore_y_from_previous_position(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_PREV);
        self.state.movement.set_y(y);
        self.sync();
    }

    pub(crate) fn restore_position_from_previous(&mut self) {
        let x = read_le_u16(self.ram, LINK_X_COORD_PREV);
        let y = read_le_u16(self.ram, LINK_Y_COORD_PREV);
        self.state.movement.set_position(x, y);
        self.sync();
    }

    pub(crate) fn cache_previous_position_from_current_xy_order(&mut self) {
        self.state.movement.cache_previous_position_from_current();
        self.sync();
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
        self.sync();
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
        self.sync();
    }

    pub(crate) fn store_overworld_exit_y_from_current(&mut self) {
        write_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD, self.state.y());
        self.sync();
    }

    pub(crate) fn restore_y_from_overworld_exit(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD);
        self.state.movement.set_y(y);
        self.sync();
    }

    pub(crate) fn restore_position_from_overworld_exit(&mut self) {
        let x = read_le_u16(self.ram, LINK_X_COORD_EXIT_OVERWORLD);
        let y = read_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD);
        self.state.movement.set_position(x, y);
        self.sync();
    }

    pub(crate) fn restore_position_from_safe_return(&mut self) {
        let x = self.state.safe_return_x();
        let y = self.state.safe_return_y();
        self.state.movement.set_position(x, y);
        self.sync();
    }

    pub(crate) fn restore_y_from_hop_origin(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_ORIGINAL);
        self.state.movement.set_y(y);
        self.sync();
    }

    pub(crate) fn clear_link_state_block_for_ending(&mut self) {
        self.ram[LINK_Y_COORD..LINK_Y_COORD + 0x70].fill(0);
        self.sync_from_ram();
        self.sync();
    }

    pub(crate) fn subtract_axis_velocity_delta(&mut self, horizontal: bool, delta: u8) {
        self.state
            .movement
            .subtract_axis_velocity_delta(horizontal, delta);
        if horizontal {
        } else {
        }
        self.sync();
    }

    pub(crate) fn clear_movement_velocity_and_direction(&mut self) {
        self.clear_movement_velocity();
        self.state.movement.set_direction(0);
        self.sync();
    }

    pub(crate) fn set_y_velocity_from_safe_return_delta_unless_ledge_hopping(&mut self) {
        if self.ram[LINK_HANDLER_STATE] != 11 {
            let value = self.state.y_low_delta_from_safe_return();
            self.state.movement.set_y_velocity(value);
            self.sync();
        }
    }

    pub(crate) fn set_x_velocity_from_safe_return_delta(&mut self) {
        let value = self
            .state
            .x_low()
            .wrapping_sub(self.state.safe_return_x() as u8);
        self.state.movement.set_x_velocity(value);
        self.sync();
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
            self.sync();
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
            self.sync();
        }
    }

    pub(crate) fn refresh_direction_from_safe_return_delta(&mut self) {
        self.set_y_velocity_from_safe_return_delta_unless_ledge_hopping();
        self.update_vertical_direction_from_movement_velocity();
        self.set_x_velocity_from_safe_return_delta();
        self.update_horizontal_direction_from_movement_velocity();
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
        self.sync();
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
        self.sync();
        delta
    }

    /// Complete the pending coordinate addition without touching the fraction.
    pub(crate) fn apply_axis_pixel_delta(&mut self, axis: PlayerAxis, delta: u16) -> u16 {
        let mut position = self.axis_position(axis);
        position.apply_pixel_delta(delta);
        self.state.movement.set_axis_position(axis, position);
        write_le_u16(self.ram, Self::axis_offsets(axis).1, position.coordinate);
        self.sync();
        position.coordinate
    }

    /// Publish only the low coordinate byte, matching `STA $20,x` before
    /// `STA $21,x`. Return the computed high byte for the suspended store.
    pub(crate) fn apply_axis_pixel_delta_low(&mut self, axis: PlayerAxis, delta: u16) -> u8 {
        let mut position = self.axis_position(axis);
        let high = position.apply_pixel_delta_low(delta);
        self.state.movement.set_axis_position(axis, position);
        self.ram[Self::axis_offsets(axis).1] = position.coordinate as u8;
        self.sync();
        high
    }

    /// Publish the retained high byte, preserving the low byte observed on reentry.
    pub(crate) fn apply_axis_coordinate_high(&mut self, axis: PlayerAxis, high: u8) -> u16 {
        let mut position = self.axis_position(axis);
        position.apply_coordinate_high(high);
        self.state.movement.set_axis_position(axis, position);
        self.ram[Self::axis_offsets(axis).1 + 1] = high;
        self.sync();
        position.coordinate
    }

    pub(crate) fn move_z_by_velocity(&mut self, velocity: u8) -> u16 {
        self.move_axis_by_velocity(PlayerAxis::Z, velocity)
    }

    pub(crate) fn prime_airborne_z_velocity(&mut self) {
        self.state.movement.prime_airborne_z_velocity();
        self.ram[LINK_Z_SUBPIXEL] = 0;
        self.sync();
    }

    pub(crate) fn add_button_mask_b_y_bits(&mut self, bits: u8) {
        self.state.input.add_button_mask_b_y_bits(bits);
        self.ram[BUTTON_MASK_B_Y] |= bits;
        self.sync();
    }

    pub(crate) fn clear_button_mask_b_y_bits(&mut self, mask: u8) {
        self.state.input.clear_button_mask_b_y_bits(mask);
        self.ram[BUTTON_MASK_B_Y] &= !mask;
        self.sync();
    }

    pub(crate) fn clear_filtered_joypad_l_bits(&mut self, bits: u8) {
        self.state.input.clear_filtered_joypad_l_bits(bits);
        self.ram[FILTERED_JOYPAD_L] &= !bits;
        self.sync();
    }

    pub(crate) fn add_y_button_action_flag_bits(&mut self, bits: u8) {
        self.state.actions.add_y_button_action_flag_bits(bits);
        self.ram[Y_BUTTON_ACTION_FLAGS] |= bits;
        self.sync();
    }

    pub(crate) fn or_defense_flags(&mut self, value: u8) {
        self.state.actions.or_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] |= value;
        self.sync();
    }

    pub(crate) fn and_defense_flags(&mut self, value: u8) {
        self.state.actions.and_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] &= value;
        self.sync();
    }

    pub(crate) fn tick_jump_ledge_timer_or_reset(&mut self) -> bool {
        let reset = self.state.movement.tick_jump_ledge_timer_or_reset();
        self.sync();
        reset
    }

    pub(crate) fn advance_animation_step(&mut self, wrap_at: u8, wrap_to: u8) {
        self.state
            .presentation
            .advance_animation_step(wrap_at, wrap_to);
        if self.ram[LINK_ANIMATION_STEPS] == wrap_at {
        }
        self.sync();
    }

    pub(crate) fn advance_animation_step_at_least(&mut self, wrap_at: u8, wrap_to: u8) {
        self.state
            .presentation
            .advance_animation_step_at_least(wrap_at, wrap_to);
        if self.ram[LINK_ANIMATION_STEPS] >= wrap_at {
        }
        self.sync();
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
        self.sync();
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
        self.sync();
    }

    pub(crate) fn tick_hard_swim_stroke(&mut self) {
        let countdown = self.ram[SWIMMING_COUNTDOWN].wrapping_sub(1);
        if (countdown as i8).is_negative() {
        }
        self.state.movement.tick_hard_swim_stroke(countdown);
        self.sync();
    }

    pub(crate) fn advance_frame_change_counter(&mut self, delay: u8) -> bool {
        let advanced = self.state.presentation.advance_frame_change_counter(delay);
        self.sync();
        advanced
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
        self.sync();
    }

    /// Link_HandleVelocity's four STZ stores in source order ($27,$28,$68,$69).
    pub(crate) fn clear_velocity_selection_store(&mut self, store: u8) {
        match store {
            0 => {
                self.state.movement.actual_y_velocity = 0;
            }
            1 => {
                self.state.movement.actual_x_velocity = 0;
            }
            2 => {
                self.state.movement.y_page_movement_delta = 0;
            }
            3 => {
                self.state.movement.x_page_movement_delta = 0;
            }
            _ => panic!("velocity-selection clear store is outside the source sequence"),
        }
        self.sync();
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
        self.sync();
    }

    pub(crate) fn clear_item_in_hand_bits(&mut self, mask: u8) {
        self.state.actions.clear_item_in_hand_bits(mask);
        self.ram[LINK_ITEM_IN_HAND] &= !mask;
        self.sync();
    }

    pub(crate) fn set_position_mode_bits(&mut self, mask: u8) {
        self.state.movement.set_position_mode_bits(mask);
        self.ram[LINK_POSITION_MODE] |= mask;
        self.sync();
    }

    pub(crate) fn clear_position_mode_bits(&mut self, mask: u8) {
        self.state.movement.clear_position_mode_bits(mask);
        self.ram[LINK_POSITION_MODE] &= !mask;
        self.sync();
    }

    pub(crate) fn reset_link_dma_animation_cycle(&mut self, countdown: u16) {
        // Write-through: the Link DMA animation words are RAM-resident (see the
        // note in FollowerLinkState::write_to_ram).
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, countdown);
        write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, 0);
        write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, 0);
        self.sync();
    }

    /// C: `scratch_1 = j` in player_oam — a call-local scratch write with no native
    /// model behind it (see FollowerLinkState::write_to_ram). Write RAM, like C.
    pub(crate) fn set_link_sprite_index_scratch(&mut self, value: u16) {
        write_le_u16(self.ram, SCRATCH_1, value);
        self.sync();
    }

    pub(crate) fn xor_hookshot_interlock(&mut self, mask: u8) {
        self.state.actions.xor_hookshot_interlock(mask);
        self.ram[RELATED_TO_HOOKSHOT] ^= mask;
        self.sync();
    }

    pub(crate) fn clear_lifting_or_carrying_state(&mut self) {
        self.state.actions.clear_lifting_or_carrying_state();
        self.ram[LINK_STATE_BITS] &= !0x80;
        self.sync();
    }

    pub(crate) fn keep_only_lifting_or_carrying_state(&mut self) {
        self.state.actions.keep_only_lifting_or_carrying_state();
        self.ram[LINK_STATE_BITS] &= 0x80;
        self.sync();
    }

    pub(crate) fn clear_state_item_and_grab_flags(&mut self) {
        self.state.clear_state_item_and_grab_flags();
        self.publish_cleared_state_item_and_grab_flags();
        self.sync();
    }

    fn publish_cleared_item_action_sequence(&mut self) {
    }

    fn publish_cleared_state_item_and_grab_flags(&mut self) {
    }

    fn publish_cancelled_sword_and_item_usage(&mut self) {
        self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
        self.ram[BUTTON_MASK_B_Y] &= !0x81;
    }

    pub(crate) fn cancel_sword_and_item_usage(&mut self) {
        self.state.cancel_sword_and_item_usage();
        self.publish_cancelled_sword_and_item_usage();
        self.sync();
    }

    fn publish_quiet_cape_removal(&mut self) {
    }

    pub(crate) fn unequip_cape_quietly(&mut self) {
        self.state.unequip_cape_quietly();
        self.publish_quiet_cape_removal();
        self.sync();
    }

    pub(crate) fn initialize_link_action_state(&mut self) {
        self.state.initialize_link_action_state();
        self.publish_cleared_item_action_sequence();
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.publish_cleared_state_item_and_grab_flags();
        self.sync();
    }

    pub(crate) fn reset_action_state(&mut self) {
        self.state.reset_action_state();
        write_le_u16(self.ram, TILEDETECT_MISC_TILES, 0);
        self.publish_cleared_item_action_sequence();
        self.publish_cleared_state_item_and_grab_flags();
        self.publish_cancelled_sword_and_item_usage();
        self.sync();
    }

    pub(crate) fn finish_initialization(&mut self) {
        self.state.finish_initialization();
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.publish_quiet_cape_removal();
        self.publish_cancelled_sword_and_item_usage();
        self.sync();
    }

    pub(crate) fn reset_movement_and_transformation_state(&mut self) {
        self.state.reset_movement_and_transformation_state();
        self.ram[PLAYER_RESET_ANCILLA_WORK_BYTE_24] = 0;
        // Single-byte store (low byte only), matching C — the high byte of the temp-bunny
        // timer is preserved (see reset_movement_and_transformation_state on the state).
        self.sync();
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
        self.sync();
    }

    pub(crate) fn enter_water_hop_state(&mut self) {
        self.state.actions.enter_water_hop_state();
        if self.ram[LINK_AUXILIARY_STATE] != 2 {
        }
        self.sync();
    }

    pub(crate) fn interrupt_swimming_for_auxiliary_state(&mut self) {
        self.state.interrupt_swimming_for_auxiliary_state();
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.sync();
    }

    pub(crate) fn reset_idle_swim_animation_if_out_of_water(&mut self) {
        self.state.reset_idle_swim_animation_if_out_of_water();
        if self.ram[LINK_HANDLER_STATE] != 4 {
        }
        self.sync();
    }

    pub(crate) fn reset_after_damaging_pit(&mut self) {
        let handler_state = if self.ram[LINK_IS_BUNNY] != 0 && self.ram[LINK_ITEM_MOON_PEARL] == 0 {
            23
        } else {
            0
        };
        self.state.reset_after_damaging_pit(handler_state);
        self.sync();
    }

    pub(crate) fn recache_bunny_state(&mut self, has_moon_pearl: bool) {
        self.state.recache_bunny_state(has_moon_pearl);
        if has_moon_pearl {
        }
        self.sync();
    }

    forward_synced! {
        state.movement;
        fn set_speed_setting(value: u8);
        fn decrement_speed_setting() -> u8;
        fn clear_speed_modifier();
        fn set_speed_modifier(value: u8);
        fn mark_lower_level();
        fn mark_lower_level_mirror();
        fn set_lower_level_state(value: u8);
        fn set_lower_level_mirror_state(value: u8);
        fn set_lower_level_states(state: u8, mirror: u8);
        fn clear_lower_level();
        fn clear_lower_level_states();
        fn toggle_lower_level_state();
        fn toggle_lower_level_mirror_state();
        fn mirror_lower_level_state();
        fn cache_lower_level_states();
        fn restore_lower_level_state_from_cached();
        fn arm_stair_speed_modifier();
        fn resolve_dash_speed_setting();
        fn promote_pending_speed_modifier();
        fn increase_near_pit_speed_modifier();
        fn advance_dash_deceleration();
        fn set_facing(value: u8);
        fn restore_facing_from_cached();
        fn set_facing_mirror(value: u8);
        fn cache_facing_to_mirror();
        fn cache_facing();
        fn set_moving_against_diag_tile(value: u8);
        fn add_moving_against_diag_tile_flags(value: u8);
        fn clear_moving_against_diag_tile();
        fn set_flag_moving(value: u8);
        fn set_quadrants_from_packed_nibbles(value: u8);
        fn set_quadrants(x: u8, y: u8);
        fn toggle_quadrant_x() -> u8;
        fn toggle_quadrant_y() -> u8;
        fn reset_direction_limits();
        fn reset_direction_masks();
        fn increment_orthogonal_direction_count();
        fn clear_orthogonal_direction_count();
        fn set_last_direction_moved_towards(value: u8);
        fn set_last_direction_from_current_direction();
        fn set_last_direction(value: u8);
        fn mask_last_direction(mask: u8);
        fn set_last_direction_from_swim_flags();
        fn set_swim_flags_from_last_direction();
        fn set_direction(value: u8);
        fn set_direction_and_last_direction(value: u8);
        fn set_direction_and_swim_flags(value: u8);
        fn mask_direction(mask: u8);
        fn add_direction_flags(flags: u8);
        fn clear_direction_flags(flags: u8);
        fn clear_direction_lock();
        fn set_direction_lock_bits(mask: u8);
        fn clear_direction_lock_bits(mask: u8);
        fn set_direction_mask_a(value: u8);
        fn set_direction_mask_b(value: u8);
        fn apply_direction_masks();
        fn force_direction_from_diag_tile_if_needed();
        fn resolve_orthogonal_direction_count_from_facing();
        fn mark_moving_floor_direction(floor_y: u16, floor_x: u16);
        fn set_last_direction_moved_towards_from_facing();
        fn set_swim_direction_flags(direction: u8);
        fn set_y(value: u16);
        fn set_x(value: u16);
        fn set_position(x: u16, y: u16);
        fn set_y_low(value: u8);
        fn set_x_low(value: u8);
        fn cache_previous_position_from_current();
        fn set_previous_position(x: u16, y: u16);
        fn store_safe_return_position(x: u16, y: u16);
        fn store_safe_return_low_from_current();
        fn store_safe_return_y(y: u16);
        fn set_safe_return_y_low(value: u8);
        fn cache_safe_return_position_from_current();
        fn cache_safe_return_high_from_current();
        fn clear_page_movement_deltas();
        fn set_page_movement_deltas(y_delta: u8, x_delta: u8);
        fn set_y_page_movement_delta_from_high_position(high: u8);
        fn set_x_page_movement_delta_from_high_position(high: u8);
        fn set_x_velocity(value: u8);
        fn set_y_velocity(value: u8);
        fn set_movement_velocity_from_delta(x_delta: u16, y_delta: u16);
        fn add_movement_velocity_delta(x_delta: u16, y_delta: u16);
        fn add_y_velocity_delta(y_delta: u8);
        fn clear_movement_velocity();
        fn clear_movement_subpixels();
        fn set_actual_x_velocity(value: u8);
        fn set_actual_y_velocity(value: u8);
        fn clear_actual_x_velocity();
        fn clear_actual_y_velocity();
        fn set_actual_velocity_xy(x: u8, y: u8);
        fn clear_actual_velocity_xy();
        fn invert_actual_velocity_xy();
        fn xor_actual_velocity_xy(mask: u8);
        fn set_actual_velocity_from_direction(direction: u8, velocity: u8);
        fn set_z(value: u16);
        fn set_z_low(value: u8);
        fn clear_z_high();
        fn restore_z_low_from_mirror();
        fn restore_z_from_mirror();
        fn cache_z_low_to_mirror();
        fn cache_z_to_mirror();
        fn set_z_mirror(value: u16);
        fn clear_z_mirror_low();
        fn clear_z_mirror_word_low();
        fn force_z_mirror_low_ff();
        fn set_z_and_mirror(value: u16);
        fn set_actual_z_velocity(value: u8);
        fn set_actual_z_velocity_and_copy(value: u8);
        fn set_actual_z_velocity_mirror_and_copy(value: u8);
        fn restore_actual_z_velocity_from_mirror();
        fn cache_actual_z_velocity_to_mirror();
        fn decrement_actual_z_velocity(delta: u8);
        fn clear_running();
        fn start_running();
        fn set_running_state(value: u8);
        fn set_dash_countdown(value: u8);
        fn increment_dash_countdown() -> u8;
        fn decrement_dash_countdown() -> u8;
        fn set_dash_counter(value: u8);
        fn prime_dash_counter();
        fn decrement_dash_counter_clamped_to_minimum(minimum: u8);
        fn cancel_dash_state();
        fn set_tile_below(value: u8);
        fn set_tile_action_index(value: u8);
        fn set_tile_coll_flag(value: u8);
        fn set_force_move_any_direction(value: u16);
        fn set_recoil_timer(value: u8);
        fn increment_recoil_timer() -> u8;
        fn reset_jump_ledge_timer();
        fn clear_about_to_jump_off_ledge();
        fn increment_about_to_jump_off_ledge();
        fn decrement_push_fatigue_timer() -> u8;
        fn set_push_fatigue_timer(value: u8);
        fn reset_push_fatigue_timer();
        fn clear_near_moveable_statue();
        fn mark_near_moveable_statue();
        fn clear_pit_correction();
        fn set_pit_correction_active();
        fn set_pit_correction_timer(value: u8);
        fn increment_pit_correction_timer();
        fn set_moving_against_diag_deadlocked(value: u8);
        fn clear_somaria_platform_state();
        fn set_somaria_platform_state(value: u8);
        fn clear_doorway_state();
        fn set_doorway_state(value: u8);
        fn clear_swim_fast_state();
        fn reset_swim_stroke_state();
        fn start_hard_swim_stroke(hard_stroke: u8);
        fn enter_deep_water_state();
        fn clear_deep_water_state();
        fn clear_conveyor_belt_state();
        fn set_conveyor_belt_state(value: u8);
        fn clear_misc_bugfix_movement_state();
        fn cache_current_quadrants();
        fn restore_quadrants_from_cached();
        fn set_recoil_z_velocity_for_dungeon_reset(value: u8);
        fn set_recoil_z_velocity(value: u8);
        fn set_whirlpool_trigger();
        fn clear_whirlpool_trigger();
        fn prevent_movement();
        fn clear_prevent_movement();
        fn set_hop_origin_delta_from_y(y: u16) -> u16;
        fn clear_actual_velocity_and_page_movement_deltas();
        fn cache_moving_floor_position(x: u16, y: u16);
        fn decrement_incapacitated_camera_timer() -> u8;
        fn clear_swim_movement_velocity();
        fn set_cached_tile_action_index(value: u8);
        fn clear_swimming_countdown();
        fn clear_force_move_high_byte();
        fn set_swim_stroke_frame_counter(offset: usize, value: u16);
        fn set_bit9_of_xcoord_word(value: u16);
        fn clear_position_mode();
        fn set_position_mode(value: u8);
        fn set_hop_origin_coord(value: u16);
        fn clear_grabbing_wall();
        fn set_grabbing_wall(value: u8);
        fn set_near_pit_state(value: u8);
        fn clear_near_pit_state();
        fn set_pit_data_index(value: u8);
        fn clear_pit_data_index();
        fn advance_pit_data_index() -> u8;
        fn begin_pit_check();
        fn set_drag_player_x(value: u16);
        fn set_drag_player_y(value: u16);
        fn add_drag_player_x(delta: u16);
        fn add_drag_player_y(delta: u16);
        fn set_gravestone_push_timeout(value: u8);
        fn decrement_gravestone_push_timeout();
        fn enter_deep_water();
    }

    forward_synced! {
        state.actions;
        fn set_handler_state(value: u8);
        fn clear_handler_state();
        fn set_ground_state();
        fn immobilize();
        fn clear_immobilized();
        fn set_menu_block_flag(value: u8);
        fn clear_menu_block();
        fn increment_menu_block_flag() -> u8;
        fn set_pull_action_state(value: u8);
        fn set_spin_attack_delay_timer(value: u8);
        fn decrement_spin_attack_delay_timer() -> u8;
        fn set_incapacitated_timer(value: u8);
        fn decrement_incapacitated_timer() -> u8;
        fn reset_elapsed_incapacitated_timer();
        fn set_y_button_action_flags(value: u8);
        fn set_y_button_action_step(value: u8);
        fn set_y_button_action_timer(value: u8);
        fn decrement_y_button_action_timer() -> u8;
        fn clear_defense_flags();
        fn set_defense_flags(value: u8);
        fn set_item_receipt_method(value: u8);
        fn clear_pull_for_rupees_sprite_need();
        fn set_pull_for_rupees_sprite_need();
        fn clear_electrocute_on_touch();
        fn set_electrocute_on_touch(value: u8);
        fn clear_item_debug_value_1();
        fn clear_hookshot_grave_latch();
        fn set_hookshot_grave_latch();
        fn clear_cape_mode();
        fn set_cape_mode(value: u8);
        fn set_cape_decrement_counter(value: u8);
        fn decrement_cape_decrement_counter();
        fn clear_transforming();
        fn set_transforming();
        fn clear_sword_delay_timer();
        fn set_sword_delay_timer(value: u8);
        fn decrement_sword_delay_timer() -> u8;
        fn clear_spin_attack_step_counter();
        fn increment_spin_attack_step_counter() -> u8;
        fn set_spin_attack_sound_latch(value: u8);
        fn clear_spin_attack_sound_latch();
        fn set_state_for_spin_attack(value: u8);
        fn clear_state_for_spin_attack();
        fn increment_immobilized_flag() -> u8;
        fn set_immobilized_flag(value: u8);
        fn clear_action_handler_timer();
        fn set_action_handler_timer(value: u8);
        fn increment_action_handler_timer() -> u8;
        fn set_item_pickup_in_progress(value: u8);
        fn set_hookshot_bg_check_off_timer(value: u8);
        fn decrement_hookshot_bg_check_off_timer();
        fn set_selected_rod(value: u8);
        fn set_flute_countdown(value: u8);
        fn decrement_flute_countdown();
        fn clear_flute_countdown();
        fn clear_item_action_step_var();
        fn increment_pull_action_state();
        fn set_item_holding_timer(value: u8);
        fn clear_ancilla_interactive_reset_flag();
        fn set_sprite_pickup_flag_cached(value: u8);
        fn clear_magic_spell_player_lock();
        fn increment_item_action_step_var() -> u8;
        fn advance_item_action_step_var_wrapping_7_to_1() -> u8;
        fn clear_given_damage();
        fn set_item_in_hand(value: u8);
        fn clear_item_in_hand();
        fn set_item_action_step_var(value: u8);
        fn set_item_action_debug_value_2(value: u8);
        fn set_current_item_y(value: u8);
        fn set_current_item_active(value: u8);
        fn set_receive_item_index(value: u8);
        fn clear_ancilla_pickup_flag();
        fn set_ancilla_pickup_flag(value: u8);
        fn clear_sprite_pickup_flag();
        fn set_sprite_pickup_flag(value: u8);
        fn set_hookshot_interlock(value: u8);
        fn clear_hookshot_interlock();
        fn enable_cutscene_immunity();
        fn set_sprite_damage_disable_timer(value: u8);
        fn clear_sprite_damage_disable_timer();
        fn increment_sprite_damage_disable_timer();
        fn set_cape_transform_timer(value: u8);
        fn tick_cape_transform_timer() -> u8;
        fn clear_cape_transform_timer();
        fn clear_bunny_mirror();
        fn clear_bunny_body_state();
        fn set_bunny_state(value: u8);
        fn clear_bunny_transform_flags();
        fn clear_bunny_transform_after_moon_pearl();
        fn clear_transform_poof_need_and_temp_bunny_timer();
        fn clear_temp_bunny_timer();
        fn set_temp_bunny_timer(value: u16);
        fn decrement_temp_bunny_timer() -> u16;
        fn set_auxiliary_state(value: u8);
        fn clear_auxiliary_state();
        fn set_state_bits(value: u8);
        fn clear_state_bits();
        fn clear_picking_throw_state();
        fn set_picking_throw_state(value: u8);
        fn start_lift_throw_state();
        fn become_bunny_handler();
    }

    forward_synced! {
        state.presentation;
        fn set_oam_x_offset(value: u8);
        fn set_oam_y_offset(value: u8);
        fn disable_oam_offsets();
        fn set_visibility_status(value: u8);
        fn clear_faint_animation_active();
        fn set_faint_animation_active(value: u8);
        fn set_dash_noise_request();
        fn clear_dash_noise_request();
        fn set_spin_offsets(value: u8);
        fn clear_blink_countdown();
        fn set_blink_countdown(value: u8);
        fn decrement_blink_countdown() -> u8;
        fn set_spin_animation_step_counter(value: u8);
        fn increment_spin_animation_step_counter() -> u8;
        fn clear_spin_animation_step_counter();
        fn clear_animation_step();
        fn set_animation_step(value: u8);
        fn increment_opening_pose();
        fn clear_animation_step_if_at_least(threshold: u8);
        fn subtract_animation_step_if_at_least(threshold: u8, delta: u8);
        fn clear_water_ripple_or_grass_state();
        fn set_water_ripple_or_grass_state(value: u8);
        fn set_secondary_water_grass_timer(value: u8);
        fn clear_index_of_dashing_sfx();
        fn decrement_index_of_dashing_sfx();
        fn increment_water_ripple_or_grass_state() -> u8;
        fn set_primary_water_grass_timer(value: u8);
        fn clear_frame_change_counter();
        fn set_sprite_oam_state_timer(value: u8);
        fn decrement_sprite_oam_state_timer() -> u8;
        fn mark_pit_landing_oam_state();
        fn increment_sleep_in_bed_state();
        fn clear_player_pose_draw_counter();
        fn increment_player_pose_draw_counter();
        fn clear_player_special_draw_flag();
        fn set_player_special_draw_flag(value: u8);
        fn set_throw_oam_state_index(value: u8);
        fn set_link_dma_graphics_index_word(value: u16);
        fn set_link_dma_left_sprite_bank_word(value: u16);
        fn set_link_dma_right_sprite_bank_word(value: u16);
        fn clear_link_dma_sprite_banks();
        fn set_palette_bits_of_oam_word(value: u16);
        fn set_sword_dma_graphics_index(value: u8);
        fn set_shield_dma_graphics_index(value: u8);
        fn set_link_dma_staging_index(value: u8);
        fn set_item_hold_pose(value: u8);
        fn clear_item_hold_pose();
        fn force_hold_sword_up();
        fn clear_force_hold_sword_up();
    }

    forward_synced! {
        state.input;
        fn set_button_mask_b_y(value: u8);
        fn set_filtered_joypad_h(value: u8);
        fn set_filtered_joypad_l(value: u8);
        fn set_joypad1h_last(value: u8);
        fn set_joypad1l_last(value: u8);
        fn set_joypad1h_last2(value: u8);
        fn set_joypad1l_last2(value: u8);
        fn clear_button_b_frames();
        fn set_button_b_frames(value: u8);
        fn increment_button_b_frames() -> u8;
    }

    forward_synced! {
        state;
        fn reset_swim_subpixel_and_defense_state();
        fn reset_incapacitated_camera_timer_from_incapacitated();
        fn set_button_b_frames_word(value: u16);
        fn decrement_button_b_frames_word() -> u16;
        fn clear_action_scratch_state();
        fn clear_lift_throw_scratch_state();
        fn start_bunny_transform_poof();
        fn finish_bunny_transform_poof();
        fn enter_item_hold_pose();
        fn clear_swimming_action_state();
        fn finish_recoil_landing();
        fn clear_swim_stroke_counters();
        fn reset_platform_and_pit_state();
        fn setup_bed_pose();
    }
}
