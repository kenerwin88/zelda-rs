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
            && self.ram[SAVED_MODULE_FOR_MENU] == 0
            && read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX) == 0x000a
        {
            let stall = match self.frame_control_view().submodule() {
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
                        "file-select-stall frame={} sub={} msg=0x{:04x} stall={} before={} seeded={} after={} return={} arr1={},{},{}",
                        self.frame_ctr_dbg,
                        self.frame_control_view().submodule(),
                        read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX),
                        stall,
                        before,
                        seeded,
                        after,
                        if after != 0 { 1 } else { 0 },
                        read_le_u16(&self.ram, SELECTFILE_ARR1),
                        read_le_u16(&self.ram, SELECTFILE_ARR1 + 2),
                        read_le_u16(&self.ram, SELECTFILE_ARR1 + 4),
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
                        "file-select-stall frame={} sub={} msg=0x{:04x} stall=0 before={} seeded={} after=0 return=0 arr1={},{},{}",
                        self.frame_ctr_dbg,
                        self.frame_control_view().submodule(),
                        read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX),
                        self.replay_reload_file_select_stall,
                        self.replay_reload_file_select_stall,
                        read_le_u16(&self.ram, SELECTFILE_ARR1),
                        read_le_u16(&self.ram, SELECTFILE_ARR1 + 2),
                        read_le_u16(&self.ram, SELECTFILE_ARR1 + 4),
                    );
                }
                self.replay_reload_file_select_stall = 0;
            }
        } else {
            if self.replay_reload_file_select_stall != 0
                && std::env::var_os("ZELDA3_REPLAY_FILE_SELECT_STALL_DUMP").is_some()
            {
                println!(
                    "file-select-stall frame={} sub={} msg=0x{:04x} stall=0 before={} seeded={} after=0 return=0 arr1={},{},{}",
                    self.frame_ctr_dbg,
                    self.frame_control_view().submodule(),
                    read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX),
                    self.replay_reload_file_select_stall,
                    self.replay_reload_file_select_stall,
                    read_le_u16(&self.ram, SELECTFILE_ARR1),
                    read_le_u16(&self.ram, SELECTFILE_ARR1 + 2),
                    read_le_u16(&self.ram, SELECTFILE_ARR1 + 4),
                );
            }
            self.replay_reload_file_select_stall = 0;
        }

        write_le_u16(&mut self.ram, BG3HOFS_COPY2, 0);
        write_le_u16(&mut self.ram, BG3VOFS_COPY2, 0);
        match self.frame_control_view().submodule() {
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
        self.ram[IS_NMI_THREAD_ACTIVE] = 0;
        self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0;
        self.ram[MUSIC_CONTROL] = 11;
        self.frame_control_view_mut().increment_submodule();
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
        self.ram[PALETTE_MAIN_INDOORS] = 6;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 6;
        self.palette_load_dungeon_set();
        self.palette_load_ow_bg3();
        self.ram[HUD_PALETTE] = 0;
        self.palette_load_hud();
        self.ram[HUD_CUR_ITEM] = 0;
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 1;
        self.ram[MAIN_TILE_THEME_INDEX] = 35;
        self.ram[AUX_TILE_THEME_INDEX] = 81;
        self.load_default_graphics();
        self.initialize_tilesets();
        self.load_file_select_graphics();
        self.intro_validate_sram();
        self.decompress_enemy_damage_subclasses();
    }

    pub(super) fn load_file_select_graphics(&mut self) {
        let data = self
            .decomp_spr_data(0x5e)
            .expect("file select sprite sheet 0x5e must decompress");
        let len = data.len().min(self.ram.len().saturating_sub(DECOMP_BUFFER));
        self.copy_to_ram(DECOMP_BUFFER, &data[..len]);
        self.do3_to_4_high_to_vram(0x5000, &data);

        let data = self
            .decomp_spr_data(0x5f)
            .expect("file select sprite sheet 0x5f must decompress");
        let len = data.len().min(self.ram.len().saturating_sub(DECOMP_BUFFER));
        self.copy_to_ram(DECOMP_BUFFER, &data[..len]);
        self.do3_to_4_high_to_vram(0x5400, &data);

        self.transfer_font_to_vram();

        let data = self
            .decomp_spr_data(0x6b)
            .expect("file select sprite sheet 0x6b must decompress");
        let len = data.len().min(self.ram.len().saturating_sub(DECOMP_BUFFER));
        self.copy_to_ram(DECOMP_BUFFER, &data[..len]);
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
        self.ram[SPRITE_Y_LO..SPRITE_Y_LO + 256 * 3].fill(0);
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
        self.ram[SELECTFILE_ARR1..SELECTFILE_ARR1 + 6].fill(0);
        self.file_select_erase_triforce();
    }

    pub(super) fn file_select_erase_triforce(&mut self) {
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0x80;
        self.enable_force_blank();
        self.erase_tile_maps_triforce();
        self.palette_load_for_file_select();
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.frame_control_view_mut().increment_submodule();
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
        self.ram[dst..dst + SELECT_FILE_GFX0.len()].copy_from_slice(&SELECT_FILE_GFX0);
        dst += SELECT_FILE_GFX0.len();

        let mut t = 0x1103u16;
        for _ in 0..18 {
            write_le_u16(&mut self.ram, dst, t.swap_bytes());
            dst += 2;
            write_le_u16(&mut self.ram, dst, 0x3240);
            dst += 2;
            write_le_u16(&mut self.ram, dst, 0x347f);
            dst += 2;
            t = t.wrapping_add(0x20);
        }
        self.ram[dst] = 0xff;
        self.frame_control_view_mut().increment_submodule();
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn select_file_func1(&mut self) -> usize {
        const TAB: [u16; 4] = [0x3581, 0x3582, 0x3591, 0x3592];
        let mut dst = VRAM_UPLOAD_DATA;
        write_le_u16(&mut self.ram, dst, 0x0010);
        dst += 2;
        write_le_u16(&mut self.ram, dst, 0xff07);
        dst += 2;
        for i in 0..1024 {
            write_le_u16(&mut self.ram, dst, TAB[((i & 0x20) >> 4) + (i & 1)]);
            dst += 2;
        }
        dst
    }

    pub(super) fn file_select_trigger_stripes_and_advance(&mut self) {
        self.ram[R16] = self.ram[SELECTFILE_VAR2];
        self.frame_control_view_mut().increment_submodule();
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 6;
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
        self.ram[VRAM_UPLOAD_DATA..VRAM_UPLOAD_DATA + data.len()].copy_from_slice(&data);
        self.ram[INIDISP_COPY] = 0x0f;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
        self.frame_control_view_mut().increment_submodule();
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 6;
    }

    pub(super) fn file_select_main(&mut self) {
        const FAERIE_Y: [u8; 5] = [0x4a, 0x6a, 0x8a, 0xaf, 0xbf];

        if self.ram[R16] < 3 {
            self.ram[SELECTFILE_VAR2] = self.ram[R16];
        }

        for k in 0..3 {
            if read_le_u16(&self.sram, k * 0x500 + 0x03e5) == 0x55aa {
                write_le_u16(&mut self.ram, SELECTFILE_ARR1 + k * 2, 1);
                self.select_file_func5_draw_oams(k);
                self.select_file_func6_draw_oams2(k);
                self.select_file_func17(k);
            }
        }

        let y = FAERIE_Y[self.ram[R16] as usize];
        self.file_select_draw_fairy(0x1c, y);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;

        let a = (self.ram[FILTERED_JOYPAD_L] & 0xc0 | self.ram[FILTERED_JOYPAD_H]) & 0xfc;
        if a & 0x2c != 0 {
            self.ram[SOUND_EFFECT_2] = 0x20;
            if a & 8 != 0 {
                self.ram[R16] = self.ram[R16].wrapping_sub(1);
                if self.ram[R16] & 0x80 != 0 {
                    self.ram[R16] = 4;
                }
            } else {
                self.ram[R16] = self.ram[R16].wrapping_add(1);
                if self.ram[R16] == 5 {
                    self.ram[R16] = 0;
                }
            }
        } else if a != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
            if self.ram[R16] < 3 {
                self.ram[R17] = 0;
                if read_le_u16(&self.ram, SELECTFILE_ARR1 + self.ram[R16] as usize * 2) == 0 {
                    self.frame_control_view_mut().set_main_module(4);
                    self.frame_control_view_mut().set_submodule(0);
                    self.frame_control_view_mut().set_subsubmodule(0);
                } else {
                    self.ram[MUSIC_CONTROL] = 0xf1;
                    let slot = self.ram[R16].wrapping_mul(2).wrapping_add(2);
                    write_le_u16(&mut self.sram, SRM_VAR1, slot as u16);
                    let save_offset = self.ram[R16] as u16 * 0x500;
                    write_le_u16(&mut self.ram, R0, save_offset);
                    self.CopySaveToWRAM();
                }
            } else if (read_le_u16(&self.ram, SELECTFILE_ARR1)
                | read_le_u16(&self.ram, SELECTFILE_ARR1 + 2)
                | read_le_u16(&self.ram, SELECTFILE_ARR1 + 4))
                != 0
            {
                let main_module = if self.ram[R16] == 3 { 2 } else { 3 };
                self.frame_control_view_mut().set_main_module(main_module);
                self.ram[R16] = 0;
                self.frame_control_view_mut().set_submodule(0);
                self.frame_control_view_mut().set_subsubmodule(0);
            } else {
                self.ram[SOUND_EFFECT_1] = 0x3c;
            }
        }
    }

    pub(super) fn file_select_draw_fairy(&mut self, x: u8, y: u8) {
        let charnum = if self.ram[FRAME_COUNTER] & 8 != 0 {
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

        write_le_u16(&mut self.ram, LINK_DMA_GRAPHICS_INDEX, 0x116 * 2);
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
            self.ram[OAM_BUF + oam * 4 + 1] = 0xf0;
            self.ram[OAM_BUF + (oam + 1) * 4 + 1] = 0xf0;
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
            self.ram[OAM_BUF + (oam + 2) * 4 + 1] = 0xf0;
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

        let mut dst = VRAM_UPLOAD_DATA + NAME_VRAM_OFFS[k];
        for i in 0..6 {
            let t =
                read_le_u16(&self.sram, sram_base + KSRM_OFFS_NAME + i * 2).wrapping_add(0x1800);
            write_le_u16(&mut self.ram, dst, t);
            write_le_u16(&mut self.ram, dst + 42, t.wrapping_add(0x10));
            dst += 2;
        }

        let mut health = self.sram[sram_base + KSRM_OFFS_HEALTH] >> 3;
        let mut dst = VRAM_UPLOAD_DATA + HEALTH_VRAM_OFFS[k];
        let dst_org = dst;
        let mut row = 10u8;
        loop {
            write_le_u16(&mut self.ram, dst, 0x0520);
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
        self.file_select_draw_fairy(0x1c, FAERIE_Y[self.ram[R16] as usize]);

        let mut k = self.ram[R16];
        if self.ram[FILTERED_JOYPAD_H] & 0x2c != 0 {
            k = if self.ram[FILTERED_JOYPAD_H] & 0x24 != 0 {
                k.wrapping_add(1)
            } else {
                k.wrapping_sub(1)
            };
            self.ram[R16] = k & 1;
            self.ram[SOUND_EFFECT_2] = 0x20;
        }

        let a = (self.ram[FILTERED_JOYPAD_L] & 0xc0 | self.ram[FILTERED_JOYPAD_H]) & 0xd0;
        if a != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
            if self.ram[R16] == 0 {
                self.ram[SOUND_EFFECT_2] = 0x22;
                self.ram[SOUND_EFFECT_1] = 0;
                let k = self.frame_control_view().subsubmodule() as usize;
                write_le_u16(&mut self.ram, SELECTFILE_ARR1 + k * 2, 0);
                let base = k * 0x500;
                self.sram[base..base + 0x500].fill(0);
                self.sram[base + 0x0f00..base + 0x1400].fill(0);
                self.zelda_write_sram();
            }
            self.return_to_file_select();
            self.frame_control_view_mut().set_subsubmodule(0);
        }
    }

    pub(super) fn module02_copy_file(&mut self) {
        self.ram[SELECTFILE_VAR2] = 0;
        match self.frame_control_view().submodule() {
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
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 7;
        self.frame_control_view_mut().increment_submodule();
        self.ram[INIDISP_COPY] = 0x0f;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
        let mut i = 0usize;
        while read_le_u16(&self.ram, SELECTFILE_ARR1 + i * 2) == 0 {
            i += 1;
        }
        self.ram[R16] = i as u8;
    }

    pub(super) fn copy_file_choose_selection(&mut self) {
        self.copy_file_selection_and_blinker();
        if self.frame_control_view().submodule() == 3 && self.ram[FRAME_COUNTER] & 0x30 == 0 {
            self.file_picker_delete_header_stripe();
        }
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn copy_file_choose_target(&mut self) {
        self.copy_file_target_selection_and_blink();
        if self.frame_control_view().submodule() == 4 && self.ram[FRAME_COUNTER] & 0x30 == 0 {
            self.file_picker_delete_header_stripe();
        }
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn copy_file_confirm_selection(&mut self) {
        self.copy_file_handle_confirmation();
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn file_picker_delete_header_stripe(&mut self) {
        const DST: [usize; 2] = [4, 0x1e];
        for j in (0..2).rev() {
            let dst = VRAM_UPLOAD_DATA + DST[j];
            for i in 0..11 {
                write_le_u16(&mut self.ram, dst + i * 2, 0x00a9);
            }
        }
    }

    pub(super) fn copy_file_selection_and_blinker(&mut self) {
        const TAB: [u8; 173] = [
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
        const TAB1: [u8; 73] = [
            0x61, 0x67, 0x40, 0xe, 0xa9, 0, 0x61, 0x87, 0x40, 0xe, 0xa9, 0, 0x61, 0xc7, 0x40, 0xe,
            0xa9, 0, 0x61, 0xe7, 0x40, 0xe, 0xa9, 0, 0x11, 0x30, 0, 1, 0x83, 0x35, 0x11, 0x31,
            0x40, 0x14, 0x85, 0x35, 0x11, 0x3c, 0, 1, 0x84, 0x35, 0x11, 0x50, 0xc0, 0xe, 0x86,
            0x35, 0x11, 0x5c, 0xc0, 0xe, 0x96, 0x35, 0x12, 0x50, 0, 1, 0x93, 0x35, 0x12, 0x51,
            0x40, 0x14, 0x95, 0x35, 0x12, 0x5c, 0, 1, 0x94, 0x35, 0xff,
        ];
        const DST: [usize; 3] = [0x3c, 0x64, 0x8c];
        const FAERIE_X: [u8; 4] = [36, 36, 36, 28];
        const FAERIE_Y: [u8; 4] = [87, 111, 135, 191];

        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, 0x00ac);
        self.ram[VRAM_UPLOAD_DATA..VRAM_UPLOAD_DATA + TAB.len()].copy_from_slice(&TAB);

        for k in 0..3 {
            if read_le_u16(&self.ram, SELECTFILE_ARR1 + k * 2) & 1 != 0 {
                let mut dst = VRAM_UPLOAD_DATA + DST[k];
                for i in 0..6 {
                    let t = read_le_u16(&self.sram, k * 0x500 + KSRM_OFFS_NAME + i * 2)
                        .wrapping_add(0x1800);
                    write_le_u16(&mut self.ram, dst, t);
                    write_le_u16(&mut self.ram, dst + 20, t.wrapping_add(0x10));
                    dst += 2;
                }
            }
        }
        let r16 = self.ram[R16] as usize;
        self.file_select_draw_fairy(FAERIE_X[r16], FAERIE_Y[r16]);

        let a = (self.ram[FILTERED_JOYPAD_L] & 0xc0 | self.ram[FILTERED_JOYPAD_H]) & 0xfc;
        if a & 0x2c != 0 {
            let mut k = self.ram[R16];
            if a & 8 != 0 {
                loop {
                    k = k.wrapping_sub(1);
                    if k & 0x80 != 0 {
                        k = 3;
                        break;
                    }
                    if read_le_u16(&self.ram, SELECTFILE_ARR1 + k as usize * 2) != 0 {
                        break;
                    }
                }
            } else {
                loop {
                    k = k.wrapping_add(1);
                    if k >= 4 {
                        k = 0;
                    }
                    if k == 3 || read_le_u16(&self.ram, SELECTFILE_ARR1 + k as usize * 2) != 0 {
                        break;
                    }
                }
            }
            self.ram[R16] = k;
            self.ram[SOUND_EFFECT_2] = 0x20;
        } else if a != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
            if self.ram[R16] == 3 {
                self.return_to_file_select();
                return;
            }
            let r16 = self.ram[R16];
            write_le_u16(&mut self.ram, SELECTFILE_VAR6, r16 as u16 * 2);
            self.ram[VRAM_UPLOAD_DATA + 52..VRAM_UPLOAD_DATA + 52 + TAB1.len()]
                .copy_from_slice(&TAB1);
            if self.ram[R16] != 2 {
                let dst = VRAM_UPLOAD_DATA + self.ram[R16] as usize * 12;
                write_le_u16(&mut self.ram, dst + 52, 0x2762);
                write_le_u16(&mut self.ram, dst + 58, 0x4762);
            }
            self.frame_control_view_mut().increment_submodule();
            self.ram[R16] = 0;
        }
    }

    pub(super) fn copy_file_target_selection_and_blink(&mut self) {
        let mut k = 1usize;
        let mut t = 4i32;
        loop {
            if t as u16 != read_le_u16(&self.ram, SELECTFILE_VAR6) {
                self.ram[SELECTFILE_ARR2 + k] = t as u8;
                k = k.wrapping_sub(1);
            }
            t -= 2;
            if t < 0 {
                break;
            }
        }

        const TAB0: [u8; 133] = [
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
        const TAB2: [u8; 49] = [
            0x61, 0xb4, 0x40, 0xe, 0xa9, 0, 0x61, 0xd4, 0x40, 0xe, 0xa9, 0, 0x62, 0xc6, 0, 0xd, 2,
            0x18, 0xe, 0x18, 0xf, 0x18, 0x28, 0x18, 0xa9, 0x18, 0xe, 0x18, 0xa, 0x18, 0x62, 0xe6,
            0, 0xd, 0x12, 0x18, 0x1e, 0x18, 0x1f, 0x18, 0x38, 0x18, 0xa9, 0x18, 0x1e, 0x18, 0x1a,
            0x18, 0xff,
        ];
        const FAERIE_X: [u8; 3] = [0x8c, 0x8c, 0x1c];
        const FAERIE_Y: [u8; 3] = [0x67, 0x7f, 0xbf];
        const DST: [usize; 2] = [0x38, 0x60];
        const TAB1: [u16; 3] = [0x18e7, 0x18e8, 0x18e9];

        self.ram[VRAM_UPLOAD_DATA..VRAM_UPLOAD_DATA + TAB0.len()].copy_from_slice(&TAB0);
        let mut j = 0usize;
        for k in 0..3 {
            if k * 2 == read_le_u16(&self.ram, SELECTFILE_VAR6) as usize {
                continue;
            }
            let mut dst = VRAM_UPLOAD_DATA + DST[j];
            j += 1;
            let t = TAB1[k];
            write_le_u16(&mut self.ram, dst, t);
            write_le_u16(&mut self.ram, dst + 20, t.wrapping_add(0x10));
            dst += 4;
            if read_le_u16(&self.ram, SELECTFILE_ARR1 + k * 2) != 0 {
                for i in 0..6 {
                    let t = read_le_u16(&self.sram, k * 0x500 + KSRM_OFFS_NAME + i * 2)
                        .wrapping_add(0x1800);
                    write_le_u16(&mut self.ram, dst, t);
                    write_le_u16(&mut self.ram, dst + 20, t.wrapping_add(0x10));
                    dst += 2;
                }
            }
        }
        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, 132);
        let r16 = self.ram[R16] as usize;
        self.file_select_draw_fairy(FAERIE_X[r16], FAERIE_Y[r16]);

        let a = (self.ram[FILTERED_JOYPAD_L] & 0xc0 | self.ram[FILTERED_JOYPAD_H]) & 0xfc;
        if a & 0x2c != 0 {
            let mut k = self.ram[R16];
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
            self.ram[R16] = k;
            self.ram[SOUND_EFFECT_2] = 0x20;
        } else if a != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
            if self.ram[R16] == 2 {
                self.return_to_file_select();
                self.ram[R16] = 0;
                return;
            }
            let r16 = self.ram[R16] as usize;
            let target = self.ram[SELECTFILE_ARR2 + r16] as u16;
            write_le_u16(&mut self.ram, R18, target);
            self.ram[VRAM_UPLOAD_DATA + 52..VRAM_UPLOAD_DATA + 52 + TAB2.len()]
                .copy_from_slice(&TAB2);
            if self.ram[R16] == 0 {
                write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 52, 0x1462);
                write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 58, 0x3462);
            }
            self.frame_control_view_mut().increment_submodule();
            self.ram[R16] = 0;
        }
    }

    pub(super) fn copy_file_handle_confirmation(&mut self) {
        const FAERIE_Y: [u8; 2] = [0xaf, 0xbf];
        self.file_select_draw_fairy(0x1c, FAERIE_Y[self.ram[R16] as usize]);

        let a = (self.ram[FILTERED_JOYPAD_L] & 0xc0 | self.ram[FILTERED_JOYPAD_H]) & 0xfc;
        if a & 0x2c != 0 {
            self.ram[SOUND_EFFECT_2] = 0x20;
            if a & 0x24 != 0 {
                self.ram[R16] = self.ram[R16].wrapping_add(1);
                if self.ram[R16] >= 2 {
                    self.ram[R16] = 0;
                }
            } else {
                self.ram[R16] = self.ram[R16].wrapping_sub(1);
                if self.ram[R16] & 0x80 != 0 {
                    self.ram[R16] = 1;
                }
            }
        } else if a != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
            if self.ram[R16] == 0 {
                let dst_slot = (read_le_u16(&self.ram, R18) >> 1) as usize;
                let src_slot = (read_le_u16(&self.ram, SELECTFILE_VAR6) >> 1) as usize;
                let src = self.sram[src_slot * 0x500..src_slot * 0x500 + 0x500].to_vec();
                self.sram[dst_slot * 0x500..dst_slot * 0x500 + 0x500].copy_from_slice(&src);
                write_le_u16(&mut self.ram, SELECTFILE_ARR1 + dst_slot * 2, 1);
                self.zelda_write_sram();
            }
            self.return_to_file_select();
            self.ram[R16] = 0;
        }
    }

    pub(super) fn module03_kill_file(&mut self) {
        match self.frame_control_view().submodule() {
            0 => self.file_select_erase_triforce(),
            1 => self.module_erase_file_1(),
            2 => self.kill_file_set_up(),
            3 => self.kill_file_handle_selection(),
            4 => self.kill_file_handle_confirmation(),
            _ => {}
        }
    }

    pub(super) fn kill_file_set_up(&mut self) {
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 8;
        self.frame_control_view_mut().increment_submodule();
        self.ram[INIDISP_COPY] = 0x0f;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
        let mut i = 0usize;
        while read_le_u16(&self.ram, SELECTFILE_ARR1 + i * 2) == 0 {
            i += 1;
        }
        self.ram[R16] = i as u8;
    }

    pub(super) fn kill_file_handle_selection(&mut self) {
        if self.ram[R16] < 3 {
            self.ram[SELECTFILE_VAR2] = self.ram[R16];
        }
        self.kill_file_choose_target();
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn kill_file_handle_confirmation(&mut self) {
        self.select_file_func16();
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn kill_file_choose_target(&mut self) {
        const FAERIE_X: [u8; 4] = [36, 36, 36, 28];
        const FAERIE_Y: [u8; 4] = [103, 127, 151, 191];
        const TAB2: [u8; 101] = [
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
        self.ram[VRAM_UPLOAD_DATA..VRAM_UPLOAD_DATA + data.len()].copy_from_slice(&data);
        for k in 0..3 {
            if read_le_u16(&self.ram, SELECTFILE_ARR1 + k * 2) != 0 {
                self.select_file_func17(k);
            }
        }

        let r16 = self.ram[R16] as usize;
        self.file_select_draw_fairy(FAERIE_X[r16], FAERIE_Y[r16]);

        let mut k = self.ram[R16];
        if self.ram[FILTERED_JOYPAD_H] & 0x2c != 0 {
            if self.ram[FILTERED_JOYPAD_H] & 0x24 == 0 {
                loop {
                    k = k.wrapping_sub(1);
                    if k & 0x80 != 0 {
                        k = 3;
                        break;
                    }
                    if read_le_u16(&self.ram, SELECTFILE_ARR1 + k as usize * 2) != 0 {
                        break;
                    }
                }
            } else {
                loop {
                    k = k.wrapping_add(1);
                    if k >= 4 {
                        k = 0;
                    }
                    if k == 3 || read_le_u16(&self.ram, SELECTFILE_ARR1 + k as usize * 2) != 0 {
                        break;
                    }
                }
            }
            self.ram[SOUND_EFFECT_2] = 0x20;
        }
        self.ram[R16] = k;

        let a = (self.ram[FILTERED_JOYPAD_L] & 0xc0 | self.ram[FILTERED_JOYPAD_H]) & 0xd0;
        if a != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
            if k == 3 {
                self.return_to_file_select();
                return;
            }
            self.ram[VRAM_UPLOAD_DATA..VRAM_UPLOAD_DATA + TAB2.len()].copy_from_slice(&TAB2);
            self.frame_control_view_mut().increment_submodule();
            if self.ram[R16] != 2 {
                let dst = VRAM_UPLOAD_DATA + self.ram[R16] as usize * 12;
                write_le_u16(&mut self.ram, dst, 0x6762);
                write_le_u16(&mut self.ram, dst + 6, 0x8762);
            }
            let subsubmodule = self.ram[R16];
            self.frame_control_view_mut().set_subsubmodule(subsubmodule);
            self.ram[R16] = 0;
        }
    }

    pub(super) fn module04_name_file(&mut self) {
        match self.frame_control_view().submodule() {
            0 => self.name_file_erase_save(),
            1 => self.module_name_player_1(),
            2 => self.module_name_player_2(),
            3 => self.name_file_do_the_naming(),
            _ => {}
        }
    }

    pub(super) fn name_file_erase_save(&mut self) {
        self.file_select_erase_triforce();
        self.ram[IRQ_FLAG] = 1;
        self.ram[SELECTFILE_VAR3] = 0;
        self.ram[SELECTFILE_VAR4] = 0;
        self.ram[SELECTFILE_VAR5] = 0;
        self.ram[SELECTFILE_ARR2] = 0;
        self.ram[SELECTFILE_VAR6] = 0;
        self.ram[SELECTFILE_VAR7] = 0x83;
        write_le_u16(&mut self.ram, SELECTFILE_VAR8, 0x01f0);
        write_le_u16(&mut self.ram, BG3HOFS_COPY2, 0);

        let offs = self.ram[R16] as usize * 0x500;
        write_le_u16(&mut self.ram, ATTRACT_LEGEND_CTR, offs as u16);
        self.sram[offs..offs + 0x500].fill(0);
        for i in 0..6 {
            write_le_u16(&mut self.sram, offs + KSRM_OFFS_NAME + i * 2, 0x00a9);
        }
    }

    pub(super) fn module_name_player_1(&mut self) {
        let dst = self.select_file_func1();
        write_le_u16(&mut self.ram, dst, 0xffff);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn module_name_player_2(&mut self) {
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 5;
        self.frame_control_view_mut().increment_submodule();
        self.ram[INIDISP_COPY] = 0x0f;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
    }

    pub(super) fn name_file_do_the_naming(&mut self) {
        const NAME_PLAYER_TAB1: [i16; 26] = [
            -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, -2, 2, -2, 2, -2, 2, -2, 2, -4,
            4,
        ];
        const NAME_PLAYER_TAB2: [u8; 4] = [0x83, 0x93, 0xa3, 0xb3];
        const NAME_PLAYER_TAB0: [u16; 40] = [
            0x1f0, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0,
            0xe0, 0xf0, 0x100, 0x110, 0x120, 0x130, 0x140, 0x150, 0x160, 0x170, 0x180, 0x190,
            0x1a0, 0x1b0, 0x1c0, 0x1d0, 0x1e0, 0x0001, 0x00ff, 0x0020, 0x00ff, 0x0000, 0x001f,
            0x9383, 0xb3a3,
        ];
        const NAME_PLAYER_X: [u8; 6] = [31, 47, 63, 79, 95, 111];
        const NAME_PLAYER_TAB3: [u8; 128] = [
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
            let mut j = self.ram[SELECTFILE_VAR9];
            if j == 0 {
                self.name_file_check_for_scroll_input_x();
                break;
            }
            if j != 0x31 {
                self.ram[SELECTFILE_VAR9] = self.ram[SELECTFILE_VAR9].wrapping_add(4);
            }
            j = j.wrapping_sub(1);
            let target = NAME_PLAYER_TAB0[self.ram[SELECTFILE_VAR3] as usize];
            if target == read_le_u16(&self.ram, SELECTFILE_VAR8) {
                self.ram[SELECTFILE_VAR9] = if self.ram[JOYPAD1H_LAST] & 3 != 0 {
                    0x30
                } else {
                    0
                };
                self.name_file_check_for_scroll_input_x();
                continue;
            }
            if self.ram[SELECTFILE_VAR10] == 0 {
                j = j.wrapping_add(2);
            }
            let delta = read_name_player_tab1_byte_word(&NAME_PLAYER_TAB1, j as usize);
            let next = read_le_u16(&self.ram, SELECTFILE_VAR8).wrapping_add(delta) & 0x01ff;
            write_le_u16(&mut self.ram, SELECTFILE_VAR8, next);
            break;
        }

        loop {
            if self.ram[SELECTFILE_VAR11] == 0 {
                self.name_file_check_for_scroll_input_y();
                break;
            }
            let target_y = NAME_PLAYER_TAB2[self.ram[SELECTFILE_VAR5] as usize];
            let diff = self.ram[SELECTFILE_VAR7].wrapping_sub(target_y);
            if diff != 0 {
                self.ram[SELECTFILE_VAR7] = if diff & 0x80 != 0 {
                    self.ram[SELECTFILE_VAR7].wrapping_add(2)
                } else {
                    self.ram[SELECTFILE_VAR7].wrapping_sub(2)
                };
                break;
            }
            self.ram[SELECTFILE_VAR11] = 0;
            self.name_file_check_for_scroll_input_y();
        }

        for i in 0..26 {
            self.set_oam_plain(
                i,
                0x18u8.wrapping_add((i as u8).wrapping_mul(8)),
                self.ram[SELECTFILE_VAR7],
                0x2e,
                0x3c,
                0,
            );
        }
        self.set_oam_plain(
            26,
            NAME_PLAYER_X[self.ram[SELECTFILE_VAR4] as usize],
            0x58,
            0x29,
            0x0c,
            0,
        );

        if (self.ram[SELECTFILE_VAR9] | self.ram[SELECTFILE_VAR11]) != 0 {
            return;
        }

        if self.ram[FILTERED_JOYPAD_H] & 0x10 == 0 {
            if (self.ram[FILTERED_JOYPAD_H] & 0xc0 | self.ram[FILTERED_JOYPAD_L] & 0xc0) == 0 {
                return;
            }

            self.ram[SOUND_EFFECT_1] = 0x2b;
            let table_index =
                self.ram[SELECTFILE_VAR3] as usize + self.ram[SELECTFILE_VAR5] as usize * 0x20;
            let t = NAME_PLAYER_TAB3[table_index];
            if t == 0x5a {
                if self.ram[SELECTFILE_VAR4] == 0 {
                    self.ram[SELECTFILE_VAR4] = 5;
                } else {
                    self.ram[SELECTFILE_VAR4] = self.ram[SELECTFILE_VAR4].wrapping_sub(1);
                }
                return;
            } else if t == 0x44 {
                self.ram[SELECTFILE_VAR4] = self.ram[SELECTFILE_VAR4].wrapping_add(1);
                if self.ram[SELECTFILE_VAR4] == 6 {
                    self.ram[SELECTFILE_VAR4] = 0;
                }
                return;
            } else if t != 0x6f {
                let p = self.ram[SELECTFILE_VAR4] as usize * 2
                    + read_le_u16(&self.ram, ATTRACT_LEGEND_CTR) as usize;
                let chr = ((t as u16 & 0xfff0) * 2) + (t as u16 & 0x0f);
                write_le_u16(&mut self.sram, p + KSRM_OFFS_NAME, chr);
                self.name_file_draw_selected_character(self.ram[SELECTFILE_VAR4] as usize, chr);
                self.ram[SELECTFILE_VAR4] = self.ram[SELECTFILE_VAR4].wrapping_add(1);
                if self.ram[SELECTFILE_VAR4] == 6 {
                    self.ram[SELECTFILE_VAR4] = 0;
                }
                return;
            }
        }

        let name_base = read_le_u16(&self.ram, ATTRACT_LEGEND_CTR) as usize;
        if (0..6).all(|i| read_le_u16(&self.sram, name_base + KSRM_OFFS_NAME + i * 2) == 0x00a9) {
            self.ram[SOUND_EFFECT_1] = 0x3c;
            return;
        }

        let slot_base = self.ram[R16] as usize * 0x500;
        write_le_u16(&mut self.sram, SRM_VAR1, self.ram[R16] as u16 * 2 + 2);
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
        self.ram[IRQ_FLAG] = 0xff;
        self.ram[SOUND_EFFECT_1] = 0x2c;
    }

    pub(super) fn name_file_draw_selected_character(&mut self, k: usize, chr: u16) {
        const TAB: [u16; 6] = [0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8e];
        let a = TAB[k] | 0x6100;
        write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA, a.swap_bytes());
        write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 2, 0x0100);
        write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 4, 0x1800 | chr);
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_DATA + 6,
            a.wrapping_add(0x20).swap_bytes(),
        );
        write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 8, 0x0100);
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_DATA + 10,
            (0x1800 | chr).wrapping_add(0x10),
        );
        self.ram[VRAM_UPLOAD_DATA + 12] = 0xff;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn name_file_check_for_scroll_input_x(&mut self) {
        const TABLE_BYTES: [u8; 14] = [
            0x01, 0x00, 0xff, 0x00, 0x20, 0x00, 0xff, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x83, 0x93,
        ];
        let a = self.ram[JOYPAD1H_LAST] & 3;
        if a != 0 {
            let k = a.wrapping_sub(1);
            let table_index = k as usize * 2;
            self.ram[SELECTFILE_VAR10] = k;
            self.ram[SELECTFILE_VAR9] = self.ram[SELECTFILE_VAR9].wrapping_add(1);
            let add = read_word_from_slice(&TABLE_BYTES, table_index);
            let cmp = read_word_from_slice(&TABLE_BYTES, 4 + table_index);
            let set = read_word_from_slice(&TABLE_BYTES, 8 + table_index);
            let mut t = (self.ram[SELECTFILE_VAR3] as u16).wrapping_add(add);
            if t == cmp {
                t = set;
            }
            self.ram[SELECTFILE_VAR3] = t as u8;
        }
    }

    pub(super) fn name_file_check_for_scroll_input_y(&mut self) {
        const TABLE_BYTES: [u8; 8] = [0x01, 0xff, 0x04, 0xff, 0x00, 0x03, 0x00, 0x00];

        let mut a = self.ram[JOYPAD1H_LAST] & 0x0c;
        if a != 0 {
            if ((a << 1) | self.ram[SELECTFILE_VAR5]) == 0x10
                || ((a << 2) | self.ram[SELECTFILE_VAR5]) == 0x13
            {
                self.ram[SELECTFILE_ARR2 + 1] = a;
                return;
            }
            a >>= 2;
            let k = a.wrapping_sub(1) as usize;
            let mut t = self.ram[SELECTFILE_VAR5].wrapping_add(TABLE_BYTES[k]);
            if t == TABLE_BYTES[2 + k] {
                t = TABLE_BYTES[4 + k];
            }
            self.ram[SELECTFILE_VAR5] = t;
            self.ram[SELECTFILE_VAR11] = self.ram[SELECTFILE_VAR11].wrapping_add(1);
            self.ram[SELECTFILE_ARR2 + 1] = a;
        } else {
            self.ram[SELECTFILE_ARR2] = 0;
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
        self.frame_control_view_mut().set_main_module(1);
        self.frame_control_view_mut().set_submodule(1);
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[R16] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_entry_vertical_scroll_down_from_top_row_matches_c() {
        let mut state = ZeldaState::new();
        state.ram[SELECTFILE_VAR5] = 0;
        state.ram[JOYPAD1H_LAST] = 0x04;

        state.name_file_check_for_scroll_input_y();

        assert_eq!(state.ram[SELECTFILE_VAR5], 1);
        assert_eq!(state.ram[SELECTFILE_VAR11], 1);
        assert_eq!(state.ram[SELECTFILE_ARR2 + 1], 1);
    }

    #[test]
    fn name_entry_vertical_scroll_down_released_settles_on_next_row() {
        let mut state = ZeldaState::new();
        state.ram[SELECTFILE_VAR5] = 0;
        state.ram[SELECTFILE_VAR7] = 0x83;
        state.ram[JOYPAD1H_LAST] = 0x04;

        state.name_file_check_for_scroll_input_y();
        state.ram[JOYPAD1H_LAST] = 0;

        for _ in 0..9 {
            state.name_file_do_the_naming();
        }

        assert_eq!(state.ram[SELECTFILE_VAR5], 1);
        assert_eq!(state.ram[SELECTFILE_VAR7], 0x93);
        assert_eq!(state.ram[SELECTFILE_VAR11], 0);
    }
}
