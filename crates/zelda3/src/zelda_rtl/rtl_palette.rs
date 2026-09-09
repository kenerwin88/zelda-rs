//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (palette).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    /// Audit the mirror's committed CGRAM image (what the renderer substitutes) against the live
    /// PPU CGRAM. Used at render-capture points to catch a stale committed image between upload
    /// commits (the main-vs-shadow audit cannot see it).
    pub fn audit_cgram_mirror(&self, ppu_cgram: &[u16]) -> zelda3_palette::BankAudit {
        self.game_state
            .display
            .palette_provenance
            .0
            .audit_cgram(ppu_cgram)
    }

    /// Diagnostic: compact per-bank source-tag histogram of the provenance mirror. Confirms a
    /// checkpoint-restored mirror carries its true derivation tags (asset/constant/computed)
    /// rather than an all-`Copied` shadow reconstitution.
    pub fn palette_mirror_tag_histogram(&self) -> String {
        self.game_state
            .display
            .palette_provenance
            .0
            .tag_histogram_line()
    }

    /// Snapshot the palette-provenance mirror at a CGRAM upload (the mirror's
    /// equivalent of `memcpy(cgram, main_palette_buffer)`), and under
    /// `ZELDA3_PALETTE_PROVENANCE_CHECK=1|panic` audit the mirror's main bank
    /// against the WRAM shadow the upload actually read. The audit line is
    /// rate-limited to changes in the (mismatch, unknown) counts.
    pub(crate) fn commit_palette_provenance_cgram(&mut self) {
        self.game_state.display.palette_provenance.0.commit_cgram();
        let Some(mode) = crate::game_state::palette_provenance_check_mode() else {
            return;
        };
        let shadow = &self.ram[crate::game_state::constants::MAIN_PALETTE_BUFFER
            ..crate::game_state::constants::MAIN_PALETTE_BUFFER + 0x200];
        let audit = self
            .game_state
            .display
            .palette_provenance
            .0
            .audit_bank(zelda3_palette::Bank::Main, shadow);
        // Also audit the committed CGRAM image (what the renderer actually consumes) against the
        // live PPU CGRAM. The main-vs-shadow audit alone provably misses a stale CGRAM image (a
        // restore that bulk-loads ppu.cgram without a following upload — see
        // `reconstitute_palette_mirror_from_shadow`). At an upload commit these agree by
        // construction; the audit guards the upload/restore paths against regressing.
        let cgram_audit = self
            .game_state
            .display
            .palette_provenance
            .0
            .audit_cgram(&self.ppu.cgram);
        use std::sync::atomic::{AtomicU64, Ordering};
        static LAST: AtomicU64 = AtomicU64::new(u64::MAX);
        let packed = ((audit.mismatches.len() as u64) << 48)
            | ((audit.unknown.len() as u64) << 32)
            | ((cgram_audit.mismatches.len() as u64) << 16)
            | cgram_audit.unknown.len() as u64;
        if LAST.swap(packed, Ordering::Relaxed) != packed {
            let first_mismatch = audit.mismatches.first().map(|w| {
                format!(
                    " first_mismatch=idx{}:mirror={:04x}:actual={:04x}",
                    w.index,
                    w.mirror.unwrap_or(0),
                    w.actual
                )
            });
            let first_unknown = audit
                .unknown
                .first()
                .map(|w| format!(" first_unknown=idx{}", w.index));
            eprintln!(
                "palette_provenance_coherence frame={} mismatches={} unknown={} cgram_mismatches={} cgram_unknown={}{}{}",
                self.frame_ctr_dbg,
                audit.mismatches.len(),
                audit.unknown.len(),
                cgram_audit.mismatches.len(),
                cgram_audit.unknown.len(),
                first_mismatch.unwrap_or_default(),
                first_unknown.unwrap_or_default(),
            );
        }
        if mode == crate::game_state::ProvenanceCheckMode::Panic
            && (!audit.is_clean() || !cgram_audit.is_clean())
        {
            panic!(
                "palette provenance mirror diverged at frame {} (main: mismatches={} unknown={}; \
                 cgram: mismatches={} unknown={})",
                self.frame_ctr_dbg,
                audit.mismatches.len(),
                audit.unknown.len(),
                cgram_audit.mismatches.len(),
                cgram_audit.unknown.len(),
            );
        }
    }

    pub(crate) fn palette_swap_enabled(&self) -> bool {
        self.game_state.sprites.follower_runtime.palette_swap_flag() != 0
    }

    pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
        self.system_signals_mut().increment_cgram_update_flag()
    }

    pub(crate) fn clear_cgram_update_flag(&mut self) {
        self.system_signals_mut().clear_cgram_update_flag();
    }

    pub(crate) fn copy_overworld_sprite_palette_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.overworld_config_table_mut()
            .copy_sprite_palette_range(dst, data, src, len);
    }

    pub(crate) fn clear_aux_visible_subpalettes(&mut self) {
        self.palette_buffer_mut().clear_aux_visible_subpalettes();
    }

    pub(crate) fn clear_main_visible_subpalettes(&mut self) {
        self.palette_buffer_mut().clear_main_visible_subpalettes();
    }

    pub(crate) fn clear_aux_sprite_subpalettes(&mut self) {
        self.palette_buffer_mut().clear_aux_sprite_subpalettes();
    }

    pub(crate) fn select_overworld_aux_palette_offset(&mut self) {
        self.palette_buffer_mut()
            .select_overworld_aux_palette_offset();
    }

    pub(crate) fn initialize_palette_mirror_from_zeroed_buffers(&mut self) {
        self.palette_buffer_mut()
            .initialize_mirror_from_zeroed_buffers();
    }

    pub(crate) fn copy_main_palette_bytes_tagged(
        &mut self,
        src: &[u8],
        len: usize,
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .copy_main_palette_bytes_tagged(src, len, source);
    }

    pub(crate) fn set_palette_main_indoors(&mut self, value: u8) {
        self.palette_buffer_mut().set_palette_main_indoors(value);
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.palette_buffer_mut().set_hud_palette(value);
    }

    pub(crate) fn set_overworld_palette_aux2_hi(&mut self, value: u8) {
        self.palette_buffer_mut()
            .set_overworld_palette_aux2_hi(value);
    }

    pub(crate) fn set_overworld_palette_aux3_lo(&mut self, value: u8) {
        self.palette_buffer_mut()
            .set_overworld_palette_aux3_lo(value);
    }

    pub(crate) fn set_overworld_palette_mode(&mut self, value: u8) {
        self.palette_buffer_mut().set_overworld_palette_mode(value);
    }

    pub(crate) fn clear_agahnim_palette_settings(&mut self, len: usize) {
        self.ram[AGAHNIM_PAL_SETTING..AGAHNIM_PAL_SETTING + len].fill(0);
    }

    pub(crate) fn agahnim_palette_word(&self, index: usize) -> u16 {
        read_le_u16(&self.ram, AGAHNIM_PAL_SETTING + index * 2)
    }

    pub(crate) fn set_agahnim_palette_word(&mut self, index: usize, value: u16) {
        write_le_u16(&mut self.ram, AGAHNIM_PAL_SETTING + index * 2, value);
    }

    pub(crate) fn set_trinexx_red_shell_palette_delay(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut().set_red_shell_delay(value);
    }

    pub(crate) fn set_trinexx_blue_shell_palette_delay(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut()
            .set_blue_shell_delay(value);
    }

    pub(crate) fn set_trinexx_red_shell_palette_step(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut().set_red_shell_step(value);
    }

    pub(crate) fn set_trinexx_blue_shell_palette_step(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut().set_blue_shell_step(value);
    }

    pub(crate) fn decrement_trinexx_red_shell_palette_delay(&mut self) {
        self.trinexx_palette_bridge_mut()
            .decrement_red_shell_delay();
    }

    pub(crate) fn decrement_trinexx_blue_shell_palette_delay(&mut self) {
        self.trinexx_palette_bridge_mut()
            .decrement_blue_shell_delay();
    }

    pub(crate) fn increment_trinexx_red_shell_palette_step(&mut self) -> u8 {
        self.trinexx_palette_bridge_mut().increment_red_shell_step()
    }

    pub(crate) fn increment_trinexx_blue_shell_palette_step(&mut self) -> u8 {
        self.trinexx_palette_bridge_mut()
            .increment_blue_shell_step()
    }

    pub(super) fn stage_dungeon_faded_filter_first_palette_scanout(&mut self) {
        // Every interrupted first walk publishes the completed shadow DMA from
        // the preceding resumed suffix. The renderer decodes early Link DMA
        // from the same host operands, reconstructed separately in
        // compose_display_oam.
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: dungeon_faded_filter_first_pass_oam_scanout(),
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn publish_completed_palette_filter_cgram_scanout(&mut self) {
        let snapshot = self
            .display_snapshot
            .as_mut()
            .expect("completed palette filter captures its display boundary first");
        if snapshot.cgram_scanout_generation == CgramScanoutGeneration::RetainPreviousPresented {
            // Entry-scoped scanout ownership is more precise than the caller's
            // post-completion room/stair identity.
            return;
        }
        snapshot.cgram_scanout_generation = CgramScanoutGeneration::LiveAfterNmi;
    }

    pub(super) fn retain_palette_filter_input_cgram_on_next_display_capture(&mut self) {
        self.next_display_cgram_override = Some(
            self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 0x200]
                .chunks_exact(2)
                .map(|word| u16::from_le_bytes([word[0], word[1]]))
                .collect(),
        );
    }

    pub(super) fn retain_completed_palette_filter_cgram_scanout(&mut self) {
        let snapshot = self
            .display_snapshot
            .as_mut()
            .expect("completed palette filter captures its display boundary first");
        snapshot.cgram_scanout_generation = CgramScanoutGeneration::RetainPreviousPresented;
    }

    pub(super) fn publish_completed_spiral_palette_filter_scanout(&mut self) {
        if !rom_spiral_palette_completion_publishes_live_cgram_and_oam(
            self.game_state.frame,
            self.game_state.world.location.dungeon_room_index(),
            self.game_state.dungeon.stair_movement.staircase_index(),
        ) {
            self.retain_completed_palette_filter_cgram_scanout();
            return;
        }
        self.publish_completed_palette_filter_cgram_scanout();
        let snapshot = self.display_snapshot.as_mut().unwrap();
        snapshot.oam_scanout_source = OamScanoutSource::ComposeCompletedWorkAfterNmi;
    }

    pub(super) fn record_display_cgram_candidates(
        &mut self,
        following: &DisplaySnapshot,
        captured_before_nmi: Option<Vec<u16>>,
    ) {
        if captured_before_nmi.is_none() {
            self.debug_display_cgram_candidates.clear();
            return;
        }
        let mut candidates = vec![DebugDisplayCgramCandidate {
            name: "selected",
            cgram: self.ppu.cgram.clone(),
        }];
        candidates.push(DebugDisplayCgramCandidate {
            name: "captured_before_nmi",
            cgram: captured_before_nmi.unwrap(),
        });
        candidates.push(DebugDisplayCgramCandidate {
            name: "live_after_main",
            cgram: following.ppu.cgram.clone(),
        });
        if let Some(cgram) = self.cgram_upload_latch.as_ref() {
            candidates.push(DebugDisplayCgramCandidate {
                name: "before_nmi_upload",
                cgram: cgram.clone(),
            });
        }
        if let Some(cgram) = self.last_presented_cgram.as_ref() {
            candidates.push(DebugDisplayCgramCandidate {
                name: "last_presented",
                cgram: cgram.clone(),
            });
        }
        if let Some(cgram) = following.cgram_scanout_override.as_ref() {
            candidates.push(DebugDisplayCgramCandidate {
                name: "scanout_override",
                cgram: cgram.clone(),
            });
        }
        self.debug_display_cgram_candidates = candidates;
    }

    pub(super) fn compose_display_cgram(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        let capture_publication_candidates =
            crate::debug_env::var_os("ZELDA3_CAPTURE_DISPLAY_CANDIDATES").is_some()
                || crate::debug_env::var_os("ZELDA3_CAPTURE_OBJ_STATE_LEDGER").is_some();
        let captured_before_nmi = capture_publication_candidates.then(|| self.ppu.cgram.clone());
        if crate::debug_env::var_os("ZELDA3_DEBUG_SPIRAL_CGRAM").is_some() {
            let frame = crate::game_state::FrameState::load_from_ram(&following.ram);
            if frame.main_module == 7 && frame.submodule == 0x0e {
                eprintln!(
                    "spiral_cgram_compose host={} frame={:02x}/{:02x}/{:02x}/fc{:02x} generation={:?} compose_live={}",
                    self.frame_ctr_dbg,
                    frame.main_module,
                    frame.submodule,
                    frame.subsubmodule,
                    frame.frame_counter,
                    following.cgram_scanout_generation,
                    plan.compose_live_cgram,
                );
            }
        }
        if let Some(cgram) = following
            .cgram_scanout_override
            .as_ref()
            .filter(|cgram| cgram.len() == self.ppu.cgram.len())
        {
            // A Live timing receipt names the palette of the exact surface
            // returned by the authority for this host call. The native
            // publication plan remains useful shadow evidence, but it cannot
            // retain a different generation after that explicit semantic
            // ownership has been attached to the immutable snapshot.
            self.ppu.cgram.copy_from_slice(cgram);
            self.record_display_cgram_candidates(following, captured_before_nmi);
            return;
        }
        if !plan.compose_live_cgram {
            self.record_display_cgram_candidates(following, captured_before_nmi);
            return;
        }

        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or_else(|| crate::game_state::FrameState::load_from_ram(&self.ram));
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        let following_staircase = crate::game_state::DungeonState::load_from_ram(&following.ram)
            .stair_movement
            .staircase_index();
        let retain_straight_palette_caller_cgram =
            straight_interroom_fadeout_no_nmi_caller_retains_presented_display(
                entry_frame,
                following_frame,
                following_room,
                following_staircase,
                following.ram[crate::game_state::constants::NMI_BOOLEAN],
                plan.link_obj_scanout_generation,
            ) || straight_interroom_palette_caller_retains_presented_display(
                entry_frame,
                following_frame,
                following_room,
                following_staircase,
            ) || straight_interroom_palette_completion_retains_presented_cgram(
                entry_frame,
                following_frame,
                following_room,
                following_staircase,
            );

        // The NMI's main-palette-buffer upload is only visible on the next
        // scanout. Direct CGRAM writes outside that upload stay same-frame
        // visible through the following image.
        if plan.world_map_fade_display {
            // The world-map fade publishes Mode 7 memory and INIDISP now, but
            // CGRAM remains on the generation consumed by the preceding NMI.
        } else if dungeon_brightness_entry_retains_presented_player_graphics(
            entry_frame,
            following_frame,
            following_room,
        ) {
            // Module07_0A publishes the new brightness immediately, but the
            // first scanout retains the player graphics and palette consumed
            // at the host boundary. The next NMI publishes the filtered CGRAM.
        } else if plan.publish_live_dungeon_state_13_palette_and_registers {
            // The room-$41 state-13 body authors the palette and subscreen
            // enable before its caller is interrupted. Both therefore belong
            // to this scanout even though the common Module 7 suffix resumes
            // on the next host frame.
            self.ppu.cgram.clone_from(&following.ppu.cgram);
        } else if plan.dungeon_state_13_phase == DungeonState13PublicationPhase::CallerReturn
            || plan.dungeon_faded_filter_phase == DungeonFadedFilterPublicationPhase::CallerReturn
        {
            // The caller-only slice does not cross a palette publication
            // boundary. Retain the palette composed for the preceding main
            // slice even though the CPU has authored the following buffer.
            if let Some(cgram) = self.last_presented_cgram.as_ref() {
                self.ppu.cgram.copy_from_slice(cgram);
            }
        } else if plan.dungeon_state_13_phase == DungeonState13PublicationPhase::RecurringMain {
            // Recurring state-13 bodies begin after an ordinary NMI. Their
            // newly authored palette buffer remains private until the next
            // upload, so this scanout owns the generation consumed on entry.
            if let Some(cgram) = self.cgram_upload_latch.as_ref() {
                self.ppu.cgram.copy_from_slice(cgram);
            } else {
                self.ppu.cgram.clone_from(&following.ppu.cgram);
            }
        } else if retain_straight_palette_caller_cgram {
            // The straight-stair palette loop alternates completed main/NMI
            // slices with caller-only slices. Without an NMI edge, the newly
            // authored palette buffer is not hardware-visible; retain the
            // palette published by the preceding completed slice.
            if let Some(cgram) = self.last_presented_cgram.as_ref() {
                self.ppu.cgram.copy_from_slice(cgram);
            }
        } else {
            match following.cgram_scanout_generation {
                CgramScanoutGeneration::RetainPreviousPresented => {
                    if crate::debug_env::var_os("ZELDA3_DEBUG_SPIRAL_CGRAM").is_some() {
                        eprintln!(
                            "spiral_cgram_source host={} composed={:04x}/{:04x}/{:04x} captured={:04x}/{:04x}/{:04x} last={:?} latch={:?}",
                            self.frame_ctr_dbg,
                            self.ppu.cgram[34],
                            self.ppu.cgram[35],
                            self.ppu.cgram[36],
                            following.ppu.cgram[34],
                            following.ppu.cgram[35],
                            following.ppu.cgram[36],
                            self.last_presented_cgram
                                .as_ref()
                                .map(|cgram| [cgram[34], cgram[35], cgram[36]]),
                            self.cgram_upload_latch
                                .as_ref()
                                .map(|cgram| [cgram[34], cgram[35], cgram[36]]),
                        );
                    }
                    if let Some(cgram) = self.last_presented_cgram.as_ref() {
                        self.ppu.cgram.copy_from_slice(cgram);
                    }
                }
                CgramScanoutGeneration::LiveAfterNmi => {
                    self.ppu.cgram.clone_from(&following.ppu.cgram);
                }
                CgramScanoutGeneration::BeforeNmiUpload => {
                    if let Some(latch) = self.cgram_upload_latch.as_ref() {
                        self.ppu.cgram.copy_from_slice(latch);
                    } else {
                        self.ppu.cgram.clone_from(&following.ppu.cgram);
                    }
                }
            }
        }
        self.record_display_cgram_candidates(following, captured_before_nmi);
    }

    pub(super) fn compose_effective_presented_cgram(&mut self, following: &DisplaySnapshot) {
        let Some(completed_cgram) = following
            .effective_presented_dma
            .as_ref()
            .and_then(|receipt| receipt.completed_cgram.as_deref())
            .filter(|cgram| cgram.len() == self.ppu.cgram.len())
        else {
            return;
        };

        // This is an observed transfer from the explicit leading NMI, not a
        // prediction from the module that happens to be live when rendering.
        // The DMA completed during vblank, so its installed palette is the
        // one fetched by the active scanout captured immediately beforehand.
        self.ppu.cgram.clone_from_slice(completed_cgram);
    }

    pub fn zelda_debug_display_cgram_candidates(&self) -> &[DebugDisplayCgramCandidate] {
        &self.debug_display_cgram_candidates
    }

    pub(super) fn record_completed_cgram_dma_for_display_boundary(&mut self) {
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            writes.completed_cgram = Some(self.ppu.cgram.clone());
        }
    }

    /// Capture CGRAM after running all active HDMA channels for the first scanline.
    ///
    /// ALttP loads dungeon floor palette entries via HDMA per-scanline. The pre-render
    /// CGRAM is black for these entries because HDMA hasn't run yet. Running one HDMA
    /// line gives a CGRAM representative of the visible screen area.
    ///
    /// Runs all 8 HDMA channels (not just 6+7) because CGRAM writes can come from
    /// any channel depending on the room. Saves and restores all PPU state modified
    /// by HDMA so the actual render call (`zelda_draw_ppu_frame`) is unaffected.
    pub fn cgram_after_first_hdma_line(&mut self) -> Vec<u16> {
        let mut channels = self.dma.channel;
        for (index, channel) in channels.iter_mut().enumerate() {
            channel.hdma_active = self.game_state.display.is_hdma_channel_enabled(index);
        }

        let saved_cgram = self.ppu.cgram.clone();
        let saved_cgram_pointer = self.ppu.cgram_pointer;
        let saved_cgram_second_write = self.ppu.cgram_second_write;
        let saved_cgram_buffer = self.ppu.cgram_buffer;
        let saved_forced_blank = self.ppu.forced_blank;
        let saved_brightness = self.ppu.brightness;
        let saved_screen_enabled = self.ppu.screen_enabled;
        let saved_window1_left = self.ppu.window1_left;
        let saved_window1_right = self.ppu.window1_right;
        let saved_window2_left = self.ppu.window2_left;
        let saved_window2_right = self.ppu.window2_right;
        let saved_scroll_prev = self.ppu.scroll_prev;
        let saved_scroll_prev2 = self.ppu.scroll_prev2;
        let saved_bg_scrolls: [(u16, u16); 4] =
            std::array::from_fn(|i| (self.ppu.bg_layer[i].h_scroll, self.ppu.bg_layer[i].v_scroll));
        let saved_m7_matrix = self.ppu.m7_matrix;
        let saved_m7_prev = self.ppu.m7_prev;

        let mut hdma: [SimpleHdma; 8] = Default::default();
        for i in 0..8 {
            self.simple_hdma_init(&mut hdma[i], &channels[i]);
        }
        for channel in hdma.iter_mut() {
            self.simple_hdma_do_line(channel);
        }

        let result = self.ppu.cgram.clone();

        self.ppu.cgram = saved_cgram;
        self.ppu.cgram_pointer = saved_cgram_pointer;
        self.ppu.cgram_second_write = saved_cgram_second_write;
        self.ppu.cgram_buffer = saved_cgram_buffer;
        self.ppu.forced_blank = saved_forced_blank;
        self.ppu.brightness = saved_brightness;
        self.ppu.screen_enabled = saved_screen_enabled;
        self.ppu.window1_left = saved_window1_left;
        self.ppu.window1_right = saved_window1_right;
        self.ppu.window2_left = saved_window2_left;
        self.ppu.window2_right = saved_window2_right;
        self.ppu.scroll_prev = saved_scroll_prev;
        self.ppu.scroll_prev2 = saved_scroll_prev2;
        for (i, &(h_scroll, v_scroll)) in saved_bg_scrolls.iter().enumerate() {
            self.ppu.bg_layer[i].h_scroll = h_scroll;
            self.ppu.bg_layer[i].v_scroll = v_scroll;
        }
        self.ppu.m7_matrix = saved_m7_matrix;
        self.ppu.m7_prev = saved_m7_prev;

        result
    }

    /// Simulate 224 HDMA scanlines and capture a full CGRAM snapshot per scanline.
    pub fn ppu_scanline_cgram(&mut self) -> Vec<Vec<u16>> {
        for i in 0..8 {
            self.dma.channel[i].hdma_active = self.game_state.display.is_hdma_channel_enabled(i);
        }
        let saved_cgram = self.ppu.cgram.clone();
        let saved_cgram_pointer = self.ppu.cgram_pointer;
        let saved_cgram_second_write = self.ppu.cgram_second_write;
        let saved_cgram_buffer = self.ppu.cgram_buffer;
        let saved_forced_blank = self.ppu.forced_blank;
        let saved_brightness = self.ppu.brightness;
        let saved_screen_enabled = self.ppu.screen_enabled;
        let saved_window1_left = self.ppu.window1_left;
        let saved_window1_right = self.ppu.window1_right;
        let saved_window2_left = self.ppu.window2_left;
        let saved_window2_right = self.ppu.window2_right;
        let saved_fcr = self.ppu.fixed_color_r;
        let saved_fcg = self.ppu.fixed_color_g;
        let saved_fcb = self.ppu.fixed_color_b;

        let channels: [_; 8] = std::array::from_fn(|i| self.dma.channel[i]);
        let mut hdma: [SimpleHdma; 8] = Default::default();
        for i in 0..8 {
            self.simple_hdma_init(&mut hdma[i], &channels[i]);
        }

        let mut result = Vec::with_capacity(224);
        for _ in 0..224 {
            for channel in hdma.iter_mut() {
                self.simple_hdma_do_line(channel);
            }
            result.push(self.ppu.cgram.clone());
        }

        self.ppu.cgram = saved_cgram;
        self.ppu.cgram_pointer = saved_cgram_pointer;
        self.ppu.cgram_second_write = saved_cgram_second_write;
        self.ppu.cgram_buffer = saved_cgram_buffer;
        self.ppu.forced_blank = saved_forced_blank;
        self.ppu.brightness = saved_brightness;
        self.ppu.screen_enabled = saved_screen_enabled;
        self.ppu.window1_left = saved_window1_left;
        self.ppu.window1_right = saved_window1_right;
        self.ppu.window2_left = saved_window2_left;
        self.ppu.window2_right = saved_window2_right;
        self.ppu.fixed_color_r = saved_fcr;
        self.ppu.fixed_color_g = saved_fcg;
        self.ppu.fixed_color_b = saved_fcb;
        result
    }

    /// Rebuild the palette-provenance mirror from the restored palette shadow
    /// after a full-state snapshot load. Each word is tagged `Copied` with its
    /// restored value, so the mirror equals the shadow (coherence check clean,
    /// no `Unknown`) and the renderer's mirror substitution stays valid. Only
    /// used at snapshot-restore boundaries, where the shadow is the authority.
    pub(super) fn reconstitute_palette_mirror_from_shadow(&mut self) {
        use zelda3_palette::{Bank, MirrorWord, SourceTag, PALETTE_WORDS};
        // The committed CGRAM image the renderer consumes tracks what the PPU holds, which the
        // restore bulk-loads directly; capture it before borrowing the mirror.
        let cgram: Vec<u16> = self.ppu.cgram.to_vec();
        let pb = &self.game_state.display.palette_buffer;
        let main: Vec<u16> = (0..PALETTE_WORDS).map(|i| pb.main_color(i)).collect();
        let aux: Vec<u16> = (0..PALETTE_WORDS).map(|i| pb.aux_color(i)).collect();
        let backup = pb.overworld_palette_backup().to_vec();
        let mirror = &mut self.game_state.display.palette_provenance.0;
        for i in 0..PALETTE_WORDS {
            mirror.bank_mut(Bank::Main)[i] = MirrorWord::Known(main[i], SourceTag::Copied);
            mirror.bank_mut(Bank::Aux)[i] = MirrorWord::Known(aux[i], SourceTag::Copied);
            let off = i * 2;
            let bv = u16::from(backup.get(off).copied().unwrap_or(0))
                | (u16::from(backup.get(off + 1).copied().unwrap_or(0)) << 8);
            mirror.bank_mut(Bank::Backup)[i] = MirrorWord::Known(bv, SourceTag::Copied);
        }
        // A restore also bulk-loads ppu.cgram; the committed CGRAM image is otherwise only
        // refreshed at upload commits (which may not run for many frames during a fade), so
        // reconstitute it here too or the renderer substitutes a stale palette.
        mirror.reconstitute_cgram(&cgram);
    }

    pub(super) fn palette_asset_word_snes(&self, addr: u32) -> Option<u16> {
        for &(base, asset) in PALETTE_ASSET_SNES_RANGES {
            let Some(byte_offset) = addr.checked_sub(base).map(|offset| offset as usize) else {
                continue;
            };
            let data = self.asset_raw(asset)?;
            if byte_offset + 1 < data.len() {
                return Some(read_word_from_slice(data, byte_offset));
            }
        }
        None
    }
}
