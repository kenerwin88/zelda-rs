//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (dialogue).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn set_big_key_door_message_triggered(&mut self, value: u16) {
        self.world_transient_mut()
            .set_big_key_door_message_triggered(value);
    }

    pub(crate) fn set_message_dma_destination_address(&mut self, value: u16) {
        self.display_core_mut()
            .set_message_dma_destination_address(value);
    }

    pub(crate) fn set_message_dma_tile_base(&mut self, value: u16) {
        self.display_core_mut().set_message_dma_tile_base(value);
    }

    pub(crate) fn set_message_dma_tile_limit(&mut self, value: u16) {
        self.display_core_mut().set_message_dma_tile_limit(value);
    }

    pub(crate) fn set_message_dma_tile_sentinel(&mut self, value: u16) {
        self.display_core_mut().set_message_dma_tile_sentinel(value);
    }

    pub(crate) fn start_shared_message_timer(&mut self, value: u16) {
        self.shared_message_timer_bridge_mut().start(value);
    }

    pub(crate) fn clear_shared_message_timer(&mut self) {
        self.shared_message_timer_bridge_mut().clear();
    }

    pub(crate) fn tick_shared_message_timer(&mut self) -> u16 {
        self.shared_message_timer_bridge_mut().tick()
    }

    pub(crate) fn copy_graphics_message_rows(
        &mut self,
        dst: usize,
        src0: usize,
        src1: usize,
        len: usize,
    ) {
        self.messaging_render_buffer_mut()
            .copy_rows_from_ram(dst, src0, src1, len);
    }

    pub(crate) fn copy_peg_tile_graphics_to_message_buffer(&mut self, first: usize, second: usize) {
        for i in 0..64 {
            let color = read_le_u16(&self.ram, PEG_TILE_GFX_BUFFER + (first >> 1) * 2 + i * 2);
            write_le_u16(&mut self.ram, MESSAGING_BUF_LOAD_GFX + i * 2, color);
        }
        for i in 0..64 {
            let color = read_le_u16(&self.ram, PEG_TILE_GFX_BUFFER + (second >> 1) * 2 + i * 2);
            write_le_u16(&mut self.ram, MESSAGING_BUF_LOAD_GFX + (64 + i) * 2, color);
        }
    }

    pub(crate) fn message_dma_tile_indices(&self) -> &[u8] {
        self.game_state.display.message_dma_tile_indices(&self.ram)
    }

    pub(crate) fn game_over_text_tile_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .game_over_text_tile_buffer(&self.ram)
    }

    pub(crate) fn game_over_text_tail_tile_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .game_over_text_tail_tile_buffer(&self.ram)
    }

    pub(crate) fn set_vwf_next_glyph_advance_prefix_sum(&mut self, index: usize, value: u8) {
        self.vwf_render_mut()
            .set_next_glyph_advance_prefix_sum(index, value);
        // C writes `vwf_arr[index + 1] = value` as raw g_ram, even past the modeled buffer
        // (the credits render lines whose glyph cursor runs beyond it). For those overflow
        // bytes the native Vec projection above does not reach RAM, so write them directly.
        let buf_len = self
            .game_state
            .messaging
            .vwf_render
            .glyph_advance_buffer_len();
        let addr = VWF_ARR + index + 1;
        if index + 1 >= buf_len && addr < self.ram.len() {
            self.ram[addr] = value;
        }
    }

    /// C: `arrval = vwf_arr[index]` — raw g_ram. In-bounds indices read the modeled buffer
    /// (kept RAM-coherent); indices past it read RAM directly, matching C's unbounded access.
    pub(crate) fn vwf_glyph_advance_prefix_sum(&self, index: usize) -> u8 {
        let vwf = &self.game_state.messaging.vwf_render;
        if index < vwf.glyph_advance_buffer_len() {
            vwf.glyph_advance_prefix_sum(index)
        } else {
            self.ram.get(VWF_ARR + index).copied().unwrap_or(0)
        }
    }

    pub(crate) fn set_vwf_glyph_cursor(&mut self, value: u16) {
        self.vwf_render_mut().set_glyph_cursor(value);
    }

    pub(crate) fn clear_vwf_glyph_cursor(&mut self) {
        self.vwf_render_mut().clear_glyph_cursor();
    }

    pub(crate) fn increment_vwf_glyph_cursor(&mut self) -> u16 {
        self.vwf_render_mut().increment_glyph_cursor()
    }

    pub(crate) fn request_vwf_next_line(&mut self, value: u16) {
        self.vwf_render_mut().request_next_line(value);
    }

    pub(crate) fn clear_vwf_next_line_request(&mut self) {
        self.vwf_render_mut().clear_next_line_request();
    }

    pub(crate) fn set_vwf_current_line(&mut self, value: u16) {
        self.vwf_render_mut().set_current_line(value);
    }

    pub(crate) fn set_vwf_line_render_offset(&mut self, value: u16) {
        self.vwf_render_mut().set_line_render_offset(value);
    }

    pub(crate) fn set_vwf_tile_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        self.vwf_render_mut()
            .set_tile_word_at_byte_offset(byte_offset, value);
    }

    /// Build Module05's Message-interface publication from the exact source
    /// return at `$0f:fdc3`. This replaces the former decompression-count
    /// estimate: only the typed receipt may expose module 14/submodule 2 or
    /// advance the frozen continuation.
    pub(super) fn original_timing_selected_game_load_message_interface_plan(
        &self,
    ) -> Option<OriginalTimingSelectedGameLoadMessageInterfacePlan> {
        if !self.rom_startup_timing()
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self
                .original_timing_semantic_receipts
                .as_ref()
                .is_none_or(|receipts| {
                    !receipts.semantic.contains(
                        &OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished,
                    )
                })
            || !matches!(
                self.game_execution_scheduler
                    .selected_game_load_destination(),
                Some(SelectedGameLoadDestination::Message)
            )
            || self
                .game_execution_scheduler
                .selected_game_load_after_pre_dungeon_audio_sprite_reset()
                .is_some()
        {
            return None;
        }
        let mut scheduler_before_transition = self.game_execution_scheduler;
        scheduler_before_transition.begin_host_frame();
        let mut scheduler_after_transition = scheduler_before_transition;
        assert_eq!(
            scheduler_after_transition.publish_selected_game_load_message_interface_from_source(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting),
            "the Message-interface receipt reached a non-selected scheduler owner",
        );
        assert_eq!(
            scheduler_after_transition.selected_game_load_after_pre_dungeon_audio_sprite_reset(),
            Some(PreDungeonSpriteResetContinuation::NotApplicable),
            "the Message selected-game load cannot own a nested Sprite_ResetAll continuation",
        );
        assert!(
            scheduler_after_transition.selected_game_load_message_interface_published(),
            "the source Message-interface transition did not preserve its publication fact",
        );
        assert!(
            self.original_timing_main_loop_return_timeline().is_none(),
            "the Message-interface publication cannot also return Module05",
        );
        let continuation = self
            .original_timing_nonterminal_continuation_plan_with_receipt_before_trailing_acceptance(
                OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished,
            )
            .expect("live Message-interface publication lost its continued-call owner");
        Some(OriginalTimingSelectedGameLoadMessageInterfacePlan {
            continuation,
            scheduler_before_transition,
            scheduler_after_transition,
        })
    }

    pub(super) fn take_original_timing_selected_game_load_message_interface_published(
        &mut self,
    ) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut published = false;
        receipts.semantic.retain(|receipt| {
            if *receipt == OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished
            {
                assert!(
                    !published,
                    "selected-game Message interface replayed in one host"
                );
                published = true;
                false
            } else {
                true
            }
        });
        published
    }

    pub(super) fn original_timing_dialogue_execution_progress(
        &self,
    ) -> Option<crate::DialogueExecutionProgress> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_ref()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple dialogue execution outcomes",
        );
        matches.first().map(|&(_, progress)| progress)
    }

    pub(super) fn take_original_timing_dialogue_execution_progress(
        &mut self,
    ) -> Option<crate::DialogueExecutionProgress> {
        let expected = self.original_timing_dialogue_execution_progress()?;
        let receipts = self
            .original_timing_semantic_receipts
            .as_mut()
            .expect("the inspected dialogue execution receipt disappeared");
        let index = receipts
            .semantic
            .iter()
            .position(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DialogueExecutionProgress(progress)
                        if *progress == expected
                )
            })
            .expect("the inspected dialogue execution receipt changed before consumption");
        Some({
            receipts.semantic.remove(index);
            expected
        })
    }

    /// Validate the source VWF progress which may accompany the terminal host
    /// of an already-scheduled `DialogueVwfReturn` caller.
    ///
    /// The translated VWF body completed before it scheduled this continuation;
    /// only the caller suffix remains. The adapter can nevertheless report the
    /// decoder endpoint reached during the source host interval which finally
    /// returns that same caller. Treat it as corroboration only at exact native
    /// equality. A current-glyph receipt or even a one-byte cursor lead/lag is a
    /// real call-stack divergence and must remain fail-closed.
    pub(super) fn original_timing_pre_main_vwf_return_progress(
        &self,
    ) -> Option<crate::DialogueExecutionProgress> {
        let progress = self.original_timing_dialogue_execution_progress()?;
        assert!(
            self.game_execution_scheduler
                .pre_main_caller_continuation_is(PreMainCallerContinuation::DialogueVwfReturn),
            "source VWF return progress requires the resident DialogueVwfReturn caller",
        );
        assert!(
            self.dialogue_fast_forward_hold_active,
            "source VWF return progress requires the resident caller hold",
        );
        assert!(
            !progress.current_glyph_started(),
            "a completed DialogueVwfReturn caller cannot own a source glyph-prefix receipt: {progress:?}",
        );
        assert_eq!(
            self.game_state.messaging.runtime.dialogue_msg_read_pos(),
            progress.message_read_position(),
            "native VWF decoder cursor disagrees with the source caller-return endpoint",
        );
        assert!(
            matches!(
                self.dialogue_vwf_glyph_cpu_phase,
                messaging::VwfGlyphCpuPhase::Ready
            ),
            "source VWF caller return cannot overlap a translated glyph body",
        );
        assert!(
            self.dialogue_scroll_cpu_is_idle(),
            "source VWF caller return cannot overlap a message-line scroll continuation",
        );
        assert!(
            !self.dialogue_fast_forward_hold_pending,
            "source VWF caller return cannot retain a second caller-suffix hold",
        );
        assert_eq!(
            self.dialogue_live_message_read_position_target, None,
            "source VWF caller return cannot overlap an endpoint catch-up target",
        );
        Some(progress)
    }

    pub(super) fn take_original_timing_pre_main_vwf_return_progress(
        &mut self,
        expected: crate::DialogueExecutionProgress,
    ) {
        assert_eq!(
            self.original_timing_pre_main_vwf_return_progress(),
            Some(expected),
            "validated source VWF caller-return progress changed before consumption",
        );
        assert_eq!(
            self.take_original_timing_dialogue_execution_progress(),
            Some(expected),
            "validated source VWF caller-return progress disappeared before consumption",
        );
    }

    pub(super) fn take_original_timing_dialogue_closed(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                matches!(receipt, OriginalTimingSemanticReceipt::DialogueClosed).then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot close dialogue twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn publish_original_timing_presented_dialogue_text(
        &mut self,
        receipt: crate::PresentedDialogueText,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_dialogue_text_override = Some(receipt);
    }

    pub(super) fn apply_original_timing_presented_dialogue_text(
        &mut self,
        receipt: &crate::PresentedDialogueText,
    ) -> bool {
        const START_WORD: usize = 0x7c00;
        let end_word = START_WORD + crate::PresentedDialogueText::WORD_COUNT;
        let native_bg = self.ppu.bg_vram_latch.as_deref().unwrap_or(&self.ppu.vram);
        let native = &native_bg[START_WORD..end_word];
        let first_mismatch = native
            .iter()
            .zip(&receipt.words)
            .position(|(native, authority)| native != authority);
        let mismatched_words = native
            .iter()
            .zip(&receipt.words)
            .filter(|(native, authority)| native != authority)
            .count();
        self.original_timing_dialogue_text_shadow_result =
            Some(crate::OriginalTimingDialogueTextShadowResult {
                compared_words: receipt.words.len(),
                mismatched_words,
                first_mismatch,
            });

        self.ppu.vram[START_WORD..end_word].copy_from_slice(&receipt.words);
        if let Some(bg_vram) = self.ppu.bg_vram_latch.as_mut() {
            bg_vram[START_WORD..end_word].copy_from_slice(&receipt.words);
        }
        mismatched_words != 0
    }

    pub fn last_original_timing_dialogue_text_shadow_result(
        &self,
    ) -> Option<crate::OriginalTimingDialogueTextShadowResult> {
        self.original_timing_dialogue_text_shadow_result
    }

    pub(super) fn stage_rescue_follower_message_obj_scanout(
        &mut self,
        event_bit: u8,
        message: u16,
    ) {
        if !self.rom_startup_timing() {
            return;
        }
        let Some(scanout) = rescue_follower_message_obj_scanout(
            self.game_state.frame,
            self.game_state.world.location.dungeon_room_index(),
            event_bit,
            message,
        ) else {
            return;
        };

        // The room-$61 east rescue prompt switches to Module $0e just before
        // vblank interrupts Sprite_Main's caller. Hardware therefore scans
        // out the preceding published Link pose and host-boundary Link graphics
        // once. Other sprites already belong to the current dialogue frame, so
        // retaining the complete resident OAM generation is too broad.
        self.set_next_display_obj_scanout(Some(scanout));
    }

    pub(crate) fn dialogue_text_scanout_from_render_buffer(&self) -> DialogueTextScanout {
        let buffer = self.background_character_buffer();
        DialogueTextScanout {
            vram: (0..0x3f0)
                .map(|index| read_word_from_slice(buffer, index * 2))
                .collect(),
            glyph_runs: self.bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: self.bg3_vwf_glyph_run_dialogue_offsets.clone(),
            dialogue_msg_read_pos: self.game_state.messaging.runtime.dialogue_msg_read_pos(),
            dialogue_message_id: self.bg3_vwf_glyph_run_dialogue_message_id,
        }
    }

    pub(crate) fn dialogue_text_scanout_from_published_display(&self) -> DialogueTextScanout {
        DialogueTextScanout {
            vram: self.ppu.vram[0x7c00..0x7ff0].to_vec(),
            glyph_runs: self.published_bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: self.published_bg3_vwf_glyph_run_dialogue_offsets.clone(),
            dialogue_msg_read_pos: self.published_dialogue_msg_read_pos,
            dialogue_message_id: self.published_dialogue_message_id,
        }
    }

    pub(super) fn dialogue_scroll_machine_mut(&mut self) -> DialogueScrollMachineMut<'_> {
        DialogueScrollMachineMut {
            continuation: &mut self.dialogue_scroll_continuation,
            frozen_scanout: &mut self.dialogue_scroll_frozen_scanout,
            publication: &mut self.dialogue_scanout_ownership,
            completion_scanout: &mut self.dialogue_scroll_completion_scanout,
            staged_completion: &mut self.dialogue_scroll_completion_staged,
        }
    }

    pub(super) fn dialogue_scroll_phase(&self) -> DialogueScrollPhase {
        dialogue_scroll_phase(
            self.dialogue_scroll_continuation,
            self.dialogue_scanout_ownership,
            self.dialogue_scroll_frozen_scanout.is_some(),
            self.dialogue_scroll_completion_scanout.is_some(),
            self.dialogue_scroll_completion_staged.is_some(),
        )
    }

    pub(crate) fn dialogue_scroll_cpu_is_idle(&self) -> bool {
        self.dialogue_scroll_continuation.is_idle()
    }

    pub(super) fn dialogue_scroll_is_copying_remaining_pixels(&self) -> bool {
        matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CopyingRemainingPixels { .. }
        )
    }

    pub(super) fn dialogue_scroll_is_return_only(&self) -> bool {
        self.dialogue_scroll_phase() == DialogueScrollPhase::ReturnOnly
    }

    pub(crate) fn dialogue_scroll_holds_nmi_registers(&self) -> bool {
        matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CopyingRemainingPixels { .. } | DialogueScrollPhase::ReturnOnly
        )
    }

    pub(super) fn dialogue_scroll_is_completion_pending_publication(&self) -> bool {
        self.dialogue_scroll_phase() == DialogueScrollPhase::CompletionPendingPublication
    }

    /// OBJ scanout for a frame consumed by the multi-slice `Text_Initialize`
    /// worker. The OAM-law lane freezes across the initializer's held vblanks
    /// and presents the last completed main's transfer; only the Link OBJ CHR
    /// lanes still need their host-boundary generation staged here.
    pub(super) fn stage_dialogue_initialization_obj_scanout(&mut self) {
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(crate) fn begin_dialogue_scroll(
        &mut self,
        generation: DialogueTextGeneration,
        completion_timing: DialogueScrollCompletionTiming,
    ) {
        let frozen_scanout = match generation {
            DialogueTextGeneration::PublishedDisplay => {
                self.dialogue_text_scanout_from_published_display()
            }
            DialogueTextGeneration::CurrentRenderBuffer => {
                self.dialogue_text_scanout_from_render_buffer()
            }
        };
        if crate::debug_env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some() {
            let vram_sum = frozen_scanout
                .vram
                .iter()
                .map(|&word| u64::from(word & 0xff) + u64::from(word >> 8))
                .sum::<u64>();
            eprintln!(
                "scroll_freeze host={} generation={generation:?} vram_sum={vram_sum}",
                self.frame_ctr_dbg,
            );
        }
        self.dialogue_scroll_machine_mut()
            .begin_scroll(frozen_scanout, completion_timing);
    }

    pub(super) fn original_timing_dialogue_scroll_progress(
        &self,
    ) -> Option<crate::DialogueScrollProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .dialogue_scroll_progress
            .first()
            .copied()
    }

    pub(super) fn take_original_timing_dialogue_scroll_progress(
        &mut self,
        entered: bool,
    ) -> Option<crate::DialogueScrollProgressReceipt> {
        let progress = self.original_timing_dialogue_scroll_progress()?;
        assert_eq!(
            progress.entered, entered,
            "dialogue scroll receipt reached the wrong native call stage"
        );
        assert!(
            progress.completed_pixel_passes <= 16,
            "dialogue scroll receipt exceeds a text line"
        );
        self.original_timing_semantic_receipts
            .as_mut()
            .unwrap()
            .dialogue_scroll_progress
            .remove(0);
        Some(progress)
    }

    pub(super) fn take_terminal_dialogue_scroll_pixel_passes(&mut self) -> u16 {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return 3;
        }
        let progress = self
            .take_original_timing_dialogue_scroll_progress(false)
            .expect("a source-completed scroll requires its pixel-copy/return receipt");
        assert!(
            progress.returned,
            "a terminal scroll caller cannot precede its copy return"
        );
        u16::from(progress.completed_pixel_passes)
    }

    /// One lag host of a Module19 message-line scroll held by wire: apply
    /// this host's copy receipt and report whether `RenderText_Draw_Scroll`
    /// returned. Mirrors the Module0E terminal without its
    /// Module0E_Interface scroll-register suffix.
    pub(super) fn advance_triforce_room_dialogue_scroll_lag_host(
        &mut self,
        completion_timing_if_returned: DialogueScrollCompletionTiming,
    ) -> bool {
        debug_assert!(self.dialogue_scroll_is_copying_remaining_pixels());
        let Some(progress) = self.take_original_timing_dialogue_scroll_progress(false) else {
            // A host spent entirely in the suspended NMI handler has no
            // source scroll-copy operation to publish.
            return false;
        };
        let passes = u16::from(progress.completed_pixel_passes);
        if !progress.returned {
            self.render_text_scroll_pixels(passes);
            return false;
        }
        // The caller names how the RTS relates to this host's vblank: a
        // terminal host that also completes the shared suffix returned before
        // it, so the completed text generation publishes at the captured
        // display boundary; otherwise a return-only host follows.
        let completion_timing = completion_timing_if_returned;
        self.dialogue_scroll_machine_mut()
            .finish_remaining_pixels(Some(completion_timing));
        let command_done = self.render_text_scroll_pixels(passes);
        if command_done {
            let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            self.messaging_state_mut()
                .set_dialogue_msg_read_pos(read_pos.wrapping_add(1));
        }
        self.finish_dialogue_character_render_call();
        true
    }

    pub(super) fn finish_dialogue_scroll_remaining_pixels(
        &mut self,
    ) -> DialogueScrollCompletionTiming {
        self.dialogue_scroll_machine_mut()
            .finish_remaining_pixels(None)
    }

    pub(super) fn finish_dialogue_scroll_remaining_pixels_with_main_loop_receipt(
        &mut self,
        progress: crate::MainLoopProgress,
    ) -> DialogueScrollCompletionTiming {
        let completion_timing = match progress {
            crate::MainLoopProgress::IterationStarted => {
                DialogueScrollCompletionTiming::BeforeNextVblank
            }
            crate::MainLoopProgress::CallStackContinued => {
                DialogueScrollCompletionTiming::AfterReturnBoundary
            }
        };
        self.dialogue_scroll_machine_mut()
            .finish_remaining_pixels(Some(completion_timing))
    }

    pub(super) fn finish_dialogue_scroll_return(&mut self) {
        self.dialogue_scroll_machine_mut().finish_return();
    }

    /// Victory-module submodules (Module15 KillAghanim_Func7/8, Module16)
    /// call RenderText BEFORE the module's Sprite_Main/LinkOam suffix, the
    /// reverse of Module0E_Interface. When vblank suspends that iteration
    /// inside a begun message-line scroll copy, the continued host copies the
    /// remaining passes and returns through RenderText first; only then does
    /// the caller reach Sprite_Main (route host 315663: [NmiHandlerCompleted,
    /// SpriteMainReturned, CallStackContinued, MainLoopCommonSuffixCompleted]
    /// after [IterationStarted, NmiAccepted(LatchHeld)] with no Sprite_Main
    /// return). Mirrors the Continued-receipt scroll terminal for Module0E
    /// without its scroll-register suffix.
    pub(super) fn complete_victory_module_dialogue_scroll_before_sprite_main(&mut self) {
        if self.dialogue_scroll_cpu_is_idle() || !self.dialogue_scroll_is_copying_remaining_pixels()
        {
            return;
        }
        let completion_timing = self
            .finish_dialogue_scroll_remaining_pixels_with_main_loop_receipt(
                crate::MainLoopProgress::CallStackContinued,
            );
        assert_eq!(
            completion_timing,
            DialogueScrollCompletionTiming::AfterReturnBoundary,
            "a Continued victory-module scroll must complete after its return boundary",
        );
        let passes = self.take_terminal_dialogue_scroll_pixel_passes();
        let command_done = self.render_text_scroll_pixels(passes);
        if command_done {
            let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            self.messaging_state_mut()
                .set_dialogue_msg_read_pos(read_pos.wrapping_add(1));
        }
        self.finish_dialogue_character_render_call();
        self.finish_dialogue_scroll_return();
        let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
        self.stage_dialogue_scroll_completion_after_return(completed_scanout);
    }

    /// Return from RenderText and Module0E before the common sprite-preparation
    /// suffix. That suffix may itself suspend after the scroll has returned.
    pub(super) fn complete_module0e_dialogue_scroll_before_common_suffix(&mut self) {
        assert_eq!(self.game_state.frame.main_module, 0x0e);
        assert!(self.dialogue_scroll_is_copying_remaining_pixels());
        let passes = self.take_terminal_dialogue_scroll_pixel_passes();
        let completion_timing = self
            .finish_dialogue_scroll_remaining_pixels_with_main_loop_receipt(
                crate::MainLoopProgress::CallStackContinued,
            );
        assert_eq!(
            completion_timing,
            DialogueScrollCompletionTiming::AfterReturnBoundary
        );
        if self.render_text_scroll_pixels(passes) {
            let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos();
            self.messaging_state_mut()
                .set_dialogue_msg_read_pos(read_pos.wrapping_add(1));
        }
        self.finish_dialogue_character_render_call();
        self.complete_module0e_interface_after_run();
        self.finish_dialogue_scroll_return();
        let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
        self.stage_dialogue_scroll_completion_after_return(completed_scanout);
    }

    pub(super) fn stage_dialogue_scroll_completion_after_return(
        &mut self,
        completed_scanout: DialogueTextScanout,
    ) {
        if crate::debug_env::var_os("ZELDA3_DEBUG_SCROLL_STAGE").is_some() {
            eprintln!(
                "[SCROLL] stage_after_return host={} words={:04x?}",
                self.frame_ctr_dbg,
                &completed_scanout.vram[0..4],
            );
        }
        self.dialogue_scroll_machine_mut()
            .stage_completion_after_return(completed_scanout);
    }

    pub(super) fn stage_early_dialogue_scroll_completion(
        &mut self,
        completed_scanout: DialogueTextScanout,
    ) {
        if crate::debug_env::var_os("ZELDA3_DEBUG_SCROLL_STAGE").is_some() {
            eprintln!(
                "[SCROLL] stage_early host={} words={:04x?}",
                self.frame_ctr_dbg,
                &completed_scanout.vram[0..4],
            );
        }
        self.dialogue_scroll_machine_mut()
            .stage_early_completion(completed_scanout);
    }

    pub(super) fn advance_dialogue_scroll_display_boundary(&mut self) {
        let retiring_text_dma = (self.dialogue_scroll_phase()
            == DialogueScrollPhase::CompletedScroll)
            .then(|| self.dialogue_text_scanout_from_render_buffer());
        self.dialogue_scroll_machine_mut()
            .advance_display_boundary(retiring_text_dma);
    }

    pub(super) fn complete_dialogue_scroll_after_text_dma_publication(
        &mut self,
        token: DialogueTextDmaPublicationToken,
    ) -> bool {
        if !matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CompletionStagedAfterFrozenScanout
                | DialogueScrollPhase::CompletionStagedAfterSnapshot
        ) {
            return false;
        }
        let staged_completion = self
            .dialogue_scroll_completion_staged
            .clone()
            .expect("staged dialogue completion lost its exact text generation");
        let active = self
            .display_snapshot
            .as_ref()
            .expect("staged dialogue publication lost its active display snapshot");
        assert_eq!(
            active.publication_epoch, token.snapshot_epoch,
            "a staged dialogue completion cannot reuse a BG3 receipt from another snapshot epoch",
        );
        assert_eq!(
            token.words.len(),
            crate::PresentedDialogueText::WORD_COUNT,
            "dialogue publication did not cover the complete BG3 text-DMA range",
        );
        if crate::debug_env::var_os("ZELDA3_DEBUG_SCROLL_STAGE").is_some() {
            let differing = token
                .words
                .iter()
                .enumerate()
                .filter(|&(ref offset, &value)| value != staged_completion.vram[*offset])
                .count();
            eprintln!(
                "[SCROLL] publish host={} phase={:?} differing={} token={:04x?} staged={:04x?}",
                self.frame_ctr_dbg,
                self.dialogue_scroll_phase(),
                differing,
                &token.words[0..4],
                &staged_completion.vram[0..4],
            );
        }
        assert!(
            token
                .words
                .iter()
                .enumerate()
                .all(|(offset, &value)| value == staged_completion.vram[offset]),
            "dialogue publication used a BG3 text generation other than the staged source completion",
        );
        assert_eq!(
            token.metadata,
            PublishedDialogueMetadata::from_scanout(&staged_completion),
            "dialogue publication lost the semantic metadata paired with its BG3 text DMA",
        );
        self.dialogue_scroll_machine_mut()
            .complete_staged_text_dma();
        true
    }

    pub(super) fn consume_staged_dialogue_text_dma_publication(
        &mut self,
        token: Option<DialogueTextDmaPublicationToken>,
    ) -> Option<DialogueTextDmaPublicationToken> {
        match token {
            Some(token)
                if matches!(
                    self.dialogue_scroll_phase(),
                    DialogueScrollPhase::CompletionStagedAfterFrozenScanout
                        | DialogueScrollPhase::CompletionStagedAfterSnapshot
                ) =>
            {
                assert!(self.complete_dialogue_scroll_after_text_dma_publication(token));
                None
            }
            token => token,
        }
    }

    pub(super) fn displayed_dialogue_scanout(&self) -> Option<DialogueTextScanout> {
        match self.dialogue_scroll_phase() {
            DialogueScrollPhase::CopyingRemainingPixels { .. }
            | DialogueScrollPhase::ReturnOnly
            | DialogueScrollPhase::CompletionPendingPublication
            | DialogueScrollPhase::CompletionStagedAfterFrozenScanout => {
                self.dialogue_scroll_frozen_scanout.clone()
            }
            DialogueScrollPhase::CompletedScroll | DialogueScrollPhase::RetiredTextDma => {
                self.dialogue_scroll_completion_scanout.clone()
            }
            DialogueScrollPhase::Idle | DialogueScrollPhase::CompletionStagedAfterSnapshot => None,
        }
    }

    pub(super) fn resolve_displayed_dialogue_metadata(
        &self,
        pristine_snapshot: &DisplaySnapshot,
        dialogue_scanout: Option<&DialogueTextScanout>,
        vram_generation: DisplayVramGeneration,
    ) -> PublishedDialogueMetadata {
        if let Some(scanout) = dialogue_scanout {
            PublishedDialogueMetadata::from_scanout(scanout)
        } else if let Some(metadata) = pristine_snapshot
            .effective_presented_dma
            .as_ref()
            .and_then(|receipt| receipt.completed_dialogue_metadata.as_ref())
        {
            metadata.clone()
        } else if vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi {
            PublishedDialogueMetadata::from_snapshot(pristine_snapshot)
        } else {
            PublishedDialogueMetadata::from_live_state(self)
        }
    }

    pub(super) fn dialogue_text_dma_publication_token(
        &self,
        writes: &EffectiveDmaWriteSet,
    ) -> Option<DialogueTextDmaPublicationToken> {
        let dialogue_publication_candidate = matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CopyingRemainingPixels { .. }
                | DialogueScrollPhase::CompletionPendingPublication
                | DialogueScrollPhase::CompletionStagedAfterFrozenScanout
                | DialogueScrollPhase::CompletionStagedAfterSnapshot
        );
        dialogue_publication_candidate
            .then_some(writes.active_snapshot_epoch)
            .flatten()
            .and_then(|snapshot_epoch| {
                let metadata = writes.completed_dialogue_metadata.clone()?;
                writes
                    .vram_words
                    .get(0x7c00..0x7ff0)
                    .is_some_and(|written| written.iter().all(|&written| written))
                    .then(|| DialogueTextDmaPublicationToken {
                        snapshot_epoch,
                        words: self.ppu.vram[0x7c00..0x7ff0].to_vec(),
                        metadata,
                    })
            })
    }

    pub(super) fn record_completed_dialogue_metadata_for_display_boundary(&mut self) {
        let metadata = PublishedDialogueMetadata::from_live_state(self);
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            writes.completed_dialogue_metadata = Some(metadata);
        }
    }

    pub fn zelda_debug_display_publication_context(
        &self,
    ) -> Option<&DebugDisplayPublicationContext> {
        self.debug_display_publication_context.as_ref()
    }

    pub fn bg3_vwf_glyph_runs(&self) -> &[Bg3VwfGlyphRun] {
        &self.bg3_vwf_glyph_runs
    }

    pub fn published_bg3_vwf_glyph_runs(&self) -> &[Bg3VwfGlyphRun] {
        &self.published_bg3_vwf_glyph_runs
    }

    pub fn bg3_vwf_glyph_run_dialogue_offsets(&self) -> &[u16] {
        &self.bg3_vwf_glyph_run_dialogue_offsets
    }

    pub fn published_bg3_vwf_glyph_run_dialogue_offsets(&self) -> &[u16] {
        &self.published_bg3_vwf_glyph_run_dialogue_offsets
    }

    pub fn published_dialogue_message_id(&self) -> u16 {
        self.published_dialogue_message_id
    }

    pub fn dialogue_ir_for_decoded_bytes(
        &self,
        decoded: &[u8],
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        zelda3_compat::legacy_dialogue_ir(self.dialogue_flags, decoded)
    }

    pub fn current_dialogue_ir(&self) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        zelda3_compat::legacy_dialogue_ir(
            self.dialogue_flags,
            self.game_state.messaging.decoded_text.as_slice(),
        )
    }

    pub fn current_dialogue_message_id(&self) -> u16 {
        self.game_state.messaging.dialogue_message_index.value()
    }

    /// Whether BG3 is currently owned by the dialogue renderer.
    ///
    /// The selected message id and decoded glyph state intentionally survive
    /// after a box closes, so neither is a valid presentation signal by itself.
    pub fn is_dialogue_display_active(&self) -> bool {
        self.game_state.messaging.runtime.module() != 0
    }

    pub fn set_current_dialogue_message_id(&mut self, message_id: u16) {
        self.dialogue_message_index_mut().set_value(message_id);
    }

    pub fn current_source_dialogue_ir(&self) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        self.source_dialogue_ir_for_message(self.current_dialogue_message_id())
            .unwrap_or_default()
    }

    pub fn current_dialogue_runtime_substitutions(
        &self,
    ) -> crate::dialogue_ir::DialogueRuntimeSubstitutions {
        let mut player_name = Vec::new();
        self.text_write_player_name_vec(&mut player_name);
        crate::dialogue_ir::DialogueRuntimeSubstitutions {
            player_name,
            number_pairs: [
                self.game_state.messaging.dialogue_number.packed_digits(0),
                self.game_state.messaging.dialogue_number.packed_digits(1),
            ],
        }
    }

    pub fn current_source_render_dialogue_ir(&self) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        let source_ir = self.current_source_dialogue_ir();
        if source_ir.is_empty() {
            return Vec::new();
        }
        crate::dialogue_ir::expand_runtime_dialogue_ir(
            &source_ir,
            &self.current_dialogue_runtime_substitutions(),
        )
    }

    pub fn current_visible_source_render_dialogue_ir(
        &self,
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        let source_ir = self.current_source_dialogue_ir();
        if source_ir.is_empty() {
            return Vec::new();
        }
        let render_ir = crate::dialogue_ir::expand_runtime_render_dialogue_ir(
            &source_ir,
            &self.current_dialogue_runtime_substitutions(),
        );
        crate::dialogue_ir::visible_dialogue_ir_prefix(
            &render_ir,
            usize::from(self.game_state.messaging.runtime.dialogue_msg_read_pos()),
        )
    }

    /// Dialogue render IR that is actually on screen right now.
    ///
    /// Returns empty unless the text engine is currently handling a message
    /// (`MESSAGING_MODULE != 0`). The message id, read position, cached layout, and even the
    /// game's live BG3 glyph-run list all persist after a message closes, so a renderer keying off
    /// any of those would paint phantom glyphs over a closed box. The messaging module is the
    /// game's own "message open/rendering" state — what classic's BG3 content reflects — so gating
    /// on it keeps the hi-res VWF overlay in step with what the box actually shows.
    pub fn current_displayed_source_render_dialogue_ir(
        &self,
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        if !self.is_dialogue_display_active() {
            return Vec::new();
        }
        self.current_visible_source_render_dialogue_ir()
    }

    /// Source dialogue semantics corresponding to the BG3 generation most
    /// recently committed by NMI_UploadBG3Text.
    pub fn published_displayed_source_render_dialogue_ir(
        &self,
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        if !self.is_dialogue_display_active() || self.published_bg3_vwf_glyph_runs.is_empty() {
            return Vec::new();
        }
        let Some(source_ir) =
            self.source_dialogue_ir_for_message(self.published_dialogue_message_id)
        else {
            return Vec::new();
        };
        let render_ir = crate::dialogue_ir::expand_runtime_render_dialogue_ir(
            &source_ir,
            &self.current_dialogue_runtime_substitutions(),
        );
        crate::dialogue_ir::visible_dialogue_ir_prefix(
            &render_ir,
            usize::from(self.published_dialogue_msg_read_pos),
        )
    }

    pub fn source_dialogue_ir_for_message(
        &self,
        message_id: u16,
    ) -> Option<Vec<crate::dialogue_ir::DialogueIrOp>> {
        self.assets
            .as_ref()
            .and_then(|assets| assets.source_dialogue_ir_for_message(message_id))
    }

    pub fn dialogue_vwf_widths(&self) -> Option<Vec<u8>> {
        let dialogue_font = self.asset_memblk(95, self.dialogue_font_blk_index)?;
        Some(find_index_in_memblk(dialogue_font, 1).ptr.to_vec())
    }

    pub fn dialogue_vwf_origin_tile_number(&self) -> u16 {
        self.game_state
            .messaging
            .vwf_render
            .tile_word_at_byte_offset(0)
            & 0x03ff
    }

    pub fn bg3_vwf_glyph_run_dialogue_ir(
        &self,
        run_index: usize,
    ) -> Option<crate::dialogue_ir::DialogueIrOp> {
        zelda3_compat::legacy_glyph_run_dialogue_ir(
            self.dialogue_flags,
            self.game_state.messaging.decoded_text.as_slice(),
            &self.bg3_vwf_glyph_run_dialogue_offsets,
            run_index,
        )
    }

    pub fn published_bg3_vwf_glyph_run_dialogue_ir(
        &self,
        run_index: usize,
    ) -> Option<crate::dialogue_ir::DialogueIrOp> {
        let offset = *self
            .published_bg3_vwf_glyph_run_dialogue_offsets
            .get(run_index)?;
        if offset == zelda3_compat::UNKNOWN_DIALOGUE_OFFSET {
            return None;
        }
        self.published_displayed_source_render_dialogue_ir()
            .into_iter()
            .find(|op| op.offset == usize::from(offset))
    }

    pub fn restore_bg3_vwf_glyph_runs(&mut self, runs: Vec<Bg3VwfGlyphRun>) {
        self.bg3_vwf_glyph_runs = runs;
        self.bg3_vwf_glyph_run_dialogue_offsets =
            vec![zelda3_compat::UNKNOWN_DIALOGUE_OFFSET; self.bg3_vwf_glyph_runs.len()];
    }

    pub(crate) fn clear_bg3_vwf_glyph_runs(&mut self) {
        self.bg3_vwf_glyph_runs.clear();
        self.bg3_vwf_glyph_run_dialogue_offsets.clear();
    }

    pub(crate) fn publish_bg3_vwf_glyph_runs(&mut self) {
        self.published_bg3_vwf_glyph_runs
            .clone_from(&self.bg3_vwf_glyph_runs);
        self.published_bg3_vwf_glyph_run_dialogue_offsets
            .clone_from(&self.bg3_vwf_glyph_run_dialogue_offsets);
        self.published_dialogue_msg_read_pos =
            self.game_state.messaging.runtime.dialogue_msg_read_pos();
        self.published_dialogue_message_id = self.bg3_vwf_glyph_run_dialogue_message_id;
    }

    pub(crate) fn record_bg3_vwf_glyph_run(
        &mut self,
        glyph_code: u8,
        glyph_x: u8,
        line_ptr: usize,
        width: u8,
        dialogue_offset: u16,
    ) {
        const TEXT_TILE_ROW_BYTES: usize = 0x150;
        const TILE_PIXEL_WIDTH: usize = 8;

        if width == 0 {
            return;
        }

        self.bg3_vwf_glyph_run_dialogue_message_id = self.current_dialogue_message_id();

        let tile_row = line_ptr / TEXT_TILE_ROW_BYTES;
        let origin_tile_number = self
            .game_state
            .messaging
            .vwf_render
            .tile_word_at_byte_offset(0)
            & 0x03ff;
        self.bg3_vwf_glyph_runs.push(Bg3VwfGlyphRun {
            glyph_code: u16::from(glyph_code),
            origin_tile_number,
            x: i16::from(glyph_x),
            y: (tile_row * TILE_PIXEL_WIDTH) as i16,
            width,
        });
        self.bg3_vwf_glyph_run_dialogue_offsets
            .push(dialogue_offset);
    }

    pub(crate) fn scroll_bg3_vwf_glyph_runs_up_one_pixel(&mut self) {
        for run in &mut self.bg3_vwf_glyph_runs {
            run.y -= 1;
        }
        let mut next_runs = Vec::with_capacity(self.bg3_vwf_glyph_runs.len());
        let mut next_offsets = Vec::with_capacity(self.bg3_vwf_glyph_run_dialogue_offsets.len());
        for (index, run) in self.bg3_vwf_glyph_runs.iter().copied().enumerate() {
            if run.y > -16 {
                next_runs.push(run);
                next_offsets.push(
                    self.bg3_vwf_glyph_run_dialogue_offsets
                        .get(index)
                        .copied()
                        .unwrap_or(zelda3_compat::UNKNOWN_DIALOGUE_OFFSET),
                );
            }
        }
        self.bg3_vwf_glyph_runs = next_runs;
        self.bg3_vwf_glyph_run_dialogue_offsets = next_offsets;
    }

    pub(super) fn zelda_run_game_loop_with_progress_and_dialogue_text_dma(
        &mut self,
        authoritative_main_loop_progress: Option<crate::MainLoopProgress>,
        dialogue_text_dma: &mut Option<DialogueTextDmaPublicationToken>,
        idle_suffix_action: Option<OriginalTimingIdleMainLoopSuffixAction>,
        dialogue_endpoint_transition: Option<messaging::SuspendedVwfEndpointTransition>,
    ) {
        let entry_frame = self.game_state.frame;
        let state_9_palette_filter_loop_master_cycles = (entry_frame.main_module == 7
            && entry_frame.submodule == 2
            && entry_frame.subsubmodule == 9
            && self.game_state.dungeon.torch.any_lights_out_request() != 0)
            .then(|| palette_filter_bounce_loop_master_cycles(self));
        let iteration_started = !matches!(
            authoritative_main_loop_progress,
            Some(crate::MainLoopProgress::CallStackContinued)
        );
        if iteration_started {
            self.game_execution_scheduler.begin_main_loop_iteration();
        }
        self.zelda_run_game_loop_body_with_dialogue_text_dma(
            authoritative_main_loop_progress,
            dialogue_text_dma,
            idle_suffix_action,
            dialogue_endpoint_transition,
        );
        if iteration_started && self.game_execution_scheduler.is_idle() {
            if let Some(continuation) = dungeon_supertile_state_9_caller_continuation(
                entry_frame,
                self.game_state.frame,
                state_9_palette_filter_loop_master_cycles,
            ) {
                // CPU continuation ownership is independent of how the main
                // iteration was entered. The same measured state-9 workload
                // may follow a leading NMI or an ordinary host boundary; in
                // either case its Module 7 suffix resumes through the next NMI
                // before a fresh state-10 iteration can tick the frame counter.
                self.game_execution_scheduler
                    .schedule_pre_main_nmi_resume(continuation);
            }
        }
        if iteration_started {
            self.game_execution_scheduler.finish_main_loop_iteration();
        }
    }

    /// Finish CPU-authored dialogue text only after the scanout which still
    /// owned the prior buffer has been captured. This is shared by ordinary
    /// main-then-NMI hosts and by CPU iterations which run after a leading NMI;
    /// only the location of the already-completed capture differs.
    pub(super) fn stage_pending_dialogue_scroll_completion_after_captured_boundary(&mut self) {
        if !self.dialogue_scroll_is_completion_pending_publication() {
            return;
        }
        let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
        self.stage_early_dialogue_scroll_completion(completed_scanout);
    }

    /// Modules whose main routine hosts the shared RenderText dialogue
    /// machinery: Module0E (submodules 2/11) and Module1B_SpawnSelect
    /// (submodule 2 — the save-quit "where to start" prompt, route hosts
    /// 160304-160319), Module12_Death submodule 8 (GameOver_Finalize_GAMEOVR,
    /// the save-and-continue prompt, route host 422578) and
    /// Module15_MirrorWarpFromAga submodules 7/8 (KillAghanim_Func7/8, route
    /// hosts 315323-315663).
    pub(super) fn frame_hosts_resident_render_text(&self) -> bool {
        matches!(
            (
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            ),
            (14, 2 | 11) | (27, 2) | (18, 8) | (21, 7 | 8)
        )
    }

    /// Modules whose iteration can own live dialogue receipts (see
    /// `frame_hosts_resident_render_text`).
    pub(super) fn frame_module_hosts_dialogue(&self) -> bool {
        matches!(self.game_state.frame.main_module, 14 | 27)
    }

    pub(super) fn original_timing_dialogue_message_endpoint(&self) -> Option<u16> {
        if !self.frame_module_hosts_dialogue()
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            return None;
        }
        // The temporary Live timing authority reports the Zelda-level decoder
        // endpoint reached by a source host interval which never began a fresh
        // main iteration. If the native owner is already at that endpoint, its
        // atomic C translation has completed exactly the same semantic prefix:
        // do not replay it as another Module0E iteration. A disagreement stays
        // on the native path for shadow diagnosis; no CPU provenance or source
        // state is copied into gameplay.
        let progress = self.original_timing_dialogue_execution_progress();
        progress.map(crate::DialogueExecutionProgress::message_read_position)
    }

    pub(super) fn take_original_timing_dialogue_message_endpoint(&mut self) -> Option<u16> {
        let expected = self.original_timing_dialogue_message_endpoint()?;
        let expected_progress = self
            .original_timing_dialogue_execution_progress()
            .expect("the inspected dialogue endpoint lost its execution progress");
        let consumed = self.take_original_timing_dialogue_execution_progress();
        assert_eq!(
            consumed,
            Some(expected_progress),
            "the inspected dialogue endpoint changed before consumption",
        );
        Some(expected)
    }

    /// Prove that the translated suspended VWF caller can reach the source
    /// decoder endpoint without executing any caller epilogue or scheduling
    /// replacement work. The probe runs on a full clone before host setup,
    /// NMI handling, audio, or scheduler normalization can mutate live state.
    pub(super) fn original_timing_suspended_vwf_endpoint_transition_plan(
        &self,
        message_read_position: u16,
        current_glyph_started: bool,
    ) -> messaging::SuspendedVwfEndpointTransition {
        assert!(
            self.dialogue_scroll_cpu_is_idle(),
            "a suspended VWF endpoint cannot overlap a message-line scroll continuation",
        );
        let scheduler_before = self.game_execution_scheduler;
        let suffix_before = self.pending_main_loop_common_suffix;
        let semantic_before = self.original_timing_semantic_receipts.clone();
        let frame_before = self.game_state.frame;
        let display_snapshot_epoch_before = self.display_snapshot_epoch;
        let display_snapshot_before = self.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
                snapshot.ppu.cgram.clone(),
                snapshot.ppu.oam.clone(),
            )
        });
        let live_ppu_before = (
            self.ppu.vram.clone(),
            self.ppu.cgram.clone(),
            self.ppu.oam.clone(),
        );
        let flags_before = (
            self.game_state.display.pending_nmi_subroutine,
            self.game_state.display.core_update_disable_flag,
        );
        let mut probe = self.clone();
        let transition = probe.advance_suspended_vwf_to_authoritative_endpoint(
            message_read_position,
            current_glyph_started,
        );
        assert_eq!(
            probe.game_execution_scheduler, scheduler_before,
            "a suspended VWF endpoint transition cannot schedule translated work",
        );
        assert_eq!(
            probe.pending_main_loop_common_suffix, suffix_before,
            "a suspended VWF endpoint transition cannot consume its caller suffix",
        );
        assert_eq!(
            probe.original_timing_semantic_receipts, semantic_before,
            "a suspended VWF endpoint probe cannot consume source receipt authority",
        );
        assert_eq!(
            probe.game_state.frame, frame_before,
            "a suspended VWF endpoint probe cannot enter another module or main-loop iteration",
        );
        assert_eq!(
            probe.display_snapshot_epoch, display_snapshot_epoch_before,
            "a suspended VWF endpoint probe cannot capture another display boundary",
        );
        assert_eq!(
            probe.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
                snapshot.ppu.cgram.clone(),
                snapshot.ppu.oam.clone(),
            )),
            display_snapshot_before,
            "a suspended VWF endpoint probe cannot refine its accepted display snapshot",
        );
        assert_eq!(
            (
                probe.ppu.vram.clone(),
                probe.ppu.cgram.clone(),
                probe.ppu.oam.clone(),
            ),
            live_ppu_before,
            "a suspended VWF endpoint probe cannot publish live PPU memory",
        );
        assert!(
            probe.dialogue_scroll_cpu_is_idle(),
            "a suspended VWF endpoint probe cannot begin a message-line scroll continuation",
        );
        assert_eq!(
            (
                probe.game_state.display.pending_nmi_subroutine,
                probe.game_state.display.core_update_disable_flag,
            ),
            flags_before,
            "a suspended VWF endpoint transition cannot publish its character epilogue",
        );
        assert!(
            probe.game_state.messaging.runtime.dialogue_msg_read_pos() >= message_read_position,
            "a suspended VWF endpoint transition did not reach its source decoder endpoint",
        );
        if transition.advanced_native_vwf() {
            assert!(probe.dialogue_fast_forward_hold_pending);
            assert_eq!(
                probe.dialogue_live_message_read_position_target,
                Some(message_read_position),
            );
        } else {
            assert!(!probe.dialogue_fast_forward_hold_pending);
            assert_eq!(probe.dialogue_live_message_read_position_target, None);
        }
        if probe
            .original_timing_dialogue_scroll_progress()
            .is_some_and(|progress| progress.entered)
        {
            assert!(
                !current_glyph_started,
                "a source scroll entry cannot retain an active glyph"
            );
            assert_eq!(
                probe.game_state.messaging.runtime.dialogue_msg_read_pos(),
                message_read_position
            );
            // Validate the next source call on this disposable native state,
            // before the real endpoint or scroll receipt can be consumed.
            probe.dialogue_fast_forward_hold_pending = false;
            probe.dialogue_live_message_read_position_target = None;
            probe.begin_live_dialogue_scroll_after_vwf_endpoint();
        }
        transition
    }

    /// Prove that the resident suspended VWF handler can return without
    /// publishing any caller epilogue, display boundary, or replacement work.
    /// The later typed common-suffix receipt supersedes any translated
    /// caller-suffix raster budget reported by the final inner slice.
    pub(super) fn original_timing_suspended_vwf_completion_transition_plan(
        &self,
    ) -> messaging::SuspendedVwfCompletionTransition {
        assert!(
            self.dialogue_scroll_cpu_is_idle(),
            "a suspended VWF completion cannot overlap a message-line scroll continuation",
        );
        let scheduler_before = self.game_execution_scheduler;
        let suffix_before = self.pending_main_loop_common_suffix;
        let semantic_before = self.original_timing_semantic_receipts.clone();
        let frame_before = self.game_state.frame;
        let display_snapshot_epoch_before = self.display_snapshot_epoch;
        let display_snapshot_before = self.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
                snapshot.ppu.cgram.clone(),
                snapshot.ppu.oam.clone(),
            )
        });
        let live_ppu_before = (
            self.ppu.vram.clone(),
            self.ppu.cgram.clone(),
            self.ppu.oam.clone(),
        );
        let flags_before = (
            self.game_state.display.pending_nmi_subroutine,
            self.game_state.display.core_update_disable_flag,
        );
        let mut probe = self.clone();
        let transition = probe.advance_suspended_vwf_to_handler_completion();
        assert_eq!(
            probe.game_execution_scheduler, scheduler_before,
            "a suspended VWF completion cannot schedule translated work",
        );
        assert_eq!(
            probe.pending_main_loop_common_suffix, suffix_before,
            "a suspended VWF completion cannot consume its caller suffix",
        );
        assert_eq!(
            probe.original_timing_semantic_receipts, semantic_before,
            "a suspended VWF completion probe cannot consume source receipt authority",
        );
        assert_eq!(
            probe.game_state.frame, frame_before,
            "a suspended VWF completion cannot enter another module or main-loop iteration",
        );
        assert_eq!(
            probe.display_snapshot_epoch, display_snapshot_epoch_before,
            "a suspended VWF completion cannot capture another display boundary",
        );
        assert_eq!(
            probe.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
                snapshot.ppu.cgram.clone(),
                snapshot.ppu.oam.clone(),
            )),
            display_snapshot_before,
            "a suspended VWF completion cannot refine its accepted display snapshot",
        );
        assert_eq!(
            (
                probe.ppu.vram.clone(),
                probe.ppu.cgram.clone(),
                probe.ppu.oam.clone(),
            ),
            live_ppu_before,
            "a suspended VWF completion cannot publish live PPU memory",
        );
        assert_eq!(
            probe.dialogue_scroll_cpu_is_idle(),
            !transition.begins_message_line_scroll(),
            "a suspended VWF completion disagreed with its typed scroll-continuation transition",
        );
        assert_eq!(
            (
                probe.game_state.display.pending_nmi_subroutine,
                probe.game_state.display.core_update_disable_flag,
            ),
            flags_before,
            "a suspended VWF completion cannot publish its character epilogue",
        );
        assert!(
            probe.dialogue_fast_forward_hold_active,
            "a suspended VWF completion probe lost its resident caller hold",
        );
        assert_eq!(
            probe.dialogue_fast_forward_hold_pending,
            transition.caller_suffix_crossed_vblank(),
            "a suspended VWF completion probe lost its final timing-shadow disposition",
        );
        assert_eq!(
            probe.dialogue_live_message_read_position_target, None,
            "a suspended VWF completion probe created an endpoint target",
        );
        transition
    }

    /// Reach and consume one source endpoint while retaining the suspended VWF
    /// caller. The eventual terminal host separately owns the character
    /// epilogue, Module0E suffix, and ZeldaRunGameLoop common suffix.
    pub(super) fn complete_original_timing_suspended_vwf_endpoint(
        &mut self,
        message_read_position: u16,
        current_glyph_started: bool,
        expected_transition: messaging::SuspendedVwfEndpointTransition,
    ) {
        let transition = self.advance_suspended_vwf_to_authoritative_endpoint(
            message_read_position,
            current_glyph_started,
        );
        assert_eq!(
            transition, expected_transition,
            "suspended VWF endpoint execution drifted after immutable preflight",
        );
        assert!(
            self.game_state.messaging.runtime.dialogue_msg_read_pos() >= message_read_position,
            "suspended VWF execution did not reach its authoritative decoder endpoint",
        );
        if transition.advanced_native_vwf() {
            assert!(
                self.dialogue_fast_forward_hold_pending,
                "native VWF catch-up lost its authoritative boundary hold",
            );
            assert_eq!(
                self.dialogue_live_message_read_position_target,
                Some(message_read_position),
                "native VWF catch-up lost its authoritative decoder target",
            );
        }
        assert_eq!(
            self.take_original_timing_dialogue_message_endpoint(),
            Some(message_read_position),
            "validated dialogue endpoint changed before its source-ordered CPU boundary",
        );
        if transition.advanced_native_vwf() {
            self.dialogue_fast_forward_hold_pending = false;
            self.dialogue_live_message_read_position_target = None;
        }
        if self
            .original_timing_dialogue_scroll_progress()
            .is_some_and(|progress| progress.entered)
        {
            assert!(
                !current_glyph_started,
                "a VWF glyph and its following scroll cannot both own the source endpoint"
            );
            assert_eq!(
                self.game_state.messaging.runtime.dialogue_msg_read_pos(),
                message_read_position
            );
            self.begin_live_dialogue_scroll_after_vwf_endpoint();
            self.dialogue_fast_forward_hold_active = false;
        }
    }

    /// Complete the native caller suffix which follows a repeated VWF endpoint
    /// on the host that finally returns to ZeldaRunGameLoop.
    pub(super) fn complete_original_timing_terminal_dialogue_endpoint(
        &mut self,
        message_read_position: u16,
        current_glyph_started: bool,
        expected_transition: messaging::SuspendedVwfEndpointTransition,
    ) {
        self.complete_original_timing_suspended_vwf_endpoint(
            message_read_position,
            current_glyph_started,
            expected_transition,
        );
        self.complete_original_timing_terminal_dialogue_postlude();
    }

    /// Finish the native inner VWF caller represented by a prior host's
    /// endpoint, then run the same source postlude as a terminal endpoint host.
    pub(super) fn complete_original_timing_terminal_suspended_vwf(
        &mut self,
        expected_transition: messaging::SuspendedVwfCompletionTransition,
    ) {
        let transition = self.advance_suspended_vwf_to_handler_completion();
        assert_eq!(
            transition, expected_transition,
            "suspended VWF completion drifted after immutable preflight",
        );
        assert_eq!(
            self.dialogue_fast_forward_hold_pending,
            transition.caller_suffix_crossed_vblank(),
            "suspended VWF completion lost its final timing-shadow disposition",
        );
        // The typed same-host common-suffix completion supersedes a coarse
        // translated caller-suffix vblank. Do not schedule another host.
        self.dialogue_fast_forward_hold_pending = false;
        self.complete_original_timing_terminal_dialogue_postlude();
    }

    pub(super) fn complete_original_timing_terminal_dialogue_postlude(&mut self) {
        // The VWF body is already represented by the repeated endpoint. Its
        // callers still publish the completed character-render gate, return
        // through Module0E_Interface's scroll-register writes, and run the
        // ordinary post-module dialogue bookkeeping before the outer
        // ZeldaRunGameLoop common suffix.
        self.finish_dialogue_character_render_call();
        if self.frame_hosts_resident_render_text()
            && self.game_state.frame.main_module == 0x12
            && self.game_state.frame.submodule == 8
        {
            // GameOver_Finalize_GAMEOVR hosts RenderText through the
            // synchronous five-call RenderText_PostDeathSaveOptions loop, not
            // Module0E_Interface. The wire-proven same-host common suffix
            // means every remaining loop call returned inside this host (route
            // host 422578: the oracle advanced to submodule 9 here, the native
            // loop still held its final call for the next host).
            self.finish_game_over_text_render_call();
            while self.game_over_text_render_loop_active() {
                self.game_over_text_render_call_in_flight = true;
                self.Text_Render();
                assert!(
                    matches!(
                        self.dialogue_vwf_glyph_cpu_phase,
                        messaging::VwfGlyphCpuPhase::Ready
                    ) && self.dialogue_scroll_cpu_is_idle(),
                    "a terminal post-death dialogue host cannot suspend a remaining RenderText_PostDeathSaveOptions call mid-glyph: remaining={} render_state={} read_pos={:#x}",
                    self.game_over_text_render_calls_remaining,
                    self.game_state.messaging.runtime.text_render_state(),
                    self.game_state.messaging.runtime.dialogue_msg_read_pos(),
                );
                // The coarse caller-suffix vblank prediction is superseded by
                // the wire's same-host common-suffix completion.
                self.dialogue_fast_forward_hold_pending = false;
                self.finish_game_over_text_render_call();
            }
        } else {
            self.complete_module0e_interface_after_run();
        }
        assert!(
            !self.dialogue_fast_forward_hold_pending,
            "a terminal dialogue endpoint cannot retain another suspended VWF hold",
        );
        self.complete_zelda_run_game_loop_after_module_routing();
    }

    pub(super) fn original_timing_completed_dialogue_return_timeline(
        &self,
        owner: &'static str,
    ) -> OriginalTimingMainLoopReturnTimeline {
        let timeline = self
            .original_timing_main_loop_return_timeline()
            .unwrap_or_else(|| panic!("{owner} omitted its typed main-loop return"));
        assert_eq!(
            timeline.progress,
            crate::MainLoopProgress::CallStackContinued,
            "{owner} cannot begin a fresh main-loop iteration",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "{owner} lost its suspended ZeldaRunGameLoop suffix",
        );
        assert!(
            !self.main_loop_sprite_preparation_completed,
            "{owner} cannot repeat an already-completed NMI_PrepareSprites suffix",
        );
        assert!(
            self.original_timing_main_loop_interruption().is_none(),
            "{owner} cannot also interrupt its common suffix",
        );
        let expected_before = if self.original_timing_nmi_publication_pending {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
                "{owner} may only carry its source latch-held NMI",
            );
            assert!(
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                "{owner} lost its carried acceptance-host scanout",
            );
            vec![OriginalTimingNmiPhase::HandlerCompleted]
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "{owner} retained an NMI gate without a carried handler",
            );
            vec![
                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                OriginalTimingNmiPhase::HandlerCompleted,
            ]
        };
        assert_eq!(
            timeline.nmi_phases_before_return, expected_before,
            "{owner} has an unsupported leading NMI lifecycle: {timeline:?}",
        );
        assert!(
            self.game_state.display.nmi_update_is_latched(),
            "{owner}'s dialogue handler disagrees with the source latch-held disposition",
        );
        let before_return = try_classify_original_timing_nmi_phases(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_return,
        )
        .unwrap_or_else(|| panic!("{owner} has an invalid leading NMI lifecycle"));
        assert!(
            before_return.handler_completion.completed()
                && !before_return.publication_pending_at_exit,
            "{owner} must close exactly one held handler before its common suffix: {timeline:?}",
        );
        let trailing_open = timeline.nmi_phases_after_return
            == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)];
        assert!(
            timeline.nmi_phases_after_return.is_empty() || trailing_open,
            "{owner} may only carry one Open NMI after its common suffix: {timeline:?}",
        );
        let after_return =
            try_classify_original_timing_nmi_phases(false, &timeline.nmi_phases_after_return)
                .unwrap_or_else(|| panic!("{owner} has an invalid trailing NMI lifecycle"));
        assert_eq!(
            after_return.handler_completion,
            OriginalTimingNmiHandlerCompletionOwner::None,
            "{owner} cannot complete a post-suffix NMI",
        );
        assert_eq!(
            after_return.publication_pending_at_exit, trailing_open,
            "{owner} has inconsistent trailing Open publication ownership",
        );

        let mut expected_semantic = expected_before
            .iter()
            .map(|phase| match phase {
                OriginalTimingNmiPhase::Accepted(gate) => {
                    OriginalTimingSemanticReceipt::NmiAccepted(*gate)
                }
                OriginalTimingNmiPhase::HandlerCompleted => {
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted
                }
            })
            .collect::<Vec<_>>();
        expected_semantic.extend([
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ]);
        if trailing_open {
            expected_semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open,
            ));
        }
        assert_eq!(
            self.original_timing_semantic_receipts
                .as_ref()
                .unwrap_or_else(|| panic!("{owner} lost its semantic receipts"))
                .semantic(),
            expected_semantic,
            "{owner} requires one of the four exact source receipt vectors",
        );
        let mut expected_gates = vec![NmiUpdateGate::LatchHeld];
        if trailing_open {
            expected_gates.push(NmiUpdateGate::Open);
        }
        assert_eq!(
            self.original_timing_expected_nmi_update_gates, expected_gates,
            "{owner} lost its exact installed NMI dispositions",
        );
        timeline
    }

    pub(super) fn original_timing_dialogue_return_only_timeline(
        &self,
    ) -> Option<OriginalTimingMainLoopReturnTimeline> {
        if !self.rom_startup_timing()
            || !self.dialogue_scroll_is_return_only()
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            return None;
        }
        assert!(
            self.game_execution_scheduler.is_idle(),
            "a live return-only dialogue slice overlaps scheduled translated work: {:?}",
            self.game_execution_scheduler.current_work(),
        );
        assert_eq!(
            self.game_execution_scheduler.pre_main_caller_continuation(),
            None,
            "a live return-only dialogue slice overlaps a pre-main caller",
        );
        Some(self.original_timing_completed_dialogue_return_timeline(
            "a live return-only dialogue slice",
        ))
    }

    pub(super) fn take_original_timing_dialogue_terminal_return(
        &mut self,
    ) -> Option<OriginalTimingMainLoopReturnTimeline> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || !self.frame_module_hosts_dialogue()
            || self.original_timing_main_loop_progress()
                != Some(crate::MainLoopProgress::CallStackContinued)
            || self.original_timing_main_loop_return_timeline().is_none()
        {
            return None;
        }

        // `$0e:c58b` and `$0e:c6e3` are outside the adapter's true VWF endpoint
        // range. The exact AfterCurrent scheduler owner proves that the interrupted
        // Text_LoadCharacterBuffer caller is ready to resume. Depending on
        // raster position, its terminal host either completes a carried Held
        // handler or accepts and completes that Held handler in-sequence, and
        // may accept one trailing Open NMI after the common suffix.
        let expected_timeline = self.original_timing_completed_dialogue_return_timeline(
            "a live terminal dialogue caller return",
        );
        let consumed_timeline = self
            .take_original_timing_main_loop_return_timeline()
            .expect("validated dialogue terminal timeline disappeared");
        assert_eq!(consumed_timeline, expected_timeline);
        Some(consumed_timeline)
    }

    pub(super) fn zelda_run_game_loop_body_with_dialogue_text_dma(
        &mut self,
        authoritative_main_loop_progress: Option<crate::MainLoopProgress>,
        dialogue_text_dma: &mut Option<DialogueTextDmaPublicationToken>,
        idle_suffix_action: Option<OriginalTimingIdleMainLoopSuffixAction>,
        dialogue_endpoint_transition: Option<messaging::SuspendedVwfEndpointTransition>,
    ) {
        // The C suffix (`NMI_PrepareSprites(); nmi_boolean = 0`) runs only after
        // Module_MainRouting returns. A source host may begin a fresh iteration
        // and return while that call stack is still active. Capture the typed
        // return fact before the module gets a chance to consume it for a more
        // specific continuation.
        let authoritative_iteration_returned_to_wait = authoritative_main_loop_progress
            .map(|_| self.original_timing_main_loop_iteration_returned_to_wait());
        if !self.dialogue_scroll_cpu_is_idle() {
            // Lag frame of an in-flight message-line scroll: the ROM's main
            // loop is still inside the scroll copy, so nothing else runs —
            // no frame-counter tick, no OAM clear, no module routing. The
            // The measured continuation copies the remaining three pixels,
            // then returns through the RenderText handler after this frame's
            // NMI. Phase 1 is consumed separately by run_frame_internal as a
            // return-only slice so its NMI stays before the game-loop suffix.
            debug_assert!(self.dialogue_scroll_is_copying_remaining_pixels());
            let passes = if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
                let Some(progress) = self.take_original_timing_dialogue_scroll_progress(false)
                else {
                    // A host spent entirely in the suspended NMI handler has
                    // no source scroll-copy operation to publish.
                    return;
                };
                let passes = u16::from(progress.completed_pixel_passes);
                if !progress.returned {
                    self.render_text_scroll_pixels(passes);
                    return;
                }
                passes
            } else {
                3
            };
            // The entry-time cycle estimate is only a fallback for translated-only
            // execution. A Live timing owner directly reports whether this source
            // host returned through ZeldaRunGameLoop and began another iteration;
            // that semantic receipt decides which side of vblank owns completion.
            let completion_timing = match authoritative_main_loop_progress {
                Some(progress) => {
                    self.finish_dialogue_scroll_remaining_pixels_with_main_loop_receipt(progress)
                }
                None => self.finish_dialogue_scroll_remaining_pixels(),
            };
            let command_done = self.render_text_scroll_pixels(passes);
            // The slow $0e:cfe2 text-buffer copy has now returned through
            // RenderText_Draw_MessageCharacters and RunInterface. The ROM
            // advances past the scroll command only when the low nibble
            // wrapped, then publishes $17/$0710 at $0e:c9f9/$0e:c9fc.
            if command_done {
                let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos();
                self.messaging_state_mut()
                    .set_dialogue_msg_read_pos(read_pos.wrapping_add(1));
            }
            self.finish_dialogue_character_render_call();
            // Only at this measured continuation boundary does the ROM
            // execute Module0E_Interface's $00:f873 scroll-register suffix.
            self.complete_module0e_interface_after_run();
            if completion_timing == DialogueScrollCompletionTiming::BeforeNextVblank {
                // The v=2 oracle entry returns through Main_PrepSpritesForNmi
                // before the next vblank. Complete the caller suffix now; the
                // outer display boundary stages its text generation only after
                // preserving the scanout that ended before this publication.
                self.nmi_prepare_sprites_for_main_loop();
                self.clear_nmi_update_latch();
            }
            if !matches!(
                authoritative_main_loop_progress,
                Some(crate::MainLoopProgress::IterationStarted)
            ) {
                return;
            }
            debug_assert_eq!(
                completion_timing,
                DialogueScrollCompletionTiming::BeforeNextVblank,
                "the source cannot begin a new ZeldaRunGameLoop iteration before an after-vblank scroll continuation returns",
            );
            // This host began at the NMI which exposed the completed text
            // generation. Retire the frozen scanout at that already-captured
            // boundary before the new iteration can start another scroll;
            // otherwise the single semantic scroll owner would still be in
            // CompletionPendingPublication when Module0E re-enters.
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
            let token = dialogue_text_dma.take().expect(
                "a fresh iteration cannot follow dialogue completion before its Open BG3 text-DMA publication",
            );
            assert!(self.complete_dialogue_scroll_after_text_dma_publication(token));
            // The completed RenderText/Module0E caller returned with enough
            // headroom for the ROM to begin ZeldaRunGameLoop again before this
            // host interval ended. Continue into that new semantic iteration;
            // returning here would drop its frame-counter/OAM/module prefix.
        }
        if let Some(dialogue_progress) = self.original_timing_dialogue_execution_progress() {
            let message_read_position = dialogue_progress.message_read_position();
            let current_glyph_started = dialogue_progress.current_glyph_started();
            assert_eq!(
                authoritative_main_loop_progress,
                Some(crate::MainLoopProgress::CallStackContinued),
                "a source dialogue endpoint without a fresh iteration must remain inside its existing ZeldaRunGameLoop call",
            );
            assert!(
                self.original_timing_main_loop_return_timeline().is_none(),
                "a terminal dialogue return must be consumed by its source-ordered handler/caller/common-suffix owner",
            );
            assert!(
                matches!(
                    self.pending_main_loop_common_suffix,
                    None | Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                ),
                "a dialogue continuation cannot replace an in-flight specialized common suffix",
            );
            assert!(
                !self.main_loop_sprite_preparation_completed,
                "a suspended dialogue caller cannot own an already-completed NMI_PrepareSprites suffix",
            );
            let transition = dialogue_endpoint_transition.unwrap_or_else(|| {
                self.original_timing_suspended_vwf_endpoint_transition_plan(
                    message_read_position,
                    current_glyph_started,
                )
            });
            self.complete_original_timing_suspended_vwf_endpoint(
                message_read_position,
                current_glyph_started,
                transition,
            );
            // The source is still inside the same Module0E invocation.
            // Preserve the one unconditional ZeldaRunGameLoop suffix for the
            // terminal host which will eventually return this call.
            if let Some(action) = idle_suffix_action {
                self.apply_original_timing_idle_main_loop_suffix_action(action);
            } else {
                self.pending_main_loop_common_suffix.get_or_insert(
                    MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                );
            }
            return;
        }
        assert_eq!(
            dialogue_endpoint_transition, None,
            "a cloned VWF endpoint transition lost its source dialogue receipt",
        );
        if self.take_original_timing_dialogue_closed() {
            assert!(
                self.frame_module_hosts_dialogue(),
                "dialogue-close receipt reached native gameplay outside Module0E/Module1B",
            );
            // Let the existing C translation own every mutation: the border
            // clear packet, messaging-module reset, submodule reset, and the
            // return to the native saved-module field. The authority supplies
            // only the fact that this semantic branch was reached.
            self.messaging_state_mut().set_text_render_state(4);
        }
        // GameOver_Finalize_GAMEOVR's synchronous RenderText_PostDeathSaveOptions
        // loop is the other VWF call stack vblank suspends: its continued hosts
        // resume the same held module iteration Module0E uses (route hosts
        // 422571-422577 drew nothing natively while the ROM finished the
        // save-and-continue prompt).
        let death_prompt_render_loop_continues = self.game_state.frame.main_module == 0x12
            && self.game_state.frame.submodule == 8
            && self.game_over_text_render_loop_active();
        if matches!(
            authoritative_main_loop_progress,
            Some(crate::MainLoopProgress::CallStackContinued)
        ) && !self.frame_module_hosts_dialogue()
            && !death_prompt_render_loop_continues
        {
            // ZeldaRunGameLoop calls Module_MainRouting exactly once, after
            // incrementing frame_counter. A source host interval which stayed
            // inside that call stack cannot enter the module router again.
            // Long native work resumes through its typed scheduler owner;
            // rerouting here would start the next atomic stage before the C
            // caller returned (observed for PreOverworld_LoadOverlays).
            return;
        }
        // A held frame is one the ROM spends inside a fast-forward message
        // render slice: the NMI does the VWF text upload but skips the core
        // game update, so the frame counter and sprites (incl. Link's
        // animation) do not advance. `dialogue_fast_forward_hold_active` stays
        // set through `module_main_routing` so `Module0E_Interface` skips its
        // sprite/Link update, then rotates below.
        if matches!(
            authoritative_main_loop_progress,
            Some(crate::MainLoopProgress::CallStackContinued)
        ) && death_prompt_render_loop_continues
        {
            self.dialogue_fast_forward_hold_active = true;
        } else if matches!(
            authoritative_main_loop_progress,
            Some(crate::MainLoopProgress::CallStackContinued)
        ) && self.frame_module_hosts_dialogue()
        {
            if self.game_state.frame.submodule == 2 {
                // The source remained inside Module0E's existing VWF call
                // stack. Resume the translated semantic owner without running
                // a second ZeldaRunGameLoop prefix or advancing Link/sprites.
                self.dialogue_fast_forward_hold_active = true;
            } else if self.game_state.frame.submodule == 11 {
                // The save menu resumes through the router as well, with the
                // core update held so no unclaimed Sprite_Main runs: its
                // initialization hold is receipt-guarded inside
                // Module0E_0B_SaveMenu (an InProgress receipt returns before
                // any state moves), so held re-entry is the mechanism that
                // consumes the wire's SaveMenuInitializationProgress fact
                // (route host 47621).
                self.dialogue_fast_forward_hold_active = true;
            } else {
                // Only the VWF renderer resumes through the module router;
                // every other Module0E submodule owns its continued stack
                // inside the one Module_MainRouting call, exactly like the
                // non-messaging modules above. Re-routing here re-ran the
                // desert-prayer palette-filter tick once per continued host
                // and finished the iris fade four frames early (route frame
                // 123062, module 0e/05/03).
                return;
            }
        }
        let hold_core = self.dialogue_fast_forward_hold_active;
        let run_game_loop_prefix = authoritative_main_loop_progress
            .map(|progress| matches!(progress, crate::MainLoopProgress::IterationStarted))
            .unwrap_or(!hold_core);
        let frame = self.game_state.frame;
        if run_game_loop_prefix
            && self.rom_startup_timing()
            && frame.main_module == 7
            && ((frame.submodule == 2 && matches!(frame.subsubmodule, 2 | 13 | 14))
                || (rom_dungeon_landing_wipe_is_active(frame.main_module, frame.submodule)
                    && matches!(frame.subsubmodule, 0 | 1)))
        {
            if self.dungeon_landing_cpu_advance_pending.is_none() {
                let timing = begin_dungeon_landing_cpu_advance(self);
                self.dungeon_landing_spotlight_reset_prefix_scanlines =
                    timing.spotlight_reset_prefix_scanlines;
                self.dungeon_landing_cpu_advance_pending = Some(timing.advance);
            }
        } else if !(frame.main_module == 7
            && ((frame.submodule == 2 && matches!(frame.subsubmodule, 4..=7 | 12))
                || frame.submodule == 0x0f))
        {
            self.dungeon_landing_cpu_advance_pending = None;
            self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
        }
        if let (Some(interruption), Some(advance)) = (
            self.original_timing_main_loop_interruption(),
            self.dungeon_landing_cpu_advance_pending.as_mut(),
        ) {
            // This replaceable source receipt is later and more authoritative
            // than the native timing shadow: reaching LinkOam proves Sprite_Main
            // returned, and reaching NMI_PrepareSprites proves the whole module
            // returned. Feed that semantic phase into the existing translated C
            // continuation machinery; no backend PC or raster escapes here.
            let sprite_main_boundary = sprite_main_cpu_boundary_from_interruption(interruption);
            advance.phase = module_cpu_phase_from_main_loop_interruption(interruption)
                .unwrap_or_else(|| {
                    panic!("dungeon landing cannot consume Module0F Link-position progress")
                });
            if let Some(boundary) = sprite_main_boundary {
                advance.sprite_main_boundary = Some(boundary);
            }
            advance.resumed_phase = None;
        }
        if run_game_loop_prefix {
            // This is the entry edge of ZeldaRunGameLoop in the C program.
            // Resumed continuations deliberately skip this reset so the
            // eventual caller suffix can tell whether its one sprite-prep
            // pass already ran before suspension.
            self.main_loop_sprite_preparation_completed = false;
            self.increment_frame_counter();
            self.dungeon_palette_cpu_advance_pending =
                if self.rom_startup_timing() && self.game_state.frame.main_module == 7 {
                    match (
                        self.game_state.frame.submodule,
                        self.game_state.frame.subsubmodule,
                    ) {
                        (1, 1 | 6) => Some(dungeon_palette_caller_cpu_advance(
                            self,
                            DUNGEON_SUBTILE_PALETTE_CPU_ENTRY,
                        )),
                        _ => None,
                    }
                } else {
                    None
                };
            self.replay_trace_ram_watch("game-loop-after-frame-counter");
            self.clear_oam_buffer();
            self.replay_trace_ram_watch("game-loop-after-clear-oam");
        }
        self.module_main_routing();
        self.complete_zelda_run_game_loop_after_module_routing();
        if let Some(iteration_returned_to_wait) = authoritative_iteration_returned_to_wait {
            if iteration_returned_to_wait {
                // The source has now reached ZeldaRunGameLoop's unconditional
                // common suffix. Any atomic-runtime partial-host marker from
                // the call which just returned is no longer authoritative and
                // must not suppress this or a later iteration's sprite prep.
                self.rom_load_partial_nmi_this_frame = false;
                // If an earlier host suspended this same ZeldaRunGameLoop
                // call, the terminal source receipt owns its common suffix
                // even when another translated continuation remains queued.
                // Retire that one-shot before any scheduler early return can
                // strand `$12` set across the following accepted NMI.
                if !self.complete_pending_main_loop_common_suffix_from_source_return() {
                    // A specialized module owner may already have consumed
                    // the fact while completing its exact C continuation.
                    let _ = self.take_original_timing_main_loop_iteration_returned_to_wait();
                }
            } else if self.original_timing_main_loop_interruption().is_none() {
                // The source is still inside Module_MainRouting. Keep the NMI
                // latch and common sprite-preparation suffix pending; a later
                // CallStackContinued receipt owns their continuation.
                //
                // `rom_load_partial_nmi_this_frame` is only an atomic-runtime
                // marker saying that this host must not run the common suffix.
                // This return already enforces that ordering. Letting the
                // marker survive the suspended source call would make an
                // unrelated later iteration skip C's unconditional
                // `NMI_PrepareSprites(); nmi_boolean = 0` suffix.
                self.rom_load_partial_nmi_this_frame = false;
                if let Some(action) = idle_suffix_action {
                    self.apply_original_timing_idle_main_loop_suffix_action(action);
                } else {
                    assert!(
                        self.pending_main_loop_common_suffix
                            .replace(
                                MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                            )
                            .is_none(),
                        "ZeldaRunGameLoop common suffix was already pending",
                    );
                }
                return;
            } else if self.take_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::SpritePreparation,
            ) {
                // The unconditional `NMI_PrepareSprites(); nmi_boolean = 0`
                // suffix started and was interrupted mid-prep. It stays
                // pending for the following host; a module with its own
                // take (the dungeon dispatcher) consumed this receipt before
                // reaching here.
                self.rom_load_partial_nmi_this_frame = false;
                if let Some(action) = idle_suffix_action {
                    self.apply_original_timing_idle_main_loop_suffix_action(action);
                } else {
                    assert!(
                        self.pending_main_loop_common_suffix
                            .replace(
                                MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                            )
                            .is_none(),
                        "ZeldaRunGameLoop common suffix was already pending",
                    );
                }
                return;
            }
        }
        if !self.dialogue_scroll_cpu_is_idle() {
            // The long scroll copy has crossed vblank before the ROM reaches
            // Main_PrepSpritesForNmi or clears $12. Its continuation is resumed
            // by the dedicated scheduler in run_frame_internal.
            if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
                eprintln!(
                    "[HOSTPATH] host={} game-loop tail: dialogue scroll busy",
                    self.frame_ctr_dbg
                );
            }
            return;
        }
        if self.rom_startup_timing()
            && (self.game_execution_scheduler.work_is_pending()
                || matches!(
                    self.game_execution_scheduler.pre_main_caller_continuation(),
                    Some(PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass { .. })
                )
                || self.pre_main_caller_continuation_is(
                    PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
                )
                || self.pre_main_caller_continuation_is(
                    PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
                ))
        {
            return;
        }
        if let Some(
            interruption @ crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                next_group_start,
            },
        ) = self.original_timing_main_loop_interruption()
        {
            assert_eq!(
                self.take_forwarded_original_timing_main_loop_interruption(interruption),
                Some(OriginalTimingBoundary::NmiAccepted),
                "extended-OAM sprite preparation progress must come from an accepted NMI",
            );
            self.nmi_prepare_sprites_through_extended_oam_packing(next_group_start);
            let previous_suffix = self.pending_main_loop_common_suffix.replace(
                MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
                    next_group_start,
                },
            );
            assert!(
                matches!(
                    previous_suffix,
                    None | Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                ),
                "partial NMI_PrepareSprites cannot overlap an incompatible common suffix owner",
            );
            return;
        }
        // In the ROM this call is after Module_MainRouting. When vblank interrupts
        // the VWF loop, the main thread has not reached NMI_PrepareSprites yet;
        // the actual interrupt still runs separately below and observes $0710.
        let partial_nmi = std::mem::take(&mut self.rom_load_partial_nmi_this_frame);
        if self.dialogue_fast_forward_hold_active
            && idle_suffix_action
                == Some(OriginalTimingIdleMainLoopSuffixAction::CompleteInSequence)
        {
            // The native VWF budget predicted a fast-forward hold, but the
            // wire's validated iteration completed its shared suffix in this
            // host (route host 344410): the suffix, not the hold, is
            // authoritative for the latch and sprite preparation.
            self.dialogue_fast_forward_hold_active = false;
        }
        if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
            eprintln!(
                "[HOSTPATH] host={} game-loop tail: fast_forward_hold={} partial_nmi={}",
                self.frame_ctr_dbg, self.dialogue_fast_forward_hold_active, partial_nmi,
            );
        }
        if !self.dialogue_fast_forward_hold_active && !partial_nmi {
            self.nmi_prepare_sprites_for_main_loop();
            self.replay_trace_ram_watch("game-loop-after-prepare-sprites");
            // The ROM clears nmi_boolean only after Module_MainRouting and
            // NMI_PrepareSprites return. A vblank-interrupted VWF slice has
            // reached neither point, so its NMI must observe the still-set
            // latch and skip NMI_DoUpdates (including OAM DMA and joypads).
            self.clear_nmi_update_latch();
        }
        self.replay_trace_ram_watch("game-loop-exit");
    }

    pub(super) fn preflight_dialogue_text_dma_before_early_stage(
        &self,
        completion_owner: OriginalTimingNmiHandlerCompletionOwner,
        iteration_started: bool,
    ) {
        if !iteration_started
            // Module0E owns the BG3 text-DMA early-stage protocol; Module15's
            // Agahnim dialogue (`KillAghanim_Func7` calling RenderText) drives
            // the same text machine without it (route host 315663).
            || self.game_state.frame.main_module != 0x0e
            || !matches!(
                self.dialogue_scroll_phase(),
                DialogueScrollPhase::CopyingRemainingPixels { .. }
            )
        {
            return;
        }
        assert!(
            completion_owner.completed(),
            "a fresh iteration cannot follow dialogue completion without its source NMI handler",
        );
        if completion_owner == OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::Open),
                "an early dialogue completion requires its carried Open NMI",
            );
        }
        assert_eq!(
            self.original_timing_expected_nmi_update_gates.first(),
            Some(&NmiUpdateGate::Open),
            "an early dialogue completion disagrees with its installed Open-NMI authority",
        );
        assert!(
            !self.game_state.display.nmi_update_is_latched(),
            "an early dialogue completion cannot publish while the native NMI latch is held",
        );
        assert_eq!(
            self.game_state.display.pending_nmi_subroutine, 2,
            "an early dialogue completion requires the source BG3 text-DMA subroutine",
        );
        assert_eq!(
            self.game_state.display.core_update_disable_flag, 2,
            "an early dialogue completion requires the source BG3 text-DMA disable state",
        );
    }
}
