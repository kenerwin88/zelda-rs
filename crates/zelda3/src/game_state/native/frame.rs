use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::write_le_u16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct FrameState {
    pub(crate) main_module: u8,
    pub(crate) submodule: u8,
    pub(crate) subsubmodule: u8,
    pub(crate) frame_counter: u8,
    pub(crate) saved_module_for_menu: u8,
    pub(crate) modal_pause_flag: u8,
}

impl FrameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            main_module: ram_byte(ram, MAIN_MODULE),
            submodule: ram_byte(ram, SUBMODULE),
            subsubmodule: ram_byte(ram, SUBSUBMODULE),
            frame_counter: ram_byte(ram, FRAME_COUNTER),
            saved_module_for_menu: ram_byte(ram, SAVED_MODULE_FOR_MENU),
            modal_pause_flag: ram_byte(ram, MODAL_PAUSE_FLAG),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MAIN_MODULE] = self.main_module;
        ram[SUBMODULE] = self.submodule;
        ram[SUBSUBMODULE] = self.subsubmodule;
        ram[FRAME_COUNTER] = self.frame_counter;
        ram[SAVED_MODULE_FOR_MENU] = self.saved_module_for_menu;
        ram[MODAL_PAUSE_FLAG] = self.modal_pause_flag;
    }

    pub(crate) fn main_module_word(&self) -> u16 {
        u16::from(self.main_module) | (u16::from(self.submodule) << 8)
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
        self.frame.saved_module_for_menu = value;
        self.ram[SAVED_MODULE_FOR_MENU] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.set_saved_module_for_menu(0);
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.set_saved_module_for_menu(self.frame.main_module);
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.set_saved_module_for_menu(self.frame.submodule);
    }

    pub(crate) fn clear_modal_pause_flag(&mut self) {
        self.set_modal_pause_flag(0);
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        self.frame.modal_pause_flag = value;
        self.ram[MODAL_PAUSE_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        let value = self.frame.modal_pause_flag.wrapping_add(1);
        self.set_modal_pause_flag(value);
        value
    }
}
