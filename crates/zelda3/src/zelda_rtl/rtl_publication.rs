//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (publication).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn staged_bg_and_sprite_decompression_buffers(&self) -> Vec<u8> {
        GraphicsDecompressionScratch::staged_bg_and_sprite_buffers(&self.ram)
    }

    pub(crate) fn mark_select_file_save_slot_present(&mut self, slot: usize) {
        self.select_file_menu_mut().mark_save_slot_present(slot);
    }

    #[track_caller]
    pub(crate) fn assert_native_display_state_matches_ram(&self) {
        self.game_state
            .display
            .debug_assert_core_matches_ram(&self.ram);
    }

    pub(super) fn stage_dungeon_faded_filter_completion_scanout(&mut self) {
        // Completion reaches the wait loop before the next leading NMI. That
        // NMI performs a real OAM DMA from the completed host-boundary shadow;
        // unlike an interrupted palette walk, it must not retain the older
        // resident/published OAM generation. Link OBJ uses operands sampled at
        // that same host boundary.
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeHostBoundaryShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn capture_animated_bg_scanout(&self) -> Option<AnimatedBgScanout> {
        if !self.rom_startup_timing() || !self.game_state.display.has_animated_tile_data_source() {
            return None;
        }
        let destination_address = self
            .game_state
            .display
            .animated_tile_vram_destination_usize();
        let vram = self
            .ppu
            .vram
            .get(destination_address..destination_address.checked_add(0x200)?)?
            .to_vec();
        Some(AnimatedBgScanout {
            destination_address,
            vram,
            logical_sources: self.vram_chr_source.clone(),
            preview_sources: self.vram_chr_preview_source.clone(),
        })
    }

    /// Stage the held-latch NMI which interrupted a long Module 7 call. Snes9x
    /// returns the following internal field from its next host call, so this is
    /// a typed next-publication receipt rather than a mutation of the image
    /// whose visible scanlines had already completed.
    #[track_caller]
    pub(super) fn stage_interrupted_dungeon_submodule_publication(&mut self) {
        let frame = self.game_state.frame;
        let retires_pre_spotlight_scanout = frame.main_module == 7
            && frame.submodule == 0x0f
            && frame.subsubmodule == 1
            && self
                .game_execution_scheduler
                .active_field_precedes_current_scheduled_work()
            && self.display_snapshot.as_ref().is_some_and(|published| {
                let published_frame = crate::game_state::FrameState::load_from_ram(&published.ram);
                published_frame.main_module == 7
                    && published_frame.submodule == 0x0f
                    && published_frame.subsubmodule == 0
            });
        let explicitly_staged_spotlight = self.next_display_spotlight_scanout.take();
        let authored_spotlight = explicitly_staged_spotlight.clone().or_else(|| {
            (frame.main_module == 7 && frame.submodule == 0x0f)
                .then(|| LiveSpotlightScanout::capture(self))
        });
        let retiring_pre_spotlight_scanout = retires_pre_spotlight_scanout.then(|| {
            authored_spotlight
                .as_ref()
                .expect("interrupted Spotlight_open requires its authored prefix")
                .clone()
        });
        debug_assert!(
            !retires_pre_spotlight_scanout || self.dungeon_landing_entry_started_after_leading_nmi
        );
        let spotlight_build_remains_suspended = self
            .game_execution_scheduler
            .scheduled_work_slices_remaining()
            .is_some_and(|remaining| remaining != 0);
        let spotlight_vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let measured_rows = self.active_dungeon_landing_spotlight_copy_visible_rows;
        let projects_live_tail = measured_rows.map_or_else(
            || spotlight_opening_projects_live_tail_before_hdma(
                self.game_state.display.spotlight_hdma.window_radius(), spotlight_vertical_center),
            |rows| rows.iter().any(|&visible| visible),
        );
        let spotlight_tail_after_projection = (!retires_pre_spotlight_scanout && projects_live_tail)
        .then(|| {
            authored_spotlight
                .as_ref()
                .expect("suspended iris builder requires its authored table")
                .hdma_tables
                .clone()
        });
        if let Some(mut after_projection) = spotlight_tail_after_projection {
            let live_tail_start = measured_rows.map_or_else(|| spotlight_mixed_scanout_live_tail_start(
                spotlight_vertical_center,
                self.game_state.display.spotlight_hdma.window_radius(),
            ), |_| 0);
            let active = self
                .display_snapshot
                .as_mut()
                .expect("interrupted iris copy requires an active display snapshot");
            let before_projection = active.effective_spotlight_hdma_tables();
            if let Some(copied) = measured_rows {
                after_projection = spotlight_copy_scanout_tables(&before_projection, &after_projection, &copied);
            }
            active.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection,
                    after_projection,
                    live_tail_start,
                };
        }
        let (oam_scanout_source, obj_cache_generation) = if retires_pre_spotlight_scanout {
            // The first interrupted field still owns the immutable
            // pre-NMI capture. No completed full-NMI OBJ generation
            // precedes it.
            (
                OamScanoutSource::RetainImmutableCapturedPpu,
                DisplayObjCacheGeneration::CapturedBeforeNmi,
            )
        } else {
            (
                OamScanoutSource::RetainImmutableCapturedPpu,
                DisplayObjCacheGeneration::HostBoundaryBeforeMain,
            )
        };
        let spotlight = if spotlight_build_remains_suspended {
            // The translated function has already produced its atomic result,
            // but the C instruction stream has not reached the table copy or
            // caller return yet. Preserve the result off-screen until the
            // scheduler reports that final crossing.
            self.interrupted_dungeon_spotlight_build_in_flight = authored_spotlight;
            None
        } else {
            // The C builder has returned before this NMI. Publish the table it
            // authored at this hardware boundary; host-frame adjacency is not
            // a display-generation owner and must not add another queue.
            authored_spotlight
        };
        self.interrupted_dungeon_submodule_publication =
            Some(InterruptedDungeonSubmodulePublication {
                retiring_pre_spotlight_scanout,
                oam_scanout_source,
                obj_cache_generation,
                ppu_registers: NmiPpuRegisterScanout::capture(&self.ppu),
                spotlight,
                provenance: core::panic::Location::caller(),
            });
    }

    pub(super) fn apply_interrupted_dungeon_submodule_publication(&mut self) {
        let Some(pending) = self.interrupted_dungeon_submodule_publication.take() else {
            return;
        };
        let active = self
            .display_snapshot
            .as_mut()
            .expect("interrupted dungeon submodule publication requires a display snapshot");
        // A held NMI may have completed DMA in CPU time without owning the
        // immutable field already selected for presentation. Only the active
        // snapshot's explicit receipt source can publish those OAM/OBJ writes.
        let observed_full_nmi_owns_obj_memory =
            active.oam_scanout_source == OamScanoutSource::ComposeObservedNmiReceipt;
        let receipt = active
            .effective_presented_dma
            .get_or_insert_with(|| EffectivePresentedDma {
                vram_writes: Vec::new(),
                decoded_bg_vram_writes: Vec::new(),
                completed_oam: None,
                completed_link_obj_dma: None,
                completed_cgram: None,
                completed_ppu_registers: None,
                completed_dialogue_metadata: None,
            });
        if !observed_full_nmi_owns_obj_memory {
            receipt
                .vram_writes
                .retain(|&(index, _)| !(0x4000..0x4400).contains(&index));
            receipt.completed_oam = None;
            receipt.completed_link_obj_dma = None;
        }
        receipt.completed_ppu_registers = Some(pending.ppu_registers);

        active.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
        active.hud_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
        active.cgram_scanout_generation = CgramScanoutGeneration::RetainPreviousPresented;
        active.completed_oam_dma_after_capture = None;
        active.closed_oam_boundary_receipt = None;
        active.accepts_nmi_dma_receipts = false;
        if !observed_full_nmi_owns_obj_memory {
            active.oam_scanout_source = if pending.retiring_pre_spotlight_scanout.is_some() {
                // The first interrupted landing call is still retiring the
                // pre-wipe field; its newly captured OAM belongs to the next
                // field even though the iris registers have entered state 1.
                OamScanoutSource::RetainPreviousPresented
            } else {
                pending.oam_scanout_source
            };
        }
        active.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
        active.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
        active.obj_cache_generation = pending.obj_cache_generation;
        active.obj_scanout_provenance = Some(pending.provenance);
        active.interrupted_dungeon_submodule_nmi_owns_scanout = true;

        if let Some(spotlight) = pending.retiring_pre_spotlight_scanout {
            active.spotlight_scanout_generation =
                SpotlightScanoutGeneration::ComposeLiveAfterNmi(spotlight);
            active.hdma_table_generation = DisplayHdmaTableGeneration::Captured;
        } else if let Some(spotlight) = pending.spotlight {
            active.spotlight_scanout_generation =
                SpotlightScanoutGeneration::ComposeLiveAfterNmi(spotlight);
            if !matches!(
                active.hdma_table_generation,
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout { .. }
            ) {
                active.hdma_table_generation = DisplayHdmaTableGeneration::Captured;
            }
        }
    }

    pub(super) fn compose_display_registers(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        // The ROM's iris setup authors window controls, HDMA channel state,
        // enable state, and both indirect tables as one scanout generation.
        // Never combine live controls with the captured (still-open) circle.
        following.spotlight_scanout_generation.compose_into(
            &mut self.ram,
            &mut self.ppu,
            &mut self.dma,
        );
        // A table projection can complete after that coupled iris generation
        // is staged but before HDMA consumes the next scanout. Apply this
        // scanout-local table generation last so it refines, rather than gets
        // overwritten by, the coherent controls/channels/tables baseline.
        following.hdma_table_generation.compose_into(&mut self.ram);
        plan.bg_scroll_source
            .compose_into(&mut self.ppu, &following.ppu);
        if plan.publish_live_screen_layers
            || plan.publish_live_dungeon_state_13_palette_and_registers
        {
            self.ppu.screen_enabled = following.ppu.screen_enabled;
        }
    }

    pub(super) fn stage_live_animated_bg_scanout(&mut self) {
        if let Some(snapshot) = self.display_snapshot.as_mut() {
            snapshot.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::LiveAfterNmi;
        }
        self.next_display_animated_bg_scanout_generation =
            Some(AnimatedBgScanoutGeneration::LiveAfterNmi);
    }

    pub(super) fn compose_display_raster(
        &mut self,
        live_forced_blank: bool,
        mut live_forced_blank_from_scanline: Option<u8>,
        live_retain_active_display_history: bool,
        captured_screen_brightness: u8,
        plan: &DisplayPublicationPlan,
    ) {
        let captured_frame = crate::game_state::FrameState::load_from_ram(&self.ram);
        live_forced_blank_from_scanline =
            live_forced_blank_from_scanline.or(self.ppu.forced_blank_from_scanline);
        if live_forced_blank && live_forced_blank_from_scanline.is_none() {
            live_forced_blank_from_scanline = dungeon_map_terminal_fade_blank_scanline(
                captured_frame.main_module,
                captured_frame.submodule,
                self.ram[OVERWORLD_MAP_STATE],
                self.ppu.brightness,
            );
        }
        // A force-blank write published by NMI takes effect before the next
        // active scanline even when other domains retain the pre-NMI snapshot.
        self.ppu.forced_blank |= live_forced_blank;
        if live_forced_blank {
            let scanout = resolve_active_display_blanking_scanout(
                self.ppu.retain_active_display_history,
                live_forced_blank_from_scanline,
                live_retain_active_display_history,
            );
            self.ppu.forced_blank_from_scanline = scanout.suffix_start_scanline;
            self.ppu.retain_active_display_history = scanout.retain_prior_surface;
        }
        self.ppu.scanout_brightness_override = self.ppu.scanout_brightness_override.or_else(|| {
            plan.world_map_mode7_brightness_is_early_published
                .then_some(captured_screen_brightness)
        });
    }

    /// Advances the hardware-visible display-publication history without
    /// rasterizing the composed frame.
    ///
    /// Late-window parity probes still need every discarded warm-up boundary
    /// to retire OAM, CGRAM, and OBJ-VRAM generations. They do not need to
    /// submit those frames to the GPU. Keeping this operation on the same
    /// `with_display_snapshot` path makes the optimization incapable of
    /// silently bypassing publication semantics.
    pub fn advance_display_publication_history(&mut self) {
        self.with_display_snapshot(|_| ());
    }

    /// Record the memory destinations written by a leading NMI. The immutable
    /// active snapshot was captured before the interrupt so CPU-authored state
    /// cannot leak into it, but these vblank transfers finish before visible
    /// scanout and must be composed into that same snapshot. OAM joins this
    /// receipt only when `record_completed_oam_dma_for_display_boundary`
    /// observes an actual transfer while the explicit boundary is active.
    #[track_caller]
    pub(super) fn begin_effective_presented_dma(&mut self) {
        if nmi::debug_frame_selection_env_matches("ZELDA3_DEBUG_OBJ_PIPE", self.frame_ctr_dbg) {
            eprintln!(
                "objpipe host={} stage=receipt_begin caller={}",
                self.frame_ctr_dbg,
                std::panic::Location::caller()
            );
        }
        debug_assert!(self.active_effective_dma_writes.is_none());
        let active_snapshot_accepts_receipt = self
            .display_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts);
        let active_snapshot_epoch = self
            .display_snapshot
            .as_ref()
            .filter(|snapshot| snapshot.accepts_nmi_dma_receipts)
            .map(|snapshot| snapshot.publication_epoch);
        self.active_effective_dma_writes = Some(EffectiveDmaWriteSet::new_for_snapshot(
            self.ppu.vram.len(),
            active_snapshot_accepts_receipt,
            active_snapshot_epoch,
        ));
    }

    pub(super) fn record_effective_presented_dma_for_active_scanout(
        &mut self,
    ) -> Option<DialogueTextDmaPublicationToken> {
        let writes = self.active_effective_dma_writes.take()?;
        let active_snapshot_accepts_receipt = writes.active_snapshot_accepts_receipt;
        let dialogue_text_token = self.dialogue_text_dma_publication_token(&writes);
        let mut receipt = EffectivePresentedDma::from_write_set(writes, self);
        let interrupted_scanout_is_retiring = self
            .display_snapshot
            .as_ref()
            .is_some_and(|active| active.interrupted_dungeon_submodule_nmi_owns_scanout);
        if !interrupted_scanout_is_retiring
            && self.display_snapshot.as_ref().is_some_and(|active| {
                active.animated_bg_scanout_generation
                    == AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
            })
        {
            let destination = self
                .game_state
                .display
                .animated_tile_vram_destination_usize();
            // HostBoundaryBeforeNmi says this field completed before the
            // animated-page upload. Preserve that raw/cache generation. The
            // C NMI has only one VRAM write; its resident result is captured
            // naturally by a later field and must not be reintroduced as a
            // synthetic decoded-cache publication on either stored snapshot.
            let _ = receipt.take_vram_range(destination..destination.saturating_add(0x200));
        }
        if interrupted_scanout_is_retiring {
            let publishes_obj_memory = receipt.completed_oam.is_some()
                || receipt
                    .vram_writes
                    .iter()
                    .any(|&(index, _)| (0x4000..0x4400).contains(&index));
            if !publishes_obj_memory {
                // A second held-latch NMI only repeats register writes. Those
                // were already captured by the interrupted-call receipt and
                // cannot advance display memory for this scanout.
                return None;
            }
            // A full update has now crossed the next field boundary. The
            // retiring display slot still owns the held-NMI result; route this
            // receipt to the staged snapshot which follows it.
            let staged = self
                .deferred_display_snapshot
                .as_mut()
                .expect("interrupted scanout requires a staged next snapshot");
            staged.oam_scanout_source = OamScanoutSource::ComposeObservedNmiReceipt;
            if let Some(existing) = staged.effective_presented_dma.as_mut() {
                existing.merge_after(receipt);
            } else {
                staged.effective_presented_dma = Some(receipt);
            }
            return None;
        }
        if !active_snapshot_accepts_receipt {
            return None;
        }
        let active = self
            .display_snapshot
            .as_mut()
            .expect("leading-NMI DMA receipt requires an active display snapshot");
        if let Some(existing) = active.effective_presented_dma.as_mut() {
            existing.merge_after(receipt);
        } else {
            active.effective_presented_dma = Some(receipt);
        }
        dialogue_text_token
    }

    pub(super) fn record_observed_animated_bg_dma_for_active_scanout(
        &mut self,
        destination: usize,
        data: &[u8],
    ) {
        let Some(active) = self
            .display_snapshot
            .as_mut()
            .filter(|active| active.accepts_nmi_dma_receipts)
        else {
            return;
        };
        let writes = data
            .chunks_exact(2)
            .take(0x200)
            .enumerate()
            .filter_map(|(offset, bytes)| {
                let index = destination.checked_add(offset)?;
                (index < active.ppu.vram.len())
                    .then_some((index, u16::from_le_bytes([bytes[0], bytes[1]])))
            })
            .collect::<Vec<_>>();
        if writes.is_empty() {
            return;
        }
        let receipt = EffectivePresentedDma::animated_bg_only(
            active.animated_bg_scanout_generation
                == AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
            writes,
        );
        if let Some(existing) = active.effective_presented_dma.as_mut() {
            existing.merge_after(receipt);
        } else {
            active.effective_presented_dma = Some(receipt);
        }
    }

    pub(super) fn compose_effective_presented_ppu_registers(
        &mut self,
        following: &DisplaySnapshot,
        bg_scroll_source: DisplayedBgScrollSource,
    ) {
        let Some(completed_registers) = following
            .effective_presented_dma
            .as_ref()
            .and_then(|receipt| receipt.completed_ppu_registers)
        else {
            return;
        };

        // This is the exact post-WritePpuRegisters hardware generation. Apply
        // it after snapshot-level composition so later live state cannot leak
        // across the boundary. BG scroll is independently owned when the
        // publication plan names a CPU-slice or live generation; reapply that
        // explicit subdomain after the coupled receipt instead of letting the
        // receipt silently overwrite its stronger provenance.
        completed_registers.publish_to(&mut self.ppu);
        bg_scroll_source.compose_into(&mut self.ppu, &following.ppu);
    }

    pub(super) fn record_completed_ppu_registers_for_display_boundary(&mut self) {
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            writes.completed_ppu_registers = Some(NmiPpuRegisterScanout::capture(&self.ppu));
        }
    }

    pub fn zelda_debug_display_publication_candidates(
        &self,
    ) -> &[DebugDisplayPublicationCandidate] {
        &self.debug_display_publication_candidates
    }

    pub(super) fn complete_dungeon_landing_goal_active_scanout(&mut self, reset_prefix: Vec<u16>) {
        let mut reset_tables = spotlight_hdma_tables_from_ram(&self.ram);
        // C resets the active dynamic table after authoring the final circle.
        // Both CPU buffers describe the one physical HDMA stream here.
        reset_tables[1] = reset_tables[0].clone();
        let Some(active) = self.display_snapshot.as_mut() else {
            return;
        };
        let mut before_reset = active.effective_spotlight_hdma_tables();
        for table in &mut before_reset {
            for (index, value) in reset_prefix.iter().copied().enumerate() {
                let offset = index * 2;
                table[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
            }
        }
        active.hdma_table_generation =
            DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                before_projection: before_reset,
                after_projection: reset_tables,
                live_tail_start: reset_prefix.len(),
            };
    }
}
