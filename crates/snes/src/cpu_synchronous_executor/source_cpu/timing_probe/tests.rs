use super::*;
use crate::{CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition};

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
