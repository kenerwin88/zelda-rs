use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::write_le_u16;

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
