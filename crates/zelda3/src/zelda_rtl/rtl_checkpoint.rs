//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (checkpoint).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    /// The committed provenance-clean CGRAM mirror for the renderer (the
    /// zero-CGRAM color source; see `zelda3_palette`).
    pub fn cgram_provenance_snapshot(&self) -> zelda3_palette::CgramProvenanceSnapshot {
        self.game_state
            .display
            .palette_provenance
            .0
            .cgram_snapshot()
    }

    /// Serialize the provenance mirror for a checkpoint trailer. The mirror is
    /// `#[serde(skip)]` in the state snapshot (a restore reconstitutes it from the
    /// shadow, tagged `Copied`); a checkpoint that carries these bytes can instead
    /// restore the mirror exactly as-derived — true provenance tags and no live-CGRAM
    /// read at the boundary. See [`Self::restore_palette_mirror_from_bytes`].
    pub fn palette_mirror_snapshot_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self.game_state.display.palette_provenance.0)
            .expect("palette mirror is a fixed-size POD struct and always serializes")
    }

    /// Install a provenance mirror captured by [`Self::palette_mirror_snapshot_bytes`],
    /// overwriting whatever the snapshot restore reconstituted from the shadow.
    pub fn restore_palette_mirror_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mirror = bincode::deserialize(bytes).map_err(|e| e.to_string())?;
        self.game_state.display.palette_provenance.0 = mirror;
        Ok(())
    }

    pub(crate) fn set_msu_resume_info(&mut self, slot: MsuResumeSlot, info: MsuResumeInfoState) {
        self.system_signals_mut().set_msu_resume_info(slot, info);
    }

    pub(crate) fn set_death_backup_current_music(&mut self, value: u8) {
        self.system_signals_mut()
            .set_death_backup_current_music(value);
    }

    pub(crate) fn set_death_backup_ambient_sound(&mut self, value: u8) {
        self.system_signals_mut()
            .set_death_backup_ambient_sound(value);
    }

    pub(crate) fn restore_spexit_area_index(&mut self) {
        self.world_region_mut().restore_spexit_area_index();
    }

    pub(crate) fn restore_exit_area_index(&mut self) {
        self.world_region_mut().restore_exit_area_index();
    }

    pub(crate) fn restore_quadrant_fullsize_from_cached(&mut self) {
        self.world_transient_mut()
            .restore_quadrant_fullsize_from_cached();
    }

    /// The layer masks belong to the display; the transient only keeps the
    /// exit backups.
    pub(crate) fn restore_spexit_layer_masks(&mut self) {
        let layer_masks = self.game_state.world.transient.special_exit_layer_masks();
        self.set_layer_masks_word(layer_masks);
    }

    pub(crate) fn restore_exit_layer_masks(&mut self) {
        let layer_masks = self.game_state.world.transient.exit_layer_masks();
        self.set_layer_masks_word(layer_masks);
    }

    pub(crate) fn backup_overworld_big_area_low(&mut self) {
        self.overworld_screen_size_mut().backup_big_area_low();
    }

    pub(crate) fn restore_previous_screen_transition_direction_bits(&mut self) {
        self.overworld_transition_mut()
            .restore_previous_direction_bits();
    }

    /// A saved Sprite_Main stack resumes during active display and returns to
    /// the ordinary ZeldaRunGameLoop suffix. The capture preceding its trailing
    /// NMI still owns the resident OAM and host-boundary Link OBJ generation;
    /// the DMA performed by that trailing NMI belongs to the following field.
    pub(super) fn stage_resumed_sprite_main_return_obj_scanout(&mut self) {
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn refine_dungeon_reset_progress_before_resume(
        &mut self,
        completed: DungeonResetSpritesCpuProgress,
    ) -> DungeonResetSpritesCpuProgress {
        let Some(receipt) = self.take_original_timing_dungeon_reset_sprites_progress() else {
            return completed;
        };
        assert_eq!(
            receipt.boundary,
            OriginalTimingBoundary::NmiAccepted,
            "a queued Dungeon_ResetSprites caller may refine only at its accepting NMI",
        );
        assert!(
            self.dungeon_advance_reset_sprites_cpu_progress(completed, receipt.progress),
            "timing authority published an unsupported Dungeon_ResetSprites progress transition",
        );
        receipt.progress
    }

    pub(super) fn schedule_dungeon_faded_filter_leading_nmi_resume(&mut self) {
        debug_assert!(self.rom_startup_timing());
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        debug_assert_eq!(self.game_state.frame.submodule, 2);
        self.game_execution_scheduler.schedule_pre_main_nmi_resume(
            PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi,
        );
    }

    pub(crate) fn dungeon_star_tile_restore_source_offsets(&self) -> (usize, usize) {
        if self.game_state.dungeon.room_effects.star_tile_phase() != 0 {
            (32, 0)
        } else {
            (0, 32)
        }
    }

    pub(crate) fn backup_overworld_palette_from_tagged(
        &mut self,
        palette: &[u8],
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .backup_overworld_palette_from_tagged(palette, source);
    }

    pub(crate) fn resume_intro_triangle_motion(&mut self) {
        self.intro_scene_bridge_mut().resume_triangle_motion();
    }

    pub(crate) fn restore_select_file_remembered_cursor(&mut self) {
        self.select_file_menu_mut().restore_remembered_cursor();
    }

    pub fn restore_saveload_hdma_scratch_bytes(&mut self, bytes: &[u8]) {
        SpotlightHdmaState::restore_saveload_scratch_bytes(&mut self.ram, bytes);
    }

    pub fn restore_hdma_dynamic_table_bytes(&mut self, bytes: &[u8]) {
        self.spotlight_hdma_mut().restore_dynamic_table_bytes(bytes);
    }

    pub(crate) fn overworld_palette_backup_mut(
        &mut self,
    ) -> NativeOverworldPaletteBackupBridgeMut<'_> {
        NativeOverworldPaletteBackupBridgeMut::new(
            &mut self.game_state.display.overworld_palette_backup,
            &mut self.ram,
        )
    }

    pub(crate) fn set_overworld_main_indoors_palette_backup(&mut self, value: u8) {
        self.overworld_palette_backup_mut()
            .set_main_indoors_backup(value);
    }

    pub(crate) fn set_overworld_aux3_bg_palette_7_backup(&mut self, value: u8) {
        self.overworld_palette_backup_mut()
            .set_aux3_bg_palette_7_backup(value);
    }

    pub(crate) fn set_overworld_main_indoors_copy_palette_backup(&mut self, value: u8) {
        self.overworld_palette_backup_mut()
            .set_main_indoors_copy_backup(value);
    }

    pub fn set_small_overworld_map16_scroll_backup_state(
        &mut self,
        state: SmallOverworldMap16ScrollBackupState,
    ) {
        self.overworld_map16_mut().set_small_scroll_backup(state);
    }

    /// Restores the live-ROM frame scheduling mode after deserializing a
    /// checkpoint without resetting the already-restored audio sequencer.
    ///
    /// `rom_startup_timing` is host runtime policy rather than game state, so
    /// it is intentionally omitted from `ZeldaState`'s serialized payload.
    /// Calling `set_rom_startup_timing(true)` here would also reapply the SPC
    /// bootstrap phase and corrupt the checkpoint's exact audio position.
    pub fn restore_live_rom_timing_after_checkpoint(&mut self) {
        let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
        self.dialogue_scroll_machine_mut()
            .restore_transient_after_checkpoint(completed_scanout);
        self.rom_startup_timing = true;
        self.invalidate_original_timing_after_checkpoint();
    }

    /// Paired emulator checkpoints may only be captured between translated
    /// game calls. The execution scheduler represents a suspended 65816 call
    /// stack and is intentionally absent from ordinary playable save states.
    pub fn paired_resume_cpu_boundary_is_quiescent(&self) -> bool {
        self.game_execution_scheduler.is_idle()
            && self.active_dungeon_sprite_main_return.is_none()
            && self.active_module09_sprite_main_return.is_none()
            // These instruction-timing plans deliberately remain runtime-only:
            // ordinary Zelda save states must not persist an emulated call
            // stack. A paired parity checkpoint therefore cannot be captured
            // while one spans host calls, or restore would silently default it
            // to `None` and resume the translated continuation without its
            // source-owned NMI schedule.
            && self.dungeon_room_load_cpu_schedule.is_none()
            && self.dungeon_submodule_cpu_schedule.is_none()
            && self.module09_cpu_schedule.is_none()
            // The translated-only intro thread phase is also a suspended
            // source call stack and is serde-skipped. Capturing it would
            // restore phase zero and resume from the wrong C boundary.
            && self.intro_poly_thread_initialization_phase == 0
            && self.intro_initialization_work_frames_pending == 0
            && self.intro_memory_darken_frame_delay == 0
            && self
                .original_timing_sprite_main_return_claims_remaining
                .is_none()
            && self.interrupted_nmi_prepare_obj_cache_vram.is_none()
            && self.pending_main_loop_common_suffix.is_none()
            // A poly bitmap completed in this host's slot is uploaded by the
            // next host's NMI; it is runtime-only and must not be dropped.
            && self.poly_completed_upload.is_none()
            // The save-quit reset hold is a suspended Death_Func15 call stack
            // spanning tens of hosts; never checkpoint inside it.
            && !self.save_quit_reset_hold
    }

    /// `FinishDungeonPushBlockHandler`: finish the interrupted loop from the
    /// saved misc object index when it is still pending, then the Module 7
    /// tail that follows it.
    pub(super) fn resume_dungeon_push_block_handler(&mut self, handler_pending: bool) {
        let owns_return_claim = self
            .original_timing_sprite_main_return_claims_remaining
            .is_none()
            && self.original_timing_owes_sprite_main_return();
        if owns_return_claim {
            self.begin_original_timing_sprite_main_return_claim_scope(1);
        }
        if handler_pending {
            // The resumed host restates the interrupting boundary at the head
            // of its vector (route host 1526914); the loop cursor already
            // sits there, so the restatement carries no new work.
            let cursor = self.game_state.dungeon.object_tracking.misc_object_index();
            self.take_original_timing_dungeon_push_blocks_restatement(cursor);
            self.dungeon_push_block_handler();
            self.replay_trace_ram_watch("module07-after-push-blocks");
        } else {
            // The resumed host restates the handler's completed return at
            // the head of its vector (cold route host 924429); the handler
            // already ran on the interrupted host.
            let _ = self.take_original_timing_dungeon_push_blocks_handled();
        }
        self.complete_module07_dungeon_after_push_block_handler(true);
        if owns_return_claim {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if self.game_execution_scheduler.is_idle() {
            if self.original_timing_host_iteration_uninterrupted {
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            } else {
                self.retire_or_defer_main_loop_common_suffix_by_wire();
            }
        }
    }

    pub(super) fn resume_dungeon_push_blocks_caller(&mut self, dungeon: DungeonSpriteMainReturn) {
        let owns_return_claim = self
            .original_timing_sprite_main_return_claims_remaining
            .is_none()
            && self.original_timing_owes_sprite_main_return();
        if owns_return_claim {
            self.begin_original_timing_sprite_main_return_claim_scope(1);
        }
        self.sprite_dungeon_draw_all_push_blocks();
        self.run_module07_sprite_main_caller(dungeon);
        if owns_return_claim {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if self.game_execution_scheduler.is_idle() {
            // The outer host timeline can consume the suffix receipt before
            // this saved caller runs. Its validated completion fact survives.
            if self.original_timing_host_iteration_uninterrupted {
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            } else {
                self.retire_or_defer_main_loop_common_suffix_by_wire();
            }
        }
    }

    pub(super) fn restore_room_61_sprite_conversion_resident_oam(&mut self) {
        if self.game_state.world.location.dungeon_room_index() != 0x61
            || !matches!(self.game_state.frame.subsubmodule, 3 | 4)
        {
            return;
        }
        let Some(oam) = self
            .active_display_obj_generation
            .retained_oam()
            .map(<[u16]>::to_vec)
        else {
            return;
        };
        self.ppu.oam.clone_from_slice(&oam);
    }

    pub(super) fn capture_display_snapshot(&mut self) {
        self.capture_display_snapshot_with_override(None);
    }

    pub(super) fn capture_display_snapshot_with_override(
        &mut self,
        publication_override: Option<DisplaySnapshotPublication>,
    ) {
        // OAM-law clause: a transfer performed at the previous vblank becomes
        // visible at this scanout — unless this frame's vblank was held (the
        // frame counter did not advance): the transfer then belongs to the
        // held vblank itself and is visible only from the scanout after it.
        // (Trace-verified at the intro poly stalls, where the pinned core's
        // poly worker owns the vblank rust's split model has already passed.)
        let frame_counter_advanced = self
            .oam_law_entry_frame_counter
            .is_none_or(|entry| entry != self.game_state.frame.frame_counter);
        if let Some(pending) = self.oam_law_pending.take_if(|_| frame_counter_advanced) {
            if nmi::debug_frame_selection_env_matches(
                "ZELDA3_DEBUG_OAM_LAW_EVENTS",
                self.frame_ctr_dbg,
            ) {
                eprintln!(
                    "oam_law_promote host={} w204={:04x}",
                    self.frame_ctr_dbg, pending[204]
                );
            }
            self.oam_law_visible = Some(pending);
        }
        // The native frame identifies the CPU publication phase. It can
        // deliberately lag a direct WRAM module handoff until NMI resumes.
        let frame = self.game_state.frame;
        let publication = publication_override.unwrap_or_else(|| {
            if rom_attract_world_map_display_is_one_frame_deferred(
                frame.main_module,
                frame.submodule,
                self.game_state.ending.attract_scene.sequence(),
                self.game_state.ending.attract_scene.state(),
            ) {
                DisplaySnapshotPublication::AdvanceStaged
            } else {
                rom_display_snapshot_publication(frame.main_module, frame.submodule)
            }
        });
        let spotlight_iteration = self.game_execution_scheduler.spotlight_iteration();
        let spotlight_vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let spotlight_live_tail_start = spotlight_mixed_scanout_live_tail_start(
            spotlight_vertical_center,
            self.game_state.display.spotlight_hdma.window_radius(),
        );
        let spotlight_projection_waits_for_staged_publication = match self
            .game_execution_scheduler
            .current_work()
        {
            Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild { iteration, .. })
            | Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration })
            | Some(GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity {
                iteration,
                ..
            })
            | Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement {
                iteration,
                ..
            })
            | Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
                iteration,
                ..
            })
            | Some(
                GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
                    iteration,
                    ..
                },
            )
            | Some(
                GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                    iteration,
                    ..
                },
            )
            | Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity {
                iteration,
                ..
            }) => iteration.phase == SpotlightIterationPhase::WholeTableAfterTablePublication,
            Some(GameWorkContinuation::FinishSpotlightIteration { iteration }) => {
                // On the short Module 15 close, the C loop authors the next
                // table during active display. The current scanout keeps the
                // previously staged circle; the new table becomes visible at
                // the caller-return boundary on the following host.
                self.game_state.frame.main_module == 15
                    && !spotlight_table_has_long_nmi_workload(spotlight_vertical_center)
                    && iteration.phase == SpotlightIterationPhase::WholeTableAfterTablePublication
            }
            Some(GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. }) => {
                // The goal build returns near the end of active display. Its
                // reserved-table memcpy has not retroactively replaced rows
                // HDMA already consumed from the preceding circle.
                true
            }
            _ => false,
        };
        let spotlight_table_is_still_building = spotlight_iteration.is_some_and(|iteration| {
            iteration.is_closing()
                && self.game_state.frame.main_module == 15
                && rom_dungeon_exit_spotlight_table_needs_entry_slice(
                    self.game_state.display.spotlight_hdma.window_radius(),
                    spotlight_vertical_center,
                )
        });
        let hdma_table_published_ahead = spotlight_iteration
            .filter(|iteration| {
                iteration.publishes_completed_hdma_table_to_active_scanout()
                    && !spotlight_table_is_still_building
                    && !spotlight_projection_waits_for_staged_publication
            })
            .map(|_| self.hdma_dynamic_table_bytes());
        let spotlight_table_before_copy = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                projection_completed: false,
                ..
            })
        )
        .then(|| {
            // $F383/$F392 write the working table, not the hardware table.
            // Until $F3B7 copies it, HDMA consumes the preceding completed
            // table. The C translation retains that copy in the reserved
            // buffer; its dynamic buffer may already hold partial new rows.
            let mut table = self.hdma_dynamic_table_bytes();
            let visible_bytes = SPOTLIGHT_VISIBLE_SCANLINES * 2;
            table[..visible_bytes].copy_from_slice(
                &self.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + visible_bytes],
            );
            table
        });
        let mixed_spotlight_after_projection = spotlight_iteration
            .filter(|iteration| iteration.phase == SpotlightIterationPhase::MixedTailAfterReturn)
            .map(|_| spotlight_hdma_tables_from_ram(&self.ram));
        let dungeon_exit_goal_after_projection = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. })
        )
        .then(|| spotlight_hdma_tables_from_ram(&self.ram));
        if self.next_display_spotlight_scanout.is_none()
            && self.game_execution_scheduler.current_work()
                == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)
            && self
                .game_execution_scheduler
                .scheduled_work_slices_remaining()
                .is_some_and(|remaining| remaining != 0)
        {
            // The atomic translation has authored the next table, but the C
            // instruction stream is still inside that builder. Snapshot the
            // last table which actually returned rather than future WRAM.
            self.next_display_spotlight_scanout = self
                .last_completed_interrupted_dungeon_spotlight_scanout
                .clone();
        }
        let trailing_force_blank_scanout_brightness = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. })
        )
        .then(|| {
            self.display_snapshot
                .as_deref()
                .filter(|display| display.ppu.scanout_brightness_override.is_none())
                .map(|display| display.ppu.scanout_brightness())
        })
        .flatten();
        self.capture_display_snapshot_with_publication(publication);
        if let Some(active_table) = spotlight_table_before_copy.or(hdma_table_published_ahead) {
            self.publish_completed_spotlight_hdma_table_to_active_scanout(active_table);
        }
        if let (Some(after_projection), Some(display)) = (
            mixed_spotlight_after_projection,
            self.display_snapshot.as_mut(),
        ) {
            let before_projection = display.effective_spotlight_hdma_tables();
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection,
                    after_projection,
                    live_tail_start: spotlight_live_tail_start,
                };
        }
        if let (Some(after_projection), Some(display)) = (
            dungeon_exit_goal_after_projection,
            self.display_snapshot.as_mut(),
        ) {
            let before_projection = display.effective_spotlight_hdma_tables();
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection,
                    after_projection,
                    live_tail_start: spotlight_live_tail_start,
                };
        }
        if let Some(scanout_brightness) = trailing_force_blank_scanout_brightness {
            // IrisSpotlight_ConfigureTable's goal write is consumed by the
            // trailing NMI after the active field. Carry the resulting PPU
            // register generation even though the visible rows keep the iris
            // table that was scanned before that NMI.
            if let Some(display) = self.display_snapshot.as_mut() {
                display.ppu.forced_blank_from_scanline = Some(TRAILING_NMI_FORCE_BLANK_SCANLINE);
                display.ppu.scanout_brightness_override = Some(scanout_brightness);
            }
        }
    }

    /// The caller has established that the queued spotlight rows describe
    /// this field, independently of the staged OAM/VRAM publication.
    pub(super) fn capture_display_snapshot_with_current_spotlight(
        &mut self, publication: Option<DisplaySnapshotPublication>,
    ) {
        let scanout = self.next_display_spotlight_scanout.take()
            .expect("current spotlight capture requires its measured rows");
        assert!(scanout.authoritative_rom_hdma_receipt);
        self.capture_display_snapshot_with_override(publication);
        let display = self.display_snapshot.as_mut().expect("captured current display");
        display.spotlight_scanout_generation = SpotlightScanoutGeneration::ComposeLiveAfterNmi(scanout);
        display.hdma_table_generation = DisplayHdmaTableGeneration::Captured;
        self.retain_spotlight_entry_graphics_before_trailing_nmi();
    }

    pub(super) fn retain_spotlight_entry_graphics_before_trailing_nmi(&mut self) {
        let display = self.display_snapshot.as_mut().expect("captured entry display");
        // The entry caller returns after the held handler, then accepts the
        // next open NMI. Its animated-page and Link uploads belong to the
        // following field, even though HDMA consumed this field's rows.
        // Snes9x's $00:f3b7 entry return retains resident VRAM, including
        // the animated page at word address $3c00 and Link's $4000 page.
        display.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
        display.host_boundary_animated_bg_scanout = self.pre_nmi_animated_bg_scanout.clone();
        display.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
        display.oam_scanout_source = OamScanoutSource::RetainPreviousPresented;
        display.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
        display.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }

    pub(super) fn capture_display_snapshot_with_publication(
        &mut self,
        publication: DisplaySnapshotPublication,
    ) {
        let same_epoch_closed_oam_receipt = self
            .display_snapshot
            .as_ref()
            .and_then(|display| display.closed_oam_boundary_receipt.as_ref())
            .filter(|receipt| receipt.publication_host_frame == self.frame_ctr_dbg)
            .cloned();
        if publication != DisplaySnapshotPublication::RetainPublished {
            self.commit_retiring_display_window_latches();
        }
        let diagnostics = CaptureDisplayDiagnostics::from_env();
        self.ppu.refresh_brightness_cache();
        // The upcoming NMI may latch a fresh pre-upload CGRAM image; the one
        // from the previous frame has been consumed by that frame's renders.
        self.cgram_upload_latch = None;
        // Advance the coherent CPU/scanout pair once per display boundary. A
        // staged completion becomes visible for exactly one scanout, while an
        // in-flight copy retains its frozen hardware generation.
        self.advance_dialogue_scroll_display_boundary();
        let frame = self.game_state.frame;
        if diagnostics.attract_timeline && (5640..=5700).contains(&self.frame_ctr_dbg) {
            let trace = format!(
                "attract_display_capture host={} state={} seq={} zoom={:02x} timer={} brightness={} c0={:04x} c1={:04x} fixed={:02x},{:02x},{:02x} math={:02x}/{:02x} table0={:04x} ppu_a={:04x}",
                self.frame_ctr_dbg,
                self.game_state.ending.attract_scene.state(),
                self.game_state.ending.attract_scene.sequence(),
                self.mode7_zoom_timer(),
                self.game_state.ending.attract_scene.scene_timer(),
                self.game_state.display.screen_brightness,
                self.ppu.cgram[0],
                self.ppu.cgram[1],
                self.ppu.fixed_color_r,
                self.ppu.fixed_color_g,
                self.ppu.fixed_color_b,
                self.ppu.math_enabled,
                self.ppu.prevent_math_mode,
                self.spotlight_hdma_table_dynamic_entry(0),
                self.ppu.m7_matrix[0] as u16,
            );
            eprintln!("{trace}");
            append_parity_trace("attract-display-timeline.trace", &trace);
        }
        if diagnostics.frame_boundary {
            eprintln!(
                "frame_boundary_before host={} main={:02x} sub={:02x} frame_counter={:02x} publication={:?} landing_handoff={:?} work={:?} caller={:?} dialogue_init={} dialogue_scroll={:?} next_obj={:?} bg1=({:04x},{:04x}) link_dma_countdown={:04x} latch={} pending={} target={:04x} disable={:02x} dialogue_runs=authored:{}/published:{}/display:{}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                frame.frame_counter,
                publication,
                self.dungeon_landing_goal_display_handoff,
                self.game_execution_scheduler.current_work(),
                self.game_execution_scheduler.pre_main_caller_continuation(),
                matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishDialogueInitializationPrefix { .. }
                            | GameWorkContinuation::FinishDialogueInitializationCallerReturn
                    )
                ),
                self.dialogue_scroll_phase(),
                self.next_display_obj_scanout_generation,
                self.ppu.bg_layer[0].h_scroll,
                self.ppu.bg_layer[0].v_scroll,
                read_le_u16(&self.ram, LINK_DMA_COUNTDOWN),
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
                self.bg3_vwf_glyph_runs.len(),
                self.published_bg3_vwf_glyph_runs.len(),
                self.display_snapshot
                    .as_ref()
                    .map_or(0, |snapshot| snapshot.published_bg3_vwf_glyph_runs.len()),
            );
        }
        // Unlike the CPU phase above, transition identity belongs to the WRAM
        // generation copied into this snapshot. Keep both meanings explicit:
        // collapsing them regresses the later staged landing-wipe cadence.
        let captured_frame = crate::game_state::FrameState::load_from_ram(&self.ram);
        let captured_messaging = crate::game_state::MessagingState::load_from_ram(&self.ram);
        let published_frame = self
            .display_snapshot
            .as_ref()
            .map(|published| crate::game_state::FrameState::load_from_ram(&published.ram));
        let leading_nmi_precedes_captured_scanout = self
            .game_execution_scheduler
            .main_return_requires_leading_nmi();
        let transition_entry_obj = self.display_snapshot.as_ref().and_then(|published| {
            let published_frame = published_frame?;
            rom_dungeon_falling_entry_retains_published_obj_generation(
                published_frame.main_module,
                published_frame.submodule,
                captured_frame.main_module,
                captured_frame.submodule,
            )
            .then(|| {
                (
                    published.ppu.oam.clone(),
                    published.ppu.vram[0x4000..0x4400].to_vec(),
                )
            })
        });
        let queued_obj_generation = self.next_display_obj_memory_generation.take();
        let obj_generation_source = if queued_obj_generation.is_some() {
            "queued"
        } else if transition_entry_obj.is_some() {
            "transition-entry"
        } else {
            "active"
        };
        let obj_generation = queued_obj_generation
            .or_else(|| {
                transition_entry_obj
                    .map(|(oam, vram)| DisplayObjGeneration::RetainCapturedMemory { oam, vram })
            })
            .unwrap_or_else(|| self.active_display_obj_generation.clone());
        if nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DISPLAY_OAM_FRAME",
            self.frame_ctr_dbg,
        ) {
            eprintln!(
                "display_obj_generation host={} phase={:02x}/{:02x}/{:02x} source={} selected={} active={} work={:?} caller={:?}",
                self.frame_ctr_dbg,
                captured_frame.main_module,
                captured_frame.submodule,
                captured_frame.subsubmodule,
                obj_generation_source,
                obj_generation.name(),
                self.active_display_obj_generation.name(),
                self.game_execution_scheduler.current_work(),
                self.game_execution_scheduler.pre_main_caller_continuation(),
            );
        }
        let interrupted_item_receipt_obj_cache =
            std::mem::take(&mut self.next_display_interrupted_item_receipt_obj_cache);
        let enemy_drop_item_graphics_live_extended_oam =
            self.enemy_drop_item_graphics_live_extended_oam_pending;
        let entry_graphics_dma_plan = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_plan)
            .unwrap_or_else(|| {
                rom_graphics_dma_plan(captured_frame.main_module, captured_frame.submodule)
            });
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or(captured_frame);
        let module_oam_scanout_source = oam_scanout_across_main(
            entry_frame,
            captured_frame,
            entry_graphics_dma_plan.oam_scanout,
        );
        let current_work = self.game_execution_scheduler.current_work();
        let obj_scanout_generation = self.next_display_obj_scanout_generation.take();
        let obj_scanout_provenance = self.next_display_obj_scanout_provenance.take();
        let link_obj_scanout_generation = obj_scanout_generation
            .map(|generation| generation.link_obj)
            .unwrap_or_else(|| {
                link_obj_scanout_across_main(
                    entry_frame,
                    captured_frame,
                    entry_graphics_dma_plan.link_obj_scanout,
                )
            });
        let link_obj_source_generation = obj_scanout_generation
            .map(|generation| generation.link_obj_sources)
            .unwrap_or_else(|| {
                link_obj_scanout_across_main(
                    entry_frame,
                    captured_frame,
                    entry_graphics_dma_plan.link_obj_scanout,
                )
            });
        let oam_dma_byte_len = self.ppu.oam.len() * 2;
        let dialogue_holds_published_oam = published_frame.is_some_and(|published| {
            dialogue_text_frame_holds_published_oam(
                published,
                captured_frame,
                captured_messaging.runtime.text_render_state(),
            )
        });
        let previously_published_shadow_oam_dma =
            self.display_snapshot.as_ref().and_then(|published| {
                dialogue_holds_published_oam
                    .then(|| published.published_shadow_oam_dma.clone())
                    .flatten()
                    .or_else(|| {
                        published
                            .ram
                            .get(OAM_BUF..OAM_BUF + oam_dma_byte_len)
                            .map(|bytes| {
                                bytes
                                    .chunks_exact(2)
                                    .map(|word| u16::from_le_bytes([word[0], word[1]]))
                                    .collect::<Vec<_>>()
                            })
                    })
            });
        let host_boundary_shadow_oam_dma = self
            .pre_main_graphics_dma
            .as_ref()
            .and_then(|graphics| graphics.oam_shadow.get(..oam_dma_byte_len))
            .map(|bytes| {
                bytes
                    .chunks_exact(2)
                    .map(|word| u16::from_le_bytes([word[0], word[1]]))
                    .collect::<Vec<_>>()
            });
        let dungeon_item_hold_entry_scanout = is_dungeon_item_hold_entry(
            entry_frame,
            captured_frame,
            self.pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.entry_link_handler_state)
                .unwrap_or_else(|| self.game_state.player.follower_link.handler_state()),
            self.game_state.player.follower_link.handler_state(),
        );
        let item_hold_entry_oam_scanout_source = oam_scanout_for_dungeon_item_hold_entry(
            module_oam_scanout_source,
            dungeon_item_hold_entry_scanout,
        );
        let game_over_spotlight_build_uses_live_oam =
            game_over_spotlight_build_uses_live_oam(current_work)
                || game_over_spotlight_build_entry_uses_live_oam(entry_frame, captured_frame)
                || game_over_spotlight_return_boundary_uses_live_oam(
                    captured_frame,
                    self.game_state.display.spotlight_hdma.window_radius(),
                    current_work,
                );
        let game_over_iris_goal_scanout_closed =
            std::mem::take(&mut self.game_over_iris_goal_scanout_closed_pending)
                || game_over_iris_goal_scanout_is_closed(entry_frame, captured_frame);
        let oam_scanout_source = if game_over_iris_goal_scanout_closed {
            OamScanoutSource::RetainResidentPpuOam
        } else if game_over_spotlight_build_uses_live_oam {
            // The build is interrupted after the preceding caller-return NMI
            // has completed the current OAM DMA. Publish the captured table
            // directly; module 12's ordinary retained path would replace it
            // with the older `published_shadow_oam_dma` generation.
            OamScanoutSource::ComposeLiveAfterNmi
        } else {
            obj_scanout_generation
                .map(|generation| generation.oam)
                .unwrap_or(item_hold_entry_oam_scanout_source)
        };
        let published_shadow_oam_dma = if oam_scanout_source
            == OamScanoutSource::ComposePublishedShadowDma
            || oam_scanout_source == OamScanoutSource::ComposeHostBoundaryShadowDma
            || oam_scanout_source == OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink
            || pre_main_caller_uses_host_boundary_shadow_oam(
                self.game_execution_scheduler.pre_main_caller_continuation(),
                self.game_state.display.palette_filter.countdown(),
            ) {
            host_boundary_shadow_oam_dma
        } else {
            previously_published_shadow_oam_dma
        };
        let bg_scroll_generation = std::mem::take(&mut self.next_display_bg_scroll_generation);
        let suspended_spiral_animated_bg_dma_crosses_scanout = entry_frame.frame_counter
            == captured_frame.frame_counter
            && captured_frame.main_module == 7
            && captured_frame.submodule == 0x0e
            && self.game_state.display.bg_tile_animation_countdown == 1;
        let spiral_retains_previous_cgram = rom_spiral_palette_slice_retains_previous_cgram(
            entry_frame,
            captured_frame,
            self.game_state.world.location.dungeon_room_index(),
            self.game_state.dungeon.stair_movement.staircase_index(),
            self.game_state.display.palette_filter.countdown_word(),
        ) || (entry_frame.frame_counter
            == captured_frame.frame_counter
            && matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                        | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
                })
            ));
        if crate::debug_env::var_os("ZELDA3_DEBUG_SPIRAL_CGRAM").is_some()
            && captured_frame.main_module == 7
            && captured_frame.submodule == 0x0e
        {
            eprintln!(
                "spiral_cgram host={} entry={:02x}/{:02x}/{:02x}/fc{:02x} captured={:02x}/{:02x}/{:02x}/fc{:02x} room={:04x} stair={:02x} palette={:02x} retain={}",
                self.frame_ctr_dbg,
                entry_frame.main_module,
                entry_frame.submodule,
                entry_frame.subsubmodule,
                entry_frame.frame_counter,
                captured_frame.main_module,
                captured_frame.submodule,
                captured_frame.subsubmodule,
                captured_frame.frame_counter,
                self.game_state.world.location.dungeon_room_index(),
                self.game_state.dungeon.stair_movement.staircase_index(),
                self.game_state.display.palette_filter.countdown_word(),
                spiral_retains_previous_cgram,
            );
        }
        let hud_upload_pending = self.game_state.system_signals.should_update_hud();
        let spiral_state_8_publishes_live_hud_tilemap =
            rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
                captured_frame,
                self.game_state.world.location.dungeon_room_index(),
                hud_upload_pending,
            );
        let hud_vram_destination = self
            .game_state
            .display
            .message_dma_destination_address_usize();
        let mut resident_ppu = self.ppu.clone();
        if let Some(resident_oam) = self
            .resident_oam_dma
            .as_deref()
            .filter(|oam| oam.len() == resident_ppu.oam.len())
        {
            resident_ppu.oam.clone_from_slice(resident_oam);
        }
        if self.debug_obj_pipe_enabled() {
            self.debug_obj_pipe(
                &format!(
                    "capture pub={publication:?} latch={} slots(disp={},def={})",
                    self.ppu.obj_vram_latch.is_some(),
                    self.display_snapshot.is_some(),
                    self.deferred_display_snapshot.is_some(),
                ),
                &self.ppu.vram[0x4000..0x4400],
            );
        }
        self.display_snapshot_epoch = self
            .display_snapshot_epoch
            .checked_add(1)
            .expect("display snapshot epoch overflow");
        let mut snapshot = Box::new(DisplaySnapshot {
            publication_epoch: self.display_snapshot_epoch,
            ram: self.ram.clone(),
            ppu: resident_ppu,
            dma: self.dma.clone(),
            vram_chr_source: self.vram_chr_source.clone(),
            vram_chr_preview_source: self.vram_chr_preview_source.clone(),
            hdma_table_generation: self
                .attract_map_hdma_projection_before
                .take()
                .map(|before_projection| {
                    DisplayHdmaTableGeneration::AttractMapProjectionDuringScanout {
                        before_projection,
                    }
                })
                .unwrap_or_default(),
            vram_generation: std::mem::take(&mut self.next_display_vram_generation),
            hud_vram_generation: (if std::mem::take(&mut self.publish_live_hud_vram_on_next_capture)
                || !hud_upload_pending
                || spiral_state_8_publishes_live_hud_tilemap
            {
                DisplayVramGeneration::ComposeLiveAfterNmi
            } else {
                DisplayVramGeneration::RetainCapturedBeforeNmi
            }),
            hud_vram_destination,
            cgram_scanout_generation: if spiral_retains_previous_cgram {
                CgramScanoutGeneration::RetainPreviousPresented
            } else {
                CgramScanoutGeneration::default()
            },
            cgram_scanout_override: self.next_display_cgram_override.take(),
            game_over_iris_goal_scanout_closed,
            link_obj_scanout_generation,
            link_obj_source_generation,
            oam_scanout_source,
            completed_oam_dma_after_capture: None,
            presented_oam_override: None,
            presented_hud_tilemap_override: None,
            presented_dialogue_text_override: None,
            presented_bg_scroll_override: None,
            presented_mode7_transform_override: None,
            presented_window_mask_override: None,
            presented_inidisp_override: None,
            presented_scanout_geometry_override: None,
            presented_animated_bg_tiles_override: None,
            presented_obj_tiles_override: None,
            closed_oam_boundary_receipt: same_epoch_closed_oam_receipt,
            effective_presented_dma: None,
            interrupted_dungeon_submodule_nmi_owns_scanout: false,
            accepts_nmi_dma_receipts: true,
            publication_host_frame: self.frame_ctr_dbg,
            dungeon_item_hold_entry_scanout,
            dungeon_item_hold_entry_bg2_scroll: dungeon_item_hold_entry_scanout
                .then_some((self.ppu.bg_layer[1].h_scroll, self.ppu.bg_layer[1].v_scroll)),
            published_shadow_oam_dma,
            obj_scanout_provenance,
            animated_bg_scanout_generation: self
                .next_display_animated_bg_scanout_generation
                .take()
                .unwrap_or_else(|| {
                    if suspended_spiral_animated_bg_dma_crosses_scanout {
                        // The projected upload completes at this vblank, after
                        // the already-captured scanout generation. Keep the
                        // resident animated tiles for this image; the live PPU
                        // carries the completed DMA into the following image.
                        return AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
                    }
                    // Main can select a new module, but cannot undo the
                    // animated-page DMA from its own leading NMI. Likewise,
                    // entering a leading-NMI phase cannot publish a future
                    // upload into the field that just completed. The entry
                    // phase owns this scanout; the exit owns the next upload.
                    let generation = entry_graphics_dma_plan.animated_bg_scanout;
                    if leading_nmi_precedes_captured_scanout
                        || rom_dungeon_item_hold_to_dialogue_publishes_live_animated_bg(
                            entry_frame,
                            captured_frame,
                        )
                        || rom_dungeon_subtile_return_publishes_live_animated_bg(
                            entry_frame,
                            captured_frame,
                        )
                        || rom_dungeon_subtile_direction_one_publishes_live_animated_bg(
                            entry_frame,
                            captured_frame,
                            self.screen_transition(),
                        )
                    {
                        AnimatedBgScanoutGeneration::LiveAfterNmi
                    } else {
                        generation
                    }
                }),
            host_boundary_animated_bg_scanout: self.pre_nmi_animated_bg_scanout.clone(),
            bg_scroll_generation,
            spotlight_scanout_generation: self
                .next_display_spotlight_scanout
                .take()
                .map(SpotlightScanoutGeneration::ComposeLiveAfterNmi)
                .unwrap_or(SpotlightScanoutGeneration::CapturedBeforeNmi),
            obj_generation,
            obj_cache_generation: DisplayObjCacheGeneration::FollowModuleCadence,
            explicit_obj_cache_vram: self.next_display_obj_cache_vram.take(),
            interrupted_item_receipt_obj_cache,
            enemy_drop_item_graphics_live_extended_oam,
            published_bg3_vwf_glyph_runs: self.published_bg3_vwf_glyph_runs.clone(),
            published_bg3_vwf_glyph_run_dialogue_offsets: self
                .published_bg3_vwf_glyph_run_dialogue_offsets
                .clone(),
            published_dialogue_msg_read_pos: self.published_dialogue_msg_read_pos,
            published_dialogue_message_id: self.published_dialogue_message_id,
            room_82_sprite_conversion_deferred_nmi: false,
        });
        if let Some(explicit_obj_cache_vram) = snapshot.explicit_obj_cache_vram.as_ref() {
            snapshot.ppu.obj_vram_latch = Some(explicit_obj_cache_vram.clone());
        }
        let nmi_forced_blank_scanlines =
            std::mem::take(&mut self.nmi_forced_blank_scanlines_pending);
        snapshot.ppu.forced_blank_scanlines = nmi_forced_blank_scanlines;
        // A mid-field INIDISP write is part of the field being scanned now.
        // The persistent `forced_blank` latch owns the following field from
        // row zero; carrying the suffix metadata forward would replay the same
        // raster edge twice. Consume legacy snapshot state without publishing it.
        let _ = self.legacy_nmi_forced_blank_from_scanline_pending.take();
        snapshot.ppu.forced_blank_from_scanline = None;
        snapshot.ppu.retain_active_display_history = false;
        if frame.main_module == 0
            && frame.submodule == 1
            && self.intro_initialization_reset_obj_control_pending
        {
            // OBSEL is still at its reset value for the interrupted initial
            // display slice. Keep this in the immutable control snapshot; the
            // live native PPU remains configured for the normal Zelda OBJ CHR
            // base used once initialization resumes.
            snapshot.ppu.obj_tile_adr1 = 0;
            // With reset OBSEL, tile-name bit 8 still advances by one 256-tile
            // page. Snes9x keeps OBJNameBase/OBJNameSelect at zero and adds
            // the implicit tile-$100 byte offset; in word-addressed native
            // state that second page begins at $1000.
            snapshot.ppu.obj_tile_adr2 = 0x1000;
        }
        // A high-bit V-counter request is a one-frame raster event. Publish it
        // in this immutable display snapshot, then advance live simulation to
        // the consumed state. Rendering must not decide whether game state
        // advances, and replaying the same snapshot must reproduce the same
        // scanlines.
        if self.game_state.display.irq_control_has_vcounter_marker() {
            self.clear_irq_control_flag();
        }
        if rom_intro_poly_thread_is_active(frame.main_module, frame.submodule) {
            self.intro_poly_vram_history.push((
                frame.frame_counter,
                self.ppu.vram[0x5800..0x5c00].to_vec(),
                self.ppu.oam.to_vec(),
            ));
            if self.intro_poly_vram_history.len() > 16 {
                self.intro_poly_vram_history.remove(0);
            }
        } else {
            self.intro_poly_vram_history.clear();
        }
        match publication {
            DisplaySnapshotPublication::AdvanceStaged => {
                let mut previous = self.deferred_display_snapshot.replace(snapshot);
                if let Some(staged) = previous.as_mut() {
                    staged.oam_scanout_source = oam_scanout_source_for_staged_promotion(
                        entry_frame.frame_counter != captured_frame.frame_counter,
                        staged.oam_scanout_source,
                    );
                }
                self.display_snapshot = previous.or_else(|| self.deferred_display_snapshot.clone());
            }
            DisplaySnapshotPublication::PublishCaptured => {
                self.deferred_display_snapshot = None;
                self.display_snapshot = Some(snapshot);
            }
            DisplaySnapshotPublication::RetainPublished => {
                if self.display_snapshot.is_none() {
                    self.display_snapshot =
                        self.deferred_display_snapshot.clone().or(Some(snapshot));
                } else if let Some(published) = self.display_snapshot.as_mut() {
                    // Retaining OAM/VRAM does not retain the preceding
                    // field's HDMA reads. A measured active-field receipt
                    // belongs to this boundary even when the caller has not
                    // yet published its next complete display snapshot.
                    if matches!(
                        &snapshot.spotlight_scanout_generation,
                        SpotlightScanoutGeneration::ComposeLiveAfterNmi(scanout)
                            if scanout.authoritative_rom_hdma_receipt
                    ) {
                        published.spotlight_scanout_generation =
                            snapshot.spotlight_scanout_generation;
                        published.hdma_table_generation = DisplayHdmaTableGeneration::Captured;
                    }
                    // The entry snapshot owns one scanout with the pre-pickup
                    // camera. Item graphics then retain that same OAM/VRAM
                    // generation across subsequent vblanks, but BG2 scroll is
                    // independently republished from the live camera starting
                    // with the first retained boundary.
                    published.dungeon_item_hold_entry_scanout = false;
                }
            }
        }
        if let Some(following_spotlight) = self.spotlight_scanout_after_active_field.take() {
            debug_assert!(self.next_display_spotlight_scanout.is_none());
            self.next_display_spotlight_scanout = Some(following_spotlight);
        }
        if self.debug_obj_pipe_enabled() {
            if let Some(published) = self.display_snapshot.as_ref() {
                let prior_publication_host = published.publication_host_frame;
                let page = published.ppu.vram[0x4000..0x4400].to_vec();
                self.debug_obj_pipe(
                    &format!(
                        "published prior_host={prior_publication_host} latch_in_snapshot={}",
                        published.ppu.obj_vram_latch.is_some()
                    ),
                    &page,
                );
            }
        }
        if let Some(published) = self.display_snapshot.as_mut() {
            if published
                .closed_oam_boundary_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.publication_host_frame != self.frame_ctr_dbg)
            {
                published.closed_oam_boundary_receipt = None;
            }
            published.publication_host_frame = self.frame_ctr_dbg;
            if publication != DisplaySnapshotPublication::RetainPublished {
                published.completed_oam_dma_after_capture = None;
            }
            // `RetainPublished` carries an already-closed hardware generation
            // across another host boundary. The following NMI advances the
            // resident PPU for future scanout, but cannot retroactively attach
            // its DMA receipts to the retained image. Reopening this window
            // made interrupted Module 7 callers overwrite the exact previous
            // OAM generation that the retention operation was meant to keep.
            published.accepts_nmi_dma_receipts =
                publication != DisplaySnapshotPublication::RetainPublished;
        }
        self.apply_interrupted_dungeon_submodule_publication();
    }

    /// Runs a renderer capture against the coherent pre-NMI display state while
    /// leaving the live post-NMI simulation untouched.
    ///
    /// This is the shared publication boundary for both the scanline renderer
    /// and the modern asset/GPU renderer. The returned value must own anything
    /// it borrows from `game`, because live state is restored before returning.
    pub fn with_display_snapshot<R>(&mut self, capture: impl FnOnce(&mut ZeldaState) -> R) -> R {
        if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
            let live: u64 = self.ppu.vram[0x5800..0x5c00]
                .iter()
                .map(|&w| u64::from(w))
                .sum();
            let snap: Option<u64> = self.display_snapshot.as_ref().map(|d| {
                d.ppu.vram[0x5800..0x5c00]
                    .iter()
                    .map(|&w| u64::from(w))
                    .sum()
            });
            eprintln!(
                "[POLY-SCANOUT] host={} live5800_sum={live:08x} snapshot5800_sum={snap:08x?} snapshot_pub_host={:?}",
                self.frame_ctr_dbg,
                self.display_snapshot.as_ref().map(|d| d.publication_host_frame)
            );
        }
        let Some(mut display) = self.display_snapshot.take() else {
            return capture(self);
        };
        let saved_presented_bg_scroll = self.active_presented_bg_scroll.take();
        self.active_presented_bg_scroll = display.presented_bg_scroll_override.clone();
        let saved_presented_mode7_transform = self.active_presented_mode7_transform.take();
        self.active_presented_mode7_transform = display.presented_mode7_transform_override.clone();
        let saved_presented_window_mask = self.active_presented_window_mask.take();
        self.active_presented_window_mask = display.presented_window_mask_override.clone();
        if self.debug_obj_pipe_enabled() {
            self.debug_obj_pipe(
                &format!(
                    "consume slot={} snapshot_pub_host={} latch_in_snapshot={}",
                    "display",
                    display.publication_host_frame,
                    display.ppu.obj_vram_latch.is_some(),
                ),
                &display.ppu.vram[0x4000..0x4400],
            );
        }
        if self.presented_history_host_frame != Some(self.frame_ctr_dbg) {
            if self.presented_history_host_frame.is_some() {
                if let Some(oam) = self.staged_presented_oam.take() {
                    self.last_presented_oam = Some(oam);
                }
                if let Some(cgram) = self.staged_presented_cgram.take() {
                    self.last_presented_cgram = Some(cgram);
                }
                if let Some(vram) = self.staged_presented_obj_vram.take() {
                    self.debug_obj_pipe("promote", &vram);
                    self.last_presented_obj_vram = Some(vram);
                }
                if let Some(source) = self.staged_presented_vram_chr_source.take() {
                    self.last_presented_vram_chr_source = Some(source);
                }
                if let Some(source) = self.staged_presented_vram_chr_preview_source.take() {
                    self.last_presented_vram_chr_preview_source = Some(source);
                }
            }
            self.presented_history_host_frame = Some(self.frame_ctr_dbg);
        }
        // Capture must be side-effect-free on live game state. The native
        // game_state is re-derived from the snapshot RAM below so capture
        // closures see coherent native views; restore the ORIGINAL native
        // state afterwards instead of re-deriving it from live RAM — a
        // RAM-derived rebuild rewinds every native field whose RAM projection
        // is stale mid-frame (write-through fields, animation countdowns),
        // which made per-frame video capture perturb game behavior.
        let saved_game_state = self.game_state.clone();
        // Compose mutates the snapshot side (VRAM/OAM/CGRAM composition,
        // latch clears). Store back the PRISTINE snapshot so repeated captures
        // and later consumers see exactly what NMI published.
        let pristine_snapshot = display.clone();
        let diagnostics = DisplayDiagnostics::from_env();

        let snapshot_frame = crate::game_state::FrameState::load_from_ram(&display.ram);
        let snapshot_attract_scene =
            crate::game_state::AttractSceneState::load_from_ram(&display.ram);
        let snapshot_messaging = crate::game_state::MessagingState::load_from_ram(&display.ram);
        let snapshot_world_location =
            crate::game_state::WorldLocationState::load_from_ram(&display.ram);
        let snapshot_dungeon = crate::game_state::DungeonState::load_from_ram(&display.ram);
        let pending_main_thread_stripe = display.ram[NMI_LOAD_BG_FROM_VRAM] == 1;
        let pending_full_tilemap_upload =
            display.ram[crate::game_state::constants::NMI_SUBROUTINE_INDEX] == 1;
        let live_pending_main_thread_stripe = self.ram[NMI_LOAD_BG_FROM_VRAM] == 1;
        // RenderText_Draw_Finish authors the fixed-source stripe that replaces
        // the dialogue box with tile 0x387f, then returns to the saved module.
        // When live NMI has consumed that exact packet, Snes9x scans out the
        // cleared BG3 tilemap immediately. The ordinary menu-stripe cadence
        // remains deferred even after its live flag is cleared.
        let consumed_dialogue_box_clear = pending_main_thread_stripe
            && !live_pending_main_thread_stripe
            && stripe_upload_clears_dialogue_box(
                &display.ram[crate::game_state::constants::VRAM_UPLOAD_DATA..],
            );
        let retained_full_tilemap_vram = rom_full_tilemap_scanout_retains_uploaded_region(
            pending_full_tilemap_upload,
            display.ppu.forced_blank_scanlines,
        )
        .then(|| {
            let target_page = display.ram[crate::game_state::constants::NMI_LOAD_TARGET_ADDR];
            let (destination, word_count) = full_tilemap_nmi_vram_region(target_page)?;
            RetainedVramRegion::capture(&display.ppu.vram, destination, word_count)
        })
        .flatten();
        // Attract_ControlMapZoom authors the final Mode 7 HDMA field before
        // AttractDramatize_WorldMap enters EnableForceBlank/EraseTileMaps.
        // When that typed projection drains from the staged pipeline, the
        // following snapshot owns both the blanking edge and cleared VRAM;
        // neither may be merged backward merely because it is live now.
        let following_staged_scanout_owns_attract_exit_generation = matches!(
            display.hdma_table_generation,
            DisplayHdmaTableGeneration::AttractMapProjectionDuringScanout { .. }
        ) && !display.ppu.forced_blank
            && self
                .deferred_display_snapshot
                .as_ref()
                .is_some_and(|following| {
                    following.ppu.forced_blank
                        || following.ram[crate::game_state::constants::INIDISP_COPY] & 0x80 != 0
                });
        let retain_previous_nmi_display_memory = (rom_display_memory_publication_is_deferred(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            snapshot_messaging.runtime.text_render_state(),
            pending_main_thread_stripe,
        ) && !consumed_dialogue_box_clear)
            || (snapshot_frame.main_module == 20
                && snapshot_frame.submodule == 0
                // Snes9x retains the pre-NMI attract image through the sequence-1
                // load/fade-out. Mode 7 begins publishing immediately once that
                // sequence has entered its fade-in state.
                && !(snapshot_attract_scene.sequence() == 1
                    && snapshot_attract_scene.state() >= 4))
            || following_staged_scanout_owns_attract_exit_generation;
        let active_display_nmi_overrun = display.ppu.forced_blank_scanlines != 0;
        let module_oam_publication_is_deferred = rom_display_oam_publication_is_deferred(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            snapshot_messaging.runtime.text_render_state(),
            active_display_nmi_overrun,
            pending_main_thread_stripe,
        );
        let dungeon_exit_crosses_nmi_boundary = rom_dungeon_exit_entry_crosses_nmi_boundary(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
            ),
        );
        let world_map_fade_display = snapshot_frame.main_module == 20
            && snapshot_frame.submodule == 0
            && snapshot_attract_scene.sequence() == 1
            && snapshot_attract_scene.state() >= 4;
        let world_map_mode7_brightness_is_early_published =
            rom_attract_world_map_mode7_brightness_is_early_published(
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                snapshot_attract_scene.sequence(),
                snapshot_attract_scene.state(),
            );
        let live_frame = crate::game_state::FrameState::load_from_ram(&self.ram);
        let host_entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or(snapshot_frame);
        // At this split boundary, Link OBJ VRAM and the doorway scroll publish
        // from live state while the remaining staged display controls retain
        // their independently measured generation.
        let publish_live_overworld_bad_weather_scroll = rom_overworld_bad_weather_scroll_is_live(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            display.ppu.bg_layer[0].h_scroll,
            display.ppu.bg_layer[0].v_scroll,
            display.ppu.bg_layer[1].h_scroll,
            display.ppu.bg_layer[1].v_scroll,
            self.ppu.bg_layer[0].h_scroll,
            self.ppu.bg_layer[0].v_scroll,
            self.ppu.bg_layer[1].h_scroll,
            self.ppu.bg_layer[1].v_scroll,
        );
        let spiral_stair_return_publication_is_active =
            self.spiral_stair_return_oam_publication_host_frame == Some(self.frame_ctr_dbg)
                && self.game_state.world.location.dungeon_room_index() == 1
                && self.game_state.dungeon.stair_movement.staircase_index() == 0x30
                && self.game_state.player.follower_link.y_button_action_step() == 2;
        let publication_signals = DisplayPublicationSignals {
            retain_previous_nmi_display_memory,
            module_oam_publication_is_deferred,
            dungeon_exit_crosses_nmi_boundary,
            publish_live_overworld_bad_weather_scroll,
            attract_map_retains_display_memory: retain_previous_nmi_display_memory
                && snapshot_frame.main_module == 20
                && snapshot_frame.submodule == 0,
            world_map_fade_display,
            world_map_mode7_brightness_is_early_published,
            // The entry scanout keeps the captured camera while composing the
            // mixed Link-OAM handoff. Starting with the following hold-item
            // scanout, the camera written by that handoff is live. Keeping the
            // two phases separate avoids publishing the scroll one frame early.
            dungeon_item_hold_publishes_live_scroll: dungeon_item_hold_publishes_live_scroll(
                snapshot_frame,
                FollowerLinkState::load_from_ram(&display.ram).handler_state(),
                display.dungeon_item_hold_entry_scanout,
            ),
            spiral_stair_landing_publishes_live_display:
                rom_spiral_stair_landing_publishes_live_display(
                    snapshot_frame,
                    snapshot_world_location.dungeon_room_index(),
                    snapshot_dungeon.stair_movement.staircase_index(),
                    FollowerLinkState::load_from_ram(&display.ram).y_button_action_step(),
                ),
            spiral_stair_motion_publishes_live_oam: rom_spiral_stair_motion_publishes_live_oam(
                snapshot_frame,
                snapshot_world_location.dungeon_room_index(),
                snapshot_dungeon.stair_movement.staircase_index(),
            ),
            spiral_stair_return_publishes_live_shadow_oam:
                spiral_stair_return_publication_is_active,
            // Module09_LoadNewSprites' completing slice authors a shadow that
            // hides Link's entries, but its suspended tail
            // (FinishOverworldSpriteReloadTail) holds the following vblanks,
            // so that shadow never reaches the PPU (route frames 6948..6950):
            // keep the previously presented resident generation instead of
            // adopting the published one while the tail is pending.
            overworld_sprite_reload_completion_retains_presented: snapshot_frame.main_module == 9
                && matches!(snapshot_frame.submodule, 4 | 5)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishOverworldSpriteReloadTail { .. })
                ),
            spiral_stair_return_publishes_live_registers: self
                .spiral_stair_return_oam_publication_host_frame
                == Some(self.frame_ctr_dbg),
            spiral_stair_return_publishes_live_obj_cache: self
                .spiral_stair_return_oam_publication_host_frame
                .is_some_and(|start| self.frame_ctr_dbg.wrapping_sub(start) == 1),
            // Module07_0A runs after a torch transition has crossed vblank.
            // Its screen-layer writes are already active for this scanout even
            // though the general captured display generation still precedes
            // that NMI boundary.
            dungeon_brightness_publishes_live_display: dungeon_brightness_screen_layers_are_live(
                host_entry_frame,
                live_frame,
            ),
            dungeon_brightness_publishes_live_animated_bg: dungeon_brightness_animated_bg_is_live(
                host_entry_frame,
                live_frame,
                self.ram[crate::game_state::constants::BG_TILE_ANIMATION_COUNTDOWN],
            ),
            dungeon_state_13_phase: if self
                .dungeon_state_13_atomic_caller_return_publication_host_frame
                == Some(self.frame_ctr_dbg)
            {
                DungeonState13PublicationPhase::AtomicCallerReturn
            } else if self.dungeon_state_13_caller_return_publication_host_frame
                == Some(self.frame_ctr_dbg)
            {
                DungeonState13PublicationPhase::CallerReturn
            } else if self.dungeon_state_13_recurring_main_publication_host_frame
                == Some(self.frame_ctr_dbg)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::State13CallerReturn,
                    })
                )
            {
                if self.dungeon_state_13_pre_main_publication_host_frame == Some(self.frame_ctr_dbg)
                {
                    DungeonState13PublicationPhase::PreMainQuadrantNmiEntry
                } else {
                    DungeonState13PublicationPhase::RecurringMain
                }
            } else {
                DungeonState13PublicationPhase::None
            },
            dungeon_faded_filter_phase: if self
                .dungeon_faded_filter_caller_return_publication_host_frame
                == Some(self.frame_ctr_dbg)
            {
                DungeonFadedFilterPublicationPhase::CallerReturn
            } else if self.dungeon_faded_filter_palette_completion_host_frame
                == Some(self.frame_ctr_dbg)
            {
                DungeonFadedFilterPublicationPhase::LandingPaletteCompletion
            } else {
                DungeonFadedFilterPublicationPhase::None
            },
        };
        let publication_plan = DisplayPublicationPlan::resolve(&display, publication_signals);
        if nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DISPLAY_OAM_FRAME",
            self.frame_ctr_dbg,
        ) {
            eprintln!(
                "display_publication_phase host={} snapshot={:02x}/{:02x}/{:02x} live={:02x}/{:02x}/{:02x} brightness_live={} faded={:?} cgram={:?} cgram_override={} effective_cgram={} oam_source={:?}",
                self.frame_ctr_dbg,
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                snapshot_frame.subsubmodule,
                live_frame.main_module,
                live_frame.submodule,
                live_frame.subsubmodule,
                publication_signals.dungeon_brightness_publishes_live_display,
                publication_signals.dungeon_faded_filter_phase,
                display.cgram_scanout_generation,
                display.cgram_scanout_override.is_some(),
                display
                    .effective_presented_dma
                    .as_ref()
                    .is_some_and(|receipt| receipt.completed_cgram.is_some()),
                publication_plan.oam_scanout_source,
            );
        }
        if crate::debug_env::var_os("ZELDA3_DEBUG_SPIRAL_RETURN").is_some()
            && self
                .spiral_stair_return_oam_publication_host_frame
                .is_some()
        {
            eprintln!(
                "spiral_return_publication host={} start={:?} player_oam={} registers={} obj_cache={} captured_link={:04x} live_link={:04x}",
                self.frame_ctr_dbg,
                self.spiral_stair_return_oam_publication_host_frame,
                publication_signals.spiral_stair_return_publishes_live_shadow_oam,
                publication_signals.spiral_stair_return_publishes_live_registers,
                publication_signals.spiral_stair_return_publishes_live_obj_cache,
                read_le_u16(&display.ram, LINK_DMA_GRAPHICS_INDEX),
                read_le_u16(&self.ram, LINK_DMA_GRAPHICS_INDEX),
            );
        }
        let display_phase_differs_from_live = snapshot_frame.main_module
            != self.game_state.frame.main_module
            || snapshot_frame.submodule != self.game_state.frame.submodule;
        if diagnostics.display_oam && display_phase_differs_from_live {
            eprintln!(
                "display_oam snapshot={:02x}/{:02x} live={:02x}/{:02x} retain={} reasons=module:{}/captured_ppu:{}/dungeon_exit:{} snapshot_math={:02x}/{:02x}/{}/{} live_math={:02x}/{:02x}/{}/{} snapshot_oam={:02x?} live_oam={:02x?}",
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                publication_plan.retain_captured_oam,
                module_oam_publication_is_deferred,
                matches!(
                    publication_plan.oam_scanout_source,
                    OamScanoutSource::RetainCapturedBeforeNmi
                ),
                dungeon_exit_crosses_nmi_boundary,
                display.ppu.math_enabled,
                display.ppu.prevent_math_mode,
                display.ppu.subtract_color,
                display.ppu.half_color,
                self.ppu.math_enabled,
                self.ppu.prevent_math_mode,
                self.ppu.subtract_color,
                self.ppu.half_color,
                &display.ppu.oam[..4],
                &self.ppu.oam[..4],
            );
        }
        let completed_nmi_forced_blank_write = display
            .effective_presented_dma
            .as_ref()
            .and_then(|receipt| receipt.completed_ppu_registers)
            .map(|registers| registers.inidisp.forced_blank);
        let live_forced_blank = live_forced_blank_for_scanout(
            display.ppu.forced_blank,
            completed_nmi_forced_blank_write,
            self.active_display_force_blank_event,
            following_staged_scanout_owns_attract_exit_generation,
        );
        let live_forced_blank_from_scanline = self
            .active_display_force_blank_event
            .or(self.ppu.forced_blank_from_scanline);
        let live_retain_active_display_history = self.ppu.retain_active_display_history;
        // This capture owns the game-authored INIDISP value for the active
        // frame. The live PPU has already run the following NMI boundary and
        // can therefore be one fade step ahead.
        let captured_screen_brightness = if display.game_over_iris_goal_scanout_closed {
            0
        } else {
            display.ram[crate::game_state::constants::INIDISP_COPY] & 0x0f
        };
        if diagnostics.nmi_latch && (1648..=1655).contains(&self.frame_ctr_dbg) {
            eprintln!(
                "display_blanking host={} snapshot_forced={} snapshot_prefix={} snapshot_from={:?} live_forced={} live_from={:?}",
                self.frame_ctr_dbg,
                display.ppu.forced_blank,
                display.ppu.forced_blank_scanlines,
                display.ppu.forced_blank_from_scanline,
                live_forced_blank,
                live_forced_blank_from_scanline,
            );
        }
        std::mem::swap(&mut self.ram, &mut display.ram);
        std::mem::swap(&mut self.ppu, &mut display.ppu);
        std::mem::swap(&mut self.dma, &mut display.dma);
        std::mem::swap(&mut self.vram_chr_source, &mut display.vram_chr_source);
        std::mem::swap(
            &mut self.vram_chr_preview_source,
            &mut display.vram_chr_preview_source,
        );
        // Preserve the exact outgoing OBJ scanout before coarse composition
        // clears or replaces its decoded latch. Effective DMA writes below
        // advance this generation, never the live post-frame PPU now stored
        // in `display.ppu`.
        let captured_obj_name_bases = [self.ppu.obj_tile_adr1, self.ppu.obj_tile_adr2];
        let captured_obj_vram_latch = self.ppu.obj_vram_latch.clone();
        self.compose_display_registers(&display, &publication_plan);
        self.compose_effective_presented_ppu_registers(&display, publication_plan.bg_scroll_source);
        let window_screen_mask_mismatches = self
            .active_presented_window_mask
            .as_ref()
            .map(|authority| {
                let authority = authority.screen_windowed_layers();
                let mismatches = self
                    .ppu
                    .screen_windowed
                    .iter()
                    .zip(authority)
                    .filter(|(native, authority)| **native != *authority)
                    .count();
                self.ppu.screen_windowed = authority;
                mismatches
            })
            .unwrap_or(0);
        if let Some(authority) = self.active_presented_window_mask.as_ref() {
            // Convert the backend-neutral per-layer predicates into the PPU
            // mirror consumed by the native renderer. No source address,
            // CPU state, or Snes9x register packing crosses the gameplay
            // receipt boundary.
            self.ppu.windowsel = authority.layer_predicates().into_iter().enumerate().fold(
                0u32,
                |packed, (layer, predicate)| {
                    let window1_enable = (predicate & 0x01 != 0) as u32;
                    let window1_outside = (predicate & 0x02 != 0) as u32;
                    let window2_enable = (predicate & 0x04 != 0) as u32;
                    let window2_outside = (predicate & 0x08 != 0) as u32;
                    let native_nibble = window1_outside
                        | (window1_enable << 1)
                        | (window2_outside << 2)
                        | (window2_enable << 3);
                    packed | (native_nibble << (layer * 4))
                },
            );
        }
        if self.active_presented_window_mask.is_some() {
            self.original_timing_window_mask_shadow_result =
                Some(crate::OriginalTimingWindowMaskShadowResult {
                    compared_scanline_windows: 0,
                    mismatched_scanline_windows: 0,
                    mismatched_screen_masks: window_screen_mask_mismatches,
                    first_mismatch: None,
                });
        }
        let previous_dialogue_scanout = self.displayed_dialogue_scanout();
        if diagnostics.scroll_retain && self.dialogue_scroll_phase() != DialogueScrollPhase::Idle {
            let dialogue_scroll_phase = self.dialogue_scroll_phase();
            let scanout_sum = |scanout: &DialogueTextScanout| {
                scanout
                    .vram
                    .iter()
                    .map(|&word| u64::from(word & 0xff) + u64::from(word >> 8))
                    .sum::<u64>()
            };
            eprintln!(
                "scroll_retain host={} phase={dialogue_scroll_phase:?} frozen={:?} completion={:?} nmi_retained={}",
                self.frame_ctr_dbg,
                self.dialogue_scroll_frozen_scanout
                    .as_ref()
                    .map(&scanout_sum),
                self.dialogue_scroll_completion_scanout
                    .as_ref()
                    .map(&scanout_sum),
                retain_previous_nmi_display_memory,
            );
        }
        self.compose_display_vram(
            &display,
            &publication_plan,
            retained_full_tilemap_vram.as_ref(),
        );
        self.compose_effective_presented_vram(&display);
        if let Some(words) = display.presented_hud_tilemap_override.as_deref() {
            debug_assert_eq!(words.len(), HUD_TILEMAP_NMI_WORDS);
            self.ppu.vram[HUD_TILEMAP_VRAM_DESTINATION
                ..HUD_TILEMAP_VRAM_DESTINATION + HUD_TILEMAP_NMI_WORDS]
                .copy_from_slice(words);
        }
        self.compose_effective_presented_bg_chr_cache(&display);
        self.compose_decoded_bg_chr_cache();
        if let Some(receipt) = display.presented_animated_bg_tiles_override.as_ref() {
            self.apply_original_timing_presented_animated_bg_tiles(receipt);
        }
        self.compose_display_chr_sources(&display, &publication_plan);
        self.compose_display_cgram(&display, &publication_plan);
        self.compose_effective_presented_cgram(&display);
        if let Some(previous_dialogue_scanout) = previous_dialogue_scanout.as_ref() {
            self.ppu.vram[0x7c00..0x7ff0].copy_from_slice(&previous_dialogue_scanout.vram);
        }
        let presented_dialogue_text_override = display.presented_dialogue_text_override.clone();
        let dialogue_text_authority_differs = presented_dialogue_text_override
            .as_ref()
            .is_some_and(|receipt| self.apply_original_timing_presented_dialogue_text(receipt));
        self.staged_presented_cgram = Some(self.ppu.cgram.clone());
        if diagnostics.scroll_retain && self.dialogue_scroll_phase() != DialogueScrollPhase::Idle {
            let dialogue_scroll_phase = self.dialogue_scroll_phase();
            let presented_sum: u64 = self.ppu.vram[0x7c00..0x7ff0]
                .iter()
                .map(|w| u64::from(w & 0xff) + u64::from(w >> 8))
                .sum();
            eprintln!(
                "scroll_present host={} presented_sum={presented_sum} phase={dialogue_scroll_phase:?}",
                self.frame_ctr_dbg,
            );
        }
        let enemy_drop_extended_oam_crossed_publication_boundary = display
            .enemy_drop_item_graphics_live_extended_oam
            && display
                .published_shadow_oam_dma
                .as_ref()
                .is_some_and(|published| published[256..] != display.ppu.oam[256..]);
        self.compose_display_oam(&display, &publication_plan);
        if enemy_drop_extended_oam_crossed_publication_boundary {
            self.enemy_drop_item_graphics_live_extended_oam_pending = false;
        }
        let explicit_obj_cache_owner = display.explicit_obj_cache_vram.is_some()
            || display.effective_obj_cache_generation()
                != DisplayObjCacheGeneration::FollowModuleCadence;
        self.compose_effective_presented_obj_from_captured_scanout(
            &display,
            captured_obj_name_bases,
            captured_obj_vram_latch.as_deref(),
        );
        let completed_dma = display
            .effective_presented_dma
            .as_ref()
            .and_then(|receipt| receipt.completed_link_obj_dma);
        if nmi::debug_frame_selection_env_matches("ZELDA3_DEBUG_OBJ_PIPE", self.frame_ctr_dbg) {
            eprintln!(
                "objpipe host={} stage=link_receipt_compose publication_host={} plan={:?} explicit={} semantic_cache={} captured={:?} completed={:?}",
                self.frame_ctr_dbg,
                display.publication_host_frame,
                publication_plan.link_obj_scanout_generation,
                explicit_obj_cache_owner,
                display.explicit_obj_cache_vram.is_some(),
                LinkDmaSources::load_from_ram(&display.ram),
                completed_dma,
            );
        }
        let scanout_sources =
            link_obj_cache_sources_for_publication(completed_dma, explicit_obj_cache_owner);
        if let Some(scanout_sources) = scanout_sources {
            let base_vram = self
                .ppu
                .obj_vram_latch
                .as_deref()
                .unwrap_or(&self.ppu.vram)
                .to_vec();
            let obj_cache_vram =
                compose_early_link_obj_cache(&base_vram, scanout_sources, self.asset_raw(57));
            // Merge the six transfers after any typed or coarse receipt so
            // unrelated completed OBJ writes survive. Live plans without a
            // hardware receipt retain the captured cache.
            self.set_obj_vram_latch_traced(Some(obj_cache_vram));
        }
        if let Some(receipt) = display.presented_obj_tiles_override.as_ref() {
            // This receipt contains sparse address-bearing OBJ tiles decoded
            // by the completed source scanout. Apply it after raw-DMA
            // invalidation and Link-cache composition so it replaces exactly
            // those tiles without claiming either complete OBSEL name page.
            self.apply_original_timing_presented_obj_tiles_from_captured_scanout(
                receipt,
                captured_obj_vram_latch.as_deref(),
            );
        }
        // Sparse OBJ receipts can replace the logical identity of only their
        // addressed tiles with exact content keys. Stage source ownership
        // after applying them so the next presented generation remains paired
        // with the same decoded cache words.
        self.staged_presented_vram_chr_source = Some(self.vram_chr_source.clone());
        self.staged_presented_vram_chr_preview_source = Some(self.vram_chr_preview_source.clone());
        if crate::debug_env::var_os("ZELDA3_AUDIT_OAM_LAW").is_some() {
            if let Some(law) = self.oam_law_visible.as_deref() {
                let limit = self.ppu.oam.len().min(law.len());
                let diffs: Vec<usize> = (0..limit)
                    .filter(|&word| self.ppu.oam[word] != law[word])
                    .collect();
                if !diffs.is_empty() {
                    eprintln!(
                        "oam_law_delta host={} words={} first={:?} source={:?}",
                        self.frame_ctr_dbg,
                        diffs.len(),
                        &diffs[..diffs.len().min(6)],
                        publication_plan.oam_scanout_source,
                    );
                }
            }
        }
        // Prefer the transfer receipt recorded by the NMI that owns this
        // scanout. The OAM-law lane reconstructs an ordinary completed-main
        // transfer from the translated post-main shadow; it remains a fallback
        // for captures without a receipt, but cannot supersede an observed DMA
        // operand when a suspended C call crosses the boundary mid-return.
        let effective_active_oam = (!publication_plan
            .oam_scanout_source
            .blocks_post_composition_override())
        .then(|| {
            display
                .effective_presented_dma
                .as_ref()
                .and_then(|receipt| receipt.completed_oam.as_deref())
        })
        .flatten()
        .filter(|oam| oam.len() == self.ppu.oam.len());
        let completed_active_oam = publication_plan
            .oam_scanout_source
            .active_nmi_dma_is_presented()
            .then_some(display.completed_oam_dma_after_capture.as_deref())
            .flatten()
            .filter(|oam| oam.len() == self.ppu.oam.len());
        if let Some(completed) = effective_active_oam.or(completed_active_oam) {
            self.ppu.oam.clone_from_slice(completed);
        } else if let Some(law) = (!publication_plan
            .oam_scanout_source
            .blocks_post_composition_override())
        .then_some(self.oam_law_visible.as_deref())
        .flatten()
        .filter(|law| law.len() == self.ppu.oam.len())
        {
            self.ppu.oam.clone_from_slice(law);
        }
        if let Some(authoritative) = display
            .presented_oam_override
            .as_deref()
            .filter(|oam| oam.len() == self.ppu.oam.len())
        {
            self.ppu.oam.clone_from_slice(authoritative);
        }
        self.staged_presented_oam = Some(self.ppu.oam.clone());
        let presented_obj_vram = self.ppu.obj_vram_latch.as_deref().unwrap_or(&self.ppu.vram);
        if self.debug_obj_pipe_enabled() {
            self.debug_obj_pipe(
                &format!("stage latch={}", self.ppu.obj_vram_latch.is_some()),
                &presented_obj_vram[0x4000..0x4400],
            );
            if let Some(snap) = &self.display_snapshot {
                let a = &presented_obj_vram[0x4000..0x4400];
                let b = &snap.ppu.vram[0x4000..0x4400];
                let offs: Vec<usize> = (0..0x400).filter(|&i| a[i] != b[i]).collect();
                eprintln!(
                    "objstage_diff host={} n={} offs={:03x?}",
                    self.frame_ctr_dbg,
                    offs.len(),
                    &offs[..offs.len().min(16)],
                );
            }
        }
        self.staged_presented_obj_vram = Some(presented_obj_vram.to_vec());
        self.compose_display_raster(
            live_forced_blank,
            live_forced_blank_from_scanline,
            live_retain_active_display_history,
            captured_screen_brightness,
            &publication_plan,
        );
        if let Some(field) = self.active_native_spotlight_field_scanout.as_ref() {
            // Main can finish a table copy or disable the iris before capture.
            // Retain the controls and individual rows consumed during this
            // field, independently of those later CPU-side publications.
            field.windows.compose_into(&mut self.ram, &mut self.ppu, &mut self.dma);
            if let Some((brightness, output_row)) = field.blanking {
                self.ppu.brightness = brightness;
                self.ppu.scanout_brightness_override = None;
                self.ppu.forced_blank = false;
                self.ppu.forced_blank_scanlines = 0;
                self.ppu.forced_blank_from_scanline = Some(output_row);
                self.ppu.retain_active_display_history = false;
                self.ppu.refresh_brightness_cache();
            }
        }
        if let Some(inidisp) = display.presented_inidisp_override {
            // The receipt describes the completed scanout's semantic raster,
            // so it replaces the native approximation for this one outgoing
            // surface without mutating the live post-frame mirror.
            self.ppu.brightness = inidisp.brightness;
            self.ppu.scanout_brightness_override = None;
            self.ppu.forced_blank_scanlines = inidisp.forced_blank_prefix;
            self.ppu.forced_blank_from_scanline = inidisp.forced_blank_suffix_start;
            // `forced_blank` is the frame-wide register shortcut. Partial
            // rasters are owned exclusively by the prefix/suffix fields so
            // renderers cannot erase rows which scanned out before the edge.
            self.ppu.forced_blank =
                usize::from(inidisp.forced_blank_prefix) == crate::PresentedInidisp::VISIBLE_LINES;
            self.ppu.retain_active_display_history = inidisp.retain_prior_surface;
            self.ppu.refresh_brightness_cache();
        }
        self.ppu.scanout_top_crop = display
            .presented_scanout_geometry_override
            .map_or(0, |geometry| geometry.top_crop);
        if display.game_over_iris_goal_scanout_closed {
            // A terminal iris can publish brightness zero before the captured
            // CPU generation advances. Game Over closes the active field
            // itself, so its final register and visible scanout both own zero.
            self.ppu.brightness = 0;
            self.ppu.refresh_brightness_cache();
        }
        if diagnostics.nmi_latch && (1648..=1655).contains(&self.frame_ctr_dbg) {
            eprintln!(
                "display_blanking_composed host={} forced={} prefix={} from={:?}",
                self.frame_ctr_dbg,
                self.ppu.forced_blank,
                self.ppu.forced_blank_scanlines,
                self.ppu.forced_blank_from_scanline,
            );
        }
        self.debug_dump_presented_state(&publication_plan, &display);
        self.sync_native_game_state_from_ram();
        // The RAM-derived rebuild reconstitutes the palette mirror from the
        // snapshot's WRAM shadow, which already holds THIS frame's palette
        // writes; hardware scanout shows the palette uploaded in the PREVIOUS
        // vblank. Re-publish the composed pre-NMI CGRAM image so effect
        // materials and live-CGRAM readers see what the PPU actually displays
        // (the attract palette filter diverges from Snes9x otherwise).
        self.game_state
            .display
            .palette_provenance
            .0
            .reconstitute_cgram(&self.ppu.cgram);
        if diagnostics.capture.attract_timeline && (5640..=5700).contains(&self.frame_ctr_dbg) {
            let trace = format!(
                "attract_display_present host={} snapshot={:02x}/{:02x} live={:02x}/{:02x} world_map_fade={} bright={} c0={:04x} c1={:04x} fixed={:02x},{:02x},{:02x} math={:02x}/{:02x}",
                self.frame_ctr_dbg,
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                world_map_fade_display,
                self.ppu.brightness,
                self.ppu.cgram[0],
                self.ppu.cgram[1],
                self.ppu.fixed_color_r,
                self.ppu.fixed_color_g,
                self.ppu.fixed_color_b,
                self.ppu.math_enabled,
                self.ppu.prevent_math_mode,
            );
            append_parity_trace("attract-display-timeline.trace", &trace);
        }
        // Semantic dialogue data is another representation of the same BG3
        // generation as the presented VRAM. Resolve that generation once
        // above and use it for both representations: independently repeating
        // only part of the VRAM-retention condition lets semantic text lead
        // the exact hardware pixels during an interrupted VWF render.
        // A dialogue scroll override still owns both representations.
        let mut presented_dialogue = self.resolve_displayed_dialogue_metadata(
            &pristine_snapshot,
            previous_dialogue_scanout.as_ref(),
            publication_plan.vram_generation,
        );
        if dialogue_text_authority_differs {
            // Native glyph metadata names the native word generation. Keep
            // semantic and raw representations coupled by retiring it when the
            // authoritative completed scanout differs; the renderer then
            // decodes the authoritative SNES character words directly.
            presented_dialogue.glyph_runs.clear();
            presented_dialogue.glyph_run_dialogue_offsets.clear();
        }
        if diagnostics.capture.frame_boundary {
            let published_shadow_oam_diff =
                display.published_shadow_oam_dma.as_deref().map(|shadow| {
                    let mismatches = self
                        .ppu
                        .oam
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .zip(shadow.iter().flat_map(|word| word.to_le_bytes()))
                        .enumerate()
                        .filter_map(|(index, (presented, published))| {
                            (presented != published).then_some((index, presented, published))
                        })
                        .collect::<Vec<_>>();
                    (mismatches.len(), mismatches.first().copied())
                });
            eprintln!(
                "frame_boundary_present host={} vram={:?} animated_bg={:?} link_obj={:?} oam={:?} bg_scroll={:?} bg1=({:04x},{:04x}) screen={:02x?} effective_screen={:?} effective_vram={} effective_dialogue_runs={} retained_obj={} published_shadow_oam_diff={:?} dialogue_runs=live:{}/captured:{}/presented:{} scroll_override={}",
                self.frame_ctr_dbg,
                publication_plan.vram_generation,
                publication_plan.animated_bg_scanout_generation,
                publication_plan.link_obj_scanout_generation,
                publication_plan.oam_scanout_source,
                publication_plan.bg_scroll_source,
                self.ppu.bg_layer[0].h_scroll,
                self.ppu.bg_layer[0].v_scroll,
                self.ppu.screen_enabled,
                pristine_snapshot
                    .effective_presented_dma
                    .as_ref()
                    .and_then(|receipt| receipt.completed_ppu_registers)
                    .map(|registers| registers.color_math.screen_enabled),
                pristine_snapshot
                    .effective_presented_dma
                    .as_ref()
                    .map_or(0, |receipt| receipt.vram_writes.len()),
                pristine_snapshot
                    .effective_presented_dma
                    .as_ref()
                    .and_then(|receipt| receipt.completed_dialogue_metadata.as_ref())
                    .map_or(0, |metadata| metadata.glyph_runs.len()),
                matches!(
                    pristine_snapshot.obj_generation,
                    DisplayObjGeneration::RetainCapturedMemory { .. }
                ),
                published_shadow_oam_diff,
                self.published_bg3_vwf_glyph_runs.len(),
                pristine_snapshot.published_bg3_vwf_glyph_runs.len(),
                presented_dialogue.glyph_runs.len(),
                previous_dialogue_scanout.is_some(),
            );
        }
        let saved_published_dialogue = presented_dialogue.replace_in(self);
        let captured = capture(self);
        saved_published_dialogue.replace_in(self);
        self.active_presented_bg_scroll = saved_presented_bg_scroll;
        self.active_presented_mode7_transform = saved_presented_mode7_transform;
        self.active_presented_window_mask = saved_presented_window_mask;

        std::mem::swap(&mut self.ram, &mut display.ram);
        std::mem::swap(&mut self.ppu, &mut display.ppu);
        std::mem::swap(&mut self.dma, &mut display.dma);
        std::mem::swap(&mut self.vram_chr_source, &mut display.vram_chr_source);
        std::mem::swap(
            &mut self.vram_chr_preview_source,
            &mut display.vram_chr_preview_source,
        );
        self.game_state = saved_game_state;
        drop(display);
        self.display_snapshot = Some(pristine_snapshot);
        captured
    }

    /// Resume a translated caller suffix that owns this pre-main CPU slice.
    ///
    /// Returning `true` means the caller reached its trailing NMI and consumed
    /// the host frame; no fresh module iteration may run afterward.
    pub(super) fn resume_pre_main_caller_continuation(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        let Some(continuation) = self.game_execution_scheduler.pre_main_caller_continuation()
        else {
            return false;
        };

        match continuation {
            PreMainCallerContinuation::DialogueVwfReturn => {
                if self.game_over_text_render_loop_active() {
                    self.finish_pre_main_caller_continuation(continuation);
                    self.dialogue_fast_forward_hold_active = false;
                    self.finish_game_over_text_render_call();
                    if self.game_over_text_render_loop_active() {
                        self.resume_game_over_text_render_loop();
                    }
                    self.dialogue_fast_forward_hold_active =
                        std::mem::take(&mut self.dialogue_fast_forward_hold_pending);
                    if !self.game_over_text_render_loop_active() {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source, false);
                    return true;
                }
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.original_timing_main_loop_return_timeline().is_some()
                {
                    let source_vwf_progress = self.original_timing_pre_main_vwf_return_progress();
                    // The wire's typed return timeline owns the carried
                    // handler, suffix, and trailing acceptance ordering: the
                    // ROM completes the carried Held handler first, then the
                    // resumed caller's suffix clears the latch before the
                    // trailing Open acceptance (route host 82757). The legacy
                    // direct path below cleared the latch before completing
                    // the carried handler and tripped the Held gate.
                    let timeline = self
                        .take_original_timing_pre_main_caller_return_timeline(continuation)
                        .expect("the inspected VWF return timeline disappeared");
                    if let Some(progress) = source_vwf_progress {
                        self.take_original_timing_pre_main_vwf_return_progress(progress);
                    }
                    self.finish_pre_main_caller_continuation(continuation);
                    self.dialogue_fast_forward_hold_active = false;
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source,
                        ZeldaState::complete_module0e_interface_after_run,
                    );
                    self.dialogue_vwf_handler_entry_phase =
                        messaging::VwfHandlerEntryPhase::AfterDeferredCallerSuffix;
                    let main_loop_progress = self.take_original_timing_main_loop_progress();
                    if matches!(
                        main_loop_progress,
                        Some(crate::MainLoopProgress::IterationStarted)
                    ) {
                        self.game_execution_scheduler
                            .mark_main_iteration_after_leading_nmi();
                        self.zelda_run_game_loop_with_progress(main_loop_progress);
                        self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
                    }
                    return true;
                }
                self.finish_pre_main_caller_continuation(continuation);
                self.dialogue_fast_forward_hold_active = false;
                self.complete_module0e_interface_after_run();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                self.capture_display_snapshot();
                // The interrupted caller has now reached the ordinary
                // game-loop boundary. Run its trailing NMI exactly once:
                // it consumes the preprocessed-audio marker and publishes
                // the completed BG3 text for the following scanout.
                self.interrupt_nmi(input, oam_dma_source, false);
                self.dialogue_vwf_handler_entry_phase =
                    messaging::VwfHandlerEntryPhase::AfterDeferredCallerSuffix;
                let main_loop_progress = self.take_original_timing_main_loop_progress();
                if matches!(
                    main_loop_progress,
                    Some(crate::MainLoopProgress::IterationStarted)
                ) {
                    // The source order is RenderText's deferred caller suffix,
                    // Module0E's scroll-register writes, Main_PrepSpritesForNmi,
                    // the leading hardware NMI, then the next ZeldaRunGameLoop
                    // entry. The replaceable timing authority observed that last
                    // semantic boundary in this same host call; do not discard it
                    // merely because the translated caller resumed pre-main.
                    self.game_execution_scheduler
                        .mark_main_iteration_after_leading_nmi();
                    self.zelda_run_game_loop_with_progress(main_loop_progress);
                    self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
                }
            }
            PreMainCallerContinuation::FileSelectCheckerboardUpload => {
                let source_return_timeline =
                    self.take_original_timing_pre_main_caller_return_timeline(continuation);
                if let Some(timeline) = source_return_timeline {
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source,
                        ZeldaState::complete_file_select_checkerboard_upload,
                    );
                } else {
                    self.complete_file_select_checkerboard_upload();
                    // Without a live source timeline, retain the translated
                    // scheduler's legacy one-boundary completion policy.
                    self.clear_nmi_update_latch();
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source, false);
                }
            }
            PreMainCallerContinuation::NamePlayerTilemapUpload => {
                let source_return_timeline =
                    self.take_original_timing_pre_main_caller_return_timeline(continuation);
                if let Some(timeline) = source_return_timeline {
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source,
                        ZeldaState::complete_module_name_player_1,
                    );
                } else {
                    self.complete_module_name_player_1();
                    // Without a live source timeline, retain the translated
                    // scheduler's legacy one-boundary completion policy.
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source, false);
                }
            }
            PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass { resumed_phase } => {
                self.finish_pre_main_caller_continuation(continuation);
                self.complete_dungeon_faded_filter_second_palette_pass();
                // Resume after the interrupted palette walk without repeating
                // the frame-counter, OAM-clear, or Link-movement prefix.
                if resumed_phase == ModuleCpuPhase::InterruptedInLinkOam {
                    let live_held_timeline =
                        (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                            && self.original_timing_semantic_receipts.is_some())
                        .then(|| {
                            self.take_original_timing_main_loop_interruption_timeline(Some(
                                crate::MainLoopInterruption::LinkOam,
                            ))
                        })
                        .flatten();
                    if let Some(timeline) = live_held_timeline {
                        // The wire owns this resumed caller: its carried
                        // handler completes, the one Sprite_Main crossing
                        // consumes the restated claim, and the caller stays
                        // suspended inside LinkOam (route host 31246).
                        assert_eq!(
                            timeline.progress,
                            crate::MainLoopProgress::CallStackContinued,
                            "a held LinkOam faded-filter caller cannot begin a fresh iteration",
                        );
                        assert!(
                            timeline.nmi_phases_after_interruption.is_empty(),
                            "a held LinkOam faded-filter caller cannot publish a post-interruption NMI lifecycle: {timeline:?}",
                        );
                        let before = classify_original_timing_nmi_phases_with_ownership(
                            self.original_timing_nmi_publication_pending,
                            &timeline.nmi_phases_before_interruption,
                        );
                        // A trailing held acceptance whose handler completes
                        // in the next host is carried (route host 1129666:
                        // [Handler, SpriteMainReturned, Held, LinkOam]).
                        let carry_trailing_acceptance = before.publication_pending_at_exit;
                        self.complete_original_timing_nmi_handler_for_active_scanout(
                            before.handler_completion,
                            input,
                            oam_dma_source,
                        )
                        .assert_no_unclaimed_dialogue_text_dma();
                        let sprite_main_return_claims = self
                            .original_timing_semantic_receipts
                            .as_ref()
                            .map(|receipts| {
                                receipts
                                    .semantic()
                                    .iter()
                                    .filter(|receipt| {
                                        **receipt
                                            == OriginalTimingSemanticReceipt::SpriteMainReturned
                                    })
                                    .count()
                            })
                            .unwrap_or(0);
                        if sprite_main_return_claims != 0 {
                            self.begin_original_timing_sprite_main_return_claim_scope(
                                sprite_main_return_claims,
                            );
                        }
                        self.dungeon_post_sprite_main_return_pending = true;
                        self.complete_module07_dungeon_after_submodule();
                        if sprite_main_return_claims != 0 {
                            self.finish_original_timing_sprite_main_return_claim_scope();
                        }
                        debug_assert!(matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
                        ));
                        if carry_trailing_acceptance {
                            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                        }
                        return true;
                    }
                    self.dungeon_post_sprite_main_return_pending = true;
                    self.complete_module07_dungeon_after_submodule();
                    debug_assert!(matches!(
                        self.game_execution_scheduler.current_work(),
                        Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
                    ));
                    return true;
                }
                // The estimate may place the interruption inside
                // NMI_PrepareSprites while the wire returns Sprite_Main in the
                // resumed host itself (route host 1129679: the fresh iteration
                // was held before Sprite_Main, then [Handler, SpriteMainReturned,
                // Continued, SuffixCompleted]); that host is a terminal return
                // of the whole resumed module tail.
                let live_terminal_with_sprite_main =
                    matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.as_ref().is_some_and(
                            |receipts| {
                                receipts
                                    .semantic()
                                    .contains(&OriginalTimingSemanticReceipt::SpriteMainReturned)
                            },
                        )
                        && self.original_timing_main_loop_interruption().is_none()
                        && self.original_timing_main_loop_return_timeline().is_some();
                if resumed_phase == ModuleCpuPhase::InterruptedInNmiPrepareSprites
                    && !live_terminal_with_sprite_main
                {
                    let live_held_timeline =
                        (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                            && self.original_timing_semantic_receipts.is_some())
                        .then(|| {
                            self.take_original_timing_main_loop_interruption_timeline(Some(
                                crate::MainLoopInterruption::SpritePreparation,
                            ))
                        })
                        .flatten();
                    let mut live_prep_carries_trailing_acceptance = false;
                    let sprite_main_return_claims = if let Some(timeline) = live_held_timeline {
                        // The wire owns this resumed caller: its carried
                        // handler completes, the one Sprite_Main crossing
                        // consumes the restated claim, and the caller stays
                        // suspended inside NMI_PrepareSprites (route host
                        // 31263).
                        assert_eq!(
                            timeline.progress,
                            crate::MainLoopProgress::CallStackContinued,
                            "a held prep-interrupted faded-filter caller cannot begin a fresh iteration",
                        );
                        assert!(
                            timeline.nmi_phases_after_interruption.is_empty(),
                            "a held prep-interrupted faded-filter caller cannot publish a post-interruption NMI lifecycle: {timeline:?}",
                        );
                        let before = classify_original_timing_nmi_phases_with_ownership(
                            self.original_timing_nmi_publication_pending,
                            &timeline.nmi_phases_before_interruption,
                        );
                        live_prep_carries_trailing_acceptance = before.publication_pending_at_exit;
                        self.complete_original_timing_nmi_handler_for_active_scanout(
                            before.handler_completion,
                            input,
                            oam_dma_source,
                        )
                        .assert_no_unclaimed_dialogue_text_dma();
                        let claims = self
                            .original_timing_semantic_receipts
                            .as_ref()
                            .map(|receipts| {
                                receipts
                                    .semantic()
                                    .iter()
                                    .filter(|receipt| {
                                        **receipt
                                            == OriginalTimingSemanticReceipt::SpriteMainReturned
                                    })
                                    .count()
                            })
                            .unwrap_or(0);
                        if claims != 0 {
                            self.begin_original_timing_sprite_main_return_claim_scope(claims);
                        }
                        claims
                    } else {
                        0
                    };
                    self.complete_module07_dungeon_after_submodule();
                    if sprite_main_return_claims != 0 {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    if live_prep_carries_trailing_acceptance {
                        // The interruption rode a second accepted-but-
                        // unfinished NMI; its handler belongs to the next
                        // host (route host 31272).
                        self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                    }
                    self.interrupted_nmi_prepare_obj_cache_vram = Some(
                        self.last_presented_obj_vram
                            .as_ref()
                            .cloned()
                            .or_else(|| self.ppu.obj_vram_latch.clone())
                            .unwrap_or_else(|| self.ppu.vram.clone()),
                    );
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
                            caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
                        },
                        1,
                    );
                    return true;
                }
                assert!(
                    resumed_phase == ModuleCpuPhase::CompleteBeforeNmi
                        || (resumed_phase == ModuleCpuPhase::InterruptedInNmiPrepareSprites
                            && live_terminal_with_sprite_main),
                    "resumed faded-filter caller phase is not implemented: host={} module={:02x}/{:02x}/{:02x} room={:04x} countdown={}",
                    self.frame_ctr_dbg,
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                    self.game_state.frame.subsubmodule,
                    self.game_state.world.location.dungeon_room(),
                    self.game_state.display.palette_filter.countdown(),
                );
                let live_terminal_timeline =
                    (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.is_some())
                    .then(|| self.take_original_timing_main_loop_return_timeline())
                    .flatten();
                if let Some(timeline) = live_terminal_timeline {
                    // The wire owns this resumed faded-filter caller: its
                    // carried handler completes first, the resumed Module 7
                    // suffix (including its one Sprite_Main) runs once, and
                    // the shared executor retires the common suffix and any
                    // trailing acceptance (route host 27103).
                    let sprite_main_return_claims = self
                        .original_timing_semantic_receipts
                        .as_ref()
                        .map(|receipts| {
                            receipts
                                .semantic()
                                .iter()
                                .filter(|receipt| {
                                    **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    if sprite_main_return_claims != 0 {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            sprite_main_return_claims,
                        );
                    }
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source,
                        |state| {
                            state.complete_module07_dungeon_after_submodule();
                            state.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                                oam: OamScanoutSource::RetainResidentPpuOam,
                                link_obj: GraphicsDmaGeneration::LiveAfterMain,
                                link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                            }));
                        },
                    );
                    if sprite_main_return_claims != 0 {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    self.retain_completed_palette_filter_cgram_scanout();
                    self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                        PreMainNmiResume::DungeonSupertileNextIterationAfterLeadingNmi,
                    );
                    return true;
                }
                self.complete_module07_dungeon_after_submodule();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainResidentPpuOam,
                    link_obj: GraphicsDmaGeneration::LiveAfterMain,
                    link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                }));
                if let Some(oam) = self.retiring_or_last_presented_oam().map(<[u16]>::to_vec) {
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam { oam });
                }
                self.capture_display_snapshot();
                self.retain_completed_palette_filter_cgram_scanout();
                // This path returned to the main wait before NMI. The next host
                // begins with that hardware NMI and may then start the following
                // fresh module iteration in the same host.
                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                    PreMainNmiResume::DungeonSupertileNextIterationAfterLeadingNmi,
                );
            }
            PreMainCallerContinuation::SpiralStairsSecondPaletteFilter => {
                let live_terminal_timeline =
                    (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.is_some())
                    .then(|| self.take_original_timing_main_loop_return_timeline())
                    .flatten();
                if let Some(timeline) = live_terminal_timeline {
                    // The wire owns this resumed filter caller: its carried
                    // handler completes first, the second palette pass and
                    // Module 7 suffix run once, and the shared executor
                    // carries the optional trailing acceptance.
                    let sprite_main_return_claims = self
                        .original_timing_semantic_receipts
                        .as_ref()
                        .map(|receipts| {
                            receipts
                                .semantic()
                                .iter()
                                .filter(|receipt| {
                                    **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    self.finish_pre_main_caller_continuation(continuation);
                    if sprite_main_return_claims != 0 {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            sprite_main_return_claims,
                        );
                    }
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source,
                        |state| {
                            state.complete_spiral_stairs_second_palette_filter();
                            state.complete_module07_dungeon_after_submodule();
                            let link_obj_dma_generation = state.presented_link_obj_dma_generation();
                            state.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                                oam: OamScanoutSource::RetainResidentPpuOam,
                                link_obj: link_obj_dma_generation,
                                link_obj_sources: link_obj_dma_generation,
                            }));
                        },
                    );
                    if sprite_main_return_claims != 0 {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    if rom_spiral_second_palette_return_publishes_live_cgram(
                        self.game_state.frame,
                        self.game_state.world.location.dungeon_room_index(),
                        self.game_state.dungeon.stair_movement.staircase_index(),
                    ) {
                        self.publish_completed_palette_filter_cgram_scanout();
                    } else {
                        self.retain_completed_palette_filter_cgram_scanout();
                    }
                    return true;
                }
                let host_main_prefix_did_not_advance =
                    self.pre_main_graphics_dma.as_ref().is_some_and(|graphics| {
                        graphics.entry_frame.frame_counter == self.game_state.frame.frame_counter
                    });
                self.finish_pre_main_caller_continuation(continuation);
                let native_return =
                    !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
                if native_return {
                    // The carried held handler completes before the second
                    // palette walk. The caller's queued uploads belong to the
                    // next Open NMI, after it has returned to the main wait.
                    self.interrupt_nmi(input, oam_dma_source, false);
                }
                self.complete_spiral_stairs_second_palette_filter();
                // Resume the ordinary game-loop suffix without repeating its
                // frame-counter/OAM-clear prefix or the Link movement that ran
                // before the interrupted first palette pass.
                self.complete_module07_dungeon_after_submodule();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                let animated_bg_operands =
                    rom_spiral_stairs_second_palette_return_uses_host_animated_bg_operands(
                        self.game_state.frame,
                        host_main_prefix_did_not_advance,
                        self.game_state.display.bg_tile_animation_countdown,
                        self.game_state.display.animated_tile_data_source_usize(),
                    )
                    .then_some(GraphicsDmaGeneration::HostBoundaryBeforeMain);
                // Resuming a translated caller proves only that the CPU
                // authored a newer Link source. It does not prove that source
                // was the last one DMAed before the active scanout. Resolve
                // the presented generation from the same entry/exit phase
                // model as every other Module 7 boundary instead of treating
                // "resumed" as a second, implicit NMI.
                let link_obj_dma_generation = self.presented_link_obj_dma_generation();
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainResidentPpuOam,
                    link_obj: link_obj_dma_generation,
                    link_obj_sources: link_obj_dma_generation,
                }));
                if crate::debug_env::var_os("ZELDA3_DEBUG_ANIMATED_BG_DMA").is_some() {
                    eprintln!(
                        "animated_bg_second_palette_return host={} countdown={:04x} source={:04x} operands={animated_bg_operands:?}",
                        self.frame_ctr_dbg,
                        self.game_state.display.bg_tile_animation_countdown,
                        self.game_state.display.animated_tile_data_source_usize(),
                    );
                }
                self.capture_display_snapshot();
                if rom_spiral_second_palette_return_publishes_live_cgram(
                    self.game_state.frame,
                    self.game_state.world.location.dungeon_room_index(),
                    self.game_state.dungeon.stair_movement.staircase_index(),
                ) {
                    self.publish_completed_palette_filter_cgram_scanout();
                } else {
                    self.retain_completed_palette_filter_cgram_scanout();
                }
                if native_return {
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                    return true;
                }
                self.interrupt_nmi_with_animated_bg_operands(
                    input,
                    oam_dma_source,
                    false,
                    animated_bg_operands,
                );
            }
            PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter => {
                let live_terminal_timeline =
                    (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.is_some())
                    .then(|| self.take_original_timing_main_loop_return_timeline())
                    .flatten();
                if let Some(timeline) = live_terminal_timeline {
                    // The wire owns this resumed grayscale filter caller: its
                    // carried handler completes first, the second pass and
                    // Module 7 suffix run once, and the shared executor
                    // carries the optional trailing acceptance.
                    let sprite_main_return_claims = self
                        .original_timing_semantic_receipts
                        .as_ref()
                        .map(|receipts| {
                            receipts
                                .semantic()
                                .iter()
                                .filter(|receipt| {
                                    **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    self.finish_pre_main_caller_continuation(continuation);
                    if sprite_main_return_claims != 0 {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            sprite_main_return_claims,
                        );
                    }
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source,
                        |state| {
                            let _ = state.stage_spiral_stairs_second_grayscale_nmi();
                            state.complete_spiral_stairs_second_grayscale_palette_filter();
                            state.complete_module07_dungeon_after_submodule();
                            let link_obj_dma_generation = state.presented_link_obj_dma_generation();
                            state.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                                oam: OamScanoutSource::RetainResidentPpuOam,
                                link_obj: link_obj_dma_generation,
                                link_obj_sources: link_obj_dma_generation,
                            }));
                        },
                    );
                    if sprite_main_return_claims != 0 {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    self.publish_completed_palette_filter_cgram_scanout();
                    return true;
                }
                self.finish_pre_main_caller_continuation(continuation);
                let animated_bg_operands = self.stage_spiral_stairs_second_grayscale_nmi();
                let native_return =
                    !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
                if native_return {
                    // The held NMI resumes this saved palette caller; it does
                    // not publish the palette that the caller has yet to write.
                    // Original host 23945 completes that handler before the
                    // second walk and common suffix. The following Open NMI
                    // is accepted at host return and completes on the next host.
                    self.interrupt_nmi_with_animated_bg_operands(
                        input, oam_dma_source, false, Some(animated_bg_operands),
                    );
                }
                self.complete_spiral_stairs_second_grayscale_palette_filter();
                // The live wire can publish the module tail's Sprite_Main
                // return inside this host and then interrupt NMI_PrepareSprites
                // with no NMI before the host ends (route host 402730, fat
                // stairs: [NmiHandlerCompleted, SpriteMainReturned,
                // CallStackContinued, MainLoopInterrupted(SpritePreparation)]).
                let live = matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.original_timing_semantic_receipts.is_some();
                let nonterminal_sprite_main_returned =
                    live && self.original_timing_owes_sprite_main_return();
                if nonterminal_sprite_main_returned {
                    self.begin_original_timing_sprite_main_return_claim_scope(1);
                }
                self.complete_module07_dungeon_after_submodule();
                if nonterminal_sprite_main_returned {
                    self.finish_original_timing_sprite_main_return_claim_scope();
                }
                if live
                    && self.original_timing_main_loop_interruption()
                        == Some(crate::MainLoopInterruption::SpritePreparation)
                {
                    let _ = self.take_original_timing_main_loop_interruption_any();
                    // No NMI runs inside this host; consume its lifecycle
                    // receipts (the carried handler's completion and the
                    // continued progress) here.
                    let phases = self.take_original_timing_nmi_phases();
                    let before = classify_original_timing_nmi_phases_with_ownership(
                        self.original_timing_nmi_publication_pending,
                        &phases,
                    );
                    self.complete_original_timing_nmi_handler_for_active_scanout(
                        before.handler_completion,
                        input,
                        oam_dma_source,
                    )
                    .assert_no_unclaimed_dialogue_text_dma();
                    assert_eq!(
                        self.take_original_timing_main_loop_progress(),
                        Some(crate::MainLoopProgress::CallStackContinued),
                        "an interrupted grayscale-filter caller host must continue its suspended stack",
                    );
                    self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                        NmiPrepareSpritesCpuCaller::DungeonModule07,
                    );
                    if before.publication_pending_at_exit {
                        self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                    }
                    let link_obj_dma_generation = self.presented_link_obj_dma_generation();
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                        oam: OamScanoutSource::RetainResidentPpuOam,
                        link_obj: link_obj_dma_generation,
                        link_obj_sources: link_obj_dma_generation,
                    }));
                    return true;
                }
                if live && self.original_timing_live_suffix_outstanding() {
                    // The module tail returned Sprite_Main and a second held
                    // acceptance landed before the shared suffix cleared the
                    // latch (route host 786735: [Held, Handler,
                    // SpriteMainReturned, Held, Continued]); the suffix stays
                    // owed to the host that completes it, and this host's
                    // lifecycle receipts are consumed here.
                    if self.pending_main_loop_common_suffix.is_none() {
                        self.pending_main_loop_common_suffix =
                            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
                    }
                    let phases = self.take_original_timing_nmi_phases();
                    let before = classify_original_timing_nmi_phases_with_ownership(
                        self.original_timing_nmi_publication_pending,
                        &phases,
                    );
                    self.complete_original_timing_nmi_handler_for_active_scanout(
                        before.handler_completion,
                        input,
                        oam_dma_source,
                    )
                    .assert_no_unclaimed_dialogue_text_dma();
                    assert_eq!(
                        self.take_original_timing_main_loop_progress(),
                        Some(crate::MainLoopProgress::CallStackContinued),
                        "a suffix-deferred grayscale-filter caller host must continue its suspended stack",
                    );
                    if before.publication_pending_at_exit {
                        self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                    }
                    let link_obj_dma_generation = self.presented_link_obj_dma_generation();
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                        oam: OamScanoutSource::RetainResidentPpuOam,
                        link_obj: link_obj_dma_generation,
                        link_obj_sources: link_obj_dma_generation,
                    }));
                    return true;
                }
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                // Publish the generation whose DMA completed before this
                // scanout. The resumed CPU suffix may already have authored
                // the following source words; the NMI path resolves those
                // future operands independently.
                let link_obj_dma_generation = self.presented_link_obj_dma_generation();
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainResidentPpuOam,
                    link_obj: link_obj_dma_generation,
                    link_obj_sources: link_obj_dma_generation,
                }));
                self.capture_display_snapshot();
                self.publish_completed_palette_filter_cgram_scanout();
                if native_return {
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                    return true;
                }
                self.interrupt_nmi_with_animated_bg_operands(
                    input,
                    oam_dma_source,
                    false,
                    Some(animated_bg_operands),
                );
            }
        }
        true
    }

    /// Resume game-thread work whose caller returns through a leading NMI.
    ///
    /// The continuation owns the display generations at that boundary. A
    /// `true` result consumes the host frame even when the continuation asks
    /// to remain scheduled for another module iteration.
    pub(super) fn resume_after_pre_main_nmi(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        let Some(resume) = self.game_execution_scheduler.take_pre_main_nmi_resume() else {
            return false;
        };
        let (quadrant_cpu_advance, quadrant_module_cpu_advance) =
            dungeon_supertile_quadrant_cpu_advance_for_resume(resume, self);
        let live_terminal_timeline =
            (matches!(
                resume,
                PreMainNmiResume::DungeonSupertileQuadrantUploads
                    | PreMainNmiResume::DungeonSupertileQuadrantUploadsAfterHeldNmi
                    | PreMainNmiResume::DungeonSupertileNextIterationAfterLeadingNmi
                    | PreMainNmiResume::DungeonSupertileCallerReturnNmi
                    | PreMainNmiResume::OverworldSpriteReloadReturn { .. }
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some())
            .then(|| self.take_original_timing_main_loop_return_timeline())
            .flatten();
        if let Some(timeline) = live_terminal_timeline {
            let caller_return_resume = resume == PreMainNmiResume::DungeonSupertileCallerReturnNmi
                // The reload-return caller completed on the preceding host;
                // this resume owns only the carried handler and the deferred
                // shared suffix (route host 117639).
                || matches!(resume, PreMainNmiResume::OverworldSpriteReloadReturn { .. });
            if caller_return_resume {
                // The carried handler interrupted NMI_PrepareSprites before
                // its final latch-clear store; restore that call-stack-local
                // latch before the handler completes (route host 31363).
                self.latch_nmi_update();
            }
            // The wire owns this resumed quadrant caller: its carried handler
            // completes first, the suspended module body and Sprite_Main run
            // once, and the shared executor retires the suffix and carries an
            // optional trailing acceptance. The estimate's own leading NMI
            // must not run.
            let sprite_main_return_claims = self
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    receipts
                        .semantic()
                        .iter()
                        .filter(|receipt| {
                            **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                        })
                        .count()
                })
                .unwrap_or(0);
            if sprite_main_return_claims != 0 {
                self.begin_original_timing_sprite_main_return_claim_scope(
                    sprite_main_return_claims,
                );
            }
            self.complete_original_timing_main_loop_return(
                timeline,
                input,
                oam_dma_source,
                |state| {
                    if caller_return_resume {
                        // The interrupted caller only finishes its shared
                        // NMI_PrepareSprites suffix, which the executor's
                        // suffix retirement owns; no new iteration runs.
                        return;
                    }
                    if let Some(advance) = quadrant_module_cpu_advance {
                        debug_assert!(state.dungeon_landing_cpu_advance_pending.is_none());
                        state.dungeon_landing_spotlight_reset_prefix_scanlines = None;
                        state.dungeon_landing_spotlight_copy_visible_rows = None;
                        state.dungeon_landing_cpu_advance_pending = Some(advance);
                    }
                    state.zelda_run_game_loop_after_leading_nmi();
                },
            );
            if sprite_main_return_claims != 0 {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            if caller_return_resume {
                // The caller then returns and clears the latch; the following
                // host boundary starts the next dungeon state.
                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                );
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        if resume == PreMainNmiResume::DungeonSupertileCallerReturnNmi {
            // This continuation resumes at the hardware boundary that
            // interrupted NMI_PrepareSprites, before its final latch-clear
            // store. The translated loop cannot retain that call-stack-local
            // position between host calls, so restore the pending latch before
            // executing Interrupt_NMI.
            self.latch_nmi_update();
        }

        let quadrant_build_or_upload_uses_published_oam = matches!(
            resume,
            PreMainNmiResume::DungeonSupertileQuadrantUploads
                | PreMainNmiResume::DungeonSupertileQuadrantUploadsAfterHeldNmi
        ) && self.game_state.frame.main_module
            == 7
            && self.game_state.frame.submodule == 2
            && self.game_state.frame.subsubmodule == 5;
        let mut scanout = resume.scanout_generations();
        if quadrant_build_or_upload_uses_published_oam {
            // The leading NMI publishes the completed shadow DMA while the
            // room-quadrant build/upload caller remains suspended. Live PPU
            // OAM has already moved on, while the prior presented image is one
            // step older; select the newly published generation between those
            // two boundaries.
            if let Some(obj) = scanout.obj.as_mut() {
                obj.oam = OamScanoutSource::ComposePublishedShadowDma;
            }
        }
        self.next_display_vram_generation = scanout.vram;
        self.next_display_animated_bg_scanout_generation = scanout.animated_bg;
        self.next_display_bg_scroll_generation = scanout.bg_scroll;
        self.set_next_display_obj_scanout(scanout.obj);
        if resume.nmi_latch_clear_phase() == Some(NmiPhase::BeforeNmi) {
            // The suspended quadrant caller returns with the software latch
            // still set, but the ROM reaches this leading hardware NMI before
            // the next module iteration can replace its upload request.
            self.clear_nmi_update_latch();
        }
        self.capture_display_snapshot_with_override(Some(scanout.publication));
        if matches!(
            resume,
            PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi
        ) {
            self.retain_completed_palette_filter_cgram_scanout();
        }
        self.interrupt_nmi_for_active_scanout_without_dialogue_owner(input, oam_dma_source, false);
        if let Some(continuation) = self.game_execution_scheduler.take_after_pre_main_nmi() {
            // The oracle reported CPU progress interrupted by this same NMI.
            // Resume that saved source stack after the interrupt instead of
            // scheduling a second synthetic boundary.
            self.complete_post_trailing_nmi_continuation(continuation, input, false, false);
            if self
                .game_execution_scheduler
                .resumed_call_stack_is_before_nmi()
                && !self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
            {
                self.game_execution_scheduler
                    .finish_call_stack_at_main_wait_before_nmi();
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        if resume == PreMainNmiResume::DungeonSupertileCallerReturnNmi {
            // The state body completed before vblank, but hardware interrupted
            // its common Module 7 caller while NMI_PrepareSprites still owned
            // the set NMI latch. Interrupt_NMI therefore skips NMI_DoUpdates at
            // this boundary. The caller then returns and clears the latch; the
            // following host boundary starts the next dungeon state.
            debug_assert_eq!(resume.nmi_latch_clear_phase(), Some(NmiPhase::AfterNmi));
            if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.pending_main_loop_common_suffix.is_some()
            {
                self.complete_pending_main_loop_common_suffix_after_module_return();
            }
            self.clear_nmi_update_latch();
            self.game_execution_scheduler
                .schedule_pre_main_nmi_resume(PreMainNmiResume::DungeonSupertileQuadrantUploads);
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        let quadrant_has_modeled_cached_sprite_continuation = quadrant_cpu_advance
            == Some(DungeonQuadrantCpuAdvance::InterruptedInModule)
            && if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
                self.original_timing_cached_sprite_execution_progress()
                    .is_some()
            } else {
                quadrant_module_cpu_advance
                    .is_some_and(|advance| advance.cached_sprite_interruption.is_some())
            };
        if quadrant_cpu_advance == Some(DungeonQuadrantCpuAdvance::InterruptedInModule)
            && !quadrant_has_modeled_cached_sprite_continuation
        {
            // These measured quadrant-upload boundaries consume one full
            // vblank before an otherwise atomic module iteration resumes.
            // Keep the caller continuation explicit so the following host
            // NMI can run main and rejoin the ordinary upload chain. A module
            // with its own semantic continuation instead runs below and
            // suspends at that exact boundary.
            self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                PreMainNmiResume::DungeonSupertileQuadrantUploadsAfterHeldNmi,
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        if resume == PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi {
            // The caller suffix completed on the preceding host boundary, but
            // the next fresh module iteration remains behind this leading
            // NMI. Consume the hardware boundary without running a second
            // translated module iteration.
            // Unlike the quadrant continuations above, this boundary consumes
            // an already-latched request; clear it only after the NMI has
            // observed it.
            debug_assert_eq!(resume.nmi_latch_clear_phase(), Some(NmiPhase::AfterNmi));
            self.clear_nmi_update_latch();
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        if let Some(advance) = quadrant_module_cpu_advance {
            debug_assert!(self.dungeon_landing_cpu_advance_pending.is_none());
            self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
            self.dungeon_landing_spotlight_copy_visible_rows = None;
            self.dungeon_landing_cpu_advance_pending = Some(advance);
        }
        self.replay_trace_ram_watch("before-game-loop");
        let resumed_main_entry = self.game_state.frame;
        if resume == PreMainNmiResume::DungeonSupertileQuadrantUploads
            && resumed_main_entry.main_module == 7
            && resumed_main_entry.submodule == 2
            && resumed_main_entry.subsubmodule == 13
            && self.game_state.world.location.dungeon_room_index() == 0x41
        {
            self.dungeon_state_13_pre_main_publication_host_frame = Some(self.frame_ctr_dbg);
        }
        self.zelda_run_game_loop_after_leading_nmi();
        self.replay_trace_ram_watch("after-game-loop");
        if quadrant_cpu_advance == Some(DungeonQuadrantCpuAdvance::InterruptedAfterModule) {
            // The coarse translated loop ran NMI_PrepareSprites and its final
            // `nmi_boolean = 0` store. The ROM CPU timing run proves hardware
            // interrupted that caller before the store, so keep the latch set
            // until DungeonSupertileCallerReturnNmi resumes the caller suffix.
            self.latch_nmi_update();
        }
        // The leading NMI already sampled the audio ports. Commands authored
        // by this fresh iteration remain queued until the following NMI;
        // original host 23932 completes the handler before spiral state 7.

        if resume.continues_after_main(self.game_state.frame)
            && self.game_execution_scheduler.is_idle()
        {
            let next_resume = match (quadrant_cpu_advance, resume) {
                (
                    Some(DungeonQuadrantCpuAdvance::InterruptedAfterModule),
                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                ) => PreMainNmiResume::DungeonSupertileCallerReturnNmi,
                _ => PreMainNmiResume::DungeonSupertileQuadrantUploads,
            };
            self.game_execution_scheduler
                .schedule_pre_main_nmi_resume(next_resume);
        }
        if resume == PreMainNmiResume::OverworldAuxGraphicsReturn {
            // The main slice authors the following upload. Keep it private
            // until the next NMI, while retaining the hardware OAM that was
            // active when the transition began.
            self.next_display_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
            self.next_display_bg_scroll_generation = DisplayBgScrollGeneration::ComposeLiveAfterNmi;
            let oam = self
                .display_snapshot
                .as_ref()
                .map(|snapshot| snapshot.ppu.oam.clone())
                .unwrap_or_else(|| self.ppu.oam.clone());
            let vram = self
                .display_snapshot
                .as_ref()
                .map(|snapshot| snapshot.ppu.vram[0x4000..0x4400].to_vec())
                .unwrap_or_else(|| self.ppu.vram[0x4000..0x4400].to_vec());
            self.active_display_obj_generation =
                DisplayObjGeneration::RetainCapturedMemory { oam, vram };
        }
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        true
    }

    /// Finish the source caller below an interrupted Module09 `Sprite_Main`.
    /// Returns true when that caller is itself the suspended overworld reload
    /// tail and therefore owns the next NMI and host return completely.
    pub(super) fn complete_resumed_module09_sprite_main_caller(
        &mut self,
        caller: Module09ItemReceiptCallerReturn,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        self.complete_resumed_module09_sprite_main_caller_with_wire(
            caller,
            input,
            oam_dma_source,
            false,
        )
    }

    pub(super) fn complete_resumed_module09_sprite_main_caller_with_wire(
        &mut self,
        caller: Module09ItemReceiptCallerReturn,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        wire_owns_return: bool,
    ) -> bool {
        self.complete_module09_overworld_after_resumed_sprite_main(caller);
        let Module09AfterSpriteMain::FinishOverworldSpriteReload {
            post_return_hold_nmi_slices,
            epilogue_phase,
            resume_scanout,
        } = caller.after_sprite_main
        else {
            return false;
        };
        self.finish_overworld_sprite_reload_return(
            post_return_hold_nmi_slices,
            epilogue_phase,
            resume_scanout,
            input,
            oam_dma_source,
            wire_owns_return,
        );
        true
    }

    /// Finish a resumed Module07 caller's suffix under the live wire: a
    /// forwarded LinkOam interruption suspends after Sprite_Main, the wire's
    /// Sprite_Main return in this host is claimed by the caller's own
    /// Sprite_Main, and the shared suffix is retired by a terminal return or
    /// deferred while the wire holds it (route hosts 307614, 307634).
    pub(super) fn complete_resumed_module07_caller_suffix_by_wire(
        &mut self,
        terminal_return: bool,
        equipment_prefix: Option<crate::LinkOamStairProgress>,
    ) {
        if self
            .take_forwarded_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::LinkOam,
            )
            .is_some()
        {
            self.dungeon_post_sprite_main_return_pending = true;
        }
        let continued_sprite_main_claims = if terminal_return {
            0
        } else {
            self.original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    receipts
                        .semantic()
                        .iter()
                        .filter(|receipt| {
                            **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                        })
                        .count()
                })
                .unwrap_or(0)
        };
        if continued_sprite_main_claims != 0 {
            self.begin_original_timing_sprite_main_return_claim_scope(continued_sprite_main_claims);
        }
        self.complete_module07_dungeon_after_submodule();
        if let Some(progress) = equipment_prefix {
            assert!(
                !terminal_return,
                "LinkOam prefix cannot own a terminal caller return"
            );
            assert_eq!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
            );
            let mut sprite_return = self
                .active_dungeon_sprite_main_return
                .take()
                .expect("LinkOam prefix requires its suspended dungeon caller");
            assert!(
                sprite_return.link_oam.is_none(),
                "LinkOam prefix was already executed"
            );
            self.set_bg2_x(sprite_return.bg2_x);
            self.set_bg2_y(sprite_return.bg2_y);
            self.set_bg1_x(sprite_return.bg1_x);
            self.set_bg1_y(sprite_return.bg1_y);
            sprite_return.link_oam = Some(match progress {
                crate::LinkOamStairProgress::PoseSelected => self.link_oam_after_pose_selection(),
                crate::LinkOamStairProgress::EquipmentSelection => self.link_oam_before_equipment(),
                crate::LinkOamStairProgress::BodySelection => {
                    let continuation = self.link_oam_before_equipment();
                    self.link_oam_before_body(continuation)
                }
                crate::LinkOamStairProgress::ShadowSelection => {
                    let continuation = self.link_oam_before_equipment();
                    self.link_oam_before_shadow(continuation)
                }
            });
            self.active_dungeon_sprite_main_return = Some(sprite_return);
        }
        if continued_sprite_main_claims != 0
            && self
                .original_timing_sprite_main_return_claims_remaining
                .is_some()
        {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // The post-Sprite_Main caller return resumes next host.
        } else if terminal_return
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        } else {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        }
    }

    pub(super) fn publish_cached_sprite_restore_before_acceptance(&mut self) {
        let Some(
            work @ GameWorkContinuation::FinishDungeonCachedSpriteMain {
                boundary: CachedSpriteCpuInterruption::Restoring { slot, live_fields },
                live_slot_backup,
                dungeon,
            },
        ) = self.game_execution_scheduler.current_work()
        else {
            return;
        };
        let Some(receipt) = self.original_timing_cached_sprite_execution_progress() else {
            return;
        };
        if receipt.boundary != OriginalTimingBoundary::NmiAccepted {
            return;
        }
        let CachedSpriteCpuInterruption::Restoring {
            slot: next_slot,
            live_fields: next_fields,
        } = receipt.progress.into()
        else {
            panic!("a cached restore cannot return to its load or execute phase");
        };
        assert_eq!(
            slot, next_slot,
            "a cached restore cannot change slots before its NMI"
        );
        self.cached_sprite_slot_mut(usize::from(slot))
            .restore_live_range_from_backup(
                &live_slot_backup,
                usize::from(live_fields),
                usize::from(next_fields),
            );
        self.game_execution_scheduler.refine_scheduled_work(
            work,
            GameWorkContinuation::FinishDungeonCachedSpriteMain {
                boundary: CachedSpriteCpuInterruption::Restoring {
                    slot,
                    live_fields: next_fields,
                },
                live_slot_backup,
                dungeon,
            },
        );
    }

    pub(super) fn restore_link_safe_return_position(&mut self) {
        self.follower_link_state_mut()
            .restore_position_from_safe_return();
    }
}
