use super::*;

#[test]
fn upload_timing_observation_preserves_protocol_and_partitions_every_gap() {
    let mut observed = SongBankHostTransfer::new(0, &[2, 0, 0, 0x20, 0x11, 0x22, 0, 0]);
    observed.timing_audit = Some(Default::default());
    observed.mark_command_scheduled_at_master_clock(0);
    let mut plain = observed.clone();
    plain.timing_audit = None;
    let mut observed_ports = [0; 6];
    let mut plain_ports = [0; 6];
    let mut retries = 0;
    loop {
        let ack = match observed.phase {
            SongBankHostTransferPhase::AwaitReceiverReadyLow
            | SongBankHostTransferPhase::AwaitReceiverReadyHigh { .. } => [0xaa, 0xbb, 0, 0],
            SongBankHostTransferPhase::AwaitBlockAck { token, .. } => {
                retries += 1;
                [if retries % 3 == 0 { token } else { token ^ 0xff }, 0, 0, 0]
            }
            SongBankHostTransferPhase::AwaitByteAck { counter, .. } => {
                retries += 1;
                [if retries % 3 == 0 { counter } else { counter ^ 0xff }, 0, 0, 0]
            }
            _ => [0; 4],
        };
        let done = observed.perform_host_access(&mut observed_ports, ack);
        assert_eq!(plain.perform_host_access(&mut plain_ports, ack), done);
        assert_eq!(observed_ports, plain_ports);
        assert_eq!(observed.next_host_access_master_clock, plain.next_host_access_master_clock);
        assert_eq!(bincode::serialize(&observed).unwrap(), bincode::serialize(&plain).unwrap());
        if done { break; }
    }
    let audit = observed.timing_audit.unwrap();
    assert_eq!(audit.clocks.iter().sum::<u64>(),
        observed.completed_port_clear_master_clock.unwrap() - audit.first_read);
    assert_eq!(audit.accesses[7], 2);
    assert_eq!(audit.accesses[10], 4);
    assert_eq!(audit.cpu.iter().sum::<u64>(),
        6 + 332 + 2 * (6 + 106 + 74 + 30) + 4 * 52 + 210 + 62
        + 2 * 6 + 236 + 82 + 4 * 52 + 82 + 304 + 90);
}

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
fn bootstrap_nmi_cannot_suppress_the_first_lagless_effect() {
    let mut clock = cpu_port_clock_fixture(&[0xe4, 0xf7, 0x2f, 0xfc]);
    clock.apu.debug_spc_instruction_trace = Some(Vec::new());
    // The opening NMI has already stored zero to APUI03. Its completed
    // transaction must not consume the next NMI's independently queued SFX.
    let bootstrap_store = advance_snes_cpu_master_clock(lagless_nmi_vector(0), 792);
    clock.cpu_port_at(bootstrap_store, 3, Some(0)).unwrap();
    let before = clock.cpu_ports.as_ref().unwrap().clone();
    clock.begin_lagless_output_after_bootstrap(bootstrap_store);
    let after = clock.cpu_ports.as_ref().unwrap();
    assert_eq!(before.clock().checkpoint(), after.clock().checkpoint());
    assert_eq!(bincode::serialize(before.machine()).unwrap(),
        bincode::serialize(after.machine()).unwrap());
    clock.advance_lagless(EngineAudioCommandBatch::from_legacy_ports([0, 0, 0, 10]), 534, 0);
    assert_eq!(clock.apu.in_ports[3], 10);
    assert!(clock.apu.debug_spc_instruction_trace.as_ref().unwrap().iter()
        .any(|step| step.pc == 0x2002 && step.a == 10),
        "the executing SPC must read the new effect, not the bootstrap clear");
    clock.advance_lagless(Default::default(), 534, 0);
    assert_eq!(clock.apu.in_ports[3], 0, "the following NMI still clears the one-shot");
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
        pending_lagless_nmis: Vec::new(),
        pending_lagless_port_writes: Vec::new(),
        pending_lagless_nmi_latches: None,
        pending_upload_return_nmi_master_clock: None,
        song_bank_transfer: None,
        completed_song_bank_id: None,
        completed_song_bank_port_clear_master_clock: None,
        host_frame_index: 1,
        host_transport: HostPortTransport::default(),
        observation_silent: false,
    }
}

#[test]
fn sound_fix1_accepted_pond_music_reaches_port_after_clear() {
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    let song = EngineAudioCommandBatch::from_legacy_ports([0x1b,0,0,0]);
    clock.advance_lagless(song, 534, 0);
    assert_eq!(clock.apu.in_ports[0], 0x1b);
    clock.apu.out_ports[0] = 0x1b;
    clock.advance_lagless(song, 534, 0);
    assert_eq!(clock.apu.in_ports[0], 0);
    clock.rearm_accepted_music_request();
    clock.advance_lagless(song, 534, 0);
    assert_eq!(clock.apu.in_ports[0], 0x1b);
    clock.advance_lagless(song, 534, 0);
    assert_eq!(clock.apu.in_ports[0], 0);
}

#[test]
fn sound_fix1_first_upload_return_publishes_ambient_after_terminal_clears() {
    // Iteration 48556: a closing NMI was sampled while the output callback
    // still owned the receiver's final clear sequence. It must run after
    // $00:88FC / $02:8566, even though nominal NMI transport is suppressed.
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    let mut transfer = SongBankHostTransfer::new(0, &[0,0]);
    transfer.command_pending = false;
    transfer.phase = SongBankHostTransferPhase::ClearPort { port: 0 };
    transfer.next_host_access_master_clock = Some(1000);
    clock.song_bank_transfer = Some(transfer);
    clock.apu.in_ports[..4].copy_from_slice(&[0xff,5,0,0x42]);
    clock.queue_upload_return_nmi_at_master(1300, [0,5,0,0x42], [0xf1,5]);
    clock.advance_lagless(EngineAudioCommandBatch::from_legacy_ports([0xf1,5,0,0x42]),64,0);
    assert!(clock.song_bank_transfer.is_none());
    assert!(clock.pending_upload_return_nmi_master_clock.is_none());
    assert_eq!(&clock.apu.in_ports[..4], &[0,5,0,0x42]);
    clock.apu.out_ports[1] = 5;
    // $8110 clears the acknowledged ambient at the next physical NMI,
    // independently of the caller's short sample window.
    let clear = host_port_target_cycles_at_nmi_entry(lagless_nmi_vector(1),
        HostPortWrites { writes: [None, Some(0), Some(0), Some(0)] })[3];
    let samples = (clear - clock.absolute_apu_cycle).div_ceil(32) as u32 + 1;
    clock.advance_lagless(EngineAudioCommandBatch::from_legacy_ports([0xf1,0,0,0]),samples,0);
    assert_eq!(&clock.apu.in_ports[1..4], &[0,0,0]);
}

#[test]
fn lagless_nmi_anchor_keeps_hardware_field_and_handler_work() {
    for host in [1, 302, 146541, 251820, 1_573_135] {
        let vector = snes_frame_start_master_clock(host) + 225 * 1364 + 12 + 62;
        assert_eq!(lagless_nmi_vector(host), vector);
        let writes = HostPortWrites { writes: [None, Some(0), Some(0), Some(10)] };
        let targets = host_port_target_cycles_at_nmi_entry(vector, writes);
        assert!(targets[1] < targets[2] && targets[2] < targets[3]);
    }
    // Source field81 $2143 bus start is 29,254,448. Eligibility omits
    // forty clocks of the interrupted instruction; never fit them into it.
    let vector = lagless_nmi_vector(81);
    assert_eq!(advance_snes_cpu_master_clock(vector, 746), 29_254_408);
}

#[test]
fn lagless_acknowledgement_waits_for_the_anchored_receiver_read() {
    let mut clock = cpu_port_clock_fixture(&[0x8f, 5, 0xf5, 0x2f, 0xfe]);
    clock.host_frame_index = 1;
    clock.host_transport.requested_ambient = 5;
    clock.capture_lagless_nmi_latches([0; 4], [0, 5]);
    // No music store: the ambient LDA $2141 is 522 executed clocks
    // after vector entry, including any crossed hardware refresh.
    clock.apu.out_ports[0] = 1;
    let read = snes_master_clock_to_apu_cycle(
        advance_snes_cpu_master_clock(lagless_nmi_vector(1), 522));
    clock.absolute_apu_cycle = read - 8;
    clock.apu.cycles = (read - 8) as u32;
    clock.apu.in_ports[1] = 5;
    assert_eq!(clock.apu.out_ports[1], 0);
    clock.advance_lagless(EngineAudioCommandBatch::from_legacy_ports([0,5,0,0]), 8, 0);
    assert_eq!(clock.apu.out_ports[1], 5, "executed MOV $f5,#5 supplies the acknowledgement");
    assert_eq!(clock.apu.in_ports[1], 0, "$8108 reads 5 and $8110 clears APUI01");
}

#[test]
fn lagless_deadlines_before_origin_retire_in_order_and_far_deadlines_carry() {
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    clock.apu_cycle_origin = 1 << 30;
    let origin = clock.apu_cycle_origin;
    clock.pending_lagless_port_writes = vec![(origin - 2, 1, 0),
        (origin - 4, 1, 5), (origin + u64::from(u32::MAX) + 1, 3, 10)];
    clock.schedule_lagless_port_writes();
    assert_eq!(clock.apu.in_ports[1], 0, "the earlier 5 precedes the source clear");
    assert_eq!(clock.pending_lagless_port_writes,
        [(origin + u64::from(u32::MAX) + 1, 3, 10)]);
}

#[test]
fn lagless_new_music_store_keeps_the_rom_prefix_before_refresh() {
    // Fresh-ledger source witness: iteration194, field391, $8034 accepts at
    // V225/H12 = 140037020. Native entry62 plus the executed $80C9..$80F3
    // prefix444 places STA $2140 at 140037526 (H518), before H534 refresh.
    // Source synchronized SPC=6685098. Independently derive the queue's
    // expected deadline from the source CPU store, rather than fitting it
    // to the synchronized receiver observation.
    let mut nmi = LaglessNmi {
        host: 391, vector: lagless_nmi_vector(391), ports: [1,0,0,0],
        last: Some([0;2]), handled: [None;4], music: None, music_work: 450,
    };
    let mut events = Vec::new();
    nmi.advance(snes_master_clock_to_apu_cycle(nmi.vector), [0;4],
        &mut HostPortTransport::default(), &mut events, true);
    assert_eq!(events[0], (snes_master_clock_to_apu_cycle(140_037_526), 0, 1));
}

#[test]
fn nmi_reads_and_stores_charge_only_crossed_refresh_events() {
    let line = snes_frame_start_master_clock(391) + 225 * 1364;
    // The v2 phase carries across odd short fields; field391/V225 is H538.
    assert_eq!(next_wram_refresh_master_clock(line), line + 538);
    let music = HostPortWrites { writes: [Some(1), Some(0), Some(0), Some(0)] };
    let clear = HostPortWrites { writes: [Some(0); 4] };
    for h in [74, 84, 100, 600, 1300] {
        let vector = line + h;
        for writes in [music, clear] {
            let work = host_port_cpu_work(writes);
            let targets = host_port_target_cycles_at_nmi_entry(vector, writes);
            for p in 0..4 {
                // Count physical refresh events independently of the store
                // helper, including a possible next-line refresh.
                let mut end = vector + work[p];
                let mut refresh = next_wram_refresh_master_clock(vector);
                while refresh < end {
                    end += SNES_WRAM_REFRESH_STALL_MASTER_CLOCKS;
                    refresh = next_wram_refresh_master_clock(refresh + 40);
                }
                assert_eq!(targets[p], snes_master_clock_to_apu_cycle(end));
            }
        }
        let nmi = LaglessNmi { host: 391, vector, ports: [0;4],
            last: Some([0;2]), handled: [None;4], music: None, music_work: 450 };
        let expected = vector + 390 + if h < 538 && h + 390 > 538 { 40 } else { 0 };
        assert_eq!(nmi.next_read_master(), expected);
    }
    assert_eq!(host_port_target_cycles_at_nmi_entry(line + 74, music)[0],
        snes_master_clock_to_apu_cycle(line + 518));
    assert_eq!(host_port_target_cycles_at_nmi_entry(line + 74, clear)[0],
        snes_master_clock_to_apu_cycle(line + 582));
}

#[test]
fn lagless_stores_are_visible_in_cycle_order_against_spc_port_reads() {
    // Read the same port repeatedly across its deadline. These instructions
    // have bus reads inside the instruction, not at its return boundary.
    for relative in [22, 23, 24] {
        let mut clock = cpu_port_clock_fixture(&[0xe4, 0xf4, 0x2f, 0xfc]);
        clock.apu.debug_spc_instruction_trace = Some(Vec::new());
        clock.apu.schedule_input_port_event(relative, 0, 0x51);
        for _ in 0..16 { clock.apu.run_cycle_sequenced_instruction_without_dsp(); }
        let reads: Vec<_> = clock.apu.debug_spc_instruction_trace.as_ref().unwrap().iter()
            .filter(|e| e.pc == 0x2002).collect();
        assert!(reads.iter().any(|e| e.a == 0));
        assert!(reads.iter().any(|e| e.a == 0x51));
        for read in reads {
            assert_eq!(read.a, if read.cycle - 1 >= relative { 0x51 } else { 0 });
        }
    }
}

#[test]
fn lagless_nmi_stores_survive_output_boundary_and_cycle_rebase() {
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    clock.apu.in_ports[..4].fill(0x55);
    // A one-pair test callback ends before the last stores. The scheduler's
    // lookahead may execute them, but neither the next callback nor rebasing
    // may drop or duplicate them.
    clock.host_frame_index = snes_frame_for_master_clock(
        apu_cycle_to_snes_master_clock((1 << 30) + 100_000));
    let first_store = host_port_target_cycles_at_nmi_entry(
        lagless_nmi_vector(clock.host_frame_index), HostPortWrites { writes: [Some(0); 4] })[0];
    clock.absolute_apu_cycle = first_store - 50;
    clock.apu.cycles = (clock.absolute_apu_cycle + 20) as u32;
    clock.apu.debug_spc_instruction_trace = Some(Vec::new());
    clock.advance_lagless(Default::default(), 1, 0);
    assert!(clock.apu_cycle_origin >= 1 << 30);
    assert_eq!(clock.apu.in_ports[0], 0);
    assert_eq!(&clock.apu.in_ports[1..4], &[0x55; 3]);
    clock.advance_lagless(Default::default(), 1, 0);
    assert_eq!(&clock.apu.in_ports[..4], &[0; 4]);
    let snapshots = clock.apu.debug_spc_instruction_trace.as_ref().unwrap();
    assert!(snapshots.iter().any(|s| s.input_ports[0] == 0 && s.input_ports[1] == 0x55));
    assert!(snapshots.iter().any(|s| s.input_ports[2] == 0 && s.input_ports[3] == 0x55));
}

#[test]
fn lagless_transport_does_not_change_the_song_bank_owner() {
    // A protocol receiver: wait for $ff, publish ready, wait for the first
    // header token, then echo counters. Exercise payload and terminal clears,
    // not just a stalled ready poll. This is a transport fixture, not a model
    // of the cartridge receiver's instruction workload.
    let mut clock = cpu_port_clock_fixture(&[
        0xe4, 0xf4, 0x68, 0xff, 0xd0, 0xfa,
        0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5,
        0xe4, 0xf4, 0x68, 0xff, 0xf0, 0xfa, 0xc4, 0xf4,
        0xe4, 0xf4, 0xc4, 0xf4, 0x2f, 0xfa,
    ]);
    clock.begin_song_bank_transfer(1, &[2, 0, 0, 0x20, 0x11, 0x22, 0, 0], true);
    let mut baseline = clock.clone();
    for _ in 0..4 {
        clock.advance_lagless(Default::default(), 534, 0);
        baseline.advance(Default::default(), 534, 0);
        assert_eq!(bincode::serialize(&clock).unwrap(), bincode::serialize(&baseline).unwrap());
        if clock.song_bank_transfer.is_none() { break; }
    }
    assert_eq!(clock.completed_song_bank_id, Some(1));
    assert_eq!(&clock.apu.in_ports[..4], &[0; 4]);
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
fn lagless_upload_command_keeps_its_remainder_on_the_output_clock_for_every_bank() {
    for bank in 0..=2 {
        for phase in [0, 1324, 100_003, 1364 * 262 - 1] {
            let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
            clock.host_frame_index = 146541;
            clock.absolute_apu_cycle = 2_504_092_629;
            let master = apu_cycle_to_snes_master_clock(clock.absolute_apu_cycle) + phase;
            let ports = clock.apu.in_ports;
            clock.begin_lagless_song_bank_transfer(bank, &[0, 0], phase);
            assert_eq!(clock.apu.in_ports, ports);
            assert!(clock.pending_main_cpu_port_writes.is_empty());
            assert_eq!(clock.pending_timed_main_cpu_port_writes,
                [(snes_master_clock_to_apu_cycle(master), 0, 0xff)]);
            let caller = if bank == 0 { 364 } else { 386 };
            assert_eq!(clock.song_bank_transfer.as_ref().unwrap().next_host_access_master_clock,
                Some(advance_snes_cpu_master_clock(master, caller)));
        }
    }
}

#[test]
fn lagless_upload_forecast_uses_the_return_suffix_and_does_not_retire_the_receiver() {
    let mut clock = cpu_port_clock_fixture(&[0x2f, 0xfe]);
    let mut transfer = SongBankHostTransfer::new(1, &[0, 0]);
    transfer.command_pending = false;
    transfer.phase = SongBankHostTransferPhase::ClearPort { port: 0 };
    transfer.next_host_access_master_clock = Some(1000);
    clock.song_bank_transfer = Some(transfer);
    let before = bincode::serialize(&clock).unwrap();
    assert_eq!(clock.preview_lagless_song_upload_return(534),
        Some(advance_snes_cpu_master_clock(1090, 174)));
    assert_eq!(bincode::serialize(&clock).unwrap(), before);
    assert!(clock.song_bank_transfer.is_some());
    assert!(clock.completed_song_bank_port_clear_master_clock.is_none());

    // A clear within the output window is insufficient if the real CPU
    // return and $4200 store are still beyond it. Equality belongs next.
    clock.song_bank_transfer = None;
    let end = 534 * APU_CYCLES_PER_DSP_SAMPLE;
    let master_end = apu_cycle_to_snes_master_clock(end);
    clock.completed_song_bank_port_clear_master_clock = Some(master_end);
    assert_eq!(clock.preview_lagless_song_upload_return(534), None);
    clock.completed_song_bank_port_clear_master_clock = Some(master_end - 1000);
    assert!(clock.preview_lagless_song_upload_return(534).is_some());
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

#[test]
fn lagless_store_at_refresh_endpoint_waits_for_hardware_drain() {
    // ROM probe: field391/V225, vector H94, command1/last0.
    // Prefix444 reaches H538 exactly; getset.h drains >= NextEvent.
    let mut nmi = LaglessNmi {
        host: 391, vector: 140_037_102, ports: [1,0,0,0], last: Some([0;2]),
        handled: [None;4], music: None, music_work: 450,
    };
    let mut events = Vec::new();
    nmi.advance(snes_master_clock_to_apu_cycle(nmi.vector), [0;4],
        &mut HostPortTransport::default(), &mut events, true);
    assert_eq!(events[0], (snes_master_clock_to_apu_cycle(140_037_586), 0, 1));
}

#[test]
fn lagless_read_at_refresh_endpoint_waits_for_hardware_drain() {
    // Same ROM prefix, command0: LDA $2140 work390 from vector H148
    // reaches H538 exactly. The preceding access drains refresh first.
    let nmi = LaglessNmi {
        host: 391, vector: 140_037_156, ports: [0;4], last: Some([0;2]),
        handled: [None;4], music: None, music_work: 450,
    };
    assert_eq!(nmi.next_read_master(), 140_037_586);
}

#[test]
fn lagless_accesses_before_at_and_after_refresh_match_executed_rom() {
    // Pinned ROM probe from $00:80C9, field391/V225 (H538 refresh).
    // Each expected bus start is independently executed source evidence;
    // H90/94/98 and H144/148/152 straddle the exact endpoint for STA/LDA.
    // H600 also proves that an already-passed refresh is not charged again.
    for (store, vector, expected) in [
        (true, 140_037_082, 140_037_526),
        (true, 140_037_098, 140_037_542),
        (true, 140_037_102, 140_037_586),
        (true, 140_037_106, 140_037_590),
        (true, 140_037_608, 140_038_052),
        (false, 140_037_082, 140_037_472),
        (false, 140_037_152, 140_037_542),
        (false, 140_037_156, 140_037_586),
        (false, 140_037_160, 140_037_590),
        (false, 140_037_608, 140_037_998),
    ] {
        let mut nmi = LaglessNmi {
            host: 391, vector, ports: [u8::from(store), 0, 0, 0],
            last: Some([0; 2]), handled: [None; 4], music: None, music_work: 450,
        };
        if store {
            let mut events = Vec::new();
            nmi.advance(snes_master_clock_to_apu_cycle(vector), [0; 4],
                &mut HostPortTransport::default(), &mut events, true);
            assert_eq!(events[0], (snes_master_clock_to_apu_cycle(expected), 0, 1));
        } else {
            assert_eq!(nmi.next_read_master(), expected);
        }
    }
}

#[test]
fn lagless_handler_paths_match_executed_rom_accesses() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourceCpuBusAccessKind,
        SourcePpuReadState};
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    // All music branches (including repeated and F2 controls), crossed with
    // ambient clear/skip/direct branches. Source execution owns expectations.
    for (music, last_music, echo_music) in [
        (0, 1, 0), (0, 1, 1), (1, 1, 0), (1, 0, 0), (0xf2, 0, 0),
    ] {
        for (ambient, last_ambient, echo_ambient) in [(0, 5, 0), (0, 5, 5), (5, 5, 0)] {
            let ports = [music, ambient, 10, 12];
            let last = [last_music, last_ambient];
            let output = [echo_music, echo_ambient, 0, 0];
            let mut unstalled = Vec::new();
            // H600 is after refresh, and ends before the next refresh. Its
            // source instruction work checks each later access independently.
            for h in [600, 74, 90, 94, 98, 144, 148, 152, 1300] {
                let timeline = CpuMasterTimeline::at_raster(391,
                    CpuRasterPosition::new(225, h), CpuBusWorkload::default(),
                    CpuFieldTiming::NON_INTERLACE_EVEN);
                let vector = timeline.clock_master_cycles();
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.cpu.pc = 0x80c9;
                machine.cpu.sp = 0x1f2;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0x34);
                machine.ram[0x12c..0x130].copy_from_slice(&ports);
                machine.ram[0x133] = last_music;
                machine.ram[0x131] = last_ambient;
                let mut apu = ApuState::new();
                apu.reset();
                apu.cpu_cycles_left = 0;
                apu.spc.pc = 0x2000;
                apu.ram[0x2000..0x2002].copy_from_slice(&[0x2f, 0xfe]);
                apu.out_ports = output;
                let owner = snes::apu::ApuHostPortTiming::new(
                    snes::apu::ApuHostPortProbe::new(apu).unwrap(),
                    Snes9xApuClockCheckpoint::new(vector, 0, 0).unwrap()).unwrap();
                let mut source = RomCpuTimingProbe::new(machine, timeline,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                source.attach_apu_port_owner(owner).unwrap();
                let mut expected_reads = Vec::new();
                let mut expected_stores = Vec::new();
                let mut accesses = Vec::new();
                for _ in 0..100 {
                    for access in source.step().unwrap().accesses {
                        if !(0x2140..=0x2143).contains(&access.address) { continue; }
                        let master = access.timestamp.master_cycles();
                        let port = (access.address - 0x2140) as u8;
                        accesses.push(master);
                        match access.kind {
                            SourceCpuBusAccessKind::Read { width: 1, .. } =>
                                expected_reads.push(master),
                            SourceCpuBusAccessKind::Write { value, width: 1 } =>
                                expected_stores.push((snes_master_clock_to_apu_cycle(master), port, value as u8)),
                            _ => panic!("unexpected audio access"),
                        }
                    }
                    if expected_stores.last().is_some_and(|e| e.1 == 3) { break; }
                }
                assert_eq!(expected_stores.last().unwrap().1, 3);
                if h == 600 {
                    unstalled = accesses.iter().map(|at| at - vector).collect();
                }
                let model: Vec<_> = unstalled.iter().map(|work|
                    nmi_audio_access_master_clock(vector, *work)).collect();
                assert_eq!(model, accesses, "refresh per access: ports={ports:?} last={last:?} output={output:?} h={h}");
                let mut nmi = LaglessNmi { host: 391, vector, ports, last: Some(last),
                    handled: [None; 4], music: None, music_work: 450 };
                let mut reads = Vec::new();
                let mut stores = Vec::new();
                loop {
                    let at = nmi.next_read_master();
                    if ports[usize::from(nmi.music.is_some())] == 0 { reads.push(at); }
                    if nmi.advance(snes_master_clock_to_apu_cycle(at), output,
                        &mut HostPortTransport::default(), &mut stores, true) { break; }
                }
                assert_eq!(reads, expected_reads, "reads: ports={ports:?} h={h}");
                assert_eq!(stores, expected_stores, "stores: ports={ports:?} h={h}");
            }
        }
    }
}
