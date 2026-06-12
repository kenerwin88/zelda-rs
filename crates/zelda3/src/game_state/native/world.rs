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
pub(crate) struct OverworldScreenSizeState {
    pub(crate) big_area: u16,
    pub(crate) big_area_backup: u8,
    pub(crate) right_bottom_scroll_bound: u16,
}

impl OverworldScreenSizeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            big_area: read_le_u16(ram, OVERWORLD_AREA_IS_BIG),
            big_area_backup: ram_byte(ram, OVERWORLD_AREA_IS_BIG_BACKUP),
            right_bottom_scroll_bound: read_le_u16(ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OVERWORLD_AREA_IS_BIG, self.big_area);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = self.big_area_backup;
        write_le_u16(
            ram,
            OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND,
            self.right_bottom_scroll_bound,
        );
    }

    pub(crate) fn is_big_area_word(&self) -> u16 {
        self.big_area
    }

    pub(crate) fn is_big_area(&self) -> bool {
        self.is_big_area_word() != 0
    }

    pub(crate) fn right_bottom_bound_word(&self) -> u16 {
        self.right_bottom_scroll_bound
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldScrollDeltaState {
    bytes: [u8; 3],
}

impl OverworldScrollDeltaState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            bytes: [
                ram_byte(ram, OVERWORLD_SCROLL_DELTA),
                ram_byte(ram, OVERWORLD_SCROLL_DELTA + 1),
                ram_byte(ram, OVERWORLD_SCROLL_DELTA + 2),
            ],
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_SCROLL_DELTA] = self.bytes[0];
        ram[OVERWORLD_SCROLL_DELTA + 1] = self.bytes[1];
        ram[OVERWORLD_SCROLL_DELTA + 2] = self.bytes[2];
    }

    pub(crate) fn vertical_delta_low_byte(&self) -> u8 {
        self.bytes[0]
    }

    pub(crate) fn horizontal_delta_low_byte(&self) -> u8 {
        self.bytes[1]
    }

    pub(crate) fn vertical_delta_word(&self) -> u16 {
        u16::from(self.bytes[0]) | (u16::from(self.bytes[1]) << 8)
    }

    pub(crate) fn horizontal_delta_word(&self) -> u16 {
        u16::from(self.bytes[1]) | (u16::from(self.bytes[2]) << 8)
    }

    pub(crate) fn set_vertical_delta_low_byte(&mut self, value: u8) {
        self.bytes[0] = value;
    }

    pub(crate) fn set_horizontal_delta_low_byte(&mut self, value: u8) {
        self.bytes[1] = value;
    }

    pub(crate) fn set_vertical_delta_word(&mut self, value: u16) {
        self.bytes[0] = value as u8;
        self.bytes[1] = (value >> 8) as u8;
    }

    pub(crate) fn set_horizontal_delta_word(&mut self, value: u16) {
        self.bytes[1] = value as u8;
        self.bytes[2] = (value >> 8) as u8;
    }

    pub(crate) fn clear_vertical_delta_low_byte(&mut self) {
        self.set_vertical_delta_low_byte(0);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OverworldMap16LoadState {
    pub src_off: u16,
    pub dst_off: u16,
    pub y_unit: u16,
}

impl OverworldMap16LoadState {
    fn load_from_ram_at(ram: &[u8], src_off: usize, dst_off: usize, y_unit: usize) -> Self {
        Self {
            src_off: read_le_u16(ram, src_off),
            dst_off: read_le_u16(ram, dst_off),
            y_unit: read_le_u16(ram, y_unit),
        }
    }

    fn write_to_ram_at(&self, ram: &mut [u8], src_off: usize, dst_off: usize, y_unit: usize) {
        write_le_u16(ram, src_off, self.src_off);
        write_le_u16(ram, dst_off, self.dst_off);
        write_le_u16(ram, y_unit, self.y_unit);
    }

    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self::load_from_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF,
            MAP16_LOAD_DST_OFF,
            MAP16_LOAD_Y_UNIT,
        )
    }

    pub(crate) fn load_previous_from_ram(ram: &[u8]) -> Self {
        Self::load_from_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF_PREV,
            MAP16_LOAD_DST_OFF_PREV,
            MAP16_LOAD_Y_UNIT_PREV,
        )
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.write_to_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF,
            MAP16_LOAD_DST_OFF,
            MAP16_LOAD_Y_UNIT,
        );
    }

    pub(crate) fn write_previous_to_ram(&self, ram: &mut [u8]) {
        self.write_to_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF_PREV,
            MAP16_LOAD_DST_OFF_PREV,
            MAP16_LOAD_Y_UNIT_PREV,
        );
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmallOverworldMap16ScrollBackupState {
    pub src_off: u16,
    pub dst_off: u16,
    pub y_unit: u16,
}

impl SmallOverworldMap16ScrollBackupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            src_off: read_le_u16(ram, ORANGE_BLUE_BARRIER_STATE),
            dst_off: read_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
            y_unit: read_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, ORANGE_BLUE_BARRIER_STATE, self.src_off);
        write_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, self.dst_off);
        write_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, self.y_unit);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldMap16State {
    pub(crate) active_load: OverworldMap16LoadState,
    pub(crate) previous_load: OverworldMap16LoadState,
    pub(crate) special_exit_src_off: u16,
    pub(crate) exit_src_off: u16,
    pub(crate) small_scroll_backup: SmallOverworldMap16ScrollBackupState,
}

impl OverworldMap16State {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            active_load: OverworldMap16LoadState::load_from_ram(ram),
            previous_load: OverworldMap16LoadState::load_previous_from_ram(ram),
            special_exit_src_off: read_le_u16(ram, MAP16_LOAD_SRC_OFF_SPEXIT),
            exit_src_off: read_le_u16(ram, MAP16_LOAD_SRC_OFF_EXIT),
            small_scroll_backup: SmallOverworldMap16ScrollBackupState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.active_load.write_to_ram(ram);
        self.previous_load.write_previous_to_ram(ram);
        write_le_u16(ram, MAP16_LOAD_SRC_OFF_SPEXIT, self.special_exit_src_off);
        write_le_u16(ram, MAP16_LOAD_SRC_OFF_EXIT, self.exit_src_off);
        self.small_scroll_backup.write_to_ram(ram);
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldState {
    pub(crate) map_ui: OverworldMapUiState,
    pub(crate) map_zoom: OverworldMapZoomState,
    pub(crate) screen_size: OverworldScreenSizeState,
    pub(crate) scroll_delta: OverworldScrollDeltaState,
    pub(crate) map16: OverworldMap16State,
    pub(crate) entrance: OverworldEntranceState,
    pub(crate) exit: OverworldExitState,
    pub(crate) transition: OverworldTransitionState,
}

impl OverworldState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            map_ui: OverworldMapUiState::load_from_ram(ram),
            map_zoom: OverworldMapZoomState::load_from_ram(ram),
            screen_size: OverworldScreenSizeState::load_from_ram(ram),
            scroll_delta: OverworldScrollDeltaState::load_from_ram(ram),
            map16: OverworldMap16State::load_from_ram(ram),
            entrance: OverworldEntranceState::load_from_ram(ram),
            exit: OverworldExitState::load_from_ram(ram),
            transition: OverworldTransitionState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.map_ui.write_to_ram(ram);
        self.map_zoom.write_to_ram(ram);
        self.screen_size.write_to_ram(ram);
        self.scroll_delta.write_to_ram(ram);
        self.map16.write_to_ram(ram);
        self.entrance.write_to_ram(ram);
        self.exit.write_to_ram(ram);
        self.transition.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldState {
    pub(crate) location: WorldLocationState,
    pub(crate) overworld: OverworldState,
}

impl WorldState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            location: WorldLocationState::load_from_ram(ram),
            overworld: OverworldState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.location.write_to_ram(ram);
        self.overworld.write_to_ram(ram);
    }
}

pub(crate) struct NativeWorldLocationBridgeMut<'a> {
    world_location: &'a mut WorldLocationState,
    ram_view: WorldStateViewMut<'a>,
}

impl<'a> NativeWorldLocationBridgeMut<'a> {
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

impl<'a> Deref for NativeWorldLocationBridgeMut<'a> {
    type Target = WorldStateViewMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ram_view
    }
}

impl<'a> DerefMut for NativeWorldLocationBridgeMut<'a> {
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

pub(crate) struct NativeOverworldScreenSizeBridgeMut<'a> {
    screen_size: &'a mut OverworldScreenSizeState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldScreenSizeBridgeMut<'a> {
    pub(crate) fn new(screen_size: &'a mut OverworldScreenSizeState, ram: &'a mut [u8]) -> Self {
        *screen_size = OverworldScreenSizeState::load_from_ram(ram);
        Self { screen_size, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.screen_size,
            OverworldScreenSizeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_big_area_high(&mut self) {
        self.screen_size.big_area &= 0x00ff;
        self.ram[OVERWORLD_AREA_IS_BIG + 1] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_big_area_low(&mut self, value: u8) {
        self.screen_size.big_area = (self.screen_size.big_area & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_AREA_IS_BIG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn backup_big_area_low(&mut self) {
        self.screen_size.big_area_backup = self.screen_size.big_area as u8;
        self.ram[OVERWORLD_AREA_IS_BIG_BACKUP] = self.screen_size.big_area_backup;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_right_bottom_bound_low(&mut self, value: u8) {
        self.screen_size.right_bottom_scroll_bound =
            (self.screen_size.right_bottom_scroll_bound & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_right_bottom_bound_high(&mut self, value: u8) {
        self.screen_size.right_bottom_scroll_bound =
            (self.screen_size.right_bottom_scroll_bound & 0x00ff) | (u16::from(value) << 8);
        self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND + 1] = value;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldScrollDeltaBridgeMut<'a> {
    scroll_delta: &'a mut OverworldScrollDeltaState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldScrollDeltaBridgeMut<'a> {
    pub(crate) fn new(scroll_delta: &'a mut OverworldScrollDeltaState, ram: &'a mut [u8]) -> Self {
        *scroll_delta = OverworldScrollDeltaState::load_from_ram(ram);
        Self { scroll_delta, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scroll_delta,
            OverworldScrollDeltaState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_vertical_delta_low_byte(&mut self, value: u8) {
        self.scroll_delta.set_vertical_delta_low_byte(value);
        self.ram[OVERWORLD_SCROLL_DELTA] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_horizontal_delta_low_byte(&mut self, value: u8) {
        self.scroll_delta.set_horizontal_delta_low_byte(value);
        self.ram[OVERWORLD_SCROLL_DELTA + 1] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_vertical_delta_word(&mut self, value: u16) {
        self.scroll_delta.set_vertical_delta_word(value);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_horizontal_delta_word(&mut self, value: u16) {
        self.scroll_delta.set_horizontal_delta_word(value);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA + 1, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_vertical_delta_low_byte(&mut self) {
        self.scroll_delta.clear_vertical_delta_low_byte();
        self.ram[OVERWORLD_SCROLL_DELTA] = 0;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldMap16BridgeMut<'a> {
    map16: &'a mut OverworldMap16State,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMap16BridgeMut<'a> {
    pub(crate) fn new(map16: &'a mut OverworldMap16State, ram: &'a mut [u8]) -> Self {
        *map16 = OverworldMap16State::load_from_ram(ram);
        Self { map16, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.map16, OverworldMap16State::load_from_ram(self.ram));
    }

    pub(crate) fn set_active_load(&mut self, state: OverworldMap16LoadState) {
        self.map16.active_load = state;
        state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_previous_load(&mut self, state: OverworldMap16LoadState) {
        self.map16.previous_load = state;
        state.write_previous_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_special_exit_src_off(&mut self, src_off: u16) {
        self.map16.special_exit_src_off = src_off;
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_SPEXIT, src_off);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_exit_src_off(&mut self, src_off: u16) {
        self.map16.exit_src_off = src_off;
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_EXIT, src_off);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_small_scroll_backup(&mut self, state: SmallOverworldMap16ScrollBackupState) {
        self.map16.small_scroll_backup = state;
        state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
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
