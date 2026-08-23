use super::*;

#[test]
fn dungeon_map_terminal_fade_blanks_after_the_measured_active_prefix() {
    assert_eq!(
        dungeon_map_terminal_fade_blank_scanline(14, 3, 1, 1),
        Some(48)
    );
    assert_eq!(dungeon_map_terminal_fade_blank_scanline(14, 3, 1, 2), None);
    assert_eq!(dungeon_map_terminal_fade_blank_scanline(14, 2, 1, 1), None);
}

#[test]
fn published_one_shot_vcounter_irq_is_repeatable_and_advances_live_state() {
    let mut state = ZeldaState::new();
    state.set_irq_control_flag(0x80);
    state.set_select_file_name_scroll_x(0x01f0);
    state.ppu.bg_layer[2].v_scroll = 0x0318;
    state.capture_display_snapshot();

    assert_eq!(state.game_state.display.irq_control_flag, 0);
    assert_eq!(state.ram[0x0128], 0);

    let first = state.with_display_snapshot(ZeldaState::ppu_scanline_windows);

    assert_eq!(first[126].6[2], 0x0318);
    assert_eq!(first[127].6[2], 0);

    let second = state.with_display_snapshot(ZeldaState::ppu_scanline_windows);
    assert_eq!(second.as_ref(), first.as_ref());
    assert_eq!(state.game_state.display.irq_control_flag, 0);
    assert_eq!(state.ram[0x0128], 0);
}

#[test]
fn nmi_active_display_overrun_is_classified_by_workload() {
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(
            false,
            false,
            1,
            StripeUploadWork {
                packets: 6,
                transfer_bytes: 216,
                fixed_source_packets: 4,
                vertical_packets: 0,
            },
        ),
        NmiActiveDisplayBlanking {
            prefix_scanlines: 0,
            suffix_start_scanline: Some(1),
        }
    );
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(
            true,
            false,
            1,
            StripeUploadWork {
                packets: 6,
                transfer_bytes: 216,
                fixed_source_packets: 4,
                vertical_packets: 0,
            },
        ),
        NmiActiveDisplayBlanking::default()
    );
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(
            false,
            false,
            1,
            StripeUploadWork {
                packets: 6,
                transfer_bytes: 228,
                fixed_source_packets: 0,
                vertical_packets: 0,
            },
        ),
        NmiActiveDisplayBlanking::default()
    );
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(false, false, 1, StripeUploadWork::default(),),
        NmiActiveDisplayBlanking::default()
    );
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(
            false,
            true,
            5,
            StripeUploadWork {
                transfer_bytes: 1_936,
                ..StripeUploadWork::default()
            },
        ),
        NmiActiveDisplayBlanking {
            prefix_scanlines: 50,
            suffix_start_scanline: None,
        }
    );
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(
            false,
            false,
            5,
            StripeUploadWork {
                transfer_bytes: 1_936,
                ..StripeUploadWork::default()
            },
        ),
        NmiActiveDisplayBlanking::default()
    );
    assert_eq!(
        nmi_active_display_blanking_for_pending_work(
            true,
            true,
            5,
            StripeUploadWork {
                transfer_bytes: 1_936,
                ..StripeUploadWork::default()
            },
        ),
        NmiActiveDisplayBlanking::default()
    );
    let mut checkerboard_packet = vec![0x00, 0x10, 0x07, 0xff];
    checkerboard_packet.extend(std::iter::repeat_n(0, 0x800));
    checkerboard_packet.push(0xff);
    assert_eq!(
        stripe_upload_work(&checkerboard_packet),
        StripeUploadWork {
            packets: 1,
            transfer_bytes: 0x800,
            fixed_source_packets: 0,
            vertical_packets: 0,
        }
    );
}

#[test]
fn published_nmi_active_display_overrun_is_repeatable_and_advances_live_state() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.nmi_forced_blank_scanlines_pending = 1;
    state.capture_display_snapshot();

    assert_eq!(state.nmi_forced_blank_scanlines_pending, 0);
    assert_eq!(state.ppu.forced_blank_scanlines, 0);

    let first = state.with_display_snapshot(ZeldaState::ppu_scanline_windows);
    assert!(first[0].8);
    assert!(!first[1].8);

    let second = state.with_display_snapshot(ZeldaState::ppu_scanline_windows);
    assert_eq!(second.as_ref(), first.as_ref());
    assert_eq!(state.nmi_forced_blank_scanlines_pending, 0);
    assert_eq!(state.ppu.forced_blank_scanlines, 0);
}

#[test]
fn c_hud_tilemap_request_is_consumed_by_the_next_nmi_and_blanks_its_top_scanline() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();

    // zelda3/src/hud.c Hud_ClearTileMap authors subroutine 1 and target page
    // $22. zelda3/src/nmi.c Interrupt_NMI consumes that request in
    // NMI_DoUpdates alongside the already-pending flag_update_hud_in_nmi DMA,
    // before its unconditional WritePpuRegisters call. The original-ROM/Snes9x
    // standard-route receipt enters this NMI at V=225, restores INIDISP at
    // V=1 H=870, and therefore presents output row zero as forced blank while
    // row one is visible.
    state.increment_hud_update_flag();
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(0x22);
    state.interrupt_nmi_for_active_scanout(0, None, false);

    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    let scanlines = state.with_display_snapshot(ZeldaState::ppu_scanline_windows);
    assert!(scanlines[0].8);
    assert!(!scanlines[1].8);
}

#[test]
fn c_full_tilemap_nmi_without_the_hud_dma_finishes_before_visible_scanout() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(1);
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();

    // C Hud_Init authors another subroutine-1 upload but does not set
    // flag_update_hud_in_nmi. The original-ROM/Snes9x receipt for that exact
    // next request enters at V=225 and resumes at V=261, before visible row
    // zero, so the full-tilemap transfer alone must not publish a blank prefix.
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(0x22);
    state.interrupt_nmi_for_active_scanout(0, None, false);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    let scanlines = state.with_display_snapshot(ZeldaState::ppu_scanline_windows);
    assert!(!scanlines[0].8);
}

#[test]
fn active_display_force_blank_edge_is_not_replayed_in_the_following_field() {
    let mut state = ZeldaState::new();
    state.ppu.forced_blank = true;
    state.ppu.forced_blank_from_scanline = Some(1);
    state.ppu.retain_active_display_history = true;

    state.capture_display_snapshot();

    let following = state.display_snapshot.as_deref().unwrap();
    assert!(following.ppu.forced_blank);
    assert_eq!(following.ppu.forced_blank_from_scanline, None);
    assert!(!following.ppu.retain_active_display_history);
    assert_eq!(state.ppu.forced_blank_from_scanline, Some(1));
    assert!(state.ppu.retain_active_display_history);
}
