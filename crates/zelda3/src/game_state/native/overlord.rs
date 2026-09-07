//! Overlord slot storage and write-through access.

use crate::game_state::constants::{
    OVERLORD_FLOOR, OVERLORD_GEN1, OVERLORD_GEN2, OVERLORD_GEN3, OVERLORD_OFFSET_SPRITE_POS,
    OVERLORD_SPAWNED_AREA, OVERLORD_TYPE, OVERLORD_X_HI, OVERLORD_X_LO, OVERLORD_Y_HI,
    OVERLORD_Y_LO, SPRITE_BUMP_DAMAGE,
};

const OVERLORD_WORK_BASE: usize = OVERLORD_TYPE;
// There are 8 overlord slots, not OVERLORD_SLOT_COUNT (16): the C overlord arrays are
// 8-slot (type@0xb00, x_lo@0xb08, ... floor@0xb40, offset_sprite_pos is uint16×8 @0xb48),
// and the block ends exactly where sprite_stunned (0xb58) begins. Using OVERLORD_SLOT_COUNT
// here projected offset_sprite_pos for 16 slots, spilling stale bytes over sprite_stunned
// (f462000). Cap the work block at the real overlord field extent.
const OVERLORD_REAL_SLOT_COUNT: usize = 8;
const OVERLORD_WORK_END: usize = OVERLORD_OFFSET_SPRITE_POS + OVERLORD_REAL_SLOT_COUNT * 2;
const OVERLORD_WORK_LEN: usize = OVERLORD_WORK_END - OVERLORD_WORK_BASE;

// overlord_spawned_in_area (0xcca) is a one-byte-per-overlord-slot array that ends exactly
// where SPRITE_BUMP_DAMAGE (0xcd2) begins — i.e. only 8 slots (0xcca..0xcd2). The overlord
// arrays are otherwise 8-slot (C: type@0xb00, x_lo@0xb08, ...; only ~8 overlords exist and
// the spawned-area loop reads slots 0..8). Projecting OVERLORD_SLOT_COUNT (16) slots here
// would spill 8 stale spawned-area bytes over the SPRITE_BUMP_DAMAGE array, re-stamping a
// frame-start value over a sprite's mid-frame bump damage (f460431, Helmasaur King boss).
const OVERLORD_SPAWNED_AREA_COUNT: usize = SPRITE_BUMP_DAMAGE - OVERLORD_SPAWNED_AREA;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverlordSlotsState {
    work: Vec<u8>,
    spawned_area: [u8; OVERLORD_SPAWNED_AREA_COUNT],
}

impl Default for OverlordSlotsState {
    fn default() -> Self {
        Self {
            work: vec![0; OVERLORD_WORK_LEN],
            spawned_area: [0; OVERLORD_SPAWNED_AREA_COUNT],
        }
    }
}

impl OverlordSlotsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..OVERLORD_SPAWNED_AREA_COUNT {
            state.spawned_area[slot] = ram.get(OVERLORD_SPAWNED_AREA + slot).copied().unwrap_or(0);
        }
        for (index, value) in state.work.iter_mut().enumerate() {
            *value = ram.get(OVERLORD_WORK_BASE + index).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (index, value) in self.work.iter().copied().enumerate() {
            ram[OVERLORD_WORK_BASE + index] = value;
        }
        ram[OVERLORD_SPAWNED_AREA..(OVERLORD_SPAWNED_AREA_COUNT + OVERLORD_SPAWNED_AREA)]
            .copy_from_slice(&self.spawned_area[..OVERLORD_SPAWNED_AREA_COUNT]);
    }

    pub(crate) fn slot(&self, slot: usize) -> NativeOverlordSlotView<'_> {
        NativeOverlordSlotView { state: self, slot }
    }

    pub(crate) fn slot_mut<'a>(
        &'a mut self,
        ram: &'a mut [u8],
        slot: usize,
    ) -> NativeOverlordSlotBridgeMut<'a> {
        NativeOverlordSlotBridgeMut {
            state: self,
            ram,
            slot,
        }
    }

    fn work_index(offset: usize) -> usize {
        offset - OVERLORD_WORK_BASE
    }

    fn byte_at(&self, offset: usize) -> u8 {
        self.work
            .get(Self::work_index(offset))
            .copied()
            .unwrap_or(0)
    }

    fn set_byte_at(&mut self, offset: usize, value: u8) {
        let index = Self::work_index(offset);
        if self.work.len() < OVERLORD_WORK_LEN {
            self.work.resize(OVERLORD_WORK_LEN, 0);
        }
        self.work[index] = value;
    }

    fn word_at(&self, offset: usize) -> u16 {
        u16::from(self.byte_at(offset)) | (u16::from(self.byte_at(offset + 1)) << 8)
    }

    fn set_word_at(&mut self, offset: usize, value: u16) {
        self.set_byte_at(offset, value as u8);
        self.set_byte_at(offset + 1, (value >> 8) as u8);
    }

    fn x(&self, slot: usize) -> u16 {
        u16::from(self.x_low(slot)) | (u16::from(self.x_high(slot)) << 8)
    }

    fn y(&self, slot: usize) -> u16 {
        u16::from(self.y_low(slot)) | (u16::from(self.y_high(slot)) << 8)
    }

    fn x_low(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_X_LO + slot)
    }

    fn adjacent_x_low_word(&self, slot: usize) -> u16 {
        self.word_at(OVERLORD_X_LO + slot)
    }

    fn x_high(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_X_HI + slot)
    }

    fn y_low(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_Y_LO + slot)
    }

    fn y_high(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_Y_HI + slot)
    }

    fn overlord_type(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_TYPE + slot)
    }

    fn gen1(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_GEN1 + slot)
    }

    fn gen1_word(&self, slot: usize) -> u16 {
        self.word_at(OVERLORD_GEN1 + slot)
    }

    fn gen2(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_GEN2 + slot)
    }

    fn gen2_word(&self, slot: usize) -> u16 {
        self.word_at(OVERLORD_GEN2 + slot)
    }

    fn gen3(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_GEN3 + slot)
    }

    fn floor(&self, slot: usize) -> u8 {
        self.byte_at(OVERLORD_FLOOR + slot)
    }

    fn spawned_area(&self, slot: usize) -> u8 {
        self.spawned_area[slot]
    }

    fn sprite_block_pos(&self, slot: usize) -> u16 {
        self.word_at(OVERLORD_OFFSET_SPRITE_POS + slot * 2)
    }

    fn set_x(&mut self, slot: usize, value: u16) {
        self.set_byte_at(OVERLORD_X_LO + slot, value as u8);
        self.set_byte_at(OVERLORD_X_HI + slot, (value >> 8) as u8);
    }

    fn set_x_low(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_X_LO + slot, value);
    }

    fn set_adjacent_x_low_word(&mut self, slot: usize, value: u16) {
        self.set_word_at(OVERLORD_X_LO + slot, value);
    }

    fn subtract_adjacent_x_low_word(&mut self, slot: usize, value: u16) -> u16 {
        let updated = self.adjacent_x_low_word(slot).wrapping_sub(value);
        self.set_adjacent_x_low_word(slot, updated);
        updated
    }

    fn set_x_high(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_X_HI + slot, value);
    }

    fn increment_x_high(&mut self, slot: usize) {
        let value = self.x_high(slot).wrapping_add(1);
        self.set_x_high(slot, value);
    }

    fn add_x_low(&mut self, slot: usize, value: u8) {
        let value = self.x_low(slot).wrapping_add(value);
        self.set_x_low(slot, value);
    }

    fn set_circle_x(&mut self, slot: usize, value: u16) {
        self.set_byte_at(OVERLORD_X_HI + slot, value as u8);
        self.set_byte_at(OVERLORD_Y_HI + slot, (value >> 8) as u8);
    }

    fn set_circle_y(&mut self, slot: usize, value: u16) {
        self.set_byte_at(OVERLORD_GEN2 + slot, value as u8);
        self.set_byte_at(OVERLORD_FLOOR + slot, (value >> 8) as u8);
    }

    fn set_y(&mut self, slot: usize, value: u16) {
        self.set_byte_at(OVERLORD_Y_LO + slot, value as u8);
        self.set_byte_at(OVERLORD_Y_HI + slot, (value >> 8) as u8);
    }

    fn set_y_low(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_Y_LO + slot, value);
    }

    fn set_y_high(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_Y_HI + slot, value);
    }

    fn subtract_x_low(&mut self, slot: usize, value: u8) {
        let value = self.x_low(slot).wrapping_sub(value);
        self.set_x_low(slot, value);
    }

    fn set_overlord_type(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_TYPE + slot, value);
    }

    fn clear(&mut self, slot: usize) {
        self.set_overlord_type(slot, 0);
    }

    fn set_gen1(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_GEN1 + slot, value);
    }

    fn add_gen1(&mut self, slot: usize, value: u8) {
        let value = self.gen1(slot).wrapping_add(value);
        self.set_gen1(slot, value);
    }

    fn add_gen1_word(&mut self, slot: usize, value: u16) {
        let next = self.gen1_word(slot).wrapping_add(value);
        self.set_word_at(OVERLORD_GEN1 + slot, next);
    }

    fn subtract_gen1(&mut self, slot: usize, value: u8) {
        let value = self.gen1(slot).wrapping_sub(value);
        self.set_gen1(slot, value);
    }

    fn set_gen2(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_GEN2 + slot, value);
    }

    fn add_gen2(&mut self, slot: usize, value: u8) {
        let value = self.gen2(slot).wrapping_add(value);
        self.set_gen2(slot, value);
    }

    fn add_gen2_word(&mut self, slot: usize, value: u16) {
        let next = self.gen2_word(slot).wrapping_add(value);
        self.set_word_at(OVERLORD_GEN2 + slot, next);
    }

    fn set_gen3(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_GEN3 + slot, value);
    }

    fn set_floor(&mut self, slot: usize, value: u8) {
        self.set_byte_at(OVERLORD_FLOOR + slot, value);
    }

    fn set_sprite_block_pos(&mut self, slot: usize, value: u16) {
        self.set_word_at(OVERLORD_OFFSET_SPRITE_POS + slot * 2, value);
    }

    fn set_spawned_area(&mut self, slot: usize, value: u8) {
        self.spawned_area[slot] = value;
    }
}

pub(crate) struct NativeOverlordSlotView<'a> {
    state: &'a OverlordSlotsState,
    slot: usize,
}

impl<'a> NativeOverlordSlotView<'a> {
    pub(crate) fn x(&self) -> u16 {
        self.state.x(self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        self.state.y(self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.state.x_low(self.slot)
    }

    pub(crate) fn adjacent_x_low_word(&self) -> u16 {
        self.state.adjacent_x_low_word(self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.state.x_high(self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.state.y_low(self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.state.y_high(self.slot)
    }

    pub(crate) fn overlord_type(&self) -> u8 {
        self.state.overlord_type(self.slot)
    }

    pub(crate) fn gen1(&self) -> u8 {
        self.state.gen1(self.slot)
    }

    pub(crate) fn gen1_word(&self) -> u16 {
        self.state.gen1_word(self.slot)
    }

    pub(crate) fn gen2(&self) -> u8 {
        self.state.gen2(self.slot)
    }

    pub(crate) fn gen2_word(&self) -> u16 {
        self.state.gen2_word(self.slot)
    }

    pub(crate) fn gen3(&self) -> u8 {
        self.state.gen3(self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        self.state.floor(self.slot)
    }

    pub(crate) fn spawned_area(&self) -> u8 {
        self.state.spawned_area(self.slot)
    }

    pub(crate) fn sprite_block_pos(&self) -> u16 {
        self.state.sprite_block_pos(self.slot)
    }
}

pub(crate) struct NativeOverlordSlotBridgeMut<'a> {
    state: &'a mut OverlordSlotsState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeOverlordSlotBridgeMut<'a> {
    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, OverlordSlotsState::load_from_ram(self.ram));
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.set_x(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.state.set_x_low(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_adjacent_x_low_word(&mut self, value: u16) {
        self.state.set_adjacent_x_low_word(self.slot, value);
        self.sync();
    }

    pub(crate) fn subtract_adjacent_x_low_word(&mut self, value: u16) -> u16 {
        let updated = self.state.subtract_adjacent_x_low_word(self.slot, value);
        self.sync();
        updated
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.state.set_x_high(self.slot, value);
        self.sync();
    }

    pub(crate) fn increment_x_high(&mut self) {
        self.state.increment_x_high(self.slot);
        self.sync();
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        self.state.add_x_low(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_circle_x(&mut self, value: u16) {
        self.state.set_circle_x(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_circle_y(&mut self, value: u16) {
        self.state.set_circle_y(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.set_y(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.state.set_y_low(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.state.set_y_high(self.slot, value);
        self.sync();
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        self.state.subtract_x_low(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_overlord_type(&mut self, value: u8) {
        self.state.set_overlord_type(self.slot, value);
        self.sync();
    }

    pub(crate) fn clear(&mut self) {
        self.state.clear(self.slot);
        self.sync();
    }

    pub(crate) fn set_gen1(&mut self, value: u8) {
        self.state.set_gen1(self.slot, value);
        self.sync();
    }

    pub(crate) fn add_gen1(&mut self, value: u8) {
        self.state.add_gen1(self.slot, value);
        self.sync();
    }

    pub(crate) fn add_gen1_word(&mut self, value: u16) {
        self.state.add_gen1_word(self.slot, value);
        self.sync();
    }

    pub(crate) fn subtract_gen1(&mut self, value: u8) {
        self.state.subtract_gen1(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_gen2(&mut self, value: u8) {
        self.state.set_gen2(self.slot, value);
        self.sync();
    }

    pub(crate) fn add_gen2(&mut self, value: u8) {
        self.state.add_gen2(self.slot, value);
        self.sync();
    }

    pub(crate) fn add_gen2_word(&mut self, value: u16) {
        self.state.add_gen2_word(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_gen3(&mut self, value: u8) {
        self.state.set_gen3(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.state.set_floor(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_sprite_block_pos(&mut self, value: u16) {
        self.state.set_sprite_block_pos(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_spawned_area(&mut self, value: u8) {
        self.state.set_spawned_area(self.slot, value);
        self.sync();
    }
}
