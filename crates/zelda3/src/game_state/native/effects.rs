use crate::game_state::constants::{DOOR_DEBRIS_DIRECTION, DOOR_DEBRIS_X, DOOR_DEBRIS_Y};
use crate::types::{read_le_u16, write_le_u16};

const DOOR_DEBRIS_BANK_LEN: usize = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EffectState {
    pub(crate) door_debris: DoorDebrisState,
}

impl EffectState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            door_debris: DoorDebrisState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.door_debris.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DoorDebrisState {
    x_bytes: [u8; DOOR_DEBRIS_BANK_LEN],
    y_bytes: [u8; DOOR_DEBRIS_BANK_LEN],
    directions: [u8; DOOR_DEBRIS_BANK_LEN],
}

impl DoorDebrisState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x_bytes: read_bank(ram, DOOR_DEBRIS_X),
            y_bytes: read_bank(ram, DOOR_DEBRIS_Y),
            directions: read_bank(ram, DOOR_DEBRIS_DIRECTION),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_bank(ram, DOOR_DEBRIS_X, self.x_bytes);
        write_bank(ram, DOOR_DEBRIS_Y, self.y_bytes);
        write_bank(ram, DOOR_DEBRIS_DIRECTION, self.directions);
    }

    pub(crate) fn x(&self, slot: usize) -> u8 {
        self.x_bytes.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn y(&self, slot: usize) -> u8 {
        self.y_bytes.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn direction(&self, slot: usize) -> u8 {
        self.directions.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn x_word(&self, slot: usize) -> u16 {
        word_from_bank(self.x_bytes, slot)
    }

    pub(crate) fn y_word(&self, slot: usize) -> u16 {
        word_from_bank(self.y_bytes, slot)
    }
}

fn read_bank(ram: &[u8], base: usize) -> [u8; DOOR_DEBRIS_BANK_LEN] {
    let mut bank = [0; DOOR_DEBRIS_BANK_LEN];
    for (index, value) in bank.iter_mut().enumerate() {
        *value = ram.get(base + index).copied().unwrap_or(0);
    }
    bank
}

fn write_bank(ram: &mut [u8], base: usize, bank: [u8; DOOR_DEBRIS_BANK_LEN]) {
    for (index, value) in bank.iter().copied().enumerate() {
        ram[base + index] = value;
    }
}

fn word_from_bank(bank: [u8; DOOR_DEBRIS_BANK_LEN], slot: usize) -> u16 {
    if slot * 2 + 1 >= bank.len() {
        return 0;
    }
    let mut bytes = [0; DOOR_DEBRIS_BANK_LEN];
    bytes.copy_from_slice(&bank);
    read_le_u16(&bytes, slot * 2)
}

pub(crate) struct DoorDebrisView<'a> {
    state: &'a DoorDebrisState,
}

impl<'a> DoorDebrisView<'a> {
    pub(crate) fn new(state: &'a DoorDebrisState) -> Self {
        Self { state }
    }

    pub(crate) fn x(&self, slot: usize) -> u8 {
        self.state.x(slot)
    }

    pub(crate) fn y(&self, slot: usize) -> u8 {
        self.state.y(slot)
    }

    pub(crate) fn direction(&self, slot: usize) -> u8 {
        self.state.direction(slot)
    }

    pub(crate) fn x_word(&self, slot: usize) -> u16 {
        self.state.x_word(slot)
    }

    pub(crate) fn y_word(&self, slot: usize) -> u16 {
        self.state.y_word(slot)
    }
}

pub(crate) struct NativeDoorDebrisBridgeMut<'a> {
    state: &'a mut DoorDebrisState,
    ram: &'a mut [u8],
}

impl<'a> NativeDoorDebrisBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DoorDebrisState, ram: &'a mut [u8]) -> Self {
        *state = DoorDebrisState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn set_direction(&mut self, slot: usize, value: u8) {
        if let Some(direction) = self.state.directions.get_mut(slot) {
            *direction = value;
            self.sync();
        }
    }

    pub(crate) fn set_y_low_and_x_low_from_word(&mut self, slot: usize, value: u16) {
        if slot < DOOR_DEBRIS_BANK_LEN {
            self.state.y_bytes[slot] = value as u8;
            self.state.x_bytes[slot] = (value >> 8) as u8;
            self.sync();
        }
    }

    pub(crate) fn set_x_word(&mut self, slot: usize, value: u16) {
        self.write_word_to_x_bank(slot, value);
        self.sync();
    }

    pub(crate) fn set_y_word(&mut self, slot: usize, value: u16) {
        self.write_word_to_y_bank(slot, value);
        self.sync();
    }

    fn write_word_to_x_bank(&mut self, slot: usize, value: u16) {
        if slot * 2 + 1 < DOOR_DEBRIS_BANK_LEN {
            write_le_u16(&mut self.state.x_bytes, slot * 2, value);
        }
    }

    fn write_word_to_y_bank(&mut self, slot: usize, value: u16) {
        if slot * 2 + 1 < DOOR_DEBRIS_BANK_LEN {
            write_le_u16(&mut self.state.y_bytes, slot * 2, value);
        }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DoorDebrisState::load_from_ram(self.ram));
    }
}
