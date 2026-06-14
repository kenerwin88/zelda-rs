// Methods ported from zelda3/src/select_file.c and included inside ZeldaState.
#![allow(non_snake_case)]

use super::*;

fn read_name_player_tab1_byte_word(tab: &[i16; 26], offs: usize) -> u16 {
    let lo = tab[offs / 2].to_le_bytes()[offs & 1] as u16;
    let hi = tab[(offs + 1) / 2].to_le_bytes()[(offs + 1) & 1] as u16;
    lo | (hi << 8)
}

impl ZeldaState {
    pub(super) fn module01_file_select(&mut self) {
        if self.state_recorder.replay_mode
            && std::env::var_os("ZELDA3_SMV_SELECT_FILE_TIMING_HACKS").is_some()
            && self.game_state.frame.saved_module_for_menu == 0
            && self.game_state.messaging.dialogue_message_index.value() == 0x000a
        {
            let stall = match self.game_state.frame.submodule {
                1 => 58,
                2 => 1,
                _ => 0,
            };
            if stall != 0 {
                let before = self.replay_reload_file_select_stall;
                if self.replay_reload_file_select_stall == 0 {
                    self.replay_reload_file_select_stall = stall + 1;
                }
                let seeded = self.replay_reload_file_select_stall;
                self.replay_reload_file_select_stall =
                    self.replay_reload_file_select_stall.wrapping_sub(1);
                let after = self.replay_reload_file_select_stall;
                if std::env::var_os("ZELDA3_REPLAY_FILE_SELECT_STALL_DUMP").is_some() {
                    println!(
                        "file-select-stall frame={} sub={} msg=0x{:04x} stall={} before={} seeded={} after={} return={} save_slot_flags={},{},{}",
                        self.frame_ctr_dbg,
                        self.game_state.frame.submodule,
                        self.game_state.messaging.dialogue_message_index.value(),
                        stall,
                        before,
                        seeded,
                        after,
                        if after != 0 { 1 } else { 0 },
                        self.game_state.messaging.select_file_menu.save_slot_flag(0),
                        self.game_state.messaging.select_file_menu.save_slot_flag(1),
                        self.game_state.messaging.select_file_menu.save_slot_flag(2),
                    );
                }
                if self.replay_reload_file_select_stall != 0 {
                    return;
                }
            } else {
                if self.replay_reload_file_select_stall != 0
                    && std::env::var_os("ZELDA3_REPLAY_FILE_SELECT_STALL_DUMP").is_some()
                {
                    println!(
                        "file-select-stall frame={} sub={} msg=0x{:04x} stall=0 before={} seeded={} after=0 return=0 save_slot_flags={},{},{}",
                        self.frame_ctr_dbg,
                        self.game_state.frame.submodule,
                        self.game_state.messaging.dialogue_message_index.value(),
                        self.replay_reload_file_select_stall,
                        self.replay_reload_file_select_stall,
                        self.game_state.messaging.select_file_menu.save_slot_flag(0),
                        self.game_state.messaging.select_file_menu.save_slot_flag(1),
                        self.game_state.messaging.select_file_menu.save_slot_flag(2),
                    );
                }
                self.replay_reload_file_select_stall = 0;
            }
        } else {
            if self.replay_reload_file_select_stall != 0
                && std::env::var_os("ZELDA3_REPLAY_FILE_SELECT_STALL_DUMP").is_some()
            {
                println!(
                    "file-select-stall frame={} sub={} msg=0x{:04x} stall=0 before={} seeded={} after=0 return=0 save_slot_flags={},{},{}",
                    self.frame_ctr_dbg,
                    self.game_state.frame.submodule,
                    self.game_state.messaging.dialogue_message_index.value(),
                    self.replay_reload_file_select_stall,
                    self.replay_reload_file_select_stall,
                    self.game_state.messaging.select_file_menu.save_slot_flag(0),
                    self.game_state.messaging.select_file_menu.save_slot_flag(1),
                    self.game_state.messaging.select_file_menu.save_slot_flag(2),
                );
            }
            self.replay_reload_file_select_stall = 0;
        }

        self.set_bg3_h_copy2(0);
        self.set_bg3_v_copy2(0);
        match self.game_state.frame.submodule {
            0 => self.module_select_file_0(),
            1 => self.file_select_re_init_save_flags_and_erase_triforce(),
            2 => self.module_erase_file_1(),
            3 => self.file_select_trigger_stripes_and_advance(),
            4 => self.file_select_trigger_name_stripes_and_advance(),
            5 => self.file_select_main(),
            _ => {}
        }
    }

    pub(super) fn module_select_file_0(&mut self) {
        self.enable_force_blank();
        self.deactivate_nmi_thread();
        self.clear_pending_polyhedral_update();
        self.set_music_control(11);
        self.increment_submodule();
        self.select_overworld_aux_palette_offset();
        self.set_palette_main_indoors(6);
        self.set_core_update_disable_flag(6);
        self.palette_load_dungeon_set();
        self.palette_load_ow_bg3();
        self.world_palette_theme_mut().set_hud_palette(0);
        self.palette_load_hud();
        self.save_progress_mut().set_hud_current_item(0);
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(1);
        self.world_palette_theme_mut().set_main_tile_theme_index(35);
        self.world_palette_theme_mut().set_aux_tile_theme_index(81);
        self.load_default_graphics();
        self.initialize_tilesets();
        self.load_file_select_graphics();
        self.intro_validate_sram();
        self.decompress_enemy_damage_subclasses();
    }

    pub(super) fn load_file_select_graphics(&mut self) {
        let data = self
            .decompressed_sprite_graphics_data(0x5e)
            .expect("file select sprite sheet 0x5e must decompress");
        self.copy_to_primary_decompression_buffer(&data);
        self.do3_to_4_high_to_vram(0x5000, &data);

        let data = self
            .decompressed_sprite_graphics_data(0x5f)
            .expect("file select sprite sheet 0x5f must decompress");
        self.copy_to_primary_decompression_buffer(&data);
        self.do3_to_4_high_to_vram(0x5400, &data);

        self.transfer_font_to_vram();

        let data = self
            .decompressed_sprite_graphics_data(0x6b)
            .expect("file select sprite sheet 0x6b must decompress");
        self.copy_to_primary_decompression_buffer(&data);
        for i in 0..0x300 {
            self.ppu.vram[0x7800 + i] = read_word_from_slice(&data, i * 2);
        }
    }

    pub(super) fn intro_validate_sram(&mut self) {
        for i in 0..3 {
            let slot = i * 0x500;
            let backup = slot + 0x0f00;
            if !Self::intro_check_cksum_slice(&self.sram[slot..slot + 0x500]) {
                if Self::intro_check_cksum_slice(&self.sram[backup..backup + 0x500]) {
                    let backup_data = self.sram[backup..backup + 0x500].to_vec();
                    self.sram[slot..slot + 0x500].copy_from_slice(&backup_data);
                } else {
                    self.sram[slot..slot + 0x500].fill(0);
                    self.sram[backup..backup + 0x500].fill(0);
                }
            }
        }
        self.sprite_system_mut().clear_live_table_pages();
    }

    fn intro_check_cksum(s: *const u8) -> bool {
        Self::intro_check_cksum_slice(unsafe { std::slice::from_raw_parts(s, 0x500) })
    }

    fn intro_check_cksum_slice(bytes: &[u8]) -> bool {
        let mut sum = 0u16;
        for i in 0..0x280 {
            sum = sum.wrapping_add(read_word_from_slice(bytes, i * 2));
        }
        sum == 0x5a5a
    }

    pub(super) fn file_select_re_init_save_flags_and_erase_triforce(&mut self) {
        self.clear_select_file_save_slot_flags();
        self.file_select_erase_triforce();
    }

    pub(super) fn file_select_erase_triforce(&mut self) {
        self.set_core_update_disable_flag(0x80);
        self.enable_force_blank();
        self.erase_tile_maps_triforce();
        self.palette_load_for_file_select();
        self.increment_cgram_update_flag();
        self.increment_submodule();
    }

    pub(super) fn module_erase_file_1(&mut self) {
        const SELECT_FILE_GFX0: [u8; 224] = [
            0x10, 0x42, 0, 0x27, 0x89, 0x35, 0x8a, 0x35, 0x8b, 0x35, 0x8c, 0x35, 0x8b, 0x35, 0x8c,
            0x35, 0x8b, 0x35, 0x8c, 0x35, 0x8b, 0x35, 0x8c, 0x35, 0x8b, 0x35, 0x8c, 0x35, 0x8b,
            0x35, 0x8c, 0x35, 0x8b, 0x35, 0x8c, 0x35, 0x8b, 0x35, 0x8c, 0x35, 0x8a, 0x75, 0x89,
            0x75, 0x10, 0x62, 0, 3, 0x99, 0x35, 0x9a, 0x35, 0x10, 0x64, 0x40, 0x1e, 0x7f, 0x34,
            0x10, 0x74, 0, 3, 0x9a, 0x75, 0x99, 0x75, 0x10, 0x82, 0, 3, 0xa9, 0x35, 0xaa, 0x35,
            0x10, 0x84, 0x40, 0x1e, 0x7f, 0x34, 0x10, 0x94, 0, 3, 0xaa, 0x75, 0xa9, 0x75, 0x10,
            0xa2, 0, 0x27, 0x9d, 0x35, 0xad, 0x35, 0x9b, 0x35, 0x9c, 0x35, 0x9b, 0x35, 0x9c, 0x35,
            0x9b, 0x35, 0x9c, 0x35, 0x9b, 0x35, 0x9c, 0x35, 0x9b, 0x35, 0x9c, 0x35, 0x9b, 0x35,
            0x9c, 0x35, 0x9b, 0x35, 0x9c, 0x35, 0x9b, 0x35, 0x9c, 0x35, 0xad, 0x75, 0x9d, 0x75,
            0x10, 0xc2, 0, 0x27, 0xab, 0x35, 0xac, 0x35, 0xab, 0x35, 0xac, 0x35, 0xab, 0x35, 0xac,
            0x35, 0xab, 0x35, 0xac, 0x35, 0xab, 0x35, 0xac, 0x35, 0xab, 0x35, 0xac, 0x35, 0xab,
            0x35, 0xac, 0x35, 0xab, 0x35, 0xac, 0x35, 0xab, 0x35, 0xac, 0x35, 0xab, 0x75, 0xac,
            0x75, 0x10, 0xe2, 0, 1, 0x83, 0x35, 0x10, 0xe3, 0x40, 0x32, 0x85, 0x35, 0x10, 0xfd, 0,
            1, 0x84, 0x35, 0x11, 2, 0xc0, 0x22, 0x86, 0x35, 0x11, 0x1d, 0xc0, 0x22, 0x96, 0x35,
            0x13, 0x42, 0, 1, 0x93, 0x35, 0x13, 0x43, 0x40, 0x32, 0x95, 0x35, 0x13, 0x5d, 0, 1,
            0x94, 0x35,
        ];

        let mut dst = self.select_file_func1();
        let upload_base = self.game_state.display.vram_upload_buffer_base();
        self.copy_vram_upload_buffer_bytes(dst - upload_base, &SELECT_FILE_GFX0);
        dst += SELECT_FILE_GFX0.len();

        let mut t = 0x1103u16;
        for _ in 0..18 {
            self.write_vram_upload_absolute_word(dst, t.swap_bytes());
            dst += 2;
            self.write_vram_upload_absolute_word(dst, 0x3240);
            dst += 2;
            self.write_vram_upload_absolute_word(dst, 0x347f);
            dst += 2;
            t = t.wrapping_add(0x20);
        }
        self.write_vram_upload_absolute_byte(dst, 0xff);
        self.increment_submodule();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn select_file_func1(&mut self) -> usize {
        const BACKGROUND_CHECKERBOARD_TILES: [u16; 4] = [0x3581, 0x3582, 0x3591, 0x3592];
        let mut dst = self.game_state.display.vram_upload_buffer_base();
        self.write_vram_upload_absolute_word(dst, 0x0010);
        dst += 2;
        self.write_vram_upload_absolute_word(dst, 0xff07);
        dst += 2;
        for i in 0..1024 {
            self.write_vram_upload_absolute_word(
                dst,
                BACKGROUND_CHECKERBOARD_TILES[((i & 0x20) >> 4) + (i & 1)],
            );
            dst += 2;
        }
        dst
    }

    pub(super) fn file_select_trigger_stripes_and_advance(&mut self) {
        self.restore_select_file_remembered_cursor();
        self.increment_submodule();
        self.set_bg_vram_load_mode(6);
    }

    pub(super) fn file_select_trigger_name_stripes_and_advance(&mut self) {
        let mut data = Vec::with_capacity(253);
        for &(addr_hi, addr_lo, first, second) in &[
            (0x61, 0x29, 0xe7, None),
            (0x61, 0x49, 0xf7, Some(0x91)),
            (0x61, 0xa9, 0xe8, None),
            (0x61, 0xc9, 0xf8, Some(0x91)),
            (0x62, 0x29, 0xe9, None),
            (0x62, 0x49, 0xf9, Some(0x91)),
        ] {
            data.extend_from_slice(&[addr_hi, addr_lo, 0, 0x25, first, 0x18]);
            let mut remaining = 18;
            if let Some(second) = second {
                data.extend_from_slice(&[second, 0x18]);
                remaining -= 1;
            }
            for _ in 0..remaining {
                data.extend_from_slice(&[0xa9, 0x18]);
            }
        }
        data.push(0xff);
        debug_assert_eq!(data.len(), 253);
        self.copy_vram_upload_buffer_bytes(0, &data);
        self.set_screen_brightness(0x0f);
        self.clear_core_update_disable_flag();
        self.increment_submodule();
        self.set_bg_vram_load_mode(6);
    }

    pub(super) fn file_select_main(&mut self) {
        const FAERIE_Y: [u8; 5] = [0x4a, 0x6a, 0x8a, 0xaf, 0xbf];

        if self.game_state.messaging.select_file_menu.cursor() < 3 {
            self.remember_select_file_cursor();
        }

        for k in 0..3 {
            if read_le_u16(&self.sram, k * 0x500 + 0x03e5) == 0x55aa {
                self.mark_select_file_save_slot_present(k);
                self.select_file_func5_draw_oams(k);
                self.select_file_func6_draw_oams2(k);
                self.select_file_func17(k);
            }
        }

        let y = FAERIE_Y[self.game_state.messaging.select_file_menu.cursor_usize()];
        self.file_select_draw_fairy(0x1c, y);
        self.set_bg_vram_load_mode(1);

        let a = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xfc;
        if a & 0x2c != 0 {
            self.set_sound_effect_2(0x20);
            if a & 8 != 0 {
                self.decrement_select_file_cursor();
                if self.game_state.messaging.select_file_menu.cursor() & 0x80 != 0 {
                    self.set_select_file_cursor(4);
                }
            } else {
                self.increment_select_file_cursor();
                if self.game_state.messaging.select_file_menu.cursor() == 5 {
                    self.clear_select_file_cursor();
                }
            }
        } else if a != 0 {
            self.set_sound_effect_1(0x2c);
            if self.game_state.messaging.select_file_menu.cursor() < 3 {
                self.clear_select_file_transition_scratch();
                if self
                    .game_state
                    .messaging
                    .select_file_menu
                    .save_slot_flag(self.game_state.messaging.select_file_menu.cursor_usize())
                    == 0
                {
                    self.set_main_module(4);
                    self.set_submodule(0);
                    self.set_subsubmodule(0);
                } else {
                    self.set_music_control(0xf1);
                    let slot = self
                        .game_state
                        .messaging
                        .select_file_menu
                        .cursor()
                        .wrapping_mul(2)
                        .wrapping_add(2);
                    self.set_selected_save_slot_x2(slot as u16);
                    let save_offset =
                        self.game_state.messaging.select_file_menu.cursor() as u16 * 0x500;
                    self.save_load_scratch_mut().set_source_offset(save_offset);
                    self.CopySaveToWRAM();
                }
            } else if self
                .game_state
                .messaging
                .select_file_menu
                .any_save_slot_flag()
            {
                let main_module = if self.game_state.messaging.select_file_menu.cursor() == 3 {
                    2
                } else {
                    3
                };
                self.set_main_module(main_module);
                self.clear_select_file_cursor();
                self.set_submodule(0);
                self.set_subsubmodule(0);
            } else {
                self.set_sound_effect_1(0x3c);
            }
        }
    }

    pub(super) fn file_select_draw_fairy(&mut self, x: u8, y: u8) {
        let charnum = if self.game_state.frame.frame_counter & 8 != 0 {
            0xaa
        } else {
            0xa8
        };
        self.set_oam_plain(0, x, y, charnum, 0x7e, 2);
    }

    pub(super) fn select_file_func5_draw_oams(&mut self, k: usize) {
        const OAM_IDX: [usize; 3] = [0x28, 0x3c, 0x50];
        const Y: [u8; 3] = [0x43, 0x63, 0x83];
        const SWORD_CHAR: [u8; 4] = [0x85, 0xa1, 0xa1, 0xa1];
        const SHIELD_CHAR: [u8; 3] = [0xc4, 0xca, 0xe0];
        const FLAGS: [u8; 3] = [0x72, 0x76, 0x7a];
        const FLAGS2: [u8; 3] = [0x32, 0x36, 0x3a];
        const FLAGS3: [u8; 3] = [0x30, 0x34, 0x38];

        self.follower_link_state_mut()
            .set_link_dma_graphics_index_word(0x116 * 2);
        let sram_base = k * 0x500;
        let oam = OAM_IDX[k] / 4;
        let x = 0x34u8;
        let y = Y[k];

        let sword = self.sram[sram_base + KSRM_OFFS_SWORD].wrapping_sub(1);
        let sword_char = SWORD_CHAR[if sword & 0x80 != 0 { 0 } else { sword as usize }];
        self.set_oam_plain(
            oam,
            x.wrapping_add(0x0c),
            y.wrapping_sub(5),
            sword_char,
            FLAGS[k],
            0,
        );
        self.set_oam_plain(
            oam + 1,
            x.wrapping_add(0x0c),
            y.wrapping_add(3),
            sword_char.wrapping_add(16),
            FLAGS[k],
            0,
        );
        if sword & 0x80 != 0 {
            self.oam_state_mut().hide_sprite_row(oam);
            self.oam_state_mut().hide_sprite_row(oam + 1);
        }

        let shield = self.sram[sram_base + KSRM_OFFS_SHIELD].wrapping_sub(1);
        let shield_char = SHIELD_CHAR[if shield & 0x80 != 0 {
            0
        } else {
            shield as usize
        }];
        self.set_oam_plain(
            oam + 2,
            x.wrapping_sub(5),
            y.wrapping_add(10),
            shield_char,
            FLAGS2[k],
            2,
        );
        if shield & 0x80 != 0 {
            self.oam_state_mut().hide_sprite_row(oam + 2);
        }
        self.set_oam_plain(oam + 3, x, y, 0, FLAGS3[k], 2);
        self.set_oam_plain(oam + 4, x, y.wrapping_add(8), 2, FLAGS3[k] | 0x40, 2);
    }

    pub(super) fn select_file_func6_draw_oams2(&mut self, k: usize) {
        const DIGIT_CHAR: [u8; 10] = [0xd0, 0xac, 0xad, 0xbc, 0xbd, 0xae, 0xaf, 0xbe, 0xbf, 0xc0];
        const OAM_IDX: [usize; 3] = [4, 16, 28];
        const X_OFFS: [i8; 3] = [12, 4, -4];
        const Y: [u8; 3] = [0x43, 0x63, 0x83];

        let sram_base = k * 0x500;
        let mut died_ctr = read_le_u16(&self.sram, sram_base + KSRM_OFFS_DIED_COUNTER);
        if died_ctr == 0xffff {
            return;
        }
        if died_ctr > 999 {
            died_ctr = 999;
        }
        let digits = [
            (died_ctr % 10) as u8,
            ((died_ctr / 10) % 10) as u8,
            (died_ctr / 100) as u8,
        ];
        let mut i = if digits[2] != 0 {
            2usize
        } else if digits[1] != 0 {
            1
        } else {
            0
        };
        let mut oam = OAM_IDX[k] / 4;
        loop {
            self.set_oam_plain(
                oam,
                0x34u8.wrapping_add(X_OFFS[i] as u8),
                Y[k].wrapping_add(0x10),
                DIGIT_CHAR[digits[i] as usize],
                0x3c,
                0,
            );
            if i == 0 {
                break;
            }
            oam += 1;
            i -= 1;
        }
    }

    pub(super) fn select_file_func17(&mut self, k: usize) {
        const NAME_VRAM_OFFS: [usize; 3] = [8, 0x5c, 0xb0];
        const HEALTH_VRAM_OFFS: [usize; 3] = [0x16, 0x6a, 0xbe];
        let sram_base = k * 0x500;

        let mut dst = self
            .game_state
            .display
            .vram_upload_buffer_address(NAME_VRAM_OFFS[k]);
        for i in 0..6 {
            let t =
                read_le_u16(&self.sram, sram_base + KSRM_OFFS_NAME + i * 2).wrapping_add(0x1800);
            self.write_vram_upload_absolute_word(dst, t);
            self.write_vram_upload_absolute_word(dst + 42, t.wrapping_add(0x10));
            dst += 2;
        }

        let mut health = self.sram[sram_base + KSRM_OFFS_HEALTH] >> 3;
        let mut dst = self
            .game_state
            .display
            .vram_upload_buffer_address(HEALTH_VRAM_OFFS[k]);
        let dst_org = dst;
        let mut row = 10u8;
        loop {
            self.write_vram_upload_absolute_word(dst, 0x0520);
            dst += 2;
            row = row.wrapping_sub(1);
            if row == 0 {
                dst = dst_org + 42;
            }
            health = health.wrapping_sub(1);
            if health == 0 {
                break;
            }
        }
    }

    pub(super) fn select_file_func16(&mut self) {
        const FAERIE_Y: [u8; 2] = [175, 191];
        self.file_select_draw_fairy(
            0x1c,
            FAERIE_Y[self.game_state.messaging.select_file_menu.cursor_usize()],
        );

        let mut k = self.game_state.messaging.select_file_menu.cursor();
        if self.game_state.player.follower_link.filtered_joypad_h() & 0x2c != 0 {
            k = if self.game_state.player.follower_link.filtered_joypad_h() & 0x24 != 0 {
                k.wrapping_add(1)
            } else {
                k.wrapping_sub(1)
            };
            self.set_select_file_cursor(k & 1);
            self.set_sound_effect_2(0x20);
        }

        let a = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xd0;
        if a != 0 {
            self.set_sound_effect_1(0x2c);
            if self.game_state.messaging.select_file_menu.cursor() == 0 {
                self.set_sound_effect_2(0x22);
                self.set_sound_effect_1(0);
                let k = self.game_state.frame.subsubmodule as usize;
                self.clear_select_file_save_slot_flag(k);
                let base = k * 0x500;
                self.sram[base..base + 0x500].fill(0);
                self.sram[base + 0x0f00..base + 0x1400].fill(0);
                self.zelda_write_sram();
            }
            self.return_to_file_select();
            self.set_subsubmodule(0);
        }
    }

    pub(super) fn module02_copy_file(&mut self) {
        self.clear_select_file_remembered_cursor();
        match self.game_state.frame.submodule {
            0 => self.file_select_erase_triforce(),
            1 => self.module_erase_file_1(),
            2 => self.module_copy_file_2(),
            3 => self.copy_file_choose_selection(),
            4 => self.copy_file_choose_target(),
            5 => self.copy_file_confirm_selection(),
            _ => {}
        }
    }

    pub(super) fn module_copy_file_2(&mut self) {
        self.set_bg_vram_load_mode(7);
        self.increment_submodule();
        self.set_screen_brightness(0x0f);
        self.clear_core_update_disable_flag();
        let mut i = 0usize;
        while self.game_state.messaging.select_file_menu.save_slot_flag(i) == 0 {
            i += 1;
        }
        self.set_select_file_cursor(i as u8);
    }

    pub(super) fn copy_file_choose_selection(&mut self) {
        self.copy_file_selection_and_blinker();
        if self.game_state.frame.submodule == 3 && self.game_state.frame.frame_counter & 0x30 == 0 {
            self.file_picker_delete_header_stripe();
        }
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn copy_file_choose_target(&mut self) {
        self.copy_file_target_selection_and_blink();
        if self.game_state.frame.submodule == 4 && self.game_state.frame.frame_counter & 0x30 == 0 {
            self.file_picker_delete_header_stripe();
        }
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn copy_file_confirm_selection(&mut self) {
        self.copy_file_handle_confirmation();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn file_picker_delete_header_stripe(&mut self) {
        const DST: [usize; 2] = [4, 0x1e];
        for j in (0..2).rev() {
            let dst = self.game_state.display.vram_upload_buffer_address(DST[j]);
            for i in 0..11 {
                self.write_vram_upload_absolute_word(dst + i * 2, 0x00a9);
            }
        }
    }

    pub(super) fn copy_file_selection_and_blinker(&mut self) {
        const COPY_SOURCE_SELECTION_STRIPE: [u8; 173] = [
            0x61, 4, 0, 0x15, 0x85, 0x18, 0x26, 0x18, 7, 0x18, 0xaf, 0x18, 2, 0x18, 7, 0x18, 0x6f,
            0x18, 0x86, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0x24, 0, 0x15, 0x95, 0x18,
            0x36, 0x18, 0x17, 0x18, 0xbf, 0x18, 0x12, 0x18, 0x17, 0x18, 0x7f, 0x18, 0x96, 0x18,
            0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0x67, 0, 0xf, 0xe7, 0x18, 0xa9, 0x18, 0xa9,
            0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0x87, 0, 0xf,
            0xf7, 0x18, 0x91, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
            0xa9, 0x18, 0x61, 0xc7, 0, 0xf, 0xe8, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
            0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0xe7, 0, 0xf, 0xf8, 0x18, 0x91, 0x18,
            0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x62, 0x27, 0,
            0xf, 0xe9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
            0x18, 0xa9, 0x18, 0x62, 0x47, 0, 0xf, 0xf9, 0x18, 0x91, 0x18, 0xa9, 0x18, 0xa9, 0x18,
            0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xff,
        ];
        const COPY_TARGET_HEADER_STRIPE: [u8; 73] = [
            0x61, 0x67, 0x40, 0xe, 0xa9, 0, 0x61, 0x87, 0x40, 0xe, 0xa9, 0, 0x61, 0xc7, 0x40, 0xe,
            0xa9, 0, 0x61, 0xe7, 0x40, 0xe, 0xa9, 0, 0x11, 0x30, 0, 1, 0x83, 0x35, 0x11, 0x31,
            0x40, 0x14, 0x85, 0x35, 0x11, 0x3c, 0, 1, 0x84, 0x35, 0x11, 0x50, 0xc0, 0xe, 0x86,
            0x35, 0x11, 0x5c, 0xc0, 0xe, 0x96, 0x35, 0x12, 0x50, 0, 1, 0x93, 0x35, 0x12, 0x51,
            0x40, 0x14, 0x95, 0x35, 0x12, 0x5c, 0, 1, 0x94, 0x35, 0xff,
        ];
        const DST: [usize; 3] = [0x3c, 0x64, 0x8c];
        const FAERIE_X: [u8; 4] = [36, 36, 36, 28];
        const FAERIE_Y: [u8; 4] = [87, 111, 135, 191];

        self.set_vram_upload_cursor(0x00ac);
        self.copy_vram_upload_buffer_bytes(0, &COPY_SOURCE_SELECTION_STRIPE);

        for k in 0..3 {
            if self.game_state.messaging.select_file_menu.save_slot_flag(k) & 1 != 0 {
                let mut dst = self.game_state.display.vram_upload_buffer_address(DST[k]);
                for i in 0..6 {
                    let t = read_le_u16(&self.sram, k * 0x500 + KSRM_OFFS_NAME + i * 2)
                        .wrapping_add(0x1800);
                    self.write_vram_upload_absolute_word(dst, t);
                    self.write_vram_upload_absolute_word(dst + 20, t.wrapping_add(0x10));
                    dst += 2;
                }
            }
        }
        let r16 = self.game_state.messaging.select_file_menu.cursor_usize();
        self.file_select_draw_fairy(FAERIE_X[r16], FAERIE_Y[r16]);

        let a = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xfc;
        if a & 0x2c != 0 {
            let mut k = self.game_state.messaging.select_file_menu.cursor();
            if a & 8 != 0 {
                loop {
                    k = k.wrapping_sub(1);
                    if k & 0x80 != 0 {
                        k = 3;
                        break;
                    }
                    if self
                        .game_state
                        .messaging
                        .select_file_menu
                        .save_slot_flag(k as usize)
                        != 0
                    {
                        break;
                    }
                }
            } else {
                loop {
                    k = k.wrapping_add(1);
                    if k >= 4 {
                        k = 0;
                    }
                    if k == 3
                        || self
                            .game_state
                            .messaging
                            .select_file_menu
                            .save_slot_flag(k as usize)
                            != 0
                    {
                        break;
                    }
                }
            }
            self.set_select_file_cursor(k);
            self.set_sound_effect_2(0x20);
        } else if a != 0 {
            self.set_sound_effect_1(0x2c);
            if self.game_state.messaging.select_file_menu.cursor() == 3 {
                self.return_to_file_select();
                return;
            }
            let r16 = self.game_state.messaging.select_file_menu.cursor();
            self.set_select_file_copy_source_slot(r16);
            self.copy_vram_upload_buffer_bytes(52, &COPY_TARGET_HEADER_STRIPE);
            if self.game_state.messaging.select_file_menu.cursor() != 2 {
                let dst = self.game_state.display.vram_upload_buffer_address(
                    self.game_state.messaging.select_file_menu.cursor_usize() * 12,
                );
                self.write_vram_upload_absolute_word(dst + 52, 0x2762);
                self.write_vram_upload_absolute_word(dst + 58, 0x4762);
            }
            self.increment_submodule();
            self.clear_select_file_cursor();
        }
    }

    pub(super) fn copy_file_target_selection_and_blink(&mut self) {
        let mut k = 1usize;
        let mut t = 4i32;
        loop {
            if t as u16
                != self
                    .game_state
                    .messaging
                    .select_file_menu
                    .copy_source_slot_x2()
            {
                self.set_select_file_choice(k, t as u8);
                k = k.wrapping_sub(1);
            }
            t -= 2;
            if t < 0 {
                break;
            }
        }

        const COPY_TARGET_SELECTION_STRIPE: [u8; 133] = [
            0x61, 0x51, 0, 0x15, 0x85, 0x18, 0x23, 0x18, 0xe, 0x18, 0xa9, 0x18, 0x26, 0x18, 7,
            0x18, 0xaf, 0x18, 2, 0x18, 7, 0x18, 0x6f, 0x18, 0x86, 0x18, 0x61, 0x71, 0, 0x15, 0x95,
            0x18, 0x33, 0x18, 0x1e, 0x18, 0xb9, 0x18, 0x36, 0x18, 0x17, 0x18, 0xbf, 0x18, 0x12,
            0x18, 0x17, 0x18, 0x7f, 0x18, 0x96, 0x18, 0x61, 0xb4, 0, 0xf, 0xa9, 0x18, 0xa9, 0x18,
            0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0xd4, 0,
            0xf, 0xa9, 0x18, 0x91, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
            0x18, 0xa9, 0x18, 0x62, 0x14, 0, 0xf, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
            0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x62, 0x34, 0, 0xf, 0xa9, 0x18, 0x91,
            0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xff,
        ];
        const COPY_TARGET_CONFIRM_STRIPE: [u8; 49] = [
            0x61, 0xb4, 0x40, 0xe, 0xa9, 0, 0x61, 0xd4, 0x40, 0xe, 0xa9, 0, 0x62, 0xc6, 0, 0xd, 2,
            0x18, 0xe, 0x18, 0xf, 0x18, 0x28, 0x18, 0xa9, 0x18, 0xe, 0x18, 0xa, 0x18, 0x62, 0xe6,
            0, 0xd, 0x12, 0x18, 0x1e, 0x18, 0x1f, 0x18, 0x38, 0x18, 0xa9, 0x18, 0x1e, 0x18, 0x1a,
            0x18, 0xff,
        ];
        const FAERIE_X: [u8; 3] = [0x8c, 0x8c, 0x1c];
        const FAERIE_Y: [u8; 3] = [0x67, 0x7f, 0xbf];
        const DST: [usize; 2] = [0x38, 0x60];
        const COPY_TARGET_SLOT_BLANK_TILES: [u16; 3] = [0x18e7, 0x18e8, 0x18e9];

        self.copy_vram_upload_buffer_bytes(0, &COPY_TARGET_SELECTION_STRIPE);
        let mut j = 0usize;
        for k in 0..3 {
            if k * 2
                == self
                    .game_state
                    .messaging
                    .select_file_menu
                    .copy_source_slot_x2() as usize
            {
                continue;
            }
            let mut dst = self.game_state.display.vram_upload_buffer_address(DST[j]);
            j += 1;
            let t = COPY_TARGET_SLOT_BLANK_TILES[k];
            self.write_vram_upload_absolute_word(dst, t);
            self.write_vram_upload_absolute_word(dst + 20, t.wrapping_add(0x10));
            dst += 4;
            if self.game_state.messaging.select_file_menu.save_slot_flag(k) != 0 {
                for i in 0..6 {
                    let t = read_le_u16(&self.sram, k * 0x500 + KSRM_OFFS_NAME + i * 2)
                        .wrapping_add(0x1800);
                    self.write_vram_upload_absolute_word(dst, t);
                    self.write_vram_upload_absolute_word(dst + 20, t.wrapping_add(0x10));
                    dst += 2;
                }
            }
        }
        self.set_vram_upload_cursor(132);
        let r16 = self.game_state.messaging.select_file_menu.cursor_usize();
        self.file_select_draw_fairy(FAERIE_X[r16], FAERIE_Y[r16]);

        let a = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xfc;
        if a & 0x2c != 0 {
            let mut k = self.game_state.messaging.select_file_menu.cursor();
            if a & 8 != 0 {
                k = k.wrapping_sub(1);
                if k & 0x80 != 0 {
                    k = 2;
                }
            } else {
                k = k.wrapping_add(1);
                if k >= 3 {
                    k = 0;
                }
            }
            self.set_select_file_cursor(k);
            self.set_sound_effect_2(0x20);
        } else if a != 0 {
            self.set_sound_effect_1(0x2c);
            if self.game_state.messaging.select_file_menu.cursor() == 2 {
                self.return_to_file_select();
                self.clear_select_file_cursor();
                return;
            }
            let r16 = self.game_state.messaging.select_file_menu.cursor_usize();
            let target = self.game_state.messaging.select_file_menu.choice(r16) as u16;
            self.set_select_file_target_word(target);
            self.copy_vram_upload_buffer_bytes(52, &COPY_TARGET_CONFIRM_STRIPE);
            if self.game_state.messaging.select_file_menu.cursor() == 0 {
                self.write_vram_upload_buffer_word(52, 0x1462);
                self.write_vram_upload_buffer_word(58, 0x3462);
            }
            self.increment_submodule();
            self.clear_select_file_cursor();
        }
    }

    pub(super) fn copy_file_handle_confirmation(&mut self) {
        const FAERIE_Y: [u8; 2] = [0xaf, 0xbf];
        self.file_select_draw_fairy(
            0x1c,
            FAERIE_Y[self.game_state.messaging.select_file_menu.cursor_usize()],
        );

        let a = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xfc;
        if a & 0x2c != 0 {
            self.set_sound_effect_2(0x20);
            if a & 0x24 != 0 {
                self.increment_select_file_cursor();
                if self.game_state.messaging.select_file_menu.cursor() >= 2 {
                    self.clear_select_file_cursor();
                }
            } else {
                self.decrement_select_file_cursor();
                if self.game_state.messaging.select_file_menu.cursor() & 0x80 != 0 {
                    self.set_select_file_cursor(1);
                }
            }
        } else if a != 0 {
            self.set_sound_effect_1(0x2c);
            if self.game_state.messaging.select_file_menu.cursor() == 0 {
                let dst_slot =
                    (self.game_state.messaging.select_file_menu.target_word() >> 1) as usize;
                let src_slot = self
                    .game_state
                    .messaging
                    .select_file_menu
                    .copy_source_slot();
                let src = self.sram[src_slot * 0x500..src_slot * 0x500 + 0x500].to_vec();
                self.sram[dst_slot * 0x500..dst_slot * 0x500 + 0x500].copy_from_slice(&src);
                self.mark_select_file_save_slot_present(dst_slot);
                self.zelda_write_sram();
            }
            self.return_to_file_select();
            self.clear_select_file_cursor();
        }
    }

    pub(super) fn module03_kill_file(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.file_select_erase_triforce(),
            1 => self.module_erase_file_1(),
            2 => self.kill_file_set_up(),
            3 => self.kill_file_handle_selection(),
            4 => self.kill_file_handle_confirmation(),
            _ => {}
        }
    }

    pub(super) fn kill_file_set_up(&mut self) {
        self.set_bg_vram_load_mode(8);
        self.increment_submodule();
        self.set_screen_brightness(0x0f);
        self.clear_core_update_disable_flag();
        let mut i = 0usize;
        while self.game_state.messaging.select_file_menu.save_slot_flag(i) == 0 {
            i += 1;
        }
        self.set_select_file_cursor(i as u8);
    }

    pub(super) fn kill_file_handle_selection(&mut self) {
        if self.game_state.messaging.select_file_menu.cursor() < 3 {
            self.remember_select_file_cursor();
        }
        self.kill_file_choose_target();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn kill_file_handle_confirmation(&mut self) {
        self.select_file_func16();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn kill_file_choose_target(&mut self) {
        const FAERIE_X: [u8; 4] = [36, 36, 36, 28];
        const FAERIE_Y: [u8; 4] = [103, 127, 151, 191];
        const KILL_FILE_CONFIRM_STRIPE: [u8; 101] = [
            0x61, 0xa7, 0x40, 0x24, 0xa9, 0, 0x61, 0xc7, 0x40, 0x24, 0xa9, 0, 0x62, 7, 0x40, 0x24,
            0xa9, 0, 0x62, 0x27, 0x40, 0x24, 0xa9, 0, 0x62, 0xc6, 0, 0x21, 4, 0x18, 0x21, 0x18, 0,
            0x18, 0x22, 0x18, 4, 0x18, 0xa9, 0x18, 0x23, 0x18, 7, 0x18, 0xaf, 0x18, 0x22, 0x18,
            0xa9, 0x18, 0xf, 0x18, 0xb, 0x18, 0, 0x18, 0x28, 0x18, 4, 0x18, 0x21, 0x18, 0x62, 0xe6,
            0, 0x21, 0x14, 0x18, 0x31, 0x18, 0x10, 0x18, 0x32, 0x18, 0x14, 0x18, 0xa9, 0x18, 0x33,
            0x18, 0x17, 0x18, 0xbf, 0x18, 0x32, 0x18, 0xa9, 0x18, 0x1f, 0x18, 0x1b, 0x18, 0x10,
            0x18, 0x38, 0x18, 0x14, 0x18, 0x31, 0x18, 0xff,
        ];

        let mut data = Vec::with_capacity(253);
        for &(addr_hi, addr_lo, first, second) in &[
            (0x61, 0xa7, 0xe7, None),
            (0x61, 0xc7, 0xf7, Some(0x91)),
            (0x62, 0x07, 0xe8, None),
            (0x62, 0x27, 0xf8, Some(0x91)),
            (0x62, 0x67, 0xe9, None),
            (0x62, 0x87, 0xf9, Some(0x91)),
        ] {
            data.extend_from_slice(&[addr_hi, addr_lo, 0, 0x25, first, 0x18]);
            let mut remaining = 18;
            if let Some(second) = second {
                data.extend_from_slice(&[second, 0x18]);
                remaining -= 1;
            }
            for _ in 0..remaining {
                data.extend_from_slice(&[0xa9, 0x18]);
            }
        }
        data.push(0xff);
        debug_assert_eq!(data.len(), 253);
        self.copy_vram_upload_buffer_bytes(0, &data);
        for k in 0..3 {
            if self.game_state.messaging.select_file_menu.save_slot_flag(k) != 0 {
                self.select_file_func17(k);
            }
        }

        let r16 = self.game_state.messaging.select_file_menu.cursor_usize();
        self.file_select_draw_fairy(FAERIE_X[r16], FAERIE_Y[r16]);

        let mut k = self.game_state.messaging.select_file_menu.cursor();
        if self.game_state.player.follower_link.filtered_joypad_h() & 0x2c != 0 {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x24 == 0 {
                loop {
                    k = k.wrapping_sub(1);
                    if k & 0x80 != 0 {
                        k = 3;
                        break;
                    }
                    if self
                        .game_state
                        .messaging
                        .select_file_menu
                        .save_slot_flag(k as usize)
                        != 0
                    {
                        break;
                    }
                }
            } else {
                loop {
                    k = k.wrapping_add(1);
                    if k >= 4 {
                        k = 0;
                    }
                    if k == 3
                        || self
                            .game_state
                            .messaging
                            .select_file_menu
                            .save_slot_flag(k as usize)
                            != 0
                    {
                        break;
                    }
                }
            }
            self.set_sound_effect_2(0x20);
        }
        self.set_select_file_cursor(k);

        let a = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xd0;
        if a != 0 {
            self.set_sound_effect_1(0x2c);
            if k == 3 {
                self.return_to_file_select();
                return;
            }
            self.copy_vram_upload_buffer_bytes(0, &KILL_FILE_CONFIRM_STRIPE);
            self.increment_submodule();
            if self.game_state.messaging.select_file_menu.cursor() != 2 {
                let dst = self.game_state.display.vram_upload_buffer_address(
                    self.game_state.messaging.select_file_menu.cursor_usize() * 12,
                );
                self.write_vram_upload_absolute_word(dst, 0x6762);
                self.write_vram_upload_absolute_word(dst + 6, 0x8762);
            }
            let subsubmodule = self.game_state.messaging.select_file_menu.cursor();
            self.set_subsubmodule(subsubmodule);
            self.clear_select_file_cursor();
        }
    }

    pub(super) fn module04_name_file(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.name_file_erase_save(),
            1 => self.module_name_player_1(),
            2 => self.module_name_player_2(),
            3 => self.name_file_do_the_naming(),
            _ => {}
        }
    }

    pub(super) fn name_file_erase_save(&mut self) {
        self.file_select_erase_triforce();
        self.set_irq_control_flag(1);
        self.clear_select_file_name_entry_state();
        self.set_bg3_h_copy2(0);

        let offs = self.game_state.messaging.select_file_menu.cursor_usize() * 0x500;
        self.attract_scene_mut().set_legend_ctr(offs as u16);
        self.sram[offs..offs + 0x500].fill(0);
        for i in 0..6 {
            write_le_u16(&mut self.sram, offs + KSRM_OFFS_NAME + i * 2, 0x00a9);
        }
    }

    pub(super) fn module_name_player_1(&mut self) {
        let dst = self.select_file_func1();
        self.write_vram_upload_absolute_word(dst, 0xffff);
        self.set_bg_vram_load_mode(1);
        self.increment_submodule();
    }

    pub(super) fn module_name_player_2(&mut self) {
        self.set_bg_vram_load_mode(5);
        self.increment_submodule();
        self.set_screen_brightness(0x0f);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn name_file_do_the_naming(&mut self) {
        const NAME_PLAYER_CURSOR_X_DELTAS: [i16; 26] = [
            -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -2, 2, -2, 2, -2, 2, -2, 2, -4,
            4,
        ];
        const NAME_PLAYER_ROW_TILE_BASES: [u8; 4] = [0x83, 0x93, 0xa3, 0xb3];
        const NAME_PLAYER_VRAM_SETUP_WORDS: [u16; 40] = [
            0x1f0, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0,
            0xe0, 0xf0, 0x100, 0x110, 0x120, 0x130, 0x140, 0x150, 0x160, 0x170, 0x180, 0x190,
            0x1a0, 0x1b0, 0x1c0, 0x1d0, 0x1e0, 0x0001, 0x00ff, 0x0020, 0x00ff, 0x0000, 0x001f,
            0x9383, 0xb3a3,
        ];
        const NAME_PLAYER_X: [u8; 6] = [31, 47, 63, 79, 95, 111];
        const NAME_PLAYER_CHAR_TILE_BY_CURSOR: [u8; 128] = [
            6, 7, 0x5f, 9, 0x59, 0x59, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x60, 0x23,
            0x59, 0x59, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x59, 0x59, 0x59, 0, 1, 2, 3, 4, 5, 0x10,
            0x11, 0x12, 0x13, 0x59, 0x59, 0x24, 0x5f, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
            0x2d, 0x59, 0x59, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f, 0x59, 0x59, 0x59, 0x0a, 0x0b, 0x0c,
            0x0d, 0x0e, 0x0f, 0x40, 0x41, 0x42, 0x59, 0x59, 0x59, 0x2e, 0x2f, 0x30, 0x31, 0x32,
            0x33, 0x40, 0x41, 0x42, 0x59, 0x59, 0x59, 0x61, 0x3f, 0x45, 0x46, 0x59, 0x59, 0x59,
            0x59, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x44, 0x59, 0x6f, 0x6f, 0x59, 0x59, 0x59,
            0x59, 0x59, 0x59, 0x59, 0x5a, 0x44, 0x59, 0x6f, 0x6f, 0x59, 0x59, 0x5a, 0x44, 0x59,
            0x6f, 0x6f, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x5a,
        ];

        loop {
            let mut j = self
                .game_state
                .messaging
                .select_file_menu
                .name_scroll_x_step();
            if j == 0 {
                self.name_file_check_for_scroll_input_x();
                break;
            }
            if j != 0x31 {
                self.advance_select_file_name_scroll_x_step_by(4);
            }
            j = j.wrapping_sub(1);
            let target = NAME_PLAYER_VRAM_SETUP_WORDS[self
                .game_state
                .messaging
                .select_file_menu
                .name_column_usize()];
            if target == self.game_state.messaging.select_file_menu.name_scroll_x() {
                let step = if self.game_state.player.follower_link.joypad1h_last() & 3 != 0 {
                    0x30
                } else {
                    0
                };
                self.set_select_file_name_scroll_x_step(step);
                self.name_file_check_for_scroll_input_x();
                continue;
            }
            if self
                .game_state
                .messaging
                .select_file_menu
                .name_scroll_x_direction()
                == 0
            {
                j = j.wrapping_add(2);
            }
            let delta = read_name_player_tab1_byte_word(&NAME_PLAYER_CURSOR_X_DELTAS, j as usize);
            let next = self
                .game_state
                .messaging
                .select_file_menu
                .name_scroll_x()
                .wrapping_add(delta)
                & 0x01ff;
            self.set_select_file_name_scroll_x(next);
            break;
        }

        loop {
            if self
                .game_state
                .messaging
                .select_file_menu
                .name_scroll_y_step()
                == 0
            {
                self.name_file_check_for_scroll_input_y();
                break;
            }
            let target_y = NAME_PLAYER_ROW_TILE_BASES
                [self.game_state.messaging.select_file_menu.name_row_usize()];
            if self.step_select_file_name_cursor_y_toward(target_y) {
                break;
            }
            self.clear_select_file_name_scroll_y_step();
            self.name_file_check_for_scroll_input_y();
        }

        for i in 0..26 {
            self.set_oam_plain(
                i,
                0x18u8.wrapping_add((i as u8).wrapping_mul(8)),
                self.game_state.messaging.select_file_menu.name_cursor_y(),
                0x2e,
                0x3c,
                0,
            );
        }
        self.set_oam_plain(
            26,
            NAME_PLAYER_X[self.game_state.messaging.select_file_menu.name_slot_usize()],
            0x58,
            0x29,
            0x0c,
            0,
        );

        if self
            .game_state
            .messaging
            .select_file_menu
            .is_name_scrolling()
        {
            return;
        }

        if self.game_state.player.follower_link.filtered_joypad_h() & 0x10 == 0 {
            if (self.game_state.player.follower_link.filtered_joypad_h() & 0xc0
                | self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
                == 0
            {
                return;
            }

            self.set_sound_effect_1(0x2b);
            let table_index = self
                .game_state
                .messaging
                .select_file_menu
                .name_column_usize()
                + self.game_state.messaging.select_file_menu.name_row_usize() * 0x20;
            let t = NAME_PLAYER_CHAR_TILE_BY_CURSOR[table_index];
            if t == 0x5a {
                self.move_select_file_name_slot_left_wrapped();
                return;
            } else if t == 0x44 {
                self.move_select_file_name_slot_right_wrapped();
                return;
            } else if t != 0x6f {
                let name_slot = self.game_state.messaging.select_file_menu.name_slot_usize();
                let p = name_slot * 2 + self.game_state.ending.attract_scene.legend_ctr() as usize;
                let chr = ((t as u16 & 0xfff0) * 2) + (t as u16 & 0x0f);
                write_le_u16(&mut self.sram, p + KSRM_OFFS_NAME, chr);
                self.name_file_draw_selected_character(name_slot, chr);
                self.move_select_file_name_slot_right_wrapped();
                return;
            }
        }

        let name_base = self.game_state.ending.attract_scene.legend_ctr() as usize;
        if (0..6).all(|i| read_le_u16(&self.sram, name_base + KSRM_OFFS_NAME + i * 2) == 0x00a9) {
            self.set_sound_effect_1(0x3c);
            return;
        }

        let cursor = self.game_state.messaging.select_file_menu.cursor();
        let slot_base = usize::from(cursor) * 0x500;
        self.set_selected_save_slot_from_cursor(cursor);
        write_le_u16(&mut self.sram, slot_base + 0x03e5, 0x55aa);
        write_le_u16(&mut self.sram, slot_base + 0x020c, 0xf000);
        write_le_u16(&mut self.sram, slot_base + 0x020e, 0xf000);
        write_le_u16(&mut self.sram, slot_base + KSRM_OFFS_DIED_COUNTER, 0xffff);

        let mut sram_init_normal = [0u8; 60];
        sram_init_normal[44] = 0x18;
        sram_init_normal[45] = 0x18;
        sram_init_normal[57] = 0xf8;
        self.sram[slot_base + 0x340..slot_base + 0x340 + sram_init_normal.len()]
            .copy_from_slice(&sram_init_normal);
        self.intro_fix_cksum_slot(slot_base);
        self.zelda_write_sram();
        self.return_to_file_select();
        self.set_irq_control_flag(0xff);
        self.set_sound_effect_1(0x2c);
    }

    pub(super) fn name_file_draw_selected_character(&mut self, k: usize, chr: u16) {
        const NAME_ENTRY_CHAR_VRAM_ADDRS: [u16; 6] = [0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8e];
        let a = NAME_ENTRY_CHAR_VRAM_ADDRS[k] | 0x6100;
        self.write_vram_upload_buffer_word(0, a.swap_bytes());
        self.write_vram_upload_buffer_word(2, 0x0100);
        self.write_vram_upload_buffer_word(4, 0x1800 | chr);
        self.write_vram_upload_buffer_word(6, a.wrapping_add(0x20).swap_bytes());
        self.write_vram_upload_buffer_word(8, 0x0100);
        self.write_vram_upload_buffer_word(10, (0x1800 | chr).wrapping_add(0x10));
        self.terminate_vram_upload_buffer_at(12);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn name_file_check_for_scroll_input_x(&mut self) {
        const NAME_ENTRY_X_SCROLL_WORDS: [u8; 14] = [
            0x01, 0x00, 0xff, 0x00, 0x20, 0x00, 0xff, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x83, 0x93,
        ];
        let a = self.game_state.player.follower_link.joypad1h_last() & 3;
        if a != 0 {
            let k = a.wrapping_sub(1);
            let table_index = k as usize * 2;
            self.set_select_file_name_scroll_x_direction(k);
            self.advance_select_file_name_scroll_x_step_by(1);
            let add = read_word_from_slice(&NAME_ENTRY_X_SCROLL_WORDS, table_index);
            let cmp = read_word_from_slice(&NAME_ENTRY_X_SCROLL_WORDS, 4 + table_index);
            let set = read_word_from_slice(&NAME_ENTRY_X_SCROLL_WORDS, 8 + table_index);
            let mut t = u16::from(self.game_state.messaging.select_file_menu.name_column())
                .wrapping_add(add);
            if t == cmp {
                t = set;
            }
            self.set_select_file_name_column(t as u8);
        }
    }

    pub(super) fn name_file_check_for_scroll_input_y(&mut self) {
        const NAME_ENTRY_Y_SCROLL_STEPS: [u8; 8] = [0x01, 0xff, 0x04, 0xff, 0x00, 0x03, 0x00, 0x00];

        let mut a = self.game_state.player.follower_link.joypad1h_last() & 0x0c;
        if a != 0 {
            let row = self.game_state.messaging.select_file_menu.name_row();
            if ((a << 1) | row) == 0x10 || ((a << 2) | row) == 0x13 {
                self.set_select_file_choice(1, a);
                return;
            }
            a >>= 2;
            let k = a.wrapping_sub(1) as usize;
            let mut t = row.wrapping_add(NAME_ENTRY_Y_SCROLL_STEPS[k]);
            if t == NAME_ENTRY_Y_SCROLL_STEPS[2 + k] {
                t = NAME_ENTRY_Y_SCROLL_STEPS[4 + k];
            }
            self.set_select_file_name_row(t);
            self.increment_select_file_name_scroll_y_step();
            self.set_select_file_choice(1, a);
        } else {
            self.set_select_file_choice(0, 0);
        }
    }

    pub(super) fn intro_fix_cksum(&mut self, s: *mut u8) {
        let bytes = unsafe { std::slice::from_raw_parts_mut(s, 0x500) };
        let mut sum = 0u16;
        for i in 0..0x27f {
            sum = sum.wrapping_add(read_word_from_slice(bytes, i * 2));
        }
        write_le_u16(bytes, 0x27f * 2, 0x5a5au16.wrapping_sub(sum));
    }

    pub(super) fn intro_fix_cksum_slot(&mut self, slot_base: usize) {
        let s = self.sram[slot_base..slot_base + 0x500].as_mut_ptr();
        self.intro_fix_cksum(s);
    }

    pub(super) fn return_to_file_select(&mut self) {
        self.set_main_module(1);
        self.set_submodule(1);
        self.set_subsubmodule(0);
        self.clear_select_file_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_entry_vertical_scroll_down_from_top_row_matches_c() {
        let mut state = ZeldaState::new();
        state.set_select_file_name_row(0);
        state.follower_link_state_mut().set_joypad1h_last(0x04);

        state.name_file_check_for_scroll_input_y();

        assert_eq!(state.game_state.messaging.select_file_menu.name_row(), 1);
        assert_eq!(
            state
                .game_state
                .messaging
                .select_file_menu
                .name_scroll_y_step(),
            1
        );
        assert_eq!(state.game_state.messaging.select_file_menu.choice(1), 1);
    }

    #[test]
    fn name_entry_vertical_scroll_down_released_settles_on_next_row() {
        let mut state = ZeldaState::new();
        state.set_select_file_name_row(0);
        state.set_select_file_name_cursor_y(0x83);
        state.follower_link_state_mut().set_joypad1h_last(0x04);

        state.name_file_check_for_scroll_input_y();
        state.follower_link_state_mut().set_joypad1h_last(0);

        for _ in 0..9 {
            state.name_file_do_the_naming();
        }

        assert_eq!(state.game_state.messaging.select_file_menu.name_row(), 1);
        assert_eq!(
            state.game_state.messaging.select_file_menu.name_cursor_y(),
            0x93
        );
        assert_eq!(
            state
                .game_state
                .messaging
                .select_file_menu
                .name_scroll_y_step(),
            0
        );
    }
}
