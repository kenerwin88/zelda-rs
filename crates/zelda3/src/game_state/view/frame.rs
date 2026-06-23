use super::*;

pub(crate) struct RamFrameStateView<'a> {
    ram: &'a [u8],
}

impl<'a> RamFrameStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn main_module(&self) -> u8 {
        byte(self.ram, MAIN_MODULE)
    }

    pub(crate) fn submodule(&self) -> u8 {
        byte(self.ram, SUBMODULE)
    }

    pub(crate) fn subsubmodule(&self) -> u8 {
        byte(self.ram, SUBSUBMODULE)
    }

    pub(crate) fn frame_counter(&self) -> u8 {
        byte(self.ram, FRAME_COUNTER)
    }

    pub(crate) fn saved_module_for_menu(&self) -> u8 {
        byte(self.ram, SAVED_MODULE_FOR_MENU)
    }

    pub(crate) fn modal_pause_flag(&self) -> u8 {
        byte(self.ram, MODAL_PAUSE_FLAG)
    }

    pub(crate) fn main_module_word(&self) -> u16 {
        u16::from(self.main_module()) | (u16::from(self.submodule()) << 8)
    }
}

pub(crate) struct RamFrameStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> RamFrameStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_main_module(&mut self, value: u8) {
        self.ram[MAIN_MODULE] = value;
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        self.ram[SUBMODULE] = value;
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.ram[SUBSUBMODULE] = value;
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.ram[FRAME_COUNTER] = value;
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.ram[SAVED_MODULE_FOR_MENU] = value;
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        self.ram[MODAL_PAUSE_FLAG] = value;
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod tests;
