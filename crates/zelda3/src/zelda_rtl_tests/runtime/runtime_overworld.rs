//! ZeldaState runtime tests — overworld.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;
use crate::tile_definition::NativeTile;

#[test]
fn selected_game_entrance_scroll_suspends_before_display_mirrors() {
    let make_state = || {
        let mut state = ZeldaState::new();
        let mut data = Vec::new();
        let mut ranges = vec![(0, 0); 56];
        for index in 28..=55 {
            let mut bytes = vec![0; 128];
            if index == 28 {
                write_le_u16(&mut bytes, 0, 0x104);
            }
            if index == 31 {
                write_le_u16(&mut bytes, 0, 0x2110);
            }
            put_test_asset(&mut data, &mut ranges, index, bytes);
        }
        state.assets = Some(AssetPack::from_data_ranges(data, ranges));
        state.ram[RESTART_CHECK_FLAG] = 1;
        state.sync_native_game_state_from_ram();
        state.set_bg2_v_copy(0x1234);
        state
    };
    let mut atomic = make_state();
    atomic.module_pre_dungeon_initial_entrance();
    let mut staged = make_state();
    staged.begin_selected_game_entrance_scroll_prefix();
    assert_eq!(
        staged.game_state.dungeon.room_tracking.room_index2_word(),
        0x104
    );
    assert_eq!(
        staged.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        0x2110
    );
    assert_eq!(
        staged.game_state.display.ppu_scroll_copy.bg2_v_copy(),
        0x1234
    );
    assert!(staged.pending_selected_game_entrance.is_some());
    staged.module_pre_dungeon_initial_entrance();
    assert!(staged.pending_selected_game_entrance.is_none());
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn selected_game_entrance_before_selection_preserves_link_until_the_source_return() {
    let mut staged = ZeldaState::new();
    staged.assets = Some(probe_entrance_asset_pack(0, 0x59));
    staged.set_dungeon_room(0x10);
    staged
        .follower_link_state_mut()
        .set_position(0x1353, 0x058b);
    let mut atomic = staged.clone();
    atomic.module_pre_dungeon_initial_entrance();
    staged.begin_selected_game_entrance_before_selection();
    assert_eq!(
        staged.game_state.dungeon.room_tracking.room_index2_word(),
        0
    );
    assert_eq!(staged.game_state.player.follower_link.x(), 0x1353);
    assert_eq!(staged.game_state.player.follower_link.y(), 0x058b);
    staged.module_pre_dungeon_initial_entrance();
    assert_eq!(staged.game_state, atomic.game_state);
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn parity_probe_overworld_screen_loads_screen_properties() {
    let mut state = ZeldaState::new();
    state.assets = Some(probe_overworld_asset_pack(0x005a));
    state.set_indoor_flag(1);

    let screen = state.parity_probe_overworld_screen(0x005a);

    assert_eq!(screen, 0x005a);
    assert_eq!(read_le_u16(&state.ram, 0x008a), 0x005a);
    assert_eq!(state.ram[0x001b], 0);
}

#[test]
fn overworld_map16_stripes_follow_rom_long_indexed_wram_reads_past_bg2_page() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 71];
    put_test_asset(&mut data, &mut ranges, 70, vec![0; 9 * 8]);

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));
    state.set_overworld_map16_load_state(OverworldMap16LoadState {
        src_off: 0x1802,
        dst_off: 0x001a,
        y_unit: 0x0008,
    });
    state.set_screen_transition_direction_bits(1);

    let crossed_page_words = [
        0x0dc4, 0x0c68, 0x0270, 0x0271, 0x0c6c, 0x0272, 0x0273, 0x0c69,
    ];
    for (index, value) in crossed_page_words.into_iter().enumerate() {
        let source_offset = 0x2032 + index * 0x80;
        write_le_u16(&mut state.ram, DUNG_BG2 + source_offset, value);
    }

    state.BufferAndBuildMap16Stripes_X(0);

    let captured = std::array::from_fn(|index| {
        state
            .game_state
            .world
            .transient
            .dung_replacement_tile_state(index)
    });
    assert_eq!(captured, crossed_page_words);
}

#[test]
fn overworld_sprite_progress_is_shadow_only_without_its_native_c_continuation() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let receipt = OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
        crate::OverworldSpriteReloadProgress::PresencePublished,
    );
    state.original_timing_semantic_receipts =
        Some(OriginalTimingHostReceipts::new(0, 0, vec![receipt]));

    assert!(state
        .take_original_timing_overworld_sprite_reload_progress()
        .is_empty());
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        &[receipt],
        "an inactive native domain must not consume or apply the authority receipt",
    );
}

#[test]
fn live_overworld_load_overlays_keeps_entry_sprite_slots_until_source_return() {
    let mut state = ZeldaState::new();
    {
        let mut sprite = state.sprite_slot_view_mut(9);
        sprite.set_sprite_type(0x0c);
        sprite.set_state(0);
        sprite.set_x(0x0f1c);
        sprite.set_y(0x0343);
    }
    {
        let mut sprite = state.sprite_slot_view_mut(11);
        sprite.set_sprite_type(0x58);
        sprite.set_state(9);
    }
    let entry_slots = state.game_state.sprites.sprite_slots.clone();
    {
        let mut sprite = state.sprite_slot_view_mut(9);
        sprite.set_sprite_type(0x55);
        sprite.set_state(8);
        sprite.set_x(0x0340);
        sprite.set_y(0x0310);
    }
    {
        let mut sprite = state.sprite_slot_view_mut(11);
        sprite.set_n_word(0x0c3c);
        sprite.set_sprite_type(0x56);
        sprite.set_state(8);
    }

    state.pending_overworld_sprite_activations = Some(std::collections::VecDeque::from([(
        11,
        state.game_state.sprites.sprite_slots.clone(),
    )]));
    state.defer_module09_sprite_slots_until_reload_return(entry_slots);
    state.sprite_disable_live_slots_before_deferred_overworld_reload();

    assert_eq!(state.sprite_slot_view(9).sprite_type(), 0x0c);
    assert_eq!(state.sprite_slot_view(9).state(), 0);
    assert_eq!(state.sprite_slot_view(9).x(), 0x0f1c);
    assert_eq!(state.sprite_slot_view(9).y(), 0x0343);
    assert_eq!(state.sprite_slot_view(11).state(), 0);

    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload,
        1,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::SpriteActivated {
                    block: 0x0c3c,
                    slot: 11,
                    sprite_type: 0x56,
                },
            ),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x034e },
            ),
        ],
    ));
    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    assert!(!state.apply_original_timing_overworld_sprite_reload_progress(progress));

    assert_eq!(state.sprite_slot_view(9).sprite_type(), 0x0c);
    assert_eq!(state.sprite_slot_view(9).state(), 0);
    assert_eq!(state.sprite_slot_view(11).sprite_type(), 0x56);
    assert_eq!(state.sprite_slot_view(11).state(), 8);
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            read_le_u16(&state.ram, BG2_X_SCROLL),
        ),
        (0x034e, 0x034e),
    );

    state.publish_deferred_module09_sprite_slots_at_reload_return();

    assert_eq!(state.sprite_slot_view(9).sprite_type(), 0x55);
    assert_eq!(state.sprite_slot_view(9).state(), 8);
    assert_eq!(state.sprite_slot_view(9).x(), 0x0340);
    assert_eq!(state.sprite_slot_view(9).y(), 0x0310);
    assert_eq!(state.sprite_slot_view(11).state(), 8);
}

#[test]
fn deferred_overworld_reload_preserves_reused_slot_activations() {
    // ROM host346491 activates block480/type41 then block420/typeAC in slot10.
    // Overworld_AllocSprite permits reuse of type41 when its retained C is nonzero.
    let mut state = ZeldaState::new();
    for slot in 11..=13 {
        state.sprite_slot_view_mut(slot).set_state(9);
    }
    state.sprite_slot_view_mut(10).set_c(1);
    state.set_overworld_sprite_presence_marker(480, 0x42);
    state.set_overworld_sprite_presence_marker(420, 0xad);
    let entry_slots = state.game_state.sprites.sprite_slots.clone();
    state.pending_overworld_sprite_activations = Some(std::collections::VecDeque::new());
    state.overworld_load_proxima_sprite_if_alive(480);
    state.overworld_load_proxima_sprite_if_alive(420);
    assert_eq!(state.sprite_slot_view(10).n_word(), 420);
    state.defer_module09_sprite_slots_until_reload_return(entry_slots);
    state.publish_deferred_module09_sprite_slot(10);
    assert_eq!(state.sprite_slot_view(10).n_word(), 480);
    assert_eq!(state.sprite_slot_view(10).sprite_type(), 0x41);
    state.publish_deferred_module09_sprite_slot(10);
    assert_eq!(state.sprite_slot_view(10).n_word(), 420);
    assert_eq!(state.sprite_slot_view(10).sprite_type(), 0xac);
    assert!(state
        .pending_overworld_sprite_activations
        .as_ref()
        .unwrap()
        .is_empty());
    state.publish_deferred_module09_sprite_slots_at_reload_return();
}

#[test]
fn live_pre_overworld_properties_applies_source_proximity_scan_coordinate() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_bg2_x(0x0700);
    state.set_overworld_horizontal_scroll_delta_low(0x34);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishPreOverworldProperties {
            overworld_screen: 43,
            sprite_presence_published: false,
        },
        3,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        652_122,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::PresencePublished,
            ),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x0720 },
            ),
        ],
    ));

    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    assert!(!state.apply_original_timing_overworld_sprite_reload_progress(progress));
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            read_le_u16(&state.ram, BG2_X_SCROLL),
        ),
        (0x0720, 0x0720),
    );
    assert_eq!(state.overworld_horizontal_scroll_delta_low(), 0xff);

    state.complete_pre_overworld_load_properties_after_sprite_reset_with_presence(43, true);
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            read_le_u16(&state.ram, BG2_X_SCROLL),
            state.overworld_horizontal_scroll_delta_low(),
        ),
        (0x0700, 0x0700, 0x34),
    );
}

#[test]
fn live_overworld_load_overlays_generation_return_starts_the_overlay_phase() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(0x0b);
    state.set_submodule(0x25);
    state.pending_overworld_sprite_reload_slots =
        Some(state.game_state.sprites.sprite_slots.clone());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload,
        1,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        172_810,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ],
    ));

    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    let returned = state.apply_original_timing_overworld_sprite_reload_progress(progress);
    assert!(returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload,
        )),
    );

    assert!(!state.complete_overworld_load_overlays_after_sprite_reload());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldLoadOverlaysOverlay,
        OVERWORLD_LOAD_OVERLAYS_OVERLAY_NMI_SLICES,
    );
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishOverworldLoadOverlaysOverlay),
    );
    assert_eq!(state.game_state.frame.submodule, 0x25);
}

#[test]
fn live_whirlpool_load_overlays_consumes_reload_return_before_overlay_phase() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(0x2e);
    state.set_subsubmodule(3);
    state.pending_overworld_sprite_reload_slots =
        Some(state.game_state.sprites.sprite_slots.clone());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishModule09LongLoad {
            step: Module09LongLoadStep::LoadOverlays2,
        },
        6,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        183_225,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ],
    ));

    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    let returned = state.apply_original_timing_overworld_sprite_reload_progress(progress);
    assert!(returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishModule09LongLoad {
                step: Module09LongLoadStep::LoadOverlays2,
            },
        )),
    );

    state.complete_module09_long_load_step(Module09LongLoadStep::LoadOverlays2);

    assert!(state.pending_overworld_sprite_reload_slots.is_none());
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishModule09LongLoad {
            step: Module09LongLoadStep::LoadOverlays2Overlay,
        }),
    );
    assert_eq!(state.game_state.frame.submodule, 0x2e);
    assert_eq!(state.game_state.frame.subsubmodule, 3);
}

#[test]
fn whirlpool_bird_travel_reload_retains_the_first_reset_generation() {
    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::parse(&assets).unwrap());
    let whirlpool_areas = state
        .asset_raw(123)
        .expect("test asset pack omitted kWhirlpoolAreas");
    let source_screen = u16::from(whirlpool_areas[0]) | (u16::from(whirlpool_areas[1]) << 8);
    state.set_overworld_screen_word(source_screen);
    state.set_overworld_area_index_word(source_screen);
    state.sprite_slot_view_mut(11).set_state(9);
    state.sprite_slot_view_mut(11).set_sprite_type(0x58);
    state.sprite_set_x(11, 0x0ef4);
    state.sprite_set_y(11, 0x02a4);

    state.begin_deferred_whirlpool_partner_exit_sprite_reload();

    let slot = state.sprite_slot_view(11);
    assert_eq!(slot.state(), 0);
    assert_eq!(slot.sprite_type(), 0x58);
    assert_eq!(slot.x(), 0x0ef4);
    assert_eq!(slot.y(), 0x02a4);
    assert!(state.pending_overworld_sprite_reload_slots.is_some());
}

#[test]
fn special_exit_mosaic_waits_for_its_source_stage_return() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishOverworldSpecialExitMosaic, 1);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        187_510,
        0,
        vec![OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned],
    ));

    let returned = state.take_original_timing_overworld_special_exit_mosaic_returned();
    assert!(returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishOverworldSpecialExitMosaic,
        )),
    );
}

#[test]
fn special_exit_mosaic_restore_and_second_decode_are_distinct_source_stages() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishOverworldSpecialExitMosaic, 4);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        266_268,
        0,
        vec![OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored],
    ));

    let restored = state.take_original_timing_overworld_special_exit_mosaic_restored();
    let returned = state.take_original_timing_overworld_special_exit_mosaic_returned();
    assert!(restored);
    assert!(!returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(restored || returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishOverworldSpecialExitMosaic,
        )),
    );

    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode,
        4,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        266_272,
        0,
        vec![OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned],
    ));
    let restored = state.take_original_timing_overworld_special_exit_mosaic_restored();
    let returned = state.take_original_timing_overworld_special_exit_mosaic_returned();
    assert!(!restored);
    assert!(returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode,
        )),
    );
}

#[test]
fn live_mirror_warp_reload_retains_and_restores_source_scan_locals() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(0x23);
    state.set_bg2_x(0x01ea);
    state.set_overworld_horizontal_scroll_delta_low(0x34);
    state.pending_overworld_sprite_reload_slots =
        Some(state.game_state.sprites.sprite_slots.clone());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishModule09LongLoad {
            step: Module09LongLoadStep::MirrorWarpSpriteLoadReload,
        },
        3,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        655_820,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::PresencePublished,
            ),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x01fa },
            ),
        ],
    ));

    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    assert!(!state.apply_original_timing_overworld_sprite_reload_progress(progress));
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            state.overworld_horizontal_scroll_delta_low(),
        ),
        (0x01fa, 0xff),
    );

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        655_822,
        0,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ],
    ));
    let progress = state.take_original_timing_overworld_sprite_reload_progress();
    let returned = state.apply_original_timing_overworld_sprite_reload_progress(progress);
    assert!(!returned);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishModule09LongLoad {
            step: Module09LongLoadStep::MirrorWarpSpriteLoadTail,
        }),
    );
    assert!(state.pending_overworld_sprite_reload_slots.is_none());
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            state.overworld_horizontal_scroll_delta_low(),
        ),
        (0x01ea, 0x34),
    );
}

#[test]
fn overworld_reload_tail_waits_for_its_nested_module09_sprite_main_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(9);
    state.set_submodule(4);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    // A carried Held handler always leaves its acceptance host's receptive
    // snapshot behind; the uninterrupted scheduled-caller timeline now
    // validates that pairing for reload-tail hosts.
    state.capture_and_carry_original_timing_nmi_publication_at_host_return();
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishOverworldSpriteReloadTail {
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::BeforeNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(NmiPhase::BeforeNmi),
        },
        1,
    );
    state.pending_overworld_sprite_reload_slots =
        Some(state.game_state.sprites.sprite_slots.clone());
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        38_517,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                crate::OverworldSpriteReloadProgress::ReloadReturned,
            ),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                crate::SpriteMainProgress::AfterSlot(14),
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 5);
    assert_eq!(state.game_execution_scheduler.pre_main_nmi_resume(), None);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(14),
            caller: SpriteMainCpuCaller::Module09 {
                boundary: OriginalTimingBoundary::HostReturn,
            },
        })
    ));
    assert!(matches!(
        state
            .active_module09_sprite_main_return
            .expect("nested Module09 Sprite_Main frame was lost")
            .after_sprite_main,
        Module09AfterSpriteMain::FinishOverworldSpriteReload {
            post_return_hold_nmi_slices: 0,
            epilogue_phase: NmiPhase::BeforeNmi,
            ..
        }
    ));
}

#[test]
fn live_world_map_ambient_return_owns_native_map8_completion() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(0x21);
    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishWorldMapAmbientMap8, 3);

    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(state.game_state.frame.submodule, 0x21);

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::WorldMapAmbientMap8Returned],
    ));
    let returned = state.take_original_timing_world_map_ambient_map8_returned();
    assert!(returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishWorldMapAmbientMap8,
        )),
    );
    assert_eq!(state.game_state.frame.submodule, 0x21);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn live_world_map_overlay_return_owns_native_reload_completion() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(0x20);
    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishWorldMapOverlayReload, 7);

    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned],
    ));
    let returned = state.take_original_timing_world_map_overlay_reload_returned();
    assert!(returned);
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishWorldMapOverlayReload,
        )),
    );
    assert_eq!(state.game_state.frame.submodule, 0x20);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn overworld_tile_attribute_uses_map16_and_map8_assets() {
    let mut state = ZeldaState::new();
    state.set_overworld_offset_base_y(0x20);
    state.set_overworld_offset_mask_y(0x1f);
    state.set_overworld_offset_base_x(3);
    state.set_overworld_offset_mask_x(0x3f);
    state.dungeon_room_tilemaps_mut().set_bg2_tile(32, 5);

    let mut data = vec![0; 0x100];
    write_le_u16(&mut data, (5 * 4 + 2) * 2, 0x4007);
    data[0x80 + 7] = 0x10;
    let mut ranges = vec![(0, 0); 164];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    assert_eq!(
        state.overworld_tile_definition_at_location(4, 0x28),
        NativeTile::from_cartridge(0x11)
    );
}

#[test]
fn boomerang_bombs_book_and_desert_prayer_match_c_state() {
    let mut boom = ZeldaState::new();
    boom.follower_link_state_mut().set_filtered_joypad_h(0x40);
    boom.inventory_items_mut().set_inventory_item(1, 1);
    boom.link_item_boomerang();
    assert_eq!(boom.game_state.player.follower_link.item_in_hand(), 0x80);
    assert_eq!(boom.ram[FLAG_FOR_BOOMERANG_IN_PLACE], 1);
    assert_eq!(link_test_byte(&boom, LINK_DELAY_TIMER_SPIN_ATTACK), 6);
    assert_eq!(link_test_byte(&boom, LINK_CANT_CHANGE_DIRECTION) & 1, 1);

    set_link_test_byte(&mut boom, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    boom.ram[PLAYER_HANDLER_TIMER] = 1;
    boom.link_item_boomerang();
    assert_eq!(boom.game_state.player.follower_link.item_in_hand(), 0);
    assert_eq!(boom.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        boom.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(link_test_byte(&boom, LINK_CANT_CHANGE_DIRECTION) & 1, 0);

    let mut bombs = ZeldaState::new();
    bombs.follower_link_state_mut().set_filtered_joypad_h(0x40);
    bombs.player_resources_mut().set_bombs(1);
    bombs.link_item_bombs();
    // C `AncillaAdd_Bomb(7, 1)` allocates via `Ancilla_AllocInit(7, 1)`, which
    // for ancilla types 7/8 walks slots [limit..0], so slot 1 receives the
    // bomb ancilla. See zelda3/src/ancilla.c:5763 and ancilla.c:6990.
    assert_eq!(bombs.ancilla_slot_view(1).ancilla_type(), 7);
    assert_eq!(
        bombs.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(link_test_byte(&bombs, LINK_ITEM_BOMBS), 0);
    assert_eq!(bombs.game_state.player.follower_link.item_in_hand(), 0);

    let mut book = ZeldaState::new();
    book.follower_link_state_mut().set_filtered_joypad_h(0x40);
    book.link_item_book();
    assert_eq!(book.game_state.system_signals.sound_effect_1() & 0x3f, 60);

    let mut prayer = ZeldaState::new();
    prayer.follower_link_state_mut().set_filtered_joypad_h(0x40);
    prayer.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = 1;
    prayer.set_main_module(9);
    set_link_test_byte(&mut prayer, LINK_DIRECTION, 0x0f);
    prayer.link_item_book();
    assert_eq!(prayer.game_state.frame.submodule, 5);
    assert_eq!(prayer.game_state.frame.saved_module_for_menu, 9);
    assert_eq!(prayer.game_state.frame.main_module, 14);
    assert_eq!(prayer.game_state.frame.modal_pause_flag, 1);
    assert_eq!(
        prayer
            .game_state
            .player
            .follower_link
            .y_button_action_timer(),
        22
    );
    assert_eq!(prayer.game_state.player.follower_link.state_bits(), 2);
    assert_eq!(link_test_byte(&prayer, LINK_DIRECTION), 0);
    assert_eq!(prayer.game_state.system_signals.ambient_sound_effect(), 17);
    assert_eq!(prayer.game_state.system_signals.music_control(), 242);
}

#[test]
fn flute_item_countdown_and_weather_vane_branch_match_c_state() {
    let mut countdown = ZeldaState::new();
    countdown
        .follower_link_state_mut()
        .set_button_mask_b_y(0x40);
    countdown.ram[FLUTE_COUNTDOWN] = 2;
    countdown.link_item_flute();
    assert_eq!(countdown.ram[FLUTE_COUNTDOWN], 1);
    assert_eq!(
        countdown.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0x40
    );

    let mut flute = ZeldaState::new();
    flute.follower_link_state_mut().set_filtered_joypad_h(0x40);
    flute.inventory_items_mut().set_flute(2);
    flute.set_overworld_screen_word(0x18);
    set_link_test_word(&mut flute, LINK_Y_COORD, 0x780);
    set_link_test_word(&mut flute, LINK_X_COORD, 0x200);
    flute.link_item_flute();
    assert_eq!(flute.ram[FLUTE_COUNTDOWN], 128);
    assert_eq!(flute.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(flute.game_state.frame.submodule, 45);
    assert_eq!(flute.ancilla_slot_view(4).ancilla_type(), 55);

    let mut shovel_dispatch = ZeldaState::new();
    shovel_dispatch
        .follower_link_state_mut()
        .set_filtered_joypad_h(0x40);
    shovel_dispatch.inventory_items_mut().set_flute(1);
    shovel_dispatch.link_item_shovel_and_flute();
    assert_eq!(
        shovel_dispatch
            .game_state
            .player
            .follower_link
            .position_mode(),
        1
    );
}

#[test]
fn mirror_item_crossing_and_follower_cleanup_match_core_state() {
    let mut mirror = ZeldaState::new();
    mirror.follower_link_state_mut().set_filtered_joypad_h(0x40);
    mirror.enhanced_features_mut().set_bits(8);
    mirror.set_overworld_screen_word(0x40);
    set_link_test_word(&mut mirror, LINK_Y_COORD, 0x1234);
    set_link_test_word(&mut mirror, LINK_X_COORD, 0x5678);
    mirror.follower_link_state_mut().set_actual_x_velocity(7);
    mirror.follower_link_state_mut().set_actual_y_velocity(9);
    mirror.link_item_mirror();
    assert_eq!(mirror.ram[LAST_LIGHT_VS_DARK_WORLD], 0x40);
    assert_eq!(mirror.bird_travel_destination(15).y, 0x1234);
    assert_eq!(mirror.bird_travel_destination(15).x, 0x5678);
    assert_eq!(mirror.game_state.frame.submodule, 35);
    assert_eq!(
        link_test_byte(&mirror, LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE),
        1
    );
    assert_eq!(mirror.game_state.player.follower_link.handler_state(), 20);
    assert_eq!(
        mirror.game_state.player.follower_link.actual_x_velocity(),
        0
    );
    assert_eq!(
        mirror.game_state.player.follower_link.actual_y_velocity(),
        0
    );

    let mut crossing = ZeldaState::new();
    crossing.ram[LAST_LIGHT_VS_DARK_WORLD] = 0;
    crossing.set_overworld_screen_word(0x40);
    let mut data = vec![0; 0x100];
    data[0x80] = 1;
    let mut ranges = vec![(0, 0); 164];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    crossing.assets = Some(AssetPack::from_data_ranges(data, ranges));
    crossing.link_state_crossing_worlds();
    assert_eq!(crossing.game_state.frame.submodule, 44);
    assert_eq!(crossing.game_state.player.follower_link.handler_state(), 20);

    let mut follower = ZeldaState::new();
    follower.follower_state_mut().set_indicator(13);
    follower.follower_state_mut().set_dropped(1);
    set_link_test_byte(&mut follower, LINK_CAPE_MODE, 1);
    follower.handle_followers_after_mirroring();
    assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_TIMER], 0xfe);
    assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_COUNTER], 0);
    assert_eq!(follower.ram[FOLLOWER_INDICATOR], 0);
    assert_eq!(link_test_byte(&follower, LINK_CAPE_MODE), 0);
    assert_eq!(link_test_byte(&follower, LINK_BUNNY_TRANSFORM_TIMER), 0);
}

#[test]
fn item_tile_behavior_routes_overworld_attr_to_tile_execute() {
    let mut state = ZeldaState::new();
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state.set_overworld_offset_mask_y(0x1f);
    state.set_overworld_offset_mask_x(0x3f);
    state.dungeon_room_tilemaps_mut().set_bg2_tile(16, 7);

    let mut data = vec![0; 0x100];
    write_le_u16(&mut data, 7 * 4 * 2, 3);
    data[0x80 + 3] = 1;
    let mut ranges = vec![(0, 0); 164];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    state.tile_detect_main_handler(1);

    assert_eq!(state.game_state.player.tile_detection.collision_bits(), 0);
    assert_eq!(read_le_u16(&state.ram, TILEDETECT_NORMAL_TILES), 1);
}

#[test]
fn world_map_fade_completion_runs_the_first_mode7_tick_immediately() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_sequence(1);
    state.attract_scene_mut().set_state(4);
    state.attract_scene_mut().set_mode7_zoom_timer(0xff);
    state.attract_scene_mut().set_scene_timer(1);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 0);
    assert_eq!(
        state.game_state.ending.attract_scene.mode7_zoom_timer(),
        0xfe
    );
    assert_eq!(state.spotlight_hdma_table_dynamic_entry(0), 0x0174);
}

#[test]
fn world_map_rom_work_completes_after_the_five_snes9x_observed_nmi_slices() {
    let mut work = ScheduledGameWork::schedule(GameWorkContinuation::FinishAttractWorldMap, 5);

    for _ in 0..4 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractWorldMap)
    );
}

#[test]
fn player_world_map_load_completes_after_the_five_post_entry_nmi_slices() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishWorldMapLightLoad,
        WORLD_MAP_LIGHT_LOAD_NMI_SLICES,
    );

    for _ in 0..WORLD_MAP_LIGHT_LOAD_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapLightLoad)
    );
}

#[test]
fn world_map_exit_tilesets_resume_after_the_measured_nmi_slices() {
    assert_eq!(WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES, 33);
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishWorldMapExitTilesets,
        WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES,
    );

    for _ in 0..WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapExitTilesets)
    );
}

#[test]
fn module09_world_map_bodies_use_the_original_rom_cpu_schedule() {
    // Production captures these plans by executing the original ROM from its
    // $00:8034 main wait. Cold Snes9x proves $09/$20 crosses six body NMIs and
    // one caller NMI after Sprite_Main slot 8, while $09/$21 crosses only three.
    for (submodule, schedule, expected_work, expected_slices) in [
        (
            0x20,
            Module09CpuSchedule {
                submodule_nmis: 6,
                caller_nmis: 1,
                caller_sprite_main_nmis: 1,
                caller_suffix_nmis: 0,
                caller_first_nmi_phase: Some(ModuleCpuPhase::InterruptedInSpriteMain),
                sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(8)),
            },
            GameWorkContinuation::FinishWorldMapOverlayReload,
            6,
        ),
        (
            0x21,
            Module09CpuSchedule {
                submodule_nmis: 3,
                ..Module09CpuSchedule::default()
            },
            GameWorkContinuation::FinishWorldMapAmbientMap8,
            3,
        ),
    ] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(9);
        state.set_submodule(submodule);
        state.module09_cpu_schedule = Some(schedule);
        state.game_execution_scheduler.begin_host_frame();
        state
            .game_execution_scheduler
            .mark_main_iteration_after_leading_nmi();
        state.game_execution_scheduler.begin_main_loop_iteration();

        if submodule == 0x20 {
            state.Overworld_LoadOverlays2();
        } else {
            state.Overworld_LoadAmbientOverlayFalse();
        }

        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(expected_work)
        );
        assert_eq!(
            state
                .game_execution_scheduler
                .scheduled_work_slices_remaining(),
            Some(expected_slices),
        );
        assert_eq!(state.module09_cpu_schedule, Some(schedule));
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
    }
}

#[test]
fn c_module09_overlay_return_has_no_self_healing_counter_transient() {
    // C Module09_Overworld keeps its four scroll locals live across
    // Sprite_Main. The ROM trace for run 6168 enters Sprite_ExecuteSingle with
    // X=7 and $12=1, proving slots 15..8 returned before NMI. Run 6169 resumes
    // the caller and clears $12 while $1a remains $69. Only run 6170's leading
    // NMI precedes INC $1a.
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_indoor_flag(0);
    state.set_main_module(9);
    state.set_submodule(0x21);
    state.set_frame_counter(0x69);
    state.set_bg2_h_copy2(0x1010);
    state.set_bg2_v_copy2(0x2020);
    state.set_bg1_h_copy2(0x3030);
    state.set_bg1_v_copy2(0x4040);
    state.set_bg1_x_offset(3);
    state.set_bg1_y_offset(5);
    state.latch_nmi_update();
    state.set_pending_nmi_subroutine(4);
    state.set_core_update_disable_flag(4);

    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishWorldMapOverlayReload, 1);
    state.game_execution_scheduler.begin_host_frame();
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishWorldMapOverlayReload,
        )),
    );
    assert!(state
        .game_execution_scheduler
        .resumed_call_stack_is_before_nmi());

    state.begin_world_map_overlay_module09_sprite_return(SpriteMainCpuBoundary::AfterSlot(8), 1);
    let caller = Module09SpriteMainReturn {
        bg2_x: 0x1010,
        bg2_y: 0x2020,
        bg1_x: 0x3030,
        bg1_y: 0x4040,
    };
    let continuation = GameWorkContinuation::FinishSpriteMain {
        boundary: SpriteMainCpuBoundary::AfterSlot(8),
        caller: SpriteMainCpuCaller::WorldMapOverlayReload { module09: caller },
    };
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(continuation)
    );

    state.capture_display_snapshot();
    state.interrupt_nmi_for_active_scanout(0, None, false);
    assert_eq!(state.game_state.frame.frame_counter, 0x69);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 4);

    assert_eq!(
        state
            .game_execution_scheduler
            .take_after_current_trailing_nmi(),
        Some(continuation),
    );
    state.complete_post_trailing_nmi_continuation(continuation, 0, false, false);
    assert_eq!(state.game_state.frame.frame_counter, 0x69);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 4);
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        0x1010
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        0x2020
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg1_h_copy2(),
        0x3030
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg1_v_copy2(),
        0x4040
    );
    state
        .game_execution_scheduler
        .finish_call_stack_at_main_wait_before_nmi();

    state.game_execution_scheduler.begin_host_frame();
    assert!(state
        .game_execution_scheduler
        .main_return_requires_leading_nmi());
    assert_eq!(state.game_state.frame.frame_counter, 0x69);

    state.module09_cpu_schedule = Some(Module09CpuSchedule {
        submodule_nmis: 3,
        ..Module09CpuSchedule::default()
    });
    state.capture_display_snapshot();
    state.interrupt_nmi_for_active_scanout(0, None, false);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.frame.frame_counter, 0x69);

    state.zelda_run_game_loop_after_leading_nmi();
    assert_eq!(state.game_state.frame.frame_counter, 0x6a);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishWorldMapAmbientMap8),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(3),
    );
}

#[test]
fn overworld_animated_bg_vram_generation_follows_scanout_authority() {
    assert_eq!(
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi.resolve_live_override(false),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi.resolve_live_override(true),
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        rom_graphics_dma_plan(7, 0x0f).animated_bg_scanout,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
}

#[test]
fn pre_overworld_load_models_measured_snes9x_nmi_boundaries() {
    let screen_build_workload = OverworldMapGraphicsWorkload {
        map32_definition_changes: 796,
    };
    let screen_build_timing = overworld_map_and_sprite_graphics_timing(screen_build_workload);
    let sprite_reset = GameWorkContinuation::PreOverworldPropertiesSpriteReset {
        overworld_screen: 0x00,
        animated_tiles: 0x58,
    };
    let stages = [
        (
            sprite_reset,
            PRE_OVERWORLD_PROPERTIES_TO_SPRITE_RESET_NMI_SLICES,
        ),
        (
            GameWorkContinuation::FinishPreOverworldProperties {
                overworld_screen: 0x00,
                sprite_presence_published: false,
            },
            PRE_OVERWORLD_PROPERTIES_AFTER_SPRITE_RESET_NMI_SLICES,
        ),
        (
            GameWorkContinuation::FinishPreOverworldOverlays,
            PRE_OVERWORLD_OVERLAYS_NMI_SLICES,
        ),
    ];

    for (continuation, nmi_slices) in stages {
        let mut work = ScheduledGameWork::schedule(continuation, nmi_slices);
        for _ in 0..nmi_slices - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
    }

    let screen_build_nmi_slices = screen_build_timing.quadrant_load_nmi_slices
        + screen_build_timing.map16_to_map8_tail_nmi_slices;
    let continuation = GameWorkContinuation::FinishPreOverworldScreenBuild;
    let mut work =
        ScheduledGameWork::schedule_before_trailing_nmi(continuation, screen_build_nmi_slices);
    // Module08_02 starts before the entry frame's trailing NMI, so only the
    // subsequent 16 boundaries are consumed by future host calls.
    for _ in 1..screen_build_nmi_slices - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(continuation)
    );
}

#[test]
fn pre_overworld_live_authority_holds_and_completes_scheduled_work_exactly_once() {
    let continuation = GameWorkContinuation::FinishPreOverworldOverlays;
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(continuation, 2);

    // The native estimate remains a shadow while Live owns completion. Even
    // after it reaches zero, gameplay must wait for the semantic C-call
    // receipt instead of completing from the old fixed envelope.
    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting)
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting)
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting)
    );

    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(true),
        Some(GameWorkStep::Complete(continuation))
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(true),
        None
    );

    // Authority may also prove a source call returned before the native
    // estimate expires. That receipt is the completion fact, not the estimate.
    scheduler.schedule_work(continuation, 5);
    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(true),
        Some(GameWorkStep::Complete(continuation))
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice_with_authoritative_completion(false),
        None
    );
}

#[test]
fn pre_overworld_sprite_reset_phase_clears_old_sprite_slots_before_the_reload_tail() {
    let mut state = ZeldaState::new();
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_sprite_type(0x73);
        sprite.set_state(9);
    }

    state.sprite_begin_reload_all_overworld();

    assert_eq!(state.sprite_slot_view(0).state(), 0);
    assert_eq!(state.sprite_slot_view(0).sprite_type(), 0x73);
}

#[test]
fn world_map_fade_publishes_the_previous_scanout_snapshot() {
    assert!(!rom_attract_world_map_display_is_one_frame_deferred(
        20, 0, 1, 3
    ));
    assert!(rom_attract_world_map_display_is_one_frame_deferred(
        20, 0, 1, 4
    ));
    assert!(!rom_attract_world_map_display_is_one_frame_deferred(
        20, 0, 0, 4
    ));
    assert!(!rom_attract_world_map_display_is_one_frame_deferred(
        0, 0, 1, 4
    ));
}

#[test]
fn world_map_fade_out_uses_the_preceding_sprite_main_workload() {
    let mut state = ZeldaState::new();
    state.ppu.forced_blank = false;
    state.set_screen_brightness(1);
    state.set_overworld_map_state(0);
    let mut workload = SpriteMainTimingWorkload::default();
    workload.record_active_sprite(0x6c, 0);
    workload.record_active_sprite(0x3f, 0);
    workload.record_active_sprite(0x3f, 0);
    workload.record_garnish_table(true, 0);
    state.last_sprite_main_timing_workload = Some(workload);

    state.WorldMap_FadeOut();

    assert_eq!(state.game_state.display.screen_brightness, 0x80);
    assert_eq!(state.overworld_map_state(), 1);
    assert!(!state.ppu.forced_blank);
    assert_eq!(state.active_display_force_blank_event, Some(48));
    assert!(live_forced_blank_for_scanout(
        state.ppu.forced_blank,
        None,
        state.active_display_force_blank_event,
        false,
    ));
    assert_eq!(state.ppu.forced_blank_from_scanline, None);
}

#[test]
fn overworld_sprite_reload_timing_tracks_the_measured_rom_workload() {
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 2,
                in_bounds_proximity_checks: 18,
            },
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 3,
            post_return_hold_nmi_slices: 1,
            return_phase: NmiPhase::BeforeNmi,
            epilogue_phase: NmiPhase::AfterNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::ByReturnPhase(
                NmiPhase::BeforeNmi,
            ),
        }
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 4,
                in_bounds_proximity_checks: 90,
            },
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 4,
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::AfterNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::ByReturnPhase(NmiPhase::AfterNmi,),
        }
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 8,
                in_bounds_proximity_checks: 66,
            },
            OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail,
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 2,
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::AfterNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
        }
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 2,
                in_bounds_proximity_checks: 18,
            },
            OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail,
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 2,
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::BeforeNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
        }
    );
}

#[test]
fn completed_overworld_reload_uses_its_measured_return_phase() {
    let cpu_slice_entry = BgScrollRegisterScanout {
        offsets: [[0x1111, 0x2222]; 4],
    };
    assert_eq!(
        GameWorkContinuation::FinishOverworldAuxGraphics
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
    );
    assert_eq!(
        GameWorkContinuation::FinishOverworldMosaicSpriteGraphics
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
    );
    // The completion boundary still owns the scroll generation measured at
    // the CPU return. Entry geometry belongs to the following resume boundary.
    for post_return_hold_nmi_slices in [0, 1] {
        for (return_phase, generation) in [
            (
                NmiPhase::BeforeNmi,
                Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
            ),
            (
                NmiPhase::AfterNmi,
                Some(DisplayBgScrollGeneration::RetainCpuSliceEntry(
                    cpu_slice_entry,
                )),
            ),
        ] {
            let publication = GameWorkContinuation::FinishOverworldSpriteReloadTail {
                post_return_hold_nmi_slices,
                return_phase,
                epilogue_phase: NmiPhase::BeforeNmi,
                resume_scanout: OverworldSpriteReloadResumeScanout::CpuSliceEntry {
                    scroll: cpu_slice_entry,
                    bg1_generation: OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn,
                },
            }
            .completion_publication(cpu_slice_entry);
            assert_eq!(publication.bg_scroll, generation);
            assert_eq!(
                publication.obj,
                Some(ObjScanoutGenerations::coherent(
                    GraphicsDmaGeneration::HostBoundaryBeforeMain,
                )),
            );
        }
    }
    assert_eq!(
        GameWorkContinuation::HoldOverworldSpriteReloadReturn
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
    );
    assert_eq!(
        GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        None,
    );
}

#[test]
fn overworld_mosaic_sprite_graphics_defers_its_source_suffix() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_screen_brightness(0);
    state.set_hdma_enable_mask(0);
    state.set_mosaic_target_level(31);
    state.set_countdown(0);
    state.set_subsubmodule(0);

    state.OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic();

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishOverworldMosaicSpriteGraphics),
    );
    assert_eq!(state.ram[INIDISP_COPY], 0);
    assert_eq!(state.ram[HDMAEN_COPY], 0);
    assert_eq!(state.ram[MOSAIC_TARGET_LEVEL], 31);
    assert_eq!(state.ram[PALETTE_FILTER_COUNTDOWN], 0);
    assert_eq!(state.ram[SUBSUBMODULE], 0);

    state.complete_overworld_mosaic_sprite_graphics();

    assert_eq!(state.ram[INIDISP_COPY], 0x0f);
    assert_eq!(state.ram[HDMAEN_COPY], 0x80);
    assert_eq!(state.ram[MOSAIC_TARGET_LEVEL], 0);
    assert_eq!(state.ram[PALETTE_FILTER_COUNTDOWN], 30);
    assert_eq!(state.ram[SUBSUBMODULE], 1);
}

#[test]
fn overworld_reload_timing_keeps_resume_geometry_separate_from_return_phase() {
    let light = OverworldSpriteReloadWorkload {
        sprite_records: 2,
        in_bounds_proximity_checks: 18,
    };
    let heavy = OverworldSpriteReloadWorkload {
        sprite_records: 4,
        in_bounds_proximity_checks: 90,
    };

    assert_eq!(
        overworld_sprite_reload_timing(
            heavy,
            OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail,
        )
        .resume_boundary,
        OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            heavy,
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration,
        )
        .resume_boundary,
        OverworldSpriteReloadResumeBoundary::ByReturnPhase(NmiPhase::AfterNmi),
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            light,
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration,
        )
        .resume_boundary,
        OverworldSpriteReloadResumeBoundary::ByReturnPhase(NmiPhase::BeforeNmi),
    );
}

#[test]
fn overworld_reload_scanout_keeps_prepublished_rain_out_of_its_bg1_generation() {
    let entry = BgScrollRegisterScanout {
        offsets: [[0x1111, 0x2222], [0x3333, 0x4444], [0, 0], [0, 0]],
    };
    let returned = BgScrollRegisterScanout {
        offsets: [[0xaaaa, 0xbbbb], [0xcccc, 0xdddd], [0, 0], [0, 0]],
    };

    for (bg1_generation, expected_bg1) in [
        (
            OverworldSpriteReloadBg1Generation::RetainBeforePrepublishedRain,
            entry.offsets[0],
        ),
        (
            OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn,
            returned.offsets[0],
        ),
    ] {
        let completed = OverworldSpriteReloadResumeScanout::CpuSliceEntry {
            scroll: entry,
            bg1_generation,
        }
        .complete_transition_return(returned);
        let OverworldSpriteReloadResumeScanout::CpuSliceEntry { scroll, .. } = completed else {
            panic!("CPU-slice scanout changed variant");
        };
        assert_eq!(scroll.offsets[0], expected_bg1);
        assert_eq!(scroll.offsets[1], returned.offsets[1]);
    }
}

#[test]
fn c_overworld_handle_rain_color_math_branches_are_source_exact() {
    // C `src/overworld.c::OverworldOverlay_HandleRain` has exactly these
    // color-math branches: 3/88 -> $32, 5/44/90 -> $72, 36 -> SFX $36 and
    // $32, with every other frame leaving both values alone.
    for (frame_counter, expected_color_math, expected_sfx) in [
        (3, 0x32, 0x00),
        (88, 0x32, 0x00),
        (5, 0x72, 0x00),
        (44, 0x72, 0x00),
        (90, 0x72, 0x00),
        (36, 0x32, 0x36),
        (37, 0x55, 0x00),
    ] {
        let mut state = ZeldaState::new();
        state.game_state.frame.set_frame_counter(frame_counter);
        state.set_color_math_control(0x55);
        state.set_sound_effect_1(0);

        state.OverworldOverlay_HandleRain();

        assert_eq!(
            state.game_state.display.palette_filter.color_math_control(),
            expected_color_math,
            "frame counter {frame_counter}",
        );
        assert_eq!(
            state.game_state.system_signals.sound_effect_1(),
            expected_sfx,
            "frame counter {frame_counter}",
        );
    }
}
