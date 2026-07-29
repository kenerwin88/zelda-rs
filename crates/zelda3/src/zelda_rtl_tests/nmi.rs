use super::*;

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
