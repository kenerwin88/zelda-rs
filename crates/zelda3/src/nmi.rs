// Methods ported from zelda3/src/nmi.c and included inside ZeldaState.

use super::*;

impl ZeldaState {
    pub(super) fn interrupt_nmi(&mut self, input: u16) {
        self.interrupt_nmi_audio_parts_locked();

        if !self.game_state.display.nmi_update_is_latched() {
            self.latch_nmi_update();
            self.nmi_do_updates();
            self.nmi_read_joypads(input);
        }

        if self.game_state.display.nmi_thread_active {
            self.nmi_update_irqgfx();
            let stack = self.game_state.display.nmi_thread_stack_pointer;
            self.set_nmi_thread_stack_pointer(if stack != 0x1f31 { 0x1f31 } else { 0x01f2 });
            if self.nmi_poly_upload_deferred != 0 {
                self.nmi_poly_upload_deferred = self.nmi_poly_upload_deferred.saturating_sub(1);
                if self.nmi_poly_upload_deferred == 0 && self.nmi_poly_upload_started {
                    self.request_polyhedral_nmi_update();
                    self.nmi_poly_upload_from_deferred =
                        self.nmi_poly_deferred_upload_bypasses_latch;
                    self.nmi_poly_deferred_upload_bypasses_latch = false;
                }
            }
        }
        self.write_ppu_registers();
    }

    pub(super) fn interrupt_nmi_audio_parts_locked(&mut self) {
        let music_control = self.game_state.system_signals.music_control();
        if music_control != 0 && !self.zelda_is_playing_music_track_with_bug(music_control) {
            self.system_signals_mut()
                .set_last_music_control(music_control);
            self.zelda_play_msu_audio_track(music_control);
            if music_control < 0xf2 {
                self.system_signals_mut()
                    .set_current_music_control(music_control);
            }
            self.system_signals_mut().set_music_control(0);
        }

        let ambient_sound_effect = self.game_state.system_signals.ambient_sound_effect();
        if ambient_sound_effect != 0 {
            self.system_signals_mut()
                .save_ambient_sound_effect_as_last();
            self.zelda_apu_write(0x2141, ambient_sound_effect);
            self.system_signals_mut().clear_ambient_sound_effect();
        } else if self.zelda_apu_read(0x2141)
            == self.game_state.system_signals.last_ambient_sound_effect()
        {
            self.zelda_apu_write(0x2141, 0);
        }

        self.zelda_apu_write(0x2142, self.game_state.system_signals.sound_effect_1());
        self.zelda_apu_write(0x2143, self.game_state.system_signals.sound_effect_2());
        self.system_signals_mut().clear_sound_effect_1();
        self.system_signals_mut().clear_sound_effect_2();
    }

    pub(super) fn nmi_do_updates(&mut self) {
        if !self.game_state.display.core_updates_are_disabled() {
            self.nmi_core_link_graphics_update();

            let src_addr = self.game_state.display.animated_tile_data_source_usize();
            let dst = self
                .game_state
                .display
                .animated_tile_vram_destination_usize();
            if dst + 0x200 <= self.ppu.vram.len() && src_addr + 0x400 <= self.ram.len() {
                let data = self.animated_tile_dma_source_bytes().to_vec();
                for i in 0..0x200 {
                    self.ppu.vram[dst + i] = read_word_from_slice(&data, i * 2);
                }
            }
        }

        if self.game_state.system_signals.should_update_cgram() {
            for i in 0..0x100 {
                self.ppu.cgram[i] = self.game_state.display.palette_buffer.main_color(i);
            }
        }

        if self.game_state.system_signals.should_update_hud() {
            let dst = self
                .game_state
                .display
                .message_dma_destination_address_usize();
            if dst + 165 <= self.ppu.vram.len() {
                let hud_buf = self.message_dma_tile_indices().to_vec();
                for i in 0..165 {
                    self.ppu.vram[dst + i] = read_word_from_slice(&hud_buf, i * 2);
                }
            }
        }

        self.system_signals_mut().clear_hud_update_flag();
        self.system_signals_mut().clear_cgram_update_flag();
        let oam_buf = self.sprite_oam_shadow_buffer().to_vec();
        for i in 0..self.ppu.oam.len() {
            self.ppu.oam[i] = read_word_from_slice(&oam_buf, i * 2);
        }

        if self.game_state.display.has_bg_vram_load() {
            match self.game_state.display.bg_vram_load_mode {
                1 => {
                    let stripes = self.vram_upload_buffer_remaining().to_vec();
                    self.handle_stripes14_slice(&stripes);
                }
                2 => {
                    let stripes = self.tilemap_upload_stripe_buffer().to_vec();
                    self.handle_stripes14_slice(&stripes);
                }
                3 => {
                    if let Some(stripes) = self
                        .assets
                        .as_ref()
                        .and_then(|assets| assets.asset(99))
                        .map(Vec::from)
                    {
                        self.handle_stripes14_slice(&stripes);
                    }
                }
                4 => {
                    let stripes = self.secondary_stripe_upload_buffer().to_vec();
                    self.handle_stripes14_slice(&stripes);
                }
                5..=9 => {
                    let asset = 95 + self.game_state.display.bg_vram_load_mode as usize;
                    if let Some(stripes) = self
                        .assets
                        .as_ref()
                        .and_then(|assets| assets.asset(asset))
                        .map(Vec::from)
                    {
                        self.handle_stripes14_slice(&stripes);
                    }
                }
                _ => panic!(
                    "invalid nmi_load_bg_from_vram {}",
                    self.game_state.display.bg_vram_load_mode
                ),
            }
            if self.game_state.display.bg_vram_load_mode == 1 {
                self.clear_vram_upload_cursor();
            }
            self.clear_bg_vram_load_mode();
        }

        if self.game_state.display.has_pending_tilemap_update() {
            let dst = self
                .game_state
                .display
                .pending_tilemap_update_vram_destination();
            let src_data = self.pending_tilemap_update_source_data().to_vec();
            if src_data.len() >= 0x200 {
                self.copy_to_vram_slice(dst, &src_data, 0x200);
            }
            self.clear_pending_tilemap_update_destination();
        }

        if self.game_state.display.has_nmi_copy_packets_request() {
            self.NMI_CopyPackets();
            self.clear_nmi_copy_packets_request();
            self.clear_core_update_disable_flag();
        }

        let nmi_subroutine_index = self.take_pending_nmi_subroutine();
        match nmi_subroutine_index {
            0 => self.nmi_upload_tilemap_do_nothing(),
            1 => self.nmi_upload_tilemap(),
            2 => self.nmi_upload_bg3_text(),
            3 => self.nmi_update_ow_scroll(),
            4 => self.nmi_update_subscreen_overlay(),
            5 => self.nmi_update_bg1_wall(),
            6 => self.nmi_tile_map_nothing(),
            7 => self.nmi_update_load_light_world_map(),
            8 => self.nmi_update_bg2_left(),
            9 => self.nmi_update_bg_char3and4(),
            10 => self.nmi_update_bg_char5and6(),
            11 => self.nmi_update_bg_char_half(),
            12 => self.nmi_upload_subscreen_overlay_latter(),
            13 => self.nmi_upload_subscreen_overlay_former(),
            14 => self.nmi_update_bg_char0(),
            15 => self.nmi_update_bg_char1(),
            16 => self.nmi_update_bg_char2(),
            17 => self.nmi_update_bg_char3(),
            18 => self.nmi_update_obj_char0(),
            19 => self.nmi_update_obj_char2(),
            20 => self.nmi_update_obj_char3(),
            21 => self.nmi_upload_dark_world_map(),
            22 => self.nmi_upload_game_over_text(),
            23 => self.nmi_update_peg_tiles(),
            24 => self.nmi_update_star_tiles(),
            _ => panic!("invalid nmi_subroutine_index {}", nmi_subroutine_index),
        }
    }

    pub(super) fn nmi_upload_tilemap(&mut self) {
        let target = NMI_VRAM_ADDRS[self.game_state.display.nmi_load_target_page() as usize] << 8;
        if target + 0x400 <= self.ppu.vram.len() {
            let buf = self.tilemap_upload_stripe_buffer().to_vec();
            for i in 0..0x400 {
                self.ppu.vram[target + i] = read_word_from_slice(&buf, i * 2);
            }
        }
        self.clear_vram_upload_cursor();
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_upload_tilemap_do_nothing(&mut self) {}

    pub(super) fn nmi_update_ow_scroll(&mut self) {
        let data = self.nmi_vram_packet_buffer().to_vec();
        if data.len() < 2 {
            return;
        }
        let f = read_word_from_slice(&data, 0);
        let step = if f & 0x8000 != 0 { 32 } else { 1 };
        let len = (f & 0x3fff) as usize;
        let words = len >> 1;
        let mut pos = 2usize;
        loop {
            if pos + 2 + len > data.len() {
                break;
            }
            let mut dst = read_word_from_slice(&data, pos) as usize;
            pos += 2;
            for i in 0..words {
                if dst < self.ppu.vram.len() {
                    self.ppu.vram[dst] = read_word_from_slice(&data, pos + i * 2);
                }
                dst += step;
            }
            pos += len;
            if pos + 1 >= data.len() || data[pos + 1] & 0x80 != 0 {
                break;
            }
        }
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_subscreen_overlay(&mut self) {
        let data = self.dungeon_bg2_attribute_table().to_vec();
        self.nmi_handle_arbitrary_tile_map_addr_data(&data, 0, 0x80);
    }

    pub(super) fn nmi_upload_subscreen_overlay_former(&mut self) {
        let data = self.dungeon_bg2_attribute_table().to_vec();
        self.nmi_handle_arbitrary_tile_map_addr_data(&data, 0, 0x40);
    }

    pub(super) fn nmi_upload_subscreen_overlay_latter(&mut self) {
        let data = self.dungeon_bg1_attribute_table().to_vec();
        self.nmi_handle_arbitrary_tile_map_addr_data(&data, 0x40, 0x80);
    }

    pub(super) fn nmi_handle_arbitrary_tile_map_addr_data(
        &mut self,
        src_data: &[u8],
        mut i: usize,
        i_end: usize,
    ) {
        let mut offset = 0usize;
        loop {
            let dst = self.arbitrary_tilemap_destination(i >> 1) as usize;
            if offset + 0x80 <= src_data.len() {
                self.copy_to_vram_slice(dst, &src_data[offset..], 0x80);
            }
            offset += 0x80;
            i += 2;
            if i == i_end {
                break;
            }
        }
        self.clear_core_update_disable_flag();
    }

    #[rustfmt::skip]
    pub(super) fn nmi_handle_arbitrary_tile_map(&mut self, src: *const u8, mut i: i32, i_end: i32) {
        let mut offset = 0usize;
        loop {
            let dst = self.arbitrary_tilemap_destination((i as usize) >> 1) as usize;
            let chunk = unsafe { std::slice::from_raw_parts(src.add(offset), 0x80) };
            self.copy_to_vram_slice(dst, chunk, 0x80);
            offset += 0x80;
            i += 2;
            if i == i_end {
                break;
            }
        }
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_bg1_wall(&mut self) {
        let target = self.game_state.display.nmi_load_target_address as usize;
        let top_buf = self.bg1_wall_top_tilemap_buffer().to_vec();
        let bottom_buf = self.bg1_wall_bottom_tilemap_buffer().to_vec();
        self.copy_to_vram_vertical_slice(target, &top_buf, 0x40);
        self.copy_to_vram_vertical_slice(target + 0x800, &bottom_buf, 0x40);
    }

    pub(super) fn nmi_tile_map_nothing(&mut self) {}

    pub(super) fn nmi_update_bg2_left(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        let buf1 = self.background_character_secondary_buffer().to_vec();
        self.copy_to_vram_slice(0, &buf, 0x800);
        self.copy_to_vram_slice(0x800, &buf1, 0x800);
    }

    pub(super) fn nmi_update_bg_char3and4(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x2c00, &buf, 0x1000);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_bg_char5and6(&mut self) {
        let buf = self.background_character_half_buffer().to_vec();
        self.copy_to_vram_slice(0x3400, &buf, 0x1000);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_bg_char0(&mut self) {
        self.nmi_run_tile_map_update_dma(0x2000);
    }

    pub(super) fn nmi_update_bg_char1(&mut self) {
        self.nmi_run_tile_map_update_dma(0x2800);
    }

    pub(super) fn nmi_update_bg_char2(&mut self) {
        self.nmi_run_tile_map_update_dma(0x3000);
    }

    pub(super) fn nmi_update_bg_char3(&mut self) {
        self.nmi_run_tile_map_update_dma(0x3800);
    }

    pub(super) fn nmi_update_obj_char0(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x4400, &buf, 0x800);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_obj_char2(&mut self) {
        self.nmi_run_tile_map_update_dma(0x5000);
    }

    pub(super) fn nmi_update_obj_char3(&mut self) {
        self.nmi_run_tile_map_update_dma(0x5800);
    }

    pub(super) fn nmi_run_tile_map_update_dma(&mut self, dst: usize) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(dst, &buf, 0x1000);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_upload_dark_world_map(&mut self) {
        let data = self.tilemap_upload_stripe_buffer().to_vec();
        let mut src = 0usize;
        let mut dst = 0x810usize;
        for _ in 0..0x20 {
            self.copy_to_vram_low_slice(&data[src..], dst, 0x20);
            src += 0x20;
            dst += 0x80;
        }
    }

    pub(super) fn nmi_upload_game_over_text(&mut self) {
        let buf = self.game_over_text_tile_buffer().to_vec();
        let tail_buf = self.game_over_text_tail_tile_buffer().to_vec();
        self.copy_to_vram_slice(0x7800, &buf, 0x800);
        self.copy_to_vram_slice(0x7d00, &tail_buf, 0x600);
    }

    pub(super) fn nmi_update_peg_tiles(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x3d00, &buf, 0x100);
    }

    pub(super) fn nmi_update_star_tiles(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x3ed0, &buf, 0x40);
    }

    pub(super) fn NMI_CopyPackets(&mut self) {
        let data = self.nmi_vram_packet_buffer().to_vec();
        let mut pos = 0usize;
        while pos + 4 <= data.len() && read_word_from_slice(&data, pos) != 0xffff {
            let dst = read_word_from_slice(&data, pos) as usize;
            let vmain = data[pos + 2];
            let len = data[pos + 3] as usize;
            pos += 4;
            if pos + len > data.len() {
                break;
            }
            if vmain == 0x80 {
                self.copy_to_vram_slice(dst, &data[pos..], len);
            } else if vmain == 0x81 {
                self.copy_to_vram_vertical_slice(dst, &data[pos..], len);
            } else {
                panic!("invalid NMI packet vmain {vmain:#04x}");
            }
            pos += len;
        }
    }

    pub(super) fn nmi_core_link_graphics_update(&mut self) {
        if let Some(link_graphics) = self.asset_raw(57).map(Vec::from) {
            for (dst, source, len) in [
                (0x4100, LinkDmaSourceSlot::BodyBottom, 0x40),
                (0x4120, LinkDmaSourceSlot::HeadBottom, 0x40),
                (0x4140, LinkDmaSourceSlot::HandRight, 0x20),
                (0x4000, LinkDmaSourceSlot::BodyTop, 0x40),
                (0x4020, LinkDmaSourceSlot::HeadTop, 0x40),
                (0x4040, LinkDmaSourceSlot::HandLeft, 0x20),
            ] {
                self.copy_asset_bytes_to_vram(dst, &link_graphics, source, len);
            }
        }

        for (dst, source, len) in [
            (0x4050, LinkDmaSourceSlot::SwordUpper, 0x40),
            (0x4070, LinkDmaSourceSlot::ShieldUpper, 0x40),
            (0x4090, LinkDmaSourceSlot::AuxUpper, 0x40),
            (0x40b0, LinkDmaSourceSlot::AnimatedTileUpper, 0x20),
            (0x40c0, LinkDmaSourceSlot::PushUpper, 0x40),
            (0x4150, LinkDmaSourceSlot::SwordLower, 0x40),
            (0x4170, LinkDmaSourceSlot::ShieldLower, 0x40),
            (0x4190, LinkDmaSourceSlot::AuxLower, 0x40),
            (0x41b0, LinkDmaSourceSlot::AnimatedTileLower, 0x20),
            (0x41c0, LinkDmaSourceSlot::PushLower, 0x40),
            (0x4200, LinkDmaSourceSlot::HeadPointerUpper, 0x40),
            (0x4220, LinkDmaSourceSlot::BodyPointerUpper, 0x40),
        ] {
            self.copy_ram_bytes_to_vram(dst, source, len);
        }
        self.copy_ram_bytes_to_vram_absolute(0x4240, 0xbd40, 0x40);
        for (dst, source) in [
            (0x4300, LinkDmaSourceSlot::HeadPointerLower),
            (0x4320, LinkDmaSourceSlot::BodyPointerLower),
        ] {
            self.copy_ram_bytes_to_vram(dst, source, 0x40);
        }
        self.copy_ram_bytes_to_vram_absolute(0x4340, 0xbd80, 0x40);

        if self.game_state.display.has_travel_bird_tile_upload() {
            for (dst, source) in [
                (0x40e0, LinkDmaSourceSlot::TravelBirdUpper),
                (0x41e0, LinkDmaSourceSlot::TravelBirdLower),
            ] {
                self.copy_ram_bytes_to_vram(dst, source, 0x40);
            }
        }
    }

    pub(super) fn copy_to_vram_slice(&mut self, dstv: usize, src: &[u8], len: usize) {
        self.copy_bytes_to_vram(dstv, src, 0, len);
    }

    #[rustfmt::skip]
    pub(super) fn copy_to_vram(&mut self, dstv: usize, src: *const u8, len: usize) {
        let src = unsafe { std::slice::from_raw_parts(src, len) };
        self.copy_to_vram_slice(dstv, src, len);
    }

    pub(super) fn copy_to_vram_vertical_slice(&mut self, mut dstv: usize, src: &[u8], len: usize) {
        assert_eq!(len & 1, 0);
        let words = len >> 1;
        for i in 0..words {
            if dstv < self.ppu.vram.len() && i * 2 + 1 < src.len() {
                self.ppu.vram[dstv] = read_word_from_slice(src, i * 2);
            }
            dstv += 32;
        }
    }

    #[rustfmt::skip]
    pub(super) fn copy_to_vram_vertical(&mut self, dstv: usize, src: *const u8, len: usize) {
        let src = unsafe { std::slice::from_raw_parts(src, len) };
        self.copy_to_vram_vertical_slice(dstv, src, len);
    }

    pub(super) fn copy_to_vram_low_slice(&mut self, src: &[u8], addr: usize, num: usize) {
        for i in 0..num {
            if addr + i < self.ppu.vram.len() && i < src.len() {
                self.ppu.vram[addr + i] = (self.ppu.vram[addr + i] & !0xff) | src[i] as u16;
            }
        }
    }

    #[rustfmt::skip]
    pub(super) fn copy_to_vram_low(&mut self, src: *const u8, addr: usize, num: usize) {
        let src = unsafe { std::slice::from_raw_parts(src, num) };
        self.copy_to_vram_low_slice(src, addr, num);
    }

    pub(super) fn copy_asset_bytes_to_vram(
        &mut self,
        dst_word: usize,
        source: &[u8],
        source_slot: LinkDmaSourceSlot,
        len: usize,
    ) {
        let source_addr = self.game_state.display.link_dma_source(source_slot) as usize;
        if source_addr < 0x8000 {
            return;
        }
        self.copy_bytes_to_vram(dst_word, source, source_addr - 0x8000, len);
    }

    pub(super) fn copy_ram_bytes_to_vram(
        &mut self,
        dst_word: usize,
        source_slot: LinkDmaSourceSlot,
        len: usize,
    ) {
        let source_addr = self.game_state.display.link_dma_source(source_slot) as usize;
        self.copy_ram_bytes_to_vram_absolute(dst_word, source_addr, len);
    }

    pub(super) fn copy_ram_bytes_to_vram_absolute(
        &mut self,
        dst_word: usize,
        source_addr: usize,
        len: usize,
    ) {
        let source = self.vram_dma_source_bytes(source_addr, len).to_vec();
        if source.len() < len {
            return;
        }
        self.copy_bytes_to_vram(dst_word, &source, 0, len);
    }

    pub(super) fn copy_bytes_to_vram(
        &mut self,
        dst_word: usize,
        source: &[u8],
        source_addr: usize,
        len: usize,
    ) {
        if source_addr + len > source.len() || dst_word + len.div_ceil(2) > self.ppu.vram.len() {
            return;
        }
        for i in 0..len {
            let word_idx = dst_word + i / 2;
            let byte = source[source_addr + i] as u16;
            if i & 1 == 0 {
                self.ppu.vram[word_idx] = (self.ppu.vram[word_idx] & 0xff00) | byte;
            } else {
                self.ppu.vram[word_idx] = (self.ppu.vram[word_idx] & 0x00ff) | (byte << 8);
            }
        }
    }

    pub(super) fn nmi_update_irqgfx(&mut self) {
        if self.game_state.display.has_pending_polyhedral_update() {
            let poly_buf = self.polyhedral_tile_buffer().to_vec();
            let mut display_vram = None;
            for i in 0..0x400 {
                let dst = 0x5800 + i;
                let value = read_word_from_slice(&poly_buf, i * 2);
                if self.ppu.vram[dst] != value && display_vram.is_none() {
                    display_vram = Some(self.ppu.vram.clone());
                }
                self.ppu.vram[dst] = value;
            }
            if self.nmi_poly_upload_from_deferred {
                self.ppu.obj_vram_latch = None;
            } else if let Some(display_vram) = display_vram {
                self.obj_vram_latch_generation = self.obj_vram_latch_generation.wrapping_add(1);
                self.ppu.obj_vram_latch = Some(display_vram);
            }
            self.nmi_poly_upload_from_deferred = false;
            self.clear_pending_polyhedral_update();
        }
    }

    pub(super) fn nmi_update_bg_char_half(&mut self) {
        let dst = self.game_state.display.nmi_load_target_page() as usize * 256;
        let buf = self.background_character_half_buffer().to_vec();
        for i in 0..0x200 {
            self.ppu.vram[dst + i] = read_word_from_slice(&buf, i * 2);
        }
    }

    pub(super) fn nmi_upload_bg3_text(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        for i in 0..0x3f0 {
            self.ppu.vram[0x7c00 + i] = read_word_from_slice(&buf, i * 2);
        }
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_load_light_world_map(&mut self) {
        const LIGHT_WORLD_TILEMAP_DSTS: [usize; 4] = [0, 0x20, 0x1000, 0x1020];
        if let Some(tilemap) = self.asset_raw(67).map(Vec::from) {
            let mut src = 0usize;
            for dst_base in LIGHT_WORLD_TILEMAP_DSTS {
                let mut dst = dst_base;
                for _ in 0..0x20 {
                    for i in 0..0x20 {
                        if src + i < tilemap.len() {
                            self.ppu.vram[dst + i] =
                                (self.ppu.vram[dst + i] & 0xff00) | tilemap[src + i] as u16;
                        }
                    }
                    src += 0x20;
                    dst += 0x80;
                }
            }
        }
    }

    pub(super) fn handle_stripes14_slice(&mut self, mut stripes: &[u8]) {
        while stripes.first().copied().unwrap_or(0x80) & 0x80 == 0 {
            if stripes.len() < 4 {
                return;
            }
            let vmem_addr = ((stripes[0] as u16) << 8) | stripes[1] as u16;
            let vertical = stripes[2] & 0x80 != 0;
            let is_memset = stripes[2] & 0x40 != 0;
            let len = ((((stripes[2] as u16) << 8) | stripes[3] as u16) & 0x3fff) as usize + 1;
            stripes = &stripes[4..];

            if !vertical {
                if is_memset {
                    if stripes.len() < 2 {
                        return;
                    }
                    let value = stripes[0] as u16 | ((stripes[1] as u16) << 8);
                    for i in 0..((len + 1) >> 1) {
                        self.ppu.vram[vmem_addr as usize + i] = value;
                    }
                    stripes = &stripes[2..];
                } else {
                    if stripes.len() < len {
                        return;
                    }
                    for i in 0..len {
                        self.write_vram_byte(vmem_addr as usize * 2 + i, stripes[i]);
                    }
                    stripes = &stripes[len..];
                }
            } else if is_memset {
                if stripes.len() < 2 {
                    return;
                }
                let value = stripes[0] as u16 | ((stripes[1] as u16) << 8);
                for i in 0..((len + 1) >> 1) {
                    self.ppu.vram[vmem_addr as usize + i * 32] = value;
                }
                stripes = &stripes[2..];
            } else {
                assert_eq!(len & 1, 0);
                let words = len >> 1;
                if stripes.len() < words * 2 {
                    return;
                }
                for i in 0..words {
                    self.ppu.vram[vmem_addr as usize + i * 32] =
                        read_word_from_slice(stripes, i * 2);
                }
                stripes = &stripes[words * 2..];
            }
        }
    }

    #[rustfmt::skip]
    pub(super) fn handle_stripes14(&mut self, p: *const u8) {
        let stripes = unsafe { std::slice::from_raw_parts(p, 0x4000) };
        self.handle_stripes14_slice(stripes);
    }

    pub(super) fn write_ppu_registers(&mut self) {
        self.zelda_ppu_write(0x2123, self.game_state.display.bg12_window_selection);
        self.zelda_ppu_write(0x2124, self.game_state.display.bg34_window_selection);
        self.zelda_ppu_write(
            0x2125,
            self.game_state.display.object_color_window_selection,
        );
        self.zelda_ppu_write(
            0x2130,
            self.game_state
                .display
                .palette_filter
                .color_window_selection(),
        );
        self.zelda_ppu_write(
            0x2131,
            self.game_state.display.palette_filter.color_math_control(),
        );
        self.zelda_ppu_write(
            0x2132,
            self.game_state.display.palette_filter.fixed_color_red(),
        );
        self.zelda_ppu_write(
            0x2132,
            self.game_state.display.palette_filter.fixed_color_green(),
        );
        self.zelda_ppu_write(
            0x2132,
            self.game_state.display.palette_filter.fixed_color_blue(),
        );
        self.zelda_ppu_write(0x212c, self.game_state.display.main_screen_layers);
        self.zelda_ppu_write(0x212d, self.game_state.display.sub_screen_layers);
        self.zelda_ppu_write(0x212e, self.game_state.display.main_screen_window_layers);
        self.zelda_ppu_write(0x212f, self.game_state.display.sub_screen_window_layers);
        self.zelda_ppu_write(
            0x210d,
            self.game_state.display.ppu_scroll_copy.bg1_h_copy_low(),
        );
        self.zelda_ppu_write(0x210d, self.game_state.display.ppu_scroll_copy.bg1_h_high());
        self.zelda_ppu_write(
            0x210e,
            self.game_state.display.ppu_scroll_copy.bg1_v_copy_low(),
        );
        self.zelda_ppu_write(0x210e, self.game_state.display.ppu_scroll_copy.bg1_v_high());
        self.zelda_ppu_write(
            0x210f,
            self.game_state.display.ppu_scroll_copy.bg2_h_copy_low(),
        );
        self.zelda_ppu_write(0x210f, self.game_state.display.ppu_scroll_copy.bg2_h_high());
        self.zelda_ppu_write(
            0x2110,
            self.game_state.display.ppu_scroll_copy.bg2_v_copy_low(),
        );
        self.zelda_ppu_write(0x2110, self.game_state.display.ppu_scroll_copy.bg2_v_high());
        self.zelda_ppu_write(
            0x2111,
            self.game_state.display.ppu_scroll_copy.bg3_h_copy2_low(),
        );
        self.zelda_ppu_write(0x2111, self.game_state.display.ppu_scroll_copy.bg3_h_high());
        self.zelda_ppu_write(
            0x2112,
            self.game_state.display.ppu_scroll_copy.bg3_v_copy2_low(),
        );
        self.zelda_ppu_write(0x2112, self.game_state.display.ppu_scroll_copy.bg3_v_high());
        self.zelda_ppu_write(0x2100, self.game_state.display.screen_brightness);
        self.zelda_ppu_write(0x2106, self.game_state.display.mosaic_copy);
        self.zelda_ppu_write(0x2105, self.game_state.display.bg_mode);
        if self.game_state.display.bg_mode & 7 == 7 {
            self.zelda_ppu_write(0x211c, 0);
            self.zelda_ppu_write(0x211c, 0);
            self.zelda_ppu_write(0x211d, 0);
            self.zelda_ppu_write(0x211d, 0);
            self.zelda_ppu_write(
                0x211f,
                self.game_state.display.ppu_scroll_copy.mode7_center_x() as u8,
            );
            self.zelda_ppu_write(
                0x211f,
                self.game_state
                    .display
                    .ppu_scroll_copy
                    .mode7_center_x_high(),
            );
            self.zelda_ppu_write(
                0x2120,
                self.game_state.display.ppu_scroll_copy.mode7_center_y() as u8,
            );
            self.zelda_ppu_write(
                0x2120,
                self.game_state
                    .display
                    .ppu_scroll_copy
                    .mode7_center_y_high(),
            );
        }
        self.zelda_ppu_write(0x210b, 0x22);
        self.zelda_ppu_write(0x210c, 0x07);
    }

    pub(super) fn write_vram_byte(&mut self, byte_offset: usize, value: u8) {
        let word = byte_offset >> 1;
        if word >= self.ppu.vram.len() {
            return;
        }
        let current = self.ppu.vram[word];
        self.ppu.vram[word] = if byte_offset & 1 == 0 {
            (current & 0xff00) | value as u16
        } else {
            (current & 0x00ff) | ((value as u16) << 8)
        };
    }

    pub(super) fn nmi_read_joypads(&mut self, joypad_input: u16) {
        let mut both = joypad_input;
        let mut reversed = 0u16;
        for _ in 0..16 {
            reversed = reversed.wrapping_mul(2).wrapping_add(both & 1);
            both >>= 1;
        }
        let r0 = reversed as u8;
        let r1 = (reversed >> 8) as u8;

        let last2_l = self.game_state.player.follower_link.joypad1l_last2();
        let filtered_joypad_l = (r0 ^ last2_l) & r0;
        self.follower_link_state_mut().set_joypad1l_last(r0);
        self.follower_link_state_mut()
            .set_filtered_joypad_l(filtered_joypad_l);
        self.follower_link_state_mut().set_joypad1l_last2(r0);

        let last2_h = self.game_state.player.follower_link.joypad1h_last2();
        let filtered_joypad_h = (r1 ^ last2_h) & r1;
        self.follower_link_state_mut().set_joypad1h_last(r1);
        self.follower_link_state_mut()
            .set_filtered_joypad_h(filtered_joypad_h);
        self.follower_link_state_mut().set_joypad1h_last2(r1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::constants::{ANIMATED_TILE_DATA_SRC, ANIMATED_TILE_VRAM_ADDR};

    #[test]
    fn nmi_core_update_copies_animated_tiles_even_from_zero_source() {
        let mut s = ZeldaState::new();
        write_le_u16(&mut s.ram, ANIMATED_TILE_DATA_SRC, 0);
        write_le_u16(&mut s.ram, ANIMATED_TILE_VRAM_ADDR, 0x6000);
        write_le_u16(&mut s.ram, 0, 0x1234);
        write_le_u16(&mut s.ram, 2, 0x5678);
        s.sync_native_game_state_from_ram();

        s.nmi_do_updates();

        assert_eq!(s.ppu.vram[0x6000], 0x1234);
        assert_eq!(s.ppu.vram[0x6001], 0x5678);
    }

    #[test]
    #[should_panic(expected = "invalid nmi_subroutine_index")]
    fn nmi_do_updates_panics_on_invalid_subroutine_like_c_table_index() {
        let mut s = ZeldaState::new();
        s.set_core_update_disable_flag(1);
        s.set_pending_nmi_subroutine(25);

        s.nmi_do_updates();
    }

    #[test]
    #[should_panic(expected = "invalid NMI packet vmain")]
    fn nmi_copy_packets_panics_on_invalid_vmain_like_c_assert() {
        let mut s = ZeldaState::new();
        write_le_u16(&mut s.ram, 0x1100, 0x2000);
        s.ram[0x1102] = 0x82;
        s.ram[0x1103] = 0;

        s.NMI_CopyPackets();
    }

    #[test]
    fn nmi_copy_packets_reads_uvram_data_not_uvram_header() {
        let mut s = ZeldaState::new();
        write_le_u16(&mut s.ram, 0x1000, 0);
        write_le_u16(&mut s.ram, 0x1002, 0x207f);
        write_le_u16(&mut s.ram, 0x1100, 0x2000);
        write_le_u16(&mut s.ram, 0x1102, 0x0480);
        s.ram[0x1104..0x1108].copy_from_slice(&[0x34, 0x12, 0x78, 0x56]);
        write_le_u16(&mut s.ram, 0x1108, 0xffff);

        s.NMI_CopyPackets();

        assert_eq!(s.ppu.vram[0x2000], 0x1234);
        assert_eq!(s.ppu.vram[0x2001], 0x5678);
    }
}
