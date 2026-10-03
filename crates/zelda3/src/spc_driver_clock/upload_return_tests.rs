use super::*;

fn cpu_port_clock_fixture(program: &[u8]) -> AbsoluteDspEventClock {
    let mut clock = clock_at_first_host_boundary();
    clock.absolute_apu_cycle = 0;
    clock.host_frame_index = 0;
    clock.apu.cycles = 0;
    clock.apu.cpu_cycles_left = 0;
    clock.apu.debug_dsp_write_trace = Some(Vec::new());
    clock.apu.ram[0x2000..0x2000 + program.len()].copy_from_slice(program);
    clock.cpu_port_clock = Some((Snes9xApuClockCheckpoint::default(), 0));
    clock
}

#[test]
fn cpu_port_reads_retain_partial_stores_and_credit_across_output_windows() {
    let mut clock = cpu_port_clock_fixture(&[0xda, 0xf4, 0x2f, 0xfe]);
    clock.apu.spc.a = 0x37;
    clock.apu.spc.y = 0x91;
    clock.host_transport.requested_ambient = 0x91;
    clock.apu.in_ports[1] = 0x77;
    let mut uninterrupted = ApuHostPortTiming::new(
        ApuHostPortProbe::new(clock.apu.clone()).unwrap(), Default::default(),
    ).unwrap();
    // MOVW has published low, but not high, at this CPU deadline.
    assert_eq!(clock.cpu_port_at(84, 1, None).unwrap(), 0);
    assert_eq!(clock.host_acknowledgements(), [0x37, 0, 0, 0]);
    assert!(clock.cpu_port_access_pending());
    assert_eq!(uninterrupted.read_cpu_port_at(84, 1).unwrap(), 0);
    assert_eq!(clock.cpu_port_at(84, 2, Some(0x66)).unwrap(), 0x66);
    uninterrupted.write_cpu_port_at(84, 2, 0x66).unwrap();
    // The later output window finishes the retained instruction once, then
    // owns further execution. It must not replay the CPU store nominally.
    clock.advance(Default::default(), 1, 0);
    assert!(!clock.cpu_port_access_pending());
    assert_eq!(clock.apu.in_ports[2], 0x66);
    // The CPU saw no ambient acknowledgement. Completing MOVW publishes
    // the matching high byte, but must not retroactively clear input port1.
    assert_eq!(clock.apu.in_ports[1], 0x77);
    assert!(!clock.pending_timed_main_cpu_port_writes.iter().any(|&(_, p, _)| p == 1));
    assert!(!clock.pending_timed_main_cpu_port_writes.iter().any(|&(_, p, _)| p == 2));
    assert_eq!(clock.cpu_port_at(1260, 1, None).unwrap(), 0x91);
    assert_eq!(uninterrupted.read_cpu_port_at(1260, 1).unwrap(), 0x91);
    let resumed = clock.cpu_ports.as_ref().unwrap();
    assert_eq!(resumed.clock().checkpoint(), uninterrupted.clock().checkpoint());
    assert_eq!(resumed.machine().save_c_saveload_prefix(), uninterrupted.machine().save_c_saveload_prefix());
    assert_eq!(bincode::serialize(&resumed.machine().spc).unwrap(),
        bincode::serialize(&uninterrupted.machine().spc).unwrap());
}

#[test]
fn cpu_port_access_preserves_dsp_writes_until_their_output_window() {
    let mut clock = cpu_port_clock_fixture(&[0x8f, 0x0c, 0xf2, 0x8f, 0x7f, 0xf3, 0x2f, 0xfe]);
    clock.cpu_port_at(252, 0, None).unwrap();
    let trace = clock.cpu_ports.as_ref().unwrap().machine().debug_dsp_write_trace.as_ref().unwrap();
    assert_eq!(trace.len(), 1);
    let (cycle, address, value) = trace[0];
    let window = clock.advance(Default::default(), 1, 0);
    assert_eq!(window.writes, vec![DspWriteEvent::new(address, value, (cycle / 32) as i32, (cycle % 32) as u8)]);
    assert!(clock.advance(Default::default(), 1, 0).writes.is_empty());
}

#[test]
fn timed_music_store_preserves_the_later_nominal_port_deadlines() {
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    let mut nominal = clock.clone();
    clock.cpu_port_at(21, 0, Some(0)).unwrap();
    clock.advance(Default::default(), 1, 0);
    nominal.advance(Default::default(), 1, 0);
    // Both NMIs took the music-clear branch. Its earlier publication must
    // neither replay port0 nor shorten the following ambient/effect paths.
    nominal.pending_timed_main_cpu_port_writes.retain(|&(_, port, _)| port != 0);
    assert_eq!(clock.pending_timed_main_cpu_port_writes,
        nominal.pending_timed_main_cpu_port_writes);
    assert_eq!(clock.pending_timed_main_cpu_port_writes.len(), 3);
}

#[test]
fn cpu_port_access_rejects_ahead_execution_and_due_unowned_writes() {
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    clock.apu.run_cycle_sequenced_instruction_without_dsp();
    let before = bincode::serialize(&clock).unwrap();
    assert!(clock.cpu_port_at(1, 0, None).unwrap_err().contains("beyond the CPU port deadline"));
    assert_eq!(bincode::serialize(&clock).unwrap(), before);
    clock.pending_timed_main_cpu_port_writes.push((4, 0, 0x22));
    assert!(clock.cpu_port_at(105, 0, None).unwrap_err().contains("undelivered host transfer"));
    assert!(!clock.cpu_port_access_pending());
}

#[test]
fn cpu_owned_audio_window_preserves_the_next_port_deadline_and_write_receipts() {
    let mut clock = cpu_port_clock_fixture(&[0x8f, 0x0c, 0xf2, 0x8f, 0x7f, 0xf3, 0x2f, 0xfe]);
    let mut uninterrupted = ApuHostPortTiming::new(
        ApuHostPortProbe::new(clock.apu.clone()).unwrap(), Default::default()).unwrap();
    clock.cpu_port_at(252, 0, None).unwrap();
    uninterrupted.read_cpu_port_at(252, 0).unwrap();
    let window = clock.advance_at_cpu_boundary(Default::default(), 1, 0, 672).unwrap();
    uninterrupted.sync_to(672).unwrap();
    assert_eq!(window.writes.len(), 1);
    assert_eq!(window.writes[0].addr, 0x0c);
    assert!(clock.cpu_port_access_pending());
    assert_eq!(clock.cpu_port_at(693, 0, None).unwrap(),
        uninterrupted.read_cpu_port_at(693, 0).unwrap());
    let owner = clock.cpu_ports.as_ref().unwrap();
    assert_eq!(owner.clock().checkpoint(), uninterrupted.clock().checkpoint());
    assert_eq!(owner.machine().save_c_saveload_prefix(), uninterrupted.machine().save_c_saveload_prefix());
    assert_eq!(bincode::serialize(&owner.machine().spc).unwrap(),
        bincode::serialize(&uninterrupted.machine().spc).unwrap());
    assert!(clock.advance_at_cpu_boundary(Default::default(), 1, 0, 1344).unwrap().writes.is_empty());
}

#[test]
fn cpu_owned_output_availability_matches_every_power_on_dsp_phase() {
    let mut dsp = snes::Snes9xDspPhaseMachine::power_on();
    let mut ram = [0; 65536];
    let mut samples = 0;
    for clocks in 1..=128 {
        samples += u64::from(dsp.run_clock(&mut ram).is_some());
        assert_eq!(samples, (clocks + DSP_CLOCKS_AFTER_OUTPUT) / APU_CYCLES_PER_DSP_SAMPLE);
    }
}

fn clock_at_first_host_boundary() -> AbsoluteDspEventClock {
    let start = snes_master_clock_to_apu_cycle(225 * SNES_MASTER_CLOCKS_PER_SCANLINE);
    let mut apu = ApuState::new();
    apu.reset();
    apu.rom_readable = false;
    apu.spc.pc = 0x2000;
    apu.cycles = start as u32;
    AbsoluteDspEventClock {
        cpu_ports: None,
        cpu_port_clock: None,
        cpu_port_actions: [None; 4],
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
fn future_cpu_port_write_survives_until_its_scheduled_bus_access() {
    let mut clock = clock_at_first_host_boundary();
    // SPC MOV A,$f4; MOV $f5,A; BRA: echo CPU port0 to output port1.
    // The Snes9x APUI/SMP port contract keeps the input latch unchanged
    // until the CPU write and then exposes that write to subsequent reads.
    // The audio window boundary itself has no port-register side effect.
    clock.apu.ram[0x2000..0x2006].copy_from_slice(&[0xe4, 0xf4, 0xc4, 0xf5, 0x2f, 0xfa]);
    clock.begin_song_bank_transfer_at(0, &[0, 0],
        Some(snes::CpuRasterPosition::new(251, 900)));
    let mut uninterrupted = clock.clone();

    clock.advance(EngineAudioCommandBatch::default(), 1, 0);
    assert_eq!(clock.apu.in_ports[0], 0, "future write must not publish early");
    assert_eq!(clock.apu.out_ports[1], 0);

    clock.advance(EngineAudioCommandBatch::default(), 99, 0);
    uninterrupted.advance(EngineAudioCommandBatch::default(), 100, 0);
    assert_eq!(clock.apu.in_ports[0], 0xff, "window end must not discard the CPU command");
    assert_eq!(clock.apu.out_ports[1], 0xff, "the SPC must observe the real input latch");
    assert_eq!(clock.apu.out_ports, uninterrupted.apu.out_ports);
    assert_eq!(clock.apu.spc.pc, uninterrupted.apu.spc.pc);
    assert_eq!(clock.apu.cycles, uninterrupted.apu.cycles);

    clock.apu.in_ports[0] = 0;
    clock.advance(EngineAudioCommandBatch::default(), 2, 0);
    assert_eq!(clock.apu.out_ports[1], 0, "a retired write must not replay");
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
fn timed_dungeon_request_matches_source_ready_poll_bus_timestamps() {
    // Snes9x comparison54019: FF at V118/C234, low/high reads660/666,
    // then718/724. The 386-clock caller crosses one40-clock refresh.
    let field = CpuFieldTiming::NON_INTERLACE_EVEN;
    let at = |h| field.master_cycles_at(54019, snes::CpuRasterPosition::new(118, h));
    let mut transfer = SongBankHostTransfer::new(1, &[0, 0]);
    transfer.timed_command = true;
    transfer.mark_command_scheduled_at_master_clock(at(234));
    assert_eq!(transfer.next_host_access_master_clock, Some(at(660)));
    let mut input = [0; 6];
    assert!(!transfer.perform_host_access(&mut input, [7, 5, 0, 0]));
    assert_eq!(transfer.next_host_access_master_clock, Some(at(666)));
    assert!(!transfer.perform_host_access(&mut input, [7, 5, 0, 0]));
    assert_eq!(transfer.next_host_access_master_clock, Some(at(718)));
    assert_eq!(input, [0; 6]);
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
    assert_eq!(clock.preview_song_upload_return(),
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
    assert_eq!(clock.preview_song_upload_return(), None);
}

#[test]
fn dungeon_upload_return_is_in_active_scanout_after_the_final_port_clear() {
    let mut clock = clock_at_first_host_boundary();
    let mut transfer = SongBankHostTransfer::new(1, &[0, 0]);
    transfer.command_pending = false;
    transfer.phase = SongBankHostTransferPhase::ClearPort { port: 0 };
    // Source comparison54043: four STZ accesses at V1/C924,954,984,1014.
    // Both bank entries share PLP/RTS, CLI/RTL, then LDA/STA $4200.
    let field_start = snes_frame_start_master_clock(1);
    transfer.next_host_access_master_clock =
        Some(field_start + SNES_MASTER_CLOCKS_PER_SCANLINE + 924);
    clock.song_bank_transfer = Some(transfer);
    assert_eq!(clock.preview_song_upload_return(),
        Some(snes::CpuRasterPosition::new(1, 1188)));
    assert!(clock.completed_song_bank_port_clear_master_clock.is_none());
    assert!(clock.song_bank_transfer.is_some(), "preview must not retire the live receiver");
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
