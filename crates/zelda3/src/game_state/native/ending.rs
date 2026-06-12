use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EndingCreditState {
    pub(crate) palace_death_count_digit_step: u16,
    pub(crate) death_count_digit_tile_base: u16,
}

impl EndingCreditState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            palace_death_count_digit_step: if ENDING_WHICH_DUNG + 1 < ram.len() {
                read_le_u16(ram, ENDING_WHICH_DUNG)
            } else {
                0
            },
            death_count_digit_tile_base: if ENDING_CREDIT_DIGIT_CHAR + 1 < ram.len() {
                read_le_u16(ram, ENDING_CREDIT_DIGIT_CHAR)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, ENDING_WHICH_DUNG, self.palace_death_count_digit_step);
        write_le_u16(
            ram,
            ENDING_CREDIT_DIGIT_CHAR,
            self.death_count_digit_tile_base,
        );
    }

    pub(crate) fn palace_death_count_index(&self) -> usize {
        (self.palace_death_count_digit_step >> 1) as usize
    }

    pub(crate) fn digit_tile_base_index(&self) -> usize {
        (self.palace_death_count_digit_step & 1) as usize
    }

    pub(crate) fn should_write_digit_for_scroll_y(
        &self,
        current_scroll_y: u16,
        scheduled_scroll_y: u16,
    ) -> bool {
        self.digit_tile_base_index() != 0 || current_scroll_y == scheduled_scroll_y
    }

    pub(crate) fn clear_palace_death_count_digit_step(&mut self) {
        self.palace_death_count_digit_step = 0;
    }

    pub(crate) fn advance_palace_death_count_digit_step(&mut self) {
        self.palace_death_count_digit_step = self.palace_death_count_digit_step.wrapping_add(1);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EndingState {
    pub(crate) credits: EndingCreditState,
}

impl EndingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            credits: EndingCreditState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.credits.write_to_ram(ram);
    }
}

pub(crate) struct NativeEndingCreditBridgeMut<'a> {
    credits: &'a mut EndingCreditState,
    ram: &'a mut [u8],
}

impl<'a> NativeEndingCreditBridgeMut<'a> {
    pub(crate) fn new(credits: &'a mut EndingCreditState, ram: &'a mut [u8]) -> Self {
        *credits = EndingCreditState::load_from_ram(ram);
        Self { credits, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.credits, EndingCreditState::load_from_ram(self.ram));
    }

    pub(crate) fn clear_palace_death_count_digit_step(&mut self) {
        self.credits.clear_palace_death_count_digit_step();
        write_le_u16(self.ram, ENDING_WHICH_DUNG, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_palace_death_count_digit_step(&mut self, value: u16) {
        self.credits.palace_death_count_digit_step = value;
        write_le_u16(self.ram, ENDING_WHICH_DUNG, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_palace_death_count_digit_step(&mut self) {
        let value = self.credits.palace_death_count_digit_step.wrapping_add(1);
        self.set_palace_death_count_digit_step(value);
    }

    pub(crate) fn set_death_count_digit_tile_base(&mut self, value: u16) {
        self.credits.death_count_digit_tile_base = value;
        write_le_u16(self.ram, ENDING_CREDIT_DIGIT_CHAR, value);
        self.debug_assert_matches_ram();
    }
}
