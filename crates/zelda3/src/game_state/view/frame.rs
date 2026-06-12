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

    pub(crate) fn main_module_word(&self) -> u16 {
        word(self.ram, MAIN_MODULE)
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

    pub(crate) fn nmi_thread_active(&self) -> bool {
        byte(self.ram, NMI_THREAD_ACTIVE) != 0
    }

    pub(crate) fn selected_run_thread(&self) -> u8 {
        if self.nmi_thread_active() && word(self.ram, POLY_THREAD_STACK) != 0x1f31 {
            RUN_POLY_THREAD
        } else {
            RUN_MAIN_THREAD
        }
    }
}
