//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (cpu_schedule).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(super) fn take_dungeon_landing_cpu_advance(&mut self) -> Option<DungeonModuleCpuAdvance> {
        let advance = self.dungeon_landing_cpu_advance_pending.take();
        self.active_dungeon_landing_spotlight_copy_visible_rows =
            self.dungeon_landing_spotlight_copy_visible_rows.take();
        self.active_dungeon_landing_spotlight_reset_prefix_scanlines =
            self.dungeon_landing_spotlight_reset_prefix_scanlines.take();
        advance
    }

    pub(super) fn take_dungeon_cached_sprite_cpu_interruption(
        &mut self,
    ) -> Option<(CachedSpriteCpuInterruption, Option<OriginalTimingBoundary>)> {
        self.dungeon_cached_sprite_cpu_interruption_pending
            .take()
            .map(|interruption| {
                (
                    interruption,
                    self.dungeon_cached_sprite_cpu_interruption_boundary.take(),
                )
            })
    }

    pub(super) fn capture_cpu_schedules_before_nmi(&mut self, nmi_is_trailing: bool, input: u16) {
        let frame = self.game_state.frame;
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 8 && frame.submodule == 2
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.pre_overworld_screen_build_cpu_nmis.is_none()
        {
            self.pre_overworld_screen_build_cpu_nmis = Some(pre_overworld_load_cpu_nmis(self, None));
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 0x0f && frame.submodule == 0
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
        {
            let entry = module_cpu_entry_after_leading_nmi(self, input,
                DUNGEON_EXIT_SPOTLIGHT_CPU_CHECKPOINT.entry_pc);
            self.dungeon_exit_spotlight_cpu_entry_envelope = Some((entry, entry));
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 9 && matches!(frame.submodule, 3 | 17)
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.native_overworld_map_graphics_nmi_slices.is_none()
        {
            self.native_overworld_map_graphics_nmi_slices =
                Some(overworld_map_graphics_cpu_nmi_slices(self, input));
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 9 && frame.submodule == 0
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.native_overworld_packing_progress.is_none()
        {
            self.native_overworld_packing_progress = overworld_main_loop_packing_interruption(self, input);
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 8 && frame.submodule == 0
            && self.resident_song_bank_is_dungeon()
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.native_overworld_song_upload.is_none()
        {
            self.native_overworld_song_upload =
                Some(pre_overworld_song_upload_command(self, nmi_is_trailing));
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 7
            && frame.submodule == 2
            && matches!(frame.subsubmodule, 2 | 13 | 14)
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.dungeon_landing_cpu_advance_pending.is_none()
        {
            // Capture the input to the leading handler. Measuring at the
            // subsequent game-loop entry would replay NMI from its already
            // mutated latch/DMA state and omit the real upload workload.
            let timing = rom_dungeon_landing_cpu_advance(self);
            self.dungeon_landing_spotlight_copy_visible_rows = timing.spotlight_copy_visible_rows;
            self.dungeon_landing_spotlight_reset_prefix_scanlines =
                timing.spotlight_reset_prefix_scanlines;
            self.dungeon_landing_cpu_advance_pending = Some(timing.advance);
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 14
            && frame.submodule == 3
            && self.overworld_map_state() == 1
            && self.game_state.dungeon_map_display.dungmap_init_state() == 3
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.pending_dungeon_map_room_drawing_nmi_slices.is_none()
        {
            self.pending_dungeon_map_room_drawing_nmi_slices =
                Some(dungeon_map_room_drawing_cpu_nmi_slices(self));
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 14 && frame.submodule == 3
            && self.overworld_map_state() == 0
            && self.game_state.display.screen_brightness == 1
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.pending_dungeon_map_force_blank_output_scanline.is_none()
        {
            self.pending_dungeon_map_force_blank_output_scanline =
                Some(dungeon_map_backup_cpu_blank_scanline(self));
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 7
            && matches!(frame.submodule, 0x11 | 0x12)
            && frame.subsubmodule == 4
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.dungeon_submodule_cpu_schedule.is_none()
        {
            self.dungeon_submodule_cpu_schedule = Some(DungeonSubmoduleCpuSchedule {
                reset_progress: straight_interroom_reset_cpu_progress(self),
                ..Default::default()
            });
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 7
            && matches!(frame.submodule, 0x11 | 0x12)
            && frame.subsubmodule == 1
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.dungeon_landing_cpu_advance_pending.is_none()
        {
            // Measure the next fadeout iteration continuously from main wait,
            // including this Open NMI. A palette-count list cannot identify
            // whether the caller stops in Sprite_Main, LinkOam, or sprite prep.
            self.dungeon_landing_cpu_advance_pending = Some(
                dungeon_module_7_cpu_advance_after_leading_nmi(self, DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT),
            );
        }
        if self.rom_startup_timing()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && frame.main_module == 8 && frame.submodule == 1
            && !self.game_state.display.nmi_update_is_latched()
            && self.game_execution_scheduler.is_idle()
            && self.pre_overworld_overlays_cpu_nmis.is_none()
        {
            self.pre_overworld_overlays_cpu_nmis = Some(pre_overworld_overlays_cpu_nmis(self));
        }
        if self.rom_startup_timing()
            && frame.main_module == 9
            && matches!(frame.submodule, 0x20 | 0x21)
            && self.game_execution_scheduler.is_idle()
            && self.module09_cpu_schedule.is_none()
        {
            self.module09_cpu_schedule = Some(module09_cpu_schedule(self));
        }
        if self.rom_startup_timing()
            && frame.main_module == 7
            && frame.submodule == 2
            && frame.subsubmodule == 12
            && self.dungeon_landing_cpu_advance_pending.is_none()
        {
            let advance = begin_dungeon_supertile_state_12_cpu_advance(self);
            self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
            self.dungeon_landing_spotlight_copy_visible_rows = None;
            self.dungeon_landing_cpu_advance_pending = Some(advance);
        }
        if self.rom_startup_timing()
            && frame.main_module == 7
            && ((frame.submodule == 0x0e && matches!(frame.subsubmodule, 3 | 6 | 7 | 19))
                || (frame.submodule == 2 && frame.subsubmodule == 3))
            && self.game_execution_scheduler.is_idle()
            && self.dungeon_submodule_cpu_schedule.is_none()
        {
            let schedule = dungeon_submodule_cpu_schedule(self);
            if crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
                eprintln!(
                    "dungeon_cpu_schedule host={} module={:02x}/{:02x}/{:02x} submodule_nmis={} caller_nmis={} sprite_main_nmis={} suffix_nmis={} caller_phase={:?} sprite_boundary={:?} cached_boundary={:?} reenters_main={}",
                    self.frame_ctr_dbg,
                    frame.main_module,
                    frame.submodule,
                    frame.subsubmodule,
                    schedule.submodule_nmis,
                    schedule.caller_nmis,
                    schedule.caller_sprite_main_nmis,
                    schedule.caller_suffix_nmis,
                    schedule.caller_first_nmi_phase,
                    schedule.sprite_main_boundary,
                    schedule.cached_sprite_interruption,
                    schedule.reenters_main_loop_before_nmi,
                );
            }
            self.dungeon_submodule_cpu_schedule = Some(schedule);
        }
    }

    pub(super) fn suspend_dungeon_subtile_palette_filter_if_cpu_interrupted(&mut self) {
        let Some(advance) = self.dungeon_palette_cpu_advance_pending.take() else {
            return;
        };
        debug_assert_eq!(
            advance.subsubmodule, self.game_state.frame.subsubmodule,
            "subtile palette ROM shadow and translated body disagree at PC {:06x}",
            advance.pc,
        );
        debug_assert_eq!(
            advance.palette_countdown,
            self.game_state.display.palette_filter.countdown(),
            "subtile palette ROM shadow and translated palette disagree at PC {:06x}",
            advance.pc,
        );
        if advance.work.reached_boundary().is_some() {
            // Every typed main-loop interruption is after the Module 7
            // submodule returned. If the continuous source owner reached
            // Sprite_Main, LinkOam, or sprite preparation in this same host,
            // the native CPU estimate's earlier palette-return suspension is
            // obsolete. Keep the later receipt on the bus for its actual
            // translated caller; only suppress this superseded timing shadow.
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_main_loop_interruption().is_some()
            {
                return;
            }
            if self.original_timing_owes_sprite_main_return()
                || self.original_timing_owes_sprite_main_progress()
            {
                // The live wire still owes this host its Sprite_Main return
                // claim — or a mid-body checkpoint proving the caller runs
                // into Sprite_Main before the host ends — so the caller
                // cannot be suspended before that body runs; the estimate's
                // vblank crossing is superseded (route hosts 32372, 85627).
                return;
            }
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonSubtilePaletteFilter,
                DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES,
            );
        }
    }

    pub(super) fn suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(
        &mut self,
        palette_countdown: u8,
    ) {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            // The native fadeout caller consumes the measured CPU phase at
            // its submodule return, before running the shared sprite suffix.
            return;
        }
        let frame = self.game_state.frame;
        if self.rom_startup_timing()
            && straight_interroom_fadeout_suffix_crosses_vblank(
                frame.main_module,
                frame.submodule,
                frame.subsubmodule,
                self.game_state.world.location.dungeon_room_index(),
                self.game_state.dungeon.stair_movement.staircase_index(),
                palette_countdown,
            )
        {
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        receipts.semantic().iter().any(|receipt| {
                            *receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                        })
                    })
            {
                // The live wire still owes this host its Sprite_Main return
                // claim, so the Module 7 caller cannot be suspended before
                // that body runs; the estimate's vblank crossing is
                // superseded (route host 25863).
                return;
            }
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishStraightInterroomFadeoutSuffix,
                DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES,
            );
        }
    }

    pub(super) fn suspend_spiral_staircase_palette_filter(
        &mut self,
        tail: SpiralStaircasePaletteTail,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        // The interrupted palette walk keeps NMI_DoUpdates gated until the
        // translated caller returns. In particular, the animated-BG DMA must
        // not consume the source advanced by the suspended spiral-staircase
        // main slice.
        self.set_core_update_disable_flag(1);
        let caller =
            InterruptedPaletteFilterCaller::from_dungeon_submodule(self.game_state.frame.submodule);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpiralStaircasePaletteFilter { tail, caller },
            DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES,
        );
        true
    }

    pub(super) fn suspend_straight_interroom_sprite_reset_before_room_load(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        let (progress, boundary) = if matches!(
            self.original_timing_owner, OriginalTimingOwnerState::Live
        ) {
            let Some(receipt) = self.take_original_timing_dungeon_reset_sprites_progress() else {
                return false;
            };
            (receipt.progress, receipt.boundary)
        } else {
            let Some(progress) = self.dungeon_submodule_cpu_schedule.take()
                .and_then(|plan| plan.reset_progress) else {
                return false;
            };
            (progress, OriginalTimingBoundary::HostReturn)
        };

        // Both timing owners identify a source statement. Apply that prefix
        // and carry its typed token into the existing continuation; room and
        // staircase identities do not select the native interruption.
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.dungeon_reset_sprites_through_cpu_progress(progress);
        let continuation = GameWorkContinuation::FinishStraightInterroomSpriteReset { progress };
        match boundary {
            OriginalTimingBoundary::HostReturn => {
                self.game_execution_scheduler.schedule_work(continuation, 1);
            }
            OriginalTimingBoundary::NmiAccepted => self
                .game_execution_scheduler
                .schedule_after_current_trailing_nmi(continuation),
        }
        true
    }

    pub(super) fn schedule_dungeon_faded_filter_completion_nmi(&mut self) {
        debug_assert!(self.rom_startup_timing());
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        debug_assert_eq!(self.game_state.frame.submodule, 2);
        debug_assert_eq!(self.game_state.frame.subsubmodule, 3);
        debug_assert_eq!(self.game_state.display.palette_filter.countdown(), 0);
        self.stage_dungeon_faded_filter_completion_scanout();
        self.schedule_dungeon_faded_filter_leading_nmi_resume();
    }

    pub(crate) fn sprite_workspace_mut(&mut self) -> NativeSpriteWorkspaceBridgeMut<'_> {
        NativeSpriteWorkspaceBridgeMut::new(
            &mut self.game_state.sprites.workspace,
            &mut self.game_state.sprites.overworld_sprite_presence,
            &mut self.ram,
        )
    }

    pub(super) fn begin_pre_overworld_properties_work(
        &mut self,
        overworld_screen: u8,
        animated_tiles: u8,
    ) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::PreOverworldPropertiesSpriteReset {
                overworld_screen,
                animated_tiles,
            },
            PRE_OVERWORLD_PROPERTIES_TO_SPRITE_RESET_NMI_SLICES,
        );
        true
    }

    /// Whether the selected-screen load parked its caller on this host.
    pub(super) fn flute_menu_selected_screen_scheduled(&self) -> bool {
        matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishFluteMenuSelectedScreen { .. })
        )
    }

    pub(super) fn begin_pre_overworld_overlays_work(&mut self) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        let nmi_slices = self.pre_overworld_overlays_cpu_nmis.take()
            .unwrap_or(PRE_OVERWORLD_OVERLAYS_NMI_SLICES);
        if nmi_slices == 0 { return false; }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreOverworldOverlays,
            nmi_slices,
        );
        true
    }

    pub(super) fn begin_pre_overworld_screen_build_work(&mut self) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        if let Some(nmis) = self.pre_overworld_screen_build_cpu_nmis.take() {
            assert_ne!(nmis, 0, "screen build must cross its measured NMI");
            // The measurement excludes the leading handler and counts every
            // held acceptance up to the caller return. Its final handler is
            // consumed by the completion lane, not prepaid on module entry.
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishPreOverworldScreenBuild, nmis,
            );
            return true;
        }
        let timing =
            overworld_map_and_sprite_graphics_timing(self.overworld_map_graphics_workload());
        self.game_execution_scheduler
            .schedule_cpu_timed_work_from_current_main_iteration(
                GameWorkContinuation::FinishPreOverworldScreenBuild,
                timing.quadrant_load_nmi_slices + timing.map16_to_map8_tail_nmi_slices,
            );
        true
    }

    #[doc(hidden)]
    pub fn zelda_debug_selected_game_load_remaining_nmi_slices(&self) -> u8 {
        self.game_execution_scheduler
            .selected_game_load_remaining_nmi_slices()
    }

    #[doc(hidden)]
    pub fn zelda_debug_game_execution_scheduler(&self) -> String {
        format!("{:?}", self.game_execution_scheduler)
    }

    pub(super) fn begin_big_key_drop_graphics_work(&mut self, sprite_slot: usize) -> bool {
        if !self.rom_startup_timing() || !self.game_execution_scheduler.is_idle() {
            return false;
        }
        let Some(dungeon) = self.active_dungeon_sprite_main_return.take() else {
            // Big-key graphics can also be prepared by atomic setup paths.
            // Only a live Module 7 sprite loop has a measured resumable caller.
            return false;
        };
        let active_scanout_scroll = BgScrollRegisterScanout::capture(&self.ppu);
        let display_already_captured = self.game_execution_scheduler
            .current_main_iteration_follows_leading_nmi();
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishBigKeyDropGraphics {
                sprite_slot: sprite_slot as u8,
                dungeon,
            },
            BIG_KEY_DROP_GRAPHICS_NMI_SLICES,
        );
        // C reaches SpritePrep_BigKey_load_graphics after the leading NMI has
        // installed this field's scroll registers. The synchronous graphics
        // call is interrupted before the following handler reaches
        // WritePpuRegisters. Carry the A4 register generation into the
        // imminent snapshot; the A2 software copies and trailing register
        // receipt belong to the following field.
        let scroll_generation = DisplayBgScrollGeneration::RetainCpuSliceEntry(active_scanout_scroll);
        if display_already_captured {
            // A leading-NMI iteration already captured this host's display.
            // Attach the entry selection to that field, not the next capture
            // after the carried handler updates scroll (original 20201-20202).
            self.display_snapshot.as_mut()
                .expect("leading-NMI graphics entry requires its active display")
                .bg_scroll_generation = scroll_generation;
        } else {
            self.next_display_bg_scroll_generation = scroll_generation;
        }
        // The entry main slice has already crossed the OAM DMA that published
        // the host-boundary shadow, but it will not reach another sprite-prep
        // epilogue before the decompressor is interrupted. Publish that prior
        // shadow once; subsequent waiting slices retain the resident table.
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
        true
    }

    pub(super) fn begin_dungeon_falling_entrance_work(
        &mut self,
        work: DungeonFallingEntranceWork,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 0x11);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonFallingEntrance { work },
            work.nmi_slices(),
        );
        true
    }

    pub(super) fn begin_dungeon_supertile_transition_work(
        &mut self,
        work: DungeonSupertileTransitionWork,
    ) -> bool {
        self.begin_dungeon_supertile_transition_work_with_palette(work, None)
    }

    pub(super) fn begin_dungeon_supertile_transition_work_with_palette(
        &mut self,
        work: DungeonSupertileTransitionWork,
        palette_filter_loop_master_cycles: Option<u32>,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        if work == DungeonSupertileTransitionWork::SpiralSpriteGraphics {
            // The sprite-graphics reload lane is shared by the spiral-stairs
            // ($0e) and warp-pad ($15) transitions.
            debug_assert!(matches!(self.game_state.frame.submodule, 0x0e | 0x15));
        } else if work == DungeonSupertileTransitionWork::FallingSpriteGraphics {
            debug_assert!(matches!(self.game_state.frame.submodule, 6 | 7));
            debug_assert_eq!(self.game_state.frame.subsubmodule, 5);
        } else if matches!(
            work,
            DungeonSupertileTransitionWork::SpiralRoomInitialization
                | DungeonSupertileTransitionWork::SpiralBackgroundSync
                | DungeonSupertileTransitionWork::SpiralRoomCallerResume
                | DungeonSupertileTransitionWork::SpiralBgCharacters34
        ) {
            debug_assert_eq!(self.game_state.frame.submodule, 0x0e);
        } else if work == DungeonSupertileTransitionWork::FallingBgCharacters34 {
            debug_assert!(matches!(self.game_state.frame.submodule, 6 | 7));
            debug_assert_eq!(self.game_state.frame.subsubmodule, 3);
        } else if matches!(
            work,
            DungeonSupertileTransitionWork::StraightInterroomRoomInitialization
                | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics
        ) {
            debug_assert_eq!(self.game_state.frame.submodule, 0x12);
        } else if work == DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild {
            debug_assert_eq!(self.game_state.frame.submodule, 2);
            debug_assert_eq!(self.game_state.frame.subsubmodule, 10);
            if !matches!(
                dungeon_supertile_state_10_cpu_advance(
                    palette_filter_loop_master_cycles
                        .expect("filtered state 10 requires its palette-loop workload"),
                ),
                CpuPhaseSequenceAdvance::ReachedBoundary {
                    boundary: CpuRasterBoundary::VblankPublication,
                    phase_index: 0,
                    ..
                }
            ) {
                return false;
            }
        } else if work == DungeonSupertileTransitionWork::QuadrantUploadCallerReturn {
            debug_assert_eq!(self.game_state.frame.submodule, 2);
            debug_assert_eq!(self.game_state.frame.subsubmodule, 11);
            if !matches!(
                dungeon_supertile_state_11_cpu_advance(
                    palette_filter_loop_master_cycles
                        .expect("filtered state 11 requires its palette-loop workload"),
                ),
                CpuPhaseSequenceAdvance::ReachedBoundary {
                    boundary: CpuRasterBoundary::VblankPublication,
                    phase_index: 0,
                    ..
                }
            ) {
                return false;
            }
        } else if work == DungeonSupertileTransitionWork::State13CallerReturn {
            debug_assert_eq!(self.game_state.frame.submodule, 2);
            // Dungeon_IntraRoomTrans_State5 may advance 13 -> 14 before the
            // measured caller suffix crosses vblank. The ROM continuation is
            // captured at dispatcher entry, while this scheduler receives the
            // post-call state.
            debug_assert!(matches!(self.game_state.frame.subsubmodule, 13 | 14));
        } else if matches!(
            work,
            DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn
                | DungeonSupertileTransitionWork::FadedFilterCallerReturn
        ) {
            debug_assert_eq!(self.game_state.frame.submodule, 2);
            debug_assert_eq!(
                self.game_state.frame.subsubmodule,
                if work == DungeonSupertileTransitionWork::FadedFilterCallerReturn {
                    15
                } else {
                    14
                }
            );
        } else {
            debug_assert_eq!(self.game_state.frame.submodule, 2);
        }
        let mut nmi_slices = work.nmi_slices();
        if let Some(schedule) = self.dungeon_room_load_cpu_schedule {
            nmi_slices = match work {
                DungeonSupertileTransitionWork::RoomLoad => schedule.room_load_nmis,
                DungeonSupertileTransitionWork::AuxiliarySpriteGraphics => {
                    schedule.auxiliary_graphics_nmis
                }
                DungeonSupertileTransitionWork::RoomLoadSpriteResetFallback => {
                    schedule.caller_prefix_nmis
                }
                DungeonSupertileTransitionWork::RoomLoadSpriteReset { .. } => 1,
                DungeonSupertileTransitionWork::RoomLoadCallerResume => schedule
                    .caller_sprite_main_nmis
                    .saturating_add(schedule.caller_suffix_nmis),
                _ => nmi_slices,
            };
        }
        if matches!(
            work,
            DungeonSupertileTransitionWork::SpriteConversion
                | DungeonSupertileTransitionWork::SpiralRoomInitialization
                | DungeonSupertileTransitionWork::SpiralBackgroundSync
                | DungeonSupertileTransitionWork::SpiralSpriteGraphics
        ) {
            let schedule = match self.dungeon_submodule_cpu_schedule {
                Some(schedule) => schedule,
                None => {
                    // A wire-proven single-host completion can advance the
                    // submodule after ordinary host setup already ran, so the
                    // pre-NMI schedule is measured here, at the exact C call
                    // boundary the estimate captured it for.
                    let schedule = dungeon_submodule_cpu_schedule(self);
                    self.dungeon_submodule_cpu_schedule = Some(schedule);
                    schedule
                }
            };
            nmi_slices = schedule.submodule_nmis;
        }
        if nmi_slices == 0 {
            if work == DungeonSupertileTransitionWork::RoomLoadCallerResume {
                self.dungeon_room_load_cpu_schedule = None;
            }
            if matches!(
                work,
                DungeonSupertileTransitionWork::SpriteConversion
                    | DungeonSupertileTransitionWork::SpiralRoomInitialization
                    | DungeonSupertileTransitionWork::SpiralBackgroundSync
                    | DungeonSupertileTransitionWork::SpiralSpriteGraphics
            ) {
                let schedule = self
                    .dungeon_submodule_cpu_schedule
                    .take()
                    .expect("zero-slice spiral work requires its CPU schedule");
                if schedule.caller_nmis != 0 {
                    // The measured estimate says the resumed caller crosses
                    // vblank, but a live host whose wire publishes no
                    // interruption and completes the shared suffix proves the
                    // whole iteration ran inside this host; the wire receipt
                    // supersedes the estimate's caller budget.
                    let wire_completes_host_uninterrupted =
                        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                            && self.original_timing_main_loop_interruption().is_none()
                            && self
                                .original_timing_semantic_receipts
                                .as_ref()
                                .is_some_and(|receipts| {
                                    receipts.semantic().iter().any(|receipt| {
                                        *receipt
                                            == OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                    })
                                });
                    // The wire can instead name where the caller stopped
                    // (NMI_PrepareSprites at route host 216998); module07's
                    // post-submodule owner resumes that phase next host.
                    let wire_names_caller_interruption =
                        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                            && self.original_timing_main_loop_interruption().is_some_and(
                                |interruption| {
                                    module_cpu_phase_from_main_loop_interruption(interruption)
                                        .is_some()
                                },
                            );
                    assert!(
                        wire_completes_host_uninterrupted || wire_names_caller_interruption,
                        "spiral submodule completed before NMI but its caller did not"
                    );
                }
            }
            return false;
        }
        if room_61_sprite_conversion_retains_resident_oam(
            work,
            self.game_state.world.location.dungeon_room_index(),
        ) {
            // The state-3 sprite conversion and its caller return run with the
            // software NMI latch set. Hardware therefore keeps the populated
            // resident OAM table until the following quadrant-upload NMI.
            self.active_display_obj_generation = DisplayObjGeneration::RetainCapturedOam {
                oam: self.ppu.oam.clone(),
            };
        }
        if matches!(
            work,
            DungeonSupertileTransitionWork::StraightInterroomRoomInitialization
                | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics
                | DungeonSupertileTransitionWork::FallingBgCharacters34
                | DungeonSupertileTransitionWork::FallingSpriteGraphics
        ) {
            // The room parser owns the translated main stack for nineteen
            // vblanks. Main_PrepSpritesForNmi cannot publish a replacement
            // shadow during that interval, so hardware keeps the resident OAM
            // generation captured at the call boundary.
            self.active_display_obj_generation = DisplayObjGeneration::RetainCapturedOam {
                oam: self.ppu.oam.clone(),
            };
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonSupertileTransition { work },
            nmi_slices,
        );
        if work == DungeonSupertileTransitionWork::RoomLoadCallerResume {
            self.dungeon_room_load_cpu_schedule = None;
        }
        true
    }

    /// Schedule the falling-transition room initialization when the live
    /// wire proves the C call crosses this host: a trailing mid-call Held
    /// acceptance remains with neither a suffix completion nor its return.
    pub(super) fn begin_dungeon_falling_room_initialization_work(&mut self) -> bool {
        if !self.rom_startup_timing()
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self.original_timing_semantic_receipts.is_none()
        {
            return false;
        }
        let trailing_held = self
            .original_timing_expected_nmi_update_gates
            .contains(&NmiUpdateGate::LatchHeld)
            || self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld)
                        )
                    })
                });
        let suffix_completed =
            self.original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                        )
                    })
                });
        // A second wire shape holds the caller past the host boundary without
        // any trailing acceptance: the iteration starts, enters the room
        // initialization, and the host ends mid-call — the interrupting NMI
        // arrives as the next host's leading acceptance (route host 92533,
        // the warp-pad transition). At this call site every leading receipt
        // has been consumed, so the wire's remainder is empty; the zero-claim
        // plan proves Sprite_Main never returns this host, and no owed
        // progress or return timeline points the interruption anywhere else.
        let open_ended_hold = self
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| receipts.semantic().is_empty())
            && self.original_timing_expected_nmi_update_gates.is_empty()
            && self.original_timing_sprite_main_return_claims_remaining == Some(0)
            && !self.original_timing_owes_sprite_main_return()
            && !self.original_timing_owes_sprite_main_progress()
            && self.original_timing_main_loop_return_timeline().is_none();
        if !(trailing_held || open_ended_hold) || suffix_completed {
            return false;
        }
        // The C call's spiral-stairs adjust publishes Link's coordinates,
        // camera, and room bounds before the first held vblank; only the
        // room-load tail waits for the terminal return.
        self.Dungeon_AdjustAfterSpiralStairs();
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonFallingRoomInitialization,
            1,
        );
        true
    }

    pub(super) fn begin_pre_dungeon_entrance_load_work(&mut self, measured_nmis: Option<u8>) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 6);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreDungeonEntranceLoad {
                sprite_reset: PreDungeonSpriteResetContinuation::Pending,
            },
            measured_nmis.unwrap_or(PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES),
        );
        true
    }

    pub(super) fn begin_pre_dungeon_song_bank_transfer_work(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        debug_assert_eq!(self.game_state.frame.submodule, 15);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreDungeonSongBankTransfer,
            PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_attract_throne_room_work(&mut self) {
        let retained_sprite_subset_2 = self.game_state.sprites.workspace.graphics_subset(2);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractThroneRoom,
            attract_throne_room_nmi_slices(retained_sprite_subset_2),
        );
    }

    pub(super) fn begin_attract_world_map_work(&mut self) {
        // Snes9x executing the original ROM reaches attract state 4 on host
        // frame 5651. The work starts at frame 5646, so exactly five NMI
        // slices elapse before the world-map continuation runs. Seven slices
        // delayed the live scene by two frames and made the source-native
        // renderer faithfully draw the wrong state.
        self.game_execution_scheduler
            .schedule_work(GameWorkContinuation::FinishAttractWorldMap, 5);
    }

    pub(super) fn begin_attract_world_map_exit_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractWorldMapExit,
            ATTRACT_WORLD_MAP_EXIT_NMI_SLICES,
        );
    }

    pub(super) fn begin_world_map_light_load_work(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        // The original CPU enters WorldMap_LoadLightWorldMap after host frame
        // 5934 and does not return until frame 5940. The entry frame performs
        // the first portion of the ROM work; five later NMI slices elapse
        // before the state increment and NMI-7 request become observable.
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishWorldMapLightLoad,
            WORLD_MAP_LIGHT_LOAD_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_attract_zelda_prison_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractZeldaPrison,
            ATTRACT_ZELDA_PRISON_NMI_SLICES,
        );
    }

    pub(super) fn begin_attract_maiden_warp_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractMaidenWarp,
            ATTRACT_MAIDEN_WARP_NMI_SLICES,
        );
    }

    pub(super) fn begin_attract_end_of_story_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractEndOfStory,
            ATTRACT_END_OF_STORY_NMI_SLICES,
        );
    }

    pub(super) fn schedule_pre_main_caller_continuation(
        &mut self,
        continuation: PreMainCallerContinuation,
    ) {
        self.game_execution_scheduler
            .schedule_pre_main_caller_continuation(continuation);
    }

    /// Record the hardware publication owned by an NMI which interrupted the
    /// translated Module 7 dispatcher before its submodule caller returned.
    /// Keep this test at the NMI boundary: the same C call can reach that
    /// boundary from an ordinary main iteration, a leading-NMI iteration, or
    /// a later slice of scheduled CPU work.
    pub(super) fn stage_suspended_dungeon_submodule_after_nmi(&mut self) {
        if self.game_execution_scheduler.current_work()
            == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)
        {
            self.stage_interrupted_dungeon_submodule_publication();
        }
    }

    pub(super) fn retire_enemy_drop_item_graphics_sound_effect_2(&mut self) {
        if let Some(sound_effect) = self.enemy_drop_item_graphics_deferred_sound_effect_2.take() {
            self.set_sound_effect_2(sound_effect);
        }
    }

    pub(super) fn complete_dungeon_after_submodule_cpu_caller_return(&mut self) {
        // The interrupting NMI landed before the indirect Module 7 submodule
        // call returned. Resume the common caller suffix exactly once without
        // replaying the translated submodule.
        if self.dungeon_peg_attribute_flip_pending.is_some() {
            if let Some(next) = self.take_original_timing_dungeon_peg_attribute_flip_progress() {
                // A terminal host may expose the loop's last source cursor at
                // its leading NMI before returning through Sprite_Main and the
                // common suffix (route host 712708). That cursor still belongs
                // to the saved helper stack, even though the surrounding
                // caller-return timeline owns the rest of this host.
                let mut pending = self
                    .dungeon_peg_attribute_flip_pending
                    .take()
                    .expect("terminal peg-attribute cursor lost its saved caller");
                self.advance_dungeon_peg_attribute_flip_between_progress(pending.progress, next);
                pending.progress = next;
                self.dungeon_peg_attribute_flip_pending = Some(pending);
            }
        }
        if let Some(continuation) = self.dungeon_peg_attribute_flip_pending.take() {
            assert_eq!(self.game_state.frame.main_module, 7);
            match continuation.caller {
                DungeonPegAttributeFlipCaller::UpdatePegs => {
                    assert_eq!(self.game_state.frame.submodule, 0x16);
                    assert_eq!(self.game_state.frame.subsubmodule, 0x10);
                    self.complete_dungeon_peg_attribute_flip_after_progress(continuation.progress);
                }
                DungeonPegAttributeFlipCaller::SupertileTransition {
                    state_12_cpu_advance,
                    quadrant_cpu_advance,
                } => {
                    assert_eq!(self.game_state.frame.submodule, 2);
                    self.complete_dungeon_peg_attribute_flip_body_after_progress(
                        continuation.progress,
                    );
                    // Resume immediately after Dungeon_LoadAttribute_Selectable,
                    // exactly where the interrupted C stack returns.
                    self.complete_module07_02_supertile_transition_after_attribute_loader(
                        state_12_cpu_advance,
                        quadrant_cpu_advance,
                    );
                }
                DungeonPegAttributeFlipCaller::SpiralStairs => {
                    assert_eq!(self.game_state.frame.submodule, 0x0e);
                    self.complete_module07_0e_spiral_stairs_after_attribute_loader();
                }
                DungeonPegAttributeFlipCaller::WarpPad => {
                    assert_eq!(self.game_state.frame.submodule, 0x15);
                    self.complete_module07_15_warp_pad_after_attribute_loader();
                }
                DungeonPegAttributeFlipCaller::FallingTransition => {
                    assert_eq!(self.game_state.frame.submodule, 7);
                    self.complete_module07_07_falling_transition_after_attribute_loader();
                }
                DungeonPegAttributeFlipCaller::FatInterRoomStairs => {
                    assert_eq!(self.game_state.frame.submodule, 6);
                    self.complete_module07_06_fat_inter_room_stairs_after_attribute_loader();
                }
                DungeonPegAttributeFlipCaller::StraightInterroomStairs => {
                    assert!(matches!(self.game_state.frame.submodule, 0x11..=0x13));
                    self.complete_module07_11_straight_interroom_stairs_after_attribute_loader();
                }
            }
        }
        if self.dungeon_landing_goal_transition_pending {
            self.dungeon_landing_goal_transition_pending = false;
            let reset_prefix_scanlines = self
                .active_dungeon_landing_spotlight_reset_prefix_scanlines
                .take()
                .unwrap_or(0);
            let reset_prefix = (0..reset_prefix_scanlines)
                .map(|index| self.spotlight_hdma_table_dynamic_entry(index))
                .collect();
            self.complete_deferred_iris_spotlight_goal_transition();
            self.complete_dungeon_landing_goal_active_scanout(reset_prefix);
            self.complete_deferred_module07_0f_operate_spotlight_suffix();
        }
        self.complete_module07_dungeon_after_submodule();
    }

    pub(super) fn prepare_dungeon_cpu_advance_after_returned_main_wait(&mut self) {
        if self.dungeon_landing_cpu_advance_pending.is_some() {
            return;
        }
        if let Some(timing) = begin_dungeon_module_cpu_timing_after_leading_nmi(self) {
            self.dungeon_landing_spotlight_copy_visible_rows = timing.spotlight_copy_visible_rows;
            self.dungeon_landing_spotlight_reset_prefix_scanlines =
                timing.spotlight_reset_prefix_scanlines;
            self.dungeon_landing_cpu_advance_pending = Some(timing.advance);
        }
    }

    /// Run the shared `ZeldaRunGameLoop` suffix once at a module-caller
    /// return: retire the armed pending owner when the wire proved the
    /// suffix belongs to this host, otherwise run the raw translated pair.
    /// Run the shared suffix at a caller return unless the live wire keeps
    /// it outstanding (no suffix receipt and no Open trailing acceptance),
    /// in which case arm its one pending owner for the later
    /// suffix-completing host (the route-host-63268 pattern).
    /// The wire interrupted a resumed Module09 caller's `NMI_PrepareSprites`
    /// at the host boundary (route host 187535): the following host's leading
    /// Held handler and continued return complete the shared suffix. Arm that
    /// suffix's one owner and the typed prep continuation instead of running
    /// the prep/latch pair here.
    pub(super) fn schedule_live_interrupted_nmi_prepare_sprites_caller_return(
        &mut self,
        caller: NmiPrepareSpritesCpuCaller,
    ) {
        if self.pending_main_loop_common_suffix.is_none() {
            self.pending_main_loop_common_suffix =
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
        }
        self.interrupted_nmi_prepare_obj_cache_vram = Some(
            self.last_presented_obj_vram
                .as_ref()
                .cloned()
                .or_else(|| self.ppu.obj_vram_latch.clone())
                .unwrap_or_else(|| self.ppu.vram.clone()),
        );
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { caller },
            1,
        );
    }

    pub(super) fn retire_or_defer_main_loop_common_suffix_by_wire(&mut self) {
        if self.original_timing_live_suffix_outstanding() {
            if self.pending_main_loop_common_suffix.is_none() {
                self.pending_main_loop_common_suffix =
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
            }
        } else {
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
    }

    pub(super) fn retire_or_run_main_loop_common_suffix_after_module_return(&mut self) {
        if self.pending_main_loop_common_suffix.is_some() {
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
    }

    pub(super) fn clear_attract_low_work_area(&mut self) {
        SystemWorkArea::clear_attract_low_work_area(&mut self.ram);
        // The cleared range is shared by the attract controller, Link state,
        // and several scratch models. Refresh every native owner before the
        // initializer writes through a bridge, otherwise stale values can
        // immediately re-project the bytes the ROM just cleared.
        self.sync_native_game_state_from_ram();
    }
}
