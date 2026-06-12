use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueMessageIndexState {
    value: u16,
}

impl DialogueMessageIndexState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            value: read_le_u16(ram, DIALOGUE_MESSAGE_INDEX),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DIALOGUE_MESSAGE_INDEX, self.value);
    }

    pub(crate) fn value(&self) -> u16 {
        self.value
    }

    pub(crate) fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MultiselectChoiceState {
    value: u8,
    backup: u8,
}

impl MultiselectChoiceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            value: ram.get(MULTISELECT_CHOICE).copied().unwrap_or(0),
            backup: ram.get(MULTISELECT_CHOICE_BACKUP).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MULTISELECT_CHOICE] = self.value;
        ram[MULTISELECT_CHOICE_BACKUP] = self.backup;
    }

    pub(crate) fn value(&self) -> u8 {
        self.value
    }

    pub(crate) fn backup(&self) -> u8 {
        self.backup
    }

    pub(crate) fn set_value(&mut self, value: u8) {
        self.value = value;
    }

    pub(crate) fn increment_value(&mut self) {
        self.set_value(self.value().wrapping_add(1));
    }

    pub(crate) fn decrement_value(&mut self) {
        self.set_value(self.value().wrapping_sub(1));
    }

    pub(crate) fn restore_backup(&mut self) {
        self.set_value(self.backup);
    }

    pub(crate) fn save_backup(&mut self) {
        self.backup = self.value();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueNumberState {
    low_pair: u8,
    high_pair: u8,
}

impl DialogueNumberState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            low_pair: ram.get(DIALOGUE_NUMBER_LO).copied().unwrap_or(0),
            high_pair: ram.get(DIALOGUE_NUMBER_HI).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DIALOGUE_NUMBER_LO] = self.low_pair;
        ram[DIALOGUE_NUMBER_HI] = self.high_pair;
    }

    pub(crate) fn packed_digits(&self, pair_index: usize) -> u8 {
        match pair_index {
            0 => self.low_pair,
            1 => self.high_pair,
            _ => 0,
        }
    }

    pub(crate) fn set_packed_digits(&mut self, low_pair: u8, high_pair: u8) {
        self.low_pair = low_pair;
        self.high_pair = high_pair;
    }

    pub(crate) fn set_low_pair(&mut self, value: u8) {
        self.low_pair = value;
    }

    pub(crate) fn set_high_pair(&mut self, value: u8) {
        self.high_pair = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MessagingRuntimeState {
    pub(crate) module: u8,
    pub(crate) text_render_state: u8,
    pub(crate) text_wait_countdown2: u8,
    pub(crate) menu_animation_timer: u8,
    pub(crate) game_over_letter_cursor: u8,
    pub(crate) flag_which_music_type_messaging: u8,
    pub(crate) dialogue_scroll_speed: u8,
    pub(crate) text_incremental_state: u8,
    pub(crate) vwf_line_speed_cur: u8,
    pub(crate) vwf_line_speed: u8,
    pub(crate) text_wait_countdown: u16,
    pub(crate) text_msgbox_topleft: u16,
    pub(crate) text_msgbox_topleft_copy: u16,
    pub(crate) text_tilemap_cur: u16,
    pub(crate) dialogue_msg_read_pos: u16,
    pub(crate) message_or_sprite_state_cache: u8,
}

impl MessagingRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            module: ram.get(MESSAGING_MODULE).copied().unwrap_or(0),
            text_render_state: ram.get(TEXT_RENDER_STATE).copied().unwrap_or(0),
            text_wait_countdown2: ram.get(TEXT_WAIT_COUNTDOWN2).copied().unwrap_or(0),
            menu_animation_timer: ram.get(MENU_ANIMATION_TIMER).copied().unwrap_or(0),
            game_over_letter_cursor: ram.get(GAME_OVER_LETTER_CURSOR).copied().unwrap_or(0),
            flag_which_music_type_messaging: ram
                .get(FLAG_WHICH_MUSIC_TYPE_MESSAGING)
                .copied()
                .unwrap_or(0),
            dialogue_scroll_speed: ram.get(DIALOGUE_SCROLL_SPEED).copied().unwrap_or(0),
            text_incremental_state: ram.get(TEXT_INCREMENTAL_STATE).copied().unwrap_or(0),
            vwf_line_speed_cur: ram.get(VWF_LINE_SPEED_CUR).copied().unwrap_or(0),
            vwf_line_speed: ram.get(VWF_LINE_SPEED).copied().unwrap_or(0),
            text_wait_countdown: read_le_u16(ram, TEXT_WAIT_COUNTDOWN),
            text_msgbox_topleft: read_le_u16(ram, TEXT_MSGBOX_TOPLEFT),
            text_msgbox_topleft_copy: read_le_u16(ram, TEXT_MSGBOX_TOPLEFT_COPY),
            text_tilemap_cur: read_le_u16(ram, TEXT_TILEMAP_CUR),
            dialogue_msg_read_pos: read_le_u16(ram, DIALOGUE_MSG_READ_POS),
            message_or_sprite_state_cache: ram
                .get(MESSAGE_OR_SPRITE_STATE_CACHE)
                .copied()
                .unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MESSAGING_MODULE] = self.module;
        ram[TEXT_RENDER_STATE] = self.text_render_state;
        ram[TEXT_WAIT_COUNTDOWN2] = self.text_wait_countdown2;
        ram[MENU_ANIMATION_TIMER] = self.menu_animation_timer;
        ram[GAME_OVER_LETTER_CURSOR] = self.game_over_letter_cursor;
        ram[FLAG_WHICH_MUSIC_TYPE_MESSAGING] = self.flag_which_music_type_messaging;
        ram[DIALOGUE_SCROLL_SPEED] = self.dialogue_scroll_speed;
        ram[TEXT_INCREMENTAL_STATE] = self.text_incremental_state;
        ram[VWF_LINE_SPEED_CUR] = self.vwf_line_speed_cur;
        ram[VWF_LINE_SPEED] = self.vwf_line_speed;
        write_le_u16(ram, TEXT_WAIT_COUNTDOWN, self.text_wait_countdown);
        write_le_u16(ram, TEXT_MSGBOX_TOPLEFT, self.text_msgbox_topleft);
        write_le_u16(ram, TEXT_MSGBOX_TOPLEFT_COPY, self.text_msgbox_topleft_copy);
        write_le_u16(ram, TEXT_TILEMAP_CUR, self.text_tilemap_cur);
        write_le_u16(ram, DIALOGUE_MSG_READ_POS, self.dialogue_msg_read_pos);
        ram[MESSAGE_OR_SPRITE_STATE_CACHE] = self.message_or_sprite_state_cache;
    }

    pub(crate) fn module(&self) -> u8 {
        self.module
    }

    pub(crate) fn text_render_state(&self) -> u8 {
        self.text_render_state
    }

    pub(crate) fn text_wait_countdown2(&self) -> u8 {
        self.text_wait_countdown2
    }

    pub(crate) fn menu_animation_timer(&self) -> u8 {
        self.menu_animation_timer
    }

    pub(crate) fn game_over_letter_cursor(&self) -> u8 {
        self.game_over_letter_cursor
    }

    pub(crate) fn effect_index(&self) -> u8 {
        self.game_over_letter_cursor
    }

    pub(crate) fn flag_which_music_type_messaging(&self) -> u8 {
        self.flag_which_music_type_messaging
    }

    pub(crate) fn dialogue_scroll_speed(&self) -> u8 {
        self.dialogue_scroll_speed
    }

    pub(crate) fn text_incremental_state(&self) -> u8 {
        self.text_incremental_state
    }

    pub(crate) fn vwf_line_speed_cur(&self) -> u8 {
        self.vwf_line_speed_cur
    }

    pub(crate) fn vwf_line_speed(&self) -> u8 {
        self.vwf_line_speed
    }

    pub(crate) fn text_wait_countdown(&self) -> u16 {
        self.text_wait_countdown
    }

    pub(crate) fn text_msgbox_topleft(&self) -> u16 {
        self.text_msgbox_topleft
    }

    pub(crate) fn text_msgbox_topleft_copy(&self) -> u16 {
        self.text_msgbox_topleft_copy
    }

    pub(crate) fn text_tilemap_cur(&self) -> u16 {
        self.text_tilemap_cur
    }

    pub(crate) fn dialogue_msg_read_pos(&self) -> u16 {
        self.dialogue_msg_read_pos
    }

    pub(crate) fn message_or_sprite_state_cache(&self) -> u8 {
        self.message_or_sprite_state_cache
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SharedMessageTimerState {
    pub(crate) timer: u16,
}

impl SharedMessageTimerState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            timer: if SHARED_MESSAGE_TIMER + 1 < ram.len() {
                read_le_u16(ram, SHARED_MESSAGE_TIMER)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, SHARED_MESSAGE_TIMER, self.timer);
    }

    pub(crate) fn tick(&mut self) -> u16 {
        self.timer = self.timer.wrapping_sub(1);
        self.timer
    }

    pub(crate) fn clear(&mut self) {
        self.timer = 0;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MessagingState {
    pub(crate) dialogue_message_index: DialogueMessageIndexState,
    pub(crate) multiselect_choice: MultiselectChoiceState,
    pub(crate) dialogue_number: DialogueNumberState,
    pub(crate) runtime: MessagingRuntimeState,
    pub(crate) shared_message_timer: SharedMessageTimerState,
}

impl MessagingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dialogue_message_index: DialogueMessageIndexState::load_from_ram(ram),
            multiselect_choice: MultiselectChoiceState::load_from_ram(ram),
            dialogue_number: DialogueNumberState::load_from_ram(ram),
            runtime: MessagingRuntimeState::load_from_ram(ram),
            shared_message_timer: SharedMessageTimerState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.dialogue_message_index.write_to_ram(ram);
        self.multiselect_choice.write_to_ram(ram);
        self.dialogue_number.write_to_ram(ram);
        self.shared_message_timer.write_to_ram(ram);
    }
}

pub(crate) struct NativeMultiselectChoiceView<'a> {
    choice: &'a MultiselectChoiceState,
    runtime: &'a MessagingRuntimeState,
}

impl<'a> NativeMultiselectChoiceView<'a> {
    pub(crate) fn new(
        choice: &'a MultiselectChoiceState,
        runtime: &'a MessagingRuntimeState,
    ) -> Self {
        Self { choice, runtime }
    }

    pub(crate) fn value(&self) -> u8 {
        self.choice.value()
    }

    pub(crate) fn value_word(&self) -> u16 {
        u16::from(self.choice.value()) | (u16::from(self.runtime.text_wait_countdown2()) << 8)
    }

    pub(crate) fn backup(&self) -> u8 {
        self.choice.backup()
    }
}

pub(crate) struct NativeDialogueMessageIndexBridgeMut<'a> {
    message_index: &'a mut DialogueMessageIndexState,
    ram: &'a mut [u8],
}

impl<'a> NativeDialogueMessageIndexBridgeMut<'a> {
    pub(crate) fn new(message_index: &'a mut DialogueMessageIndexState, ram: &'a mut [u8]) -> Self {
        *message_index = DialogueMessageIndexState::load_from_ram(ram);
        Self { message_index, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.message_index,
            DialogueMessageIndexState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_value(&mut self, value: u16) {
        self.message_index.set_value(value);
        write_le_u16(self.ram, DIALOGUE_MESSAGE_INDEX, value);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeMultiselectChoiceBridgeMut<'a> {
    choice: &'a mut MultiselectChoiceState,
    ram: &'a mut [u8],
}

impl<'a> NativeMultiselectChoiceBridgeMut<'a> {
    pub(crate) fn new(choice: &'a mut MultiselectChoiceState, ram: &'a mut [u8]) -> Self {
        *choice = MultiselectChoiceState::load_from_ram(ram);
        Self { choice, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.choice,
            MultiselectChoiceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_value(&mut self, value: u8) {
        self.choice.set_value(value);
        self.ram[MULTISELECT_CHOICE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_value(&mut self) {
        self.choice.increment_value();
        self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE].wrapping_add(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_value(&mut self) {
        self.choice.decrement_value();
        self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE].wrapping_sub(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_backup(&mut self) {
        self.choice.restore_backup();
        self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE_BACKUP];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn save_backup(&mut self) {
        self.choice.save_backup();
        self.ram[MULTISELECT_CHOICE_BACKUP] = self.ram[MULTISELECT_CHOICE];
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeDialogueNumberBridgeMut<'a> {
    number: &'a mut DialogueNumberState,
    ram: &'a mut [u8],
}

impl<'a> NativeDialogueNumberBridgeMut<'a> {
    pub(crate) fn new(number: &'a mut DialogueNumberState, ram: &'a mut [u8]) -> Self {
        *number = DialogueNumberState::load_from_ram(ram);
        Self { number, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.number, DialogueNumberState::load_from_ram(self.ram));
    }

    pub(crate) fn set_packed_digits(&mut self, low_pair: u8, high_pair: u8) {
        self.number.set_packed_digits(low_pair, high_pair);
        self.ram[DIALOGUE_NUMBER_LO] = low_pair;
        self.ram[DIALOGUE_NUMBER_HI] = high_pair;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_low_pair(&mut self, value: u8) {
        self.number.set_low_pair(value);
        self.ram[DIALOGUE_NUMBER_LO] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_high_pair(&mut self, value: u8) {
        self.number.set_high_pair(value);
        self.ram[DIALOGUE_NUMBER_HI] = value;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeMessagingRuntimeBridgeMut<'a> {
    messaging: &'a mut MessagingState,
    ram: &'a mut [u8],
}

impl<'a> NativeMessagingRuntimeBridgeMut<'a> {
    pub(crate) fn new(messaging: &'a mut MessagingState, ram: &'a mut [u8]) -> Self {
        *messaging = MessagingState::load_from_ram(ram);
        Self { messaging, ram }
    }

    fn debug_assert_runtime_matches_ram(&self) {
        debug_assert_eq!(
            self.messaging.runtime,
            MessagingRuntimeState::load_from_ram(self.ram)
        );
        debug_assert_eq!(
            self.messaging.multiselect_choice,
            MultiselectChoiceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_module(&mut self, value: u8) {
        self.messaging.runtime.module = value;
        self.ram[MESSAGING_MODULE] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_module(&mut self) {
        self.set_module(0);
    }

    pub(crate) fn clear_message_or_sprite_state_cache(&mut self) {
        self.messaging.runtime.message_or_sprite_state_cache = 0;
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_render_state(&mut self, value: u8) {
        self.messaging.runtime.text_render_state = value;
        self.ram[TEXT_RENDER_STATE] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn increment_text_render_state(&mut self) -> u8 {
        let next = self.messaging.runtime.text_render_state.wrapping_add(1);
        self.set_text_render_state(next);
        next
    }

    pub(crate) fn set_text_wait_countdown2(&mut self, value: u8) {
        self.messaging.runtime.text_wait_countdown2 = value;
        self.ram[TEXT_WAIT_COUNTDOWN2] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_text_wait_countdown2(&mut self) {
        self.set_text_wait_countdown2(0);
    }

    pub(crate) fn decrement_text_wait_countdown2(&mut self) -> u8 {
        let next = self.messaging.runtime.text_wait_countdown2.wrapping_sub(1);
        self.set_text_wait_countdown2(next);
        next
    }

    pub(crate) fn set_menu_animation_timer(&mut self, value: u8) {
        self.messaging.runtime.menu_animation_timer = value;
        self.ram[MENU_ANIMATION_TIMER] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn decrement_menu_animation_timer(&mut self) -> u8 {
        let next = self.messaging.runtime.menu_animation_timer.wrapping_sub(1);
        self.set_menu_animation_timer(next);
        next
    }

    pub(crate) fn set_game_over_letter_cursor(&mut self, value: u8) {
        self.messaging.runtime.game_over_letter_cursor = value;
        self.ram[GAME_OVER_LETTER_CURSOR] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_effect_index(&mut self, value: u8) {
        self.set_game_over_letter_cursor(value);
    }

    pub(crate) fn or_effect_index(&mut self, value: u8) {
        let next = self.messaging.runtime.game_over_letter_cursor | value;
        self.set_game_over_letter_cursor(next);
    }

    pub(crate) fn clear_game_over_letter_cursor(&mut self) {
        self.set_game_over_letter_cursor(0);
    }

    pub(crate) fn clear_effect_index(&mut self) {
        self.clear_game_over_letter_cursor();
    }

    pub(crate) fn increment_game_over_letter_cursor(&mut self) -> u8 {
        let next = self
            .messaging
            .runtime
            .game_over_letter_cursor
            .wrapping_add(1);
        self.set_game_over_letter_cursor(next);
        next
    }

    pub(crate) fn decrement_game_over_letter_cursor(&mut self) -> u8 {
        let next = self
            .messaging
            .runtime
            .game_over_letter_cursor
            .wrapping_sub(1);
        self.set_game_over_letter_cursor(next);
        next
    }

    pub(crate) fn clear_flag_which_music_type_messaging(&mut self) {
        self.messaging.runtime.flag_which_music_type_messaging = 0;
        self.ram[FLAG_WHICH_MUSIC_TYPE_MESSAGING] = 0;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn xor_message_or_sprite_state_cache(&mut self, value: u8) {
        self.messaging.runtime.message_or_sprite_state_cache ^= value;
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] ^= value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn increment_text_incremental_state(&mut self) {
        self.messaging.runtime.text_incremental_state = self
            .messaging
            .runtime
            .text_incremental_state
            .wrapping_add(1);
        self.ram[TEXT_INCREMENTAL_STATE] = self.ram[TEXT_INCREMENTAL_STATE].wrapping_add(1);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn decrement_vwf_line_speed_cur(&mut self) {
        self.messaging.runtime.vwf_line_speed_cur =
            self.messaging.runtime.vwf_line_speed_cur.wrapping_sub(1);
        self.ram[VWF_LINE_SPEED_CUR] = self.ram[VWF_LINE_SPEED_CUR].wrapping_sub(1);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_vwf_line_speed(&mut self, value: u8) {
        self.messaging.runtime.vwf_line_speed = value;
        self.ram[VWF_LINE_SPEED] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_vwf_line_speed_cur(&mut self, value: u8) {
        self.messaging.runtime.vwf_line_speed_cur = value;
        self.ram[VWF_LINE_SPEED_CUR] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_dialogue_scroll_speed(&mut self, value: u8) {
        self.messaging.runtime.dialogue_scroll_speed = value;
        self.ram[DIALOGUE_SCROLL_SPEED] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_wait_countdown(&mut self, value: u16) {
        self.messaging.runtime.text_wait_countdown = value;
        write_le_u16(self.ram, TEXT_WAIT_COUNTDOWN, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_text_wait_countdown(&mut self) {
        self.messaging.runtime.text_wait_countdown &= 0xff00;
        self.ram[TEXT_WAIT_COUNTDOWN] = 0;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_msgbox_topleft(&mut self, value: u16) {
        self.messaging.runtime.text_msgbox_topleft = value;
        write_le_u16(self.ram, TEXT_MSGBOX_TOPLEFT, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_msgbox_topleft_copy(&mut self, value: u16) {
        self.messaging.runtime.text_msgbox_topleft_copy = value;
        write_le_u16(self.ram, TEXT_MSGBOX_TOPLEFT_COPY, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_tilemap_cur(&mut self, value: u16) {
        self.messaging.runtime.text_tilemap_cur = value;
        write_le_u16(self.ram, TEXT_TILEMAP_CUR, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_dialogue_msg_read_pos(&mut self) {
        self.set_dialogue_msg_read_pos(0);
    }

    pub(crate) fn set_dialogue_msg_read_pos(&mut self, value: u16) {
        self.messaging.runtime.dialogue_msg_read_pos = value;
        write_le_u16(self.ram, DIALOGUE_MSG_READ_POS, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn init_msgbox_state_from(&mut self, data: &[u8]) {
        self.ram[TEXT_MSGBOX_TOPLEFT_COPY..TEXT_MSGBOX_TOPLEFT_COPY + data.len()]
            .copy_from_slice(data);
        self.messaging.runtime = MessagingRuntimeState::load_from_ram(self.ram);
        self.messaging.multiselect_choice = MultiselectChoiceState::load_from_ram(self.ram);
        self.debug_assert_runtime_matches_ram();
    }
}

pub(crate) struct NativeSharedMessageTimerBridgeMut<'a> {
    timer: &'a mut SharedMessageTimerState,
    ram: &'a mut [u8],
}

impl<'a> NativeSharedMessageTimerBridgeMut<'a> {
    pub(crate) fn new(timer: &'a mut SharedMessageTimerState, ram: &'a mut [u8]) -> Self {
        *timer = SharedMessageTimerState::load_from_ram(ram);
        Self { timer, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.timer,
            SharedMessageTimerState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn start(&mut self, value: u16) {
        self.timer.timer = value;
        write_le_u16(self.ram, SHARED_MESSAGE_TIMER, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear(&mut self) {
        self.timer.clear();
        write_le_u16(self.ram, SHARED_MESSAGE_TIMER, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn tick(&mut self) -> u16 {
        let value = self.timer.tick();
        write_le_u16(self.ram, SHARED_MESSAGE_TIMER, value);
        self.debug_assert_matches_ram();
        value
    }
}
