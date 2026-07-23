// Methods ported from zelda3/src/nmi.c and included inside ZeldaState.

use super::*;
use crate::game_output::{AudioSfxBank, EngineAudioCommand};

// Snes9x's DMA trace at ROM $008aa2/$008adb: first boot NMI source $7e:0000.
const FIRST_BOOT_NMI_DMA_SOURCE: [u8; 0x40] = [
    0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x80, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

impl ZeldaState {
    /// Optional hot-reloaded parity experiment for NMI publication timing.
    ///
    /// `config/parity-runtime.toml` is read at each NMI (or the path named by
    /// `ZELDA3_PARITY_RUNTIME_CONFIG`). A rule only holds the NMI at one host
    /// frame and must also match the live module/submodule/pending work, so it
    /// cannot mask an unrelated rendering mismatch. This is deliberately a
    /// scheduler control point; it never edits PPU pixels or state.
    fn parity_runtime_hold_nmi_this_frame(&self) -> bool {
        self.parity_runtime_nmi_rule_matches("hold_nmi")
    }

    /// Match an exact NMI state against a hot-reloaded parity-policy rule.
    /// Prefixes keep independent experiments from being combined accidentally.
    pub(crate) fn parity_runtime_nmi_rule_matches(&self, prefix: &str) -> bool {
        let path = std::env::var_os("ZELDA3_PARITY_RUNTIME_CONFIG")
            .unwrap_or_else(|| "config/parity-runtime.toml".into());
        let Ok(text) = std::fs::read_to_string(path) else {
            return false;
        };
        let value = |name: &str| {
            text.lines().find_map(|line| {
                let line = line.split('#').next()?.trim();
                let (key, value) = line.split_once('=')?;
                (key.trim() == name).then(|| {
                    let value = value.trim().trim_matches('"');
                    if let Some(hex) = value.strip_prefix("0x") {
                        u32::from_str_radix(hex, 16).ok()
                    } else {
                        value.parse().ok()
                    }
                })?
            })
        };
        matches!(
            (
                value(&format!("{prefix}_host_frame")),
                value(&format!("{prefix}_main")),
                value(&format!("{prefix}_sub")),
                value(&format!("{prefix}_pending")),
                value(&format!("{prefix}_disable")),
            ),
            (Some(host), Some(main), Some(sub), Some(pending), Some(disable))
                if host == self.frame_ctr_dbg
                    && main == u32::from(self.game_state.frame.main_module)
                    && sub == u32::from(self.game_state.frame.submodule)
                    && pending == u32::from(self.game_state.display.pending_nmi_subroutine)
                    && disable == u32::from(self.game_state.display.core_update_disable_flag)
        )
    }

    fn parity_runtime_defer_pending_nmi_this_frame(&self) -> bool {
        self.parity_runtime_nmi_rule_matches("defer_pending_nmi")
    }

    pub(super) fn interrupt_nmi(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        defer_bg_vram_upload: bool,
    ) {
        let trace_nmi = std::env::var_os("ZELDA3_DEBUG_NMI_LATCH").is_some();
        if trace_nmi {
            let frame = self.game_state.frame;
            eprintln!(
                "nmi_before host={} main={:02x} sub={:02x} latch={} pending={} target={:04x} disable={:02x} link_tile_src={:04x} ram0000={:02x}{:02x}{:02x} vram40b0={:04x}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
                self.live_link_dma_source(LinkDmaSourceSlot::AnimatedTileUpper),
                self.ram[0x0000],
                self.ram[0x0001],
                self.ram[0x0002],
                self.ppu.vram.get(0x40b0).copied().unwrap_or(0),
            );
        }
        let joypad_already_sampled = std::mem::take(&mut self.joypad_sampled_before_main);
        let audio_already_processed = std::mem::take(&mut self.audio_nmi_processed_before_main);
        if !audio_already_processed {
            self.interrupt_nmi_audio_parts_locked();
        }

        if self.parity_runtime_hold_nmi_this_frame() {
            self.latch_nmi_update();
            if trace_nmi {
                eprintln!("nmi_runtime_policy held host={}", self.frame_ctr_dbg);
            }
        }
        if !self.game_state.display.nmi_update_is_latched() {
            let blank_scanlines = nmi_active_display_blank_scanlines_for_pending_work(
                self.game_state.display.core_updates_are_disabled(),
                self.game_state.system_signals.should_update_hud(),
                self.game_state.display.pending_nmi_subroutine,
            );
            self.nmi_forced_blank_scanlines_pending =
                self.nmi_forced_blank_scanlines_pending.max(blank_scanlines);
            self.latch_nmi_update();
            self.nmi_do_updates_from(oam_dma_source, defer_bg_vram_upload);
            if !joypad_already_sampled {
                self.nmi_read_joypads(input);
            }
        }

        if self.game_state.display.nmi_thread_active {
            let frame = self.game_state.frame;
            let timed_intro_poly_worker = self.rom_startup_timing()
                && rom_intro_poly_thread_is_active(frame.main_module, frame.submodule);
            if !timed_intro_poly_worker {
                self.nmi_update_irqgfx();
            }
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
        // The real NMI checks $012A (the interruptible-thread flag) right
        // before the register-mirror publication and, when the thread is
        // active, jumps straight to the epilogue: no $2123..$2132, TM/TS,
        // scroll, mode, or INIDISP writes happen that vblank, so the PPU keeps
        // last frame's register state. The attract history scene runs its
        // text-writer thread across many frames and relies on this (its
        // CGWSEL/COLDATA setup only reaches the PPU on thread-idle vblanks).
        // The intro (module 0) poly thread is emulated in coarse slices, so
        // our flag overstates the real at-vblank thread activity there; the
        // dedicated poly timing machinery covers that module instead.
        let thread_holds_registers =
            self.game_state.display.nmi_thread_active && self.game_state.frame.main_module == 0x14;
        if !thread_holds_registers {
            self.write_ppu_registers();
        }
        // After all CHR DMAs have settled this frame, refresh the OBJ CHR logical
        // sources by content hash so the off-VRAM sprite path resolves live cells.
        self.rehash_streamed_obj_sources();
        if trace_nmi {
            let frame = self.game_state.frame;
            eprintln!(
                "nmi_after host={} main={:02x} sub={:02x} latch={} pending={} target={:04x} disable={:02x} link_tile_src={:04x} vram40b0={:04x}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
                self.live_link_dma_source(LinkDmaSourceSlot::AnimatedTileUpper),
                self.ppu.vram.get(0x40b0).copied().unwrap_or(0),
            );
        }
    }

    pub(super) fn interrupt_nmi_audio_parts_locked(&mut self) {
        let music_control = self.game_state.system_signals.music_control();
        if music_control != 0 && !self.zelda_is_playing_music_track_with_bug(music_control) {
            self.set_last_music_control(music_control);
            self.zelda_play_msu_audio_track(music_control);
            if music_control < 0xf2 {
                self.set_current_music_control(music_control);
            }
            self.set_music_control(0);
        }

        let ambient_sound_effect = self.game_state.system_signals.ambient_sound_effect();
        if ambient_sound_effect != 0 {
            self.save_ambient_sound_effect_as_last();
            self.zelda_emit_audio_command(EngineAudioCommand::from_sfx_port_value(
                AudioSfxBank::Ambient,
                ambient_sound_effect,
            ));
            self.clear_ambient_sound_effect();
        } else if self.zelda_audio_command_acknowledged(EngineAudioCommand::from_sfx_port_value(
            AudioSfxBank::Ambient,
            self.game_state.system_signals.last_ambient_sound_effect(),
        )) {
            self.zelda_emit_audio_command(EngineAudioCommand::ClearSfx {
                bank: AudioSfxBank::Ambient,
            });
        }

        self.zelda_emit_audio_command(EngineAudioCommand::from_sfx_port_value(
            AudioSfxBank::Effect1,
            self.game_state.system_signals.sound_effect_1(),
        ));
        self.zelda_emit_audio_command(EngineAudioCommand::from_sfx_port_value(
            AudioSfxBank::Effect2,
            self.game_state.system_signals.sound_effect_2(),
        ));
        self.clear_sound_effect_1();
        self.clear_sound_effect_2();
    }

    pub(super) fn nmi_do_updates(&mut self) {
        self.nmi_do_updates_from(None, false);
    }

    fn nmi_do_updates_from(&mut self, oam_dma_source: Option<&[u8]>, defer_bg_vram_upload: bool) {
        if !self.game_state.display.core_updates_are_disabled() {
            self.nmi_core_link_graphics_update();

            let src_addr = self.game_state.display.animated_tile_data_source_usize();
            let dst = self
                .game_state
                .display
                .animated_tile_vram_destination_usize();
            if dst + 0x200 <= self.ppu.vram.len() && src_addr + 0x400 <= self.ram.len() {
                let data = self.animated_tile_dma_source_bytes().to_vec();
                if std::env::var_os("ZELDA3_DEBUG_BOOT_DMA_SOURCE").is_some()
                    && self.rom_startup_timing()
                    && self.game_state.frame.main_module == 0
                    && self.game_state.frame.submodule == 0
                {
                    eprintln!(
                        "boot_dma_source host={} src={src_addr:04x} dst={dst:04x} bytes={:02x?}",
                        self.frame_ctr_dbg,
                        &data[..data.len().min(16)],
                    );
                }
                for i in 0..0x200 {
                    self.ppu.vram[dst + i] = read_word_from_slice(&data, i * 2);
                }
                // Tag the per-frame animated BG tiles (VRAM 0x3c00 overworld water /
                // flowers). These are tagged CHR_KIND_BG_ANIM, but the off-VRAM
                // extractor decodes them from LIVE VRAM (see `extract_modern_frame_from_
                // sources`), NOT from the assets atlas: the assets dump captures these
                // slots' pixels at frame-end while several overworld animations rewrite
                // the same 0xa680 buffer position in-place per phase, so no static
                // `(pack, position)` key OR content hash the dump records matches the
                // pixels the live frame streams (frame 250000 waterfall: atlas cell
                // wrong for BOTH key schemes; only live-VRAM decode is byte-exact).
                // The `(pack, base_off)` tag is kept as metadata / debug provenance.
                const ANIMATED_TILE_BUFFER_BASE: usize = 0xa680;
                if dst == 0x3c00 && src_addr >= ANIMATED_TILE_BUFFER_BASE {
                    let base_off = ((src_addr - ANIMATED_TILE_BUFFER_BASE) / 32) as u16;
                    self.vram_chr_source.record_tiles_from(
                        dst,
                        0x20,
                        crate::chr_source::CHR_KIND_BG_ANIM,
                        self.animated_tile_pack,
                        base_off,
                    );
                } else if dst != 0x3c00 {
                    // DUNGEON animated tiles (dst 0x3b00): re-DMA'd over slots tagged
                    // static kind=1 BG by initialize_tilesets. Content-hash (32-bit) so
                    // the off-VRAM path resolves the live animated cell. Unlike the OW
                    // path above, the dungeon dump binding is stable (verified byte-exact
                    // at frame 435000), so the atlas cell is correct here.
                    for t in 0..0x20usize {
                        let word0 = dst + t * 16;
                        if word0 + 16 <= self.ppu.vram.len() {
                            let hash = crate::chr_source::chr_content_hash32(
                                &self.ppu.vram[word0..word0 + 16],
                            );
                            self.vram_chr_source.record_tile_content_hash(
                                dst / 16 + t,
                                crate::chr_source::CHR_KIND_BG_STREAM,
                                hash,
                            );
                        }
                    }
                }
            }
        }

        let frame = self.game_state.frame;
        let defer_intro_initialization_publication = self.rom_startup_timing()
            && frame.main_module == 0
            && frame.submodule == 1;
        let defer_intro_cgram = defer_intro_initialization_publication
            || (self.rom_startup_timing()
                && frame.main_module == 0
                && frame.submodule == 7
                && !matches!(self.intro_bg_fade_poly_phase, 1 | 3));
        if self.game_state.system_signals.should_update_cgram() && !defer_intro_cgram {
            // C: memcpy(g_zenv.ppu->cgram, main_palette_buffer, 0x200)
            // Read directly from WRAM so that set_aux_color() calls that overflow the
            // aux buffer into main_palette_buffer (when overworld_palette_aux_or_main=0x200)
            // are visible here.  The native palette_buffer.main is bounded to 0x100 entries
            // and silently drops out-of-bounds writes, leaving it stale in that case.
            // Hardware performs this upload in the vblank AFTER the frame the
            // display snapshot describes; latch the pre-upload image so the
            // compose can show what this frame's scanout actually used (the
            // attract palette filter diverges from Snes9x otherwise). The
            // intro module's palette effects (poly flash, logo fades) are
            // IRQ-driven mid-frame writes on hardware that this port folds
            // into the NMI upload — those stay same-frame visible.
            if frame.main_module != 0 && self.cgram_upload_latch.is_none() {
                self.cgram_upload_latch = Some(self.ppu.cgram.to_vec());
            }
            for i in 0..0x100 {
                self.ppu.cgram[i] = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
            }
            self.commit_palette_provenance_cgram();
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

        self.clear_hud_update_flag();
        if !defer_intro_cgram {
            self.clear_cgram_update_flag();
        }
        let mut oam_buf = self.sprite_oam_shadow_buffer().to_vec();
        let frame = self.game_state.frame;
        // Snes9x shows the normal $00:0800 OAM DMA on the first initialized
        // vblank.  The startup CPU work has not yet authored the regular
        // sprite list, but its hardware-visible reset word is still a real
        // DMA source; suppressing this transfer was the reason a renderer-side
        // OAM/VRAM substitute had accumulated here.
        let defer_intro_initialization_oam_dma = false;
        if self.rom_startup_timing() && frame.main_module == 0 && matches!(frame.submodule, 6 | 7) {
            if let Some(boundary_oam) = oam_dma_source {
                // The title sword/sparkle writer runs after the OAM-DMA boundary in
                // the interrupted intro main thread. Other title OBJ entries have
                // already settled for this boundary, so retain only this subsystem's
                // prior shadow region (entries $50..$5b and their packed size bits).
                oam_buf[0x140..0x170].copy_from_slice(&boundary_oam[0x140..0x170]);
                oam_buf[0x214..0x217].copy_from_slice(&boundary_oam[0x214..0x217]);
            }
        }
        let _ = std::mem::take(&mut self.rom_lag_frame_skip_oam_dma);
        if !defer_intro_initialization_oam_dma {
            for i in 0..self.ppu.oam.len() {
                self.ppu.oam[i] = read_word_from_slice(&oam_buf, i * 2);
            }
        }

        if std::env::var_os("ZELDA3_DEBUG_ATTRACT_NMI_UPLOAD").is_some()
            && frame.main_module == 20
            && self.game_state.display.has_bg_vram_load()
        {
            eprintln!(
                "attract_nmi_bg_upload mode={} defer={} sequence={} state={} nmi={}",
                self.game_state.display.bg_vram_load_mode,
                defer_bg_vram_upload,
                self.game_state.ending.attract_scene.sequence(),
                self.game_state.ending.attract_scene.state_word(),
                self.game_state.display.pending_nmi_subroutine,
            );
        }
        if self.game_state.display.has_bg_vram_load() && !defer_bg_vram_upload {
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

        // A scheduling experiment may defer the dispatch while retaining the
        // rest of the NMI's normal OAM/PPU maintenance. This is intentionally
        // distinct from holding the whole NMI: it leaves the pending byte
        // intact exactly as hardware does when main-loop publication misses a
        // vblank boundary.
        let nmi_subroutine_index = if self.parity_runtime_defer_pending_nmi_this_frame() {
            0
        } else {
            self.take_pending_nmi_subroutine()
        };
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
        // Animation-modeled asset renderer: this NMI DMA re-streams room-specific
        // dungeon BG CHR over VRAM 0x2c00-0x3bff every room transition, leaving the
        // room-entry do3->4 `(BG, pack, off)` tag stale. Re-tag the 256 streamed
        // tiles with a 24-bit content hash of the just-written VRAM words so the
        // off-VRAM path resolves the live floor/room cell, not the initial blockset.
        // Indoor-only: the overworld owns these slots via its own keys and is at 0.
        if self.game_state.world.location.indoor_flag() != 0 {
            const BASE: usize = 0x2c00;
            // copy_to_vram_slice len is in BYTES: 0x1000 bytes = 0x800 words = 0x80 tiles
            // (16 words/tile), covering slots 0x2c0..0x340 (VRAM words 0x2c00..0x3400).
            // Do NOT over-tag into 0x3400 (char5and6's region).
            const TILES: usize = 0x80;
            for t in 0..TILES {
                let word0 = BASE + t * 16;
                let hash = crate::chr_source::chr_content_hash32(&self.ppu.vram[word0..word0 + 16]);
                self.vram_chr_source.record_tile_content_hash(
                    BASE / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        } else {
            // OUTDOORS: re-tag GENERIC so the off-VRAM path decodes from live VRAM,
            // clearing any stale BG_STREAM tag left over these slots by a prior
            // indoor area (same stale-tag class as nmi_run_tile_map_update_dma).
            self.tag_stream_generic(0x2c00, 0x80);
        }
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_bg_char5and6(&mut self) {
        let buf = self.background_character_half_buffer().to_vec();
        self.copy_to_vram_slice(0x3400, &buf, 0x1000);
        // Animation-modeled asset renderer (M1/approach A): VRAM 0x3400 is the OW
        // BG2 char base. `initialize_tilesets` statically tagged it once with the
        // area's aux BG pack, but the overworld re-DMAs the half buffer here on
        // every area transition with DIFFERENT decompressed aux gfx, leaving the
        // per-slot CHR source tag stale (non-injective across the route — the same
        // `(BG, 0x4d, off)` key resolves to different pixels in different areas).
        // Re-tag the streamed slots with a per-AUX-THEME-stable, injective key:
        // the half-buffer content at slot N is deterministic in the aux tile theme
        // (LoadTransAuxGFX rebuilds it from `aux_tileset(theme)`), so
        // `(BG, 0x8000 | theme, N)` maps 1:1 to the streamed pixels. The 0x8000 bit
        // keeps it disjoint from real BG packs (< 0x100). Overworld-only (the indoor
        // path does not stream different content over this region).
        if self.game_state.world.location.indoor_flag() == 0 {
            let theme = self.game_state.world.palette_theme.aux_tile_theme_index() as u16;
            // 0x1000 BYTES = 0x800 words = 0x80 tiles. (Was 0x100 = 256 tiles, which
            // SPILLED past 0x3c00 into the OBJ/Link region 0x4000-0x43ff, stamping a
            // BG tag over sprite slots and corrupting the off-VRAM sprite path.)
            self.vram_chr_source.record_tiles(
                0x3400,
                0x80,
                crate::chr_source::CHR_KIND_BG,
                0x8000 | theme,
            );
        } else {
            // DUNGEON: re-stream of room BG CHR over 0x3400; content-hash like the
            // other streaming writers (theme key was non-injective for room-specific
            // tiles). 0x80 tiles = 0x800 words.
            const TILES: usize = 0x80;
            for t in 0..TILES {
                let word0 = 0x3400 + t * 16;
                let hash = crate::chr_source::chr_content_hash32(&self.ppu.vram[word0..word0 + 16]);
                self.vram_chr_source.record_tile_content_hash(
                    0x3400 / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        }
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

    /// Per-frame: content-hash the non-Link OBJ CHR region (VRAM 0x4400-0x5fff)
    /// into the source table. OBJ subset keys are non-injective (subset numbers,
    /// especially pack 0, are reused with different gfx across areas, and the
    /// per-frame incremental upload re-DMAs them), so the static `(SPRITE, pack,
    /// off)` tag is wrong for the live pixels. Hashing the settled VRAM each frame
    /// is injective and order-independent, regardless of which writer (do3->4,
    /// incremental upload, obj-char DMA) last touched the slot. Link (0x4000-0x43ff)
    /// keeps its `CHR_KIND_LINK` pose key. Bookkeeping only — does not touch VRAM.
    pub(super) fn rehash_streamed_obj_sources(&mut self) {
        // Start just past the Link CHR (0x4000-0x423f, tagged CHR_KIND_LINK by the
        // pose DMA) so Link keeps its injective pose key; cover the rest of the OBJ
        // banks (common sprites, per-area subsets, incremental upload region).
        const OBJ_START: usize = 0x4240;
        const OBJ_END: usize = 0x6000;
        self.tag_stream_content_hash(OBJ_START, (OBJ_END - OBJ_START) / 16);
    }

    /// Re-tag `tiles` CHR slots starting at VRAM word `dst_word` with a 24-bit
    /// content hash of the just-written pixels (animation-modeled asset renderer).
    /// Used by streaming NMI DMAs whose room/area-specific content makes the static
    /// `do3->4` tag stale; the content hash is injective and self-healing.
    /// Re-tag `tiles` CHR slots as GENERIC BG (kind 1) so the off-VRAM path decodes
    /// them from live VRAM. Used by outdoor BG streaming to CLEAR a stale BG_STREAM
    /// tag (left by indoor streaming over the same slots). Generic BG ignores
    /// pack/tile_off (decoded from VRAM), so pack 0 is fine. Bookkeeping only.
    pub(super) fn tag_stream_generic(&mut self, dst_word: usize, tiles: usize) {
        self.vram_chr_source
            .record_tiles(dst_word, tiles, crate::chr_source::CHR_KIND_BG, 0);
    }

    pub(super) fn tag_stream_content_hash(&mut self, dst_word: usize, tiles: usize) {
        for t in 0..tiles {
            let word0 = dst_word + t * 16;
            if word0 + 16 <= self.ppu.vram.len() {
                let hash = crate::chr_source::chr_content_hash32(&self.ppu.vram[word0..word0 + 16]);
                self.vram_chr_source.record_tile_content_hash(
                    dst_word / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        }
    }

    pub(super) fn nmi_update_obj_char0(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x4400, &buf, 0x800);
        // OBJ CHR (common sprites / items at 0x4400) is streamed per-frame and
        // outlives its static do3->4 tag; content-hash it so the off-VRAM sprite
        // path resolves the live sprite. NOT indoor-gated — sprites appear in OW too.
        // 0x800 bytes = 0x400 words = 0x40 tiles.
        self.tag_stream_content_hash(0x4400, 0x40);
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
        // 0x1000 BYTES = 0x800 words = 0x80 tiles. OBJ destinations (>=0x4000:
        // 0x5000/0x5800) are content-hashed unconditionally. BG destinations
        // (<0x4000): content-hash INDOORS; OUTDOORS re-tag GENERIC so the off-VRAM
        // path decodes from live VRAM — clearing any stale BG_STREAM tag left from a
        // previous indoor area (the dark-world pyramid streams BG over slots that
        // were content-hash-tagged while indoors; without this they render the prior
        // area). Outdoor generic-BG decodes from VRAM, so it's correct + injective.
        if dst >= 0x4000 || self.game_state.world.location.indoor_flag() != 0 {
            self.tag_stream_content_hash(dst, 0x80);
        } else {
            self.tag_stream_generic(dst, 0x80);
        }
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
        // Animation-modeled asset renderer M1: tag the Link CHR VRAM slots with
        // the active Link DMA graphics index as the logical source. Write-only
        // bookkeeping; does not affect the VRAM bytes written below.
        let link_pack = (self
            .game_state
            .player
            .follower_link
            .link_dma_graphics_index_word()
            >> 1) as u16;

        if let Some(link_graphics) = self.asset_raw(57).map(Vec::from) {
            for (dst, source, len) in [
                (0x4100, LinkDmaSourceSlot::BodyBottom, 0x40),
                (0x4120, LinkDmaSourceSlot::HeadBottom, 0x40),
                (0x4140, LinkDmaSourceSlot::HandRight, 0x20),
                (0x4000, LinkDmaSourceSlot::BodyTop, 0x40),
                (0x4020, LinkDmaSourceSlot::HeadTop, 0x40),
                (0x4040, LinkDmaSourceSlot::HandLeft, 0x20),
            ] {
                // Key the tile by its source identity (offset within the static
                // Link sprite asset, 32 bytes / 4bpp tile) so distinct pose pieces
                // that share `(pack, relative-tile)` no longer collide. Asset
                // offsets use the `0x8000`-relative source address; buffer flag 0.
                let src_addr = self.live_link_dma_source(source);
                let base_off = (src_addr.saturating_sub(0x8000) >> 5) as u16;
                self.copy_asset_bytes_to_vram(dst, &link_graphics, source, len);
                self.vram_chr_source.record_tiles_from(
                    dst,
                    (len / 2).div_ceil(16),
                    crate::chr_source::CHR_KIND_LINK,
                    link_pack,
                    base_off,
                );
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
            // WRAM-sourced Link tiles: key by the WRAM source tile offset, tagged
            // with the buffer flag so they never collide with asset-sourced tiles
            // (the raw address spaces overlap). See `CHR_LINK_SRC_RAM_FLAG`.
            let src_addr = self.live_link_dma_source(source);
            let base_off = crate::chr_source::CHR_LINK_SRC_RAM_FLAG | ((src_addr >> 5) as u16);
            self.copy_ram_bytes_to_vram(dst, source, len);
            self.vram_chr_source.record_tiles_from(
                dst,
                (len / 2).div_ceil(16),
                crate::chr_source::CHR_KIND_LINK,
                link_pack,
                base_off,
            );
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

    /// Snes9x/ROM truth: NMI DMA follows the live WRAM address words, not the
    /// engine's cached display projection. Several of those words are reused by
    /// attract/text code, so reading the projection can upload stale graphics.
    fn live_link_dma_source(&self, slot: LinkDmaSourceSlot) -> usize {
        let address = slot.ram_address();
        self.ram
            .get(address..address + 2)
            .map(|bytes| usize::from(u16::from_le_bytes([bytes[0], bytes[1]])))
            .unwrap_or(0)
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
        let source_addr = self.live_link_dma_source(source_slot);
        if self.is_first_boot_nmi_dma_source(source_addr, len) {
            self.copy_bytes_to_vram(dst_word, &FIRST_BOOT_NMI_DMA_SOURCE, 0, len);
            return;
        }
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
        let source_addr = self.live_link_dma_source(source_slot);
        // At the first boot NMI, ROM $008aa2 DMA uploads this exact
        // $7e:0000 source through the startup Link channels. Snes9x records
        // the bytes as
        // `00 80 00 ... 00 00 01 80 00 01 ...`. Some of those zero-page
        // offsets are represented by dedicated native control fields here
        // (not direct WRAM), so copying them into `ram` would incorrectly
        // latch/suppress the NMI. Keep that representation, while making the
        // hardware-visible DMA source exact at the ROM-equivalent sites.
        if self.is_first_boot_nmi_dma_source(source_addr, len) {
            self.copy_bytes_to_vram(dst_word, &FIRST_BOOT_NMI_DMA_SOURCE, 0, len);
            return;
        }
        self.copy_ram_bytes_to_vram_absolute(dst_word, source_addr, len);
    }

    fn is_first_boot_nmi_dma_source(&self, source_addr: usize, len: usize) -> bool {
        self.rom_startup_timing()
            && self.game_state.frame.main_module == 0
            && self.game_state.frame.submodule == 0
            && source_addr == 0
            && len <= FIRST_BOOT_NMI_DMA_SOURCE.len()
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
        if dst < 0x4000 {
            if self.game_state.world.location.indoor_flag() != 0 {
                const TILES: usize = 0x20; // 0x200 words / 16
                for t in 0..TILES {
                    let word0 = dst + t * 16;
                    let hash =
                        crate::chr_source::chr_content_hash32(&self.ppu.vram[word0..word0 + 16]);
                    self.vram_chr_source.record_tile_content_hash(
                        dst / 16 + t,
                        crate::chr_source::CHR_KIND_BG_STREAM,
                        hash,
                    );
                }
            } else {
                // OUTDOORS: re-tag GENERIC (decode from live VRAM) so a stale
                // BG_STREAM tag from a prior indoor area is cleared. 0x20 tiles.
                self.tag_stream_generic(dst, 0x20);
            }
        }
    }

    pub(super) fn nmi_upload_bg3_text(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        for i in 0..0x3f0 {
            self.ppu.vram[0x7c00 + i] = read_word_from_slice(&buf, i * 2);
        }
        // Semantic glyph placements are a second representation of the same
        // VWF pixels. Publish them at the DMA boundary too; exposing the
        // CPU-authored list earlier lets the modern renderer paint text that
        // the SNES PPU has not received yet.
        self.publish_bg3_vwf_glyph_runs();
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
#[path = "nmi_tests.rs"]
mod tests;
