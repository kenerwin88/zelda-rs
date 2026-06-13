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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SystemSignalsState {
    music_control: u8,
    current_music_control: u8,
    last_music_control: u8,
    queued_music_control: u8,
    sound_effect_1: u8,
    sound_effect_2: u8,
    ambient_sound_effect: u8,
    last_ambient_sound_effect: u8,
    msu_volume: u8,
    apui00: u8,
    raw_sfx_pan_value: u8,
    update_cgram_flag: u8,
    update_hud_flag: u8,
    game_over_check_flag: u8,
    restart_check_flag: u8,
    bugs_fixed: u8,
    death_backup_current_music: u8,
    death_backup_ambient_sound: u8,
}

impl SystemSignalsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            music_control: ram_byte(ram, MUSIC_CONTROL),
            current_music_control: ram_byte(ram, CURRENT_MUSIC_CONTROL),
            last_music_control: ram_byte(ram, LAST_MUSIC_CONTROL),
            queued_music_control: ram_byte(ram, QUEUED_MUSIC_CONTROL),
            sound_effect_1: ram_byte(ram, SOUND_EFFECT_1),
            sound_effect_2: ram_byte(ram, SOUND_EFFECT_2),
            ambient_sound_effect: ram_byte(ram, SOUND_EFFECT_AMBIENT),
            last_ambient_sound_effect: ram_byte(ram, SOUND_EFFECT_AMBIENT_LAST),
            msu_volume: ram_byte(ram, MSU_VOLUME),
            apui00: ram_byte(ram, RAM_APUI00),
            raw_sfx_pan_value: ram_byte(ram, RAW_SFX_PAN_VALUE),
            update_cgram_flag: ram_byte(ram, FLAG_UPDATE_CGRAM_IN_NMI),
            update_hud_flag: ram_byte(ram, FLAG_UPDATE_HUD_IN_NMI),
            game_over_check_flag: ram_byte(ram, GAME_OVER_CHECK_FLAG),
            restart_check_flag: ram_byte(ram, RESTART_CHECK_FLAG),
            bugs_fixed: ram_byte(ram, RAM_BUGS_FIXED),
            death_backup_current_music: ram_byte(ram, DEATH_BACKUP_CURRENT_MUSIC),
            death_backup_ambient_sound: ram_byte(ram, DEATH_BACKUP_AMBIENT_SOUND),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MUSIC_CONTROL] = self.music_control;
        ram[CURRENT_MUSIC_CONTROL] = self.current_music_control;
        ram[LAST_MUSIC_CONTROL] = self.last_music_control;
        ram[QUEUED_MUSIC_CONTROL] = self.queued_music_control;
        ram[SOUND_EFFECT_1] = self.sound_effect_1;
        ram[SOUND_EFFECT_2] = self.sound_effect_2;
        ram[SOUND_EFFECT_AMBIENT] = self.ambient_sound_effect;
        ram[SOUND_EFFECT_AMBIENT_LAST] = self.last_ambient_sound_effect;
        ram[MSU_VOLUME] = self.msu_volume;
        ram[RAM_APUI00] = self.apui00;
        ram[RAW_SFX_PAN_VALUE] = self.raw_sfx_pan_value;
        ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.update_cgram_flag;
        ram[FLAG_UPDATE_HUD_IN_NMI] = self.update_hud_flag;
        ram[GAME_OVER_CHECK_FLAG] = self.game_over_check_flag;
        ram[RESTART_CHECK_FLAG] = self.restart_check_flag;
        ram[RAM_BUGS_FIXED] = self.bugs_fixed;
        ram[DEATH_BACKUP_CURRENT_MUSIC] = self.death_backup_current_music;
        ram[DEATH_BACKUP_AMBIENT_SOUND] = self.death_backup_ambient_sound;
    }

    pub(crate) fn apui00_offset() -> usize {
        RAM_APUI00
    }

    pub(crate) fn music_control(&self) -> u8 {
        self.music_control
    }

    pub(crate) fn current_music_control(&self) -> u8 {
        self.current_music_control
    }

    pub(crate) fn last_music_control(&self) -> u8 {
        self.last_music_control
    }

    pub(crate) fn queued_music_control(&self) -> u8 {
        self.queued_music_control
    }

    pub(crate) fn sound_effect_1(&self) -> u8 {
        self.sound_effect_1
    }

    pub(crate) fn sound_effect_2(&self) -> u8 {
        self.sound_effect_2
    }

    pub(crate) fn ambient_sound_effect(&self) -> u8 {
        self.ambient_sound_effect
    }

    pub(crate) fn last_ambient_sound_effect(&self) -> u8 {
        self.last_ambient_sound_effect
    }

    pub(crate) fn msu_volume(&self) -> u8 {
        self.msu_volume
    }

    pub(crate) fn apui00(&self) -> u8 {
        self.apui00
    }

    pub(crate) fn has_sound_effect_1(&self) -> bool {
        self.sound_effect_1 != 0
    }

    pub(crate) fn has_sound_effect_2(&self) -> bool {
        self.sound_effect_2 != 0
    }

    pub(crate) fn ambient_sound_effect_is_clear(&self) -> bool {
        self.ambient_sound_effect == 0
    }

    pub(crate) fn raw_sfx_pan_value(&self) -> u8 {
        self.raw_sfx_pan_value
    }

    pub(crate) fn should_update_cgram(&self) -> bool {
        self.update_cgram_flag != 0
    }

    pub(crate) fn should_update_hud(&self) -> bool {
        self.update_hud_flag != 0
    }

    pub(crate) fn game_over_check_flag(&self) -> u8 {
        self.game_over_check_flag
    }

    pub(crate) fn restart_check_flag(&self) -> u8 {
        self.restart_check_flag
    }

    pub(crate) fn bugs_fixed(&self) -> u8 {
        self.bugs_fixed
    }

    pub(crate) fn death_backup_current_music(&self) -> u8 {
        self.death_backup_current_music
    }

    pub(crate) fn death_backup_ambient_sound(&self) -> u8 {
        self.death_backup_ambient_sound
    }
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

pub(crate) struct NativeSystemSignalsBridgeMut<'a> {
    system_signals: &'a mut SystemSignalsState,
    ram: &'a mut [u8],
}

impl<'a> NativeSystemSignalsBridgeMut<'a> {
    pub(crate) fn new(system_signals: &'a mut SystemSignalsState, ram: &'a mut [u8]) -> Self {
        *system_signals = SystemSignalsState::load_from_ram(ram);
        Self {
            system_signals,
            ram,
        }
    }

    fn sync(&mut self) {
        self.system_signals.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.system_signals,
            SystemSignalsState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_music_control(&mut self, value: u8) {
        self.system_signals.music_control = value;
        self.sync();
    }

    pub(crate) fn set_current_music_control(&mut self, value: u8) {
        self.system_signals.current_music_control = value;
        self.sync();
    }

    pub(crate) fn set_last_music_control(&mut self, value: u8) {
        self.system_signals.last_music_control = value;
        self.sync();
    }

    pub(crate) fn set_queued_music_control(&mut self, value: u8) {
        self.system_signals.queued_music_control = value;
        self.sync();
    }

    pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
        self.system_signals.ambient_sound_effect = value;
        self.sync();
    }

    pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
        self.system_signals.sound_effect_1 = value;
        self.sync();
    }

    pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
        self.system_signals.sound_effect_2 = value;
        self.sync();
    }

    pub(crate) fn set_apui00(&mut self, value: u8) {
        self.system_signals.apui00 = value;
        self.sync();
    }

    pub(crate) fn set_msu_volume(&mut self, value: u8) {
        self.system_signals.msu_volume = value;
        self.sync();
    }

    pub(crate) fn set_sound_effect_1_word(&mut self, value: u16) {
        self.system_signals.sound_effect_1 = value as u8;
        self.system_signals.sound_effect_2 = (value >> 8) as u8;
        self.sync();
    }

    pub(crate) fn set_ambient_sound_effect_word(&mut self, value: u16) {
        self.system_signals.ambient_sound_effect = value as u8;
        self.system_signals.sound_effect_1 = (value >> 8) as u8;
        self.sync();
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
        if self.system_signals.sound_effect_1 == 0 {
            self.system_signals.sound_effect_1 = value;
            self.sync();
            true
        } else {
            false
        }
    }

    pub(crate) fn queue_sound_effect_2_if_empty(&mut self, value: u8) -> bool {
        if self.system_signals.sound_effect_2 == 0 {
            self.system_signals.sound_effect_2 = value;
            self.sync();
            true
        } else {
            false
        }
    }

    pub(crate) fn increment_hud_update_flag(&mut self) -> u8 {
        self.system_signals.update_hud_flag = self.system_signals.update_hud_flag.wrapping_add(1);
        self.sync();
        self.system_signals.update_hud_flag
    }

    pub(crate) fn clear_hud_update_flag(&mut self) {
        self.system_signals.update_hud_flag = 0;
        self.sync();
    }

    pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
        self.system_signals.update_cgram_flag =
            self.system_signals.update_cgram_flag.wrapping_add(1);
        self.sync();
        self.system_signals.update_cgram_flag
    }

    pub(crate) fn clear_cgram_update_flag(&mut self) {
        self.system_signals.update_cgram_flag = 0;
        self.sync();
    }

    pub(crate) fn set_bugs_fixed(&mut self, value: u8) {
        self.system_signals.bugs_fixed = value;
        self.sync();
    }

    pub(crate) fn save_current_music_as_last(&mut self) {
        self.system_signals.last_music_control = self.system_signals.current_music_control;
        self.sync();
    }

    pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
        self.system_signals.last_ambient_sound_effect = self.system_signals.ambient_sound_effect;
        self.sync();
    }

    pub(crate) fn clear_game_over_check_flag(&mut self) {
        self.system_signals.game_over_check_flag = 0;
        self.sync();
    }

    pub(crate) fn clear_restart_check_flag(&mut self) {
        self.system_signals.restart_check_flag = 0;
        self.sync();
    }

    pub(crate) fn set_restart_check_flag(&mut self, value: u8) {
        self.system_signals.restart_check_flag = value;
        self.sync();
    }

    pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
        self.system_signals.raw_sfx_pan_value = value;
        self.sync();
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u8) {
        self.system_signals.game_over_check_flag = value;
        self.sync();
    }

    pub(crate) fn increment_game_over_check_flag(&mut self) {
        self.system_signals.game_over_check_flag =
            self.system_signals.game_over_check_flag.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn set_death_backup_current_music(&mut self, value: u8) {
        self.system_signals.death_backup_current_music = value;
        self.sync();
    }

    pub(crate) fn set_death_backup_ambient_sound(&mut self, value: u8) {
        self.system_signals.death_backup_ambient_sound = value;
        self.sync();
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
        self.frame.main_module = value;
        self.sync();
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        self.frame.main_module = value as u8;
        self.frame.submodule = (value >> 8) as u8;
        self.sync();
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        self.frame.submodule = value;
        self.sync();
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.frame.subsubmodule = value;
        self.sync();
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
        self.sync();
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        let value = self.frame.frame_counter.wrapping_add(1);
        self.set_frame_counter(value);
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.frame.saved_module_for_menu = value;
        self.sync();
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
        self.sync();
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        let value = self.frame.modal_pause_flag.wrapping_add(1);
        self.set_modal_pause_flag(value);
        value
    }
}
