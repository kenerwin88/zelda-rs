//! Player actions, combat, transformations, and item use.

use super::{
    PLAYER_HANDLER_STATE_BOMBOS, PLAYER_HANDLER_STATE_ETHER, PLAYER_HANDLER_STATE_GROUND,
    PLAYER_HANDLER_STATE_HOOKSHOT, PLAYER_HANDLER_STATE_QUAKE, PLAYER_HANDLER_STATE_RECOIL_OTHER,
    PLAYER_HANDLER_STATE_START_DASH, PLAYER_HANDLER_STATE_SWIMMING,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct PlayerActionsState {
    pub(super) menu_block_flag: u8,
    pub(super) handler_state: u8,
    pub(super) immobilized: u8,
    pub(super) action_state_bits: u8,
    pub(super) auxiliary_state: u8,
    pub(super) picking_throw_state: u8,
    pub(super) spin_attack_delay_timer: u8,
    pub(super) spin_attack_step_counter: u8,
    pub(super) spin_attack_state: u8,
    pub(super) spin_attack_sound_latch: u8,
    pub(super) incapacitated_timer: u8,
    pub(super) y_button_action_flags: u8,
    pub(super) y_button_action_step: u8,
    pub(super) y_button_action_timer: u8,
    pub(super) defense_flags: u8,
    pub(super) item_receipt_method: u8,
    pub(super) action_handler_timer: u8,
    pub(super) bunny_transform_timer: u8,
    pub(super) bunny_state: u8,
    pub(super) bunny_mirror: u8,
    pub(super) temp_bunny_timer: u16,
    pub(super) transform_poof_needed: u8,
    pub(super) magic_spell_player_lock: u8,
    pub(super) item_holding_timer: u8,
    pub(super) ancilla_interactive_reset_flag: u8,
    pub(super) item_action_step: u8,
    pub(super) item_action_debug_value_2: u8,
    pub(super) item_debug_value_1: u8,
    pub(super) given_damage: u8,
    pub(super) pull_action_state: u8,
    pub(super) current_item_y: u8,
    pub(super) current_item_active: u8,
    pub(super) receive_item_index: u8,
    pub(super) item_in_hand: u8,
    pub(super) item_pickup_in_progress: u8,
    pub(super) selected_rod: u8,
    pub(super) ancilla_pickup_flag: u8,
    pub(super) sprite_pickup_flag: u8,
    pub(super) sprite_pickup_flag_cached: u8,
    pub(super) pull_for_rupees_sprite_needed: u8,
    pub(super) hookshot_interlock: u8,
    pub(super) hookshot_grave_latch: u8,
    pub(super) electrocute_on_touch: u8,
    pub(super) cape_mode: u8,
    pub(super) cape_decrement_counter: u8,
    pub(super) sword_delay_timer: u8,
    pub(super) transforming: u8,
    pub(super) flute_countdown: u8,
    pub(super) hookshot_bg_check_off_timer: u8,
    pub(super) sprite_damage_disabled: u8,
}

impl PlayerActionsState {
    pub(crate) fn menu_block_flag(&self) -> u8 {
        self.menu_block_flag
    }

    pub(crate) fn is_menu_blocked(&self) -> bool {
        self.menu_block_flag != 0
    }

    pub(crate) fn has_menu_block_flag(&self, value: u8) -> bool {
        self.menu_block_flag == value
    }

    pub(crate) fn handler_state(&self) -> u8 {
        self.handler_state
    }

    pub(crate) fn is_edge_transition_blocked_by_handler_state(&self) -> bool {
        matches!(self.handler_state, 3 | 8 | 9 | 10)
    }

    pub(crate) fn is_ground_swim_or_dash_start(&self) -> bool {
        matches!(
            self.handler_state,
            PLAYER_HANDLER_STATE_GROUND
                | PLAYER_HANDLER_STATE_SWIMMING
                | PLAYER_HANDLER_STATE_START_DASH
        )
    }

    pub(crate) fn is_using_medallion(&self) -> bool {
        matches!(
            self.handler_state,
            PLAYER_HANDLER_STATE_ETHER | PLAYER_HANDLER_STATE_BOMBOS | PLAYER_HANDLER_STATE_QUAKE
        )
    }

    pub(crate) fn is_swimming(&self) -> bool {
        self.handler_state == PLAYER_HANDLER_STATE_SWIMMING
    }

    pub(crate) fn is_immobilized(&self) -> bool {
        self.immobilized != 0
    }

    pub(crate) fn immobilized_flag(&self) -> u8 {
        self.immobilized
    }

    pub(crate) fn is_hookshot(&self) -> bool {
        self.handler_state == PLAYER_HANDLER_STATE_HOOKSHOT
    }

    pub(crate) fn is_recoiling_from_other_source(&self) -> bool {
        self.handler_state == PLAYER_HANDLER_STATE_RECOIL_OTHER
    }

    pub(crate) fn has_action_state(&self) -> bool {
        self.action_state_bits != 0
    }

    pub(crate) fn state_bits(&self) -> u8 {
        self.action_state_bits
    }

    pub(crate) fn state_bits_has(&self, mask: u8) -> bool {
        self.action_state_bits & mask != 0
    }

    pub(crate) fn has_non_lift_action_state(&self) -> bool {
        self.action_state_bits & 0x7f != 0
    }

    pub(crate) fn is_lifting_or_carrying(&self) -> bool {
        self.action_state_bits & 0x80 != 0
    }

    pub(crate) fn auxiliary_state(&self) -> u8 {
        self.auxiliary_state
    }

    pub(crate) fn is_in_auxiliary_state(&self, value: u8) -> bool {
        self.auxiliary_state == value
    }

    pub(crate) fn has_auxiliary_state(&self) -> bool {
        self.auxiliary_state != 0
    }

    pub(crate) fn item_in_hand(&self) -> u8 {
        self.item_in_hand
    }

    pub(crate) fn has_item_in_hand(&self) -> bool {
        self.item_in_hand != 0
    }

    pub(crate) fn item_in_hand_has(&self, mask: u8) -> bool {
        self.item_in_hand & mask != 0
    }

    pub(crate) fn picking_throw_state(&self) -> u8 {
        self.picking_throw_state
    }

    pub(crate) fn picking_throw_state_has(&self, mask: u8) -> bool {
        self.picking_throw_state & mask != 0
    }

    pub(crate) fn has_picking_throw_state(&self) -> bool {
        self.picking_throw_state != 0
    }

    pub(crate) fn is_lift_throw_primed(&self) -> bool {
        self.picking_throw_state & 1 != 0
    }

    pub(crate) fn spin_attack_delay_timer(&self) -> u8 {
        self.spin_attack_delay_timer
    }

    pub(crate) fn spin_attack_step_counter(&self) -> u8 {
        self.spin_attack_step_counter
    }

    pub(crate) fn incapacitated_timer(&self) -> u8 {
        self.incapacitated_timer
    }

    pub(crate) fn y_button_action_flags(&self) -> u8 {
        self.y_button_action_flags
    }

    pub(crate) fn y_button_action_step(&self) -> u8 {
        self.y_button_action_step
    }

    pub(crate) fn y_button_action_timer(&self) -> u8 {
        self.y_button_action_timer
    }

    pub(crate) fn defense_flags(&self) -> u8 {
        self.defense_flags
    }

    pub(crate) fn electrocute_on_touch(&self) -> u8 {
        self.electrocute_on_touch
    }

    pub(crate) fn is_cape_active(&self) -> bool {
        self.cape_mode != 0
    }

    pub(crate) fn cape_decrement_counter(&self) -> u8 {
        self.cape_decrement_counter
    }

    pub(crate) fn sprite_damage_disable_timer(&self) -> u8 {
        self.sprite_damage_disabled
    }

    pub(crate) fn item_receipt_method(&self) -> u8 {
        self.item_receipt_method
    }

    pub(crate) fn action_handler_timer(&self) -> u8 {
        self.action_handler_timer
    }

    pub(crate) fn is_bunny(&self) -> bool {
        self.bunny_state != 0
    }

    pub(crate) fn is_bunny_mirror(&self) -> bool {
        self.bunny_mirror != 0
    }

    pub(crate) fn temp_bunny_timer(&self) -> u16 {
        self.temp_bunny_timer
    }

    pub(crate) fn needs_transform_poof(&self) -> bool {
        self.transform_poof_needed != 0
    }

    pub(crate) fn state_for_spin_attack(&self) -> u8 {
        self.spin_attack_state
    }

    pub(crate) fn spin_attack_sound_latch(&self) -> u8 {
        self.spin_attack_sound_latch
    }

    pub(crate) fn item_action_step_var(&self) -> u8 {
        self.item_action_step
    }

    pub(crate) fn item_debug_value_1(&self) -> u8 {
        self.item_debug_value_1
    }

    pub(crate) fn sprite_pickup_flag_cached(&self) -> u8 {
        self.sprite_pickup_flag_cached
    }

    pub(crate) fn is_transforming(&self) -> bool {
        self.transforming != 0
    }

    pub(crate) fn needs_pull_for_rupees_sprite(&self) -> bool {
        self.pull_for_rupees_sprite_needed != 0
    }

    pub(crate) fn given_damage(&self) -> u8 {
        self.given_damage
    }

    pub(crate) fn has_pull_action_state(&self) -> bool {
        self.pull_action_state != 0
    }

    pub(crate) fn pull_action_state(&self) -> u8 {
        self.pull_action_state
    }

    pub(crate) fn current_item_y(&self) -> u8 {
        self.current_item_y
    }

    pub(crate) fn current_item_active(&self) -> u8 {
        self.current_item_active
    }

    pub(crate) fn receive_item_index(&self) -> u8 {
        self.receive_item_index
    }

    pub(crate) fn item_pickup_in_progress(&self) -> bool {
        self.item_pickup_in_progress != 0
    }

    pub(crate) fn selected_rod(&self) -> u8 {
        self.selected_rod
    }

    pub(crate) fn ancilla_pickup_flag(&self) -> u8 {
        self.ancilla_pickup_flag
    }

    pub(crate) fn sprite_pickup_flag(&self) -> u8 {
        self.sprite_pickup_flag
    }

    pub(crate) fn hookshot_interlock(&self) -> u8 {
        self.hookshot_interlock
    }

    pub(crate) fn has_hookshot_interlock(&self) -> bool {
        self.hookshot_interlock != 0
    }

    pub(crate) fn hookshot_grave_latch(&self) -> bool {
        self.hookshot_grave_latch != 0
    }

    pub(crate) fn flute_countdown(&self) -> u8 {
        self.flute_countdown
    }

    pub(crate) fn hookshot_bg_check_off_timer(&self) -> u8 {
        self.hookshot_bg_check_off_timer
    }

    pub(crate) fn hookshot_interlock_has(&self, mask: u8) -> bool {
        self.hookshot_interlock & mask != 0
    }

    pub(crate) fn can_drop_follower(&self) -> bool {
        self.auxiliary_state != 1 && !self.is_lifting_or_carrying()
    }

    pub(crate) fn should_transform_old_man_from_recoil(&self) -> bool {
        (self.auxiliary_state & 1) != 0 && self.is_recoiling_from_other_source()
    }

    pub(crate) fn should_transform_old_man_from_auxiliary_state(&self) -> bool {
        self.auxiliary_state & 2 != 0
    }

    pub(super) fn set_menu_block_flag(&mut self, value: u8) {
        self.menu_block_flag = value;
    }

    pub(super) fn clear_menu_block(&mut self) {
        self.menu_block_flag = 0;
    }

    pub(super) fn increment_menu_block_flag(&mut self) -> u8 {
        self.menu_block_flag = self.menu_block_flag.wrapping_add(1);
        self.menu_block_flag
    }

    pub(super) fn set_handler_state(&mut self, value: u8) {
        self.handler_state = value;
    }

    pub(super) fn clear_handler_state(&mut self) {
        self.handler_state = 0;
    }

    pub(super) fn set_ground_state(&mut self) {
        self.handler_state = PLAYER_HANDLER_STATE_GROUND;
    }

    pub(super) fn immobilize(&mut self) {
        self.immobilized = 1;
    }

    pub(super) fn clear_immobilized(&mut self) {
        self.immobilized = 0;
    }

    pub(super) fn set_pull_action_state(&mut self, value: u8) {
        self.pull_action_state = value;
    }

    pub(super) fn set_spin_attack_delay_timer(&mut self, value: u8) {
        self.spin_attack_delay_timer = value;
    }

    pub(super) fn decrement_spin_attack_delay_timer(&mut self) -> u8 {
        self.spin_attack_delay_timer = self.spin_attack_delay_timer.wrapping_sub(1);
        self.spin_attack_delay_timer
    }

    pub(super) fn set_incapacitated_timer(&mut self, value: u8) {
        self.incapacitated_timer = value;
    }

    pub(super) fn decrement_incapacitated_timer(&mut self) -> u8 {
        self.incapacitated_timer = self.incapacitated_timer.wrapping_sub(1);
        self.incapacitated_timer
    }

    pub(super) fn reset_elapsed_incapacitated_timer(&mut self) {
        if self.incapacitated_timer == 0 {
            self.incapacitated_timer = 1;
        }
    }

    pub(super) fn set_y_button_action_flags(&mut self, value: u8) {
        self.y_button_action_flags = value;
    }

    pub(super) fn add_y_button_action_flag_bits(&mut self, bits: u8) {
        self.y_button_action_flags |= bits;
    }

    pub(super) fn set_y_button_action_step(&mut self, value: u8) {
        self.y_button_action_step = value;
    }

    pub(super) fn set_y_button_action_timer(&mut self, value: u8) {
        self.y_button_action_timer = value;
    }

    pub(super) fn decrement_y_button_action_timer(&mut self) -> u8 {
        self.y_button_action_timer = self.y_button_action_timer.wrapping_sub(1);
        self.y_button_action_timer
    }

    pub(super) fn clear_defense_flags(&mut self) {
        self.defense_flags = 0;
    }

    pub(super) fn set_defense_flags(&mut self, value: u8) {
        self.defense_flags = value;
    }

    pub(super) fn or_defense_flags(&mut self, value: u8) {
        self.defense_flags |= value;
    }

    pub(super) fn and_defense_flags(&mut self, value: u8) {
        self.defense_flags &= value;
    }

    pub(super) fn set_item_receipt_method(&mut self, value: u8) {
        self.item_receipt_method = value;
    }

    pub(super) fn clear_item_debug_value_1(&mut self) {
        self.item_debug_value_1 = 0;
    }

    pub(super) fn clear_hookshot_grave_latch(&mut self) {
        self.hookshot_grave_latch = 0;
    }

    pub(super) fn set_hookshot_grave_latch(&mut self) {
        self.hookshot_grave_latch = 1;
    }

    pub(super) fn clear_pull_for_rupees_sprite_need(&mut self) {
        self.pull_for_rupees_sprite_needed = 0;
    }

    pub(super) fn set_pull_for_rupees_sprite_need(&mut self) {
        self.pull_for_rupees_sprite_needed = 1;
    }

    pub(super) fn clear_electrocute_on_touch(&mut self) {
        self.electrocute_on_touch = 0;
    }

    pub(super) fn set_electrocute_on_touch(&mut self, value: u8) {
        self.electrocute_on_touch = value;
    }

    pub(super) fn clear_cape_mode(&mut self) {
        self.cape_mode = 0;
    }

    pub(super) fn set_cape_mode(&mut self, value: u8) {
        self.cape_mode = value;
    }

    pub(super) fn set_cape_decrement_counter(&mut self, value: u8) {
        self.cape_decrement_counter = value;
    }

    pub(super) fn decrement_cape_decrement_counter(&mut self) {
        self.cape_decrement_counter = self.cape_decrement_counter.wrapping_sub(1);
    }

    pub(super) fn clear_transforming(&mut self) {
        self.transforming = 0;
    }

    pub(super) fn set_transforming(&mut self) {
        self.transforming = 1;
    }

    pub(super) fn clear_sword_delay_timer(&mut self) {
        self.sword_delay_timer = 0;
    }

    pub(super) fn set_sword_delay_timer(&mut self, value: u8) {
        self.sword_delay_timer = value;
    }

    pub(super) fn decrement_sword_delay_timer(&mut self) -> u8 {
        self.sword_delay_timer = self.sword_delay_timer.wrapping_sub(1);
        self.sword_delay_timer
    }

    pub(super) fn clear_spin_attack_step_counter(&mut self) {
        self.spin_attack_step_counter = 0;
    }

    pub(super) fn increment_spin_attack_step_counter(&mut self) -> u8 {
        self.spin_attack_step_counter = self.spin_attack_step_counter.wrapping_add(1);
        self.spin_attack_step_counter
    }

    pub(super) fn set_spin_attack_sound_latch(&mut self, value: u8) {
        self.spin_attack_sound_latch = value;
    }

    pub(super) fn clear_spin_attack_sound_latch(&mut self) {
        self.spin_attack_sound_latch = 0;
    }

    pub(super) fn set_state_for_spin_attack(&mut self, value: u8) {
        self.spin_attack_state = value;
    }

    pub(super) fn clear_state_for_spin_attack(&mut self) {
        self.spin_attack_state = 0;
    }

    pub(super) fn increment_immobilized_flag(&mut self) -> u8 {
        self.immobilized = self.immobilized.wrapping_add(1);
        self.immobilized
    }

    pub(super) fn set_immobilized_flag(&mut self, value: u8) {
        self.immobilized = value;
    }

    pub(super) fn clear_action_handler_timer(&mut self) {
        self.action_handler_timer = 0;
    }

    pub(super) fn set_action_handler_timer(&mut self, value: u8) {
        self.action_handler_timer = value;
    }

    pub(super) fn increment_action_handler_timer(&mut self) -> u8 {
        self.action_handler_timer = self.action_handler_timer.wrapping_add(1);
        self.action_handler_timer
    }

    pub(super) fn set_cape_transform_timer(&mut self, value: u8) {
        self.bunny_transform_timer = value;
    }

    pub(super) fn tick_cape_transform_timer(&mut self) -> u8 {
        self.bunny_transform_timer = self.bunny_transform_timer.wrapping_sub(1);
        self.bunny_transform_timer
    }

    pub(super) fn clear_cape_transform_timer(&mut self) {
        self.bunny_transform_timer = 0;
    }

    pub(super) fn clear_bunny_mirror(&mut self) {
        self.bunny_mirror = 0;
    }

    pub(super) fn clear_bunny_body_state(&mut self) {
        self.bunny_state = 0;
    }

    pub(super) fn set_bunny_state(&mut self, value: u8) {
        self.bunny_state = value;
        self.bunny_mirror = value;
    }

    pub(super) fn clear_bunny_transform_flags(&mut self) {
        self.transform_poof_needed = 0;
        self.bunny_state = 0;
        self.bunny_mirror = 0;
    }

    pub(super) fn clear_bunny_transform_after_moon_pearl(&mut self) {
        self.clear_bunny_transform_flags();
        self.temp_bunny_timer &= 0xff00;
    }

    pub(super) fn clear_transform_poof_need_and_temp_bunny_timer(&mut self) {
        self.transform_poof_needed = 0;
        self.temp_bunny_timer = 0;
    }

    pub(super) fn clear_temp_bunny_timer(&mut self) {
        self.temp_bunny_timer = 0;
    }

    pub(super) fn set_temp_bunny_timer(&mut self, value: u16) {
        self.temp_bunny_timer = value;
    }

    pub(super) fn decrement_temp_bunny_timer(&mut self) -> u16 {
        self.temp_bunny_timer = self.temp_bunny_timer.wrapping_sub(1);
        self.temp_bunny_timer
    }

    pub(super) fn set_item_pickup_in_progress(&mut self, value: u8) {
        self.item_pickup_in_progress = value;
    }

    pub(super) fn set_hookshot_bg_check_off_timer(&mut self, value: u8) {
        self.hookshot_bg_check_off_timer = value;
    }

    pub(super) fn decrement_hookshot_bg_check_off_timer(&mut self) {
        self.hookshot_bg_check_off_timer = self.hookshot_bg_check_off_timer.wrapping_sub(1);
    }

    pub(super) fn set_selected_rod(&mut self, value: u8) {
        self.selected_rod = value;
    }

    pub(super) fn set_flute_countdown(&mut self, value: u8) {
        self.flute_countdown = value;
    }

    pub(super) fn decrement_flute_countdown(&mut self) {
        self.flute_countdown = self.flute_countdown.wrapping_sub(1);
    }

    pub(super) fn clear_flute_countdown(&mut self) {
        self.flute_countdown = 0;
    }

    pub(super) fn clear_item_action_step_var(&mut self) {
        self.item_action_step = 0;
    }

    pub(super) fn increment_item_action_step_var(&mut self) -> u8 {
        self.item_action_step = self.item_action_step.wrapping_add(1);
        self.item_action_step
    }

    pub(super) fn advance_item_action_step_var_wrapping_7_to_1(&mut self) -> u8 {
        self.item_action_step = if self.item_action_step.wrapping_add(1) == 7 {
            1
        } else {
            self.item_action_step.wrapping_add(1)
        };
        self.item_action_step
    }

    pub(super) fn clear_given_damage(&mut self) {
        self.given_damage = 0;
    }

    pub(super) fn set_given_damage(&mut self, value: u8) {
        self.given_damage = value;
    }

    pub(super) fn set_item_in_hand(&mut self, value: u8) {
        self.item_in_hand = value;
    }

    pub(super) fn clear_item_in_hand(&mut self) {
        self.item_in_hand = 0;
    }

    pub(super) fn clear_item_in_hand_bits(&mut self, mask: u8) {
        self.item_in_hand &= !mask;
    }

    pub(super) fn set_item_action_step_var(&mut self, value: u8) {
        self.item_action_step = value;
    }

    pub(super) fn set_item_action_debug_value_2(&mut self, value: u8) {
        self.item_action_debug_value_2 = value;
    }

    pub(super) fn set_current_item_y(&mut self, value: u8) {
        self.current_item_y = value;
    }

    pub(super) fn set_current_item_active(&mut self, value: u8) {
        self.current_item_active = value;
    }

    pub(super) fn set_receive_item_index(&mut self, value: u8) {
        self.receive_item_index = value;
    }

    pub(super) fn clear_ancilla_pickup_flag(&mut self) {
        self.ancilla_pickup_flag = 0;
    }

    pub(super) fn set_ancilla_pickup_flag(&mut self, value: u8) {
        self.ancilla_pickup_flag = value;
    }

    pub(super) fn clear_sprite_pickup_flag(&mut self) {
        self.sprite_pickup_flag = 0;
    }

    pub(super) fn set_sprite_pickup_flag(&mut self, value: u8) {
        self.sprite_pickup_flag = value;
    }

    pub(super) fn set_hookshot_interlock(&mut self, value: u8) {
        self.hookshot_interlock = value;
    }

    pub(super) fn clear_hookshot_interlock(&mut self) {
        self.hookshot_interlock = 0;
    }

    pub(super) fn xor_hookshot_interlock(&mut self, mask: u8) {
        self.hookshot_interlock ^= mask;
    }

    pub(super) fn enable_cutscene_immunity(&mut self) {
        self.sprite_damage_disabled = 1;
    }

    pub(super) fn set_sprite_damage_disable_timer(&mut self, value: u8) {
        self.sprite_damage_disabled = value;
    }

    pub(super) fn clear_sprite_damage_disable_timer(&mut self) {
        self.sprite_damage_disabled = 0;
    }

    pub(super) fn increment_sprite_damage_disable_timer(&mut self) {
        self.sprite_damage_disabled = self.sprite_damage_disabled.wrapping_add(1);
    }

    pub(super) fn set_auxiliary_state(&mut self, value: u8) {
        self.auxiliary_state = value;
    }

    pub(super) fn clear_auxiliary_state(&mut self) {
        self.auxiliary_state = 0;
    }

    pub(super) fn set_state_bits(&mut self, value: u8) {
        self.action_state_bits = value;
    }

    pub(super) fn clear_state_bits(&mut self) {
        self.action_state_bits = 0;
    }

    pub(super) fn clear_lifting_or_carrying_state(&mut self) {
        self.action_state_bits &= !0x80;
    }

    pub(super) fn keep_only_lifting_or_carrying_state(&mut self) {
        self.action_state_bits &= 0x80;
    }

    pub(super) fn clear_picking_throw_state(&mut self) {
        self.picking_throw_state = 0;
    }

    pub(super) fn set_picking_throw_state(&mut self, value: u8) {
        self.picking_throw_state = value;
    }

    pub(super) fn start_lift_throw_state(&mut self) {
        self.picking_throw_state = 1;
        self.action_state_bits = 0x80;
    }

    pub(super) fn clear_magic_spell_player_lock(&mut self) {
        self.magic_spell_player_lock = 0;
    }

    pub(super) fn increment_pull_action_state(&mut self) {
        self.pull_action_state = self.pull_action_state.wrapping_add(1);
    }

    pub(super) fn set_item_holding_timer(&mut self, value: u8) {
        self.item_holding_timer = value;
    }

    pub(super) fn clear_ancilla_interactive_reset_flag(&mut self) {
        self.ancilla_interactive_reset_flag = 0;
    }

    pub(super) fn set_sprite_pickup_flag_cached(&mut self, value: u8) {
        self.sprite_pickup_flag_cached = value;
    }

    pub(super) fn land_after_splash_with_handler(&mut self, handler_state: u8) {
        self.handler_state = handler_state;
    }

    pub(super) fn enter_water_hop_state(&mut self) {
        if self.auxiliary_state != 2 {
            self.auxiliary_state = 1;
            self.electrocute_on_touch = 0;
        }
        self.handler_state = 6;
    }

    pub(super) fn become_bunny_handler(&mut self) {
        self.handler_state = 23;
        self.bunny_state = 1;
        self.bunny_mirror = 1;
    }
}
