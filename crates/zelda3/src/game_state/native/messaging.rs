use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

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
    pub(crate) shared_message_timer: SharedMessageTimerState,
}

impl MessagingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            shared_message_timer: SharedMessageTimerState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.shared_message_timer.write_to_ram(ram);
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
