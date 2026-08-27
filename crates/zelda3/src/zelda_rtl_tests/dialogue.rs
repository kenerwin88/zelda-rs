use super::*;
use crate::dialogue_ir::{DialogueIrKind, TEXT_COMMAND_START_US};

#[test]
fn bg3_vwf_glyph_runs_track_unaligned_glyphs_and_scroll() {
    let mut state = ZeldaState::new();
    state
        .messaging_text_mut()
        .load_decoded_dialogue(&[0, 1, 2, 0x78, 3]);
    for i in 0..126 {
        state.set_vwf_tile_word_at_byte_offset(i * 2, 0x3980 + i as u16);
    }

    state.record_bg3_vwf_glyph_run(1, 7, 0, 8, 1);
    assert_eq!(
        state.bg3_vwf_glyph_runs(),
        &[Bg3VwfGlyphRun {
            glyph_code: 1,
            origin_tile_number: 0x180,
            x: 7,
            y: 0,
            width: 8,
        }]
    );
    assert_eq!(state.bg3_vwf_glyph_run_dialogue_offsets(), &[1]);
    assert_eq!(
        state.bg3_vwf_glyph_run_dialogue_ir(0).map(|op| op.kind),
        Some(DialogueIrKind::Glyph { code: 1 })
    );
    state.scroll_bg3_vwf_glyph_runs_up_one_pixel();
    assert_eq!(state.bg3_vwf_glyph_runs()[0].y, -1);
    assert_eq!(state.bg3_vwf_glyph_run_dialogue_offsets(), &[1]);

    state.clear_bg3_vwf_glyph_runs();
    assert!(state.bg3_vwf_glyph_runs().is_empty());
    assert!(state.bg3_vwf_glyph_run_dialogue_offsets().is_empty());
}

#[test]
fn dialogue_snapshot_exposes_only_nmi_published_vwf_metadata() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_vwf_tile_word_at_byte_offset(0, 0x0180);
    state.record_bg3_vwf_glyph_run(1, 0, 0, 6, 0);
    state.publish_bg3_vwf_glyph_runs();
    state.record_bg3_vwf_glyph_run(2, 6, 0, 6, 1);
    state.dialogue_fast_forward_hold_active = true;

    state.capture_display_snapshot();
    let before_dma = state.with_display_snapshot(|display| {
        display
            .published_bg3_vwf_glyph_runs()
            .iter()
            .map(|run| run.glyph_code)
            .collect::<Vec<_>>()
    });
    assert_eq!(before_dma, vec![1]);

    state.nmi_upload_bg3_text();
    state.capture_display_snapshot();
    let after_dma = state.with_display_snapshot(|display| {
        display
            .published_bg3_vwf_glyph_runs()
            .iter()
            .map(|run| run.glyph_code)
            .collect::<Vec<_>>()
    });
    assert_eq!(after_dma, vec![1, 2]);
}

#[test]
fn dialogue_scroll_publishes_nmi_work_only_after_its_final_copy_slice() {
    let mut state = ZeldaState::new();
    state.game_execution_scheduler.begin_host_frame();
    state.set_main_module(14);
    state.set_submodule(2);
    state
        .messaging_text_mut()
        .load_decoded_dialogue(&[TEXT_COMMAND_START_US + 12]);
    state.messaging_state_mut().set_dialogue_scroll_speed(4);

    state.RenderText_Draw_MessageCharacters();
    assert!(state.dialogue_scroll_is_copying_remaining_pixels());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0);

    state.zelda_run_game_loop();
    assert!(state.dialogue_scroll_is_return_only());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 2);
    assert_eq!(state.game_state.display.core_update_disable_flag, 2);
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        0
    );
}

#[test]
fn dialogue_return_only_boundary_publishes_nmi_scroll_scanout() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(0xa680);
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    state.audio_nmi_processed_before_main = true;

    let current_scanout = [
        [0x0111, 0x01db],
        [0x0222, 0x0233],
        [0x0344, 0x0355],
        [0x0066, 0x0077],
    ];
    for (layer, [h_scroll, v_scroll]) in state.ppu.bg_layer.iter_mut().zip(current_scanout) {
        layer.h_scroll = h_scroll;
        layer.v_scroll = v_scroll;
    }

    state.set_bg1_h_copy(0x0211);
    state.set_bg1_v_copy(0x02db);
    state.set_bg2_h_copy(0x0322);
    state.set_bg2_v_copy(0x0333);
    state.set_bg3_h_copy2(0x0044);
    state.set_bg3_v_copy2(0x0055);

    state.run_frame_internal(0, crate::RUN_MAIN);

    let next_scanout = state
        .ppu
        .bg_layer
        .map(|layer| [layer.h_scroll, layer.v_scroll]);
    assert_eq!(
        next_scanout,
        [
            [0x0211, 0x02db],
            [0x0322, 0x0333],
            [0x0044, 0x0055],
            current_scanout[3],
        ]
    );

    let displayed = state.with_display_snapshot(|display| {
        display
            .ppu
            .bg_layer
            .map(|layer| [layer.h_scroll, layer.v_scroll])
    });
    assert_eq!(displayed, next_scanout);
}

#[test]
fn dialogue_return_only_receipt_continues_into_the_source_next_iteration() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(0xa680);
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    state.audio_nmi_processed_before_main = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
    ));
    // The assertion targets the shared ZeldaRunGameLoop boundary rather than
    // another dialogue command. Keep Module0E active but choose its C no-op
    // text-render dispatch state so the unit test needs no external ROM asset.
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(5);
    let frame_counter = state.game_state.frame.frame_counter;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_state.frame.frame_counter,
        frame_counter.wrapping_add(1),
        "the source-started C iteration must not be dropped behind ReturnOnly",
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn dialogue_vwf_return_suffix_releases_the_preprocessed_nmi_generation() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(0xa680);
    state.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    state.dialogue_fast_forward_hold_active = true;
    state.audio_nmi_processed_before_main = true;
    state.set_sound_effect_2(12);
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    state.set_bg_tile_animation_countdown(5);
    state.set_messaging_render_buffer_word(0, 0x1234);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert!(!state.dialogue_fast_forward_hold_active);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 4);
    assert!(!state.audio_nmi_processed_before_main);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 12);
    assert_eq!(state.ppu.vram[0x7c00], 0x1234);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0);

    state.interrupt_nmi_audio_parts();
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);
    assert_eq!(state.zelda_debug_apu_write_ports()[3], 12);
}

#[test]
fn dialogue_vwf_return_receipt_continues_into_the_source_next_iteration() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(0xa680);
    state.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    state.dialogue_fast_forward_hold_active = true;
    state.audio_nmi_processed_before_main = true;
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        48_105,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
    ));
    // ZeldaRunGameLoop increments the frame counter before Module_MainRouting.
    // Keep Module0E's dispatch on a source-valid no-op state so the test proves
    // the caller/NMI/main-loop order without requiring an external dialogue ROM.
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(5);
    state.set_frame_counter(0xe0);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0xe1);
    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn snes9x_dialogue_scroll_holds_the_coupled_register_generation() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(0xa680);
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    state.audio_nmi_processed_before_main = true;

    state.ppu.math_enabled = 0x32;
    state.ppu.half_color = false;
    state.set_color_math_control(0x72);
    state.ppu.bg_layer[0].h_scroll = 0x00f0;
    state.ppu.bg_layer[0].v_scroll = 0x00c4;
    state.set_bg1_h_copy(0x01f0);
    state.set_bg1_v_copy(0x01c4);
    state.set_bg1_h_copy2(0x01f0);
    state.set_bg1_v_copy2(0x01c4);

    state.run_frame_internal(0, crate::RUN_MAIN);

    // Original-ROM/Snes9x cold receipt at host frame 5377 (module $0e/$02,
    // input $0020) retains the preceding color-math and scroll registers while
    // the dialogue call stack crosses NMI. Publishing the mirrors here changes
    // 2,221 exact pixels. The atomic C frontend cannot express this suspended
    // call timing, so this is a hardware-timing contract rather than C logic.
    assert!(!state.ppu.half_color);
    assert_eq!(
        [
            state.ppu.bg_layer[0].h_scroll,
            state.ppu.bg_layer[0].v_scroll
        ],
        [0x00f0, 0x00c4]
    );
    let displayed = state.with_display_snapshot(|display| {
        (
            display.ppu.half_color,
            [
                display.ppu.bg_layer[0].h_scroll,
                display.ppu.bg_layer[0].v_scroll,
            ],
        )
    });
    assert_eq!(displayed, (false, [0x00f0, 0x00c4]));
}
