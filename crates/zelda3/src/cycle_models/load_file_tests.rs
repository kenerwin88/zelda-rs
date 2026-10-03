use super::super::{intro_clear::Access, native_bus_tests::ApuFixture};
use super::*;
use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};

#[test]
fn load_file_native_assets_match_source_data() {
    let mut state = crate::ZeldaState::new();
    state.set_assets(&std::fs::read(
        std::env::var_os("ZELDA3_ASSETS").expect("load-game tests require native assets"),
    ).unwrap()).unwrap();
    let rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    ).unwrap();
    let data = state.native_load_file_data().unwrap();
    for (&address, &value) in &data {
        assert_ne!(address & 0x8000, 0, "non-cartridge load-game asset {address:06x}");
        let offset = ((address >> 16) as usize & 127) * 0x8000 + (address as usize & 0x7fff);
        assert_eq!(value, rom[offset], "native load-game asset {address:06x}");
    }
    eprintln!("verified {} native load-game asset bytes", data.len());
}

#[test]
fn load_file_composes_all_destinations_with_live_children() {
    let mut rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    )
    .unwrap();
    for address in (0x02811e..0x028136).chain(0x02a1e5..0x02a1e9) {
        assert_eq!(
            crate::ZeldaState::native_load_file_table_byte(address),
            Some(rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)]),
            "load-game selector {address:06x}"
        );
    }
    // Isolated routine fixture: warm the devices with interrupts disabled,
    // then give both executions the same input state at the source entry.
    rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
    let normalize = |a: u32| {
        if a & 0x400000 == 0 && a & 65535 < 0x6000 {
            a & 65535
        } else {
            a
        }
    };
    let mut instructions = 0;
    let mut scanlines = 0;
    let mut parent_instructions = 0;
    let mut parent_publications = 0;
    let mut native_child_returns = 0;
    let mut parent_stores = std::collections::BTreeSet::new();
    let mut visited = std::collections::BTreeSet::new();
    for case in 0..16usize {
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let wanted = if case & 1 == 0 {
            224 * 1364 + 1338
        } else {
            261 * 1364 + 1350
        };
        let mut apu = ApuFixture::new();
        apu.warm_to(&mut source, wanted);
        let mut native = NativeNmiContext {
            a: 0xab05,
            x: 0x71,
            y: 0x83,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x8136,
            sp: 0x1fa,
            p: 0x31,
        };
        let mut registers = source.machine().snes().cpu.clone();
        registers.a = native.a;
        registers.x = native.x;
        registers.y = native.y;
        registers.dp = 0;
        registers.db = 0;
        registers.k = 2;
        registers.pc = native.pc;
        registers.sp = native.sp;
        registers.e = false;
        registers.unpack_flags(native.p);
        let mut ram = source.machine().snes().ram.clone();
        ram.fill(0);
        ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
        ram[0x10] = if case == 3 { 14 } else { 5 };
        ram[0xf359] = (case % 4) as u8;
        ram[0xf35a] = (case % 3) as u8;
        ram[0xf35b] = (case % 3) as u8;
        ram[0xf36c] = 24;
        ram[0xf36d] = 16;
        ram[0xf353] = (case & 1) as u8 * 2;
        ram[0xf3c5] = match case {
            8 => 0,
            9..=12 => 1,
            14 => 3,
            _ => 2,
        };
        ram[0xf3c6] = if case == 9 { 0x10 } else { 0 };
        ram[0xf3ca] = u8::from(case < 2 || (4..8).contains(&case));
        ram[0x1b] = u8::from(case >= 4);
        ram[0x56] = u8::from(case == 6);
        ram[0x2e0] = u8::from(case == 7);
        // Resident bank inputs exercise the actual SongBank no-transfer path.
        // Runtime bank transfers have their independent live-APU lifecycle test.
        ram[0x136] = u8::from(case != 5);
        ram[0x10e] = match case {
            8 | 9 => 0,
            10 => 2,
            11 => 3,
            12 => 20,
            13 => 4,
            14 => 5,
            15 => 6,
            _ => case.saturating_sub(4) as u8,
        };
        ram[0xc011] = u8::from(case == 13);
        ram[0x10a] = u8::from(case == 14);
        ram[0xf3c8] = if case == 15 { 5 } else { 0 };
        ram[0xf3cc] = if case == 7 { 13 } else { 0 };
        let sram = source.machine().snes().cart.ram.clone();
        source
            .rebase_private_timing_state_at(source.machine().timestamp(), &registers, &ram, &sram)
            .unwrap();
        let mut clock = source.machine().timeline().clone();
        let mut ppu = source.machine().snes().ppu.clone();
        let mut dma = source.machine().snes().dma.clone();
        let mut work = LoadFile::default();
        let mut state = crate::ZeldaState::new();
        state.ram.copy_from_slice(&ram);
        state.sync_native_game_state_from_ram();
        let mut dma_count = 0;
        let mut count = 0;
        while !work.complete() {
            let parent = work.parent_active();
            let origin = (u32::from(native.bank) << 16) | u32::from(native.pc);
            visited.insert(origin);
            let receipt = source.step().unwrap();
            assert!(receipt.accepted_interrupt.is_none());
            let expected: Vec<_> = receipt
                .accesses
                .into_iter()
                .filter_map(|a| match a.kind {
                    Kind::Read { value, width } if a.address != origin + 1 => Some((
                        a.timestamp.master_cycles(),
                        Access::Read(normalize(a.address), width),
                        value,
                    )),
                    Kind::Write { value, width } => Some((
                        a.timestamp.master_cycles(),
                        Access::Write(normalize(a.address), value, width),
                        0,
                    )),
                    _ => None,
                })
                .collect();
            let is_dma = expected
                .iter()
                .any(|(_, a, _)| matches!(a, Access::Write(0x420b, _, _)));
            let mut actual = Vec::new();
            work.step(
                &mut native,
                &mut clock,
                &mut dma,
                apu.bus(|at, access| {
                    let read = |a: u32| {
                        let a = normalize(a);
                        if a < 0x2000 {
                            ram[a as usize]
                        } else if (0x7e0000..0x800000).contains(&a) {
                            ram[(a - 0x7e0000) as usize]
                        } else {
                            assert_ne!(a & 0x8000, 0, "load fixture read {a:06x}");
                            if let Some(value) = crate::ZeldaState::native_load_file_table_byte(a) {
                                return value;
                            }
                            rom[((a >> 16) as usize & 127) * 0x8000 + (a as usize & 0x7fff)]
                        }
                    };
                    let (access, value) = match access {
                        Access::Read(a, width) => {
                            let v = if width == 1 {
                                u16::from(read(a))
                            } else {
                                u16::from_le_bytes([read(a), read(a + 1)])
                            };
                            (Access::Read(normalize(a), width), v)
                        }
                        Access::Write(a, value, width) => {
                            let a = normalize(a);
                            if a < 0x2000 || (0x7e0000..0x800000).contains(&a) {
                                let i = if a < 0x2000 {
                                    a as usize
                                } else {
                                    (a - 0x7e0000) as usize
                                };
                                if parent {
                                    if state.native_load_file_parent_store(a, value, width) {
                                        parent_publications += 1;
                                        parent_stores.insert((a, width));
                                    } else {
                                        assert!((0x100..=0x1ff).contains(&a), "unowned parent store {a:06x} at {origin:06x}");
                                        state.ram[i..i + usize::from(width)]
                                            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                    }
                                }
                                ram[i..i + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            } else if (0x2100..0x2140).contains(&a) {
                                assert_eq!(width, 1);
                                ppu.write(a as u8, value as u8);
                            } else {
                                assert!(
                                    matches!(a, 0x420b | 0x420c | 0x4300..=0x437f),
                                    "load fixture store {a:06x}"
                                );
                            }
                            (Access::Write(a, value, width), 0)
                        }
                        Access::Vblank { .. } => return 0,
                    };
                    if !is_dma || matches!(access, Access::Write(0x420b, _, _)) {
                        actual.push((at, access, value));
                    }
                    value
                }),
            )
            .unwrap();
            let label = format!(
                "load module case{case} pc{origin:06x} owner={:06x}",
                work.routine()
            );
            apu.assert_matches(&source, &label);
            assert_eq!(actual, expected, "{label}");
            let c = &source.machine().snes().cpu;
            assert_eq!(
                native,
                NativeNmiContext {
                    a: c.a,
                    x: c.x,
                    y: c.y,
                    d: c.dp,
                    db: c.db,
                    bank: c.k,
                    pc: c.pc,
                    sp: c.sp,
                    p: c.pack_flags()
                },
                "{label}"
            );
            assert_eq!(clock, *source.machine().timeline(), "{label}");
            assert_eq!(ram, source.machine().snes().ram, "{label}");
            if parent {
                parent_instructions += 1;
                assert_eq!(state.ram, ram, "parent publication {label}");
                assert!(state.game_state == crate::game_state::GameState::load_from_ram(&ram),
                    "{label}: native owners {:?}", state.game_state.report_incoherent_with_ram(&ram));
            } else if work.parent_active() {
                // This test isolates parent publication. A returning child
                // supplies its independently computed native WRAM as the next
                // segment's input; no source RAM feeds the candidate. Full
                // production child publication is a separate integration gate.
                state.ram.copy_from_slice(&ram);
                state.sync_native_game_state_from_ram();
                native_child_returns += 1;
            }
            assert_eq!(ppu.vram, source.machine().snes().ppu.vram, "{label}");
            assert_eq!(
                dma.save_c_saveload(),
                source.machine().snes().dma.save_c_saveload(),
                "{label}"
            );
            if is_dma {
                assert!(work.take_dma_end().unwrap() < clock.clock_master_cycles());
                dma_count += 1;
            }
            work = work.clone();
            count += 1;
            assert!(count < 1000000, "{label}");
        }
        assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
        assert_eq!(dma_count, if case < 4 { 4 } else { 12 });
        assert_eq!(
            ram[0x10],
            if case < 2 {
                8
            } else if case < 4 {
                0x1b
            } else {
                7
            }
        );
        if (2..4).contains(&case) {
            assert_eq!(
                u16::from_le_bytes([ram[0x1cf0], ram[0x1cf1]]),
                if case & 1 == 0 { 0x184 } else { 0x185 }
            );
            assert_eq!(ram[0x10c], if case == 3 { 0 } else { 5 });
        }
        instructions += count;
        scanlines += apu.lines;
    }
    eprintln!("16 complete load fixtures matched {instructions} instructions through all three destinations with 160 DMAs");
    eprintln!("16 complete load fixtures retained independent APU hardware across {scanlines} scanlines; {} parent/message positions", OWNED_POSITIONS.iter().filter(|p| visited.contains(p)).count());
    eprintln!("load parent native publications: instructions={parent_instructions} stores={parent_publications} domains={} native_child_returns={native_child_returns}", parent_stores.len());
    let missing: Vec<_> = OWNED_POSITIONS
        .iter()
        .filter(|p| !visited.contains(p))
        .collect();
    assert!(
        missing.is_empty(),
        "uncovered parent positions: {missing:x?}"
    );
}
