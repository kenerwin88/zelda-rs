use super::*;
use crate::{
    CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, MASTER_CYCLES_PER_SCANLINE,
};

fn seed(program: &[u8], v: u16, h: u16, odd: bool) -> (Snes, CpuMasterTimeline) {
    let mut rom = vec![0xea; 0x8000];
    rom[..program.len()].copy_from_slice(program);
    let mut snes = Snes::new();
    snes.cart.load(CartType::LoRom, &rom, 0x2000);
    snes.cpu.pc = 0x8000;
    snes.cpu.e = false;
    snes.cpu.mf = true;
    snes.cpu.xf = true;
    (
        snes,
        CpuMasterTimeline::at_raster(
            0,
            CpuRasterPosition::new(v, h),
            CpuBusWorkload::default(),
            CpuFieldTiming::non_interlace(odd),
        ),
    )
}

#[test]
fn real_probe_reproduces_source_rng_bus_timestamps_and_result() {
    // Original $0d:ba71..ba7e and source comparison56389 register/RAM inputs.
    let (mut snes, timeline) = seed(
        &[
            0xad, 0x37, 0x21, 0xad, 0x3c, 0x21, 0x65, 0x1a, 0x6d, 0xa1, 0x0f, 0x8d, 0xa1, 0x0f,
        ],
        103,
        1168,
        false,
    );
    snes.cpu.db = 6;
    snes.cpu.a = 0x0401;
    snes.cpu.c = true;
    snes.ram[0x1a] = 173;
    snes.ram[0xfa1] = 90;
    let mut ppu = SourcePpuReadState::snes9x_reset();
    ppu.open_bus1 = 1;
    let mut probe = RomCpuTimingProbe::new(snes, timeline, ppu).unwrap();
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for _ in 0..5 {
        let step = probe.step().unwrap();
        for access in step.accesses {
            let cycle = access.timestamp.master_cycles() - 103 * 1364;
            match access.kind {
                SourceCpuBusAccessKind::Read { value, width: 1 }
                    if (access.address as u16) < 0x8000 =>
                {
                    reads.push((access.address, cycle, value))
                }
                SourceCpuBusAccessKind::Write { value, width: 1 } => {
                    writes.push((access.address, cycle, value))
                }
                _ => {}
            }
        }
    }
    assert_eq!(
        reads,
        vec![
            (0x06_2137, 1192, 1),
            (0x06_213c, 1222, 0x2a),
            (0x1a, 1244, 173),
            (0x06_0fa1, 1276, 90)
        ]
    );
    assert_eq!(writes, vec![(0x06_0fa1, 1308, 0x32)]);
    assert_eq!(probe.snes().ram[0xfa1], 0x32);
    assert_eq!(probe.snes().cpu.a, 0x0432);
    assert!(probe.snes().cpu.c);
    assert_eq!(
        probe.timeline().raster_position(),
        CpuRasterPosition::new(103, 1316)
    );
}

#[test]
fn counter_latch_gate_falling_edge_and_buses_match_source() {
    let (_, mut timeline) = seed(&[], 5, 400, false);
    timeline.begin_synchronous_timeline().unwrap();
    let beam = timeline.synchronous_beam_position().unwrap();
    let mut ppu = SourcePpuReadState {
        wrio: 0,
        open_bus1: 0xa5,
        h_latched: 0x123,
        v_latched: 4,
        ..SourcePpuReadState::snes9x_reset()
    };
    assert_eq!(ppu.read(0x2137, beam), Some(0xa5));
    assert_eq!((ppu.h_latched, ppu.v_latched), (0x123, 4));
    assert_eq!(ppu.read(0x213c, beam), Some(0x23));
    assert_eq!(ppu.read(0x213c, beam), Some(0x23));
    ppu.write_wrio(0xff, beam);
    assert_eq!(ppu.h_latched, 0x123);
    ppu.write_wrio(0x12, beam);
    assert_eq!((ppu.h_latched, ppu.v_latched), (100, 5));
    assert_eq!(ppu.read(0x4213, beam), Some(0x12));
    assert_eq!(ppu.read(0x213f, beam), Some(0x63));
    assert!(!ppu.counter_latched);
    assert!(!ppu.h_read_high && !ppu.v_read_high);
    assert_eq!(ppu.read(0x213f, beam), Some(0x23));
}

#[test]
fn long_dots_and_short_odd_scanline_use_the_owned_field() {
    for (h, long, short) in [
        (1290, 322, 322),
        (1292, 322, 323),
        (1294, 323, 323),
        (1308, 326, 327),
        (1310, 326, 327),
        (1312, 327, 328),
        (1359, 338, 339),
    ] {
        for (v, odd, expected) in [(103, false, long), (240, false, long), (240, true, short)] {
            let (_, mut timeline) = seed(&[], v, h, odd);
            timeline.begin_synchronous_timeline().unwrap();
            let beam = timeline.synchronous_beam_position().unwrap();
            let mut ppu = SourcePpuReadState::snes9x_reset();
            ppu.read(0x2137, beam);
            assert_eq!(ppu.h_latched, expected, "v={v} h={h} odd={odd}");
            assert_eq!(ppu.read(0x213f, beam), Some(if odd { 0xc3 } else { 0x43 }));
        }
    }
}

#[test]
fn pending_hmax_does_not_advance_cpu_visible_counter_before_drain() {
    for (v, h, odd) in [(103, 1360, false), (240, 1356, true), (261, 1360, false)] {
        let (_, mut timeline) = seed(&[], v, h, odd);
        timeline.begin_synchronous_timeline().unwrap();
        timeline.advance_synchronous_pcbase_opcode_fetch(8);
        let beam = timeline.synchronous_beam_position().unwrap();
        assert_eq!(beam.scanline, v);
        assert_eq!(beam.cycles, u32::from(h) + 8);
        assert_eq!(beam.odd_field, odd);
        assert_ne!(timeline.raster_position().coordinates().0, v);
        timeline
            .advance_synchronous_after_semantics_with(0, |_, _| Ok::<_, ()>(0))
            .unwrap();
        let after = timeline.synchronous_beam_position().unwrap();
        assert_eq!(after.scanline, (v + 1) % 262);
        assert_eq!(after.cycles, 4);
        assert_eq!(after.odd_field, odd ^ (v == 261));
    }
}

#[test]
fn unsupported_hardware_poison_never_returns_a_cached_value() {
    let (snes, timeline) = seed(&[0xad, 0x40, 0x21], 103, 1168, false);
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    assert!(matches!(
        probe.step(),
        Err(SourceCpuError::UnsupportedBusMap { address: 0x2140 })
    ));
    assert!(probe.is_poisoned());
    let time = probe.timeline().timestamp();
    assert!(matches!(probe.step(), Err(SourceCpuError::Poisoned)));
    assert_eq!(probe.timeline().timestamp(), time);
}

#[test]
fn fastrom_operand_uses_opcode_memory_speed_before_counter_read() {
    let (mut snes, timeline) = seed(&[0xad, 0x37, 0x21], 103, 1168, false);
    snes.cpu.k = 0x80;
    snes.fast_mem = true;
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    let step = probe.step().unwrap();
    assert_eq!(
        step.transactions
            .iter()
            .map(|t| t.duration_master_cycles)
            .collect::<Vec<_>>(),
        vec![6, 12, 6]
    );
    let read = step.accesses.last().unwrap();
    assert_eq!(read.address, 0x2137);
    assert_eq!(read.timestamp.master_cycles(), 103 * 1364 + 1186);
    assert_eq!(probe.ppu_reads().h_latched, 296);
}

#[test]
#[ignore = "requires the local external Zelda3 ROM; source RNG/Cucco witness"]
fn local_rom_counter_probe_matches_source_cucco_branch() {
    let rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    )
    .unwrap();
    let entry = 0x0d * 0x8000 + 0x3a71;
    assert_eq!(
        &rom[entry..entry + 14],
        &[0xad, 0x37, 0x21, 0xad, 0x3c, 0x21, 0x65, 0x1a, 0x6d, 0xa1, 0x0f, 0x8d, 0xa1, 0x0f]
    );
    for (start, expected_rng, expected_h) in [(1168, 0x32, 52), (1156, 0x2f, 40)] {
        for initial_bus2 in [0, 0xff] {
            let mut snes = Snes::new();
            crate::load_rom(&mut snes, &rom).unwrap();
            snes.cpu.k = 0x0d;
            snes.cpu.pc = 0xba71;
            snes.cpu.db = 6;
            snes.cpu.dp = 0;
            snes.cpu.a = 0x0401;
            snes.cpu.x = 13;
            snes.cpu.y = 6;
            snes.cpu.sp = 0x1e8;
            snes.cpu.c = true;
            snes.cpu.mf = true;
            snes.cpu.xf = true;
            snes.cpu.e = false;
            snes.ram[0x1a] = 173;
            snes.ram[0xfa1] = 90;
            // Source stack at $0d:ba71: RTL returns to $06:a7f9.
            snes.ram[0x1e9..0x1ec].copy_from_slice(&[0xf8, 0xa7, 0x06]);
            let timeline = CpuMasterTimeline::at_raster(
                0,
                CpuRasterPosition::new(103, start),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let ppu = SourcePpuReadState {
                open_bus1: 1,
                open_bus2: initial_bus2,
                ..SourcePpuReadState::snes9x_reset()
            };
            let before_ram = snes.ram.clone();
            let mut probe = RomCpuTimingProbe::new(snes, timeline, ppu).unwrap();
            for _ in 0..9 {
                probe.step().unwrap();
            }
            assert_eq!(probe.snes().ram[0xfa1], expected_rng);
            assert_eq!(probe.snes().ram[0xf], expected_rng);
            assert_eq!(probe.program_address(), 0x06_a7ff);
            assert_eq!(
                probe.timeline().raster_position(),
                CpuRasterPosition::new(104, expected_h)
            );
            assert_eq!(probe.snes().cpu.a, 0x0402);
            assert_eq!(
                (probe.snes().cpu.sp, probe.snes().cpu.x, probe.snes().cpu.y),
                (0x1eb, 13, 6)
            );
            assert!(probe.snes().cpu.c);
            for (address, (&old, &new)) in before_ram.iter().zip(&probe.snes().ram).enumerate() {
                if !matches!(address, 0x0f | 0xfa1) {
                    assert_eq!(new, old, "unexpected RNG/caller write at ${address:05x}");
                }
            }
            eprintln!("ROM counter witness start=V103/C{start} initial_ppu_bus2={initial_bus2:02x} rng={:02x} pc={:06x} end={:?}",
                probe.snes().ram[0xfa1], probe.program_address(), probe.timeline().raster_position());
        }
    }
}

#[test]
fn seed_rejects_pending_dma_and_ambiguous_refresh_ownership() {
    for mutate in [
        (|s: &mut Snes| s.dma.dma_busy = true) as fn(&mut Snes),
        |s| s.dma.dma_timer = 1,
        |s| s.dma.hdma_timer = 1,
        |s| s.dma.channel[0].hdma_active = true,
        |s| s.cpu.nmi_wanted = true,
    ] {
        let (mut snes, timeline) = seed(&[], 103, 100, false);
        mutate(&mut snes);
        assert!(matches!(
            RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()),
            Err(RomCpuTimingProbeSeedError::ActiveHardware)
        ));
    }
    let (snes, timeline) = seed(&[], 103, 540, false);
    assert!(matches!(
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()),
        Err(RomCpuTimingProbeSeedError::Timeline(
            CpuSynchronousTimelineStartError::AmbiguousEventState { .. }
        ))
    ));
}

#[test]
fn active_hdma_stalls_the_source_ordered_cpu_at_the_scanline_event() {
    let (mut snes, _) = seed(&[0xea], 1, 1100, false);
    let channel = &mut snes.dma.channel[7];
    channel.hdma_active = true;
    channel.terminated = false;
    channel.do_transfer = true;
    channel.rep_count = 27;
    channel.indirect = true;
    channel.ind_bank = 0x7e;
    channel.size = 0x1baa;
    channel.b_adr = 0x1e;
    channel.mode = 2;
    channel.from_b = false;
    snes.ram[0x1baa..0x1bac].copy_from_slice(&[0, 255]);
    let timeline = CpuMasterTimeline::at_raster(
        0,
        CpuRasterPosition::new(1, 1100),
        CpuBusWorkload::with_dynamic_hdma(),
        CpuFieldTiming::non_interlace(false),
    );
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    probe.step().unwrap();
    // NOP's 8-clock opcode fetch and 6-clock internal cycle cross H=1106.
    // The source's channel-7, mode-2 transfer at frame 56,458 costs 42
    // master cycles: 8+16+16 bus clocks and two CPU/DMA sync clocks.
    assert_eq!(probe.timeline().raster_position().coordinates(), (1, 1156));
    assert_eq!(probe.snes().dma.channel[7].rep_count, 26);
    assert_eq!(probe.snes().dma.channel[7].size, 0x1bac);
    assert_eq!(probe.snes().dma.hdma_timer, 0);
}

#[test]
fn active_hdma_initializes_the_source_descriptor_at_line_zero() {
    let (mut snes, _) = seed(&[0xea], 0, 14, false);
    let channel = &mut snes.dma.channel[7];
    channel.hdma_active = true;
    channel.indirect = true;
    channel.a_bank = 0x7e;
    channel.a_adr = 0x1ba0;
    snes.ram[0x1ba0..0x1ba3].copy_from_slice(&[27, 0xaa, 0x1b]);
    let timeline = CpuMasterTimeline::at_raster(
        0,
        CpuRasterPosition::new(0, 14),
        CpuBusWorkload::with_dynamic_hdma(),
        CpuFieldTiming::non_interlace(false),
    );
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    probe.step().unwrap();
    assert_eq!(probe.timeline().raster_position().coordinates(), (0, 70));
    let channel = &probe.snes().dma.channel[7];
    assert_eq!(channel.table_adr, 0x1ba3);
    assert_eq!(channel.rep_count, 27);
    assert_eq!(channel.size, 0x1baa);
    assert!(channel.do_transfer);
    assert_eq!(probe.snes().dma.hdma_timer, 0);
}

#[test]
fn source_cpu_configures_hdma_channels_before_the_timeline_owns_them() {
    let program = [
        0xa9, 0x42, 0x8d, 0x70, 0x43, // LDA #$42; STA $4370: mode 2, indirect
        0xa9, 0x1e, 0x8d, 0x71, 0x43, // LDA #$1E; STA $4371: M7B
        0xa9, 0x80, 0x8d, 0x0c, 0x42, // LDA #$80; STA $420C: enable channel 7
    ];
    let (snes, _) = seed(&program, 1, 100, false);
    let timeline = CpuMasterTimeline::at_raster(
        0,
        CpuRasterPosition::new(1, 100),
        CpuBusWorkload::with_dynamic_hdma(),
        CpuFieldTiming::non_interlace(false),
    );
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    for _ in 0..6 {
        probe.step().unwrap();
    }
    let channel = &probe.snes().dma.channel[7];
    assert!(channel.hdma_active && channel.indirect);
    assert_eq!(channel.mode, 2);
    assert_eq!(channel.b_adr, 0x1e);

    let (snes, timeline) = seed(&program[10..], 1, 100, false);
    let mut unowned =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    unowned.step().unwrap();
    assert!(matches!(
        unowned.step(),
        Err(SourceCpuError::UnsupportedBusMap { address: 0x420c })
    ));
    assert!(!unowned.snes().dma.channel[7].hdma_active);
    assert!(unowned.is_poisoned());
}

#[test]
fn auto_read_ports_and_nmi_enable_remain_source_ordered() {
    let (mut snes, timeline) = seed(
        &[
            0xad, 0x18, 0x42, // LDA $4218
            0xad, 0x19, 0x42, // LDA $4219
            0xa9, 0x80, 0x8d, 0x00, 0x42, // enable NMI outside VBlank
        ],
        1,
        100,
        false,
    );
    snes.port_auto_read[0] = 0x1234;
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    probe.step().unwrap();
    assert_eq!(probe.snes().cpu.a as u8, 0x34);
    probe.step().unwrap();
    assert_eq!(probe.snes().cpu.a as u8, 0x12);
    probe.step().unwrap();
    probe.step().unwrap();
    assert!(probe.snes().nmi_enabled);

    let (mut snes, timeline) = seed(&[0xa9, 0x80, 0x8d, 0x00, 0x42], 225, 100, false);
    snes.in_vblank = true;
    snes.in_nmi = true;
    let mut pending =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    pending.step().unwrap();
    assert!(matches!(
        pending.step(),
        Err(SourceCpuError::UnsupportedBusMap { address: 0x4200 })
    ));
    assert!(!pending.snes().nmi_enabled);
}

#[test]
fn counter_read_bus_and_flip_survive_a_field_boundary() {
    let (snes, _) = seed(&[0xad, 0x3c, 0x21], 261, 1340, false);
    let timeline = CpuMasterTimeline::at_raster(
        0,
        CpuRasterPosition::new(261, 1340),
        CpuBusWorkload::default(),
        CpuFieldTiming::non_interlace(false),
    );
    let ppu_reads = SourcePpuReadState {
        open_bus2: 0xeb,
        h_latched: 190,
        h_read_high: true,
        counter_latched: true,
        ..SourcePpuReadState::snes9x_reset()
    };
    let mut probe = RomCpuTimingProbe::new(snes, timeline, ppu_reads).unwrap();
    probe.step().unwrap();
    assert_eq!(probe.timeline().raster_position().coordinates(), (0, 6));
    assert_eq!(probe.snes().cpu.a as u8, 0xea);
    assert!(!probe.ppu_reads().h_read_high);
    assert_eq!(probe.ppu_reads().open_bus2, 0xea);
    assert!(probe.ppu_reads().counter_latched);
}

#[test]
fn handoff_keeps_the_ppu_read_bus_and_timeline_event_cursor_together() {
    let (snes, _) = seed(&[0xad, 0x3c, 0x21, 0xad, 0x3c, 0x21], 261, 1320, false);
    let timeline = CpuMasterTimeline::at_raster(
        0,
        CpuRasterPosition::new(261, 1320),
        CpuBusWorkload::default(),
        CpuFieldTiming::non_interlace(false),
    );
    let ppu_reads = SourcePpuReadState {
        h_latched: 0xeb,
        ..SourcePpuReadState::snes9x_reset()
    };
    let mut first = RomCpuTimingProbe::new(snes, timeline, ppu_reads).unwrap();
    first.step().unwrap();
    assert_eq!(first.snes().cpu.a as u8, 0xeb);
    assert_eq!(
        first.timeline().raster_position().coordinates(),
        (261, 1350)
    );

    let handoff = first.into_handoff().ok().expect("healthy boundary");
    let mut second = RomCpuTimingProbe::from_handoff(handoff);
    second.step().unwrap();
    assert_eq!(second.timeline().raster_position().coordinates(), (0, 16));
    assert_eq!(second.snes().cpu.a as u8, 0xea);
    assert_eq!(second.ppu_reads().open_bus2, 0xea);
    assert!(!second.ppu_reads().h_read_high);
}

#[test]
fn enabled_nmi_is_accepted_only_by_the_external_interrupt_owner() {
    let (mut snes, timeline) = seed(&[0xea], 224, 1350, false);
    snes.nmi_enabled = true;
    snes.cpu.sp = 0x1ff;
    snes.cart.rom[0x7fea..0x7fec].copy_from_slice(&[0xc9, 0x80]);
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    probe.step().unwrap();
    assert_eq!(probe.timeline().raster_position().coordinates(), (225, 0));
    assert!(probe.snes().in_nmi);
    assert!(!probe.snes().cpu.nmi_wanted);
    assert_eq!(probe.program_address(), 0x8001);
    let receipt = probe.accept_native_nmi().unwrap();
    assert_eq!(receipt.interrupted_pc, 0x8001);
    assert_eq!(probe.program_address(), 0x80c9);
}

#[test]
fn direct_operand_cannot_silently_cross_the_pcbase_bank_boundary() {
    let (mut snes, timeline) = seed(&[], 103, 100, false);
    snes.cpu.pc = 0xfffe;
    snes.cart.rom[0x7ffe] = 0xa9;
    snes.cart.rom[0x7fff] = 0x5a;
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    assert!(matches!(
        probe.step(),
        Err(SourceCpuError::UnsupportedBusMap { address: 0xffff })
    ));
    assert_eq!(probe.snes().cpu.a, 0);
    assert!(probe.is_poisoned());
    assert!(probe.into_handoff().is_err());
}

#[test]
fn accepted_nmi_owns_source_stack_vector_and_bus_transactions() {
    for (bank, fast, cycles) in [(0, false, 62), (0x80, true, 60)] {
        let (mut snes, timeline) = seed(&[0xea], 225, 14, false);
        snes.cart.rom[0xc9] = 0x40; // RTI
        snes.cart.rom[0x7fea..0x7fec].copy_from_slice(&[0xc9, 0x80]);
        snes.cpu.k = bank;
        snes.cpu.sp = 0x1ff;
        snes.cpu.a = 0x2468;
        snes.cpu.x = 0x12;
        snes.cpu.y = 0x34;
        snes.cpu.c = true;
        snes.cpu.d = true;
        snes.cpu.i = false;
        snes.fast_mem = fast;
        snes.in_nmi = true;
        let status = snes.cpu.pack_flags();
        let before = snes.ram.clone();
        let mut probe =
            RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
        let nmi = probe.accept_native_nmi().unwrap();
        assert_eq!(
            nmi.ended_at.master_cycles() - nmi.started_at.master_cycles(),
            cycles
        );
        assert_eq!(
            nmi.transactions
                .iter()
                .map(|t| t.duration_master_cycles)
                .collect::<Vec<_>>(),
            vec![if fast { 12 } else { 14 }, 8, 16, 8, 16]
        );
        assert_eq!(
            nmi.accesses.iter().map(|a| a.address).collect::<Vec<_>>(),
            vec![0x1ff, 0x1fd, 0x1fc, 0xffea]
        );
        assert!(nmi
            .accesses
            .iter()
            .all(|a| !matches!(a.kind, SourceCpuBusAccessKind::OpcodeFetch { .. })));
        assert_eq!(&probe.snes().ram[0x1fc..0x200], &[status, 0x00, 0x80, bank]);
        assert_eq!(probe.program_address(), 0x80c9);
        assert_eq!(probe.snes().cpu.sp, 0x1fb);
        assert!(probe.snes().cpu.i && !probe.snes().cpu.d);
        assert!(probe.snes().in_nmi, "entry does not acknowledge RDNMI");
        for (address, (&old, &new)) in before.iter().zip(&probe.snes().ram).enumerate() {
            if !(0x1fc..0x200).contains(&address) {
                assert_eq!(new, old, "unexpected NMI write at ${address:05x}");
            }
        }
        let rti = probe.step().unwrap();
        assert_eq!(
            rti.ended_at.master_cycles() - rti.started_at.master_cycles(),
            52
        );
        assert_eq!(probe.program_address(), (u32::from(bank) << 16) | 0x8000);
        assert_eq!(probe.snes().cpu.pack_flags(), status);
        assert_eq!(
            (
                probe.snes().cpu.a,
                probe.snes().cpu.x,
                probe.snes().cpu.y,
                probe.snes().cpu.sp
            ),
            (0x2468, 0x12, 0x34, 0x1ff)
        );
    }
}

#[test]
fn nmi_acknowledges_only_the_rdnmi_read_and_rejects_dma_enable() {
    let (mut snes, timeline) = seed(
        &[0xad, 0x10, 0x42, 0xa9, 0x01, 0x8d, 0x0b, 0x42],
        225,
        100,
        false,
    );
    snes.in_nmi = true;
    snes.in_vblank = true;
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    probe.step().unwrap();
    assert!(!probe.snes().in_nmi);
    assert!(probe.snes().in_vblank);
    assert_eq!(probe.snes().cpu.a, 0xc2); // immediate operand publishes $42 before RDNMI
    probe.step().unwrap();
    assert!(matches!(
        probe.step(),
        Err(SourceCpuError::UnsupportedBusMap { address: 0x420b })
    ));
    assert!(!probe.snes().dma.dma_busy);
    assert!(probe.snes().dma.channel.iter().all(|c| !c.dma_active));
}

/// Pinned `cpu.cpp:S9xSoftResetCPU` hands the first instruction a machine that
/// has already read the reset vector through the bus (182 + a 16-clock direct
/// word read) and published its high byte to CPU OpenBus.
fn cold_lorom_reset_seed(rom: &[u8]) -> (Snes, CpuMasterTimeline) {
    let mut snes = Snes::new();
    snes.cart.load(CartType::LoRom, rom, 0x2000);
    snes.cart.ram.fill(0x60);
    snes.ram.fill(0x55);
    let reset_pc = u16::from(snes.cart.rom[0x7ffc]) | (u16::from(snes.cart.rom[0x7ffd]) << 8);
    snes.cpu.pc = reset_pc;
    snes.cpu.k = 0;
    snes.cpu.db = 0;
    snes.cpu.dp = 0;
    snes.cpu.sp = 0x01ff;
    snes.cpu.a = 0;
    snes.cpu.x = 0;
    snes.cpu.y = 0;
    snes.cpu.e = true;
    snes.cpu.mf = true;
    snes.cpu.xf = true;
    snes.cpu.i = true;
    snes.open_bus = (reset_pc >> 8) as u8;
    let mut timeline = CpuMasterTimeline::new(
        198,
        CpuBusWorkload::default(),
        CpuFieldTiming::NON_INTERLACE_EVEN,
    );
    timeline.begin_synchronous_timeline().unwrap();
    (snes, timeline)
}

#[test]
#[ignore = "requires the local external Zelda3 ROM; cold APUI ownership witness"]
fn local_rom_probe_apu_ports_match_the_pinned_cold_boot_writes() {
    use crate::apu::{ApuHostPortProbe, ApuHostPortTiming, ApuState};
    use crate::test_bootstrap_fixture::{cpu_apu_accesses, records};
    use crate::Snes9xApuClockCheckpoint;

    let rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    )
    .unwrap();
    let fixture = records();
    let bootstrap = fixture
        .iter()
        .find(|record| record["kind"] == "bootstrap-events")
        .unwrap();
    let expected: Vec<_> = cpu_apu_accesses(bootstrap)
        .into_iter()
        .take_while(|access| access.v_counter == 0 && !access.is_read)
        .collect();
    assert_eq!(
        expected
            .iter()
            .map(|access| (access.port, access.value))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0), (2, 0), (3, 0)],
        "the recorded cold boot opens with four zeroed APU port writes"
    );

    let (snes, timeline) = cold_lorom_reset_seed(&rom);
    let mut probe = RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset())
        .expect("the LoROM reset seed owns no active hardware");

    let mut apu = ApuState::new();
    apu.reset_snes9x_coroutine();
    assert!(apu.has_exact_dsp_owner());
    let owner = ApuHostPortTiming::new(
        ApuHostPortProbe::from_snes9x_coroutine(apu).unwrap(),
        Snes9xApuClockCheckpoint::new(0, 0, 0).unwrap(),
    )
    .unwrap();
    probe.attach_apu_port_owner(owner).unwrap();

    let mut observed = Vec::new();
    let mut stopped_at = None;
    for _ in 0..64 {
        let origin_pc = probe.program_address();
        match probe.step() {
            Ok(receipt) => {
                for access in &receipt.accesses {
                    let Some(port) = Snes::synchronous_cpu_apu_port(access.address) else {
                        continue;
                    };
                    let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind else {
                        panic!("the boot APU access is a single-byte write");
                    };
                    let cycles = access.timestamp.master_cycles();
                    observed.push((
                        port & 3,
                        value as u8,
                        (cycles / u64::from(MASTER_CYCLES_PER_SCANLINE)) as u16,
                        (cycles % u64::from(MASTER_CYCLES_PER_SCANLINE)) as u16,
                        receipt.ended_at.master_cycles() - receipt.started_at.master_cycles(),
                        probe.apu_ports().unwrap().machine().cycles,
                    ));
                }
            }
            Err(error) => {
                stopped_at = Some((origin_pc, error));
                break;
            }
        }
    }

    assert_eq!(observed.len(), expected.len());
    for (index, (access, actual)) in expected.iter().zip(&observed).enumerate() {
        assert_eq!(
            (actual.0, actual.1, actual.2, actual.3, actual.5),
            (
                access.port & 3,
                access.value,
                access.v_counter,
                access.cpu_cycle,
                access.apu_cycle_after,
            ),
            "recorded cold APU port write {index}"
        );
        // `STZ abs` in emulation mode: an 8-clock opcode fetch, a 16-clock
        // word operand and the 6-clock `$21xx` store.
        assert_eq!(actual.4, 30);
    }

    // The boot's next PPU register write is outside the audited bus map, so
    // the probe fails closed instead of guessing at unowned hardware.
    let (pc, error) = stopped_at.expect("the probe must stop at unowned hardware");
    assert_eq!(pc, 0x00_8018);
    assert!(matches!(
        error,
        SourceCpuError::UnsupportedBusMap { address: 0x00_2100 }
    ));
}

#[test]
fn apu_port_owner_is_refused_when_its_clock_is_ahead_of_the_cpu() {
    use crate::apu::{ApuHostPortProbe, ApuHostPortTiming, ApuState};
    use crate::Snes9xApuClockCheckpoint;

    let (snes, timeline) = seed(&[0xad, 0x40, 0x21], 103, 1168, false);
    let mut probe =
        RomCpuTimingProbe::new(snes, timeline, SourcePpuReadState::snes9x_reset()).unwrap();
    let timeline_cycles = probe.timeline().timestamp().master_cycles();

    let build_owner = |reference: u64| {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        ApuHostPortTiming::new(
            ApuHostPortProbe::from_snes9x_coroutine(apu).unwrap(),
            Snes9xApuClockCheckpoint::new(reference, 0, 0).unwrap(),
        )
        .unwrap()
    };

    assert!(matches!(
        probe.attach_apu_port_owner(build_owner(timeline_cycles + 1)),
        Err(RomCpuTimingProbeSeedError::ApuPortsAheadOfCpu { .. })
    ));
    // Without an owner the APUI read stays outside the audited bus map.
    assert!(matches!(
        probe.step(),
        Err(SourceCpuError::UnsupportedBusMap { address: 0x00_2140 })
    ));
    assert!(probe.is_poisoned());
    assert!(matches!(
        probe.attach_apu_port_owner(build_owner(timeline_cycles)),
        Err(RomCpuTimingProbeSeedError::ApuPortsIntoPoisonedProbe)
    ));
}
