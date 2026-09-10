use super::ram_byte;
use crate::game_state::constants::*;
use crate::game_state::native::ram_target::RamTarget;

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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(MAIN_MODULE, self.main_module);
        ram.write_byte(SUBMODULE, self.submodule);
        ram.write_byte(SUBSUBMODULE, self.subsubmodule);
        ram.write_byte(FRAME_COUNTER, self.frame_counter);
        ram.write_byte(SAVED_MODULE_FOR_MENU, self.saved_module_for_menu);
        ram.write_byte(MODAL_PAUSE_FLAG, self.modal_pause_flag);
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
    before: Vec<(usize, u8)>,
    frame: &'a mut FrameState,
    ram: &'a mut [u8],
}

impl<'a> NativeFrameStateBridgeMut<'a> {
    pub(crate) fn new(frame: &'a mut FrameState, ram: &'a mut [u8]) -> Self {
        *frame = FrameState::load_from_ram(&*ram);
        let before =
            crate::game_state::native::ram_target::capture(&*ram, |log| frame.write_to_ram(log));
        Self { before, frame, ram }
    }

    fn sync(&mut self) {
        let now = crate::game_state::native::ram_target::capture(&*self.ram, |log| {
            self.frame.write_to_ram(log)
        });
        crate::game_state::native::ram_target::publish_changes(&self.before, &now, self.ram);
        self.before = now;
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

    forward_synced! {
        frame;
        fn set_frame_counter(value: u8);
        fn increment_frame_counter();
        fn set_saved_module_for_menu(value: u8);
        fn clear_saved_module_for_menu();
        fn save_main_module_for_menu();
        fn save_submodule_for_menu();
        fn clear_modal_pause_flag();
        fn set_modal_pause_flag(value: u8);
        fn increment_modal_pause_flag() -> u8;
    }
}
