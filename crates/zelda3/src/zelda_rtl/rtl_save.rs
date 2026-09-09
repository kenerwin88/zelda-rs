//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (save).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

#[cfg(test)]
#[path = "../save_progress_tests.rs"]
mod save_progress_tests;

impl ZeldaState {
    pub(crate) fn saved_room_flags(&self, room: usize) -> u16 {
        if room < crate::game_state::DUNGEON_ROOM_COUNT {
            self.game_state
                .inventory
                .save_progress
                .dungeon_info_word(room)
        } else {
            crate::game_state::save_format::LiveSave::from_wram(&self.ram).indexed_word(room)
        }
    }

    pub(crate) fn set_saved_room_flags(&mut self, room: usize, flags: u16) {
        if room < crate::game_state::DUNGEON_ROOM_COUNT {
            self.save_progress_mut().set_dungeon_info_word(room, flags);
        } else {
            // Extended indices are source aliases into other save domains.
            // Preserve the single-word write and their existing import timing.
            crate::game_state::save_format::write_indexed_word(&mut self.ram, room, flags);
            // A word can also straddle named progress fields or counters.
            // Import at this explicit cross-domain transfer boundary.
            self.game_state.inventory.save_progress =
                crate::game_state::SaveProgressState::load_from_ram(&self.ram);
        }
    }

    pub(crate) fn add_saved_room_flags(&mut self, room: usize, flags: u16) -> u16 {
        let value = self.saved_room_flags(room) | flags;
        self.set_saved_room_flags(room, value);
        value
    }

    // Whole-save transfers are explicit compatibility boundaries. Refresh the
    // progress owner here; other owners retain their existing frame/import
    // schedule, including the source-ordered writes following CopySaveToWRAM.
    pub(crate) fn clear_live_save(&mut self) {
        crate::game_state::save_format::clear_live_save(&mut self.ram);
        self.game_state.inventory.save_progress =
            crate::game_state::SaveProgressState::load_from_ram(&self.ram);
    }

    pub(crate) fn replace_live_save(&mut self, source: &[u8]) {
        crate::game_state::save_format::replace_live_save(&mut self.ram, source);
        self.game_state.inventory.save_progress =
            crate::game_state::SaveProgressState::load_from_ram(&self.ram);
    }

    pub(crate) fn selected_save_slot_x2(&self) -> u16 {
        read_le_u16(&self.sram, SELECTED_SAVE_SLOT_X2)
    }

    pub(crate) fn selected_save_slot_byte(&self) -> u8 {
        self.selected_save_slot_x2() as u8
    }

    pub(crate) fn selected_save_slot_index(&self) -> usize {
        ((self.selected_save_slot_x2() >> 1).wrapping_sub(1)) as usize
    }

    pub(crate) fn selected_save_slot_offset(&self) -> usize {
        self.selected_save_slot_index() * 0x500
    }

    pub(crate) fn selected_save_slot_source_offset(&self) -> u16 {
        ((u16::from(self.selected_save_slot_byte()) >> 1).wrapping_sub(1)).wrapping_mul(0x500)
    }

    pub(crate) fn set_selected_save_slot_x2(&mut self, value: u16) {
        write_le_u16(&mut self.sram, SELECTED_SAVE_SLOT_X2, value);
    }

    pub(crate) fn set_selected_save_slot_from_cursor(&mut self, cursor: u8) {
        self.set_selected_save_slot_x2(u16::from(cursor) * 2 + 2);
    }

    pub(crate) fn clear_selected_save_slot(&mut self) {
        self.set_selected_save_slot_x2(0);
    }

    pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
        self.system_signals_mut()
            .save_ambient_sound_effect_as_last();
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.frame_state_mut().set_saved_module_for_menu(value);
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.frame_state_mut().clear_saved_module_for_menu();
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.frame_state_mut().save_main_module_for_menu();
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.frame_state_mut().save_submodule_for_menu();
    }

    pub(crate) fn save_spexit_area_index(&mut self) {
        self.world_region_mut().save_spexit_area_index();
    }

    pub(crate) fn save_exit_area_index(&mut self) {
        self.world_region_mut().save_exit_area_index();
    }

    pub(crate) fn save_spexit_tm_copy(&mut self) {
        let layer_masks = self.game_state.display.layer_masks_word();
        self.world_transient_mut().save_spexit_tm_copy(layer_masks);
    }

    pub(crate) fn save_exit_tm_copy(&mut self) {
        let layer_masks = self.game_state.display.layer_masks_word();
        self.world_transient_mut().save_exit_tm_copy(layer_masks);
    }

    pub(crate) fn save_previous_screen_transition_direction_bits(&mut self) {
        self.overworld_transition_mut()
            .save_previous_direction_bits();
    }

    pub(crate) fn set_select_file_choice(&mut self, index: usize, value: u8) {
        self.select_file_menu_mut().set_choice(index, value);
    }

    pub(crate) fn set_select_file_cursor(&mut self, value: u8) {
        self.select_file_menu_mut().set_cursor(value);
    }

    pub(crate) fn clear_select_file_cursor(&mut self) {
        self.select_file_menu_mut().clear_cursor();
    }

    pub(crate) fn clear_select_file_transition_scratch(&mut self) {
        self.select_file_menu_mut().clear_transition_scratch();
    }

    pub(crate) fn increment_select_file_cursor(&mut self) -> u8 {
        self.select_file_menu_mut().increment_cursor()
    }

    pub(crate) fn decrement_select_file_cursor(&mut self) -> u8 {
        self.select_file_menu_mut().decrement_cursor()
    }

    pub(crate) fn clear_select_file_remembered_cursor(&mut self) {
        self.select_file_menu_mut().clear_remembered_cursor();
    }

    pub(crate) fn remember_select_file_cursor(&mut self) {
        self.select_file_menu_mut().remember_current_cursor();
    }

    pub(crate) fn set_select_file_target_word(&mut self, value: u16) {
        self.select_file_menu_mut().set_target_word(value);
    }

    pub(crate) fn set_select_file_copy_source_slot(&mut self, slot: u8) {
        self.select_file_menu_mut().set_copy_source_slot(slot);
    }

    pub(crate) fn clear_select_file_name_entry_state(&mut self) {
        self.select_file_menu_mut().clear_name_entry_state();
    }

    pub(crate) fn set_select_file_name_column(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_column(value);
    }

    pub(crate) fn set_select_file_name_cursor_y(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_cursor_y(value);
    }

    pub(crate) fn step_select_file_name_cursor_y_toward(&mut self, target_y: u8) -> bool {
        self.select_file_menu_mut()
            .step_name_cursor_y_toward(target_y)
    }

    pub(crate) fn move_select_file_name_slot_left_wrapped(&mut self) -> u8 {
        self.select_file_menu_mut().move_name_slot_left_wrapped()
    }

    pub(crate) fn move_select_file_name_slot_right_wrapped(&mut self) -> u8 {
        self.select_file_menu_mut().move_name_slot_right_wrapped()
    }

    pub(crate) fn set_select_file_name_row(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_row(value);
    }

    pub(crate) fn clear_select_file_save_slot_flag(&mut self, slot: usize) {
        self.select_file_menu_mut().clear_save_slot_flag(slot);
    }

    pub(crate) fn clear_select_file_save_slot_flags(&mut self) {
        self.select_file_menu_mut().clear_save_slot_flags();
    }

    #[track_caller]
    pub(crate) fn assert_native_save_progress_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.inventory.save_progress,
            crate::game_state::SaveProgressState::load_from_ram(&self.ram),
            "native save-progress state diverged from compatibility RAM",
        );
    }

    pub fn save_func(ctx_in: &mut ByteArray, data: &mut [u8]) {
        ByteArray_AppendData(ctx_in, data);
    }

    pub(super) fn save_load_call(func: &mut SaveLoadFunc<'_, '_>, data: &mut [u8]) {
        match func {
            SaveLoadFunc::Save(ctx) => Self::save_func(ctx, data),
            SaveLoadFunc::Load(ctx) => Self::load_func(ctx, data),
        }
    }

    pub(super) fn internal_save_load(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        let mut junk = [0u8; 58];
        Self::save_load_call(func, &mut junk[..27]);

        let mut apu_ram = if matches!(func, SaveLoadFunc::Save(_)) {
            self.save_audio_apu_ram_c_saveload().to_vec()
        } else {
            vec![0; APU_RAM_SAVELOAD_SIZE]
        };
        Self::save_load_call(func, &mut apu_ram);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.load_audio_apu_ram_c_saveload(&apu_ram);
        }

        let mut junk40 = [0u8; 40];
        Self::save_load_call(func, &mut junk40);

        let mut dsp = if matches!(func, SaveLoadFunc::Save(_)) {
            self.save_audio_dsp_c_saveload()
        } else {
            vec![0; DSP_SAVELOAD_SIZE]
        };
        Self::save_load_call(func, &mut dsp);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.load_audio_dsp_c_saveload(&dsp)
                .expect("invalid DSP saveload block");
        }

        let mut junk15 = [0u8; 15];
        Self::save_load_call(func, &mut junk15);

        let mut dma_slot = if matches!(func, SaveLoadFunc::Save(_)) {
            self.dma.save_c_saveload()
        } else {
            vec![0; DMA_SAVELOAD_SLOT_SIZE]
        };
        Self::save_load_call(func, &mut dma_slot);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.dma
                .load_c_saveload(&dma_slot)
                .expect("invalid DMA saveload block");
        }

        let mut ppu_slot = if matches!(func, SaveLoadFunc::Save(_)) {
            self.ppu.save_c_saveload()
        } else {
            vec![0; PPU_SAVELOAD_SLOT_SIZE]
        };
        Self::save_load_call(func, &mut ppu_slot);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.ppu
                .load_c_saveload(&ppu_slot)
                .expect("invalid PPU saveload block");
        }

        Self::save_load_call(func, &mut self.sram);

        Self::save_load_call(func, &mut junk);
        Self::save_load_call(func, &mut self.ram);

        let mut junk4 = [0u8; 4];
        Self::save_load_call(func, &mut junk4);
    }

    pub(super) fn save_snes_state(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        self.backup_spotlight_hdma_to_saveload_buffer();
        self.zelda_save_music_state_to_ram_locked();
        self.internal_save_load(func);
    }

    pub fn state_recorder_save<W: Write>(&mut self, sr: &mut StateRecorder, f: &mut W) {
        let mut arr = ByteArray::default();
        let mut save = SaveLoadFunc::Save(&mut arr);
        self.save_snes_state(&mut save);
        assert!(sr.base_snapshot.data.is_empty() || sr.base_snapshot.size() == arr.size());

        let mut hdr = [0u32; 8];
        hdr[0] = 1;
        hdr[1] = sr.total_frames;
        hdr[2] = sr.log.size() as u32;
        hdr[3] = sr.last_inputs as u32;
        hdr[4] = sr.frames_since_last;
        hdr[5] = if sr.base_snapshot.size() != 0 { 1 } else { 0 };
        hdr[6] = arr.size() as u32;
        if sr.replay_mode {
            hdr[5] |= sr.replay_pos_last_complete << 1;
            hdr[7] = sr.replay_frame_counter;
        }
        for value in hdr {
            f.write_all(&value.to_le_bytes()).expect("fwrite failed");
        }
        f.write_all(&sr.log.data).expect("fwrite failed");
        f.write_all(&sr.base_snapshot.data).expect("fwrite failed");
        f.write_all(&arr.data).expect("fwrite failed");
    }

    pub fn save_load_slot(&mut self, cmd: SaveLoadCommand, which: i32) {
        if let Some(path) = Self::save_slot_path(cmd, which) {
            if cmd == SaveLoadCommand::Save {
                if let Ok(mut file) = fs::File::create(path) {
                    println!("*** Saving slot {which}");
                    let mut state_recorder = std::mem::take(&mut self.state_recorder);
                    self.state_recorder_save(&mut state_recorder, &mut file);
                    self.state_recorder = state_recorder;
                }
            } else {
                if let Ok(mut file) = fs::File::open(path) {
                    let action = if cmd == SaveLoadCommand::Load {
                        "Loading"
                    } else {
                        "Replaying"
                    };
                    println!("*** {action} slot {which}");
                    let mut state_recorder = std::mem::take(&mut self.state_recorder);
                    self.state_recorder_load(
                        &mut state_recorder,
                        &mut file,
                        cmd == SaveLoadCommand::Replay,
                    );
                    self.state_recorder = state_recorder;
                }
            }
        }
    }

    pub fn replay_save_file(&mut self, path: &Path) -> std::io::Result<()> {
        let mut file = fs::File::open(path)?;
        let mut state_recorder = std::mem::take(&mut self.state_recorder);
        self.state_recorder_load(&mut state_recorder, &mut file, true);
        self.state_recorder = state_recorder;
        Ok(())
    }

    pub(super) fn save_slot_path(cmd: SaveLoadCommand, which: i32) -> Option<PathBuf> {
        if which & 256 != 0 {
            if cmd == SaveLoadCommand::Save {
                return None;
            }
            let index = (which - 256) as usize;
            Some(Path::new("saves/ref").join(REFERENCE_SAVE_NAMES[index]))
        } else {
            Some(PathBuf::from(format!("saves/save{which}.sav")))
        }
    }

    pub fn zelda_read_sram(&mut self) {
        let path = Self::sram_path();
        if let Ok(mut file) = fs::File::open(&path) {
            let mut total = 0usize;
            while total < SRAM_SIZE {
                match file.read(&mut self.sram[total..SRAM_SIZE]) {
                    Ok(0) => break,
                    Ok(n) => total += n,
                    Err(_) => break,
                }
            }
            if total != SRAM_SIZE {
                eprintln!("Error reading {}", path.display());
            }
            self.emu_synchronize_whole_state();
        }
    }

    pub fn zelda_write_sram(&self) {
        let path = Self::sram_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let backup_path = path.with_extension("bak");
        let _ = fs::rename(&path, &backup_path);
        match fs::File::create(&path) {
            Ok(mut file) => {
                let _ = file.write_all(&self.sram);
            }
            Err(_) => eprintln!("Unable to write {}", path.display()),
        }
    }

    pub(super) fn sram_path() -> PathBuf {
        Self::sram_path_from_env(
            env::var_os("ZELDA3_SAVE_DIR"),
            env::var_os("XDG_DATA_HOME"),
            env::var_os("HOME"),
        )
    }

    pub(super) fn sram_path_from_env(
        save_dir: Option<OsString>,
        xdg_data_home: Option<OsString>,
        home: Option<OsString>,
    ) -> PathBuf {
        if let Some(save_dir) = non_empty_path(save_dir) {
            return save_dir.join("sram.dat");
        }
        if let Some(xdg_data_home) = non_empty_path(xdg_data_home) {
            return xdg_data_home
                .join("zelda3-rs")
                .join("saves")
                .join("sram.dat");
        }
        if let Some(home) = non_empty_path(home) {
            return home
                .join(".local")
                .join("share")
                .join("zelda3-rs")
                .join("saves")
                .join("sram.dat");
        }
        PathBuf::from("saves/sram.dat")
    }
}
