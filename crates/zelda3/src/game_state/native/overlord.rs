//! Named views of the shared overlord and boss scratch bytes in WRAM.

use crate::game_state::constants::{
    OVERLORD_FLOOR, OVERLORD_GEN1, OVERLORD_GEN2, OVERLORD_GEN3, OVERLORD_OFFSET_SPRITE_POS,
    OVERLORD_SPAWNED_AREA, OVERLORD_TYPE, OVERLORD_X_HI, OVERLORD_X_LO, OVERLORD_Y_HI,
    OVERLORD_Y_LO, SPRITE_BUMP_DAMAGE,
};

// Eight physical slots occupy this bank. Some boss routines intentionally index
// across field boundaries, so access is bounded by the bank rather than by slot.
const OVERLORD_WORK_BASE: usize = OVERLORD_TYPE;
const OVERLORD_WORK_END: usize = OVERLORD_OFFSET_SPRITE_POS + 8 * 2;
const OVERLORD_SPAWNED_AREA_COUNT: usize = SPRITE_BUMP_DAMAGE - OVERLORD_SPAWNED_AREA;

pub(crate) struct OverlordSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> OverlordSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    fn byte_at(&self, offset: usize) -> u8 {
        // Preserve the former bank's zero reads beyond its end, without exposing
        // the adjacent sprite-stunned array through an overlord accessor.
        self.ram[OVERLORD_WORK_BASE..OVERLORD_WORK_END]
            .get(offset - OVERLORD_WORK_BASE)
            .copied()
            .unwrap_or(0)
    }

    fn word_at(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.byte_at(offset), self.byte_at(offset + 1)])
    }

    pub(crate) fn x(&self) -> u16 {
        u16::from(self.x_low()) | (u16::from(self.x_high()) << 8)
    }

    pub(crate) fn y(&self) -> u16 {
        u16::from(self.y_low()) | (u16::from(self.y_high()) << 8)
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.byte_at(OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn adjacent_x_low_word(&self) -> u16 {
        self.word_at(OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.byte_at(OVERLORD_X_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.byte_at(OVERLORD_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.byte_at(OVERLORD_Y_HI + self.slot)
    }

    pub(crate) fn overlord_type(&self) -> u8 {
        self.byte_at(OVERLORD_TYPE + self.slot)
    }

    pub(crate) fn gen1(&self) -> u8 {
        self.byte_at(OVERLORD_GEN1 + self.slot)
    }

    pub(crate) fn gen1_word(&self) -> u16 {
        self.word_at(OVERLORD_GEN1 + self.slot)
    }

    pub(crate) fn gen2(&self) -> u8 {
        self.byte_at(OVERLORD_GEN2 + self.slot)
    }

    pub(crate) fn gen2_word(&self) -> u16 {
        self.word_at(OVERLORD_GEN2 + self.slot)
    }

    pub(crate) fn gen3(&self) -> u8 {
        self.byte_at(OVERLORD_GEN3 + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        self.byte_at(OVERLORD_FLOOR + self.slot)
    }

    pub(crate) fn spawned_area(&self) -> u8 {
        self.ram[OVERLORD_SPAWNED_AREA..OVERLORD_SPAWNED_AREA + OVERLORD_SPAWNED_AREA_COUNT]
            [self.slot]
    }

    pub(crate) fn sprite_block_pos(&self) -> u16 {
        self.word_at(OVERLORD_OFFSET_SPRITE_POS + self.slot * 2)
    }
}

pub(crate) struct OverlordSlotMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> OverlordSlotMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    fn view(&self) -> OverlordSlotView<'_> {
        OverlordSlotView::new(self.ram, self.slot)
    }

    fn set_byte_at(&mut self, offset: usize, value: u8) {
        self.ram[OVERLORD_WORK_BASE..OVERLORD_WORK_END][offset - OVERLORD_WORK_BASE] = value;
    }

    fn set_pair_at(&mut self, low: usize, high: usize, value: u16) {
        // Validate both destinations before publishing either byte, as the old
        // bridge only published after the complete native mutation succeeded.
        assert!((OVERLORD_WORK_BASE..OVERLORD_WORK_END).contains(&low));
        assert!((OVERLORD_WORK_BASE..OVERLORD_WORK_END).contains(&high));
        self.set_byte_at(low, value as u8);
        self.set_byte_at(high, (value >> 8) as u8);
    }

    fn set_word_at(&mut self, offset: usize, value: u16) {
        self.set_pair_at(offset, offset + 1, value);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.set_pair_at(OVERLORD_X_LO + self.slot, OVERLORD_X_HI + self.slot, value);
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_X_LO + self.slot, value);
    }

    pub(crate) fn set_adjacent_x_low_word(&mut self, value: u16) {
        self.set_word_at(OVERLORD_X_LO + self.slot, value);
    }

    pub(crate) fn subtract_adjacent_x_low_word(&mut self, value: u16) -> u16 {
        let updated = self.view().adjacent_x_low_word().wrapping_sub(value);
        self.set_adjacent_x_low_word(updated);
        updated
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_X_HI + self.slot, value);
    }

    pub(crate) fn increment_x_high(&mut self) {
        let value = self.view().x_high().wrapping_add(1);
        self.set_x_high(value);
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        let value = self.view().x_low().wrapping_add(value);
        self.set_x_low(value);
    }

    pub(crate) fn set_circle_x(&mut self, value: u16) {
        self.set_pair_at(OVERLORD_X_HI + self.slot, OVERLORD_Y_HI + self.slot, value);
    }

    pub(crate) fn set_circle_y(&mut self, value: u16) {
        self.set_pair_at(OVERLORD_GEN2 + self.slot, OVERLORD_FLOOR + self.slot, value);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.set_pair_at(OVERLORD_Y_LO + self.slot, OVERLORD_Y_HI + self.slot, value);
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_Y_LO + self.slot, value);
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_Y_HI + self.slot, value);
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        let value = self.view().x_low().wrapping_sub(value);
        self.set_x_low(value);
    }

    pub(crate) fn set_overlord_type(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_TYPE + self.slot, value);
    }

    pub(crate) fn clear(&mut self) {
        self.set_overlord_type(0);
    }

    pub(crate) fn set_gen1(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_GEN1 + self.slot, value);
    }

    pub(crate) fn add_gen1(&mut self, value: u8) {
        let value = self.view().gen1().wrapping_add(value);
        self.set_gen1(value);
    }

    pub(crate) fn add_gen1_word(&mut self, value: u16) {
        let next = self.view().gen1_word().wrapping_add(value);
        self.set_word_at(OVERLORD_GEN1 + self.slot, next);
    }

    pub(crate) fn subtract_gen1(&mut self, value: u8) {
        let value = self.view().gen1().wrapping_sub(value);
        self.set_gen1(value);
    }

    pub(crate) fn set_gen2(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_GEN2 + self.slot, value);
    }

    pub(crate) fn add_gen2(&mut self, value: u8) {
        let value = self.view().gen2().wrapping_add(value);
        self.set_gen2(value);
    }

    pub(crate) fn add_gen2_word(&mut self, value: u16) {
        let next = self.view().gen2_word().wrapping_add(value);
        self.set_word_at(OVERLORD_GEN2 + self.slot, next);
    }

    pub(crate) fn set_gen3(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_GEN3 + self.slot, value);
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.set_byte_at(OVERLORD_FLOOR + self.slot, value);
    }

    pub(crate) fn set_sprite_block_pos(&mut self, value: u16) {
        self.set_word_at(OVERLORD_OFFSET_SPRITE_POS + self.slot * 2, value);
    }

    pub(crate) fn set_spawned_area(&mut self, value: u8) {
        self.ram[OVERLORD_SPAWNED_AREA..OVERLORD_SPAWNED_AREA + OVERLORD_SPAWNED_AREA_COUNT]
            [self.slot] = value;
    }
}

#[cfg(test)]
#[path = "overlord_tests.rs"]
mod tests;
