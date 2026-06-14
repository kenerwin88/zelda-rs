use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const ATTRACT_LOW_WORK_AREA_START: usize = 0x20;
const ATTRACT_LOW_WORK_AREA_LEN: usize = 0x51;
const MSU_RESUME_INFO_ALT: usize = 0x1db20;
const MSU_RESUME_INFO: usize = 0x1db60;
const STARTUP_LOW_MEMORY_START: usize = 0;
const STARTUP_LOW_MEMORY_LEN: usize = 0x2000;
const POLY_THREAD_WORK_AREA_START: usize = 0x1f00;
const POLY_THREAD_WORK_AREA_LEN: usize = 0x100;
const POLY_THREAD_BOOTSTRAP_BYTES_OFFSET: usize = 0x1f32;
const POLY_THREAD_BOOTSTRAP_BYTES: [u8; 13] = [9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9];
const INTRO_CLEAR_BLOCK_BASE: usize = 0x2000;
const INTRO_CLEAR_BLOCK_STRIDE: usize = 0x2000;

#[derive(Clone, Copy)]
pub(crate) enum MsuResumeSlot {
    Primary,
    Alternate,
}

impl MsuResumeSlot {
    fn offset(self) -> usize {
        match self {
            Self::Primary => MSU_RESUME_INFO,
            Self::Alternate => MSU_RESUME_INFO_ALT,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MsuResumeInfoState {
    pub(crate) tag: u32,
    pub(crate) offset: u32,
    pub(crate) samples_until_repeat: u32,
    pub(crate) range_cur: u16,
    pub(crate) range_repeat: u16,
    pub(crate) initial_packet_bytes: u64,
    pub(crate) orig_track: u8,
    pub(crate) actual_track: u8,
}

impl MsuResumeInfoState {
    fn load_from_ram_slot(ram: &[u8], slot: MsuResumeSlot) -> Self {
        let offset = slot.offset();
        if offset + 28 > ram.len() {
            return Self::default();
        }
        Self {
            tag: read_le_u32(ram, offset).unwrap_or(0),
            offset: read_le_u32(ram, offset + 4).unwrap_or(0),
            samples_until_repeat: read_le_u32(ram, offset + 8).unwrap_or(0),
            range_cur: read_le_u16(ram, offset + 12),
            range_repeat: read_le_u16(ram, offset + 14),
            initial_packet_bytes: read_le_u64(ram, offset + 16).unwrap_or(0),
            orig_track: ram[offset + 24],
            actual_track: ram[offset + 25],
        }
    }

    fn write_to_ram_slot(self, ram: &mut [u8], slot: MsuResumeSlot) {
        let offset = slot.offset();
        if offset + 28 > ram.len() {
            return;
        }
        write_le_u32(ram, offset, self.tag);
        write_le_u32(ram, offset + 4, self.offset);
        write_le_u32(ram, offset + 8, self.samples_until_repeat);
        write_le_u16(ram, offset + 12, self.range_cur);
        write_le_u16(ram, offset + 14, self.range_repeat);
        write_le_u64(ram, offset + 16, self.initial_packet_bytes);
        ram[offset + 24] = self.orig_track;
        ram[offset + 25] = self.actual_track;
    }
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
    msu_resume_primary: MsuResumeInfoState,
    msu_resume_alternate: MsuResumeInfoState,
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
            msu_resume_primary: MsuResumeInfoState::load_from_ram_slot(ram, MsuResumeSlot::Primary),
            msu_resume_alternate: MsuResumeInfoState::load_from_ram_slot(
                ram,
                MsuResumeSlot::Alternate,
            ),
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
        self.msu_resume_primary
            .write_to_ram_slot(ram, MsuResumeSlot::Primary);
        self.msu_resume_alternate
            .write_to_ram_slot(ram, MsuResumeSlot::Alternate);
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

    pub(crate) fn msu_resume_info(&self, slot: MsuResumeSlot) -> MsuResumeInfoState {
        match slot {
            MsuResumeSlot::Primary => self.msu_resume_primary,
            MsuResumeSlot::Alternate => self.msu_resume_alternate,
        }
    }

    pub(crate) fn set_music_control(&mut self, value: u8) {
        self.music_control = value;
    }

    pub(crate) fn set_current_music_control(&mut self, value: u8) {
        self.current_music_control = value;
    }

    pub(crate) fn set_last_music_control(&mut self, value: u8) {
        self.last_music_control = value;
    }

    pub(crate) fn set_queued_music_control(&mut self, value: u8) {
        self.queued_music_control = value;
    }

    pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
        self.sound_effect_1 = value;
    }

    pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
        self.sound_effect_2 = value;
    }

    pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
        self.ambient_sound_effect = value;
    }

    pub(crate) fn set_apui00(&mut self, value: u8) {
        self.apui00 = value;
    }

    pub(crate) fn set_msu_volume(&mut self, value: u8) {
        self.msu_volume = value;
    }

    pub(crate) fn set_sound_effect_1_word(&mut self, value: u16) {
        self.sound_effect_1 = value as u8;
        self.sound_effect_2 = (value >> 8) as u8;
    }

    pub(crate) fn set_ambient_sound_effect_word(&mut self, value: u16) {
        self.ambient_sound_effect = value as u8;
        self.sound_effect_1 = (value >> 8) as u8;
    }

    pub(crate) fn clear_sound_effect_1(&mut self) {
        self.sound_effect_1 = 0;
    }

    pub(crate) fn clear_sound_effect_2(&mut self) {
        self.sound_effect_2 = 0;
    }

    pub(crate) fn clear_ambient_sound_effect(&mut self) {
        self.ambient_sound_effect = 0;
    }

    pub(crate) fn queue_sound_effect_1_if_empty(&mut self, value: u8) -> bool {
        if self.sound_effect_1 == 0 {
            self.sound_effect_1 = value;
            true
        } else {
            false
        }
    }

    pub(crate) fn queue_sound_effect_2_if_empty(&mut self, value: u8) -> bool {
        if self.sound_effect_2 == 0 {
            self.sound_effect_2 = value;
            true
        } else {
            false
        }
    }

    pub(crate) fn increment_hud_update_flag(&mut self) -> u8 {
        self.update_hud_flag = self.update_hud_flag.wrapping_add(1);
        self.update_hud_flag
    }

    pub(crate) fn clear_hud_update_flag(&mut self) {
        self.update_hud_flag = 0;
    }

    pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
        self.update_cgram_flag = self.update_cgram_flag.wrapping_add(1);
        self.update_cgram_flag
    }

    pub(crate) fn clear_cgram_update_flag(&mut self) {
        self.update_cgram_flag = 0;
    }

    pub(crate) fn set_bugs_fixed(&mut self, value: u8) {
        self.bugs_fixed = value;
    }

    pub(crate) fn save_current_music_as_last(&mut self) {
        self.last_music_control = self.current_music_control;
    }

    pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
        self.last_ambient_sound_effect = self.ambient_sound_effect;
    }

    pub(crate) fn clear_game_over_check_flag(&mut self) {
        self.game_over_check_flag = 0;
    }

    pub(crate) fn clear_restart_check_flag(&mut self) {
        self.restart_check_flag = 0;
    }

    pub(crate) fn set_restart_check_flag(&mut self, value: u8) {
        self.restart_check_flag = value;
    }

    pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
        self.raw_sfx_pan_value = value;
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u8) {
        self.game_over_check_flag = value;
    }

    pub(crate) fn increment_game_over_check_flag(&mut self) {
        self.game_over_check_flag = self.game_over_check_flag.wrapping_add(1);
    }

    pub(crate) fn set_death_backup_current_music(&mut self, value: u8) {
        self.death_backup_current_music = value;
    }

    pub(crate) fn set_death_backup_ambient_sound(&mut self, value: u8) {
        self.death_backup_ambient_sound = value;
    }

    pub(crate) fn set_msu_resume_info(&mut self, slot: MsuResumeSlot, info: MsuResumeInfoState) {
        match slot {
            MsuResumeSlot::Primary => self.msu_resume_primary = info,
            MsuResumeSlot::Alternate => self.msu_resume_alternate = info,
        }
    }
}

pub(crate) struct SystemWorkArea;

impl SystemWorkArea {
    pub(crate) fn clear_attract_low_work_area(ram: &mut [u8]) {
        ram[ATTRACT_LOW_WORK_AREA_START..ATTRACT_LOW_WORK_AREA_START + ATTRACT_LOW_WORK_AREA_LEN]
            .fill(0);
    }

    pub(crate) fn clear_startup_low_memory(ram: &mut [u8]) {
        ram[STARTUP_LOW_MEMORY_START..STARTUP_LOW_MEMORY_START + STARTUP_LOW_MEMORY_LEN].fill(0);
    }

    pub(crate) fn clear_poly_thread_work_area(ram: &mut [u8]) {
        ram[POLY_THREAD_WORK_AREA_START..POLY_THREAD_WORK_AREA_START + POLY_THREAD_WORK_AREA_LEN]
            .fill(0);
    }

    pub(crate) fn write_poly_thread_bootstrap_bytes(ram: &mut [u8]) {
        ram[POLY_THREAD_BOOTSTRAP_BYTES_OFFSET
            ..POLY_THREAD_BOOTSTRAP_BYTES_OFFSET + POLY_THREAD_BOOTSTRAP_BYTES.len()]
            .copy_from_slice(&POLY_THREAD_BOOTSTRAP_BYTES);
    }

    pub(crate) fn clear_intro_wram_block_columns(
        ram: &mut [u8],
        start_offset: u16,
        stop_offset: u16,
    ) -> u16 {
        let mut offset = start_offset;
        loop {
            for block in 0..15 {
                let base =
                    INTRO_CLEAR_BLOCK_BASE + offset as usize + block * INTRO_CLEAR_BLOCK_STRIDE;
                ram[base] = 0;
                ram[base + 1] = 0;
            }
            offset = offset.wrapping_sub(2);
            if offset == stop_offset {
                return offset;
            }
        }
    }
}

pub(crate) struct NativeSystemWorkAreaBridgeMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> NativeSystemWorkAreaBridgeMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_attract_low_work_area(&mut self) {
        SystemWorkArea::clear_attract_low_work_area(self.ram);
    }

    pub(crate) fn clear_startup_low_memory(&mut self) {
        SystemWorkArea::clear_startup_low_memory(self.ram);
    }

    pub(crate) fn clear_poly_thread_work_area(&mut self) {
        SystemWorkArea::clear_poly_thread_work_area(self.ram);
    }

    pub(crate) fn write_poly_thread_bootstrap_bytes(&mut self) {
        SystemWorkArea::write_poly_thread_bootstrap_bytes(self.ram);
    }

    pub(crate) fn clear_intro_wram_block_columns(
        &mut self,
        start_offset: u16,
        stop_offset: u16,
    ) -> u16 {
        SystemWorkArea::clear_intro_wram_block_columns(self.ram, start_offset, stop_offset)
    }
}

pub(crate) struct NativeSystemSignalsBridgeMut<'a> {
    system_signals: &'a mut SystemSignalsState,
    ram: &'a mut [u8],
}

impl<'a> NativeSystemSignalsBridgeMut<'a> {
    pub(crate) fn new(system_signals: &'a mut SystemSignalsState, ram: &'a mut [u8]) -> Self {
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
        self.system_signals.set_music_control(value);
        self.sync();
    }

    pub(crate) fn set_current_music_control(&mut self, value: u8) {
        self.system_signals.set_current_music_control(value);
        self.sync();
    }

    pub(crate) fn set_last_music_control(&mut self, value: u8) {
        self.system_signals.set_last_music_control(value);
        self.sync();
    }

    pub(crate) fn set_queued_music_control(&mut self, value: u8) {
        self.system_signals.set_queued_music_control(value);
        self.sync();
    }

    pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
        self.system_signals.set_ambient_sound_effect(value);
        self.sync();
    }

    pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
        self.system_signals.set_sound_effect_1(value);
        self.sync();
    }

    pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
        self.system_signals.set_sound_effect_2(value);
        self.sync();
    }

    pub(crate) fn set_apui00(&mut self, value: u8) {
        self.system_signals.set_apui00(value);
        self.sync();
    }

    pub(crate) fn set_msu_volume(&mut self, value: u8) {
        self.system_signals.set_msu_volume(value);
        self.sync();
    }

    pub(crate) fn set_msu_resume_info(&mut self, slot: MsuResumeSlot, info: MsuResumeInfoState) {
        self.system_signals.set_msu_resume_info(slot, info);
        self.sync();
    }

    pub(crate) fn set_sound_effect_1_word(&mut self, value: u16) {
        self.system_signals.set_sound_effect_1_word(value);
        self.sync();
    }

    pub(crate) fn set_ambient_sound_effect_word(&mut self, value: u16) {
        self.system_signals.set_ambient_sound_effect_word(value);
        self.sync();
    }

    pub(crate) fn clear_sound_effect_1(&mut self) {
        self.system_signals.clear_sound_effect_1();
        self.sync();
    }

    pub(crate) fn clear_sound_effect_2(&mut self) {
        self.system_signals.clear_sound_effect_2();
        self.sync();
    }

    pub(crate) fn clear_ambient_sound_effect(&mut self) {
        self.system_signals.clear_ambient_sound_effect();
        self.sync();
    }

    pub(crate) fn queue_sound_effect_1_if_empty(&mut self, value: u8) -> bool {
        if self.system_signals.queue_sound_effect_1_if_empty(value) {
            self.sync();
            true
        } else {
            false
        }
    }

    pub(crate) fn queue_sound_effect_2_if_empty(&mut self, value: u8) -> bool {
        if self.system_signals.queue_sound_effect_2_if_empty(value) {
            self.sync();
            true
        } else {
            false
        }
    }

    pub(crate) fn increment_hud_update_flag(&mut self) -> u8 {
        let value = self.system_signals.increment_hud_update_flag();
        self.sync();
        value
    }

    pub(crate) fn clear_hud_update_flag(&mut self) {
        self.system_signals.clear_hud_update_flag();
        self.sync();
    }

    pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
        let value = self.system_signals.increment_cgram_update_flag();
        self.sync();
        value
    }

    pub(crate) fn clear_cgram_update_flag(&mut self) {
        self.system_signals.clear_cgram_update_flag();
        self.sync();
    }

    pub(crate) fn set_bugs_fixed(&mut self, value: u8) {
        self.system_signals.set_bugs_fixed(value);
        self.sync();
    }

    pub(crate) fn save_current_music_as_last(&mut self) {
        self.system_signals.save_current_music_as_last();
        self.sync();
    }

    pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
        self.system_signals.save_ambient_sound_effect_as_last();
        self.sync();
    }

    pub(crate) fn clear_game_over_check_flag(&mut self) {
        self.system_signals.clear_game_over_check_flag();
        self.sync();
    }

    pub(crate) fn clear_restart_check_flag(&mut self) {
        self.system_signals.clear_restart_check_flag();
        self.sync();
    }

    pub(crate) fn set_restart_check_flag(&mut self, value: u8) {
        self.system_signals.set_restart_check_flag(value);
        self.sync();
    }

    pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
        self.system_signals.set_raw_sfx_pan_value(value);
        self.sync();
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u8) {
        self.system_signals.set_game_over_check_flag(value);
        self.sync();
    }

    pub(crate) fn increment_game_over_check_flag(&mut self) {
        self.system_signals.increment_game_over_check_flag();
        self.sync();
    }

    pub(crate) fn set_death_backup_current_music(&mut self, value: u8) {
        self.system_signals.set_death_backup_current_music(value);
        self.sync();
    }

    pub(crate) fn set_death_backup_ambient_sound(&mut self, value: u8) {
        self.system_signals.set_death_backup_ambient_sound(value);
        self.sync();
    }
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_le_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let slice = bytes.get(offset..offset + 8)?;
    Some(u64::from_le_bytes([
        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
    ]))
}

fn write_le_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_le_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
