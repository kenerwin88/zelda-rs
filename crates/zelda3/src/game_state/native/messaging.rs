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
    value: u16,
    backup: u8,
}

impl MultiselectChoiceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            value: read_le_u16(ram, MULTISELECT_CHOICE),
            backup: ram.get(MULTISELECT_CHOICE_BACKUP).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, MULTISELECT_CHOICE, self.value);
        ram[MULTISELECT_CHOICE_BACKUP] = self.backup;
    }

    pub(crate) fn value(&self) -> u8 {
        self.value as u8
    }

    pub(crate) fn value_word(&self) -> u16 {
        self.value
    }

    pub(crate) fn backup(&self) -> u8 {
        self.backup
    }

    pub(crate) fn set_value(&mut self, value: u8) {
        self.value = (self.value & 0xff00) | u16::from(value);
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
    pub(crate) shared_message_timer: SharedMessageTimerState,
}

impl MessagingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dialogue_message_index: DialogueMessageIndexState::load_from_ram(ram),
            multiselect_choice: MultiselectChoiceState::load_from_ram(ram),
            dialogue_number: DialogueNumberState::load_from_ram(ram),
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
