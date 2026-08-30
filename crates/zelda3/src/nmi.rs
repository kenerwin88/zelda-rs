// Methods ported from zelda3/src/nmi.c and included inside ZeldaState.

use super::*;
use crate::game_output::{AudioSfxBank, EngineAudioCommand};

// Snes9x's DMA trace at ROM $008aa2/$008adb: first boot NMI source $7e:0000.
const FIRST_BOOT_NMI_DMA_SOURCE: [u8; 0x40] = [
    0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x80, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_0: usize = 0x0000;
const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_1: usize = 0x0001;
const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_2: usize = 0x0002;

// DMA channel 0 is deliberately reused throughout the ROM's NMI handler.
// Most transfer sites program both of these registers, but the HUD upload at
// $00:8b87 only programs the source, length, and VRAM address. It therefore
// inherits the target selected by the preceding transfer, even across NMIs.
const DMA_MODE_ONE_REGISTER: u8 = 0;
const DMA_MODE_TWO_REGISTERS: u8 = 1;
const PPU_BBUS_OAM_DATA: u8 = 0x04;
const PPU_BBUS_VRAM_DATA_LOW: u8 = 0x18;
const PPU_BBUS_CGRAM_DATA: u8 = 0x22;

const DMA_BBUS_OFFSETS: [[u8; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NmiVramCopyDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NmiVramCopyPacket<'a> {
    pub(super) destination: usize,
    pub(super) direction: NmiVramCopyDirection,
    pub(super) data: &'a [u8],
}

/// Decode the packet stream consumed by `NMI_CopyPackets`.
///
/// Keeping packet interpretation in one place is important for display
/// publication: the active scanout may retain the pre-NMI words at these exact
/// destinations while live emulation advances through the DMA.
pub(super) fn nmi_vram_copy_packets(data: &[u8]) -> Vec<NmiVramCopyPacket<'_>> {
    let mut packets = Vec::new();
    let mut pos = 0usize;
    while pos + 4 <= data.len() && read_word_from_slice(data, pos) != 0xffff {
        let destination = read_word_from_slice(data, pos) as usize;
        let vmain = data[pos + 2];
        let len = data[pos + 3] as usize;
        pos += 4;
        if pos + len > data.len() {
            break;
        }
        let direction = match vmain {
            0x80 => NmiVramCopyDirection::Horizontal,
            0x81 => NmiVramCopyDirection::Vertical,
            _ => panic!("invalid NMI packet vmain {vmain:#04x}"),
        };
        packets.push(NmiVramCopyPacket {
            destination,
            direction,
            data: &data[pos..pos + len],
        });
        pos += len;
    }
    packets
}

/// Whether `frame` matches a comma-separated frame selection (`N`, `N-M`)
/// held in `env_name`. Absent or empty env → no match. Shared by the
/// per-frame display/hardware debug probes so every probe accepts ranges.
pub(crate) fn debug_frame_selection_env_matches(env_name: &str, frame: u32) -> bool {
    let Some(selection) = std::env::var_os(env_name) else {
        return false;
    };
    selection.to_string_lossy().split(',').any(|part| {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            return start.trim().parse::<u32>().ok().is_some_and(|start| {
                end.trim()
                    .parse::<u32>()
                    .ok()
                    .is_some_and(|end| start <= frame && frame <= end)
            });
        }
        part.parse::<u32>().ok() == Some(frame)
    })
}

fn debug_hardware_frame_matches(frame: u32) -> bool {
    if std::env::var_os("ZELDA3_DEBUG_HARDWARE_FRAMES").is_none() {
        return true;
    }
    debug_frame_selection_env_matches("ZELDA3_DEBUG_HARDWARE_FRAMES", frame)
}

impl ZeldaState {
    /// Record the two DMA-channel registers that select how channel 0 writes
    /// the PPU B-bus. The semantic NMI implementation performs transfers in
    /// bulk, but these registers remain hardware state between transfers.
    pub(super) fn program_dma0_ppu_target(&mut self, mode: u8, b_adr: u8) {
        let channel = &mut self.dma.channel[0];
        channel.mode = mode & 7;
        channel.b_adr = b_adr;
        channel.fixed = false;
        channel.decrement = false;
        channel.indirect = false;
        channel.from_b = false;
        channel.unused_bit = false;
    }

    /// Execute the ROM's HUD DMA using channel 0 exactly as it is currently
    /// configured. Unlike the normal VRAM/OAM/CGRAM transfer sites, the ROM
    /// does not write DMAP0 or BBAD0 here, so the previous transfer owns the
    /// destination and transfer pattern.
    pub(super) fn complete_hud_dma_from_persistent_channel0(
        &mut self,
        source: &[u8],
        dst_word: usize,
    ) {
        let len = HUD_TILEMAP_NMI_WORDS * 2;
        assert!(
            source.len() >= len,
            "HUD DMA source is shorter than its payload"
        );

        // These PPU and DMA source registers are programmed by the HUD path
        // regardless of the inherited B-bus destination.
        self.ppu.write(0x15, 0x80);
        self.ppu.write(0x16, dst_word as u8);
        self.ppu.write(0x17, (dst_word >> 8) as u8);
        {
            let channel = &mut self.dma.channel[0];
            channel.a_bank = 0x7e;
            channel.a_adr = HUD_TILE_INDICES_BUFFER as u16;
            channel.size = len as u16;
            channel.dma_active = true;
            channel.off_index = 0;
        }

        let channel = self.dma.channel[0];
        let mode = usize::from(channel.mode & 7);
        let mut touched_oam = false;
        let mut touched_cgram = false;
        for (index, &value) in source[..len].iter().enumerate() {
            let b_adr = channel
                .b_adr
                .wrapping_add(DMA_BBUS_OFFSETS[mode][index & 3]);
            match b_adr {
                0x18 | 0x19 => {
                    self.mark_effective_dma_vram_word(usize::from(self.ppu.vram_pointer & 0x7fff));
                }
                PPU_BBUS_OAM_DATA => touched_oam = true,
                PPU_BBUS_CGRAM_DATA => touched_cgram = true,
                _ => {}
            }
            self.ppu.write(b_adr, value);
        }

        {
            let channel = &mut self.dma.channel[0];
            if !channel.fixed {
                channel.a_adr = if channel.decrement {
                    channel.a_adr.wrapping_sub(len as u16)
                } else {
                    channel.a_adr.wrapping_add(len as u16)
                };
            }
            channel.size = 0;
            channel.dma_active = false;
            channel.off_index = 0;
        }
        self.dma.dma_busy = false;

        if touched_oam {
            self.resident_oam_dma = Some(self.ppu.oam.clone());
            self.record_completed_oam_dma_for_display_boundary();
        }
        if touched_cgram {
            self.record_completed_cgram_dma_for_display_boundary();
        }
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
        self.interrupt_nmi_with_animated_bg_operands(
            input,
            oam_dma_source,
            defer_bg_vram_upload,
            None,
        );
    }

    /// Run an NMI at an explicit leading scanout boundary and record its
    /// presentation effects. The snapshot already owns the CPU/register
    /// generation, while vblank DMA completes before its visible scanlines.
    #[track_caller]
    pub(super) fn interrupt_nmi_for_active_scanout(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        defer_bg_vram_upload: bool,
    ) -> Option<super::DialogueTextDmaPublicationToken> {
        self.begin_effective_presented_dma();
        self.interrupt_nmi_with_animated_bg_operands(
            input,
            oam_dma_source,
            defer_bg_vram_upload,
            Some(GraphicsDmaGeneration::HostBoundaryBeforeMain),
        );
        let dialogue_text_dma = self.record_effective_presented_dma_for_active_scanout();
        let dialogue_text_dma =
            self.consume_staged_dialogue_text_dma_publication(dialogue_text_dma);
        // OAM-law clause: a leading NMI's transfer completes before this
        // frame's visible scanlines, so it is visible immediately.
        if let Some(pending) = self.oam_law_pending.take() {
            self.oam_law_visible = Some(pending);
        }
        dialogue_text_dma
    }

    /// Run an active-scanout NMI for an owner which cannot immediately stage
    /// dialogue text. Reject a dialogue lifecycle conflict before the handler
    /// mutates hardware or consumes its one-shot DMA evidence.
    pub(super) fn interrupt_nmi_for_active_scanout_without_dialogue_owner(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        defer_bg_vram_upload: bool,
    ) {
        assert!(
            matches!(
                self.dialogue_scroll_phase(),
                super::DialogueScrollPhase::Idle
                    | super::DialogueScrollPhase::CompletedScroll
                    | super::DialogueScrollPhase::RetiredTextDma
            ),
            "an active-scanout NMI without a dialogue owner overlaps an in-flight text publication",
        );
        assert!(
            self.interrupt_nmi_for_active_scanout(input, oam_dma_source, defer_bg_vram_upload,)
                .is_none(),
            "a non-dialogue NMI produced unclaimed text-DMA evidence",
        );
    }

    pub(super) fn interrupt_nmi_with_animated_bg_operands(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        defer_bg_vram_upload: bool,
        animated_bg_operands: Option<GraphicsDmaGeneration>,
    ) {
        self.validate_next_original_timing_nmi_update_gate();
        // Ordinary NMI writes cannot re-enter the active field, but an
        // AdvanceStaged publication may already have materialized the next
        // field. Record exact destinations so that staged owner can be refined.
        let records_trailing_nmi_receipts = self.active_effective_dma_writes.is_none();
        if records_trailing_nmi_receipts {
            self.begin_trailing_nmi_receipts();
        }
        // CPU continuation timing must observe the same pre-NMI RAM, latch,
        // DMA, and raster generation as the real handler. Capture centrally
        // so every hardware-NMI entry path has identical provenance.
        self.debug_obj_pipe("nmi_entry", &self.ppu.vram[0x4000..0x4400]);
        self.capture_cpu_schedules_before_nmi();
        let trace_nmi = std::env::var_os("ZELDA3_DEBUG_NMI_LATCH").is_some()
            && debug_hardware_frame_matches(self.frame_ctr_dbg);
        self.ppu.forced_blank_from_scanline = None;
        self.ppu.retain_active_display_history = false;
        let forced_blank_at_entry = self.ppu.forced_blank;
        let mut prior_active_display_blanking = NmiActiveDisplayBlanking::default();
        if trace_nmi {
            let frame = self.game_state.frame;
            eprintln!(
                "nmi_before host={} main={:02x} sub={:02x} subsub={:02x} frame_counter={:02x} scheduler={:?} latch={} pending={} target={:04x} disable={:02x} bgload={} forced_blank={} blank_lines_pending={} blank_from_pending={:?} blank_from_candidate={:?} link_tile_src={:04x} ram0000={:02x}{:02x}{:02x} vram40b0={:04x}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                frame.subsubmodule,
                frame.frame_counter,
                self.game_execution_scheduler,
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
                self.game_state.display.bg_vram_load_mode,
                self.ppu.forced_blank,
                self.nmi_forced_blank_scanlines_pending,
                self.legacy_nmi_forced_blank_from_scanline_pending,
                self.nmi_active_display_blanking_candidate.suffix_start_scanline,
                self.live_link_dma_source(LinkDmaSourceSlot::AnimatedTileUpper),
                self.ram[FIRST_BOOT_NMI_DMA_SOURCE_BYTE_0],
                self.ram[FIRST_BOOT_NMI_DMA_SOURCE_BYTE_1],
                self.ram[FIRST_BOOT_NMI_DMA_SOURCE_BYTE_2],
                self.ppu.vram.get(0x40b0).copied().unwrap_or(0),
            );
        }
        let joypad_already_sampled = std::mem::take(&mut self.joypad_sampled_before_main);
        let audio_already_processed = std::mem::take(&mut self.audio_nmi_processed_before_main);
        if !audio_already_processed {
            self.interrupt_nmi_audio_parts();
        }

        if !self.game_state.display.nmi_update_is_latched() {
            let bg_vram_load_mode = self.game_state.display.bg_vram_load_mode;
            let stripe_work = match bg_vram_load_mode {
                1 => stripe_upload_work(self.vram_upload_buffer_remaining()),
                5..=9 => self
                    .assets
                    .as_ref()
                    .and_then(|assets| assets.asset(95 + usize::from(bg_vram_load_mode)))
                    .map(stripe_upload_work)
                    .unwrap_or_default(),
                _ => StripeUploadWork::default(),
            };
            if trace_nmi && bg_vram_load_mode != 0 {
                eprintln!(
                    "nmi_workload host={} bgload={} stripe_packets={} stripe_bytes={} stripe_fixed={} stripe_vertical={}",
                    self.frame_ctr_dbg,
                    bg_vram_load_mode,
                    stripe_work.packets,
                    stripe_work.transfer_bytes,
                    stripe_work.fixed_source_packets,
                    stripe_work.vertical_packets,
                );
            }
            let blanking = nmi_active_display_blanking_for_pending_work(
                self.game_state.display.core_updates_are_disabled(),
                self.ppu.forced_blank,
                bg_vram_load_mode,
                stripe_work,
            );
            let hud_upload_is_pending = self.game_state.system_signals.should_update_hud();
            let full_tilemap_upload_is_pending = self.game_state.display.pending_nmi_subroutine
                == 1
                && !self.game_state.display.core_updates_are_disabled();
            let current_scanout_prefix = hud_and_full_tilemap_nmi_forced_blank_prefix(
                hud_upload_is_pending,
                full_tilemap_upload_is_pending,
            );
            if let Some(display) = self.display_snapshot.as_mut() {
                display.ppu.forced_blank_scanlines = display
                    .ppu
                    .forced_blank_scanlines
                    .max(current_scanout_prefix);
            }
            prior_active_display_blanking =
                std::mem::replace(&mut self.nmi_active_display_blanking_candidate, blanking);
            self.nmi_forced_blank_scanlines_pending = self
                .nmi_forced_blank_scanlines_pending
                .max(blanking.prefix_scanlines);
            self.latch_nmi_update();
            self.nmi_do_updates_from(oam_dma_source, defer_bg_vram_upload, animated_bg_operands);
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
        // The atomic C frontend calls `WritePpuRegisters` unconditionally, but
        // it cannot represent an NMI taken while the original 65816 dialogue
        // or attract call stack is still executing. The original-ROM/Snes9x
        // frame-5377 receipt keeps the preceding coupled register generation
        // at that boundary. Hold the whole set; never publish color math,
        // scroll, mode, or brightness independently.
        let main_module = self.game_state.frame.main_module;
        let dialogue_scroll_holds_registers = self.dialogue_scroll_holds_nmi_registers();
        let thread_holds_registers = (main_module == 0x14
            && self.game_state.display.nmi_thread_active)
            || (main_module == 0x0e
                && (self.dialogue_fast_forward_hold_active || dialogue_scroll_holds_registers));
        if trace_nmi {
            eprintln!(
                "nmi_register_publication host={} main={main_module:02x} held={thread_holds_registers} thread={} dialogue_fast={} dialogue_scroll={} ppu_bg1v={:04x} mirror_bg1v={:04x} ppu_cgwsel={:02x} mirror_cgwsel={:02x}",
                self.frame_ctr_dbg,
                self.game_state.display.nmi_thread_active,
                self.dialogue_fast_forward_hold_active,
                dialogue_scroll_holds_registers,
                self.ppu.bg_layer[0].v_scroll,
                self.game_state.display.ppu_scroll_copy.bg1_v_copy(),
                (self.ppu.clip_mode << 6)
                    | (self.ppu.prevent_math_mode << 4)
                    | (u8::from(self.ppu.add_subscreen) << 1),
                self.game_state
                    .display
                    .palette_filter
                    .color_window_selection(),
            );
        }
        if !thread_holds_registers {
            self.write_ppu_registers();
        }
        if !forced_blank_at_entry && self.ppu.forced_blank {
            if let Some(start) = prior_active_display_blanking.suffix_start_scanline {
                self.ppu.forced_blank_from_scanline = Some(start);
                self.ppu.retain_active_display_history = true;
            }
        }
        // After all CHR DMAs have settled this frame, refresh the OBJ CHR logical
        // sources by content hash so the off-VRAM sprite path resolves live cells.
        self.rehash_streamed_obj_sources();
        if trace_nmi {
            let frame = self.game_state.frame;
            eprintln!(
                "nmi_after host={} main={:02x} sub={:02x} latch={} pending={} target={:04x} disable={:02x} bgload={} forced_blank={} blank_lines_pending={} blank_from_live={:?} blank_from_pending={:?} blank_from_candidate={:?} link_tile_src={:04x} vram40b0={:04x}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
                self.game_state.display.bg_vram_load_mode,
                self.ppu.forced_blank,
                self.nmi_forced_blank_scanlines_pending,
                self.ppu.forced_blank_from_scanline,
                self.legacy_nmi_forced_blank_from_scanline_pending,
                self.nmi_active_display_blanking_candidate.suffix_start_scanline,
                self.live_link_dma_source(LinkDmaSourceSlot::AnimatedTileUpper),
                self.ppu.vram.get(0x40b0).copied().unwrap_or(0),
            );
        }
        self.debug_obj_pipe("nmi_exit", &self.ppu.vram[0x4000..0x4400]);
        if records_trailing_nmi_receipts {
            self.record_trailing_nmi_receipts();
        }
        self.close_display_boundary_dma_receipts();
    }

    pub(super) fn interrupt_nmi_audio_parts(&mut self) {
        if std::env::var("ZELDA3_DEBUG_AUDIO_NMI_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(self.frame_ctr_dbg)
        {
            eprintln!(
                "audio_nmi_sample host={} phase={:02x}/{:02x}/{:02x} ambient={:02x} effect1={:02x} effect2={:02x}",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
                self.game_state.system_signals.ambient_sound_effect(),
                self.game_state.system_signals.sound_effect_1(),
                self.game_state.system_signals.sound_effect_2(),
            );
        }
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
        self.consume_nmi_effect_latches();
    }

    fn consume_nmi_effect_latches(&mut self) {
        self.clear_sound_effect_1();
        self.clear_sound_effect_2();
    }

    pub(super) fn nmi_do_updates(&mut self) {
        self.nmi_do_updates_from(None, false, None);
    }

    pub(super) fn nmi_core_animated_bg_update(&mut self, graphics_dma_plan: GraphicsDmaPlan) {
        let animated_bg_operands = animated_bg_operands_for_dungeon_landing(
            self.game_state.frame,
            self.game_state.world.location.dungeon_room_index(),
            self.game_state.player.follower_link.last_direction(),
            graphics_dma_plan.animated_bg_operands,
        );
        let host_main_prefix_did_not_advance =
            self.pre_main_graphics_dma.as_ref().is_some_and(|graphics| {
                graphics.entry_frame.frame_counter == self.game_state.frame.frame_counter
            });
        let pre_main_dma = if matches!(
            animated_bg_operands,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ) {
            self.pre_main_graphics_dma
                .as_mut()
                .and_then(|graphics| graphics.animated_tile.take())
        } else {
            None
        };
        let (mut src_addr, dst, mut data) = pre_main_dma.map_or_else(
            || {
                (
                    self.game_state.display.animated_tile_data_source_usize(),
                    self.game_state
                        .display
                        .animated_tile_vram_destination_usize(),
                    self.animated_tile_dma_source_bytes().to_vec(),
                )
            },
            |dma| (dma.source_address, dma.destination_address, dma.data),
        );
        if matches!(animated_bg_operands, GraphicsDmaGeneration::LiveAfterMain) {
            if let Some(projected_source) = rom_spiral_stairs_suspended_animated_bg_source_address(
                self.game_state.frame,
                host_main_prefix_did_not_advance,
                self.game_state.display.bg_tile_animation_countdown,
                src_addr,
            ) {
                // The translated spiral-filter continuation exposes the
                // pre-decrement software countdown here. Snes9x has already
                // completed the preceding caller's animation-source advance,
                // so project that hardware-facing DMA operand without
                // mutating the still-suspended CPU state.
                if let Some(projected_data) =
                    self.ram.get(projected_source..projected_source + 0x400)
                {
                    src_addr = projected_source;
                    data = projected_data.to_vec();
                }
            }
        }
        if std::env::var_os("ZELDA3_DEBUG_ANIMATED_BG_DMA").is_some() {
            let live_source = self
                .ram
                .get(src_addr..src_addr.saturating_add(0x400))
                .unwrap_or_default();
            let first_live_mismatch = data
                .iter()
                .zip(live_source)
                .position(|(captured, live)| captured != live);
            eprintln!(
                "animated_bg_dma host={} source={src_addr:04x} destination={dst:04x} captured={} first_live_mismatch={first_live_mismatch:?} source_prefix={:02x?} live_prefix={:02x?} vram_prefix_before={:04x?}",
                self.frame_ctr_dbg,
                matches!(
                    animated_bg_operands,
                    GraphicsDmaGeneration::HostBoundaryBeforeMain
                ),
                &data[..data.len().min(8)],
                &live_source[..live_source.len().min(8)],
                &self.ppu.vram[dst.min(self.ppu.vram.len())..(dst + 4).min(self.ppu.vram.len())],
            );
        }
        if dst + 0x200 > self.ppu.vram.len() || data.len() < 0x400 {
            return;
        }
        self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
        self.mark_effective_dma_vram_range(dst..dst + 0x200);
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
        if std::env::var_os("ZELDA3_DEBUG_ANIMATED_BG_DMA").is_some() {
            eprintln!(
                "animated_bg_dma_after host={} destination={dst:04x} vram_prefix={:04x?}",
                self.frame_ctr_dbg,
                &self.ppu.vram[dst..dst + 4],
            );
        }
        // Tag the per-frame animated BG tiles (VRAM 0x3c00 overworld water /
        // flowers). These are tagged CHR_KIND_BG_ANIM, but the off-VRAM
        // extractor decodes them from live VRAM, not from the static atlas.
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
            // Dungeon animated tiles overwrite statically tagged BG slots.
            // Content hashes retain the exact live streamed generation.
            for t in 0..0x20usize {
                let word0 = dst + t * 16;
                if word0 + 16 <= self.ppu.vram.len() {
                    let hash =
                        crate::chr_source::chr_content_hash32(&self.ppu.vram[word0..word0 + 16]);
                    self.vram_chr_source.record_tile_content_hash(
                        dst / 16 + t,
                        crate::chr_source::CHR_KIND_BG_STREAM,
                        hash,
                    );
                }
            }
        }
    }

    fn nmi_do_updates_from(
        &mut self,
        oam_dma_source: Option<&[u8]>,
        defer_bg_vram_upload: bool,
        animated_bg_operands: Option<GraphicsDmaGeneration>,
    ) {
        if !self.game_state.display.core_updates_are_disabled() {
            if let Some(uses_host_operands) = self
                .next_core_nmi_active_scanout_uses_host_animated_bg_operands
                .take()
            {
                let (destination, data) = if uses_host_operands {
                    let dma = self
                        .pre_main_graphics_dma
                        .as_ref()
                        .and_then(|graphics| graphics.animated_tile.as_ref())
                        .expect(
                            "instruction-timed animated-BG receipt requires main-entry operands",
                        );
                    (dma.destination_address, dma.data.clone())
                } else {
                    (
                        self.game_state
                            .display
                            .animated_tile_vram_destination_usize(),
                        self.animated_tile_dma_source_bytes().to_vec(),
                    )
                };
                // This is the DMA completed by the hardware NMI which the
                // deferred CPU caller resumed from, not the live PPU write of
                // the atomic NMI below. Keep those consecutive generations
                // separate so the retiring scanout receives the observed
                // payload while the following scanout inherits live PPU state.
                self.record_observed_animated_bg_dma_for_active_scanout(destination, &data);
            }
            let mut graphics_dma_plan = rom_graphics_dma_plan(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
            if let Some(generation) = animated_bg_operands {
                graphics_dma_plan.animated_bg_operands = generation;
            }
            // Link's source words are sampled by the NMI after the current
            // main-thread slice. If that slice switched modules, its exit
            // phase owns the operands; ordinary leading-NMI phases still
            // select the captured host-boundary sources through this plan.
            let entry_frame = self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.entry_frame)
                .unwrap_or(self.game_state.frame);
            let link_obj_operands_generation = self.following_nmi_link_obj_dma_generation();
            let host_boundary_link_operands = self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.link_operands);
            let captured_link_operands = matches!(
                link_obj_operands_generation,
                GraphicsDmaGeneration::HostBoundaryBeforeMain
            )
            .then_some(host_boundary_link_operands)
            .flatten();
            let completed_link_obj_sources = captured_link_operands
                .map(|operands| operands.sources)
                .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
            let trace_link_dma = std::env::var_os("ZELDA3_DEBUG_LINK_DMA").is_some()
                && debug_hardware_frame_matches(self.frame_ctr_dbg);
            if trace_link_dma {
                let live_operands = PreMainLinkDmaOperands::capture(&self.ram);
                let captured_head_top = host_boundary_link_operands
                    .map(|operands| operands.sources.source(LinkDmaSourceSlot::HeadTop));
                let live_head_top = self.live_link_dma_source(LinkDmaSourceSlot::HeadTop);
                // Entry and exit module phases are both printed: the plan keys on
                // the exit frame while divergence reports name the entry frame,
                // and the subsubmodule is the field the operand rules turn on.
                let captured_body_upper = host_boundary_link_operands
                    .map(|operands| operands.sources.source(LinkDmaSourceSlot::BodyPointerUpper));
                let live_body_upper =
                    self.live_link_dma_source(LinkDmaSourceSlot::BodyPointerUpper);
                let early_source_pairs = EARLY_LINK_OBJ_DMA_TRANSFERS.map(|(_, slot, _)| {
                    (
                        slot,
                        host_boundary_link_operands.map(|operands| operands.sources.source(slot)),
                        live_operands.sources.source(slot),
                    )
                });
                let first_expanded_mismatch = host_boundary_link_operands.and_then(|operands| {
                    operands
                        .expanded_high_planes
                        .iter()
                        .zip(live_operands.expanded_high_planes)
                        .position(|(captured, live)| *captured != live)
                });
                eprintln!(
                    "link_dma host={} entry={:02x}/{:02x}/{:02x} exit={:02x}/{:02x}/{:02x} scheduler={:?} operands={link_obj_operands_generation:?} captured={} early_sources={early_source_pairs:?} captured_body_upper={captured_body_upper:?} live_body_upper={live_body_upper:04x} captured_head_top={captured_head_top:?} live_head_top={live_head_top:04x} captured_pack={:?} live_pack={} first_expanded_mismatch={first_expanded_mismatch:?} captured_high={:02x?} live_high={:02x?} vram_head_top_before={:04x} vram_high_before={:04x}",
                    self.frame_ctr_dbg,
                    entry_frame.main_module,
                    entry_frame.submodule,
                    entry_frame.subsubmodule,
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                    self.game_state.frame.subsubmodule,
                    self.game_execution_scheduler,
                    captured_link_operands.is_some(),
                    host_boundary_link_operands.map(|operands| operands.link_pack),
                    live_operands.link_pack,
                    host_boundary_link_operands.map(|operands| {
                        <[u8; 4]>::try_from(&operands.expanded_high_planes[..4])
                            .expect("four-byte Link staging prefix")
                    }),
                    &self.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START
                        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + 4],
                    self.ppu.vram[0x4020],
                    self.ppu.vram[0x4240],
                );
            }
            self.nmi_core_link_graphics_update(captured_link_operands);
            self.record_completed_link_obj_dma_for_display_boundary(
                completed_link_obj_sources,
                link_obj_operands_generation,
            );
            self.link_obj_dma_completed_this_frame = true;
            if trace_link_dma {
                eprintln!(
                    "link_dma host={} vram_head_top_after={:04x} vram_high_after={:04x}",
                    self.frame_ctr_dbg, self.ppu.vram[0x4020], self.ppu.vram[0x4240],
                );
            }

            self.nmi_core_animated_bg_update(graphics_dma_plan);
        }

        let frame = self.game_state.frame;
        let defer_intro_initialization_publication =
            self.rom_startup_timing() && frame.main_module == 0 && frame.submodule == 1;
        let defer_intro_cgram = defer_intro_initialization_publication
            || (self.rom_startup_timing()
                && frame.main_module == 0
                && frame.submodule == 7
                && !matches!(self.intro_bg_fade_poly_phase, 1 | 3));
        let debug_display_vram = std::env::var("ZELDA3_DEBUG_DISPLAY_VRAM_FRAME")
            .ok()
            .and_then(|frame| frame.parse::<u32>().ok())
            .is_some_and(|frame| frame == self.frame_ctr_dbg);
        if debug_display_vram {
            let dst = self
                .game_state
                .display
                .message_dma_destination_address_usize();
            eprintln!(
                "nmi_hud_candidate host={} enabled={} destination={dst:04x} dma0={:02x}/{:02x} floor_timer_low={:02x} floor_words={:04x}/{:04x}/{:04x}/{:04x} vram_60c3={:04x} native_buffer_131={:04x} ram_buffer_131={:04x}",
                self.frame_ctr_dbg,
                self.game_state.system_signals.should_update_hud(),
                self.dma.channel[0].mode,
                self.dma.channel[0].b_adr,
                self.game_state
                    .display
                    .hud_tilemap
                    .floor_changed_timer_low(),
                self.game_state.display.hud_tilemap.tile_word(0xf2 / 2),
                self.game_state.display.hud_tilemap.tile_word(0xf4 / 2),
                self.game_state.display.hud_tilemap.tile_word(0x132 / 2),
                self.game_state.display.hud_tilemap.tile_word(0x134 / 2),
                self.ppu.vram[0x60c3],
                self.game_state.display.hud_tilemap.tile_word(131),
                read_word_from_slice(self.message_dma_tile_indices(), 131 * 2),
            );
        }
        if self.game_state.system_signals.should_update_hud() {
            let dst = self
                .game_state
                .display
                .message_dma_destination_address_usize();
            let hud_buf = self.message_dma_tile_indices().to_vec();
            self.complete_hud_dma_from_persistent_channel0(&hud_buf, dst);
        }
        if debug_display_vram {
            eprintln!(
                "nmi_hud_selected host={} vram_60c3={:04x}",
                self.frame_ctr_dbg, self.ppu.vram[0x60c3],
            );
        }

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
            self.program_dma0_ppu_target(DMA_MODE_ONE_REGISTER, PPU_BBUS_CGRAM_DATA);
            for i in 0..0x100 {
                self.ppu.cgram[i] = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
            }
            self.commit_palette_provenance_cgram();
            self.record_completed_cgram_dma_for_display_boundary();
        }

        self.clear_hud_update_flag();
        if !defer_intro_cgram {
            self.clear_cgram_update_flag();
        }
        let frame = self.game_state.frame;
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or(frame);
        // OAM DMA consumes the operand captured at the hardware boundary. The
        // caller passes `None` only when an explicit scheduler event places
        // this NMI after CPU work which authored a new shadow. Inferring the
        // operand from the current module made ordinary main-thread movement
        // leak into the preceding scanout and required transition-specific
        // publication rules to hide the resulting one-frame lead.
        let oam_operands_generation = if oam_dma_source.is_some() {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else {
            GraphicsDmaGeneration::LiveAfterMain
        };
        if std::env::var_os("ZELDA3_DEBUG_OAM_DMA").is_some()
            && debug_hardware_frame_matches(self.frame_ctr_dbg)
        {
            let captured = oam_dma_source.unwrap_or_default();
            let live = self.sprite_oam_shadow_buffer();
            eprintln!(
                "oam_dma host={} entry={:02x}/{:02x}/{:02x} exit={:02x}/{:02x}/{:02x} generation={oam_operands_generation:?} captured={:02x?} live={:02x?}",
                self.frame_ctr_dbg,
                entry_frame.main_module,
                entry_frame.submodule,
                entry_frame.frame_counter,
                frame.main_module,
                frame.submodule,
                frame.frame_counter,
                [captured.get(369).copied(), captured.get(373).copied()],
                [live.get(369).copied(), live.get(373).copied()],
            );
        }
        let mut oam_buf = oam_dma_source
            .map(Vec::from)
            .unwrap_or_else(|| self.sprite_oam_shadow_buffer().to_vec());
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
        if !defer_intro_initialization_oam_dma {
            self.complete_oam_dma_from_source(&oam_buf);
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

    /// Complete one hardware OAM DMA from its already-selected WRAM operand.
    ///
    /// Both an ordinary NMI and an effective leading-NMI continuation use this
    /// primitive so resident PPU state and display-boundary receipts cannot
    /// disagree about which byte generation the transfer installed.
    pub(super) fn complete_oam_dma_from_source(&mut self, source: &[u8]) {
        assert!(
            source.len() >= self.ppu.oam.len() * 2,
            "OAM DMA source is shorter than the hardware OAM payload"
        );
        self.program_dma0_ppu_target(DMA_MODE_ONE_REGISTER, PPU_BBUS_OAM_DATA);
        for i in 0..self.ppu.oam.len() {
            self.ppu.oam[i] = read_word_from_slice(source, i * 2);
        }
        self.resident_oam_dma = Some(self.ppu.oam.clone());
        self.record_completed_oam_dma_for_display_boundary();
        // OAM-law clauses: the real transfer carries the software shadow as
        // it stands at this vblank (post-main), independent of the operand the
        // legacy pipeline selected above — and C performs exactly one such
        // transfer per completed main iteration (`nmi_boolean` gates
        // NMI_DoUpdates). A modeled NMI on a frame whose counter has not
        // advanced is a held vblank hardware skipped.
        // Scheduled work pending across this NMI means the main iteration has
        // not completed yet (C's nmi_boolean stays set until the iteration's
        // suspended tail returns); the transfer belongs to the completion
        // slice, which carries the post-iteration shadow. Held NMIs never
        // reach this point at all (the latch skips NMI_DoUpdates).
        if self.game_execution_scheduler.current_work().is_none() {
            let mut law = vec![0u16; self.ppu.oam.len()];
            if publish_oam_shadow(&mut law, self.sprite_oam_shadow_buffer()) {
                if debug_frame_selection_env_matches(
                    "ZELDA3_DEBUG_OAM_LAW_EVENTS",
                    self.frame_ctr_dbg,
                ) {
                    eprintln!(
                        "oam_law_transfer host={} w204={:04x} fc={:02x}",
                        self.frame_ctr_dbg, law[204], self.game_state.frame.frame_counter
                    );
                }
                self.oam_law_pending = Some(law);
            }
        }
    }

    pub(super) fn nmi_upload_tilemap(&mut self) {
        let Some((target, word_count)) =
            full_tilemap_nmi_vram_region(self.game_state.display.nmi_load_target_page())
        else {
            return;
        };
        if target + word_count <= self.ppu.vram.len() {
            self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
            let buf = self.tilemap_upload_stripe_buffer().to_vec();
            self.mark_effective_dma_vram_range(target..target + word_count);
            for i in 0..word_count {
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
        self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
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
                    self.mark_effective_dma_vram_word(dst);
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
        for packet in nmi_vram_copy_packets(&data) {
            match packet.direction {
                NmiVramCopyDirection::Horizontal => {
                    self.copy_to_vram_slice(packet.destination, packet.data, packet.data.len());
                }
                NmiVramCopyDirection::Vertical => {
                    self.copy_to_vram_vertical_slice(
                        packet.destination,
                        packet.data,
                        packet.data.len(),
                    );
                }
            }
        }
    }

    pub(super) fn nmi_core_link_graphics_update(
        &mut self,
        captured_operands: Option<PreMainLinkDmaOperands>,
    ) {
        // Animation-modeled asset renderer M1: tag the Link CHR VRAM slots with
        // the active Link DMA graphics index as the logical source. Write-only
        // bookkeeping; does not affect the VRAM bytes written below.
        let link_pack = captured_operands.map_or_else(
            || {
                (self
                    .game_state
                    .player
                    .follower_link
                    .link_dma_graphics_index_word()
                    >> 1) as u16
            },
            |operands| operands.link_pack,
        );

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
                let src_addr = captured_operands
                    .map(|operands| usize::from(operands.sources.source(source)))
                    .unwrap_or_else(|| self.live_link_dma_source(source));
                let base_off = (src_addr.saturating_sub(0x8000) >> 5) as u16;
                self.copy_asset_bytes_to_vram(dst, &link_graphics, src_addr, len);
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
            let src_addr = captured_operands
                .map(|operands| usize::from(operands.sources.source(source)))
                .unwrap_or_else(|| self.live_link_dma_source(source));
            let base_off = crate::chr_source::CHR_LINK_SRC_RAM_FLAG | ((src_addr >> 5) as u16);
            self.copy_ram_bytes_to_vram(dst, src_addr, len);
            self.vram_chr_source.record_tiles_from(
                dst,
                (len / 2).div_ceil(16),
                crate::chr_source::CHR_KIND_LINK,
                link_pack,
                base_off,
            );
        }
        if let Some(operands) = captured_operands {
            self.copy_to_vram_slice(
                0x4240,
                &operands.expanded_high_planes[..LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN],
                LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN,
            );
        } else {
            self.copy_ram_bytes_to_vram_absolute(
                0x4240,
                LINK_DMA_EXPANDED_HIGH_PLANES_START,
                LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN,
            );
        }
        for (dst, source) in [
            (0x4300, LinkDmaSourceSlot::HeadPointerLower),
            (0x4320, LinkDmaSourceSlot::BodyPointerLower),
        ] {
            let src_addr = captured_operands
                .map(|operands| usize::from(operands.sources.source(source)))
                .unwrap_or_else(|| self.live_link_dma_source(source));
            self.copy_ram_bytes_to_vram(dst, src_addr, 0x40);
        }
        if let Some(operands) = captured_operands {
            self.copy_to_vram_slice(
                0x4340,
                &operands.expanded_high_planes[LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN..],
                LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN,
            );
        } else {
            self.copy_ram_bytes_to_vram_absolute(
                0x4340,
                LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN,
                LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN,
            );
        }

        if self.game_state.display.has_travel_bird_tile_upload() {
            for (dst, source) in [
                (0x40e0, LinkDmaSourceSlot::TravelBirdUpper),
                (0x41e0, LinkDmaSourceSlot::TravelBirdLower),
            ] {
                let src_addr = captured_operands
                    .map(|operands| usize::from(operands.sources.source(source)))
                    .unwrap_or_else(|| self.live_link_dma_source(source));
                self.copy_ram_bytes_to_vram(dst, src_addr, 0x40);
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
        self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
        let words = len >> 1;
        for i in 0..words {
            if dstv < self.ppu.vram.len() && i * 2 + 1 < src.len() {
                self.mark_effective_dma_vram_word(dstv);
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
        self.program_dma0_ppu_target(DMA_MODE_ONE_REGISTER, PPU_BBUS_VRAM_DATA_LOW);
        for i in 0..num {
            if addr + i < self.ppu.vram.len() && i < src.len() {
                self.mark_effective_dma_vram_word(addr + i);
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
        source_addr: usize,
        len: usize,
    ) {
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
        source_addr: usize,
        len: usize,
    ) {
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
        self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
        self.mark_effective_dma_vram_range(dst_word..dst_word + len.div_ceil(2));
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
            self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
            let poly_buf = self.polyhedral_tile_buffer().to_vec();
            let mut display_vram = None;
            self.mark_effective_dma_vram_range(0x5800..0x5c00);
            for i in 0..0x400 {
                let dst = 0x5800 + i;
                let value = read_word_from_slice(&poly_buf, i * 2);
                if self.ppu.vram[dst] != value && display_vram.is_none() {
                    display_vram = Some(self.ppu.vram.clone());
                }
                self.ppu.vram[dst] = value;
            }
            if self.nmi_poly_upload_from_deferred {
                self.set_obj_vram_latch_traced(None);
            } else if let Some(display_vram) = display_vram {
                self.obj_vram_latch_generation = self.obj_vram_latch_generation.wrapping_add(1);
                self.set_obj_vram_latch_traced(Some(display_vram));
            }
            self.nmi_poly_upload_from_deferred = false;
            self.clear_pending_polyhedral_update();
        }
    }

    pub(super) fn nmi_update_bg_char_half(&mut self) {
        let dst = self.game_state.display.nmi_load_target_page() as usize * 256;
        let buf = self.background_character_half_buffer().to_vec();
        self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
        self.mark_effective_dma_vram_range(dst..dst + 0x200);
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
        self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);
        self.mark_effective_dma_vram_range(0x7c00..0x7ff0);
        for i in 0..0x3f0 {
            self.ppu.vram[0x7c00 + i] = read_word_from_slice(&buf, i * 2);
        }
        // Semantic glyph placements are a second representation of the same
        // VWF pixels. Publish them at the DMA boundary too; exposing the
        // CPU-authored list earlier lets the modern renderer paint text that
        // the SNES PPU has not received yet.
        self.publish_bg3_vwf_glyph_runs();
        self.record_completed_dialogue_metadata_for_display_boundary();
        self.clear_core_update_disable_flag();
    }

    pub(super) fn nmi_update_load_light_world_map(&mut self) {
        const LIGHT_WORLD_TILEMAP_DSTS: [usize; 4] = [0, 0x20, 0x1000, 0x1020];
        if let Some(tilemap) = self.asset_raw(67).map(Vec::from) {
            self.program_dma0_ppu_target(DMA_MODE_ONE_REGISTER, PPU_BBUS_VRAM_DATA_LOW);
            let mut src = 0usize;
            for dst_base in LIGHT_WORLD_TILEMAP_DSTS {
                let mut dst = dst_base;
                for _ in 0..0x20 {
                    for i in 0..0x20 {
                        if src + i < tilemap.len() {
                            self.mark_effective_dma_vram_word(dst + i);
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
            self.program_dma0_ppu_target(DMA_MODE_TWO_REGISTERS, PPU_BBUS_VRAM_DATA_LOW);

            if !vertical {
                if is_memset {
                    if stripes.len() < 2 {
                        return;
                    }
                    let value = stripes[0] as u16 | ((stripes[1] as u16) << 8);
                    for i in 0..((len + 1) >> 1) {
                        self.mark_effective_dma_vram_word(vmem_addr as usize + i);
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
                    self.mark_effective_dma_vram_word(vmem_addr as usize + i * 32);
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
                    self.mark_effective_dma_vram_word(vmem_addr as usize + i * 32);
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
        self.record_completed_ppu_registers_for_display_boundary();
    }

    pub(super) fn write_vram_byte(&mut self, byte_offset: usize, value: u8) {
        let word = byte_offset >> 1;
        if word >= self.ppu.vram.len() {
            return;
        }
        let current = self.ppu.vram[word];
        self.mark_effective_dma_vram_word(word);
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
