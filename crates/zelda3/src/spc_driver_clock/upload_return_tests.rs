use super::*;

fn clock_at_first_host_boundary() -> AbsoluteDspEventClock {
    let start = snes_master_clock_to_apu_cycle(225 * SNES_MASTER_CLOCKS_PER_SCANLINE);
    let mut apu = ApuState::new();
    apu.reset();
    apu.rom_readable = false;
    apu.spc.pc = 0x2000;
    apu.cycles = start as u32;
    AbsoluteDspEventClock {
        startup_template: apu.clone(),
        apu,
        absolute_apu_cycle: start,
        apu_cycle_origin: 0,
        next_poll_boundary: DRIVER_POLL_PUSH_Y_PC,
        pending_writes: Vec::new(),
        pending_main_cpu_port_writes: Vec::new(),
        pending_timed_main_cpu_port_writes: Vec::new(),
        pending_upload_return_nmi_master_clock: None,
        song_bank_transfer: None,
        completed_song_bank_id: None,
        completed_song_bank_port_clear_master_clock: None,
        host_frame_index: 1,
        host_transport: HostPortTransport::default(),
    }
}

#[test]
fn native_upload_polls_cannot_observe_future_stores_in_the_same_spc_instruction() {
    let mut clock = clock_at_first_host_boundary();
    // MOVW $f4,YA publishes the receiver-ready bytes separately, then BRA
    // keeps the SPC idle. The CPU's first low-port read precedes those stores
    // and must fail, even though both bytes exist when MOVW returns.
    clock.apu.ram[0x2000..0x2004].copy_from_slice(&[0xda, 0xf4, 0x2f, 0xfe]);
    clock.apu.spc.a = 0xaa;
    clock.apu.spc.y = 0xbb;
    clock.apu.in_ports[2] = 0x77;
    clock.begin_song_bank_transfer(1, &[0, 0, 0, 8], true);
    clock.pending_main_cpu_port_writes.clear();
    let start = clock.absolute_apu_cycle;
    let transfer = clock.song_bank_transfer.as_mut().unwrap();
    transfer.command_pending = false;
    transfer.next_host_access_master_clock = Some(apu_cycle_to_snes_master_clock(start + 1));
    clock.advance(EngineAudioCommandBatch::default(), 0, 0);
    assert_eq!(clock.apu.out_ports[..2], [0xaa, 0xbb]);
    assert_eq!(clock.apu.in_ports[2], 0x77,
        "a prematurely successful ready poll published the block header early");
}

#[test]
fn upload_command_uses_its_cpu_position_instead_of_the_audio_window_end() {
    for (position, field) in [
        (snes::CpuRasterPosition::new(31, 900), 1),
        (snes::CpuRasterPosition::new(251, 900), 0),
    ] {
        let mut clock = clock_at_first_host_boundary();
        let ports = clock.apu.in_ports;
        clock.begin_song_bank_transfer_at(0, &[0, 0], Some(position));
        let master = CpuFieldTiming::NON_INTERLACE_EVEN.master_cycles_at(field, position);
        assert_eq!(clock.pending_timed_main_cpu_port_writes,
            [(snes_master_clock_to_apu_cycle(master), 0, 0xff)]);
        assert!(clock.pending_main_cpu_port_writes.is_empty());
        assert_eq!(clock.apu.in_ports, ports, "scheduling is not a port publication");
        assert_eq!(clock.song_bank_transfer.as_ref().unwrap().next_host_access_master_clock,
            Some(advance_snes_cpu_master_clock(master,
                TIMED_OVERWORLD_COMMAND_TO_FIRST_READY_READ_MASTER_CLOCKS)));
    }
}

#[test]
fn timed_overworld_request_matches_source_ready_poll_bus_timestamps() {
    // Cold Snes9x run37662, APUI bus accesses: FF at V31/C832,
    // CMP low at1196, high at1202, failed-pair next low at1254.
    // These are bus timestamps, not the following instruction's PC trace.
    let field = CpuFieldTiming::NON_INTERLACE_EVEN;
    let at = |h| field.master_cycles_at(37662, snes::CpuRasterPosition::new(31, h));
    let mut transfer = SongBankHostTransfer::new(0, &[0, 0]);
    transfer.timed_command = true;
    transfer.mark_command_scheduled_at_master_clock(at(832));
    assert_eq!(transfer.next_host_access_master_clock, Some(at(1196)));
    let mut input = [0; 6];
    assert!(!transfer.perform_host_access(&mut input, [20, 5, 0, 0]));
    assert_eq!(transfer.next_host_access_master_clock, Some(at(1202)));
    assert!(!transfer.perform_host_access(&mut input, [20, 5, 0, 0]));
    assert_eq!(transfer.next_host_access_master_clock, Some(at(1254)));
    assert_eq!(input, [0; 6], "polling must not publish a host write");
}

#[test]
fn upload_return_forecast_preserves_the_live_receiver_and_source_return_phase() {
    let mut clock = clock_at_first_host_boundary();
    let mut transfer = SongBankHostTransfer::new(0, &[0, 0]);
    transfer.command_pending = false;
    transfer.phase = SongBankHostTransferPhase::ClearPort { port: 0 };
    // Snes9x run37686 APUI writes occur at V251/C470,500,530,600.
    // Refresh separates the last two accesses. The PLP/RTS and
    // CLI/RTL/LDA/STA suffix restores $4200 at bus timestamp C774.
    transfer.next_host_access_master_clock =
        Some(251 * SNES_MASTER_CLOCKS_PER_SCANLINE + 470);
    clock.song_bank_transfer = Some(transfer);
    let before = clock.clone();
    assert_eq!(clock.preview_overworld_song_upload_return(),
        Some(snes::CpuRasterPosition::new(251, 774)));
    assert_eq!(clock.absolute_apu_cycle, before.absolute_apu_cycle);
    assert_eq!(clock.apu.in_ports, before.apu.in_ports);
    assert!(clock.completed_song_bank_port_clear_master_clock.is_none());
    assert!(matches!(clock.song_bank_transfer.as_ref().unwrap().phase,
        SongBankHostTransferPhase::ClearPort { port: 0 }));
    clock.advance(EngineAudioCommandBatch::default(), 534, 0);
    assert_eq!(clock.completed_song_bank_port_clear_master_clock,
        Some(251 * SNES_MASTER_CLOCKS_PER_SCANLINE + 600));
    assert!(clock.song_bank_transfer.is_none());
    assert_eq!(clock.take_completed_song_bank_id(), Some(0));
    assert_eq!(clock.take_completed_song_bank_id(), None);
}

#[test]
fn upload_return_after_the_next_nmi_cannot_retire_this_host() {
    let mut clock = clock_at_first_host_boundary();
    clock.completed_song_bank_port_clear_master_clock =
        Some(snes_frame_start_master_clock(1) + 226 * SNES_MASTER_CLOCKS_PER_SCANLINE);
    assert_eq!(clock.preview_overworld_song_upload_return(), None);
}

#[test]
fn upload_return_nmi_republishes_ports_at_source_bus_positions() {
    let mut clock = clock_at_first_host_boundary();
    let mut transfer = SongBankHostTransfer::new(0, &[0, 0]);
    transfer.command_pending = false;
    transfer.timed_command = true;
    transfer.phase = SongBankHostTransferPhase::ClearPort { port: 0 };
    transfer.next_host_access_master_clock = Some(251 * SNES_MASTER_CLOCKS_PER_SCANLINE + 470);
    clock.song_bank_transfer = Some(transfer);
    clock.host_transport.requested_music = 0xf1;
    clock.host_transport.requested_ambient = 5;
    clock.host_transport.ambient_input = 5;
    clock.queue_upload_return_nmi(snes::CpuRasterPosition::new(251, 774), [0, 5, 0, 0], [0xf1, 5]);
    let entry = clock.pending_upload_return_nmi_master_clock.unwrap().0;
    assert_eq!(entry, 251 * SNES_MASTER_CLOCKS_PER_SCANLINE + 884);
    let writes = HostPortWrites { writes: [None, Some(5), Some(0), Some(0)] };
    let targets = host_port_target_cycles_at_nmi_entry(entry, writes);
    // Cold Snes9x37686: $4200 bus774, NMI acceptance822, vector884;
    // APUI01/02/03 stores follow at V252/C80,174,236. The source's
    // ambient5 store must survive the upload's earlier four port clears.
    for (port, h) in [(1, 80), (2, 174), (3, 236)] {
        assert_eq!(targets[port], snes_master_clock_to_apu_cycle(
            252 * SNES_MASTER_CLOCKS_PER_SCANLINE + h));
    }
    clock.advance(EngineAudioCommandBatch::default(), 534, 0);
    assert!(clock.song_bank_transfer.is_none());
    assert!(clock.pending_upload_return_nmi_master_clock.is_none());
    assert_eq!(&clock.apu.in_ports[..4], &[0, 5, 0, 0]);
}
