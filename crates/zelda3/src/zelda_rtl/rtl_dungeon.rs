//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (dungeon).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn resident_song_bank_is_dungeon(&self) -> bool {
        self.game_state
            .system_signals
            .resident_song_bank_is_dungeon()
    }

    pub(crate) fn select_dungeon_song_bank(&mut self) {
        self.system_signals_mut().select_dungeon_song_bank();
    }

    pub(crate) fn set_dungeon_room(&mut self, value: u16) {
        self.world_location_mut().set_dungeon_room(value);
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        self.world_location_mut().set_dungeon_room_index(value);
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        self.world_location_mut()
            .increment_dungeon_room_index_by(value)
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        self.world_location_mut()
            .decrement_dungeon_room_index_by(value)
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        self.world_location_mut().set_indoor_flag(value);
    }

    pub(crate) fn set_room_transitioning_flags(&mut self, value: u8) {
        self.world_transient_mut()
            .set_room_transitioning_flags(value);
    }

    pub(crate) fn set_cached_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.world_transient_mut()
            .set_cached_room_bounds(y_start, y_end, x_start, x_end);
    }

    pub(crate) fn set_standing_in_doorway_cached(&mut self, value: u8) {
        self.world_transient_mut()
            .set_standing_in_doorway_cached(value);
    }

    pub(crate) fn apply_dungeon_layout_quadrant_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        vertical_mask: u8,
        blast_wall_x_open: bool,
        blast_wall_y_open: bool,
    ) {
        self.world_transient_mut()
            .apply_dungeon_layout_quadrant_fullsize(
                layout_flags,
                horizontal_mask,
                vertical_mask,
                blast_wall_x_open,
                blast_wall_y_open,
            );
    }

    pub(crate) fn apply_dungeon_layout_horizontal_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        blast_wall_x_open: bool,
    ) {
        self.world_transient_mut()
            .apply_dungeon_layout_horizontal_fullsize(
                layout_flags,
                horizontal_mask,
                blast_wall_x_open,
            );
    }

    pub(crate) fn apply_dungeon_layout_vertical_fullsize(
        &mut self,
        layout_flags: u8,
        vertical_mask: u8,
        blast_wall_y_open: bool,
    ) {
        self.world_transient_mut()
            .apply_dungeon_layout_vertical_fullsize(layout_flags, vertical_mask, blast_wall_y_open);
    }

    pub(crate) fn force_horizontal_fullsize_for_blast_wall(&mut self) {
        self.world_transient_mut()
            .force_horizontal_fullsize_for_blast_wall();
    }

    pub(crate) fn force_vertical_fullsize_for_blast_wall(&mut self) {
        self.world_transient_mut()
            .force_vertical_fullsize_for_blast_wall();
    }

    // OVERWORLD_MAP_STATE (0x200) is SNES byte-reused: OverworldMapUiState.map_state (the HUD
    // redraw counter) and DungeonRoomItemState.chest_reveal_cursor_x2 both model/project it.
    // Both native states persist and sync independently, so a HUD-counter change made via
    // map_ui is clobbered when the (stale) dungeon chest-cursor bridge syncs the same byte
    // afterward — leaving OVERWORLD_MAP_STATE stuck (e.g. =5) across a dungeon room transition,
    // which kept hud_refill_logic gated off and skewed rupee/heart-drain timing (~rf 126k-132k).
    // Keep the dungeon model in sync with the HUD counter so neither clobbers the other.
    pub(super) fn sync_dungeon_chest_cursor_with_map_state(&mut self) {
        let value = self.game_state.world.overworld.map_ui.map_state;
        self.dungeon_room_items_mut()
            .set_chest_reveal_cursor_x2(value);
    }

    pub(super) fn complete_straight_interroom_sprite_reset_after_timing_boundary(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
    ) {
        self.complete_straight_interroom_sprite_reset_body(progress);
        self.retire_or_run_main_loop_common_suffix_after_module_return();
    }

    /// The resumed `Dungeon_ResetSprites` caller through Module07's suffix,
    /// without the shared ZeldaRunGameLoop suffix (a typed terminal return
    /// retires that itself; route host 307622).
    pub(super) fn complete_straight_interroom_sprite_reset_body(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
    ) {
        let progress = self.refine_dungeon_reset_progress_before_resume(progress);
        self.dungeon_resume_reset_sprites_after_cpu_progress(progress);
        // This caller crossed its leading display boundary inside
        // Dungeon_ResetSprites. Keep that execution provenance after the
        // one-shot continuation is consumed so the later quadrant-upload
        // states retain the same NMI-first cadence.
        self.game_execution_scheduler
            .begin_leading_nmi_upload_pipeline();
        self.complete_module07_dungeon_after_submodule();
    }

    pub(crate) fn dungeon_tile_definition(
        &self,
        tile: usize,
    ) -> crate::tile_definition::NativeTile {
        self.game_state
            .dungeon
            .room_parser
            .tile_definition(&self.ram, tile)
    }

    pub(crate) fn increment_dungeon_map_init_state(&mut self) {
        self.dungeon_map_mut().increment_dungmap_init_state();
    }

    pub(crate) fn clear_dungeon_map_init_state(&mut self) {
        self.dungeon_map_mut().clear_dungmap_init_state();
    }

    pub(crate) fn set_dungeon_map_current_floor(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_cur_floor(value);
    }

    pub(crate) fn decrement_dungeon_map_current_floor_byte(&mut self) {
        self.dungeon_map_mut().decrement_dungmap_cur_floor_byte();
    }

    pub(crate) fn set_dungeon_map_idx(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_idx(value);
    }

    pub(crate) fn clear_dungeon_map_idx(&mut self) {
        self.dungeon_map_mut().clear_dungmap_idx();
    }

    pub(crate) fn set_dungeon_map_player_marker_x(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_player_marker_x(value);
    }

    pub(crate) fn set_dungeon_map_player_marker_y(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_player_marker_y(value);
    }

    pub(crate) fn set_dungeon_map_location_marker_base_y(&mut self, value: u8) {
        self.dungeon_map_mut().set_location_marker_base_y(value);
    }

    pub(crate) fn reset_dungeon_map_marker_offsets(&mut self) {
        self.dungeon_map_mut().reset_marker_offsets();
    }

    pub(crate) fn shift_dungeon_map_marker_x_left(&mut self) -> u16 {
        self.dungeon_map_mut().shift_marker_x_left()
    }

    pub(crate) fn reset_dungeon_map_marker_x_and_shift_marker_y_low_up(&mut self) {
        self.dungeon_map_mut()
            .reset_marker_x_and_shift_marker_y_low_up();
    }

    pub(crate) fn set_dungeon_map_marker_y_offset(&mut self, value: u16) {
        self.dungeon_map_mut().set_marker_y_offset(value);
    }

    pub(crate) fn add_dungeon_map_marker_y_offset_signed(&mut self, value: i16) -> u16 {
        self.dungeon_map_mut().add_marker_y_offset_signed(value)
    }

    pub(crate) fn set_sp6r_indoors(&mut self, value: u8) {
        self.palette_buffer_mut().set_sp6r_indoors(value);
    }

    pub(crate) fn rotate_animated_dungeon_tile_planes(&mut self) {
        for i in 0..256 {
            let base = 0x9000 + i * 2;
            let x = read_le_u16(&self.ram, base + 0x1880);
            let a = read_le_u16(&self.ram, base + 0x1c80);
            let b = read_le_u16(&self.ram, base + 0x1e80);
            let c = read_le_u16(&self.ram, base + 0x1a80);
            write_le_u16(&mut self.ram, base + 0x1880, a);
            write_le_u16(&mut self.ram, base + 0x1c80, b);
            write_le_u16(&mut self.ram, base + 0x1e80, c);
            write_le_u16(&mut self.ram, base + 0x1a80, x);
        }
    }

    pub(crate) fn set_dungeon_line_pointer_row0(&mut self, index: usize, value: u16) {
        write_le_u16(&mut self.ram, DUNG_LINE_PTRS_ROW0 + index * 2, value);
    }

    pub(crate) fn bombos_blast_mut(&mut self, slot: usize) -> NativeBombosBlastBridgeMut<'_> {
        NativeBombosBlastBridgeMut::new(
            &mut self.game_state.effects.bombos_spell,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn tower_seal_orbit_mut(
        &mut self,
        slot: usize,
    ) -> NativeTowerSealOrbitBridgeMut<'_> {
        NativeTowerSealOrbitBridgeMut::new(
            &mut self.game_state.effects.tower_seal,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn tower_seal_sparkle_mut(
        &mut self,
        slot: usize,
    ) -> NativeTowerSealSparkleBridgeMut<'_> {
        NativeTowerSealSparkleBridgeMut::new(
            &mut self.game_state.effects.tower_seal,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_explosion_mut(
        &mut self,
        slot: usize,
    ) -> NativeBlastWallExplosionBridgeMut<'_> {
        NativeBlastWallExplosionBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_fragment_mut(
        &mut self,
        slot: usize,
    ) -> NativeBlastWallFragmentBridgeMut<'_> {
        NativeBlastWallFragmentBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_fireball_mut(
        &mut self,
        slot: usize,
    ) -> NativeBlastWallFireballBridgeMut<'_> {
        NativeBlastWallFireballBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_direction(&self) -> u8 {
        self.game_state
            .effects
            .entrance_effects
            .blast_wall_direction()
    }

    pub(crate) fn skull_woods_fire_mut(
        &mut self,
        slot: usize,
    ) -> NativeSkullWoodsFireSlotBridgeMut<'_> {
        NativeSkullWoodsFireSlotBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn skull_woods_fire_has_started_entrance_opening(&self) -> bool {
        self.game_state
            .effects
            .entrance_effects
            .skull_woods_fire_has_started_entrance_opening()
    }

    pub(crate) fn skull_woods_fire_inner_x(&self) -> u16 {
        self.game_state
            .effects
            .entrance_effects
            .skull_woods_fire_inner_x()
    }

    pub(crate) fn swamola_target_mut(&mut self, slot: usize) -> NativeSwamolaTargetBridgeMut<'_> {
        NativeSwamolaTargetBridgeMut::new(
            &mut self.game_state.effects.sprite_histories,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn swamola_history_mut(&mut self, slot: usize) -> NativeSwamolaHistoryBridgeMut<'_> {
        NativeSwamolaHistoryBridgeMut::new(
            &mut self.game_state.effects.sprite_histories,
            &mut self.ram,
            slot,
        )
    }

    pub(super) fn continue_dungeon_room_load_after_sprite_reset(
        &mut self,
        schedule: DungeonRoomLoadCpuSchedule,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        reset_resumed_after_nmi: bool,
    ) -> bool {
        debug_assert_eq!(
            schedule.caller_nmis,
            schedule
                .caller_prefix_nmis
                .saturating_add(schedule.caller_sprite_main_nmis)
                .saturating_add(schedule.caller_suffix_nmis),
            "room-load caller phases must cover every measured NMI",
        );
        self.dungeon_room_load_module_suffix_nmi_slices = schedule.caller_suffix_nmis;
        // Fine-grained cached-sprite progress changes within the old
        // synthetic room-load entry envelope. Only a continuous timing owner
        // may publish this semantic domain; absence of a receipt means no
        // cached-sprite interruption was observed for this host call.
        let cached_sprite_interruption = self
            .take_original_timing_cached_sprite_execution_progress()
            .map(|receipt| (receipt.progress.into(), receipt.boundary));
        if schedule.caller_sprite_main_nmis != 0 {
            if let Some((boundary, authority_boundary)) = cached_sprite_interruption {
                assert_eq!(
                    schedule.caller_sprite_main_nmis, 1,
                    "cached-sprite room-load continuation crossed more than one NMI",
                );
                assert!(
                    self.dungeon_cached_sprite_cpu_interruption_pending
                        .replace(boundary)
                        .is_none(),
                    "cached-sprite room-load continuation was already armed",
                );
                assert!(
                    self.dungeon_cached_sprite_cpu_interruption_boundary
                        .replace(authority_boundary)
                        .is_none(),
                    "cached-sprite room-load semantic boundary was already armed",
                );
                // Sprite_Main's ordinary 15..0 walk has completed. Its
                // cached-sprite suffix is interrupted while temporarily
                // swapping an old-room sprite into the live slot.
                self.complete_module07_dungeon_after_submodule();
                debug_assert!(matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonCachedSpriteMain {
                        boundary: scheduled_boundary,
                        ..
                    }) if scheduled_boundary == boundary
                ));
            } else if let Some(boundary) = schedule.sprite_main_boundary {
                self.sprite_main_cpu_boundary = Some(boundary);
                self.sprite_main_cpu_nmi_slices = schedule.caller_sprite_main_nmis;
                self.complete_module07_dungeon_after_submodule();
                debug_assert!(self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack());
            } else {
                // No translated slot commit point was reached before the ROM
                // interruption. Keep the measured remainder attached to the
                // caller rather than inventing a sprite or room predicate.
                self.dungeon_room_load_module_suffix_nmi_slices = 0;
                self.dungeon_room_load_cpu_schedule = Some(schedule);
                let scheduled = self.begin_dungeon_supertile_transition_work(
                    DungeonSupertileTransitionWork::RoomLoadCallerResume,
                );
                debug_assert!(scheduled);
            }
        } else if schedule.caller_suffix_nmis == 0 {
            assert!(
                cached_sprite_interruption.is_none(),
                "cached-sprite progress requires a caller Sprite_Main NMI",
            );
            debug_assert!(schedule.sprite_main_boundary.is_none());
            if reset_resumed_after_nmi {
                // The typed reset receipt already owns the interrupt which
                // suspended Dungeon_ResetSprites. Resume the C caller and
                // return to its main wait without fabricating the following
                // leading NMI in the same host interval.
                self.finish_dungeon_room_load_caller_after_trailing_nmi();
            } else {
                self.complete_dungeon_supertile_caller_return_host(
                    DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    input,
                    oam_dma_source,
                    false,
                );
            }
            return true;
        } else {
            debug_assert!(schedule.sprite_main_boundary.is_none());
            // Sprite_Main returns before the next NMI; run it now and schedule
            // only the measured Module 7 suffix that remains after it.
            self.complete_module07_dungeon_after_submodule();
            debug_assert!(matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                })
            ));
        }
        false
    }

    pub(super) fn complete_room_load_sprite_reset_after_timing_boundary(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        let progress = self.refine_dungeon_reset_progress_before_resume(progress);
        self.dungeon_resume_reset_sprites_after_cpu_progress(progress);
        self.complete_module07_02_01_after_dungeon_reset_sprites();
        let mut schedule = self
            .dungeon_room_load_cpu_schedule
            .take()
            .expect("room-load CPU schedule must survive sprite reset");
        // One libretro host can accept the reset's NMI, resume that same
        // source stack, finish the room load, and reach a second NMI inside
        // Sprite_Main. The reset receipt owns only the first acceptance; the
        // later Sprite_Main checkpoint must refine the remaining caller phase
        // in this same host instead of being deferred behind another slice.
        let progressed_boundary = self.original_timing_sprite_main_progress_boundary();
        let interrupted_phase = self
            .original_timing_main_loop_interruption()
            .filter(|interruption| interruption.is_sprite_main());
        let interrupted_boundary =
            interrupted_phase.and_then(sprite_main_cpu_boundary_from_interruption);
        if let (Some(progressed), Some(interrupted)) = (progressed_boundary, interrupted_boundary) {
            assert!(
                same_sprite_main_source_checkpoint(progressed, interrupted),
                "one room-load host reported incompatible Sprite_Main progress and interruption checkpoints",
            );
        }
        let wire_sprite_main_boundary = interrupted_boundary.or(progressed_boundary);
        if let Some(interruption) = interrupted_phase {
            assert!(
                self.take_original_timing_main_loop_interruption(interruption),
                "peeked room-load Sprite_Main interruption disappeared",
            );
        }
        if wire_sprite_main_boundary.is_some() {
            schedule.caller_prefix_nmis = 0;
            schedule.caller_sprite_main_nmis = 1;
            schedule.caller_suffix_nmis = 0;
            schedule.caller_nmis = 1;
            schedule.sprite_main_boundary = wire_sprite_main_boundary;
        }

        self.continue_dungeon_room_load_after_sprite_reset(schedule, input, oam_dma_source, true)
    }

    pub(super) fn apply_pre_dungeon_garnish_disable_prefix(&mut self, slot: u8) {
        assert!(slot <= 30);
        let work = self
            .game_execution_scheduler
            .current_work()
            .expect("garnish disable requires a caller");
        let GameWorkContinuation::FinishPreDungeonEntranceLoad { sprite_reset } = work else {
            panic!("garnish disable reached another caller");
        };
        let prior = match sprite_reset {
            PreDungeonSpriteResetContinuation::Pending => {
                self.complete_module_pre_dungeon_before_sprite_reset();
                self.apply_sprite_disable_actions_through(
                    None,
                    crate::DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                );
                self.sprite_disable_all_before_garnish_clear();
                30
            }
            PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(previous) => {
                self.apply_sprite_disable_actions_through(
                    Some(
                        crate::DungeonSpriteDisableCpuProgress::SpriteStatesThrough {
                            slot: previous,
                        },
                    ),
                    crate::DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                );
                self.sprite_disable_all_before_garnish_clear();
                30
            }
            PreDungeonSpriteResetContinuation::GarnishDisableThrough(previous) => {
                assert!(slot < previous, "garnish clear did not advance");
                previous
            }
            _ => panic!("garnish clear reached a completed source stage"),
        };
        for k in (usize::from(slot)..usize::from(prior)).rev() {
            self.garnish_slot_view_mut(k).set_garnish_type(0);
        }
        self.game_execution_scheduler.refine_scheduled_work(
            work,
            GameWorkContinuation::FinishPreDungeonEntranceLoad {
                sprite_reset: PreDungeonSpriteResetContinuation::GarnishDisableThrough(slot),
            },
        );
    }

    pub(super) fn apply_pre_dungeon_sprite_disable_prefix(&mut self, slot: u8) {
        assert!(slot < 16);
        let work = self
            .game_execution_scheduler
            .current_work()
            .expect("pre-dungeon disable requires a caller");
        let GameWorkContinuation::FinishPreDungeonEntranceLoad { sprite_reset } = work else {
            panic!("pre-dungeon disable reached another caller");
        };
        let prior = match sprite_reset {
            PreDungeonSpriteResetContinuation::Pending => {
                self.complete_module_pre_dungeon_before_sprite_reset();
                None
            }
            PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(previous) => {
                assert!(
                    slot < previous,
                    "pre-dungeon disable prefix did not advance"
                );
                Some(crate::DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot: previous })
            }
            _ => panic!("pre-dungeon disable reached a completed source stage"),
        };
        self.apply_sprite_disable_actions_through(
            prior,
            crate::DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot },
        );
        self.game_execution_scheduler.refine_scheduled_work(
            work,
            GameWorkContinuation::FinishPreDungeonEntranceLoad {
                sprite_reset: PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(slot),
            },
        );
    }

    pub(super) fn apply_pre_dungeon_sprite_reset_progress(
        &mut self,
        receipt: SpriteResetAllProgressReceipt,
    ) {
        // A host may return either after the source call has advanced into
        // `Sprite_ResetAll_noDisable`, or at the NMI which interrupted that
        // suffix. Both boundaries prove the same completed C call prefix;
        // scheduling still owns whether the caller resumes before or after
        // the interrupt.
        match receipt.progress {
            SpriteResetAllProgress::SpriteDisableAllCompleted => {
                if let Some(GameWorkContinuation::FinishPreDungeonEntranceLoad {
                    sprite_reset: PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(slot),
                }) = self.game_execution_scheduler.current_work()
                {
                    self.complete_sprite_disable_after_states(slot);
                } else if let Some(GameWorkContinuation::FinishPreDungeonEntranceLoad {
                    sprite_reset: PreDungeonSpriteResetContinuation::GarnishDisableThrough(slot),
                }) = self.game_execution_scheduler.current_work()
                {
                    self.complete_sprite_disable_garnish_clear(slot);
                } else {
                    self.complete_module_pre_dungeon_through_sprite_disable_all();
                }
            }
        }
        assert!(
            self.game_execution_scheduler
                .mark_pre_dungeon_sprite_reset_progress(receipt.progress),
            "Module_PreDungeon Sprite_ResetAll progress was duplicated or out of sequence",
        );
    }

    pub(super) fn apply_pre_dungeon_dungeon_reset_progress(
        &mut self,
        receipt: DungeonResetSpritesProgressReceipt,
    ) {
        let GameWorkContinuation::FinishPreDungeonEntranceLoad { sprite_reset } = self
            .game_execution_scheduler
            .current_work()
            .expect("pre-dungeon Dungeon_ResetSprites progress requires active entrance work")
        else {
            panic!("pre-dungeon Dungeon_ResetSprites progress reached the wrong work kind");
        };
        match sprite_reset {
            PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted
            | PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(_)
            | PreDungeonSpriteResetContinuation::GarnishDisableThrough(_) => {
                if let PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(slot) =
                    sprite_reset
                {
                    self.complete_sprite_disable_after_states(slot);
                }
                if let PreDungeonSpriteResetContinuation::GarnishDisableThrough(slot) = sprite_reset
                {
                    self.complete_sprite_disable_garnish_clear(slot);
                }
                // The source returned from Sprite_ResetAll_noDisable before it
                // entered Dungeon_ResetSprites. Publish that intervening tail
                // once, then the exact prefix of the new room's sprite load.
                self.complete_module_pre_dungeon_after_sprite_disable_all();
                self.dungeon_reset_sprites_through_cpu_progress(receipt.progress);
            }
            PreDungeonSpriteResetContinuation::DungeonResetSprites(completed) => {
                assert!(
                    self.dungeon_advance_reset_sprites_cpu_progress(completed, receipt.progress),
                    "timing authority published an unsupported pre-dungeon Dungeon_ResetSprites progress transition",
                );
            }
            _ => panic!(
                "pre-dungeon Dungeon_ResetSprites progress followed invalid state {sprite_reset:?}"
            ),
        }
        assert!(
            self.game_execution_scheduler
                .mark_pre_dungeon_dungeon_reset_progress(receipt.progress),
            "pre-dungeon Dungeon_ResetSprites progress was duplicated or out of sequence",
        );
    }

    pub(super) fn complete_pre_dungeon_sprite_reset_continuation(
        &mut self,
        progress: PreDungeonSpriteResetContinuation,
    ) {
        match progress {
            PreDungeonSpriteResetContinuation::Pending => {
                self.complete_module_pre_dungeon_before_return_suffix();
            }
            PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(slot) => {
                self.complete_sprite_disable_after_states(slot);
                self.complete_module_pre_dungeon_after_sprite_disable_all();
            }
            PreDungeonSpriteResetContinuation::GarnishDisableThrough(slot) => {
                self.complete_sprite_disable_garnish_clear(slot);
                self.complete_module_pre_dungeon_after_sprite_disable_all();
            }
            PreDungeonSpriteResetContinuation::NotApplicable => {
                panic!("a non-dungeon destination cannot resume Module_PreDungeon Sprite_ResetAll")
            }
            PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted => {
                self.complete_module_pre_dungeon_after_sprite_disable_all();
            }
            PreDungeonSpriteResetContinuation::DungeonResetSprites(progress) => {
                self.dungeon_resume_reset_sprites_after_cpu_progress(progress);
            }
            PreDungeonSpriteResetContinuation::SpriteResetAllCompleted
            | PreDungeonSpriteResetContinuation::DungeonResetSpritesCompleted => {}
        }
    }

    pub(super) fn complete_pre_dungeon_authoritative_return_after(
        &mut self,
        progress: PreDungeonSpriteResetContinuation,
    ) {
        if matches!(
            progress,
            PreDungeonSpriteResetContinuation::DungeonResetSprites(_)
                | PreDungeonSpriteResetContinuation::DungeonResetSpritesCompleted
        ) {
            self.complete_module_pre_dungeon_after_dungeon_reset_authoritative_return();
        } else {
            self.complete_module_pre_dungeon_authoritative_return();
        }
    }

    pub(super) fn complete_pre_dungeon_entrance_load_after(
        &mut self,
        progress: PreDungeonSpriteResetContinuation,
    ) {
        if matches!(
            progress,
            PreDungeonSpriteResetContinuation::DungeonResetSprites(_)
                | PreDungeonSpriteResetContinuation::DungeonResetSpritesCompleted
        ) {
            self.complete_module_pre_dungeon_after_dungeon_reset_entrance_load();
        } else {
            self.complete_module_pre_dungeon_entrance_load();
        }
    }

    pub(super) fn finish_pre_dungeon_caller_at_main_wait(
        &mut self,
    ) -> Option<crate::MainLoopProgress> {
        // The first Module 7 iteration begins after the caller's return NMI.
        // Its instruction-derived landing CPU advance owns whichever routine
        // the next vblank actually interrupts. Do not pre-arm Sprite_Main:
        // when Spotlight_open crosses that boundary, the resumed C caller can
        // still finish Sprite_Main before the following NMI.
        debug_assert!(self.sprite_main_cpu_boundary.is_none());
        debug_assert_eq!(self.sprite_main_cpu_nmi_slices, 0);
        match self.take_original_timing_main_loop_progress() {
            Some(progress @ crate::MainLoopProgress::IterationStarted) => Some(progress),
            Some(crate::MainLoopProgress::CallStackContinued) => {
                panic!(
                    "native pre-dungeon caller completed before the timing authority returned from its source call stack"
                )
            }
            None => {
                self.game_execution_scheduler
                    .finish_call_stack_at_main_wait_before_nmi();
                None
            }
        }
    }

    pub(super) fn finish_dungeon_room_load_caller_at_main_wait(&mut self) {
        // ZeldaRunGameLoop returns from Module_MainRouting through
        // NMI_PrepareSprites and then clears $12 before it waits for the next
        // hardware NMI. The scheduler records that pending hardware boundary;
        // the software latch must not be used as a second cadence owner.
        self.clear_nmi_update_latch();
        self.game_execution_scheduler
            .finish_call_stack_at_main_wait_before_nmi();
    }

    pub(super) fn complete_dungeon_after_submodule_caller_return(&mut self) {
        // A held suffix may follow a completed Sprite_Main even though the
        // enclosing ZeldaRunGameLoop has not returned. Bind that source fact
        // to the actual native slot-loop return, before deferring the suffix.
        let claims = self
            .original_timing_semantic_receipts
            .as_ref()
            .map_or(0, |receipts| {
                receipts
                    .semantic()
                    .iter()
                    .filter(|receipt| {
                        **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                    })
                    .count()
            });
        if claims != 0 {
            self.begin_original_timing_sprite_main_return_claim_scope(claims);
        }
        self.complete_dungeon_after_submodule_cpu_caller_return();
        if claims != 0 {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        let live_suffix_outstanding = self.original_timing_live_suffix_outstanding();
        if live_suffix_outstanding {
            // A live host whose wire holds the update latch through its
            // trailing boundary keeps the shared ZeldaRunGameLoop suffix
            // outstanding; arm its one owner for the later suffix-completing
            // host (route host 63268).
            if self.pending_main_loop_common_suffix.is_none() {
                self.pending_main_loop_common_suffix =
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
            }
        } else {
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
        self.prepare_dungeon_cpu_advance_after_returned_main_wait();
    }

    /// The shared suffix after a parked dungeon Sprite_Main resumed through the
    /// after-trailing lane: a wire-interrupted NMI_PrepareSprites resumes on
    /// the next host, an outstanding live suffix stays with the wire, and the
    /// estimate lane (or a typed return) runs the prep/latch pair here.
    pub(super) fn complete_parked_dungeon_sprite_main_suffix_by_wire(
        &mut self,
        live_nonterminal: bool,
    ) {
        if live_nonterminal
            && self.original_timing_main_loop_interruption()
                == Some(crate::MainLoopInterruption::SpritePreparation)
        {
            let _ = self.take_original_timing_main_loop_interruption_any();
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::DungeonModule07,
            );
            return;
        }
        if live_nonterminal
            && self.original_timing_main_loop_interruption()
                == Some(crate::MainLoopInterruption::LinkOam)
        {
            // The Module 7 tail's LinkOam_Main was interrupted at the host
            // boundary (route host 413554). The native tail has already run
            // it; the OAM image it authored is only visible after the next
            // NMI, and the shared suffix stays with the wire.
            let _ = self.take_original_timing_main_loop_interruption_any();
            self.retire_or_defer_main_loop_common_suffix_by_wire();
            return;
        }
        if live_nonterminal {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        } else {
            // A typed terminal main-loop return already owns the shared
            // ZeldaRunGameLoop suffix (`pending_main_loop_common_suffix`);
            // retire that one owner here instead of running a bare
            // NMI_PrepareSprites and then the pending one again (route host
            // 717302: the second preparation advanced the Link animated-tile
            // DMA cycle one frame ahead of the oracle, visible at 732911).
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
    }

    pub(super) fn run_dungeon_submodule(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.module07_00_player_control(),
            1 => self.Module07_01_SubtileTransition(),
            2 => self.Module07_02_SupertileTransition(),
            3 => self.Module07_03_OverlayChange(),
            4 => self.Module07_04_UnlockDoor(),
            5 => self.Module07_05_ControlShutters(),
            6 => self.Module07_06_FatInterRoomStairs(),
            7 => self.Module07_07_FallingTransition(),
            8 => self.Module07_08_NorthIntraRoomStairs(),
            9 => self.Module07_09_OpenCrackedDoor(),
            10 => self.Module07_0A_ChangeBrightness(),
            11 => self.Module07_0B_DrainSwampPool(),
            12 => self.Module07_0C_FloodSwampWater(),
            13 => self.Module07_0D_FloodDam(),
            14 => self.Module07_0E_SpiralStairs(),
            15 => self.Module07_0F_LandingWipe(),
            16 => self.Module07_10_SouthIntraRoomStairs(),
            17..=19 => self.Module07_11_StraightInterroomStairs(),
            20 => self.Module07_14_RecoverFromFall(),
            21 => self.Module07_15_WarpPad(),
            22 => self.Module07_16_UpdatePegs(),
            23 => self.Module07_17_PressurePlate(),
            24 => self.Module07_18_RescuedMaiden(),
            25 => self.Module07_19_MirrorFade(),
            26 => self.Module07_1A_RoomDraw_OpenTriforceDoor_bounce(),
            _ => panic!("invalid dungeon submodule index"),
        }
    }
}
