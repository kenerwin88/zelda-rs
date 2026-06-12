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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldMapZoomState {
    pub(crate) step_counter: u8,
    pub(crate) timer: u8,
}

impl OverworldMapZoomState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            step_counter: ram_byte(ram, MODE7_ZOOM_STEP_COUNTER),
            timer: ram_byte(ram, TIMER_FOR_MODE7_ZOOM),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MODE7_ZOOM_STEP_COUNTER] = self.step_counter;
        ram[TIMER_FOR_MODE7_ZOOM] = self.timer;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldEntranceState {
    pub(crate) special_entrance_trigger: u8,
    pub(crate) sequence_counter: u8,
}

impl OverworldEntranceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            special_entrance_trigger: ram_byte(ram, TRIGGER_SPECIAL_ENTRANCE),
            sequence_counter: ram_byte(ram, OVERWORLD_ENTRANCE_SEQUENCE_COUNTER),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TRIGGER_SPECIAL_ENTRANCE] = self.special_entrance_trigger;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = self.sequence_counter;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldExitState {
    pub(crate) exit_screen: u16,
    pub(crate) special_exit_screen: u16,
}

impl OverworldExitState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            exit_screen: read_le_u16(ram, OVERWORLD_SCREEN_INDEX_EXIT),
            special_exit_screen: read_le_u16(ram, OVERWORLD_SCREEN_INDEX_SPEXIT),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX_EXIT, self.exit_screen);
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX_SPEXIT, self.special_exit_screen);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldTransitionState {
    pub(crate) edge_direction_bits: u16,
    pub(crate) direction_bits: u16,
    pub(crate) direction_enum: u8,
    pub(crate) screen_transition: u16,
    pub(crate) transition_counter: u8,
    pub(crate) countdown: u8,
    pub(crate) previous_direction_bits: u16,
    pub(crate) previous_direction_bits2: u16,
    pub(crate) previous_screen_transition: u8,
}

impl OverworldTransitionState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            edge_direction_bits: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS),
            direction_bits: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2),
            direction_enum: ram_byte(ram, OVERWORLD_TRANSITION_DIR),
            screen_transition: read_le_u16(ram, OVERWORLD_SCREEN_TRANSITION),
            transition_counter: ram_byte(ram, TRANSITION_COUNTER),
            countdown: ram_byte(ram, OW_COUNTDOWN_TRANSITION),
            previous_direction_bits: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV),
            previous_direction_bits2: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV),
            previous_screen_transition: ram_byte(ram, OVERWORLD_SCREEN_TRANSITION_PREV),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(
            ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS,
            self.edge_direction_bits,
        );
        write_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, self.direction_bits);
        ram[OVERWORLD_TRANSITION_DIR] = self.direction_enum;
        write_le_u16(ram, OVERWORLD_SCREEN_TRANSITION, self.screen_transition);
        ram[TRANSITION_COUNTER] = self.transition_counter;
        ram[OW_COUNTDOWN_TRANSITION] = self.countdown;
        write_le_u16(
            ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV,
            self.previous_direction_bits,
        );
        write_le_u16(
            ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV,
            self.previous_direction_bits2,
        );
        ram[OVERWORLD_SCREEN_TRANSITION_PREV] = self.previous_screen_transition;
    }

    pub(crate) fn edge_direction_bits(&self) -> u8 {
        self.edge_direction_bits as u8
    }

    pub(crate) fn direction_bits(&self) -> u8 {
        self.direction_bits as u8
    }

    pub(crate) fn direction_bits_word(&self) -> u16 {
        self.direction_bits
    }

    pub(crate) fn direction_enum(&self) -> u8 {
        self.direction_enum
    }

    pub(crate) fn has_direction_bits(&self) -> bool {
        self.direction_bits() != 0
    }

    pub(crate) fn screen_transition(&self) -> u8 {
        self.screen_transition as u8
    }

    pub(crate) fn screen_transition_word(&self) -> u16 {
        self.screen_transition
    }

    pub(crate) fn countdown(&self) -> u8 {
        self.countdown
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

pub(crate) struct NativeOverworldMapZoomBridgeMut<'a> {
    zoom: &'a mut OverworldMapZoomState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMapZoomBridgeMut<'a> {
    pub(crate) fn new(zoom: &'a mut OverworldMapZoomState, ram: &'a mut [u8]) -> Self {
        *zoom = OverworldMapZoomState::load_from_ram(ram);
        Self { zoom, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.zoom, OverworldMapZoomState::load_from_ram(self.ram));
    }

    pub(crate) fn set_step_counter(&mut self, value: u8) {
        self.zoom.step_counter = value;
        self.ram[MODE7_ZOOM_STEP_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.zoom.timer = value;
        self.ram[TIMER_FOR_MODE7_ZOOM] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_timer(&mut self) {
        let next = self.zoom.timer.wrapping_sub(1);
        self.set_timer(next);
    }
}

pub(crate) struct NativeOverworldEntranceBridgeMut<'a> {
    entrance: &'a mut OverworldEntranceState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldEntranceBridgeMut<'a> {
    pub(crate) fn new(entrance: &'a mut OverworldEntranceState, ram: &'a mut [u8]) -> Self {
        *entrance = OverworldEntranceState::load_from_ram(ram);
        Self { entrance, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.entrance,
            OverworldEntranceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_special_entrance_trigger(&mut self, value: u8) {
        self.entrance.special_entrance_trigger = value;
        self.ram[TRIGGER_SPECIAL_ENTRANCE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_special_entrance_trigger(&mut self) {
        self.set_special_entrance_trigger(0);
    }

    pub(crate) fn set_sequence_counter(&mut self, value: u8) {
        self.entrance.sequence_counter = value;
        self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sequence_counter(&mut self) {
        self.set_sequence_counter(0);
    }

    pub(crate) fn increment_sequence_counter(&mut self) -> u8 {
        let next = self.entrance.sequence_counter.wrapping_add(1);
        self.set_sequence_counter(next);
        next
    }

    pub(crate) fn decrement_sequence_counter(&mut self) -> u8 {
        let next = self.entrance.sequence_counter.wrapping_sub(1);
        self.set_sequence_counter(next);
        next
    }
}

pub(crate) struct NativeOverworldExitBridgeMut<'a> {
    exit: &'a mut OverworldExitState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldExitBridgeMut<'a> {
    pub(crate) fn new(exit: &'a mut OverworldExitState, ram: &'a mut [u8]) -> Self {
        *exit = OverworldExitState::load_from_ram(ram);
        Self { exit, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.exit, OverworldExitState::load_from_ram(self.ram));
    }

    pub(crate) fn set_exit_screen(&mut self, value: u16) {
        self.exit.exit_screen = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_EXIT, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_special_exit_screen(&mut self, value: u16) {
        self.exit.special_exit_screen = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_SPEXIT, value);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldTransitionBridgeMut<'a> {
    transition: &'a mut OverworldTransitionState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldTransitionBridgeMut<'a> {
    pub(crate) fn new(transition: &'a mut OverworldTransitionState, ram: &'a mut [u8]) -> Self {
        *transition = OverworldTransitionState::load_from_ram(ram);
        Self { transition, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.transition,
            OverworldTransitionState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_direction_bits(&mut self, value: u8) {
        self.transition.direction_bits =
            (self.transition.direction_bits & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_bits_word(&mut self, value: u16) {
        self.transition.direction_bits = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_bits(&mut self) {
        self.set_direction_bits(0);
    }

    pub(crate) fn clear_direction_bits_word(&mut self) {
        self.set_direction_bits_word(0);
    }

    pub(crate) fn and_direction_bits(&mut self, value: u8) {
        let next = self.transition.direction_bits() & value;
        self.set_direction_bits(next);
    }

    pub(crate) fn or_direction_bits(&mut self, value: u8) {
        let next = self.transition.direction_bits() | value;
        self.set_direction_bits(next);
    }

    pub(crate) fn or_direction_bits_word(&mut self, value: u16) -> u16 {
        let next = self.transition.direction_bits_word() | value;
        self.set_direction_bits_word(next);
        next
    }

    pub(crate) fn set_edge_direction_bits(&mut self, value: u8) {
        self.transition.edge_direction_bits =
            (self.transition.edge_direction_bits & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_edge_direction_bits_word(&mut self, value: u16) {
        self.transition.edge_direction_bits = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_edge_direction_bits(&mut self) {
        self.set_edge_direction_bits(0);
    }

    pub(crate) fn set_direction_enum(&mut self, value: u8) {
        self.transition.direction_enum = value;
        self.ram[OVERWORLD_TRANSITION_DIR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_screen_transition(&mut self, value: u8) {
        self.transition.screen_transition =
            (self.transition.screen_transition & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_SCREEN_TRANSITION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_screen_transition_word(&mut self, value: u16) {
        self.transition.screen_transition = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANSITION, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_screen_transition(&mut self) {
        self.set_screen_transition(0);
    }

    pub(crate) fn set_transition_counter(&mut self, value: u8) {
        self.transition.transition_counter = value;
        self.ram[TRANSITION_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_transition_counter(&mut self) -> u8 {
        let next = self.transition.transition_counter.wrapping_add(1);
        self.set_transition_counter(next);
        next
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.transition.countdown = value;
        self.ram[OW_COUNTDOWN_TRANSITION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_countdown(&mut self) -> u8 {
        let next = self.transition.countdown.wrapping_sub(1);
        self.set_countdown(next);
        next
    }

    pub(crate) fn save_previous_direction_bits(&mut self) {
        self.transition.previous_direction_bits = u16::from(self.transition.edge_direction_bits());
        self.transition.previous_direction_bits2 = self.transition.direction_bits_word();
        write_le_u16(
            self.ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV,
            self.transition.previous_direction_bits,
        );
        write_le_u16(
            self.ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV,
            self.transition.previous_direction_bits2,
        );
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_previous_direction_bits(&mut self) {
        self.set_edge_direction_bits_word(self.transition.previous_direction_bits);
        self.set_direction_bits_word(self.transition.previous_direction_bits2);
    }

    pub(crate) fn set_previous_screen_transition(&mut self, value: u8) {
        self.transition.previous_screen_transition = value;
        self.ram[OVERWORLD_SCREEN_TRANSITION_PREV] = value;
        self.debug_assert_matches_ram();
    }
}
