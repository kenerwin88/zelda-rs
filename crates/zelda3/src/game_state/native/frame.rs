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

    pub(crate) fn owned_byte(&self, address: usize) -> Option<u8> {
        Some(match address {
            MAIN_MODULE => self.main_module,
            SUBMODULE => self.submodule,
            SUBSUBMODULE => self.subsubmodule,
            FRAME_COUNTER => self.frame_counter,
            SAVED_MODULE_FOR_MENU => self.saved_module_for_menu,
            MODAL_PAUSE_FLAG => self.modal_pause_flag,
            _ => return None,
        })
    }

    pub(crate) fn write_owned_byte(&mut self, address: usize, value: u8) -> bool {
        match address {
            MAIN_MODULE => self.main_module = value,
            SUBMODULE => self.submodule = value,
            SUBSUBMODULE => self.subsubmodule = value,
            FRAME_COUNTER => self.frame_counter = value,
            SAVED_MODULE_FOR_MENU => self.saved_module_for_menu = value,
            MODAL_PAUSE_FLAG => self.modal_pause_flag = value,
            _ => return false,
        }
        true
    }

    pub(crate) fn export_owned_range(&self, image: &mut [u8], lo: usize, hi: usize) {
        for address in lo..=hi { image[address] = self.owned_byte(address).expect("frame range"); }
    }

    pub(crate) fn import_owned_range(&mut self, image: &[u8], lo: usize, hi: usize) {
        for address in lo..=hi { assert!(self.write_owned_byte(address, image[address])); }
    }

    #[cfg(test)]
    pub(crate) fn encode_into_image(&self, image: &mut [u8]) {
        image[MAIN_MODULE] = self.main_module;
        image[SUBMODULE] = self.submodule;
        image[SUBSUBMODULE] = self.subsubmodule;
        image[FRAME_COUNTER] = self.frame_counter;
        image[SAVED_MODULE_FOR_MENU] = self.saved_module_for_menu;
        image[MODAL_PAUSE_FLAG] = self.modal_pause_flag;
    }

    pub(crate) fn main_module_word(&self) -> u16 {
        u16::from(self.main_module) | (u16::from(self.submodule) << 8)
    }

    #[track_caller]
    pub(crate) fn set_main_module(&mut self, value: u8) {
        crate::types::ww_check(0x10, 1, "set_main_module", u32::from(value));
        self.main_module = value;
    }

    #[track_caller]
    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        crate::types::ww_check(0x10, 2, "set_main_module_word", u32::from(value));
        self.main_module = value as u8;
        self.submodule = (value >> 8) as u8;
    }

    #[track_caller]
    pub(crate) fn set_submodule(&mut self, value: u8) {
        crate::types::ww_check(0x11, 1, "set_submodule", u32::from(value));
        self.submodule = value;
    }

    #[track_caller]
    pub(crate) fn increment_submodule(&mut self) {
        self.submodule = self.submodule.wrapping_add(1);
        crate::types::ww_check(0x11, 1, "increment_submodule", u32::from(self.submodule));
    }

    #[track_caller]
    pub(crate) fn decrement_submodule(&mut self) {
        self.submodule = self.submodule.wrapping_sub(1);
        crate::types::ww_check(0x11, 1, "decrement_submodule", u32::from(self.submodule));
    }

    #[track_caller]
    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        crate::types::ww_check(0xb0, 1, "set_subsubmodule", u32::from(value));
        self.subsubmodule = value;
    }

    #[track_caller]
    pub(crate) fn increment_subsubmodule(&mut self) {
        self.subsubmodule = self.subsubmodule.wrapping_add(1);
        crate::types::ww_check(0xb0, 1, "increment_subsubmodule", u32::from(self.subsubmodule));
    }

    #[track_caller]
    pub(crate) fn decrement_subsubmodule(&mut self) {
        self.subsubmodule = self.subsubmodule.wrapping_sub(1);
        crate::types::ww_check(0xb0, 1, "decrement_subsubmodule", u32::from(self.subsubmodule));
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

// State-v2 audit: cartridge fields (including targeted/mode-gated writes)
// are covered by wram_image; recurse only to classify native children.
non_wram_hash! { FrameState {
    wram: [
        main_module, submodule, subsubmodule, frame_counter, saved_module_for_menu,
        modal_pause_flag,
    ],
    children: [],
    metadata: [],
} }
