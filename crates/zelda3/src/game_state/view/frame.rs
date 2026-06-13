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
mod tests {
    use super::*;
    use crate::game_state::FrameState;
    use snes::WRAM_SIZE;

    #[test]
    fn ram_frame_state_view_matches_native_frame_projection() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 7;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 9;
        ram[FRAME_COUNTER] = 0x42;
        ram[SAVED_MODULE_FOR_MENU] = 5;
        ram[MODAL_PAUSE_FLAG] = 1;

        let native = FrameState::load_from_ram(&ram);
        let view = RamFrameStateView::new(&ram);

        assert_eq!(view.main_module(), native.main_module);
        assert_eq!(view.submodule(), native.submodule);
        assert_eq!(view.subsubmodule(), native.subsubmodule);
        assert_eq!(view.frame_counter(), native.frame_counter);
        assert_eq!(view.saved_module_for_menu(), native.saved_module_for_menu);
        assert_eq!(view.modal_pause_flag(), native.modal_pause_flag);
        assert_eq!(view.main_module_word(), native.main_module_word());
    }

    #[test]
    fn ram_frame_state_view_mut_writes_byte_backed_projection() {
        let mut ram = vec![0; WRAM_SIZE];

        {
            let mut view = RamFrameStateViewMut::new(&mut ram);
            view.set_main_module(14);
            view.set_submodule(3);
            view.set_subsubmodule(1);
            view.set_frame_counter(0x80);
            view.set_saved_module_for_menu(7);
            view.set_modal_pause_flag(2);
        }

        assert_eq!(
            FrameState::load_from_ram(&ram),
            FrameState {
                main_module: 14,
                submodule: 3,
                subsubmodule: 1,
                frame_counter: 0x80,
                saved_module_for_menu: 7,
                modal_pause_flag: 2,
            }
        );
    }
}
