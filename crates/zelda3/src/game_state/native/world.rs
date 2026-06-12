use super::ram_byte;
use crate::game_state::constants::*;
use crate::game_state::WorldStateViewMut;
use crate::types::{read_le_u16, write_le_u16};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldLocationState {
    pub(crate) dungeon_room: u16,
    pub(crate) overworld_screen: u16,
    pub(crate) indoor_flag: u8,
}

impl WorldLocationState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dungeon_room: read_le_u16(ram, DUNGEON_ROOM),
            overworld_screen: read_le_u16(ram, OVERWORLD_SCREEN_INDEX),
            indoor_flag: ram_byte(ram, PLAYER_IS_INDOORS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_ROOM, self.dungeon_room);
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX, self.overworld_screen);
        ram[PLAYER_IS_INDOORS] = self.indoor_flag;
    }

    pub(crate) fn dungeon_room_index(&self) -> u8 {
        self.dungeon_room as u8
    }

    pub(crate) fn overworld_screen_index(&self) -> u8 {
        self.overworld_screen as u8
    }

    pub(crate) fn is_indoors(&self) -> bool {
        self.indoor_flag != 0
    }

    pub(crate) fn is_outdoors(&self) -> bool {
        !self.is_indoors()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldMapUiState {
    pub(crate) map_state: u16,
    pub(crate) map_flags: u8,
    pub(crate) birdtravel_status: u16,
}

impl OverworldMapUiState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            map_state: read_le_u16(ram, OVERWORLD_MAP_STATE),
            map_flags: ram_byte(ram, OVERWORLD_MAP_FLAGS),
            birdtravel_status: read_le_u16(ram, BIRDTRAVEL_STATUS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OVERWORLD_MAP_STATE, self.map_state);
        ram[OVERWORLD_MAP_FLAGS] = self.map_flags;
        write_le_u16(ram, BIRDTRAVEL_STATUS, self.birdtravel_status);
    }

    pub(crate) fn map_state(&self) -> u8 {
        self.map_state as u8
    }

    pub(crate) fn map_state_word(&self) -> u16 {
        self.map_state
    }

    pub(crate) fn birdtravel_status(&self) -> u8 {
        self.birdtravel_status as u8
    }

    pub(crate) fn birdtravel_status_word(&self) -> u16 {
        self.birdtravel_status
    }
}

pub(crate) struct NativeWorldLocationViewMut<'a> {
    world_location: &'a mut WorldLocationState,
    ram_view: WorldStateViewMut<'a>,
}

impl<'a> NativeWorldLocationViewMut<'a> {
    pub(crate) fn new(world_location: &'a mut WorldLocationState, ram: &'a mut [u8]) -> Self {
        *world_location = WorldLocationState::load_from_ram(ram);
        Self {
            world_location,
            ram_view: WorldStateViewMut::new(ram),
        }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.world_location,
            WorldLocationState {
                dungeon_room: self.ram_view.dungeon_room(),
                overworld_screen: self.ram_view.overworld_screen_word(),
                indoor_flag: self.ram_view.indoor_flag(),
            }
        );
    }

    pub(crate) fn set_dungeon_room(&mut self, value: u16) {
        self.world_location.dungeon_room = value;
        self.ram_view.set_dungeon_room(value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        self.world_location.dungeon_room =
            (self.world_location.dungeon_room & 0xff00) | u16::from(value);
        self.ram_view.set_dungeon_room_index(value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.world_location.dungeon_room_index().wrapping_add(value);
        self.set_dungeon_room_index(next);
        next
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.world_location.dungeon_room_index().wrapping_sub(value);
        self.set_dungeon_room_index(next);
        next
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        self.world_location.overworld_screen =
            (self.world_location.overworld_screen & 0xff00) | u16::from(value);
        self.ram_view.set_overworld_screen(value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        self.world_location.overworld_screen = value;
        self.ram_view.set_overworld_screen_word(value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        self.world_location.indoor_flag = value;
        self.ram_view.set_indoor_flag(value);
        self.debug_assert_matches_ram();
    }
}

impl<'a> Deref for NativeWorldLocationViewMut<'a> {
    type Target = WorldStateViewMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ram_view
    }
}

impl<'a> DerefMut for NativeWorldLocationViewMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ram_view
    }
}

pub(crate) struct NativeOverworldMapUiBridgeMut<'a> {
    map_ui: &'a mut OverworldMapUiState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMapUiBridgeMut<'a> {
    pub(crate) fn new(map_ui: &'a mut OverworldMapUiState, ram: &'a mut [u8]) -> Self {
        *map_ui = OverworldMapUiState::load_from_ram(ram);
        Self { map_ui, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.map_ui, OverworldMapUiState::load_from_ram(self.ram));
    }

    pub(crate) fn set_map_state(&mut self, value: u8) {
        self.map_ui.map_state = (self.map_ui.map_state & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_MAP_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_map_state_word(&mut self, value: u16) {
        self.map_ui.map_state = value;
        write_le_u16(self.ram, OVERWORLD_MAP_STATE, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_map_state(&mut self) {
        let next = self.map_ui.map_state().wrapping_add(1);
        self.set_map_state(next);
    }

    pub(crate) fn set_map_flags(&mut self, value: u8) {
        self.map_ui.map_flags = value;
        self.ram[OVERWORLD_MAP_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn and_map_flags(&mut self, value: u8) {
        let next = self.map_ui.map_flags & value;
        self.set_map_flags(next);
    }

    pub(crate) fn or_map_flags(&mut self, value: u8) {
        let next = self.map_ui.map_flags | value;
        self.set_map_flags(next);
    }

    pub(crate) fn set_birdtravel_status(&mut self, value: u8) {
        self.map_ui.birdtravel_status = (self.map_ui.birdtravel_status & 0xff00) | u16::from(value);
        self.ram[BIRDTRAVEL_STATUS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_birdtravel_status_word(&mut self, value: u16) {
        self.map_ui.birdtravel_status = value;
        write_le_u16(self.ram, BIRDTRAVEL_STATUS, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn and_birdtravel_status(&mut self, value: u8) {
        let next = self.map_ui.birdtravel_status() & value;
        self.set_birdtravel_status(next);
    }

    pub(crate) fn decrement_birdtravel_status(&mut self) {
        let next = self.map_ui.birdtravel_status().wrapping_sub(1);
        self.set_birdtravel_status(next);
    }

    pub(crate) fn increment_birdtravel_status(&mut self) {
        let next = self.map_ui.birdtravel_status().wrapping_add(1);
        self.set_birdtravel_status(next);
    }
}
