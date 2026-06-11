use super::*;

pub(crate) struct FrameControlView<'a> {
    ram: &'a [u8],
}

impl<'a> FrameControlView<'a> {
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

pub(crate) struct FrameControlViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> FrameControlViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_main_module(&mut self, value: u8) {
        self.ram[MAIN_MODULE] = value;
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        write_le_u16(self.ram, MAIN_MODULE, value);
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        self.ram[SUBMODULE] = value;
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.ram[SUBSUBMODULE] = value;
    }

    pub(crate) fn increment_submodule(&mut self) {
        self.ram[SUBMODULE] = self.ram[SUBMODULE].wrapping_add(1);
    }

    pub(crate) fn decrement_submodule(&mut self) {
        self.ram[SUBMODULE] = self.ram[SUBMODULE].wrapping_sub(1);
    }

    pub(crate) fn increment_subsubmodule(&mut self) {
        self.ram[SUBSUBMODULE] = self.ram[SUBSUBMODULE].wrapping_add(1);
    }

    pub(crate) fn decrement_subsubmodule(&mut self) {
        self.ram[SUBSUBMODULE] = self.ram[SUBSUBMODULE].wrapping_sub(1);
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.ram[FRAME_COUNTER] = value;
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.ram[SAVED_MODULE_FOR_MENU] = value;
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.set_saved_module_for_menu(0);
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.ram[SAVED_MODULE_FOR_MENU] = self.ram[MAIN_MODULE];
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.ram[SAVED_MODULE_FOR_MENU] = self.ram[SUBMODULE];
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        self.ram[FRAME_COUNTER] = self.ram[FRAME_COUNTER].wrapping_add(1);
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

pub(crate) struct SystemSignalsView<'a> {
    ram: &'a [u8],
}

impl<'a> SystemSignalsView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn apui00_offset() -> usize {
        RAM_APUI00
    }

    pub(crate) fn music_control(&self) -> u8 {
        byte(self.ram, MUSIC_CONTROL)
    }

    pub(crate) fn current_music_control(&self) -> u8 {
        byte(self.ram, CURRENT_MUSIC_CONTROL)
    }

    pub(crate) fn last_music_control(&self) -> u8 {
        byte(self.ram, LAST_MUSIC_CONTROL)
    }

    pub(crate) fn queued_music_control(&self) -> u8 {
        byte(self.ram, QUEUED_MUSIC_CONTROL)
    }

    pub(crate) fn sound_effect_1(&self) -> u8 {
        byte(self.ram, SOUND_EFFECT_1)
    }

    pub(crate) fn sound_effect_2(&self) -> u8 {
        byte(self.ram, SOUND_EFFECT_2)
    }

    pub(crate) fn ambient_sound_effect(&self) -> u8 {
        byte(self.ram, SOUND_EFFECT_AMBIENT)
    }

    pub(crate) fn last_ambient_sound_effect(&self) -> u8 {
        byte(self.ram, SOUND_EFFECT_AMBIENT_LAST)
    }

    pub(crate) fn apui00(&self) -> u8 {
        byte(self.ram, RAM_APUI00)
    }

    pub(crate) fn has_sound_effect_1(&self) -> bool {
        self.sound_effect_1() != 0
    }

    pub(crate) fn has_sound_effect_2(&self) -> bool {
        self.sound_effect_2() != 0
    }

    pub(crate) fn ambient_sound_effect_is_clear(&self) -> bool {
        self.ambient_sound_effect() == 0
    }

    pub(crate) fn should_update_cgram(&self) -> bool {
        byte(self.ram, FLAG_UPDATE_CGRAM_IN_NMI) != 0
    }

    pub(crate) fn should_update_hud(&self) -> bool {
        byte(self.ram, FLAG_UPDATE_HUD_IN_NMI) != 0
    }

    pub(crate) fn game_over_check_flag(&self) -> u8 {
        byte(self.ram, GAME_OVER_CHECK_FLAG)
    }

    pub(crate) fn restart_check_flag(&self) -> u8 {
        byte(self.ram, RESTART_CHECK_FLAG)
    }

    pub(crate) fn bugs_fixed(&self) -> u8 {
        byte(self.ram, RAM_BUGS_FIXED)
    }

    pub(crate) fn death_backup_current_music(&self) -> u8 {
        byte(self.ram, DEATH_BACKUP_CURRENT_MUSIC)
    }

    pub(crate) fn death_backup_ambient_sound(&self) -> u8 {
        byte(self.ram, DEATH_BACKUP_AMBIENT_SOUND)
    }
}

pub(crate) struct SystemSignalsViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SystemSignalsViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_music_control(&mut self, value: u8) {
        self.ram[MUSIC_CONTROL] = value;
    }

    pub(crate) fn set_current_music_control(&mut self, value: u8) {
        self.ram[CURRENT_MUSIC_CONTROL] = value;
    }

    pub(crate) fn set_last_music_control(&mut self, value: u8) {
        self.ram[LAST_MUSIC_CONTROL] = value;
    }

    pub(crate) fn set_queued_music_control(&mut self, value: u8) {
        self.ram[QUEUED_MUSIC_CONTROL] = value;
    }

    pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
        self.ram[SOUND_EFFECT_AMBIENT] = value;
    }

    pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
        self.ram[SOUND_EFFECT_1] = value;
    }

    pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
        self.ram[SOUND_EFFECT_2] = value;
    }

    pub(crate) fn set_apui00(&mut self, value: u8) {
        self.ram[RAM_APUI00] = value;
    }

    pub(crate) fn set_sound_effect_1_word(&mut self, value: u16) {
        write_le_u16(self.ram, SOUND_EFFECT_1, value);
    }

    pub(crate) fn set_ambient_sound_effect_word(&mut self, value: u16) {
        write_le_u16(self.ram, SOUND_EFFECT_AMBIENT, value);
    }

    pub(crate) fn clear_sound_effect_1(&mut self) {
        self.set_sound_effect_1(0);
    }

    pub(crate) fn clear_sound_effect_2(&mut self) {
        self.set_sound_effect_2(0);
    }

    pub(crate) fn clear_ambient_sound_effect(&mut self) {
        self.set_ambient_sound_effect(0);
    }

    pub(crate) fn queue_sound_effect_1_if_empty(&mut self, value: u8) -> bool {
        if self.ram[SOUND_EFFECT_1] == 0 {
            self.ram[SOUND_EFFECT_1] = value;
            true
        } else {
            false
        }
    }

    pub(crate) fn queue_sound_effect_2_if_empty(&mut self, value: u8) -> bool {
        if self.ram[SOUND_EFFECT_2] == 0 {
            self.ram[SOUND_EFFECT_2] = value;
            true
        } else {
            false
        }
    }

    pub(crate) fn increment_hud_update_flag(&mut self) -> u8 {
        self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
        self.ram[FLAG_UPDATE_HUD_IN_NMI]
    }

    pub(crate) fn clear_hud_update_flag(&mut self) {
        self.ram[FLAG_UPDATE_HUD_IN_NMI] = 0;
    }

    pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI]
    }

    pub(crate) fn clear_cgram_update_flag(&mut self) {
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = 0;
    }

    pub(crate) fn save_current_music_as_last(&mut self) {
        self.ram[LAST_MUSIC_CONTROL] = self.ram[CURRENT_MUSIC_CONTROL];
    }

    pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
        self.ram[SOUND_EFFECT_AMBIENT_LAST] = self.ram[SOUND_EFFECT_AMBIENT];
    }

    pub(crate) fn clear_game_over_check_flag(&mut self) {
        self.ram[GAME_OVER_CHECK_FLAG] = 0;
    }

    pub(crate) fn clear_restart_check_flag(&mut self) {
        self.ram[RESTART_CHECK_FLAG] = 0;
    }

    pub(crate) fn set_restart_check_flag(&mut self, value: u8) {
        self.ram[RESTART_CHECK_FLAG] = value;
    }

    pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
        self.ram[RAW_SFX_PAN_VALUE] = value;
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u8) {
        self.ram[GAME_OVER_CHECK_FLAG] = value;
    }

    pub(crate) fn increment_game_over_check_flag(&mut self) {
        self.ram[GAME_OVER_CHECK_FLAG] = self.ram[GAME_OVER_CHECK_FLAG].wrapping_add(1);
    }

    pub(crate) fn set_death_backup_current_music(&mut self, value: u8) {
        self.ram[DEATH_BACKUP_CURRENT_MUSIC] = value;
    }

    pub(crate) fn set_death_backup_ambient_sound(&mut self, value: u8) {
        self.ram[DEATH_BACKUP_AMBIENT_SOUND] = value;
    }
}
