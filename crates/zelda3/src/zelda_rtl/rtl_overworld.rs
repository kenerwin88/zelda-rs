//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (overworld).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn select_overworld_song_bank(&mut self) {
        self.system_signals_mut().select_overworld_song_bank();
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        self.world_location_mut().set_overworld_screen(value);
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        self.world_location_mut().set_overworld_screen_word(value);
    }

    pub(crate) fn set_overworld_offset_base_y(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_base_y(value);
    }

    pub(crate) fn set_overworld_offset_base_x(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_base_x(value);
    }

    pub(crate) fn set_overworld_offset_mask_y(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_mask_y(value);
    }

    pub(crate) fn set_overworld_offset_mask_x(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_mask_x(value);
    }

    pub(crate) fn set_overworld_area_index(&mut self, value: u8) {
        self.world_region_mut().set_overworld_area_index(value);
    }

    pub(crate) fn set_overworld_area_index_word(&mut self, value: u16) {
        self.world_region_mut().set_overworld_area_index_word(value);
    }

    pub(crate) fn set_flag_overworld_area_changed(&mut self, value: u8) {
        self.world_region_mut()
            .set_flag_overworld_area_changed(value);
    }

    pub(crate) fn clear_flag_overworld_area_changed(&mut self) {
        self.world_region_mut().clear_flag_overworld_area_changed();
    }

    pub(crate) fn set_fullsize_overworld_quadrants(&mut self) {
        self.world_transient_mut()
            .set_fullsize_overworld_quadrants();
    }

    pub(crate) fn set_overworld_hole_scan_step(&mut self, value: u8) {
        self.world_transient_mut()
            .set_overworld_hole_scan_step(value);
    }

    pub(crate) fn set_overworld_peg_puzzle_progress(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_peg_puzzle_progress(value);
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_x(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_bomb_tile_sweep_x(value);
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_y_end(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_bomb_tile_sweep_y_end(value);
    }

    pub(crate) fn overworld_map_state(&self) -> u8 {
        self.game_state.world.overworld.map_ui.map_state()
    }

    pub(crate) fn overworld_map_state_word(&self) -> u16 {
        self.game_state.world.overworld.map_ui.map_state_word()
    }

    pub(crate) fn set_overworld_map_state(&mut self, value: u8) {
        self.overworld_map_ui_mut().set_map_state(value);
        self.sync_dungeon_chest_cursor_with_map_state();
    }

    pub(crate) fn set_overworld_map_state_word(&mut self, value: u16) {
        self.overworld_map_ui_mut().set_map_state_word(value);
        self.sync_dungeon_chest_cursor_with_map_state();
    }

    pub(crate) fn increment_overworld_map_state(&mut self) {
        self.overworld_map_ui_mut().increment_map_state();
        self.sync_dungeon_chest_cursor_with_map_state();
    }

    pub(crate) fn overworld_map_flags(&self) -> u8 {
        self.game_state.world.overworld.map_ui.map_flags
    }

    pub(crate) fn set_overworld_map_flags(&mut self, value: u8) {
        self.overworld_map_ui_mut().set_map_flags(value);
    }

    pub(crate) fn and_overworld_map_flags(&mut self, value: u8) {
        self.overworld_map_ui_mut().and_map_flags(value);
    }

    pub(crate) fn or_overworld_map_flags(&mut self, value: u8) {
        self.overworld_map_ui_mut().or_map_flags(value);
    }

    pub(crate) fn overworld_is_big_area_word(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .screen_size
            .is_big_area_word()
    }

    pub(crate) fn overworld_is_big_area(&self) -> bool {
        self.game_state.world.overworld.screen_size.is_big_area()
    }

    pub(crate) fn clear_overworld_big_area_high(&mut self) {
        self.overworld_screen_size_mut().clear_big_area_high();
    }

    pub(crate) fn set_overworld_big_area_low(&mut self, value: u8) {
        self.overworld_screen_size_mut().set_big_area_low(value);
    }

    pub(crate) fn set_overworld_right_bottom_bound_low(&mut self, value: u8) {
        self.overworld_screen_size_mut()
            .set_right_bottom_bound_low(value);
    }

    pub(crate) fn set_overworld_right_bottom_bound_high(&mut self, value: u8) {
        self.overworld_screen_size_mut()
            .set_right_bottom_bound_high(value);
    }

    pub(crate) fn overworld_transition_countdown(&self) -> u8 {
        self.game_state.world.overworld.transition.countdown()
    }

    pub(crate) fn set_overworld_transition_countdown(&mut self, value: u8) {
        self.overworld_transition_mut().set_countdown(value);
    }

    pub(crate) fn decrement_overworld_transition_countdown(&mut self) -> u8 {
        self.overworld_transition_mut().decrement_countdown()
    }

    pub(crate) fn overworld_tile_attribute_word(&self, index: usize) -> u16 {
        self.game_state
            .display
            .overworld_tile_attribute_word(&self.ram, index)
    }

    pub(crate) fn set_overworld_tile_attribute_word(&mut self, index: usize, value: u16) {
        let address = crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + index * 2;
        write_le_u16(&mut self.ram, address, value);
        debug_assert_eq!(
            self.game_state
                .display
                .overworld_tile_attribute_word(&self.ram, index),
            value
        );
    }

    pub(crate) fn set_overworld_tile_upload_word(&mut self, index: usize, value: u16) {
        let address = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + index * 2;
        write_le_u16(&mut self.ram, address, value);
        debug_assert_eq!(
            self.game_state
                .display
                .overworld_tile_upload_word(&self.ram, index),
            value
        );
    }

    pub(crate) fn terminate_overworld_tile_upload_words(&mut self, index: usize) {
        self.set_overworld_tile_upload_word(index, 0xffff);
    }

    pub(crate) fn set_overworld_fixed_color_adjustment(&mut self, value: u8) {
        self.dungeon_room_effects_mut()
            .set_fixed_color_plusminus_write_through(value);
    }

    pub(crate) fn set_overworld_event_bits(&mut self, screen: usize, mask: u8) {
        self.overworld_event_info_mut().set_event_bits(screen, mask);
    }

    pub(crate) fn set_overworld_event_info(&mut self, screen: usize, value: u8) {
        self.overworld_event_info_mut()
            .set_event_info(screen, value);
    }

    pub(crate) fn clear_overworld_event_bits(&mut self, screen: usize, mask: u8) {
        self.overworld_event_info_mut()
            .clear_event_bits(screen, mask);
    }

    pub(crate) fn overworld_config_table(&self) -> OverworldConfigTableRead<'_> {
        OverworldConfigTableRead::new(
            &self.game_state.world.overworld.config_table,
            usize::from(self.game_state.world.location.overworld_screen_index()),
        )
    }

    pub(crate) fn copy_overworld_music_primary(&mut self, data: &[u8]) {
        self.overworld_config_table_mut().copy_music_primary(data);
    }

    pub(crate) fn copy_overworld_music_secondary(&mut self, data: &[u8]) {
        self.overworld_config_table_mut().copy_music_secondary(data);
    }

    pub(crate) fn set_overworld_music(&mut self, screen: usize, value: u8) {
        self.overworld_config_table_mut().set_music(screen, value);
    }

    pub(crate) fn clear_overworld_aux_or_main_offset(&mut self) {
        self.palette_buffer_mut()
            .clear_overworld_aux_or_main_offset();
    }

    pub(crate) fn keep_overworld_aux_or_main_low_byte(&mut self) {
        self.palette_buffer_mut()
            .keep_overworld_aux_or_main_low_byte();
    }

    pub(crate) fn overworld_map16_decode(&self) -> OverworldMap16Decode<'_> {
        OverworldMap16Decode::new(&self.ram)
    }

    pub(crate) fn copy_overworld_map16_decode_source_from(&mut self, data: &[u8]) {
        OverworldMap16DecodeScratch::copy_source_from(&mut self.ram, data);
    }

    pub(crate) fn copy_overworld_map16_scratch_to_source_words_high(&mut self, len: usize) {
        OverworldMap16DecodeScratch::copy_scratch_to_source_words_high(&mut self.ram, len);
    }

    pub(crate) fn copy_overworld_map16_scratch_to_source_words_low(&mut self, len: usize) {
        OverworldMap16DecodeScratch::copy_scratch_to_source_words_low(&mut self.ram, len);
    }

    pub(crate) fn write_overworld_map16_decompressed_byte(&mut self, dst: usize, value: u8) {
        OverworldMap16DecodeScratch::write_decompressed_byte(&mut self.ram, dst, value);
    }

    pub(crate) fn copy_overworld_map16_decompressed_byte(
        &mut self,
        dst_org: usize,
        dst: usize,
        offset: usize,
    ) {
        OverworldMap16DecodeScratch::copy_decompressed_byte(&mut self.ram, dst_org, dst, offset);
    }

    pub(crate) fn fill_overworld_map16_decode_block(&mut self, dst: usize, table: &[u8], x: usize) {
        OverworldMap16DecodeScratch::decode_block_fill(&mut self.ram, dst, table, x);
    }

    pub(crate) fn set_overworld_map16_decode_last(&mut self, value: u16) {
        OverworldMap16DecodeScratch::set_decode_last(&mut self.ram, value);
    }

    pub(crate) fn set_overworld_map16_decode_tmp(&mut self, value: u16) {
        OverworldMap16DecodeScratch::set_decode_tmp(&mut self.ram, value);
    }

    pub(crate) fn bombos_fire_column(&self, slot: usize) -> BombosFireColumnState {
        self.game_state.effects.bombos_spell.fire_column(slot)
    }

    pub(crate) fn bombos_fire_column_mut(
        &mut self,
        slot: usize,
    ) -> NativeBombosFireColumnBridgeMut<'_> {
        NativeBombosFireColumnBridgeMut::new(
            &mut self.game_state.effects.bombos_spell,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn set_weather_vane_countdown(&mut self, value: u16) {
        self.weather_vane_bridge_mut().set_countdown(value);
    }

    pub(crate) fn tick_weather_vane_countdown(&mut self) -> u16 {
        self.weather_vane_bridge_mut().tick_countdown()
    }

    pub(crate) fn set_weather_vane_source_slot(&mut self, value: u8) {
        self.weather_vane_bridge_mut().set_source_slot(value);
    }

    pub(crate) fn weather_vane_debris(&self, slot: usize) -> WeatherVaneDebrisSlotState {
        self.game_state.effects.weather_vane_debris.debris(slot)
    }

    pub(crate) fn weather_vane_debris_mut(
        &mut self,
        slot: usize,
    ) -> NativeWeatherVaneDebrisBridgeMut<'_> {
        NativeWeatherVaneDebrisBridgeMut::new(
            &mut self.game_state.effects.weather_vane_debris,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn set_overworld_sprite_presence_marker(&mut self, index: usize, value: u8) {
        self.overworld_sprite_presence_mut()
            .set_marker(index, value);
    }

    pub(crate) fn clear_overworld_sprite_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.overworld_sprite_loaded_mut()
            .clear_loaded_mask(block, loaded_mask);
    }

    pub(crate) fn clear_overworld_sprite_loaded_mask_wrapped(
        &mut self,
        block: u16,
        loaded_mask: u8,
    ) {
        // C computes the SNES address `addr = 0xEF80 + (blk >> 3)` in 16-bit
        // (so it wraps mod 0x10000), THEN dereferences `&g_ram[addr + 0x10000]`.
        // The bank add happens AFTER the 16-bit wrap, so a large `blk` whose
        // 16-bit address wraps below 0xEF80 lands in 0x10000..=0x10F7F (the BG
        // char buffer), not in low WRAM. Replicate that exact two-step math:
        // mask the 16-bit address first, then add the 0x10000 bank.
        let addr16 = 0xEF80u16.wrapping_add(block >> 3);
        let address = 0x10000 + usize::from(addr16);
        // A large `block` (e.g. a killed dungeon sprite whose load block is near
        // 0xffff) wraps the 16-bit address below 0xEF80 and the raw `&=` spills out
        // of the loaded table into 0x10000..=0x10F7F (the BG char buffer bank). No
        // native state models that window, so the raw write needs no model adopt;
        // only in-table addresses update the native loaded-mask mirror.
        self.ram[address] &= !loaded_mask;
        if let Some(index) = address.checked_sub(OVERWORLD_SPRITE_WAS_LOADED) {
            if index < crate::game_state::OVERWORLD_SPRITE_FLAG_COUNT {
                self.game_state
                    .sprites
                    .overworld_sprite_loaded
                    .clear_loaded_mask(block, loaded_mask);
            }
        }
    }

    pub(crate) fn set_overworld_sprite_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.overworld_sprite_loaded_mut()
            .set_loaded_mask(block, loaded_mask);
    }

    pub(crate) fn clear_all_overworld_sprite_loaded_masks(&mut self) {
        self.overworld_sprite_loaded_mut().clear_all();
    }

    pub fn set_overworld_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.overworld_map16_mut().set_active_load(state);
    }

    pub fn overworld_prev_map16_load_state(&self) -> OverworldMap16LoadState {
        self.game_state.world.overworld.map16.previous_load
    }

    pub fn set_overworld_prev_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.overworld_map16_mut().set_previous_load(state);
    }

    pub fn overworld_spexit_map16_src_off(&self) -> u16 {
        self.game_state.world.overworld.map16.special_exit_src_off
    }

    pub fn set_overworld_spexit_map16_src_off(&mut self, src_off: u16) {
        self.overworld_map16_mut().set_special_exit_src_off(src_off);
    }

    pub fn overworld_exit_map16_src_off(&self) -> u16 {
        self.game_state.world.overworld.map16.exit_src_off
    }

    pub fn set_overworld_exit_map16_src_off(&mut self, src_off: u16) {
        self.overworld_map16_mut().set_exit_src_off(src_off);
    }

    pub fn sync_overworld_map16_state_from_ram(&mut self) {
        self.overworld_map16_mut().sync_from_ram();
    }

    pub(super) fn advance_flute_menu_reload_to_scan_if_needed(&mut self) -> bool {
        let expected = GameWorkContinuation::FinishFluteMenuSelectedScreen {
            step: FluteMenuSelectedScreenStep::OverworldReloadReset,
        };
        if self.game_execution_scheduler.current_work() != Some(expected) {
            return false;
        }
        self.complete_bird_travel_reload_reset_before_presence();
        self.overworld_load_sprites();
        self.begin_overworld_proximity_scan_continuation();
        self.set_overworld_horizontal_scroll_delta_low(0xff);
        self.game_execution_scheduler.refine_scheduled_work(
            expected,
            GameWorkContinuation::FinishFluteMenuSelectedScreen {
                step: FluteMenuSelectedScreenStep::OverworldReloadScan,
            },
        );
        true
    }

    pub(super) fn begin_overworld_proximity_scan_continuation(&mut self) {
        if self.overworld_proximity_scan_saved_scroll.is_none() {
            self.overworld_proximity_scan_saved_scroll = Some((
                self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
                self.overworld_horizontal_scroll_delta_low(),
            ));
        }
    }

    pub(super) fn complete_overworld_proximity_scan_continuation(&mut self) {
        let (bg2_h, horizontal_delta_low) = self
            .overworld_proximity_scan_saved_scroll
            .take()
            .expect("source-completed overworld proximity scan lost its native call locals");
        self.set_overworld_horizontal_scroll_delta_low(horizontal_delta_low);
        self.set_bg2_x(bg2_h);
    }

    /// `FluteMenu_LoadSelectedScreen`'s two source callers: the bird-travel
    /// menu (Module0E/$0A state 7) and Module18_GanonEmerges' pyramid-area
    /// load (state 3), which shares the whole synchronous load stack.
    pub(super) fn flute_menu_selected_screen_caller_is_active(&self) -> bool {
        let frame = &self.game_state.frame;
        (frame.main_module == 0x0e && frame.submodule == 0x0a)
            || (frame.main_module == 0x18
                && frame.submodule == 0
                && self.overworld_map_state() == 3)
    }

    pub(super) fn apply_flute_menu_sprite_reset_progress(
        &mut self,
        receipt: SpriteResetAllProgressReceipt,
    ) {
        assert_eq!(
            receipt.progress,
            SpriteResetAllProgress::SpriteDisableAllCompleted,
        );
        assert!(
            matches!(
                receipt.boundary,
                crate::OriginalTimingBoundary::HostReturn
                    | crate::OriginalTimingBoundary::NmiAccepted
            ),
            "the bird-travel reload reset checkpoint requires a host or NMI boundary",
        );
        let current = self.game_execution_scheduler.current_work();
        let initial = GameWorkContinuation::FinishFluteMenuSelectedScreen {
            step: FluteMenuSelectedScreenStep::InitialSpriteReset,
        };
        match current {
            Some(work) if work == initial => {
                self.complete_bird_travel_initial_reset_and_begin_reload();
                self.game_execution_scheduler.refine_scheduled_work(
                    initial,
                    GameWorkContinuation::FinishFluteMenuSelectedScreen {
                        step: FluteMenuSelectedScreenStep::OverworldReloadReset,
                    },
                );
            }
            Some(GameWorkContinuation::FinishFluteMenuSelectedScreen {
                step: FluteMenuSelectedScreenStep::OverworldReloadReset,
            }) => {
                // A long `Sprite_ResetAll_noDisable` MVN may cross another
                // host/NMI after its caller's `Sprite_DisableAll` checkpoint.
                // This receipt restates the phase already carried by the
                // continuation; no completed statement is replayed.
            }
            _ => panic!(
                "bird-travel sprite-reset progress arrived outside its reset continuation: {current:?}",
            ),
        }
    }

    pub(super) fn finish_overworld_sprite_reload_return(
        &mut self,
        post_return_hold_nmi_slices: u8,
        epilogue_phase: NmiPhase,
        resume_scanout: OverworldSpriteReloadResumeScanout,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        wire_owns_return: bool,
    ) {
        // The provisional sprite generation may have advanced rain before the
        // suspended ROM call stack reached it. Complete the typed scanout with
        // the actual Module09 return generation only after its nested
        // Sprite_Main and caller suffix have both returned.
        let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
        let resume_scanout = resume_scanout.complete_transition_return(returned_scroll);
        if epilogue_phase == NmiPhase::BeforeNmi {
            if wire_owns_return {
                // The terminal-return timeline already consumed the suffix
                // receipt; it proves the suffix belongs to this host.
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            } else {
                // Without a terminal return the wire can hold the caller's
                // shared suffix past this host (route host 117638: the
                // reload returns, Sprite_Main runs, and the trailing
                // acceptance stays held).
                self.retire_or_defer_main_loop_common_suffix_by_wire();
            }
        }
        if wire_owns_return {
            // The wire's terminal return owns any trailing acceptance; the
            // staged host-return carry publishes it to the next host (route
            // host 38518).
        } else if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && !self.original_timing_expected_nmi_update_gates.is_empty()
        {
            // The wire accepted the trailing NMI at this return boundary;
            // its handler belongs to the next host and the generic
            // scheduled-caller host-return staging carries it (route host
            // 117638). Synthesizing an NMI here would run that handler one
            // host early.
        } else {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source, false);
        }
        if epilogue_phase == NmiPhase::AfterNmi {
            // The suspended iteration's shared suffix owner (armed while the
            // reload held the fresh iteration) retires here; running the
            // prep/latch pair directly would strand it and reject the next
            // host's install (route host 108684). Without a wire-proven
            // terminal return the suffix can instead stay outstanding past
            // this host entirely (route host 117638).
            if wire_owns_return {
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            } else {
                self.retire_or_defer_main_loop_common_suffix_by_wire();
            }
        }
        if post_return_hold_nmi_slices != 0 {
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::HoldOverworldSpriteReloadReturn,
                post_return_hold_nmi_slices,
            );
        } else {
            self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                PreMainNmiResume::OverworldSpriteReloadReturn {
                    scanout: resume_scanout,
                },
            );
        }
    }

    pub(super) fn set_link_z_coord_mirror_low_ff(&mut self) {
        self.follower_link_state_mut().force_z_mirror_low_ff();
    }
}
