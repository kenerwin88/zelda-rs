// Methods ported from zelda3/src/misc.c and included inside ZeldaState.

use super::*;

const DUNG_TORCH_TIMERS_MISC: usize = 0x04f0;
const DUNG_TORCH_DATA_MISC: usize = 0x0fb40;
const VRAM_UPLOAD_TILE_BUF_MISC: usize = 0x1100;
const OVERWORLD_TILEATTR_MISC: usize = 0x2000;
const SAVE_OW_EVENT_INFO_MISC: usize = 0x0f280;

fn calculate_sfx_pan(x: u16) -> u8 {
    ZeldaState::calculate_sfx_pan_with_scroll(x, 0)
}

fn receive_item_tab1_misc(item: u8) -> u8 {
    const TAB1: [u8; 76] = [
        0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
    ];
    TAB1.get(item as usize).copied().unwrap_or(0)
}

fn receive_item_tab2_misc(item: u8) -> i8 {
    const TAB2: [i8; 76] = [
        -5, -5, -5, -5, -5, -4, -4, -5, -5, -4, -4, -4, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4,
        -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -5, -4, -4, -4, -4, -4, -4, -2, -4, -4, -4,
        -4, -4, -4, -4, -4, -4, -2, -2, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -2, -2, -4,
        -2, -4, -4, -4, -5, -4, -4,
    ];
    TAB2.get(item as usize).copied().unwrap_or(0)
}

fn receive_item_tab3_misc(item: u8) -> u8 {
    const TAB3: [u8; 76] = [
        4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
    ];
    TAB3.get(item as usize).copied().unwrap_or(0)
}

fn receive_item_gfx_misc(item: u8) -> u8 {
    const GFX: [u8; 76] = [
        6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a,
        0x1c, 0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28,
        0x27, 4, 4, 0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24,
        0x24, 0x24, 0x23, 0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2,
        0x32, 0x36, 0x37, 0x2c, 6, 0x0c, 0x38,
    ];
    GFX.get(item as usize).copied().unwrap_or(0)
}

fn memory_location_to_give_item_to_misc(item: u8) -> usize {
    const MEMORY_LOCATIONS: [usize; 76] = [
        0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342,
        0xf340, 0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350,
        0xf35c, 0xf36b, 0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a,
        0xf34d, 0xf35b, 0xf35b, 0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c,
        0xf35c, 0xf35c, 0xf36d, 0xf36e, 0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360,
        0xf374, 0xf374, 0xf374, 0xf340, 0xf340, 0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360,
        0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360, 0xf35c, 0xf359, 0xf34c, 0xf355,
    ];
    MEMORY_LOCATIONS.get(item as usize).copied().unwrap_or(0)
}

fn value_to_give_item_to_misc(item: u8) -> u8 {
    const VALUES: [u8; 76] = [
        1, 2, 3, 4, 1, 2, 3, 1, 1, 1, 1, 1, 1, 2, 0xff, 1, 1, 1, 1, 1, 2, 1, 0xff, 0xff, 1, 1, 2,
        1, 2, 1, 1, 1, 0xff, 1, 0xff, 2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 2, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfb, 0xec, 0xff, 0xff, 0xff, 1, 3, 0xff, 0xff,
        0xff, 0xff, 0x9c, 0xce, 0xff, 1, 10, 0xff, 0xff, 0xff, 0xff, 1, 3, 1,
    ];
    VALUES.get(item as usize).copied().unwrap_or(0xff)
}

impl ZeldaState {
    #[track_caller]
    pub(super) fn get_random_number(&mut self) -> u8 {
        let before = self.ram[RNG_SEED];
        let mut t = self.ram[RNG_SEED].wrapping_add(self.ram[FRAME_COUNTER]);
        t = if t & 1 != 0 { t >> 1 } else { (t >> 1) ^ 0xb8 };
        self.ram[RNG_SEED] = t;
        let trace_rng = std::env::var_os("ZELDA3_TRACE_RNG").is_some();
        let trace_frame_matches = std::env::var("ZELDA3_TRACE_RNG_FRAME")
            .ok()
            .and_then(|value| {
                let trimmed = value.trim();
                if let Some(hex) = trimmed.strip_prefix("0x") {
                    u8::from_str_radix(hex, 16).ok()
                } else {
                    trimmed.parse::<u8>().ok()
                }
            })
            .is_none_or(|frame| frame == self.ram[FRAME_COUNTER]);
        if trace_rng && trace_frame_matches {
            let loc = std::panic::Location::caller();
            eprintln!(
                "R rng fc={} before=0x{:02x} after=0x{:02x} site={}:{} link=0x{:04x},0x{:04x}",
                self.ram[FRAME_COUNTER],
                before,
                t,
                loc.file(),
                loc.line(),
                self.player_state_view().x(),
                self.player_state_view().y(),
            );
        }
        t
    }

    pub(super) fn module_unknown0(&mut self) {
        // C Module_Unknown0 is an assert(0) main-module slot.
        panic!("module_unknown0 reached");
    }

    pub(super) fn module_unknown1(&mut self) {
        // C Module_Unknown1 is an assert(0) main-module slot.
        panic!("module_unknown1 reached");
    }

    pub(super) fn sound_load_intro_song_bank(&mut self) {
        self.load_song_bank_asset(0);
    }

    pub(super) fn load_overworld_songs(&mut self) {
        self.load_song_bank_asset(0);
    }

    pub(super) fn load_dungeon_songs(&mut self) {
        self.load_song_bank_asset(1);
    }

    pub(super) fn load_credits_songs(&mut self) {
        self.load_song_bank_asset(2);
    }

    fn load_song_bank_asset(&mut self, asset: usize) {
        if let Some(bank) = self.asset_raw(asset).map(<[u8]>::to_vec) {
            self.load_song_bank(&bank);
        }
    }

    pub fn zelda_debug_song_bank_summary(&self, asset: usize) -> String {
        let Some(bank) = self.asset_raw(asset) else {
            return format!("song_bank[{asset}]=missing");
        };
        let mut ram = [0u8; 0x10000];
        let mut cursor = 0usize;
        let mut blocks = 0usize;
        let mut total_bytes = 0usize;
        let mut first_target = None;
        let mut min_target = 0xffffusize;
        let mut max_written = 0usize;
        while cursor + 4 <= bank.len() {
            let num_bytes = u16::from_le_bytes([bank[cursor], bank[cursor + 1]]) as usize;
            if num_bytes == 0 {
                cursor += 2;
                break;
            }
            let mut target = u16::from_le_bytes([bank[cursor + 2], bank[cursor + 3]]) as usize;
            cursor += 4;
            if cursor + num_bytes > bank.len() {
                return format!(
                    "song_bank[{asset}] malformed blocks={blocks} target={target:04x} len={num_bytes} remaining={}",
                    bank.len().saturating_sub(cursor)
                );
            }
            first_target.get_or_insert(target);
            min_target = min_target.min(target);
            for &byte in &bank[cursor..cursor + num_bytes] {
                ram[target & 0xffff] = byte;
                target = target.wrapping_add(1);
            }
            max_written = max_written.max(target.wrapping_sub(1) & 0xffff);
            cursor += num_bytes;
            blocks += 1;
            total_bytes += num_bytes;
        }
        let first_target = first_target.unwrap_or(0);
        let reset_vector = u16::from_le_bytes([ram[0xfffe], ram[0xffff]]);
        let code800 = &ram[0x800..0x808];
        let code878 = &ram[0x878..0x880];
        format!(
            "song_bank[{asset}] size={} blocks={blocks} total_bytes={total_bytes} first_target={first_target:04x} range={min_target:04x}-{max_written:04x} reset={reset_vector:04x} ram0800={code800:02x?} ram0878={code878:02x?} parse_end={cursor}",
            bank.len()
        )
    }

    fn kill_agahnim_load_music(&mut self) {
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        self.frame_control_view_mut().increment_submodule();
        self.LoadOWMusicIfNeeded();
    }

    fn kill_aghanim_init(&mut self) {
        self.ram[MUSIC_CONTROL] = 8;
        self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS] = 8;
        self.InitializeMirrorHDMA();
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.PaletteFilter_InitializeWhiteFilter();
        self.Overworld_LoadGFXAndScreenSize();
        self.frame_control_view_mut().increment_submodule();
        self.ram[LINK_PLAYER_HANDLER_STATE] = 20;
        write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, 0);
        write_le_u16(&mut self.ram, LINK_Y_VEL, 0);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, 0x7fff);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 32 * 2, 0x7fff);
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }

    fn kill_aghanim_func2(&mut self) {
        self.ram[HDMAEN_COPY] = 192;
        self.MirrorWarp_BuildWavingHDMATable();
        self.frame_control_view_mut().increment_submodule();
        self.frame_control_view_mut().set_subsubmodule(0);
    }

    fn kill_aghanim_func3(&mut self) {
        self.MirrorWarp_BuildWavingHDMATable();
        if self.frame_control_view().subsubmodule() != 0 {
            self.frame_control_view_mut().set_subsubmodule(0);
            self.frame_control_view_mut().increment_submodule();
        }
    }

    fn kill_aghanim_func4(&mut self) {
        self.MirrorWarp_BuildDewavingHDMATable();
        if self.frame_control_view().subsubmodule() != 0 {
            self.frame_control_view_mut().set_subsubmodule(0);
            self.frame_control_view_mut().increment_submodule();
        }
    }

    fn kill_aghanim_func5(&mut self) {
        self.hdma_setup(0, 0xf2fb, 0x41, 0, 0x26, 0);
        for i in 0..240 {
            write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0xff00);
        }
        self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0;
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x35);
        self.main_show_text_message();
        self.ReloadPreviouslyLoadedSheets();
        self.hud_rebuild_indoor();
        self.ram[HDMAEN_COPY] = 0x80;
        self.frame_control_view_mut().set_main_module(21);
        self.frame_control_view_mut().set_submodule(6);
        self.frame_control_view_mut().set_subsubmodule(24);
    }

    fn kill_aghanim_func6(&mut self) {
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() == 0 {
            self.frame_control_view_mut().increment_submodule();
            self.ram[SOUND_EFFECT_AMBIENT] = 9;
        }
    }

    fn kill_aghanim_func7(&mut self) {
        self.RenderText();
        if self.frame_control_view().submodule() == 0 {
            self.ram[OVERWORLD_MAP_STATE] = 0;
            self.ram[SOUND_EFFECT_AMBIENT] = 5;
            if self.ram[LINK_ITEM_MOON_PEARL] == 0 {
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x36);
                self.main_show_text_message();
                self.ram[SOUND_EFFECT_AMBIENT] = 0;
                self.frame_control_view_mut().set_main_module(21);
                self.frame_control_view_mut().set_submodule(8);
            } else {
                self.frame_control_view_mut().set_submodule(9);
            }
        }
    }

    fn kill_aghanim_func8(&mut self) {
        self.RenderText();
        if self.frame_control_view().submodule() == 0 {
            self.frame_control_view_mut().set_subsubmodule(32);
            self.frame_control_view_mut().set_submodule(12);
        }
    }

    fn kill_aghanim_func12(&mut self) {
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() != 0 {
            return;
        }
        self.ResetAncillaAndCutscene();
        self.Overworld_SetSongList();
        self.ram[SAVE_OW_EVENT_INFO_MISC + 0x1b] |= 32;
        self.ram[CUR_PALACE_INDEX_X2] = 255;
        self.frame_control_view_mut().set_submodule(0);
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
        self.frame_control_view_mut().set_main_module(9);
        self.ram[BG1VOFS_COPY2] = 0;
        self.ram[MUSIC_CONTROL] = if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
            9
        } else {
            4
        };
        self.ram[SAVEGAME_MAP_ICONS_INDICATOR] = 6;
    }

    pub(super) fn dungeon_light_torch(&mut self) {
        if self.ram[DUNGEON_TORCH_ATTR] & 0xf0 != 0xc0 {
            self.ram[DUNGEON_TORCH_ATTR] = 0;
            return;
        }

        let r8 = if (self.world_state_view().dungeon_room() as u8) == 0 {
            0x80
        } else {
            0xc0
        };
        let i = ((self.ram[DUNGEON_TORCH_ATTR] & 0x0f) as usize)
            + ((read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES_START) >> 1) as usize);
        let opos = read_le_u16(&self.ram, DUNG_OBJECT_POS_IN_OBJDATA + i * 2) as usize;
        let mut tilemap_pos = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2);
        if tilemap_pos & 0x8000 != 0 {
            return;
        }
        tilemap_pos |= 0x8000;
        write_le_u16(&mut self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2, tilemap_pos);
        if r8 == 0 {
            write_le_u16(&mut self.ram, DUNG_TORCH_DATA_MISC + opos * 2, tilemap_pos);
        }

        let x = tilemap_pos & 0x3fff;
        self.room_draw_adjust_torch_lighting_change(x, 0x0eca, x);
        self.ram[SOUND_EFFECT_1] = 42 | self.calculate_sfx_pan_arbitrary(((x & 0x7f) * 2) as u8);
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;

        if self.ram[DUNG_WANT_LIGHTS_OUT] != 0 {
            let lit = self.ram[DUNG_NUM_LIT_TORCHES];
            self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_add(1);
            if lit < 3 {
                const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
                self.ram[TS_COPY] = 0;
                self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] =
                    LIT_TORCHES_COLOR_PLUS[self.ram[DUNG_NUM_LIT_TORCHES] as usize];
                self.frame_control_view_mut().set_submodule(10);
                self.frame_control_view_mut().set_subsubmodule(0);
            }
        }

        let torch_timer = (self.ram[DUNGEON_TORCH_ATTR] & 0x0f) as usize;
        self.ram[DUNG_TORCH_TIMERS_MISC + torch_timer] = r8;
        self.ram[DUNGEON_TORCH_ATTR] = 0;
    }

    pub(super) fn room_draw_adjust_torch_lighting_change(&mut self, x: u16, y: u16, r8: u16) {
        let ptr = self.src_ptr(y);
        let x = (x >> 1) as usize;
        write_le_u16(&mut self.ram, OVERWORLD_TILEATTR_MISC + (x + 0) * 2, ptr[0]);
        write_le_u16(
            &mut self.ram,
            OVERWORLD_TILEATTR_MISC + (x + 64) * 2,
            ptr[1],
        );
        write_le_u16(&mut self.ram, OVERWORLD_TILEATTR_MISC + (x + 1) * 2, ptr[2]);
        write_le_u16(
            &mut self.ram,
            OVERWORLD_TILEATTR_MISC + (x + 65) * 2,
            ptr[3],
        );
        self.dungeon_prep_overlay_dma_next_prep(0, r8);
    }

    pub(super) fn dungeon_prep_overlay_dma_next_prep(&mut self, dst: usize, r8: u16) -> usize {
        let r6 = 0x0880 + u16::from((r8 & 0x003f) >= 0x003a);
        self.dungeon_prep_overlay_dma_watergate(dst, r8, r6, 4)
    }

    #[rustfmt::skip]
    pub(super) fn dungeon_prep_overlay_dma_watergate(&mut self, dst: usize, r8: u16, r6: u16, loops: usize) -> usize {
        let mut dst = dst;
        let mut r8 = r8;
        for _ in 0..loops {
            let x = (r8 >> 1) as usize;
            write_le_u16(
                &mut self.ram,
                VRAM_UPLOAD_TILE_BUF_MISC + (dst + 0) * 2,
                ((r8 & 0x40) << 4) | ((r8 & 0x303f) >> 1) | ((r8 & 0x0f80) >> 2),
            );
            write_le_u16(&mut self.ram, VRAM_UPLOAD_TILE_BUF_MISC + (dst + 1) * 2, r6);
            let attr0 = read_le_u16(&self.ram, OVERWORLD_TILEATTR_MISC + (x + 0) * 2);
            write_le_u16(
                &mut self.ram,
                VRAM_UPLOAD_TILE_BUF_MISC + (dst + 2) * 2,
                attr0,
            );
            if r6 & 1 == 0 {
                for j in 1..=3 {
                    let attr = read_le_u16(&self.ram, OVERWORLD_TILEATTR_MISC + (x + j) * 2);
                    write_le_u16(
                        &mut self.ram,
                        VRAM_UPLOAD_TILE_BUF_MISC + (dst + 2 + j) * 2,
                        attr,
                    );
                }
                r8 = r8.wrapping_add(128);
            } else {
                for (j, offset) in [64usize, 128, 192].into_iter().enumerate() {
                    let attr = read_le_u16(&self.ram, OVERWORLD_TILEATTR_MISC + (x + offset) * 2);
                    write_le_u16(
                        &mut self.ram,
                        VRAM_UPLOAD_TILE_BUF_MISC + (dst + 3 + j) * 2,
                        attr,
                    );
                }
                r8 = r8.wrapping_add(2);
            }
            dst += 6;
        }
        write_le_u16(&mut self.ram, VRAM_UPLOAD_TILE_BUF_MISC + dst * 2, 0xffff);
        dst
    }

    fn src_ptr(&self, src: u16) -> [u16; 4] {
        let Some(data) = self.asset_raw(69) else {
            return [0; 4];
        };
        let offset = src as usize;
        [
            read_word_from_slice(data, offset),
            read_word_from_slice(data, offset + 2),
            read_word_from_slice(data, offset + 4),
            read_word_from_slice(data, offset + 6),
        ]
    }

    pub(super) fn module05_load_file(&mut self) {
        if self.state_recorder.replay_mode
            && std::env::var_os("ZELDA3_SMV_LOADFILE_TIMING_HACKS").is_some()
            && self.frame_control_view().main_module() == 5
            && self.frame_control_view().submodule() == 0
            && self.ram[SAVED_MODULE_FOR_MENU] == 0
            && read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX) == 0x000a
        {
            if self.replay_loadfile_stall == 0 {
                self.replay_loadfile_stall = 75;
            }
            self.replay_loadfile_stall = self.replay_loadfile_stall.wrapping_sub(1);
            if self.replay_loadfile_stall != 0 {
                return;
            }
        } else {
            self.replay_loadfile_stall = 0;
        }

        self.enable_force_blank();
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] = 0;
        self.ram[TAGALONG_SHARED_STATE_A] = 0;
        self.ram[TAGALONG_ANIM_FRAME_COUNTER] = 0;
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 0;
        self.ram[PLAYER_POSE_DRAW_COUNTER] = 0;
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = 0;
        self.erase_tile_maps_normal();
        self.load_default_graphics();
        self.sprite_load_graphics_properties();
        self.init_load_default_tile_attr();
        self.decompress_sword_graphics();
        self.decompress_shield_graphics();
        self.link_initialize();
        self.load_follower_graphics();
        self.ram[SPRITE_GFX_SUBSET_0] = 70;
        self.ram[SPRITE_GFX_SUBSET_1] = 70;
        self.ram[SPRITE_GFX_SUBSET_2] = 70;
        self.ram[SPRITE_GFX_SUBSET_3] = 70;
        write_le_u16(&mut self.ram, SHARED_MESSAGE_TIMER, 0x0200);
        self.ram[VIRQ_TRIGGER] = 48;

        if self.ram[SAVEGAME_IS_DARKWORLD] != 0 {
            if self.ram[PLAYER_IS_INDOORS] != 0 {
                self.load_dungeon_room_rebuild_hud();
                return;
            }
            self.hud_search_for_equipped_item();
            self.hud_rebuild();
            self.hud_update_equipped_item();
            self.ram[GAME_OVER_CHECK_FLAG] = 0;
            write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX, 32);
            self.frame_control_view_mut().set_main_module(8);
            self.frame_control_view_mut().set_submodule(0);
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[RESTART_CHECK_FLAG] = 0;
        } else if self.ram[MOSAIC_LEVEL] != 0
            || (self.ram[GAME_OVER_CHECK_FLAG] != 0 && self.ram[RESTART_CHECK_FLAG] == 0)
            || self.ram[SRAM_PROGRESS_INDICATOR] < 2
            || self.ram[WHICH_STARTING_POINT] == 5
        {
            self.load_dungeon_room_rebuild_hud();
        } else {
            let message = if self.ram[LINK_ITEM_MIRROR] == 2 {
                0x0185
            } else {
                0x0184
            };
            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, message);
            self.main_show_text_message();
            self.dungeon_load_palettes();
            self.ram[INIDISP_COPY] = 15;
            self.ram[TM_COPY] = 4;
            self.ram[TS_COPY] = 0;
            self.frame_control_view_mut().set_main_module(27);
        }
    }

    pub(super) fn load_dungeon_room_rebuild_hud(&mut self) {
        self.ram[MOSAIC_LEVEL] = 0;
        self.ram[MOSAIC_COPY] = 7;
        self.hud_search_for_equipped_item();
        self.hud_rebuild();
        self.hud_update_equipped_item();
        self.module_pre_dungeon();
    }

    pub(super) fn patch_new_game_entrance_state(&mut self) {
        if self.ram[SRAM_PROGRESS_INDICATOR] != 0 {
            return;
        }

        write_le_u16(&mut self.ram, CAMERA_Y_COORD_SCROLL_LOW, 0x017f);
        write_le_u16(&mut self.ram, CAMERA_Y_COORD_SCROLL_HI, 0x0181);
        write_le_u16(&mut self.ram, DUNG_QUADRANTS_VISITED, 2);
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, 2);
        self.ram[PALETTE_SP0L] = 0;
        self.ram[PALETTE_SP5L] = 3;
        self.ram[PALETTE_SP6L] = 1;
        self.ram[PALETTE_MAIN_INDOORS] = 4;
        self.load_new_game_room_sprite();
    }

    fn load_new_game_room_sprite(&mut self) {
        self.ram[SPRITE_Y_LO] = 0x70;
        self.ram[SPRITE_X_LO] = 0xa0;
        self.ram[SPRITE_Y_HI] = 0x21;
        self.ram[SPRITE_X_HI] = 0x09;
        self.ram[SPRITE_STATE] = 0x08;
        self.ram[SPRITE_TYPE] = 0x73;
        self.ram[SPRITE_ROOM_ORIGIN_X_HI] = 0x08;
        self.ram[SPRITE_ROOM_ORIGIN_Y_HI] = 0x20;
        self.ram[SPRITE_SHARED_SCRATCH_A] = 0x1a;
    }

    pub(super) fn load_pre_dungeon_keys(&mut self) {
        let d = read_le_u16(&self.ram, CUR_PALACE_INDEX_X2);
        self.ram[LINK_NUM_KEYS] = if d != 0x00ff {
            let index = if d == 2 { 0 } else { (d >> 1) as usize };
            self.ram
                .get(LINK_KEYS_EARNED_PER_DUNGEON + index)
                .copied()
                .unwrap_or(0)
        } else {
            0xff
        };
    }

    pub(super) fn init_load_default_tile_attr(&mut self) {
        self.ram[ATTRIBUTES_FOR_TILE..ATTRIBUTES_FOR_TILE + 0x140]
            .copy_from_slice(&DUNGEON_DEFAULT_ATTR[..0x140]);
        self.ram[ATTRIBUTES_FOR_TILE + 0x1c0..ATTRIBUTES_FOR_TILE + 0x200]
            .copy_from_slice(&DUNGEON_DEFAULT_ATTR[0x140..0x180]);
    }

    pub(super) fn module13_boss_victory_pendant(&mut self) {
        match self.frame_control_view().submodule() {
            0 => self.boss_victory_heal(),
            1 => self.dungeon_start_victory_spin(),
            2 => self.dungeon_run_victory_spin(),
            3 => self.dungeon_close_victory_spin(),
            4 => self.Dungeon_PrepExitWithSpotlight(),
            5 => self.Spotlight_ConfigureTableAndControl(),
            _ => {}
        }
        self.sprite_main();
        self.link_oam_main();
    }

    pub(super) fn boss_victory_heal(&mut self) {
        if !self.hud_refill_magic_power() {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }
        if !self.hud_refill_health() {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }
        if self.ram[OVERWORLD_MAP_STATE] == 0 {
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
            self.Dungeon_ResetTorchBackgroundAndPlayerInner();
            self.ram[LINK_DIRECTION_FACING] = 2;
            self.ram[LINK_DIRECTION_LAST] = 2 << 1;
            self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
            self.frame_control_view_mut().increment_submodule();
            self.frame_control_view_mut().set_subsubmodule(16);
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = self.ram[FLAG_IS_LINK_IMMOBILIZED].wrapping_add(1);
        }
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.hud_refill_logic();
    }

    pub(super) fn dungeon_start_victory_spin(&mut self) {
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() != 0 {
            return;
        }
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.ram[LINK_DIRECTION_FACING] = 2;
        self.link_animate_victory_spin();
        self.ancilla_terminate_select_interactives(0);
        self.ancilla_add_victory_spin();
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn dungeon_run_victory_spin(&mut self) {
        self.link_main();
        if self.ram[LINK_PLAYER_HANDLER_STATE] != 0 {
            return;
        }
        if self.ram[LINK_SWORD_TYPE].wrapping_add(1) & 0xfe != 0 {
            self.ram[SOUND_EFFECT_1] = 0x2c;
        }
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
        self.frame_control_view_mut().set_subsubmodule(32);
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn dungeon_close_victory_spin(&mut self) {
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() != 0 {
            return;
        }
        self.frame_control_view_mut().increment_submodule();
        self.ram[LINK_Y_VEL] = 0;
        self.ram[LINK_X_VEL] = 0;
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = 0;
    }

    pub(super) fn module15_mirror_warp_from_aga(&mut self) {
        match self.frame_control_view().submodule() {
            0 => self.kill_agahnim_load_music(),
            1 => self.kill_aghanim_init(),
            2 => self.kill_aghanim_func2(),
            3 => self.kill_aghanim_func3(),
            4 => self.kill_aghanim_func4(),
            5 => self.kill_aghanim_func5(),
            6 => self.kill_aghanim_func6(),
            7 => self.kill_aghanim_func7(),
            8 => self.kill_aghanim_func8(),
            9 => self.boss_victory_heal(),
            10 => self.dungeon_start_victory_spin(),
            11 => self.dungeon_run_victory_spin(),
            12 => self.kill_aghanim_func12(),
            _ => {}
        }
        if self.frame_control_view().submodule() < 2 || self.frame_control_view().submodule() >= 5 {
            self.sprite_main();
            self.link_oam_main();
        }
    }

    pub(super) fn module16_boss_victory_crystal(&mut self) {
        match self.frame_control_view().submodule() {
            0 => self.boss_victory_heal(),
            1 => self.dungeon_start_victory_spin(),
            2 => self.dungeon_run_victory_spin(),
            3 => self.dungeon_close_victory_spin(),
            4 => self.module16_04_fade_and_end(),
            _ => {}
        }
        self.sprite_main();
        self.link_oam_main();
    }

    pub(super) fn module16_04_fade_and_end(&mut self) {
        self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
        if self.ram[INIDISP_COPY] != 0 {
            return;
        }
        self.ram[BG1_X_OFFSET] = 0;
        self.ram[BG1_Y_OFFSET] = 0;
        self.ram[LINK_Y_VEL] = 0;
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.Palette_RevertTranslucencySwap();
        self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        self.ram[LINK_RECEIVEITEM_INDEX] = 0;
        self.ram[LINK_POSE_FOR_ITEM] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        let saved_module = self.ram[SAVED_MODULE_FOR_MENU];
        self.frame_control_view_mut().set_main_module(saved_module);
        self.frame_control_view_mut().set_submodule(0);
        self.frame_control_view_mut().set_subsubmodule(0);
        self.OpenSpotlight_Next2();
    }

    pub(super) fn triforce_room_link_approach_triforce(&mut self) {
        let y = self.ram[LINK_Y_COORD];
        if y < 152 {
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_DIRECTION_LAST] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
            if self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] == 0 {
                self.ram[LINK_POSE_FOR_ITEM] = 2;
                self.frame_control_view_mut().increment_subsubmodule();
            }
        } else {
            if y < 169 {
                self.ram[LINK_SPEED_SETTING] = 0x14;
            }
            self.ram[LINK_DIRECTION] = 8;
            self.ram[LINK_DIRECTION_LAST] = 8;
            self.ram[LINK_DIRECTION_FACING] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 64;
        }
    }

    pub(super) fn ancilla_add_item_receipt(&mut self, ain: u8, yin: u8, chest_pos: u16) {
        let Some(k) = self.ancilla_add_simple(ain, yin) else {
            return;
        };
        let item = self.ram[LINK_RECEIVEITEM_INDEX];

        self.ram[FLAG_IS_LINK_IMMOBILIZED] = if item == 0x20 { 2 } else { 1 };
        if item == 0 {
            self.ram[memory_location_to_give_item_to_misc(4)] = value_to_give_item_to_misc(0);
        }

        let value_addr = memory_location_to_give_item_to_misc(item);
        let value = value_to_give_item_to_misc(item);
        if (value as i8) >= 0 {
            self.ram[value_addr] = value;
        }

        if item == 0x1f {
            self.ram[LINK_IS_BUNNY] = 0;
        } else if item == 0x4b || item == 0x1e {
            self.ram[LINK_ABILITY_FLAGS] |= if item == 0x4b { 4 } else { 2 };
        }

        if item == 0x1b || item == 0x1c {
            self.Palette_UpdateGlovesColor();
        } else if matches!(item, 0x37 | 0x38 | 0x39) {
            let bit = match item {
                0x37 => 4,
                0x38 => 1,
                _ => 2,
            };
            self.ram[value_addr] |= bit;
            if self.ram[value_addr] & 7 == 7 {
                self.ram[SAVEGAME_MAP_ICONS_INDICATOR] = 4;
            }
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        } else if item == 0x22 {
            if self.ram[value_addr] == 0 {
                self.ram[value_addr] = 1;
            }
        } else if matches!(item, 0x25 | 0x32 | 0x33) {
            let mask = 0x8000u16 >> ((self.ram[CUR_PALACE_INDEX_X2] >> 1) as u16);
            let current = self.read_u16_ram(value_addr);
            self.write_u16_ram(value_addr, current | mask);
        } else if item == 0x3e {
            if self.ram[LINK_STATE_BITS] & 0x80 != 0 {
                self.ram[LINK_PICKING_THROW_STATE] = 2;
            }
        } else if item == 0x20 {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
            for i in (0..=4).rev() {
                if self.ram[ANCILLA_TYPE + i] == 7 || self.ram[ANCILLA_TYPE + i] == 0x2c {
                    self.ram[ANCILLA_TYPE + i] = 0;
                    self.ram[LINK_STATE_BITS] = 0;
                    self.ram[LINK_PICKING_THROW_STATE] = 0;
                }
            }
            if self.ram[LINK_CAPE_MODE] != 0 {
                self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 32;
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                self.ram[LINK_CAPE_MODE] = 0;
                self.ancilla_add_cape_poof(0x23, 4);
                self.ram[SOUND_EFFECT_1] = 0x15 | self.link_calculate_sfx_pan();
            }
        } else if item == 0x29 {
            if self.ram[LINK_ITEM_MUSHROOM] != 2 {
                self.ram[value_addr] = 1;
                self.hud_refresh_icon();
            }
        } else if item == 0x24
            || (self.ram[ITEM_RECEIPT_METHOD] != 2 && matches!(item, 0x27 | 0x28 | 0x31))
        {
            let add = match item {
                0x28 => 3,
                0x31 => 10,
                _ => 1,
            };
            self.ram[value_addr] = self.ram[value_addr].saturating_add(add).min(99);
            self.hud_refresh_icon();
        } else if item == 0x17 {
            self.ram[value_addr] = self.ram[value_addr].wrapping_add(1) & 3;
            self.ram[SOUND_EFFECT_2] = 0x2d | self.link_calculate_sfx_pan();
        } else if item == 1 {
            self.Overworld_SetSongList();
        } else {
            self.item_receipt_give_bottled_item(item);
        }

        let mut gfx = receive_item_gfx_misc(item);
        if gfx == 0xff {
            gfx = 0;
        } else if gfx == 0x20 || gfx == 0x2d || gfx == 0x2e {
            self.DecompressShieldGraphics();
            self.Palette_Load_Shield();
        }
        self.DecodeAnimatedSpriteTile_variable(gfx);
        if (gfx == 6 || gfx == 0x18) && item != 0 {
            self.DecompressSwordGraphics();
            self.Palette_Load_Sword();
        }

        self.ram[ANCILLA_ITEM_TO_LINK + k] = item;
        self.ram[ANCILLA_ARR1 + k] = 0;
        if item == 1 && self.ram[ITEM_RECEIPT_METHOD] != 2 {
            self.ram[ANCILLA_TIMER + k] = 160;
            self.frame_control_view_mut().set_submodule(43);
            self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
            self.ancilla_add_ms_cutscene(0x35, 4);
            self.ram[ANCILLA_ARR3 + k] = 2;
        } else {
            self.ram[ANCILLA_ARR3 + k] = 9;
        }
        self.ram[ANCILLA_ARR4 + k] = 5;
        self.ram[ANCILLA_STEP + k] = self.ram[ITEM_RECEIPT_METHOD];
        self.ram[ANCILLA_AUX_TIMER + k] = if matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
            0x68
        } else if item == 0x26 {
            0x02
        } else if self.ram[ITEM_RECEIPT_METHOD] != 0 {
            0x38
        } else {
            0x60
        };

        let (x, y) = if self.ram[ITEM_RECEIPT_METHOD] == 1 {
            let y = ((chest_pos & 0x1f80) >> 4)
                .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY) & 0xff00)
                .wrapping_add(receive_item_tab2_misc(item) as i16 as u16);
            let x = ((chest_pos & 0x007e) << 2)
                .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY) & 0xff00)
                .wrapping_add(receive_item_tab3_misc(item) as u16);
            (x, y)
        } else {
            if self.ram[ANCILLA_STEP + k] == 0 && item == 1 {
                self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 0x2c;
            } else if matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
                self.ram[MUSIC_CONTROL] = self.link_calculate_sfx_pan() | 0x13;
            } else if item != 0x3e && item != 0x17 {
                self.ram[SOUND_EFFECT_2] = self.link_calculate_sfx_pan() | 0x0f;
            }
            let method = if self.ram[ITEM_RECEIPT_METHOD] == 3 {
                0
            } else {
                self.ram[ITEM_RECEIPT_METHOD]
            };
            let x_offset = if method != 0 {
                receive_item_tab3_misc(item) as u16
            } else if receive_item_tab1_misc(item) == 0 {
                10
            } else if item == 0x20 {
                0
            } else {
                6
            };
            let y_offset = if method != 0 {
                receive_item_tab2_misc(item) as i16
            } else {
                -14
            } + if method == 2 { -8 } else { 0 };
            (
                self.player_state_view().x().wrapping_add(x_offset),
                self.player_state_view().y().wrapping_add(y_offset as u16),
            )
        };
        self.ancilla_set_xy(k, x, y);
    }

    pub(super) fn item_receipt_give_bottled_item(&mut self, item: u8) {
        const BOTTLE_LIST: [u8; 7] = [0x16, 0x2b, 0x2c, 0x2d, 0x3d, 0x3c, 0x48];
        const POTION_LIST: [u8; 5] = [0x2e, 0x2f, 0x30, 0xff, 0x0e];

        if let Some(j) = BOTTLE_LIST.iter().position(|&candidate| candidate == item) {
            for i in 0..4 {
                if self.ram[LINK_ITEM_BOTTLE_INFO + i] < 2 {
                    self.ram[LINK_ITEM_BOTTLE_INFO + i] = j as u8 + 2;
                    return;
                }
            }
        }

        if let Some(j) = POTION_LIST.iter().position(|&candidate| candidate == item) {
            for i in 0..4 {
                if self.ram[LINK_ITEM_BOTTLE_INFO + i] == 2 {
                    self.ram[LINK_ITEM_BOTTLE_INFO + i] = j as u8 + 3;
                    return;
                }
            }
        }
    }

    pub(super) fn module17_save_and_quit(&mut self) {
        match self.frame_control_view().submodule() {
            0 => {
                self.frame_control_view_mut().increment_submodule();
            }
            1 => {}
            _ => {}
        }
        if self.frame_control_view().submodule() == 1 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
            if self.ram[INIDISP_COPY] == 0 {
                self.ram[MOSAIC_COPY] = 15;
                self.frame_control_view_mut().set_subsubmodule(1);
                self.Death_Func15(false);
            }
        }
        self.sprite_main();
        self.link_oam_main();
    }

    pub(super) fn wall_master_send_player_to_last_entrance(&mut self) {
        self.SaveDungeonKeys_misc();
        self.Dungeon_FlagRoomData_Quadrants();
        self.sprite_reset_all();
        self.ram[RESTART_CHECK_FLAG] = 0;
        self.frame_control_view_mut().set_main_module(17);
        self.frame_control_view_mut().set_submodule(0);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 0;
        self.reset_some_things_after_death(17);
    }

    pub(super) fn sprite_sfx_queue_sfx1_with_pan(&mut self, k: usize, a: u8) {
        if self.ram[SOUND_EFFECT_AMBIENT] == 0 {
            let x = self.ram[SPRITE_X_LO + k] as u16 | ((self.ram[SPRITE_X_HI + k] as u16) << 8);
            self.ram[SOUND_EFFECT_AMBIENT] =
                a | Self::calculate_sfx_pan_with_scroll(x, read_le_u16(&self.ram, BG2HOFS_COPY2));
        }
    }

    pub(super) fn SaveDungeonKeys_misc(&mut self) {
        let idx = self.ram[CUR_PALACE_INDEX_X2];
        if idx == 0xff {
            return;
        }
        let slot = if idx == 2 { 0 } else { (idx >> 1) as usize };
        self.ram[LINK_KEYS_EARNED_PER_DUNGEON + slot] = self.ram[LINK_NUM_KEYS];
    }

    pub(super) fn handle_item_tile_action_overworld(&mut self, x: u16, y: u16) -> u8 {
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.HandleItemTileAction_Dungeon(x, y)
        } else {
            self.Overworld_ToolAndTileInteraction(x, y) as u8
        }
    }
}

pub(super) const DUNG_ANIMATED_TILES: [u8; 24] = [
    0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5f, 0x5d, 0x5f, 0x5f, 0x5e, 0x5f, 0x5e, 0x5e, 0x5d,
    0x5d, 0x5e, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
];

const DUNGEON_DEFAULT_ATTR: [u8; 384] = [
    1, 1, 1, 0, 2, 1, 2, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 1, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0,
    0, 0, 0, 0x2a, 1, 0x20, 1, 1, 4, 1, 1, 0x18, 1, 2, 0x1c, 1, 0x28, 0x28, 0x2a, 0x2a, 1, 2, 1, 1,
    4, 0, 0, 0, 0x28, 1, 0xa, 0, 1, 1, 0xc, 0xc, 2, 2, 2, 2, 0x28, 0x2a, 0x20, 0x20, 0x20, 2, 8, 0,
    4, 4, 1, 1, 1, 2, 2, 2, 0, 0, 0x20, 0x20, 0, 2, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x18, 0x10, 0x10, 1, 1, 1, 1, 1, 4, 4, 4, 4, 4, 4, 1, 2, 2,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x62, 0x62, 0, 0, 0x24, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x62, 0x62, 0x27, 2, 2, 2,
    0x27, 0x27, 1, 0, 0, 0, 0, 0x24, 0, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x10, 2, 1, 0, 0, 0,
    0x24, 0, 0, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 2, 2, 2, 0x24, 0, 0, 0, 0, 0x27, 0x27,
    0x27, 0x27, 0x27, 0x20, 2, 2, 1, 2, 2, 0x23, 2, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x20, 2,
    0x27, 2, 0x54, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 0x27, 0x27, 2, 0x27, 2, 0x54, 0, 0,
    0x27, 2, 2, 2, 0x27, 0x27, 0, 0x27, 0x60, 0x60, 1, 1, 1, 1, 2, 2, 0xd, 0, 0, 0x4b, 0x67, 0x67,
    0x67, 0x67, 0x66, 0x66, 0x66, 0x66, 0, 0, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x27, 0x63, 0x27,
    0x55, 0x55, 1, 0x44, 0, 1, 0x20, 2, 2, 0x1c, 0x3a, 0x3b, 0, 0x27, 0x63, 0x27, 0x53, 0x53, 1,
    0x44, 1, 0xd, 0, 0, 0, 9, 9, 9, 9,
];

impl ZeldaState {
    pub(super) fn calculate_sfx_pan_with_scroll(x: u16, bg2hofs: u16) -> u8 {
        let relative = x.wrapping_sub(bg2hofs.wrapping_add(80));
        if relative < 80 {
            0
        } else if (relative as i16) >= 0 {
            0x40
        } else {
            0x80
        }
    }

    pub(super) fn link_calculate_sfx_pan(&self) -> u8 {
        Self::calculate_sfx_pan_with_scroll(
            self.player_state_view().x(),
            read_le_u16(&self.ram, BG2HOFS_COPY2),
        )
    }

    pub(super) fn play_sfx_set_pan(&mut self, a: u8) -> u8 {
        self.ram[RAW_SFX_PAN_VALUE] = a;
        let out = a | self.link_calculate_sfx_pan();
        self.replay_trace_sfx("play_sfx_set_pan", None, a, out);
        out
    }

    pub(super) fn ancilla_sfx2_near(&mut self, a: u8) -> u8 {
        let panned = self.play_sfx_set_pan(a);
        self.ram[SOUND_EFFECT_1] = panned;
        panned
    }

    pub(super) fn ancilla_sfx3_near(&mut self, a: u8) {
        self.ram[SOUND_EFFECT_2] = self.play_sfx_set_pan(a);
    }

    pub(super) fn calculate_sfx_pan_arbitrary(&self, a: u8) -> u8 {
        const TORCH_PANS: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        TORCH_PANS[((a.wrapping_sub(self.ram[BG2HOFS_COPY2]) >> 5) & 7) as usize]
    }

    pub(super) fn sprite_calculate_sfx_pan(&self, k: usize) -> u8 {
        let x = self.ram[SPRITE_X_LO + k] as u16 | ((self.ram[SPRITE_X_HI + k] as u16) << 8);
        Self::calculate_sfx_pan_with_scroll(x, read_le_u16(&self.ram, BG2HOFS_COPY2))
    }

    pub(super) fn sprite_sfx_queue_sfx2_with_pan(&mut self, k: usize, a: u8) {
        if self.ram[SOUND_EFFECT_1] == 0 {
            self.ram[SOUND_EFFECT_1] = a | self.sprite_calculate_sfx_pan(k);
        }
    }

    pub(super) fn sprite_sfx_queue_sfx3_with_pan(&mut self, k: usize, a: u8) {
        if self.ram[SOUND_EFFECT_2] == 0 {
            self.ram[SOUND_EFFECT_2] = a | self.sprite_calculate_sfx_pan(k);
        }
    }

    pub(super) fn main_show_text_message(&mut self) {
        if self.frame_control_view().main_module() != 14 {
            self.ram[TILE_INTERACTION_SHARED_FLAG] = 0;
            self.ram[MESSAGING_MODULE] = 0;
            self.frame_control_view_mut().set_submodule(2);
            self.ram[SAVED_MODULE_FOR_MENU] = self.frame_control_view().main_module();
            self.frame_control_view_mut().set_main_module(14);
        }
    }

    pub(super) fn nmi_prepare_sprites(&mut self) {
        const LINK_DMA_SOURCES1: [u16; 303] = [
            0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x9440, 0x8080, 0x8080, 0x8080, 0x9400, 0x8040,
            0x80c0, 0x80c0, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080,
            0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0xa8c0,
            0xa900, 0x8000, 0xa8c0, 0xa900, 0x9100, 0x8080, 0x8080, 0x90c0, 0x8040, 0x8000, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8000, 0x9a00, 0x9140, 0x9180, 0x8000, 0x9500, 0x9480, 0x94c0,
            0x94c0, 0x9ae0, 0x8080, 0x8080, 0x9a60, 0x80c0, 0x80c0, 0x9aa0, 0x8000, 0x8000, 0x9aa0,
            0x8000, 0x8000, 0x8080, 0x8080, 0x8100, 0x8100, 0x85c0, 0x8000, 0x8000, 0x85c0, 0x8000,
            0x8000, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xad40, 0xad40, 0xad40, 0xad40, 0xad40,
            0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0x8040, 0x9400, 0x8040, 0x8000, 0x8080,
            0x8080, 0x9440, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8040, 0x8040, 0x8000, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8000, 0xc440, 0x8140, 0x8140, 0xca40, 0x8000, 0x8000, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8040, 0x85c0, 0x8040, 0x85c0, 0x8100, 0x80c0, 0x91c0, 0x8080,
            0x8080, 0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8080, 0x9100, 0xa0c0,
            0xa100, 0xa100, 0xa1c0, 0xa400, 0xa440, 0xa1c0, 0xa400, 0xa440, 0x8080, 0xc480, 0x8080,
            0x8040, 0x8040, 0xca80, 0xca80, 0xca00, 0xc400, 0xca00, 0xc400, 0x81c0, 0x8080, 0x8080,
            0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040,
            0x8040, 0x8040, 0x8000, 0xa8c0, 0xa900, 0x8000, 0x8000, 0xa8c0, 0xa900, 0x8000, 0xa8c0,
            0xa900, 0x8000, 0x8000, 0xa8c0, 0xa900, 0x8040, 0x8040, 0x8040, 0x8080, 0x8080, 0x8040,
            0x8040, 0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0xd080, 0x8080, 0x90c0, 0xd000,
            0x9080, 0xd040, 0x9080, 0xd040, 0xd080, 0xd080, 0xd080, 0xd080, 0xd080, 0xd000, 0xd000,
            0xd000, 0xd000, 0xd000, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040, 0x8040, 0xd000,
            0x85c0, 0x85c0, 0x85c0, 0xdc40, 0xdc40, 0xdc40, 0x85c0, 0x85c0, 0x85c0, 0xdc40, 0xdc40,
            0xdc40, 0xe1c0, 0xd000, 0x8000, 0xe400, 0xe400, 0xe440, 0x90c0, 0x90c0, 0xd000, 0x8000,
            0x8000, 0xd040, 0x8000, 0x8000, 0xd040, 0xe400, 0xe400, 0xe400, 0x9080, 0xa5c0, 0xac40,
            0xe480, 0x8180, 0x90c0, 0x80c0, 0xe180, 0xd000, 0xe4c0, 0xe4c0, 0xe840, 0xe840, 0xe840,
            0xe540, 0xe540, 0xe540, 0xe900, 0xe900, 0xe900, 0xe900, 0x8080, 0x8080, 0x8000, 0xa9c0,
            0x8080, 0x8140, 0x91c0, 0x8040, 0xa800, 0xa840,
        ];
        const LINK_DMA_SOURCES2: [u16; 303] = [
            0x8840, 0x8800, 0x8580, 0x8800, 0x8580, 0x84c0, 0x8500, 0x8540, 0x8500, 0x8540, 0x8400,
            0x8440, 0x8480, 0x8400, 0x8440, 0x8480, 0x9640, 0x8c40, 0x8c80, 0xad00, 0x9600, 0x8980,
            0x8c00, 0xacc0, 0x8880, 0x88c0, 0x8900, 0x8940, 0x8880, 0x88c0, 0x8900, 0x8940, 0xb0c0,
            0xb100, 0xb140, 0xb100, 0xb140, 0xb000, 0xb040, 0xb080, 0xec80, 0xecc0, 0xb180, 0xd440,
            0xb1c0, 0xb180, 0xd440, 0xb1c0, 0x8c80, 0xad00, 0x95c0, 0x99c0, 0xb440, 0x9580, 0xb480,
            0xb4c0, 0x9580, 0xb480, 0xb4c0, 0x9c20, 0x8000, 0x8000, 0x8000, 0x9700, 0x9680, 0x96c0,
            0x96c0, 0x9ce0, 0x8c80, 0xb540, 0x9c60, 0xb580, 0x8c00, 0x9ca0, 0x8900, 0xb500, 0x9ca0,
            0x8900, 0xb500, 0x8c40, 0xec40, 0x8c00, 0xec00, 0x8dc0, 0x9540, 0x89c0, 0x8dc0, 0x9540,
            0x89c0, 0xb940, 0xb980, 0xb9c0, 0xb980, 0xb9c0, 0xb5c0, 0xb800, 0xb840, 0xb800, 0xb840,
            0xb880, 0xb8c0, 0xb900, 0xb880, 0xb8c0, 0xb900, 0x8980, 0x9600, 0xbcc0, 0x8400, 0xbc80,
            0x8c40, 0x9640, 0xa040, 0xa080, 0xa000, 0xbc40, 0xbd40, 0x8500, 0xbd00, 0xbd80, 0xbd80,
            0x88c0, 0x8900, 0xe9c0, 0x8900, 0xc640, 0xc040, 0xc000, 0xcc40, 0x8940, 0x88c0, 0x8900,
            0xe9c0, 0x8900, 0x8940, 0x8d40, 0x8d80, 0x8d40, 0x8d80, 0xbd00, 0xb000, 0xb000, 0xa480,
            0xa480, 0xa480, 0xa480, 0xac00, 0xac00, 0xac00, 0xac00, 0xa140, 0xa180, 0xa180, 0xa4c0,
            0xa4c0, 0xa500, 0x9d40, 0x9d80, 0x9dc0, 0x9d40, 0x9d80, 0x9dc0, 0x8d00, 0xc680, 0xc180,
            0xc140, 0x8c00, 0xcc80, 0xcc80, 0xcc00, 0xc600, 0xcc00, 0xc600, 0xbd00, 0x8580, 0x8800,
            0xc9c0, 0xccc0, 0xcdc0, 0xcd00, 0xcd40, 0xcd80, 0x8500, 0x8540, 0xc940, 0xc980, 0x8540,
            0xc940, 0xc980, 0x8440, 0x8480, 0xc1c0, 0xc900, 0xc580, 0xc5c0, 0xc8c0, 0x8440, 0x8480,
            0xc1c0, 0xc900, 0xc580, 0xc5c0, 0xc8c0, 0xbd00, 0xacc0, 0xc040, 0xd540, 0xd580, 0xd4c0,
            0xd500, 0xd4c0, 0xd500, 0xd440, 0xd480, 0xd440, 0xd480, 0xd1c0, 0xd400, 0xd100, 0xd100,
            0xd140, 0xd180, 0xd140, 0xd180, 0xb0c0, 0xb100, 0xb140, 0xb100, 0xb140, 0xdd40, 0xdd80,
            0xddc0, 0xdd80, 0xddc0, 0xdc80, 0xdcc0, 0xdd00, 0xdc80, 0xdcc0, 0xdd00, 0xd100, 0xd100,
            0xe000, 0xe040, 0xe080, 0xe0c0, 0xe100, 0xe140, 0xe000, 0xe040, 0xe080, 0xe0c0, 0xe100,
            0xe140, 0x8000, 0xd0c0, 0x8000, 0xb940, 0xb980, 0xb940, 0xdd40, 0xdd80, 0xdd40, 0xdc80,
            0xdcc0, 0xc0c0, 0xdc80, 0xdcc0, 0xc0c0, 0xb9c0, 0xb980, 0xb9c0, 0xa560, 0xa5a0, 0xac80,
            0xed00, 0x8000, 0x8cc0, 0xbd00, 0xe380, 0xbdc0, 0xe500, 0xe500, 0xe880, 0xe8c0, 0xe8c0,
            0xe800, 0xe5c0, 0xe5c0, 0xe940, 0xe980, 0xe940, 0xe980, 0xbd40, 0x8c80, 0xa080, 0x8000,
            0xa980, 0xbd00, 0xbdc0, 0xb400, 0xa880, 0xedc0,
        ];
        const LINK_DMA_SOURCES3: [u16; 27] = [
            0x9a40, 0x9e00, 0x9d20, 0x9f20, 0x9b20, 0xbc20, 0xbc20, 0xbe20, 0xbe20, 0xbe00, 0xbe00,
            0xbe00, 0xbe00, 0xa540, 0xa540, 0xa540, 0xa540, 0xbc00, 0xbc00, 0xbc00, 0xbc00, 0xa740,
            0xa740, 0xa740, 0xa740, 0xe780, 0xe780,
        ];
        const LINK_DMA_SOURCES4: [u16; 8] = [
            0x9000, 0x9020, 0x9060, 0x91e0, 0x90a0, 0x90c0, 0x9100, 0x9140,
        ];
        const LINK_DMA_SOURCES5: [u16; 3] = [0x9300, 0x9340, 0x9380];
        const LINK_DMA_SOURCES6: [u16; 128] = [
            0x9480, 0x94c0, 0x94e0, 0x95c0, 0x9500, 0x9520, 0x9540, 0x9480, 0x9640, 0x9680, 0x96a0,
            0x9780, 0x96c0, 0x96e0, 0x9700, 0x9480, 0x9800, 0x9840, 0x98a0, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9ac0, 0x9b00, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9bc0,
            0x9c00, 0x9c40, 0x9c80, 0x9cc0, 0x9d00, 0x9d40, 0x9480, 0x9f40, 0x9f80, 0x9fc0, 0x9fe0,
            0xa000, 0x9480, 0x9480, 0x9480, 0xa100, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x98c0, 0x9900,
            0x99c0, 0x99e0, 0x9a00, 0x9a20, 0x9a40, 0x9a60, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9a80, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
        ];
        const LINK_DMA_SOURCES7: [u16; 16] = [
            0x00e0, 0x00e0, 0x0060, 0x0080, 0x01c0, 0x00e0, 0x0040, 0x0000, 0x0080, 0x0000, 0x0040,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ];
        const LINK_DMA_SOURCES8: [u16; 4] = [0xa480, 0xa4c0, 0xa500, 0xa540];
        const LINK_DMA_CTRS0: [u16; 6] = [14, 4, 6, 16, 6, 8];
        const LINK_DMA_SOURCES9: [u16; 15] = [
            0, 0x20, 0x40, 0, 0x20, 0x40, 0, 0x40, 0x80, 0, 0x40, 0x80, 0xb340, 0xb400, 0xb4c0,
        ];
        fn link_dma_table_value(table: &[u16], index: usize, table_name: &str) -> u16 {
            // C indexes these static DMA tables directly; this keeps an invalid
            // translated index from becoming a silent wrong DMA source.
            table.get(index).copied().unwrap_or_else(|| {
                panic!(
                    "nmi_prepare_sprites {table_name} index {index} out of bounds; table_len={}",
                    table.len()
                )
            })
        }

        for i in 0..32 {
            self.ram[EXTENDED_OAM + i] = (self.ram[BYTEWISE_EXTENDED_OAM + 3 + i * 4] << 6)
                | (self.ram[BYTEWISE_EXTENDED_OAM + 2 + i * 4] << 4)
                | (self.ram[BYTEWISE_EXTENDED_OAM + 1 + i * 4] << 2)
                | self.ram[BYTEWISE_EXTENDED_OAM + i * 4];
        }

        let link_dma_graphics_index =
            (read_le_u16(&self.ram, LINK_DMA_GRAPHICS_INDEX) >> 1) as usize;
        let link_dma_var1 = (read_le_u16(&self.ram, LINK_DMA_VAR1) >> 1) as usize;
        let link_dma_var2 = (read_le_u16(&self.ram, LINK_DMA_VAR2) >> 1) as usize;
        let link_dma_var3 = (self.ram[LINK_DMA_SWORD_GRAPHICS_INDEX] >> 1) as usize;
        let link_dma_var4 = (self.ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] >> 1) as usize;
        let link_dma_var5 = self.ram[LINK_DMA_VAR5] as usize;
        let link_dma_var5_group = (self.ram[LINK_DMA_VAR5] >> 3) as usize;

        let source3 = link_dma_table_value(
            &LINK_DMA_SOURCES1,
            link_dma_graphics_index,
            "LINK_DMA_SOURCES1",
        );
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_3, source3);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_0,
            source3.wrapping_add(0x200),
        );

        let source4 = link_dma_table_value(
            &LINK_DMA_SOURCES2,
            link_dma_graphics_index,
            "LINK_DMA_SOURCES2",
        );
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_4, source4);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_1,
            source4.wrapping_add(0x200),
        );

        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_5,
            link_dma_table_value(&LINK_DMA_SOURCES3, link_dma_var1, "LINK_DMA_SOURCES3/var1"),
        );
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_2,
            link_dma_table_value(&LINK_DMA_SOURCES3, link_dma_var2, "LINK_DMA_SOURCES3/var2"),
        );

        let source6 = link_dma_table_value(&LINK_DMA_SOURCES4, link_dma_var3, "LINK_DMA_SOURCES4");
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_6, source6);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_11,
            source6.wrapping_add(0x180),
        );

        let source7 = if self.ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] == 0x8b {
            0xe099
        } else {
            link_dma_table_value(&LINK_DMA_SOURCES5, link_dma_var4, "LINK_DMA_SOURCES5")
        };
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_7, source7);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_12,
            source7.wrapping_add(0x00c0),
        );

        let source8 = link_dma_table_value(&LINK_DMA_SOURCES6, link_dma_var5, "LINK_DMA_SOURCES6");
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_8, source8);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_13,
            source8.wrapping_add(link_dma_table_value(
                &LINK_DMA_SOURCES7,
                link_dma_var5_group,
                "LINK_DMA_SOURCES7",
            )),
        );

        let source10 = LINK_DMA_SOURCES8[(self.ram[PUSHED_BLOCK_MODE] & 3) as usize];
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_10, source10);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_15,
            source10.wrapping_add(0x100),
        );

        if self.decrement_word(BG_TILE_ANIMATION_COUNTDOWN) == 0 {
            let overlay = self.ram[OVERWORLD_SCREEN_INDEX + 2] as u16;
            let countdown = if overlay == 0xb5 || overlay == 0xbc {
                0x17
            } else {
                9
            };
            write_le_u16(&mut self.ram, BG_TILE_ANIMATION_COUNTDOWN, countdown);

            let mut source_offset =
                read_le_u16(&self.ram, LINK_DMA_SOURCE_OFFSET).wrapping_add(0x400);
            if source_offset == 0x0c00 {
                source_offset = 0;
            }
            write_le_u16(&mut self.ram, LINK_DMA_SOURCE_OFFSET, source_offset);
            write_le_u16(
                &mut self.ram,
                ANIMATED_TILE_DATA_SRC,
                0xa680u16.wrapping_add(source_offset),
            );
        }
        if self.decrement_word(LINK_DMA_COUNTDOWN) == 0 {
            let mut t = read_le_u16(&self.ram, LINK_DMA_TILE_OFFSET).wrapping_add(2);
            if t == 12 {
                t = 0;
            }
            write_le_u16(&mut self.ram, LINK_DMA_TILE_OFFSET, t);

            let index = (t >> 1) as usize;
            write_le_u16(&mut self.ram, LINK_DMA_COUNTDOWN, LINK_DMA_CTRS0[index]);
            let source9 = LINK_DMA_SOURCES9[index].wrapping_add(0xb280);
            write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_9, source9);
            write_le_u16(
                &mut self.ram,
                DMA_SOURCE_ADDR_14,
                source9.wrapping_add(0x60),
            );
        }

        let source16 = 0xb940u16.wrapping_add((self.ram[DMA_HEAD_POINTER] as u16).wrapping_mul(2));
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_16, source16);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_18,
            source16.wrapping_add(0x200),
        );

        let source17 = 0xb940u16.wrapping_add((self.ram[DMA_BODY_POINTER] as u16).wrapping_mul(2));
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_17, source17);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_19,
            source17.wrapping_add(0x200),
        );

        let source20 = 0xb540u16.wrapping_add((self.ram[FLAG_TRAVEL_BIRD] as u16).wrapping_mul(2));
        write_le_u16(&mut self.ram, DMA_SOURCE_ADDR_20, source20);
        write_le_u16(
            &mut self.ram,
            DMA_SOURCE_ADDR_21,
            source20.wrapping_add(0x200),
        );
    }

    pub(super) fn module_main_routing(&mut self) {
        match self.frame_control_view().main_module() {
            0 => self.Module00_Intro(),
            1 => self.module01_file_select(),
            2 => self.module02_copy_file(),
            3 => self.module03_kill_file(),
            4 => self.module04_name_file(),
            5 => self.module05_load_file(),
            6 => self.module_pre_dungeon(),
            7 => self.module07_dungeon(),
            8 | 10 => self.Module08_OverworldLoad(),
            9 | 11 => self.Module09_Overworld(),
            12 => self.module_unknown0(),
            13 => self.module_unknown1(),
            14 => self.Module0E_Interface(),
            15 => self.Module0F_SpotlightClose(),
            16 => self.Module10_SpotlightOpen(),
            17 => self.Module11_DungeonFallingEntrance(),
            18 => self.Module12_GameOver(),
            19 => self.module13_boss_victory_pendant(),
            20 => self.module14_attract(),
            21 => self.module15_mirror_warp_from_aga(),
            22 => self.module16_boss_victory_crystal(),
            23 => self.module17_save_and_quit(),
            24 => self.module18_ganon_emerges(),
            25 => self.module19_triforce_room(),
            26 => self.module1_a_credits(),
            27 => self.Module1B_SpawnSelect(),
            _ => {}
        }
    }
}
