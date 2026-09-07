//! ZeldaState runtime tests — items.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn emu_runframe_callback_consumes_one_external_oracle_receipt() {
    let mut state = ZeldaState::new();
    state.set_rom(&exact_timing_test_rom(0x18));
    state.set_rom_startup_timing(true);
    state.zelda_setup_emu_callbacks(None, Some(test_run_frame_callback), None);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x00b0,
            Vec::new(),
        ))
        .unwrap();
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );

    state.zelda_run_frame_with_replay_input_override(0x00b0, None);

    assert_eq!(state.ram[0x43], 1, "emulator callback ran more than once");
    assert_eq!(state.ram[0x44], 0xa0);
    assert_eq!(state.rom_reset_frame_delay, 81);
    assert_eq!(state.original_timing_owner(), OriginalTimingOwner::Live);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(0));
    assert!(!state.original_timing_cold_start_eligible);
    assert!(!state.original_timing_host_dispatch_active);
}

#[test]
fn pinned_snes9x_receipts_are_typed_input_bound_and_consumed_once() {
    let mut disabled = ZeldaState::new();
    assert_eq!(
        disabled.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            Vec::new(),
        )),
        Err(OriginalTimingReceiptInstallError::TimingDisabled),
    );

    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.zelda_setup_emu_callbacks(
        None,
        Some(test_run_frame_callback_observes_snes9x_semantic_receipts),
        None,
    );
    let progress =
        sprite::DungeonResetSpritesCpuProgress::Load(sprite::DungeonLoadSpritesCpuProgress {
            normal_load_ordinal: 1,
            slot: 1,
            checkpoint: sprite::DungeonSpriteLoadCheckpoint::YHigh,
        });
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x00b0,
            vec![
                dungeon_reset_progress_receipt(progress, OriginalTimingBoundary::NmiAccepted),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .unwrap();
    assert_eq!(
        state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x00a0,
            Vec::new()
        )),
        Err(OriginalTimingReceiptInstallError::ReceiptAlreadyInstalled)
    );

    state.zelda_run_frame_with_replay_input_override(0x00b0, None);

    assert_eq!(state.ram[0x43], 1);
    assert_eq!(state.original_timing_owner(), OriginalTimingOwner::Live);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(0));
    assert!(!state.original_timing_host_dispatch_active);
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );

    let encoded = bincode::serialize(&state).unwrap();
    let restored: ZeldaState = bincode::deserialize(&encoded).unwrap();
    assert!(restored.original_timing_semantic_receipts.is_none());

    assert_eq!(
        state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x00b0,
            Vec::new(),
        )),
        Err(OriginalTimingReceiptInstallError::OutOfSequence {
            expected: 1,
            actual: 0,
        }),
        "identical input must not make a consumed host receipt replayable",
    );

    state.zelda_setup_emu_callbacks(None, Some(test_run_frame_callback), None);
    state.zelda_run_frame_with_replay_input_override(0x00b0, None);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::MissingAuthorityReceipt),
        "an authoritative oracle owner must never coast through a host call without a receipt",
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), None);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
}

#[test]
fn live_overworld_sprite_receipts_advance_the_suspended_native_c_caller() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishPreOverworldProperties {
            overworld_screen: 0,
            sprite_presence_published: false,
        },
        3,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::PresencePublished,
            ),
        ],
    ));

    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    state.apply_original_timing_overworld_sprite_reload_progress(progress);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishPreOverworldProperties {
            sprite_presence_published: true,
            ..
        })
    ));

    state.set_overworld_sprite_presence_marker(0x0198, 0xad);
    state.sprite_slot_view_mut(15).set_state(1);
    state.sprite_slot_view_mut(14).set_state(1);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::SpriteActivated {
                    block: 0x0198,
                    slot: 13,
                    sprite_type: 0xac,
                },
            ),
        ],
    ));

    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    state.apply_original_timing_overworld_sprite_reload_progress(progress);
    assert_eq!(state.sprite_slot_view(13).n_word(), 0x0198);
    assert_eq!(state.sprite_slot_view(13).sprite_type(), 0xac);
    assert_eq!(state.sprite_slot_view(13).state(), 8);
}

#[test]
fn live_overworld_sprite_return_receipt_owns_native_reload_completion() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(4);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldSpriteReloadTail {
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::BeforeNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(NmiPhase::BeforeNmi),
        },
        4,
    );

    // The native estimate advances in shadow but cannot publish completion
    // while the authoritative source call is still running.
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(state.game_state.frame.submodule, 4);

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::ReloadReturned,
            ),
        ],
    ));
    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    let returned = state.apply_original_timing_overworld_sprite_reload_progress(progress);
    let step = state
        .game_execution_scheduler
        .advance_work_one_nmi_slice_with_authoritative_completion(returned);
    assert!(matches!(
        step,
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishOverworldSpriteReloadTail { .. }
        ))
    ));

    // This is the source-ordered tail owned by that semantic receipt. It
    // reaches Overworld_StartScrollTransition once and publishes submodule 5.
    state.complete_module09_load_new_sprites_after_reload();
    assert_eq!(state.game_state.frame.submodule, 5);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn live_map_quadrant_receipt_prevents_early_native_submodule_publication() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(3);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldMapQuadrants {
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        },
        2,
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(state.game_state.frame.submodule, 3);

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished],
    ));
    let published = state.take_original_timing_overworld_map_quadrants_published();
    let step = state
        .game_execution_scheduler
        .advance_work_one_nmi_slice_with_authoritative_completion(published);
    assert!(matches!(
        step,
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishOverworldMapQuadrants {
                scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
            }
        ))
    ));

    // The production completion arm now has authority to invoke the existing
    // source-order `complete_module09_load_new_map_quadrants`; this unit keeps
    // the asset-heavy map decompressor out of the scheduler proof.
    assert_eq!(state.game_state.frame.submodule, 3);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn live_presentation_receipt_publishes_scanout_domains_after_translated_capture() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.ppu.vram[0x4000] = 0x1234;
    state.ppu.vram[0x4010] = 0x5678;
    state.ppu.cgram[33] = 0x1234;
    state.ppu.oam[0] = 0x1234;
    for (layer, bg) in state.ppu.bg_layer.iter_mut().enumerate() {
        bg.tilemap_adr = (layer * 0x400) as u16;
    }
    state.ppu.bg_layer[0].h_scroll = 0x0100;
    state.ppu.bg_layer[0].v_scroll = 0x0200;
    state.ppu.m7_matrix = [1, 2, 3, 4, 5, 6, 7, 8];
    state.ppu.vram[0x0750] = 0x19e1;
    state.ppu.vram[0x3c00] = 0x4444;
    state.ppu.vram[0x7c00] = 0x1111;
    write_le_u16(&mut state.ram, ANIMATED_TILE_VRAM_ADDR, 0x3c00);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);
    // Native NMI reaches the correct following name-table generation, while
    // the already-captured outgoing surface still holds the preceding word.
    state.ppu.vram[0x0750] = 0x19e2;
    state.ppu.vram[0x7c00] = 0x2222;
    state.ppu.bg_layer[0].h_scroll = 0x0102;

    let mut pixels = vec![0; crate::PresentedObjTiles::PIXELS_PER_TILE];
    pixels[0] = 0x0f;
    let presentation = crate::PresentedObjTiles::new(vec![0x4000], pixels).unwrap();
    let expected_presentation = presentation.clone();
    let mut animated_bg_pixels = vec![0; crate::PresentedAnimatedBgTiles::TILE_COUNT * 64];
    animated_bg_pixels[..64].fill(1);
    let presented_animated_bg = crate::PresentedAnimatedBgTiles::new(
        crate::PresentedAnimatedBgDestination::Overworld,
        animated_bg_pixels,
    )
    .unwrap();
    let mut colors = vec![0; crate::PresentedCgram::COLOR_COUNT];
    colors[33] = 0x0421;
    let presented_cgram = crate::PresentedCgram::new(colors).unwrap();
    let mut oam = vec![0; crate::PresentedOam::BYTE_COUNT];
    oam[0] = 0x78;
    oam[1] = 0x56;
    let presented_oam = crate::PresentedOam::new(oam).unwrap();
    let mut hud = vec![0; crate::PresentedHudTilemap::WORD_COUNT];
    hud[0x79] = 0x2508;
    let presented_hud = crate::PresentedHudTilemap::new(hud).unwrap();
    let mut dialogue_text = vec![0; crate::PresentedDialogueText::WORD_COUNT];
    dialogue_text[0] = 0x3333;
    let presented_dialogue_text = crate::PresentedDialogueText::new(dialogue_text).unwrap();
    let presented_bg_tilemaps = crate::PresentedBgTilemaps::new(
        (0..crate::PresentedBgTilemaps::LAYER_COUNT)
            .map(|layer| {
                let mut words = vec![0; crate::PresentedBgTilemapLayer::WORDS_PER_SCREEN];
                if layer == 1 {
                    words[0x350] = 0x19e2;
                }
                crate::PresentedBgTilemapLayer::new(
                    layer as u8,
                    (layer * 0x400) as u16,
                    false,
                    false,
                    words,
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    let presented_bg_scroll = crate::PresentedBgScroll::new(vec![
        [[0x00fe, 0x0200], [0, 0], [0, 0], [0, 0]];
        crate::PresentedBgScroll::VISIBLE_LINES
    ])
    .unwrap();
    let presented_mode7_transform = crate::PresentedMode7Transform::new(vec![
        [9, 10, 11, 12, 13, 14, 15, 16];
        crate::PresentedMode7Transform::VISIBLE_LINES
    ])
    .unwrap();
    let presented_window_mask = crate::PresentedWindowMask::new(
        vec![[[8, 248], [1, 0]]; crate::PresentedWindowMask::VISIBLE_LINES],
        [0x16, 0],
        [0, 0x03, 0x03, 0, 0x03, 0],
    )
    .unwrap();
    let presented_inidisp = crate::PresentedInidisp::new(1, 0, Some(48))
        .unwrap()
        .with_retained_prior_surface(true);
    let presented_scanout_geometry = crate::PresentedScanoutGeometry::new(7).unwrap();
    state.ppu.brightness = 1;
    state.ppu.forced_blank = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_host_dispatch_active = true;
    state.original_timing_semantic_receipts = Some(
        OriginalTimingHostReceipts::new(0, 0, Vec::new())
            .with_presented_animated_bg_tiles(presented_animated_bg.clone())
            .with_presented_cgram(presented_cgram)
            .with_presented_inidisp(presented_inidisp)
            .with_presented_scanout_geometry(presented_scanout_geometry)
            .with_presented_hud_tilemap(presented_hud)
            .with_presented_dialogue_text(presented_dialogue_text)
            .with_presented_bg_tilemaps(presented_bg_tilemaps)
            .with_presented_bg_scroll(presented_bg_scroll.clone())
            .with_presented_mode7_transform(presented_mode7_transform.clone())
            .with_presented_window_mask(presented_window_mask.clone())
            .with_presented_oam(presented_oam)
            .with_presented_obj_tiles(presentation),
    );

    state.finish_original_timing_host_dispatch(true);

    let display = state.display_snapshot.as_ref().unwrap();
    assert_eq!(display.ppu.cgram[33], 0x0421);
    assert_eq!(state.ppu.cgram[33], 0x1234, "live palette stays private");
    assert_eq!(display.ppu.oam[0], 0x1234);
    assert_eq!(
        display.presented_oam_override.as_deref().unwrap()[0],
        0x5678
    );
    assert_eq!(state.ppu.oam[0], 0x1234, "live OAM stays private");
    assert_eq!(
        display.presented_hud_tilemap_override.as_deref().unwrap()[0x79],
        0x2508
    );
    assert_ne!(
        state.ppu.vram[0x6040 + 0x79],
        0x2508,
        "live VRAM stays private",
    );
    assert_eq!(
        display
            .presented_dialogue_text_override
            .as_ref()
            .unwrap()
            .words()[0],
        0x3333,
    );
    assert_eq!(
        state.ppu.vram[0x7c00], 0x2222,
        "live dialogue character VRAM stays private",
    );
    assert_eq!(
        state.last_original_timing_dialogue_text_shadow_result(),
        None
    );
    assert_eq!(display.ppu.vram[0x0750], 0x19e2);
    assert_eq!(display.ppu.bg_layer[1].tilemap_adr, 0x0400);
    assert_eq!(display.ppu.bg_layer[0].h_scroll, 0x0100);
    assert_eq!(display.ppu.bg_layer[0].v_scroll, 0x0200);
    assert_eq!(
        display.presented_bg_scroll_override,
        Some(presented_bg_scroll.clone())
    );
    assert_eq!(
        display.presented_mode7_transform_override,
        Some(presented_mode7_transform)
    );
    assert_eq!(
        display.presented_window_mask_override,
        Some(presented_window_mask)
    );
    assert_eq!(state.ppu.bg_layer[0].h_scroll, 0x0102);
    assert_eq!(state.ppu.vram[0x0750], 0x19e2, "live VRAM stays private");
    assert_eq!(
        state.last_original_timing_bg_tilemap_shadow_result(),
        Some(crate::OriginalTimingBgTilemapShadowResult {
            mismatched_geometry_layers: 0,
            compared_words: 4 * 0x400,
            mismatched_words: 0,
            first_mismatch: None,
        }),
    );
    assert_eq!(
        state.last_original_timing_bg_scroll_shadow_result(),
        None,
        "the native scroll shadow runs when the renderer consumes scanlines",
    );
    assert_eq!(display.presented_inidisp_override, Some(presented_inidisp));
    assert_eq!(
        display.presented_scanout_geometry_override,
        Some(presented_scanout_geometry)
    );
    assert_eq!(
        display.presented_animated_bg_tiles_override,
        Some(presented_animated_bg)
    );
    assert_eq!(state.ppu.brightness, 1, "live INIDISP stays private");
    assert!(state.ppu.forced_blank, "live INIDISP stays private");
    assert_eq!(
        display.presented_obj_tiles_override,
        Some(expected_presentation)
    );
    assert!(display.explicit_obj_cache_vram.is_none());
    assert!(state.original_timing_semantic_receipts.is_none());

    let (
        rendered_color,
        rendered_oam,
        rendered_hud,
        rendered_brightness,
        rendered_blank,
        rendered_blank_prefix,
        rendered_blank_suffix,
        rendered_retain_prior_surface,
        rendered_top_crop,
        rendered_animated_bg_word,
        rendered_dialogue_text_word,
        rendered_obj_word,
        rendered_bg1_scroll,
        rendered_mode7_transform,
        rendered_window1,
        rendered_windowsel,
    ) = state.with_display_snapshot(|display| {
        let scanlines = display.ppu_scanline_windows();
        (
            display.ppu.cgram[33],
            display.ppu.oam[0],
            display.ppu.vram[0x6040 + 0x79],
            display.ppu.brightness,
            display.ppu.forced_blank,
            display.ppu.forced_blank_scanlines,
            display.ppu.forced_blank_from_scanline,
            display.ppu.retain_active_display_history,
            display.ppu.scanout_top_crop,
            display
                .ppu
                .bg_vram_latch
                .as_deref()
                .unwrap_or(&display.ppu.vram)[0x3c00],
            display
                .ppu
                .bg_vram_latch
                .as_deref()
                .unwrap_or(&display.ppu.vram)[0x7c00],
            display
                .ppu
                .obj_vram_latch
                .as_deref()
                .unwrap_or(&display.ppu.vram)[0x4000],
            [scanlines[0].5[0], scanlines[0].6[0]],
            scanlines[0].7,
            [scanlines[0].0, scanlines[0].1],
            display.ppu.windowsel,
        )
    });
    assert_eq!(
        rendered_color, 0x0421,
        "native generation composition must not replace an authoritative completed-scanout palette",
    );
    assert_eq!(
        rendered_oam, 0x5678,
        "native OAM composition must not replace an authoritative completed-scanout OAM image",
    );
    assert_eq!(
        rendered_hud, 0x2508,
        "native VRAM composition must not replace an authoritative completed HUD DMA",
    );
    assert_eq!(rendered_brightness, 1);
    assert!(!rendered_blank);
    assert_eq!(rendered_blank_prefix, 0);
    assert_eq!(rendered_blank_suffix, Some(48));
    assert!(rendered_retain_prior_surface);
    assert_eq!(rendered_top_crop, 7);
    assert_eq!(
        rendered_animated_bg_word, 0x00ff,
        "the decoded semantic receipt must replace the stale 0x4444 animated generation",
    );
    assert_eq!(rendered_dialogue_text_word, 0x3333);
    assert_eq!(
        rendered_obj_word, 0x8080,
        "native OAM composition must not replace an authoritative decoded OBJ cache",
    );
    assert_eq!(rendered_bg1_scroll, [0x00fe, 0x0200]);
    assert_eq!(rendered_mode7_transform, [9, 10, 11, 12, 13, 14, 15, 16]);
    assert_eq!(rendered_window1, [8, 248]);
    assert_eq!(rendered_windowsel, 0x0003_0330);
    assert_eq!(
        state.last_original_timing_bg_scroll_shadow_result(),
        Some(crate::OriginalTimingBgScrollShadowResult {
            compared_scanline_layers: crate::PresentedBgScroll::VISIBLE_LINES
                * crate::PresentedBgScroll::LAYER_COUNT,
            mismatched_scanline_layers: crate::PresentedBgScroll::VISIBLE_LINES,
            first_mismatch: Some((0, 0)),
        }),
    );
    assert_eq!(
        state.last_original_timing_mode7_transform_shadow_result(),
        Some(crate::OriginalTimingMode7TransformShadowResult {
            compared_scanline_fields: crate::PresentedMode7Transform::VISIBLE_LINES
                * crate::PresentedMode7Transform::FIELD_COUNT,
            mismatched_scanline_fields: crate::PresentedMode7Transform::VISIBLE_LINES
                * crate::PresentedMode7Transform::FIELD_COUNT,
            first_mismatch: Some((0, 0)),
        }),
    );
    assert_eq!(
        state.last_original_timing_window_mask_shadow_result(),
        Some(crate::OriginalTimingWindowMaskShadowResult {
            compared_scanline_windows: crate::PresentedWindowMask::VISIBLE_LINES
                * crate::PresentedWindowMask::WINDOW_COUNT,
            mismatched_scanline_windows: crate::PresentedWindowMask::VISIBLE_LINES,
            mismatched_screen_masks: 1,
            first_mismatch: Some((0, 0)),
        }),
    );
    assert_eq!(
        state.last_original_timing_dialogue_text_shadow_result(),
        Some(crate::OriginalTimingDialogueTextShadowResult {
            compared_words: crate::PresentedDialogueText::WORD_COUNT,
            mismatched_words: 1,
            first_mismatch: Some(0),
        }),
    );
    assert_eq!(
        [
            state.ppu.bg_layer[0].h_scroll,
            state.ppu.bg_layer[0].v_scroll,
        ],
        [0x0102, 0x0200],
        "the authority receipt must not mutate live BG scroll state",
    );
    assert_eq!(
        state.ppu.m7_matrix,
        [1, 2, 3, 4, 5, 6, 7, 8],
        "the authority receipt must not mutate live Mode 7 state",
    );
}

#[test]
fn live_main_loop_preparation_receipt_suspends_the_completed_dungeon_caller() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(5);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::SpritePreparation,
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 6);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
            caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
        }),
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    state.game_execution_scheduler.begin_host_frame();
    let continuation = match state.game_execution_scheduler.advance_work_one_nmi_slice() {
        Some(GameWorkStep::Complete(continuation)) => continuation,
        step => panic!("sprite-preparation return must resume after its accepted NMI: {step:?}"),
    };
    // This focused semantic test has no Zelda ROM. The publication decision is
    // independent of the optional instruction-timing continuation.
    state.rom_startup_timing = false;
    state.complete_post_trailing_nmi_continuation(continuation, 0, false, false);
    assert!(matches!(
        state.next_display_obj_memory_generation,
        Some(DisplayObjGeneration::RetainCapturedMemory { .. }),
    ));
    assert!(state.next_display_obj_cache_vram.is_some());
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }),
    );
}

#[test]
fn live_link_oam_receipt_suspends_the_completed_dungeon_caller() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(4);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 5);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn),
    );
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_big_key_publication_enters_the_existing_source_call_continuation() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    state.sprite_slot_view_mut(2).set_state(6);
    state.sprite_slot_view_mut(2).set_sprite_type(0x6a);
    state.sprite_slot_view_mut(2).set_delay_main(0);
    state.sprite_slot_view_mut(2).set_die_action(2);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        20_201,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::BigKeyDropGraphicsStarted(2),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.sprite_slot_view(2).sprite_type(), 0xe5);
    assert_eq!(state.sprite_slot_view(2).state(), 6);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishBigKeyDropGraphics {
            sprite_slot: 2,
            dungeon: DungeonSpriteMainReturn {
                link_oam: None,
                bg2_x: 0,
                bg2_y: 0,
                bg1_x: 0,
                bg1_y: 0,
            },
        }),
    );
    assert!(state.active_dungeon_sprite_main_return.is_none());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn big_key_partial_slot_receipt_is_a_sprite_main_module_phase() {
    assert_eq!(
        module_cpu_phase_from_main_loop_interruption(
            crate::MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(2),
        ),
        Some(ModuleCpuPhase::InterruptedInSpriteMain),
    );
    assert_eq!(
        sprite_main_cpu_boundary_from_interruption(
            crate::MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(2),
        ),
        Some(SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(2)),
    );
    assert!(crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(12).is_sprite_main());
    assert_eq!(
        module_cpu_phase_from_main_loop_interruption(
            crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(12),
        ),
        Some(ModuleCpuPhase::InterruptedInSpriteMain),
    );
}

#[test]
fn live_link_oam_receipt_suspends_landing_before_shared_sprite_preparation() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(15);
    state.set_subsubmodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 1);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn),
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1),
        "an oracle host return must survive this host's trailing NMI",
    );
    assert!(state.active_dungeon_sprite_main_return.is_none());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    // The following source host accepts the NMI inside the suspended LinkOam
    // call, returns through the saved Module 7 suffix, and accepts the next NMI
    // at the host boundary. The second NMI has not published yet.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    state.original_timing_expected_nmi_ppu_register_operands = vec![None, None];
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_module09_caller_phase_comes_from_the_typed_interruption_receipt() {
    for (receipt, phase) in [
        (
            crate::MainLoopInterruption::LinkOam,
            ModuleCpuPhase::InterruptedInLinkOam,
        ),
        (
            crate::MainLoopInterruption::SpritePreparation,
            ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        ),
    ] {
        let mut state = ZeldaState::new();
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            14_680,
            0x20,
            vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(receipt)],
        ));

        assert_eq!(
            state.take_authoritative_module09_caller_phase(Some(phase)),
            phase,
        );
        assert!(state
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| receipts.semantic().is_empty()));
    }

    let mut unclaimed = ZeldaState::new();
    unclaimed.original_timing_owner = OriginalTimingOwnerState::Live;
    unclaimed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        17_790,
        0x10,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        )],
    ));
    assert_eq!(
        unclaimed
            .take_authoritative_module09_caller_phase(Some(ModuleCpuPhase::InterruptedInLinkOam,)),
        ModuleCpuPhase::InterruptedInLinkOam,
        "an unclaimed phase remains owned by the native timing backend",
    );
    assert_eq!(
        unclaimed
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        &[OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )],
    );
}

#[test]
fn pinned_snes9x_receipt_input_mismatch_fails_closed_without_exposing_semantics() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.zelda_setup_emu_callbacks(None, Some(test_run_frame_callback), None);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x0001,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open,
            )],
        ))
        .unwrap();

    state.zelda_run_frame_with_replay_input_override(0x0002, None);

    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(
            OriginalTimingUnavailableReason::AuthorityReceiptInputMismatch,
        ),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), None);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn item_receipt_places_chest_item_with_c_offsets() {
    let mut state = ZeldaState::new();
    state.ram[ITEM_RECEIPT_METHOD] = 1;
    state
        .dungeon_room_load_mut()
        .set_loading_bg_offsets(0x1200, 0x3400);

    set_link_test_byte(&mut state, LINK_RECEIVEITEM_INDEX, 0);
    state.ancilla_add_item_receipt(0x22, 4, 0x0182);

    assert_eq!(state.ram[ANCILLA_X_LO + 4], 0x0c);
    assert_eq!(state.ram[ANCILLA_X_HI + 4], 0x12);
    assert_eq!(state.ram[ANCILLA_Y_LO + 4], 0x13);
    assert_eq!(state.ram[ANCILLA_Y_HI + 4], 0x34);
}

#[test]
fn receive_item_enters_hold_item_state_for_normal_receipts() {
    let mut state = ZeldaState::new();
    state.ram[ITEM_RECEIPT_METHOD] = 0;
    state.follower_link_state_mut().set_auxiliary_state(1);
    state.follower_link_state_mut().set_incapacitated_timer(7);
    state.ram[COUNTDOWN_FOR_BLINK] = 8;
    state.follower_link_state_mut().set_state_bits(0xff);
    state.follower_link_state_mut().set_button_mask_b_y(0xff);
    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0xff);
    state.follower_link_state_mut().set_button_b_frames(0xff);
    state.follower_link_state_mut().set_speed_setting(0xff);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);
    state.follower_link_state_mut().set_item_in_hand(0xff);
    state.follower_link_state_mut().set_position_mode(0xff);
    state.ram[PLAYER_HANDLER_TIMER] = 0xff;
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 0);

    state.link_receive_item(0x20, 0);

    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 0);
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        0
    );
    assert_eq!(state.ram[COUNTDOWN_FOR_BLINK], 0);
    assert_eq!(link_test_byte(&state, LINK_RECEIVEITEM_INDEX), 0x20);
    assert_eq!(link_test_byte(&state, LINK_ITEM_HOLDING_TIMER), 0x60);
    assert_eq!(state.game_state.player.follower_link.state_bits(), 0);
    assert_eq!(state.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0
    );
    assert_eq!(state.game_state.player.follower_link.button_b_frames(), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
    assert_eq!(state.game_state.player.follower_link.item_in_hand(), 0);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 21);
    assert_eq!(link_test_byte(&state, LINK_POSE_FOR_ITEM), 2);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
}

#[test]
fn bottled_item_receipt_fills_first_open_bottle() {
    let mut state = ZeldaState::new();
    state.inventory_items_mut().set_bottle(0, 2);
    state.inventory_items_mut().set_bottle(1, 4);

    state.item_receipt_give_bottled_item(0x2f);

    assert_eq!(link_test_byte(&state, LINK_ITEM_BOTTLE_INFO), 4);
    assert_eq!(state.ram[LINK_ITEM_BOTTLE_INFO + 1], 4);
}

#[test]
fn direct_host_calls_without_oracle_receipts_never_seed_a_cpu_emulator() {
    let mut state = ZeldaState::new();
    state.set_rom(&exact_timing_test_rom(0x4c));
    state.set_rom_startup_timing(true);

    state.run_frame_internal(0x1234, 0);

    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::MissingAuthorityReceipt),
    );
    assert!(!state.original_timing_host_dispatch_active);

    state.run_frame_internal(0, 0);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::MissingAuthorityReceipt),
        "missing authority is terminal and never falls back to opcode execution",
    );
}

#[test]
fn every_sprite_disable_receipt_prefix_resumes_to_the_atomic_c_endpoint() {
    let phases = [
        DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot: 15 },
        DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot: 7 },
        DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot: 0 },
        DungeonSpriteDisableCpuProgress::AncillasThrough { slot: 9 },
        DungeonSpriteDisableCpuProgress::AncillasThrough { slot: 4 },
        DungeonSpriteDisableCpuProgress::AncillasThrough { slot: 0 },
        DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared,
        DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
    ];
    for phase in phases {
        let mut state = ZeldaState::new();
        state.set_dungeon_room_index(0x51);
        state.set_indoor_flag(1);
        for slot in 0..16 {
            state.sprite_slot_view_mut(slot).set_state(9);
            state
                .sprite_slot_view_mut(slot)
                .set_sprite_type(0x60 + slot as u8);
        }
        state.ram[ANCILLA_TYPE..ANCILLA_TYPE + 10].fill(3);
        state.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 7;
        state.ram[SPRITE_LIMIT_INSTANCE] = 9;
        state.sync_native_game_state_from_ram();
        let mut atomic = state.clone();
        atomic.dungeon_reset_sprites();

        let progress = DungeonResetSpritesCpuProgress::Disable(phase);
        state.dungeon_reset_sprites_through_cpu_progress(progress);
        state.dungeon_resume_reset_sprites_after_cpu_progress(progress);

        assert_eq!(state.ram, atomic.ram, "RAM mismatch after {phase:?}");
        assert_eq!(
            state.game_state.sprites, atomic.game_state.sprites,
            "native sprite mismatch after {phase:?}",
        );
    }
}

#[test]
fn snes9x_semantic_receipt_survives_the_typed_sprite_reset_continuation() {
    let progress = sprite::DungeonResetSpritesCpuProgress::Cache {
        slot: 15,
        field: CachedSpriteCacheField::StateClear,
    };
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    for slot in 0..16 {
        let mut live = state.sprite_slot_view_mut(slot);
        live.set_state(9);
        live.set_sprite_type(0x20 + slot as u8);
        live.set_x_low(0x30 + slot as u8);
        live.set_y_low(0x50 + slot as u8);
    }
    for field in CachedSpriteCacheField::C_SOURCE_ORDER {
        for slot in 0..16 {
            state.ram[field.alt_address() + slot] = 0xa5;
        }
    }
    state.sync_native_game_state_from_ram();
    let mut atomic = state.clone();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![dungeon_reset_progress_receipt(
            progress,
            OriginalTimingBoundary::NmiAccepted,
        )],
    ));

    let observed = state
        .take_original_timing_dungeon_reset_sprites_progress()
        .expect("typed oracle receipt must identify the interrupted C statement");
    state.dungeon_reset_sprites_through_cpu_progress(observed.progress);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::RoomLoadSpriteReset {
                progress: observed.progress,
            },
        },
        1,
    );
    let Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSupertileTransition {
        work: DungeonSupertileTransitionWork::RoomLoadSpriteReset { progress: resumed },
    })) = state.game_execution_scheduler.advance_work_one_nmi_slice()
    else {
        panic!("scheduler lost the typed Dungeon_ResetSprites continuation");
    };
    assert_eq!(resumed, progress);
    state.dungeon_resume_reset_sprites_after_cpu_progress(resumed);
    atomic.dungeon_reset_sprites();

    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    assert!(state
        .take_original_timing_dungeon_reset_sprites_progress()
        .is_none());
}

#[test]
fn live_module07_sprite_receipt_stops_before_the_unreturned_slot() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    for slot in 0..=1 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    let interruption = crate::MainLoopInterruption::SpriteMainAfterSlot(1);
    let mut receipts = OriginalTimingHostReceipts::new(
        9_015,
        0,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        )],
    );
    receipts
        .forward_main_loop_interruption(interruption, OriginalTimingBoundary::NmiAccepted)
        .unwrap();
    state.original_timing_semantic_receipts = Some(receipts);
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.complete_module07_dungeon_after_submodule();

    assert_eq!(state.sprite_slot_view(1).state(), 9);
    assert_eq!(state.sprite_slot_view(0).state(), 8);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(1),
            caller: SpriteMainCpuCaller::DungeonModule07Live {
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        }),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic()
            == [OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]));
}

#[test]
fn purple_chest_follower_graphics_retains_the_deleted_sprite_prefix() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(0x80, 0x80);
    {
        let mut sprite = state.sprite_slot_view_mut(8);
        sprite.set_state(9);
        sprite.set_sprite_type(0xb4);
        sprite.set_ai_state(1);
        sprite.set_x(0x80);
        sprite.set_y(0x80);
    }
    let mut atomic = state.clone();
    atomic.sprite_b4_purple_chest(8);
    assert!(state.sprite_b4_purple_chest_before_follower_graphics(8));
    assert_eq!(state.sprite_slot_view(8).state(), 0);
    assert_eq!(state.game_state.sprites.follower_runtime.indicator(), 12);
    state.load_follower_graphics();
    state.sprite_become_follower(8);
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn big_key_drop_publishes_entry_dma_then_holds_it_across_waiting_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.last_presented_oam = Some(vec![0x1111; 272]);
    state.staged_presented_oam = Some(vec![0x2222; 272]);
    state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 1,
        bg2_y: 2,
        bg1_x: 3,
        bg1_y: 4,
    });

    assert!(state.begin_big_key_drop_graphics_work(2));
    assert_eq!(state.next_display_obj_memory_generation, None);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );

    state.stage_big_key_drop_waiting_obj_scanout();
    assert_eq!(
        state.next_display_obj_memory_generation,
        Some(DisplayObjGeneration::RetainCapturedOam {
            oam: vec![0x2222; 272],
        })
    );
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishBigKeyDropGraphics {
            sprite_slot: 2,
            dungeon: DungeonSpriteMainReturn {
                link_oam: None,
                bg2_x: 1,
                bg2_y: 2,
                bg1_x: 3,
                bg1_y: 4,
            },
        })
    );
}

#[test]
fn c_big_key_decompression_retains_leading_nmi_scroll_until_its_nmi_returns() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.ppu.bg_layer[0].h_scroll = 0x00a5;
    state.ppu.bg_layer[1].h_scroll = 0x00a5;
    state.ppu.bg_layer[0].v_scroll = 0x1010;
    state.ppu.bg_layer[1].v_scroll = 0x1010;
    state.ppu.mode = 1;
    state.ppu.vram[0x1234] = 0x5678;
    state.ppu.cgram[7] = 0x1357;
    state.capture_display_snapshot();

    // The leading NMI installs A4 after the retiring field was captured.
    state.ppu.bg_layer[0].h_scroll = 0x00a4;
    state.ppu.bg_layer[1].h_scroll = 0x00a4;
    let active_scanout_scroll = BgScrollRegisterScanout::capture(&state.ppu);
    state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 1,
        bg2_y: 2,
        bg1_x: 3,
        bg1_y: 4,
    });
    assert!(state.begin_big_key_drop_graphics_work(2));
    assert_eq!(
        state.next_display_bg_scroll_generation,
        DisplayBgScrollGeneration::RetainCpuSliceEntry(active_scanout_scroll),
    );
    state.capture_display_snapshot();
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .bg_scroll_generation,
        DisplayBgScrollGeneration::RetainCpuSliceEntry(active_scanout_scroll),
    );

    // The resumed Module07 camera authors $00a2 after the synchronous call has
    // begun but before Rust captures the atomic main result.
    state.ppu.bg_layer[0].h_scroll = 0x00a2;
    state.ppu.bg_layer[1].h_scroll = 0x00a2;
    state.ppu.mode = 3;
    let completed_registers = NmiPpuRegisterScanout::capture(&state.ppu);
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .effective_presented_dma = Some(EffectivePresentedDma::ppu_registers_only(
        completed_registers,
    ));
    let completion_host_entry = BgScrollRegisterScanout::capture(&state.ppu);
    let publication = state
        .game_execution_scheduler
        .current_work()
        .unwrap()
        .completion_publication(completion_host_entry);
    assert_eq!(
        publication.bg_scroll, None,
        "the call-origin A4 generation is one-shot and must not be republished at completion",
    );

    // C Module07_Dungeon runs Dungeon_HandleCamera before Sprite_Main. On the
    // observed ROM boundary the leading NMI has installed $00a4, then the
    // resumed camera authors $00a2 before SpritePrep_BigKey_load_graphics is
    // interrupted in Decompression_GetNextByte. That NMI has not yet reached
    // WritePpuRegisters when the field retires, so $00a2 is next-field state.
    let receipt = state
        .display_snapshot
        .as_deref()
        .unwrap()
        .effective_presented_dma
        .as_ref()
        .unwrap();
    assert!(receipt.completed_ppu_registers.is_some());
    assert!(receipt.vram_writes.is_empty());
    assert!(receipt.completed_oam.is_none());

    let displayed = state.with_display_snapshot(|display| {
        (
            display.ppu.bg_layer[0].h_scroll,
            display.ppu.bg_layer[1].h_scroll,
            display.ppu.vram[0x1234],
            display.ppu.cgram[7],
            display.ppu.mode,
        )
    });
    assert_eq!(displayed, (0x00a4, 0x00a4, 0x5678, 0x1357, 3));
    assert_eq!(state.ppu.bg_layer[0].h_scroll, 0x00a2);
    assert_eq!(state.ppu.bg_layer[1].h_scroll, 0x00a2);

    state.capture_display_snapshot();
    let following = state.with_display_snapshot(|display| {
        (
            display.ppu.bg_layer[0].h_scroll,
            display.ppu.bg_layer[1].h_scroll,
        )
    });
    assert_eq!(following, (0x00a2, 0x00a2));
}

#[test]
fn pre_dungeon_return_consumes_same_host_main_iteration_receipt() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();

    assert_eq!(
        state.finish_pre_dungeon_caller_at_main_wait(),
        Some(crate::MainLoopProgress::IterationStarted)
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
    assert!(!state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
}

#[test]
fn enemy_drop_receipt_sound_retires_at_the_measured_graphics_return() {
    let continuation = GameWorkContinuation::FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x22,
            ground_apress_tail: None,
        },
    };
    let mut state = ZeldaState::new();
    state.set_sound_effect_2(0x2f);
    state
        .game_execution_scheduler
        .schedule_work(continuation, ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES);

    state.publish_or_defer_item_receipt_sound_effect_2(0x0f);

    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x2f);
    assert_eq!(
        state.enemy_drop_item_graphics_deferred_sound_effect_2,
        Some(0x0f)
    );
    state.game_execution_scheduler.finish_work();
    state.retire_enemy_drop_item_graphics_sound_effect_2();

    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x0f);
    assert_eq!(state.enemy_drop_item_graphics_deferred_sound_effect_2, None);
}

#[test]
fn ordinary_item_receipt_sound_publishes_without_enemy_drop_work() {
    let mut state = ZeldaState::new();
    state.set_sound_effect_2(0x2f);

    state.publish_or_defer_item_receipt_sound_effect_2(0x0f);

    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x0f);
    assert_eq!(state.enemy_drop_item_graphics_deferred_sound_effect_2, None);
}

#[test]
fn standard_item_receipt_graphics_hold_the_four_snes9x_observed_nmi_slices() {
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x14),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x06),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x0c),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x24),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    // The separately packed $5d sheet crosses the same four boundaries: the
    // heart-container receipt (gfx $3e) held the live wire across them at
    // route host 102905.
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x23),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x3e),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    // The fourth slice completes gfx $14's OBJ upload and the caller prepares
    // player OAM before scanout; keep this receipt aligned with the observed
    // live/live boundary instead of an older retained-CHR assumption.
    assert_eq!(
        atomic_item_graphics_return_obj_scanout(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x14,
                ground_apress_tail: None
            },
        ),
        ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLivePlayerOamAfterMain,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        }
    );
    assert_eq!(
        atomic_item_graphics_return_obj_scanout(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x22,
                ground_apress_tail: None
            },
        ),
        ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        }
    );

    let continuation = GameWorkContinuation::FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x14,
            ground_apress_tail: None,
        },
    };
    let mut work =
        ScheduledGameWork::schedule(continuation, ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES);
    assert!(!work.suspends_translated_call_stack());
    assert_eq!(work.in_flight_display_snapshot_publication_override(), None);
    let mut waiting_publication = work;
    assert_eq!(
        waiting_publication.advance_one_nmi_slice(),
        GameWorkStep::Waiting
    );
    assert_eq!(
        waiting_publication.in_flight_display_snapshot_publication_override(),
        Some(DisplaySnapshotPublication::RetainPublished)
    );
    let suspended = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                receipt: ItemReceiptReturn {
                    ancilla_slot: 4,
                    item: 0,
                    chest_position: 0,
                },
                sprite_slot: 0,
                dungeon: DungeonSpriteMainReturn {
                    link_oam: None,
                    bg2_x: 1,
                    bg2_y: 2,
                    bg1_x: 3,
                    bg1_y: 4,
                },
            },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );
    assert!(suspended.suspends_translated_call_stack());
    assert_eq!(
        suspended.in_flight_display_snapshot_publication_override(),
        None
    );
    let chest_receipt = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x10,
                ground_apress_tail: Some(ItemReceiptReturn {
                    ancilla_slot: 4,
                    item: 0x24,
                    chest_position: 0x0182,
                }),
            },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );
    assert!(
        chest_receipt.suspends_translated_call_stack(),
        "the synchronous chest decompressor must retain Module 7's pre-Sprite_Main stack",
    );
    for _ in 0..ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(continuation)
    );
}

#[test]
fn run4586_terminal_ground_item_receipt_completes_one_handler_sprite_return_and_suffix() {
    const ANIMATED_DESTINATION: usize = 0x3b00;

    let mut state = live_terminal_ground_item_receipt_state();
    let continuation = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishItemReceiptGraphics { continuation }) => continuation,
        work => panic!("run4586 fixture installed the wrong work: {work:?}"),
    };
    let snapshot_identity = state
        .display_snapshot
        .as_ref()
        .map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.ppu.vram[ANIMATED_DESTINATION],
            )
        })
        .unwrap();
    let epoch_before = state.display_snapshot_epoch;
    let frame_counter_before = state.game_state.frame.frame_counter;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.game_state.frame.frame_counter, frame_counter_before);
    let receipt_ancilla = state.ancilla_slot_view(4);
    assert_eq!(receipt_ancilla.item_to_link(), 0x12);
    assert_eq!(receipt_ancilla.work_byte_1(), 0);
    assert_eq!(receipt_ancilla.work_byte_3(), 9);
    assert_eq!(receipt_ancilla.work_byte_4(), 5);
    assert_eq!(receipt_ancilla.step(), 0);
    assert_eq!(receipt_ancilla.aux_timer(), 0x60);
    assert_eq!(state.display_snapshot_epoch, epoch_before + 1);
    assert_eq!(
        state.display_snapshot.as_ref().map(|snapshot| (
            snapshot.publication_epoch,
            snapshot.ppu.vram[ANIMATED_DESTINATION]
        )),
        Some(snapshot_identity),
        "RetainPublished must preserve the exact entry scanout generation",
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| !snapshot.accepts_nmi_dma_receipts));
    assert_eq!(
        state.ppu.vram[ANIMATED_DESTINATION], 0x5a5a,
        "the one specialized Held handler must execute animated-BG DMA",
    );
    // A second item-handler NMI would require and consume another gate; the
    // drained one-cell queue therefore also proves audio/NMI execution occurred
    // exactly once rather than falling through the legacy completion arm.
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(atomic_item_graphics_return_obj_scanout(continuation)),
    );
}

#[test]
fn terminal_ground_item_receipt_owner_is_not_tied_to_route_gfx14() {
    let mut state = live_terminal_ground_item_receipt_state_with_gfx(0x06);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None
    );
    assert_eq!(state.ancilla_slot_view(4).item_to_link(), 0);
}

#[test]
fn terminal_ground_item_receipt_gfx22_keeps_enemy_drop_sound_ownership_disjoint() {
    let mut state = live_terminal_ground_item_receipt_state_with_gfx(0x22);
    let continuation = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishItemReceiptGraphics { continuation }) => continuation,
        work => panic!("gfx22 ground fixture installed the wrong work: {work:?}"),
    };

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.enemy_drop_item_graphics_deferred_sound_effect_2, None);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(atomic_item_graphics_return_obj_scanout(continuation)),
    );
}

#[test]
fn terminal_ground_item_receipt_preflight_is_failure_atomic() {
    fn assert_rejected(label: &str, mut state: ZeldaState) {
        state.original_timing_host_dispatch_active = true;
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let scheduler_before = state.game_execution_scheduler;
        let suffix_before = state.pending_main_loop_common_suffix;
        let ram_before = state.ram.clone();
        let ppu_before = (
            state.ppu.vram.clone(),
            state.ppu.oam.clone(),
            state.ppu.cgram.clone(),
            state.ppu.screen_enabled,
            state.ppu.math_enabled,
            state.ppu.prevent_math_mode,
        );
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )
        });
        let epoch_before = state.display_snapshot_epoch;
        let pending_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        );
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let sidecars_before = (
            state.main_loop_sprite_preparation_completed,
            state.original_timing_sprite_main_return_claims_remaining,
            state.item_receipt_completion_live_link_dma_host,
            state.next_display_obj_scanout_generation,
            state.oam_law_pending.clone(),
            state.enemy_drop_item_graphics_deferred_sound_effect_2,
        );
        let audio_before = (
            state.game_state.system_signals.ambient_sound_effect(),
            state.game_state.system_signals.last_ambient_sound_effect(),
            state.zelda_debug_apu_write_ports(),
            state.audio_nmi_processed_before_main,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err(), "{label} unexpectedly reached execution");
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(
            (
                state.ppu.vram.clone(),
                state.ppu.oam.clone(),
                state.ppu.cgram.clone(),
                state.ppu.screen_enabled,
                state.ppu.math_enabled,
                state.ppu.prevent_math_mode,
            ),
            ppu_before,
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
            ),
            pending_before,
        );
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert_eq!(
            (
                state.main_loop_sprite_preparation_completed,
                state.original_timing_sprite_main_return_claims_remaining,
                state.item_receipt_completion_live_link_dma_host,
                state.next_display_obj_scanout_generation,
                state.oam_law_pending.clone(),
                state.enemy_drop_item_graphics_deferred_sound_effect_2,
            ),
            sidecars_before,
            "{label} mutated a terminal item sidecar before preflight rejection",
        );
        assert_eq!(
            (
                state.game_state.system_signals.ambient_sound_effect(),
                state.game_state.system_signals.last_ambient_sound_effect(),
                state.zelda_debug_apu_write_ports(),
                state.audio_nmi_processed_before_main,
            ),
            audio_before,
        );
    }

    let mut wrong_gate = live_terminal_ground_item_receipt_state();
    wrong_gate.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected("wrong gate queue", wrong_gate);

    let mut wrong_native_latch = live_terminal_ground_item_receipt_state();
    wrong_native_latch.clear_nmi_update_latch();
    assert_rejected("wrong native latch", wrong_native_latch);

    let mut reordered_return = live_terminal_ground_item_receipt_state();
    reordered_return
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .swap(1, 2);
    assert_rejected("reordered Sprite_Main return", reordered_return);

    let mut missing_return = live_terminal_ground_item_receipt_state();
    missing_return
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .remove(2);
    assert_rejected("missing Sprite_Main return", missing_return);

    let mut specialized_suffix = live_terminal_ground_item_receipt_state();
    specialized_suffix.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    assert_rejected("specialized suffix", specialized_suffix);

    // A completed caller with no ground A-press tail only loads graphics in
    // later NMIs; its hosts are ordinary iterations (route host 514805), so
    // that continuation is no longer rejected here.

    // A gfx-$21 ground-item terminal outside handler 21 is no longer a
    // rejected shape: route host 14076 proved its ordinary module epilogue
    // on the wire, so the terminal plan now accepts it with the plain
    // capture-and-interrupt publication instead of failing closed.

    let mut stale_enemy_drop_sound = live_terminal_ground_item_receipt_state();
    stale_enemy_drop_sound.enemy_drop_item_graphics_deferred_sound_effect_2 = Some(0x0f);
    assert_rejected("stale enemy-drop sound owner", stale_enemy_drop_sound);
}

#[test]
fn bottle_vendor_suffix_waits_for_the_live_source_call_return_receipt() {
    let mut state = ZeldaState::new();
    let slot = 12usize;
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.active_module09_sprite_main_return = Some(Module09ItemReceiptCallerReturn {
        link_oam: None,
        scroll: Module09SpriteMainReturn {
            bg2_x: 0x1111,
            bg2_y: 0x2222,
            bg1_x: 0x3333,
            bg1_y: 0x4444,
        },
        rain_already_published: false,
        after_sprite_main: Module09AfterSpriteMain::Ordinary,
    });
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: slot as u8 },
                progress: SourceCallProgress::Suspended,
            },
        )],
    ));
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state.clear_modal_pause_flag();
    state.set_submodule(0);
    state.follower_link_state_mut().clear_auxiliary_state();
    state.follower_link_state_mut().clear_item_hold_pose();
    state.follower_link_state_mut().clear_state_bits();
    state.follower_link_state_mut().set_x(0x1000);
    state.follower_link_state_mut().set_y(0x1000);
    for ancilla in 0..5 {
        state.ancilla_slot_view_mut(ancilla).clear();
    }
    {
        let mut sprite = state.sprite_slot_view_mut(slot);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(2);
    }
    state.player_resources_mut().set_rupees_goal(150);

    state.sprite_bottle_vendor(slot);

    assert_eq!(state.sprite_slot_view(slot).ai_state(), 2);
    assert_eq!(
        state
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 2,
        0
    );
    assert_eq!(
        state.game_state.inventory.player_resources.rupees_goal(),
        150
    );
    let (continuation, module09) = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishItemReceiptGraphics {
            continuation:
                ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                    receipt,
                    sprite_slot,
                    suffix,
                    caller,
                },
        }) => {
            assert_eq!(sprite_slot, slot as u8);
            assert_eq!(suffix, SpriteMainItemReceiptSuffix::BottleVendor);
            let SpriteMainItemReceiptCallerReturn::Module09(module09) = caller else {
                panic!("BottleVendor retained the wrong module caller: {caller:?}");
            };
            assert_eq!(
                module09,
                Module09ItemReceiptCallerReturn {
                    link_oam: None,
                    scroll: Module09SpriteMainReturn {
                        bg2_x: 0x1111,
                        bg2_y: 0x2222,
                        bg1_x: 0x3333,
                        bg1_y: 0x4444,
                    },
                    rain_already_published: false,
                    after_sprite_main: Module09AfterSpriteMain::Ordinary,
                }
            );
            (receipt, module09)
        }
        work => panic!("BottleVendor did not retain its source caller: {work:?}"),
    };

    state.complete_ancilla_add_item_receipt(continuation);
    state.complete_link_receive_item(continuation.item);
    state.complete_bottle_vendor_item_receipt(slot);
    state.set_main_module(9);
    state.set_submodule(0);
    state.complete_module09_overworld_after_resumed_sprite_main(module09);

    assert_eq!(state.sprite_slot_view(slot).ai_state(), 0);
    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(
        state
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 2,
        2
    );
    assert_eq!(
        state.game_state.inventory.player_resources.rupees_goal(),
        50
    );
}

#[test]
fn direct_sprite_item_pickup_consumes_the_returned_source_graphics_receipt_once() {
    let mut state = ZeldaState::new();
    let slot = 3u8;
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.sprite_system_mut().set_cur_object_index(slot);
    state.follower_link_state_mut().set_item_receipt_method(0);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMainDirect { slot },
                progress: SourceCallProgress::Returned,
            },
        )],
    ));

    let status = state.begin_item_receipt_graphics_work(
        0x22,
        ItemReceiptReturn {
            ancilla_slot: 2,
            item: 0x32,
            chest_position: 0,
        },
        ItemReceiptCaller::AtomicCaller,
    );

    assert_eq!(status, GameCallStatus::Returned);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .take_original_timing_item_receipt_graphics_progress()
        .is_none());
}

#[test]
fn sick_kid_suffix_waits_for_the_live_source_call_return_receipt() {
    let mut state = ZeldaState::new();
    let slot = 0usize;
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0x1111,
        bg2_y: 0x2222,
        bg1_x: 0x3333,
        bg1_y: 0x4444,
    });
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: slot as u8 },
                progress: SourceCallProgress::Suspended,
            },
        )],
    ));
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state.set_submodule(0);
    state.follower_link_state_mut().set_immobilized_flag(1);
    for ancilla in 0..5 {
        state.ancilla_slot_view_mut(ancilla).clear();
    }
    {
        let mut sprite = state.sprite_slot_view_mut(slot);
        sprite.set_state(9);
        sprite.set_sprite_type(0x1f);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(2);
        sprite.set_graphics(2);
    }

    state.sprite_1_f_sick_kid(slot);

    assert_eq!(state.sprite_slot_view(slot).ai_state(), 2);
    assert_eq!(state.game_state.player.follower_link.immobilized_flag(), 1);
    let receipt = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishItemReceiptGraphics {
            continuation:
                ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                    receipt,
                    sprite_slot,
                    suffix,
                    caller,
                },
        }) => {
            assert_eq!(sprite_slot, slot as u8);
            assert_eq!(suffix, SpriteMainItemReceiptSuffix::SickKid);
            assert_eq!(
                caller,
                SpriteMainItemReceiptCallerReturn::Module07(DungeonSpriteMainReturn {
                    link_oam: None,
                    bg2_x: 0x1111,
                    bg2_y: 0x2222,
                    bg1_x: 0x3333,
                    bg1_y: 0x4444,
                })
            );
            receipt
        }
        work => panic!("SickKid did not retain its source caller: {work:?}"),
    };

    state.complete_ancilla_add_item_receipt(receipt);
    state.complete_link_receive_item(receipt.item);
    state.complete_sick_kid_item_receipt(slot);

    assert_eq!(state.sprite_slot_view(slot).ai_state(), 3);
    assert_eq!(state.game_state.player.follower_link.immobilized_flag(), 0);
}

#[test]
fn live_dungeon_exit_projection_receipt_suspends_the_entry_suffix() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1704);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(126);
    state.set_spotlight_window_state(0);
    for index in 0..224 {
        write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE + index * 2, 0x5a5a);
    }
    let iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication);

    assert!(state.begin_dungeon_exit_spotlight_entry(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 119,
            checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 157 },
        }),
        iteration,
    ));

    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 126);
    assert_eq!(
        read_le_u16(&state.ram, RESERVED_HDMA_TABLE + 156 * 2),
        state.spotlight_hdma_table_dynamic_entry(156),
    );
    assert_eq!(
        read_le_u16(&state.ram, RESERVED_HDMA_TABLE + 157 * 2),
        0x5a5a,
    );
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry {
        table_build,
        iteration: pending,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("projection receipt did not retain the entry continuation");
    };
    assert_eq!(pending, iteration);

    state.game_execution_scheduler.finish_work();
    state.complete_dungeon_exit_spotlight_entry(table_build, iteration);

    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 119);
    for index in 0..224 {
        assert_eq!(
            read_le_u16(&state.ram, RESERVED_HDMA_TABLE + index * 2),
            state.spotlight_hdma_table_dynamic_entry(index),
        );
    }
}

#[test]
fn live_recurring_close_projection_receipt_suspends_link_movement() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(119);
    state.set_spotlight_window_state(0);
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);

    assert!(state.begin_dungeon_exit_spotlight_build(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 120,
            checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 143 },
        }),
        iteration,
    ));

    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 119);
    assert_eq!(state.game_state.player.follower_link.y(), 1703);
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        iteration: pending,
        projection_completed: false,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("projection receipt did not retain the recurring close continuation");
    };
    assert!(pending.prepares_main_loop_sprites_before_second_nmi());

    state.game_execution_scheduler.finish_work();
    state.complete_dungeon_exit_spotlight_build(table_build, false, pending, false, false);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 112);
}

#[test]
fn live_module10_lower_write_receipt_defers_the_goal_transition_to_the_next_host() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0x10);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1880);
    state.follower_link_state_mut().set_y(1044);
    state.set_bg2_h_copy2(1758);
    state.set_bg2_v_copy2(1024);
    state.set_spotlight_window_radius(119);
    state.set_spotlight_window_state(2);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            crate::SpotlightTableBuildProgressReceipt {
                progress: crate::SpotlightTableBuildProgress {
                    completed_iterations: 185,
                    checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                        lower_cursor: 39,
                        circle_value: 0xff00,
                    },
                },
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();

    state.Module10_SpotlightOpen();

    assert_eq!(state.game_state.frame.main_module, 0x10);
    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 119,);
    let step = state
        .game_execution_scheduler
        .advance_work_one_nmi_slice()
        .expect("the interrupted table build owns one resumable host slice");
    let GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpotlightBuild {
        table_build,
        phase,
        projection_completed,
        iteration,
    }) = step
    else {
        panic!("unexpected spotlight continuation: {step:?}");
    };
    assert_eq!(phase, OverworldSpotlightBuildPhase::Recurring);
    assert!(!projection_completed);
    assert!(!state.complete_overworld_spotlight_build(
        table_build,
        phase,
        projection_completed,
        iteration,
        None,
    ));

    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 126,);
}

#[test]
fn dungeon_exit_crossing_publishes_the_completed_oam_dma_receipt() {
    let mut state = ZeldaState::new();
    state.ppu.oam[0] = 0x1111;
    state.capture_display_snapshot();
    let mut following = *state.display_snapshot.take().unwrap();
    following.ppu.oam[0] = 0x2222;
    following.completed_oam_dma_after_capture = Some(vec![0x3333; state.ppu.oam.len()]);
    following.closed_oam_boundary_receipt = Some(ClosedOamBoundaryReceipt {
        publication_host_frame: following.publication_host_frame,
        active_oam: vec![0x4444; state.ppu.oam.len()],
    });
    let plan = DisplayPublicationPlan::resolve(
        &following,
        DisplayPublicationSignals {
            dungeon_exit_crosses_nmi_boundary: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposeCompletedWorkAfterNmi
    );
    state.compose_display_oam(&following, &plan);
    assert_eq!(state.ppu.oam[0], 0x3333);

    following.oam_scanout_source = OamScanoutSource::ComposeCompletedWorkAfterNmi;
    state.oam_law_visible = Some(vec![0x5555; state.ppu.oam.len()]);
    state.display_snapshot = Some(Box::new(following));
    let presented = state.with_display_snapshot(|display| display.ppu.oam[0]);
    assert_eq!(presented, 0x3333);

    let snapshot = state.display_snapshot.as_mut().unwrap();
    snapshot.effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: Vec::new(),
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: Some(vec![0x6666; state.ppu.oam.len()]),
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });
    let presented = state.with_display_snapshot(|display| display.ppu.oam[0]);
    assert_eq!(presented, 0x6666);

    let return_scanout = dungeon_exit_spotlight_entry_return_obj_scanout();
    assert_eq!(
        return_scanout.oam,
        OamScanoutSource::ComposeCompletedWorkAfterNmi
    );
    assert_eq!(
        return_scanout.link_obj,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        return_scanout.link_obj_sources,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
}

#[test]
fn animated_bg_operand_generation_is_explicit_not_receipt_owned() {
    const CAPTURED_SOURCE: usize = 0xaa80;
    const LIVE_SOURCE: usize = 0xae80;
    const DESTINATION: usize = 0x3b00;

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_animated_tile_data_source_address(LIVE_SOURCE as u16);
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].fill(0x11);
    state.ram[LIVE_SOURCE..LIVE_SOURCE + 0x400].fill(0x22);
    state.capture_display_snapshot();

    let entry_frame = state.game_state.frame;
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: Some(PreMainAnimatedTileDma {
            source_address: CAPTURED_SOURCE,
            destination_address: DESTINATION,
            data: state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].to_vec(),
        }),
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });

    let mut leading_nmi_plan = rom_graphics_dma_plan(7, 0x0e);
    leading_nmi_plan.animated_bg_operands = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    state.nmi_core_animated_bg_update(leading_nmi_plan);

    assert_eq!(state.ppu.vram[DESTINATION], 0x1111);
    assert_eq!(
        state.game_state.display.animated_tile_data_source_usize(),
        LIVE_SOURCE
    );

    state.pre_main_graphics_dma.as_mut().unwrap().animated_tile = Some(PreMainAnimatedTileDma {
        source_address: CAPTURED_SOURCE,
        destination_address: DESTINATION,
        data: state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].to_vec(),
    });
    state.ppu.vram[DESTINATION..DESTINATION + 0x200].fill(0);
    state.begin_effective_presented_dma();
    state.nmi_core_animated_bg_update(rom_graphics_dma_plan(7, 0x0e));

    assert_eq!(state.ppu.vram[DESTINATION], 0x2222);
    assert!(state
        .pre_main_graphics_dma
        .as_ref()
        .unwrap()
        .animated_tile
        .is_some());
}

#[test]
fn overworld_sprite_activation_receipt_executes_the_native_c_leaf() {
    let mut state = ZeldaState::new();
    state.garnish_state_mut().set_sprcoll_x_base(0x0600);
    state.garnish_state_mut().set_sprcoll_y_base(0x0400);
    state.set_overworld_sprite_presence_marker(0x0198, 0xad);
    state.sprite_slot_view_mut(15).set_state(1);
    state.sprite_slot_view_mut(14).set_state(1);

    state.overworld_load_proxima_sprite_if_alive(0x0198);

    let sprite = state.sprite_slot_view(13);
    assert_eq!(sprite.n_word(), 0x0198);
    assert_eq!(sprite.sprite_type(), 0xac);
    assert_eq!(sprite.state(), 8);
    assert_eq!(sprite.x(), 0x0780);
    assert_eq!(sprite.y(), 0x0490);
}

#[test]
fn pre_overworld_live_stage_receipt_is_consumed_once_without_oracle_provenance() {
    let stage = crate::PreOverworldStageCompletion::OverlaysReturned;
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::PreOverworldStageCompleted(
            stage,
        )],
    ));

    assert!(state.take_original_timing_pre_overworld_stage_completion(stage));
    assert!(!state.take_original_timing_pre_overworld_stage_completion(stage));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn fresh_sprite_main_same_slot_item_receipt_supersedes_its_outer_prefix() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let item_progress = ItemReceiptGraphicsProgressReceipt {
        caller: ItemReceiptGraphicsCaller::SpriteMainDirect { slot: 13 },
        progress: SourceCallProgress::Suspended,
    };
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        177046,
        0,
        vec![
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                crate::SpriteMainProgress::AfterTimersAndOam(13),
            ),
            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(item_progress),
        ],
    ));

    assert_eq!(
        state.take_original_timing_sprite_main_boundary_for_fresh_caller(),
        Some((
            SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(13),
            OriginalTimingBoundary::HostReturn,
        )),
    );
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        &[OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            item_progress,
        )],
        "the item call itself remains the sole consumer of its suspension receipt",
    );
}

#[test]
fn live_authority_rejects_synthetic_cached_progress_without_a_typed_receipt() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let advance = DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 5,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(0)),
        cached_sprite_interruption: Some(CachedSpriteCpuInterruption::Restoring {
            slot: 7,
            live_fields: 4,
        }),
    };

    assert!(state.arm_dungeon_sprite_main_cpu_continuation(advance));
    assert_eq!(state.dungeon_cached_sprite_cpu_interruption_pending, None);
    assert_eq!(
        state.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterSlot(0)),
    );
}

#[test]
fn live_cached_sprite_receipt_supersedes_conflicting_shadow_progress_once() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![cached_sprite_progress_receipt(
            crate::CachedSpriteExecutionProgress::Restoring {
                slot: 7,
                live_fields: 4,
            },
            OriginalTimingBoundary::NmiAccepted,
        )],
    ));
    let advance = DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 5,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(0)),
        cached_sprite_interruption: Some(CachedSpriteCpuInterruption::Loading {
            slot: 2,
            copied_fields: 12,
        }),
    };

    assert!(state.arm_dungeon_sprite_main_cpu_continuation(advance));
    assert_eq!(state.sprite_main_cpu_boundary, None);
    assert_eq!(
        state.dungeon_cached_sprite_cpu_interruption_pending,
        Some(CachedSpriteCpuInterruption::Restoring {
            slot: 7,
            live_fields: 4,
        }),
    );
    assert_eq!(
        state.take_original_timing_cached_sprite_execution_progress(),
        None,
        "the semantic receipt must transfer exactly once into the continuation",
    );
}

#[test]
fn live_cached_sprite_receipt_overrides_the_synthetic_quadrant_phase() {
    for shadow_phase in [
        ModuleCpuPhase::CompleteBeforeNmi,
        ModuleCpuPhase::InterruptedAfterSpriteMain,
    ] {
        let mut state = ZeldaState::new();
        state.set_subsubmodule(5);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![cached_sprite_progress_receipt(
                crate::CachedSpriteExecutionProgress::Restoring {
                    slot: 2,
                    live_fields: 20,
                },
                OriginalTimingBoundary::NmiAccepted,
            )],
        ));
        let advance = DungeonModuleCpuAdvance {
            phase: shadow_phase,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 5,
            palette_countdown: 0,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        };

        state.apply_dungeon_quadrant_cpu_advance(advance);

        assert!(state.dungeon_quadrant_cpu_continuation_active);
        assert!(!state.dungeon_post_sprite_main_return_pending);
        assert_eq!(
            state.dungeon_cached_sprite_cpu_interruption_pending,
            Some(CachedSpriteCpuInterruption::Restoring {
                slot: 2,
                live_fields: 20,
            }),
        );
        assert_eq!(
            state.dungeon_cached_sprite_cpu_interruption_boundary,
            Some(OriginalTimingBoundary::NmiAccepted),
        );
        assert_eq!(
            state.take_original_timing_cached_sprite_execution_progress(),
            None,
            "the Live receipt must transfer exactly once for {shadow_phase:?}",
        );
    }
}

#[test]
fn cached_sprite_restore_receipt_resumes_to_the_atomic_c_endpoint() {
    let mut base = ZeldaState::new();
    base.set_indoor_flag(1);
    base.set_submodule(2);
    {
        let mut slot = base.sprite_slot_view_mut(2);
        slot.set_state(8);
        slot.set_sprite_type(0x6d);
        slot.set_x(0x04ab);
        slot.set_y(0x0543);
        slot.set_direction(1);
        slot.set_graphics(2);
    }
    base.dungeon_cache_trans_sprites();
    {
        let mut slot = base.sprite_slot_view_mut(2);
        slot.set_sprite_type(0x6f);
        slot.set_x(0x0380);
        slot.set_y(0x0460);
        slot.set_direction(0);
        slot.set_graphics(0);
    }

    let mut atomic = base.clone();
    atomic.execute_cached_sprites();

    let boundary = CachedSpriteCpuInterruption::Restoring {
        slot: 2,
        live_fields: 20,
    };
    let mut resumed = base;
    resumed.dungeon_cached_sprite_cpu_interruption_pending = Some(boundary);
    resumed.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });
    resumed.execute_cached_sprites();
    let (scheduled_boundary, live_slot_backup) = match resumed
        .game_execution_scheduler
        .current_work()
        .expect("the semantic boundary must suspend cached-sprite execution")
    {
        GameWorkContinuation::FinishDungeonCachedSpriteMain {
            boundary,
            live_slot_backup,
            ..
        } => (boundary, live_slot_backup),
        work => panic!("unexpected cached-sprite continuation: {work:?}"),
    };
    assert_eq!(scheduled_boundary, boundary);

    resumed
        .complete_cached_sprite_main_after_interrupted_slot(scheduled_boundary, &live_slot_backup);

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn cached_sprite_receipt_boundary_selects_the_source_resume_side_of_nmi() {
    let make_state = |boundary| {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        state.set_submodule(2);
        {
            let mut slot = state.sprite_slot_view_mut(2);
            slot.set_state(8);
            slot.set_sprite_type(0x6d);
            slot.set_x(0x04ab);
            slot.set_y(0x0543);
        }
        state.dungeon_cache_trans_sprites();
        state.sprite_slot_view_mut(2).set_sprite_type(0x6f);
        state.dungeon_cached_sprite_cpu_interruption_pending =
            Some(CachedSpriteCpuInterruption::Restoring {
                slot: 2,
                live_fields: 9,
            });
        state.dungeon_cached_sprite_cpu_interruption_boundary = Some(boundary);
        state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
            link_oam: None,
            bg2_x: 0,
            bg2_y: 0,
            bg1_x: 0,
            bg1_y: 0,
        });
        state
    };

    let mut accepted = make_state(OriginalTimingBoundary::NmiAccepted);
    accepted.execute_cached_sprites();
    assert_eq!(
        accepted
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        None,
        "an already-accepted NMI must not be counted again",
    );
    accepted.game_execution_scheduler.begin_host_frame();
    assert!(matches!(
        accepted
            .game_execution_scheduler
            .take_after_current_trailing_nmi(),
        Some(GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }),
    ));

    let mut returned = make_state(OriginalTimingBoundary::HostReturn);
    returned.execute_cached_sprites();
    assert_eq!(
        returned
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1),
        "a SCAN_KEYS return before NMI must retain the pending crossing",
    );
}

#[test]
fn overworld_graphics_timing_uses_measured_work_receipts() {
    assert_eq!(
        overworld_aux_graphics_timing(OverworldAuxGraphicsWorkload {
            background_packs_to_decompress: 0,
        }),
        OverworldAuxGraphicsTiming {
            load_nmi_slices: 11,
        }
    );
    assert_eq!(
        overworld_aux_graphics_timing(OverworldAuxGraphicsWorkload {
            background_packs_to_decompress: 2,
        }),
        OverworldAuxGraphicsTiming {
            load_nmi_slices: 15,
        }
    );

    let light_map_timing = overworld_map_and_sprite_graphics_timing(OverworldMapGraphicsWorkload {
        map32_definition_changes: 670,
    });
    assert_eq!(
        light_map_timing,
        OverworldMapAndSpriteGraphicsTiming {
            quadrant_load_nmi_slices: 13,
            map16_to_map8_tail_nmi_slices: 3,
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        }
    );
    assert_eq!(
        overworld_map_and_sprite_graphics_timing(OverworldMapGraphicsWorkload {
            map32_definition_changes: 796,
        }),
        OverworldMapAndSpriteGraphicsTiming {
            quadrant_load_nmi_slices: 14,
            map16_to_map8_tail_nmi_slices: 3,
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        }
    );

    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishOverworldMapQuadrants {
            scroll_map_and_sprite_gfx_tail_nmi_slices: light_map_timing
                .scroll_map_and_sprite_gfx_tail_nmi_slices,
        },
        light_map_timing.quadrant_load_nmi_slices,
    );
    for _ in 1..light_map_timing.quadrant_load_nmi_slices {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishOverworldMapQuadrants {
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        })
    );
}

#[test]
fn later_source_sprite_main_receipt_supersedes_palette_return_timing_shadow() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_subsubmodule(2);
    state.set_countdown(0);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::SpriteMainAfterSlot(1),
        )],
    ));
    state.dungeon_palette_cpu_advance_pending = Some(DungeonPaletteCpuAdvance {
        work: CpuWorkAdvance::ReachedBoundary {
            boundary: CpuRasterBoundary::CpuNmiAcceptance,
            remaining_work_master_cycles: 0,
        },
        pc: 0x0d_fcd1,
        subsubmodule: 2,
        palette_countdown: 0,
    });

    state.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(
        state.original_timing_main_loop_interruption(),
        Some(crate::MainLoopInterruption::SpriteMainAfterSlot(1)),
    );
}

#[test]
fn live_iteration_receipt_supersedes_poly_only_legacy_dispatch_guess() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(6);
    state.set_subsubmodule(42);
    state.set_frame_counter(105);
    state.activate_nmi_thread();
    state.reset_bg_tile_animation_countdown(5);
    for i in 0..32 {
        state.oam_state_mut().set_packed_extended_oam_byte(i, 0x5a);
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            846,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                        next_group_start: 4,
                    },
                ),
            ],
        ))
        .unwrap();

    // The old cooperative-thread selector chooses only the poly worker on
    // this host. The continuous source receipt nevertheless proves that the
    // 65816 also entered ZeldaRunGameLoop before the host returned, so that
    // semantic fact owns the main prefix and suspended caller lifecycle.
    state.run_frame_internal(0, crate::RUN_POLY);

    assert_eq!(state.game_state.frame.frame_counter, 106);
    assert_eq!(state.game_state.frame.subsubmodule, 41);
    assert!(!state.game_state.display.nmi_thread_active);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(
            MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
                next_group_start: 4,
            },
        ),
    );
    let packed_extended_oam = crate::game_state::constants::EXTENDED_OAM;
    let expected_packed_extended_oam = (0..32)
        .map(|i| state.game_state.oam.packed_extended_oam_byte(i))
        .collect::<Vec<_>>();
    assert_eq!(
        state.ram[packed_extended_oam..packed_extended_oam + 8],
        [0x5a; 8]
    );
    assert_eq!(
        state.ram[packed_extended_oam + 8..packed_extended_oam + 32],
        expected_packed_extended_oam[8..],
    );
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 5);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(846));
    assert!(state.original_timing_semantic_receipts.is_none());

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            847,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_POLY);

    assert_eq!(state.game_state.frame.frame_counter, 106);
    assert_eq!(state.game_state.frame.subsubmodule, 41);
    assert_eq!(
        state.ram[packed_extended_oam..packed_extended_oam + 32],
        expected_packed_extended_oam,
    );
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 4);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(847));
    assert!(state.original_timing_semantic_receipts.is_none());
}
