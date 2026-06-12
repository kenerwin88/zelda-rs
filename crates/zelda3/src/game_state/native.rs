//! Native game-state model.
//!
//! Byte-backed views remain the compatibility surface while native state is
//! proven subsystem by subsystem. Native structs own domain fields and can be
//! projected to or loaded from WRAM during the transition.

use crate::game_state::constants::*;
use crate::game_state::WorldStateViewMut;
use crate::types::{read_le_u16, write_le_u16};
use std::ops::{Deref, DerefMut};

fn ram_byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GameState {
    pub(crate) frame: FrameState,
    pub(crate) world_location: WorldLocationState,
}

impl GameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            frame: FrameState::load_from_ram(ram),
            world_location: WorldLocationState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.frame.write_to_ram(ram);
        self.world_location.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct FrameState {
    pub(crate) main_module: u8,
    pub(crate) submodule: u8,
    pub(crate) subsubmodule: u8,
    pub(crate) frame_counter: u8,
}

impl FrameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            main_module: ram_byte(ram, MAIN_MODULE),
            submodule: ram_byte(ram, SUBMODULE),
            subsubmodule: ram_byte(ram, SUBSUBMODULE),
            frame_counter: ram_byte(ram, FRAME_COUNTER),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MAIN_MODULE] = self.main_module;
        ram[SUBMODULE] = self.submodule;
        ram[SUBSUBMODULE] = self.subsubmodule;
        ram[FRAME_COUNTER] = self.frame_counter;
    }

    pub(crate) fn main_module_word(&self) -> u16 {
        u16::from(self.main_module) | (u16::from(self.submodule) << 8)
    }
}

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

pub(crate) struct NativeFrameStateView<'a> {
    frame: &'a FrameState,
    ram: &'a [u8],
}

impl<'a> NativeFrameStateView<'a> {
    pub(crate) fn new(frame: &'a FrameState, ram: &'a [u8]) -> Self {
        Self { frame, ram }
    }

    pub(crate) fn main_module(&self) -> u8 {
        self.frame.main_module
    }

    pub(crate) fn main_module_word(&self) -> u16 {
        self.frame.main_module_word()
    }

    pub(crate) fn submodule(&self) -> u8 {
        self.frame.submodule
    }

    pub(crate) fn subsubmodule(&self) -> u8 {
        self.frame.subsubmodule
    }

    pub(crate) fn frame_counter(&self) -> u8 {
        self.frame.frame_counter
    }

    pub(crate) fn saved_module_for_menu(&self) -> u8 {
        ram_byte(self.ram, SAVED_MODULE_FOR_MENU)
    }

    pub(crate) fn raw_sfx_pan_value(&self) -> u8 {
        ram_byte(self.ram, RAW_SFX_PAN_VALUE)
    }

    pub(crate) fn modal_pause_flag(&self) -> u8 {
        ram_byte(self.ram, MODAL_PAUSE_FLAG)
    }

    pub(crate) fn nmi_thread_active(&self) -> bool {
        ram_byte(self.ram, NMI_THREAD_ACTIVE) != 0
    }

    pub(crate) fn selected_run_thread(&self) -> u8 {
        if self.nmi_thread_active()
            && crate::types::read_le_u16(self.ram, POLY_THREAD_STACK) != 0x1f31
        {
            RUN_POLY_THREAD
        } else {
            RUN_MAIN_THREAD
        }
    }
}

pub(crate) struct NativeFrameStateViewMut<'a> {
    frame: &'a mut FrameState,
    ram: &'a mut [u8],
}

impl<'a> NativeFrameStateViewMut<'a> {
    pub(crate) fn new(frame: &'a mut FrameState, ram: &'a mut [u8]) -> Self {
        *frame = FrameState::load_from_ram(ram);
        Self { frame, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.frame, FrameState::load_from_ram(self.ram));
    }

    pub(crate) fn set_main_module(&mut self, value: u8) {
        self.frame.main_module = value;
        self.ram[MAIN_MODULE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        self.frame.main_module = value as u8;
        self.frame.submodule = (value >> 8) as u8;
        write_le_u16(self.ram, MAIN_MODULE, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        self.frame.submodule = value;
        self.ram[SUBMODULE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.frame.subsubmodule = value;
        self.ram[SUBSUBMODULE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_submodule(&mut self) {
        let value = self.frame.submodule.wrapping_add(1);
        self.set_submodule(value);
    }

    pub(crate) fn decrement_submodule(&mut self) {
        let value = self.frame.submodule.wrapping_sub(1);
        self.set_submodule(value);
    }

    pub(crate) fn increment_subsubmodule(&mut self) {
        let value = self.frame.subsubmodule.wrapping_add(1);
        self.set_subsubmodule(value);
    }

    pub(crate) fn decrement_subsubmodule(&mut self) {
        let value = self.frame.subsubmodule.wrapping_sub(1);
        self.set_subsubmodule(value);
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.frame.frame_counter = value;
        self.ram[FRAME_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        let value = self.frame.frame_counter.wrapping_add(1);
        self.set_frame_counter(value);
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.ram[SAVED_MODULE_FOR_MENU] = value;
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.set_saved_module_for_menu(0);
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.ram[SAVED_MODULE_FOR_MENU] = self.frame.main_module;
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.ram[SAVED_MODULE_FOR_MENU] = self.frame.submodule;
    }

    pub(crate) fn clear_modal_pause_flag(&mut self) {
        self.ram[MODAL_PAUSE_FLAG] = 0;
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        self.ram[MODAL_PAUSE_FLAG] = value;
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        self.ram[MODAL_PAUSE_FLAG] = self.ram[MODAL_PAUSE_FLAG].wrapping_add(1);
        self.ram[MODAL_PAUSE_FLAG]
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

#[cfg(test)]
mod tests {
    use super::*;
    use snes::WRAM_SIZE;

    #[test]
    fn frame_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 7;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 9;
        ram[FRAME_COUNTER] = 0x42;

        let mut frame = FrameState::load_from_ram(&ram);
        assert_eq!(frame.main_module, 7);
        assert_eq!(frame.main_module_word(), 0x0207);
        assert_eq!(frame.submodule, 2);
        assert_eq!(frame.subsubmodule, 9);
        assert_eq!(frame.frame_counter, 0x42);

        frame.main_module = 14;
        frame.submodule = 3;
        frame.subsubmodule = 1;
        frame.frame_counter = 0x80;
        frame.write_to_ram(&mut ram);

        assert_eq!(ram[MAIN_MODULE], 14);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[SUBSUBMODULE], 1);
        assert_eq!(ram[FRAME_COUNTER], 0x80);
    }

    #[test]
    fn native_frame_mut_view_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 1;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 3;
        ram[FRAME_COUNTER] = 4;

        let mut frame = FrameState::default();
        {
            let mut view = NativeFrameStateViewMut::new(&mut frame, &mut ram);
            view.increment_submodule();
            view.set_subsubmodule(9);
            view.increment_frame_counter();
            view.save_main_module_for_menu();
        }

        assert_eq!(frame.main_module, 1);
        assert_eq!(frame.submodule, 3);
        assert_eq!(frame.subsubmodule, 9);
        assert_eq!(frame.frame_counter, 5);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[SUBSUBMODULE], 9);
        assert_eq!(ram[FRAME_COUNTER], 5);
        assert_eq!(ram[SAVED_MODULE_FOR_MENU], 1);
    }

    #[test]
    fn world_location_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;

        let mut world = WorldLocationState::load_from_ram(&ram);
        assert_eq!(world.dungeon_room, 0x0124);
        assert_eq!(world.dungeon_room_index(), 0x24);
        assert_eq!(world.overworld_screen, 0x0040);
        assert_eq!(world.overworld_screen_index(), 0x40);
        assert!(world.is_indoors());
        assert!(!world.is_outdoors());

        world.dungeon_room = 0x0181;
        world.overworld_screen = 0x005b;
        world.indoor_flag = 0;
        world.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    }

    #[test]
    fn native_world_location_mut_view_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;

        let mut world = WorldLocationState::default();
        {
            let mut view = NativeWorldLocationViewMut::new(&mut world, &mut ram);
            view.increment_dungeon_room_index_by(2);
            view.set_overworld_screen(0x5b);
            view.set_indoor_flag(0);
        }

        assert_eq!(world.dungeon_room, 0x0126);
        assert_eq!(world.overworld_screen, 0x005b);
        assert_eq!(world.indoor_flag, 0);
        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0126);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    }
}
