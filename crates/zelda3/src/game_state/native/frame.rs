use super::ram_byte;
use crate::game_state::constants::*;

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

    pub(crate) fn set_main_module(&mut self, value: u8) {
        self.main_module = value;
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        self.main_module = value as u8;
        self.submodule = (value >> 8) as u8;
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        self.submodule = value;
    }

    pub(crate) fn increment_submodule(&mut self) {
        self.submodule = self.submodule.wrapping_add(1);
    }

    pub(crate) fn decrement_submodule(&mut self) {
        self.submodule = self.submodule.wrapping_sub(1);
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.subsubmodule = value;
    }

    pub(crate) fn increment_subsubmodule(&mut self) {
        self.subsubmodule = self.subsubmodule.wrapping_add(1);
    }

    pub(crate) fn decrement_subsubmodule(&mut self) {
        self.subsubmodule = self.subsubmodule.wrapping_sub(1);
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.frame_counter = value;
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.saved_module_for_menu = value;
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.saved_module_for_menu = 0;
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.saved_module_for_menu = self.main_module;
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.saved_module_for_menu = self.submodule;
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        self.modal_pause_flag = value;
    }

    pub(crate) fn clear_modal_pause_flag(&mut self) {
        self.modal_pause_flag = 0;
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        self.modal_pause_flag = self.modal_pause_flag.wrapping_add(1);
        self.modal_pause_flag
    }
}

pub(crate) struct NativeFrameStateBridgeMut<'a> {
    frame: &'a mut FrameState,
    ram: &'a mut [u8],
}

impl<'a> NativeFrameStateBridgeMut<'a> {
    pub(crate) fn new(frame: &'a mut FrameState, ram: &'a mut [u8]) -> Self {
        Self { frame, ram }
    }

    fn sync(&mut self) {
        self.frame.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.frame, FrameState::load_from_ram(self.ram));
    }

    pub(crate) fn set_main_module(&mut self, value: u8) {
        crate::types::ww_check(0x10, 1, "set_main_module", u32::from(value));
        self.frame.set_main_module(value);
        self.sync();
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        crate::types::ww_check(0x10, 2, "set_main_module_word", u32::from(value));
        self.frame.set_main_module_word(value);
        self.sync();
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        crate::types::ww_check(0x11, 1, "set_submodule", u32::from(value));
        self.frame.set_submodule(value);
        self.sync();
    }

    #[track_caller]
    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        crate::types::ww_check(0xb0, 1, "set_subsubmodule", u32::from(value));
        self.frame.set_subsubmodule(value);
        self.sync();
    }

    #[track_caller]
    pub(crate) fn increment_submodule(&mut self) {
        self.frame.increment_submodule();
        crate::types::ww_check(
            0x11,
            1,
            "increment_submodule",
            u32::from(self.frame.submodule),
        );
        self.sync();
    }

    #[track_caller]
    pub(crate) fn decrement_submodule(&mut self) {
        self.frame.decrement_submodule();
        crate::types::ww_check(
            0x11,
            1,
            "decrement_submodule",
            u32::from(self.frame.submodule),
        );
        self.sync();
    }

    #[track_caller]
    pub(crate) fn increment_subsubmodule(&mut self) {
        self.frame.increment_subsubmodule();
        crate::types::ww_check(
            0xb0,
            1,
            "increment_subsubmodule",
            u32::from(self.frame.subsubmodule),
        );
        self.sync();
    }

    #[track_caller]
    pub(crate) fn decrement_subsubmodule(&mut self) {
        self.frame.decrement_subsubmodule();
        crate::types::ww_check(
            0xb0,
            1,
            "decrement_subsubmodule",
            u32::from(self.frame.subsubmodule),
        );
        self.sync();
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.frame.set_frame_counter(value);
        self.sync();
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        self.frame.increment_frame_counter();
        self.sync();
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.frame.set_saved_module_for_menu(value);
        self.sync();
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.frame.clear_saved_module_for_menu();
        self.sync();
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.frame.save_main_module_for_menu();
        self.sync();
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.frame.save_submodule_for_menu();
        self.sync();
    }

    pub(crate) fn clear_modal_pause_flag(&mut self) {
        self.frame.clear_modal_pause_flag();
        self.sync();
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        self.frame.set_modal_pause_flag(value);
        self.sync();
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        let value = self.frame.increment_modal_pause_flag();
        self.sync();
        value
    }
}
