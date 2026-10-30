use super::*;

#[test]
fn lag_free_bootstrap_uploads_match_first_rom_nmi() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let mut native = ZeldaState::new();
    native.set_assets(&std::fs::read(root.join("zelda3_assets.dat")).unwrap()).unwrap();
    native.set_lag_free(true);
    let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset_with_sram(
        &rom, Some(&native.sram)).unwrap();
    let mut accepted = false;
    let mut first_ppu = None;
    for _ in 0..100 {
        source.run_until_main_loop_return_with_quiescent_step_hook(|step, exec| {
            if matches!(step.accepted_interrupt, Some(snes::SourceCpuAcceptedInterrupt::Nmi { .. })) {
                accepted = true;
            }
            if accepted && step.opcode == 0x40 && first_ppu.is_none() {
                first_ppu = Some(exec.machine().snes().ppu.clone());
            }
            Ok::<(), snes::SourceCpuError>(())
        }).unwrap();
        if first_ppu.is_some() { break; }
    }
    let source = first_ppu.expect("reset reaches its opening NMI");
    let frame = native.frame_ctr_dbg;
    native.zelda_initialization_code();
    assert_eq!(native.frame_ctr_dbg, frame, "bootstrap is not a logical iteration");
    assert_eq!(native.ppu.vram, source.vram, "all reset DMA destinations, including live stack bytes");
    assert_eq!(native.ppu.cgram, source.cgram);
    assert_eq!(native.ppu.oam, source.oam);
    assert!(native.native_startup_cpu.is_none());
    assert!(native.native_bg3_scroll_scanout.is_none());
}

#[test]
#[cfg(feature = "rom-timing")]
fn link_dma_reads_live_wram_in_both_source_banks() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.zelda_initialization_code();
    state.latch_nmi_update();
    assert_eq!(state.ram[0x12], 1);

    // Both startup and later Link DMA must see writes made after the
    // initializer. Bank $10's low mirror and bank $7E own the same bytes.
    for module in [0, 7] {
        state.game_state.frame.main_module = module;
        for address in [0, 0x100, 0x1fc0] {
            state.ram[address + 0x20] = 0x96 ^ module;
            state.copy_asset_bytes_to_vram(0x4000, &[0xee; 0x40], address, 0x40);
            state.copy_ram_bytes_to_vram(0x4100, address, 0x40);
            for word in 0..0x20 {
                let expected = read_le_u16(&state.wram_image(), address + word * 2);
                assert_eq!(state.ppu.vram[0x4000 + word], expected);
                assert_eq!(state.ppu.vram[0x4100 + word], expected);
            }
        }
    }
    // The upper bank still reads ROM, even when low WRAM differs.
    state.copy_asset_bytes_to_vram(0x4200, &[0x34, 0x12, 0x78, 0x56], 0x8002, 2);
    assert_eq!(state.ppu.vram[0x4200], 0x5678);
}

#[test]
fn native_nmi_entry_cost_matches_source_bus_transactions() {
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, Snes, SourcePpuReadState,
    };

    let mut rom = vec![0xea; 0x8000];
    rom[0x7fea..0x7fec].copy_from_slice(&[0xc9, 0x80]);
    // The translated game executes in slow-ROM banks. Interrupt entry is
    // independent of the interrupted routine and its register widths. Check
    // both an ordinary entry and one spanning WRAM refresh: the ledger owns
    // CPU transactions, while the hardware timeline owns the stall.
    for bank in [0x00, 0x06, 0x0e, 0x1d] {
        for (m8, x8) in [(false, false), (false, true), (true, false), (true, true)] {
            for cycle in [100, 520] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.cpu.k = bank;
                machine.cpu.pc = 0x8000;
                machine.cpu.sp = 0x1cf;
                machine.cpu.e = false;
                machine.cpu.mf = m8;
                machine.cpu.xf = x8;
                let timeline = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(100, cycle),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut source = RomCpuTimingProbe::new(
                    machine, timeline, SourcePpuReadState::snes9x_reset(),
                ).unwrap();
                let receipt = source.accept_native_nmi().unwrap();
                assert_eq!(receipt.memory_speed, 8);
                let transactions: Vec<_> = receipt.transactions.iter()
                    .map(|transaction| transaction.duration_master_cycles).collect();
                // MemorySpeed + ONE_CYCLE, PBR/PC/P pushes, vector word.
                assert_eq!(transactions, [14, 8, 16, 8, 16]);
                let cpu_work: u64 = transactions.into_iter().map(u64::from).sum();
                assert_eq!(nmi::NATIVE_NMI_ENTRY_MASTER_CYCLES, cpu_work);
                let elapsed = receipt.ended_at.master_cycles() - receipt.started_at.master_cycles();
                assert_eq!(elapsed, cpu_work + if cycle == 520 { 40 } else { 0 });
            }
        }
    }
}

#[test]
fn force_blank_write_respects_the_scanline_render_event() {
    assert_eq!(force_blank_output_row(49, 320), 48);
    assert_eq!(force_blank_output_row(27, 511), 26);
    assert_eq!(force_blank_output_row(27, 512), 27);
    assert_eq!(force_blank_output_row(1, 0), 0);
    assert_eq!(force_blank_output_row(224, 1000), 224);
    assert_eq!(force_blank_output_row(225, 1000), 0);
}

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
fn nmi_handler_records_its_ledger_and_dma_cost_for_the_resumed_host_budget() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(1);
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();
    assert_eq!(state.last_nmi_handler_master_cycles, None);

    // A full-tilemap upload (subroutine 1, page $22) plus the OAM transfer:
    // the recorded cost carries the handler's ledger charge and at least the
    // DMA bus time of those transfers (0x1000 + 544 bytes at 8 each, plus 26
    // per transfer), and stays well inside one frame.
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(0x22);
    state.interrupt_nmi_for_active_scanout(0, None, false);
    let cost = state
        .last_nmi_handler_master_cycles
        .expect("the NMI handler records its cost");
    assert!(cost > (0x1000 + 544) * 8 + 2 * 26, "nmi cost {cost}");
    assert!(cost < 357_368, "nmi cost {cost}");
    assert!(state.nmi_dma_accounting.is_none());

    // Outside the handler the accounting hook is inert.
    state.account_nmi_dma_transfer(0x200);
    assert_eq!(state.last_nmi_handler_master_cycles, Some(cost));
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
