use super::*;

fn check_torch_door_call(rom: &[u8], mut state: ZeldaState, label: &str,
    visited: &mut std::collections::BTreeSet<u32>) {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    state.sync_native_game_state_from_ram();
    let checkpoint = RomCpuCheckpoint {
        entry_pc: 0x01_ce70, stop_pc: 0x00_8036,
        a: 0, x: 0, y: 0, sp: 0x01fb, dp: 0, db: 0,
        carry: true, zero: false, overflow: true, negative: false,
        interrupt_disable: false, decimal: false,
        accumulator_is_8_bit: true, index_is_8_bit: true,
        emulation: false, waiting: false,
        stack_address: 0x01fc, stack_bytes: &[0x35, 0x80, 0],
    };
    let mut run = RomCpuTimingRun::new(rom, &state.ram, &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = state.ram.clone();
    let mut own_writes = std::collections::BTreeSet::new();
    let mut cycles = 0;
    let mut instructions = 0;
    while !run.is_complete() {
        let pc = run.pc();
        let step = run.step();
        let own = (0x01_ce70..=0x01_d1f3).contains(&pc);
        if own { cycles += u64::from(step.master_cycles); visited.insert(pc); }
        for (address, value) in run.take_cpu_wram_writes() {
            expected[address] = value;
            if own && address >= 0x10 && !(0x100..0x200).contains(&address) {
                own_writes.insert(address);
            }
        }
        instructions += 1;
        assert!(instructions < 100_000, "door call did not return: {label}");
    }
    let actual = crate::cycle_ledger::measure_routine_self_work(0x01_ce70,
        || state.dungeon_process_torches_and_doors());
    assert_eq!(actual, cycles, "{label}");
    for range in [0x10..0x19, 0x12e..0x130, 0x400..0x402, 0x460..0x462,
        0x4b8..0x4ba, 0x68c..0x696, 0x1cf0..0x1cf2, 0xf36f..0xf371,
        0x1100..0x1200, 0x2000..0x4000, 0x12000..0x14000] {
        own_writes.extend(range);
    }
    for address in own_writes {
        assert_eq!(state.ram[address], expected[address], "{label} address={address:05x}");
    }
}

#[test]
fn torch_door_branches_work_and_publications_match_rom() {
    let Some(rom) = dungeon_render_test_rom() else { return };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = std::env::var_os("ZELDA3_ASSETS").map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("zelda3_assets.dat"));
    let Ok(assets) = std::fs::read(path) else { return };
    let mut template = ZeldaState::new();
    template.set_assets(&assets).unwrap();
    template.ram[0x10] = 7;
    template.ram[0x1a] = 1; // Timer prefix has its own independent all-values test.
    template.ram[0x1b] = 1;
    template.ram[0x436] = 0x80;
    template.ram[0x20..0x22].copy_from_slice(&256u16.to_le_bytes());
    template.ram[0x22..0x24].copy_from_slice(&256u16.to_le_bytes());
    for k in 0..16 {
        template.ram[0x19a0 + k * 2..0x19a2 + k * 2].copy_from_slice(&(0x800u16 + k as u16 * 4).to_le_bytes());
    }
    let mut visited = std::collections::BTreeSet::new();
    let mut cases = 0;
    for facing in [0u8, 2, 4, 6] {
        for slot in [0, 15] {
            for probe in 0..3 {
                for kind in [0u8, 1, 0x1a, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21,
                    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x32, 0x33, 0xff] {
                    for scenario in 0..16 {
                        let mut state = template.clone();
                        state.ram[0x2f] = facing;
                        state.ram[0x1980 + slot * 2] = kind;
                        state.ram[0x19c0 + slot * 2] = (facing / 2 + if scenario & 8 != 0 { 1 } else { 0 }) & 3;
                        state.ram[0xf36f] = scenario & 1;
                        state.ram[0xf367] = if scenario & 2 != 0 { 0x80 } else { 0 };
                        state.ram[0x4b9] = if scenario & 4 != 0 { 1 } else { 0 };
                        state.ram[0xee] = (scenario >> 2) & 1;
                        // Independent ROM coordinate tables in data bank00.
                        let word = |table| {
                            let o = crate::rom_cpu_timing::lorom_offset(table + u32::from(facing)).unwrap();
                            u16::from_le_bytes([rom[o], rom[o + 1]])
                        };
                        let pos = (((256u16.wrapping_add(word(0x00_99ea)) & 0x1f8) << 3)
                            | ((256u16.wrapping_add(word(0x00_99f2)) & 0x1f8) >> 3)
                            | if state.ram[0xee] != 0 { 0x1000 } else { 0 }) as usize;
                        if probe < 2 {
                            let pos = pos + if probe == 1 { word(0x00_99fa) as usize } else { 0 };
                            state.ram[0x12000 + pos] = 0xf0 | slot as u8;
                        }
                        check_torch_door_call(&rom, state, &format!("door facing={facing} slot={slot} probe={probe} kind={kind:02x} scenario={scenario}"), &mut visited);
                        cases += 1;
                    }
                }
            }
        }
    }
    for facing in [0u8, 2, 4, 6] {
        for running in [0, 1] {
            for counter in [0, 62, 63, 64, 255] {
                for follower_flags in [0, 1, 128] {
                    for full in [false, true] {
                        let mut state = template.clone();
                        state.ram[0x2f] = facing;
                        state.ram[0x1980] = 0x28;
                        state.ram[0x19c0] = facing / 2;
                        state.ram[0x372] = running;
                        state.ram[0x2f1] = counter;
                        state.ram[0x2f2] = follower_flags;
                        state.ram[0x12000..0x14000].fill(0xf0);
                        if full { state.ram[0xc4a..0xc54].fill(1); }
                        check_torch_door_call(&rom, state, &format!("dash facing={facing} running={running} counter={counter} follower={follower_flags} full={full}"), &mut visited);
                        cases += 1;
                    }
                }
            }
        }
    }
    for direction in [0u8, 2, 4, 6, 128] {
        for facing in [0u8, 2, 4, 6] {
            for slot in [0, 3, 15] {
                for scenario in 0..8 {
                    let mut state = template.clone();
                    state.ram[0x2e4] = 1;
                    state.ram[0x436] = direction;
                    state.ram[0x437] = slot as u8 * 2;
                    state.ram[0x2f] = facing;
                    state.ram[0x23] = if scenario & 1 != 0 { 12 } else { 1 };
                    state.ram[0x6c] = (scenario >> 1) & 1;
                    state.ram[0x1980 + slot * 2] = 0x44;
                    state.ram[0x19c0 + slot * 2] = (direction & 6) / 2;
                    if scenario & 4 != 0 { state.ram[0x68c..0x68e].fill(255); }
                    check_torch_door_call(&rom, state, &format!("eye direction={direction} facing={facing} slot={slot} scenario={scenario}"), &mut visited);
                    cases += 1;
                }
            }
        }
    }
    for offset in [0u8, 127, 128, 255] {
        for probe in 0..5 {
            for attr in [0x6cu8, 0x6d, 0x6e, 0x6f, 0xf0, 0xff] {
                for neighbors in 0..4 {
                    for door_type in [0, 0x32, 0x33] {
                        let mut state = template.clone();
                        state.ram[0x2e4] = 1;
                        state.ram[0x3a] = 0x80;
                        state.ram[0x3c] = 4;
                        state.ram[0x44] = offset;
                        state.ram[0x45] = offset;
                        let coordinate = 256u16.wrapping_add(offset as i8 as i16 as u16);
                        let pos = (((coordinate & 0x1f8) << 3) | ((coordinate & 0x1f8) >> 3)) as usize;
                        if probe < 4 {
                            let hit = pos + [0, 1, 64, 65][probe];
                            state.ram[0x12000 + hit] = attr;
                            if attr < 0xf0 {
                                if neighbors & 1 != 0 { state.ram[0x12000 + hit - 64] = attr; }
                                if neighbors & 2 != 0 { state.ram[0x12000 + hit - 1] = attr; }
                                if neighbors == 3 { state.ram[0x12000 + hit - 65] = attr; }
                            }
                        }
                        for slot in [0, 15] { state.ram[0x1980 + slot * 2] = door_type; }
                        check_torch_door_call(&rom, state, &format!("slash offset={offset} probe={probe} attr={attr:02x} neighbors={neighbors} type={door_type:02x}"), &mut visited);
                        cases += 1;
                    }
                }
            }
        }
    }
    for pc in [0x01_cec9, 0x01_cf30, 0x01_cf62, 0x01_cf6b, 0x01_cfab, 0x01_cfda,
        0x01_d024, 0x01_d06b, 0x01_d083, 0x01_d106, 0x01_d128, 0x01_d160, 0x01_d1a1] {
        assert!(visited.contains(&pc), "uncovered branch {pc:06x}");
    }
    eprintln!("torch/door ROM fixtures: {cases}, {} caller instructions covered", visited.len());
}

fn dungeon_render_test_rom() -> Option<Vec<u8>> {
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let mut rom = std::fs::read(path).ok()?;
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    Some(rom)
}

fn dungeon_render_source(rom: &[u8], ram: &[u8], entry: u32, y: u16) -> (u64, Vec<u8>) {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let single_block = entry == 0x07_f0d9;
    let bank = (entry >> 16) as u8;
    let checkpoint = RomCpuCheckpoint {
        entry_pc: entry, stop_pc: (entry & 0xff0000) | 0x8036,
        a: 0, x: 0, y, sp: if single_block { 0x01fc } else { 0x01fb }, dp: 0, db: bank,
        carry: true, zero: false, overflow: true, negative: false,
        interrupt_disable: false, decimal: false,
        accumulator_is_8_bit: true, index_is_8_bit: true,
        emulation: false, waiting: false,
        stack_address: if single_block { 0x01fd } else { 0x01fc },
        stack_bytes: if single_block { &[0x35, 0x80] }
            else if bank == 7 { &[0x35, 0x80, 7] } else { &[0x35, 0x80, 10] },
    };
    let mut run = RomCpuTimingRun::new(rom, ram, &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = ram.to_vec();
    let mut cycles = 0;
    let mut instructions = 0;
    let own_range = match entry {
        0x07_f0ac => 0x07_f0ac..=0x07_f0ca,
        0x07_f0d9 => 0x07_f0d9..=0x07_f13b,
        0x0a_fe80 => 0x0a_fe80..=0x0a_fffa,
        _ => unreachable!(),
    };
    while !run.is_complete() {
        let pc = run.pc();
        let step = run.step();
        if own_range.contains(&pc) { cycles += u64::from(step.master_cycles); }
        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
        instructions += 1;
        assert!(instructions < 100_000, "dungeon render routine did not return");
    }
    (cycles, expected)
}

#[test]
fn dungeon_push_block_scan_and_shared_draw_match_rom() {
    let Some(rom) = dungeon_render_test_rom() else { return };
    for mode in 0..9 {
        for coordinate in [0u16, 255, 256, 0xffff] {
            for scroll in [0u16, 255, 256, 0xffff] {
                // Four slot masks exercise scan order and its empty fast exit;
                // two direct entries independently check the shared child work.
                for variant in 0..6 {
                    let mut state = ZeldaState::new();
                    state.nmi_prepare_sprites();
                    state.ram[0x2c3] = mode;
                    state.ram[0x5fc] = variant & 1;
                    state.ram[0x5fd] = (variant >> 1) & 1;
                    for k in 0..2 {
                        let x = coordinate.wrapping_add(k as u16);
                        let y = coordinate.wrapping_sub(k as u16);
                        state.ram[0x5e4 + k * 2] = x as u8;
                        state.ram[0x5e0 + k * 2] = (x >> 8) as u8;
                        state.ram[0x5f0 + k * 2] = y as u8;
                        state.ram[0x5ec + k * 2] = (y >> 8) as u8;
                    }
                    state.ram[0xe2..0xe4].copy_from_slice(&scroll.to_le_bytes());
                    state.ram[0xe8..0xea].copy_from_slice(&scroll.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    let entry = if variant < 4 { 0x07_f0ac } else { 0x07_f0d9 };
                    let slot_word = u16::from(variant.saturating_sub(4)) * 2;
                    let (cycles, expected) = dungeon_render_source(&rom, &state.ram, entry, slot_word);
                    let actual = crate::cycle_ledger::measure_routine_self_work(entry, || {
                        if variant < 4 { state.sprite_dungeon_draw_all_push_blocks(); }
                        else { state.sprite_dungeon_draw_single_push_block(slot_word as usize); }
                    });
                    let label = format!("mode={mode} coordinate={coordinate} scroll={scroll} variant={variant}");
                    assert_eq!(actual, cycles, "{label}");
                    for range in [0x90..0x94, 0x800..0xaa0, 0xfe0..0xfec] {
                        assert_eq!(&state.ram[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
                    }
                }
            }
        }
    }
}

#[test]
fn dungeon_layer_effect_work_and_owned_state_match_rom() {
    let Some(rom) = dungeon_render_test_rom() else { return };
    for effect in 0..8 {
        for variation in 0u16..512 {
            let mut state = ZeldaState::new();
            state.ram[0xad] = effect;
            state.ram[0x1a] = variation as u8;
            state.ram[0x15] = if variation & 32 != 0 { 255 } else { 0 };
            state.ram[0x16] = 0xa5;
            state.ram[0x403] = if variation & 128 != 0 { 128 } else { 0 };
            for (address, value) in [(0x41a, variation & 0x10f),
                (0x41c, [0u16, 0x7fff, 0x8000, 0xffff][variation as usize % 4]),
                (0x422, 0x7fff), (0x424, 0xffff), (0x312, 1), (0x310, 0xffff),
                (0xe2, 0xffff), (0xe8, 0x7fff), (0x4c5, 0xa5ff)] {
                state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
            }
            // Independently vary the first three Ganon words and later words,
            // which only the invisible-floor routine should count.
            for i in 0..16 {
                let active = if i < 4 { variation & (1 << i) != 0 }
                    else { variation & 16 != 0 };
                let value = if active { 0x8000u16 } else { 0x1234 };
                state.ram[0x540 + i * 2..0x542 + i * 2].copy_from_slice(&value.to_le_bytes());
            }
            for i in 0..256 {
                let value = (i as u16 * 71) & 0x7fff;
                state.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&value.to_le_bytes());
            }
            let floor_color = if variation & 64 != 0 { 0x2940u16 } else { 0 };
            state.ram[0xc3f6..0xc3f8].copy_from_slice(&floor_color.to_le_bytes());
            state.sync_native_game_state_from_ram();
            let (cycles, expected) = dungeon_render_source(&rom, &state.ram, 0x0a_fe80, 0);
            let actual = crate::cycle_ledger::measure_routine_self_work(0x0a_fe80,
                || state.Dungeon_HandleLayerEffect());
            assert_eq!(actual, cycles, "effect={effect} variation={variation}");
            for range in [0x15..0x17, 0x1d..0x1e, 0x9a..0x9b, 0xad..0xae,
                0xe0..0xea, 0x310..0x314, 0x41a..0x426, 0x4c5..0x4c7, 0xc300..0xc700] {
                assert_eq!(&state.ram[range.clone()], &expected[range.clone()],
                    "effect={effect} variation={variation} range={range:x?}");
            }
        }
    }
}

#[test]
fn dungeon_player_control_menu_fallthrough_and_caller_work_match_rom() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for gates in 0..8 {
        for buttons in 0..8 {
            for palace in [0, 0xff] {
                for room in [0u16, 1, 0x100] {
                    for progress in [0, 1] {
                        for map_state in [0, 5] {
                            let mut state = ZeldaState::new();
                            state.ram[0x10] = 7;
                            state.ram[0x1b] = 1;
                            state.ram[0x112] = gates & 1;
                            state.ram[0x2e4] = gates & 2;
                            state.ram[0xffc] = gates & 4;
                            state.ram[0xf4] = if buttons & 1 != 0 { 0x10 } else { 0 };
                            state.ram[0xf6] = if buttons & 2 != 0 { 0x40 } else { 0 };
                            state.ram[0xf0] = if buttons & 4 != 0 { 0x20 } else { 0 };
                            state.ram[0x40c] = palace;
                            state.ram[0xa0..0xa2].copy_from_slice(&room.to_le_bytes());
                            state.ram[0xf3c5] = progress;
                            state.ram[0x200] = map_state;
                            state.ram[0x1ce8] = 3;
                            state.ram[0x1cf4] = 9;
                            state.ram[0x1cf0] = 0x55;
                            state.ram[0xb0] = 0xa5;
                            state.sync_native_game_state_from_ram();
                            let checkpoint = RomCpuCheckpoint {
                                entry_pc: 0x02_88de, stop_pc: 0x02_8036,
                                a: 0, x: 0, y: 0, sp: 0x01fc, dp: 0, db: 2,
                                carry: true, zero: false, overflow: true, negative: false,
                                interrupt_disable: false, decimal: false,
                                accumulator_is_8_bit: true, index_is_8_bit: true,
                                emulation: false, waiting: false,
                                stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
                            };
                            let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
                                &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                            run.enable_cpu_write_trace();
                            let mut expected = state.ram.clone();
                            let mut cycles = 0;
                            let mut instructions = 0;
                            while !run.is_complete() {
                                let pc = run.pc();
                                let step = run.step();
                                if (0x02_88de..=0x02_894b).contains(&pc) {
                                    cycles += u64::from(step.master_cycles);
                                }
                                for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                                instructions += 1;
                                assert!(instructions < 100_000, "player control did not return");
                            }
                            let actual = crate::cycle_ledger::measure_routine_self_work(0x02_88de,
                                || state.module07_00_player_control());
                            let label = format!("gates={gates} buttons={buttons} palace={palace} room={room} progress={progress} map_state={map_state}");
                            assert_eq!(actual, cycles, "{label}");
                            for range in [0x10..0x14, 0xb0..0xb1, 0x10c..0x10d, 0x200..0x201,
                                0x1ce8..0x1ce9, 0x1cf0..0x1cf2, 0x1cf4..0x1cf5, 0xfc1..0xfc6] {
                                assert_eq!(&state.ram[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn dungeon_screen_edge_caller_work_and_transition_state_match_rom() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for x in [0u16, 7, 8, 488, 489, 511] {
        for y in [0u16, 3, 4, 475, 476, 511] {
            for velocities in 0..4 {
                for module in [6, 7] {
                    for incapacitated in [0, 1] {
                        for direction in [0, 3, 12, 15] {
                            let mut state = staircase_fixture(&rom, direction, 0, 0, 0);
                            state.ram[0x10] = module;
                            state.ram[0x20..0x22].copy_from_slice(&(512 + y).to_le_bytes());
                            state.ram[0x22..0x24].copy_from_slice(&(1024 + x).to_le_bytes());
                            state.ram[0x30] = velocities & 1;
                            state.ram[0x31] = velocities & 2;
                            state.ram[0x46] = incapacitated;
                            state.sync_native_game_state_from_ram();
                            let checkpoint = RomCpuCheckpoint {
                                entry_pc: 0x02_885e, stop_pc: 0x02_8036,
                                a: 0, x: 0, y: 0, sp: 0x01fc, dp: 0, db: 2,
                                carry: true, zero: false, overflow: true, negative: false,
                                interrupt_disable: false, decimal: false,
                                accumulator_is_8_bit: true, index_is_8_bit: true,
                                emulation: false, waiting: false,
                                stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
                            };
                            let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
                                &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                            run.enable_cpu_write_trace();
                            let mut expected = state.ram.clone();
                            let mut cycles = 0;
                            let mut instructions = 0;
                            while !run.is_complete() {
                                let pc = run.pc();
                                let step = run.step();
                                if (0x02_885e..=0x02_88c0).contains(&pc) {
                                    cycles += u64::from(step.master_cycles);
                                }
                                for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                                instructions += 1;
                                assert!(instructions < 100_000, "screen edge did not return");
                            }
                            let actual = crate::cycle_ledger::measure_routine_self_work(0x02_885e,
                                || state.Dungeon_TryScreenEdgeTransition());
                            let label = format!("x={x} y={y} velocities={velocities} module={module} incapacitated={incapacitated} direction={direction}");
                            assert_eq!(actual, cycles, "{label}");
                            for range in [0x10..0x14, 0x20..0x24, 0x30..0x32, 0x46..0x47,
                                0x67..0x68, 0xa0..0xa4, 0xa9..0xab, 0xe2..0xe4, 0xe8..0xea,
                                0x600..0x610] {
                                assert_eq!(&state.ram[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn staircase_fixture(rom: &[u8], direction: u8, floor: u8, head: u8, landing: u8) -> ZeldaState {
    use crate::rom_cpu_timing::lorom_offset;
    let mut state = ZeldaState::new();
    state.ram[0x10] = 7;
    state.ram[0x67] = direction;
    state.ram[0xee] = floor;
    for (address, word) in [(0x20, 632u16), (0x22, 1152), (0xa0, 0x12), (0xa2, 0x12),
        (0xfc4, 611), (0x600, 0), (0x602, 256), (0x604, 256), (0x606, 512),
        (0x608, 0), (0x60a, 256), (0x60c, 256), (0x60e, 512)] {
        state.ram[address..address + 2].copy_from_slice(&word.to_le_bytes());
    }
    // Derive the probe from the ROM's own table, independently of the native
    // DUNGEON_DETECT_STAIRCASE_BUGGY_LOOKUP constant.
    let k = direction & 12;
    // The dungeon caller retains data bank 00 even though this code is in 01.
    let offset = lorom_offset(0x00_99ea + u32::from(k)).unwrap();
    let offset = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
    let pos = ((120u16.wrapping_add(offset) & 0x1f8) << 3)
        | ((128u16 & 0x1f8) >> 3) | if floor != 0 { 0x1000 } else { 0 };
    state.ram[0x12000 + pos as usize + if k == 4 { 0x80 } else { 0 }] = head;
    state.ram[0x12000 + pos as usize + 0x40] = landing;
    for j in 0..4 {
        state.ram[0xc001 + j] = 0x30 + j as u8;
        state.ram[0x63d + j] = 0x32 + j as u8;
    }
    state.sync_native_game_state_from_ram();
    state
}

fn check_staircase_against_rom(rom: &[u8], mut state: ZeldaState, label: &str) {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let checkpoint = RomCpuCheckpoint {
        entry_pc: 0x01_c329, stop_pc: 0x01_8036,
        a: 0, x: 0, y: 0, sp: 0x01fc, dp: 0, db: 0,
        carry: true, zero: false, overflow: true, negative: false,
        interrupt_disable: false, decimal: false,
        accumulator_is_8_bit: true, index_is_8_bit: true,
        emulation: false, waiting: false,
        stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
    };
    let mut run = RomCpuTimingRun::new(rom, &state.ram, &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = state.ram.clone();
    let mut own_cycles = 0;
    let mut instructions = 0;
    while !run.is_complete() {
        let pc = run.pc();
        let cycles = u64::from(run.step().master_cycles);
        if (0x01_c325..=0x01_c431).contains(&pc) { own_cycles += cycles; }
        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
        instructions += 1;
        assert!(instructions < 20_000, "staircase did not return: {label}");
    }
    let actual_cycles = crate::cycle_ledger::measure_routine_self_work(0x01_c329,
        || state.Dungeon_DetectStaircase());
    assert_eq!(actual_cycles, own_cycles, "{label}");
    for range in [0x10..0x14, 0x20..0x24, 0x3a..0x3e, 0x48..0x49, 0x50..0x51,
        0xa0..0xa4, 0xb0..0xb1, 0x12e..0x12f, 0x371..0x372, 0x462..0x465,
        0x48a..0x48b, 0x492..0x493, 0xe2..0xe4, 0xe8..0xea, 0x600..0x610] {
        assert_eq!(&state.ram[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
    }
}

#[test]
fn staircase_caller_rejection_and_transition_work_match_rom() {
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for direction in 0..16 {
        for floor in [0, 1] {
            for head in 0..=255 {
                check_staircase_against_rom(&rom, staircase_fixture(&rom, direction, floor, head, 0),
                    &format!("rejection direction={direction} floor={floor} head={head:02x}"));
            }
        }
    }
    for direction in [4, 8, 12] {
        for floor in [0, 1] {
            for head in [0x26, 0x38, 0x39, 0x5e, 0x5f] {
                for landing in 0x30..=0x37 {
                    for carry in [0, 0x80] {
                        for mirror in [0, 1] {
                            for quadrant_y in [0, 2] {
                                for saved_room in [0, 0x12, 0x23] {
                                    let mut state = staircase_fixture(&rom, direction, floor, head, landing);
                                    state.ram[0x308] = carry;
                                    state.ram[0x476] = mirror;
                                    state.ram[0xaa] = quadrant_y;
                                    state.ram[0x48e] = saved_room;
                                    state.sync_native_game_state_from_ram();
                                    check_staircase_against_rom(&rom, state, &format!(
                                        "stair direction={direction} floor={floor} head={head:02x} landing={landing:02x} carry={carry} mirror={mirror} quadrant_y={quadrant_y} saved_room={saved_room:02x}"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn dungeon_camera_work_and_scroll_publications_match_rom() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    let mut visited = std::collections::BTreeSet::new();
    for vy in [0u8, 1, 3, 127, 128, 253, 255] {
        for vx in [0u8, 1, 3, 127, 128, 253, 255] {
            for scenario in 0..192 {
                let mut state = ZeldaState::new();
                let coordinate = [0u16, 110, 120, 250, 511, 0xffff][scenario / 32];
                let bound = [0u16, 100, 103, 0xffff][scenario % 4];
                let subpixel = [0u16, 0x7fff, 0x8000, 0xffff][(scenario / 4) % 4];
                let z = [0u16, 3, 0xffff][scenario % 3];
                let room = if scenario & 16 == 0 { 0x20u16 } else { 0xffff };
                for (address, value) in [
                    (0x20, coordinate), (0x22, coordinate), (0x24, z), (0xa0, room),
                    (0xe0, 0xffff), (0xe2, 100), (0xe6, 0), (0xe8, 100),
                    (0x600, bound), (0x602, bound), (0x604, bound), (0x606, bound),
                    (0x608, bound), (0x60a, bound), (0x60c, bound), (0x60e, bound),
                    (0x618, 120), (0x61a, 122), (0x61c, 120), (0x61e, 122),
                    (0x620, subpixel), (0x622, subpixel),
                ] { state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes()); }
                state.ram[0x30] = vy;
                state.ram[0x31] = vx;
                state.ram[0x78] = (scenario & 1) as u8;
                state.ram[0xa6] = ((scenario / 8) % 2 * 2) as u8;
                state.ram[0xa7] = ((scenario / 8) % 2 * 2) as u8;
                state.ram[0x414] = (scenario % 8) as u8;
                state.sync_native_game_state_from_ram();
                let checkpoint = RomCpuCheckpoint {
                    entry_pc: 0x02_ba31, stop_pc: 0x02_8036,
                    a: 0, x: 0, y: 0, sp: 0x01fc, dp: 0, db: 2,
                    carry: true, zero: false, overflow: true, negative: false,
                    interrupt_disable: false, decimal: false,
                    accumulator_is_8_bit: true, index_is_8_bit: true,
                    emulation: false, waiting: false,
                    stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
                };
                let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
                    &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                run.enable_cpu_write_trace();
                let mut expected = state.ram.clone();
                let mut cycles = 0u64;
                let mut instructions = 0;
                while !run.is_complete() {
                    visited.insert(run.pc());
                    cycles += u64::from(run.step().master_cycles);
                    for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                    instructions += 1;
                    assert!(instructions < 15000, "camera did not return");
                }
                let before = crate::cycle_ledger::master();
                state.dungeon_handle_camera();
                assert_eq!(crate::cycle_ledger::master() - before, cycles,
                    "vy={vy} vx={vx} scenario={scenario}");
                for range in [0xe0..0xea, 0x618..0x624] {
                    assert_eq!(&state.ram[range.clone()], &expected[range],
                        "vy={vy} vx={vx} scenario={scenario}");
                }
            }
        }
    }
    // Both loop bodies, signed half-rate paths, skipped movement and the
    // conditional mirror tail must actually have been exercised.
    for pc in [0x02_ba46, 0x02_ba67, 0x02_ba83, 0x02_ba8c, 0x02_ba97,
        0x02_baaf, 0x02_bab2, 0x02_baf2, 0x02_bb0e, 0x02_bb17,
        0x02_bb22, 0x02_bb3a, 0x02_bb3d, 0x02_bb7b, 0x02_bb85] {
        assert!(visited.contains(&pc), "camera branch {pc:06x} was not exercised");
    }
}

#[test]
fn module07_scroll_caller_work_and_saved_words_match_rom() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for collision in [0, 1, 2, 255] {
        for coordinate in [0u16, 1, 0x7fff, 0x8000, 0xffff] {
            for offset in [0u16, 1, 0x7fff, 0x8000, 0xffff] {
                let mut native = ZeldaState::new();
                for (address, word) in [
                    (0xe0, coordinate), (0xe2, coordinate.wrapping_add(17)),
                    (0xe6, coordinate.wrapping_sub(35)), (0xe8, coordinate.wrapping_add(255)),
                    (0x11a, offset), (0x11c, offset.wrapping_add(1)),
                    (0x422, offset.wrapping_sub(1)), (0x424, offset.wrapping_add(23)),
                ] { native.ram[address..address + 2].copy_from_slice(&word.to_le_bytes()); }
                native.ram[0x428] = collision;
                native.sync_native_game_state_from_ram();
                let mut expected = native.ram.clone();
                let mut saved = None;
                for restoring in [false, true] {
                    if restoring {
                        // Callee drawing may change live scroll; the return
                        // must consume its saved words, including replaced BG1.
                        native.set_bg1_x(0x4567);
                        native.set_bg2_y(0x89ab);
                        expected[0xe0..0xe2].copy_from_slice(&0x4567u16.to_le_bytes());
                        expected[0xe8..0xea].copy_from_slice(&0x89abu16.to_le_bytes());
                    }
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: if restoring { 0x02_8842 } else { 0x02_87e5 },
                        stop_pc: if restoring { 0x0d_a18e } else { 0x02_8838 },
                        a: 0, x: 0, y: 0, sp: if restoring { 0x01f3 } else { 0x01fb }, dp: 0, db: 2,
                        carry: true, zero: false, overflow: true, negative: false,
                        interrupt_disable: false, decimal: false,
                        accumulator_is_8_bit: true, index_is_8_bit: true,
                        emulation: false, waiting: false, stack_address: 0, stack_bytes: &[],
                    };
                    let mut run = RomCpuTimingRun::new(&rom, &expected, &[0; 0x2000],
                        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                    run.enable_cpu_write_trace();
                    let mut cycles = 0u64;
                    let mut steps = 0;
                    while !run.is_complete() {
                        cycles += u64::from(run.step().master_cycles);
                        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                        steps += 1;
                        assert!(steps < 100, "scroll phase did not complete");
                    }
                    let before = crate::cycle_ledger::master();
                    if restoring {
                        native.restore_module07_scroll_before_link_oam(saved.as_ref().unwrap());
                    } else {
                        saved = Some(native.begin_module07_sprite_scroll());
                    }
                    assert_eq!(crate::cycle_ledger::master() - before, cycles,
                        "collision={collision} coordinate={coordinate:04x} offset={offset:04x} restoring={restoring}");
                    assert_eq!(&native.ram[0xe0..0xea], &expected[0xe0..0xea]);
                    assert_eq!(&native.ram[0x11e..0x126], &expected[0x11e..0x126]);
                }
            }
        }
    }
}

#[test]
fn room_tag_caller_and_empty_returns_match_rom_for_every_skip_value() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for skip in 0..=u8::MAX {
        let mut state = ZeldaState::new();
        state.ram[0x4c7] = skip;
        state.ram[0xe] = 0xa5;
        state.ram[0xf] = 0x5a;
        state.sync_native_game_state_from_ram();
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x01_c2fd, stop_pc: 0x00_8036,
            a: 0, x: 0, y: 0, sp: 0x01fb, dp: 0, db: 1,
            carry: false, zero: false, overflow: false, negative: false,
            interrupt_disable: false, decimal: false,
            accumulator_is_8_bit: true, index_is_8_bit: true,
            emulation: false, waiting: false,
            stack_address: 0x01fc, stack_bytes: &[0x35, 0x80, 0],
        };
        let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
            &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint)
            .unwrap();
        run.enable_cpu_write_trace();
        let mut source_cycles = 0;
        let mut source_ram = state.ram.clone();
        let mut steps = 0;
        let mut in_staircase = false;
        while !run.is_complete() {
            let pc = run.pc();
            if pc == 0x01_c329 { in_staircase = true; }
            if pc == 0x01_c307 { in_staircase = false; }
            let step = run.step();
            // C326/C328 are also the staircase's shared return tail. That
            // invocation belongs to the staircase, not either empty tag.
            if !in_staircase && (0x01_c2fd..=0x01_c328).contains(&pc) {
                source_cycles += u64::from(step.master_cycles);
            }
            for (address, value) in run.take_cpu_wram_writes() {
                source_ram[address] = value;
            }
            steps += 1;
            assert!(steps < 100_000, "skip {skip}: caller did not return");
        }
        let before = crate::cycle_ledger::master();
        if skip == 0 { state.clone().Dungeon_DetectStaircase(); }
        let staircase_cycles = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        state.dungeon_handle_room_tags();
        assert_eq!(crate::cycle_ledger::master() - before - staircase_cycles,
            source_cycles, "skip {skip}");
        assert_eq!(&state.ram[0xe..0x10], &source_ram[0xe..0x10], "skip {skip}");
        assert_eq!(state.ram[0x4c7], source_ram[0x4c7], "skip {skip}");
    }
}

#[test]
fn torch_timer_prefix_cost_and_timer_writes_match_rom() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }

    // Every timer value, each expiry slot, both gates, and simultaneous
    // expirations. Only the prefix's own ROM instructions are compared:
    // the extinguish routine has separate, still incomplete pricing.
    let mut cases = Vec::new();
    for timer in 0..=u8::MAX {
        cases.push((0, 0, [timer; 16]));
    }
    for slot in 0..16 {
        let mut timers = [2; 16];
        timers[slot] = 1;
        cases.push((252, 0, timers));
    }
    for frame in 0..=u8::MAX {
        for spell in [0, 1, 255] {
            cases.push((frame, spell, [0, 1, 2, 255, 0, 1, 2, 255,
                                      0, 1, 2, 255, 0, 1, 2, 255]));
        }
    }
    for (case, (frame, spell, timers)) in cases.into_iter().enumerate() {
        let mut state = ZeldaState::new();
        state.ram[0x1a] = frame;
        state.ram[0x112] = spell;
        state.ram[0x4f0..0x500].copy_from_slice(&timers);
        state.sync_native_game_state_from_ram();
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x01_ce70, stop_pc: 0x01_ce98,
            a: 0xabcd, x: 0x31, y: 0x42, sp: 0x01ff, dp: 0, db: 1,
            carry: true, zero: false, overflow: true, negative: true,
            interrupt_disable: false, decimal: false,
            accumulator_is_8_bit: true, index_is_8_bit: true,
            emulation: false, waiting: false, stack_address: 0, stack_bytes: &[],
        };
        let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
            &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint)
            .unwrap();
        run.enable_cpu_write_trace();
        let mut source_cycles = 0;
        let mut source_timers = timers;
        let mut steps = 0;
        while !run.is_complete() {
            let pc = run.pc();
            let step = run.step();
            if (0x01_ce70..0x01_ce98).contains(&pc) {
                source_cycles += u64::from(step.master_cycles);
            }
            for (address, value) in run.take_cpu_wram_writes() {
                if (0x4f0..0x500).contains(&address) {
                    source_timers[address - 0x4f0] = value;
                }
            }
            steps += 1;
            assert!(steps < 100_000, "case {case}: prefix did not return");
        }

        // Measure the native callees independently so adding their missing
        // charges later does not turn this prefix test into a false failure.
        let mut callees = state.clone();
        let before = crate::cycle_ledger::master();
        if frame & 3 == 0 && spell == 0 {
            for (slot, timer) in timers.into_iter().enumerate() {
                if timer != 0 {
                    callees.dungeon_torch_mut().set_timer(slot, timer - 1);
                    if timer == 1 {
                        callees.dungeon_torch_mut().set_target(NativeTile::torch(slot));
                        callees.Dungeon_ExtinguishTorch();
                    }
                }
            }
        }
        let callee_cycles = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        state.dungeon_process_torch_timers();
        assert_eq!(crate::cycle_ledger::master() - before - callee_cycles,
            source_cycles, "case {case}: frame={frame} spell={spell} timers={timers:?}");
        assert_eq!(&state.ram[0x4f0..0x500], &source_timers,
            "case {case}: timer publication");
        for (slot, timer) in source_timers.into_iter().enumerate() {
            assert_eq!(state.game_state.dungeon.torch.timer(slot), timer,
                "case {case}: native timer {slot}");
        }
    }
}

#[test]
fn quadrant_copy_after_trailing_nmi_returns_without_running_another_handler() {
    for words in [0, 900, 1024] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_animated_tile_data_source_address(0xa680);
        state.set_indoor_flag(1);
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(11);
        state.set_frame_counter(0xa5);
        state.latch_nmi_update();
        state.pending_main_loop_common_suffix =
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
        let vram = state.ppu.vram.clone();
        let mut copy = state.begin_dungeon_quadrant_upload(ROOM_BG2_TILEMAP_BASE, 0, 0);
        state.advance_dungeon_quadrant_upload(&mut copy, words);
        state.complete_post_trailing_nmi_continuation(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::QuadrantUploadCopy { copy },
            }, 0, false, false);
        assert_eq!(state.game_state.frame.subsubmodule, 12);
        assert_eq!(state.game_state.frame.frame_counter, 0xa5);
        assert_eq!(state.game_state.dungeon.room_load.quadrant_upload_index(), 4);
        assert!(!state.game_state.display.nmi_update_is_latched());
        assert_eq!(state.ram[crate::game_state::constants::NMI_SUBROUTINE_INDEX], 1);
        assert_eq!(state.ram[crate::game_state::constants::NMI_DISABLE_CORE_UPDATES], 1);
        assert_eq!(state.ppu.vram, vram, "the following Open NMI owns the upload");
        state.game_execution_scheduler.begin_host_frame();
        assert!(state.game_execution_scheduler.main_return_requires_leading_nmi());
    }
}

#[test]
fn cached_sprite_cpu_receipt_transfers_once_between_module_and_caller() {
    let mut state = ZeldaState::new();
    let receipt = crate::CachedSpriteExecutionProgressReceipt {
        progress: crate::CachedSpriteExecutionProgress::Executing {
            slot: 3, progress: crate::CachedSpriteExecutionBodyProgress::AfterCoordinates,
        },
        boundary: OriginalTimingBoundary::NmiAccepted,
    };
    state.native_exact_cpu_host_trace = Some(super::super::rtl_native_exact_cpu::NativeExactCpuHostTrace {
        cached_sprite_progress: Some(receipt), ..Default::default()
    });
    assert!(state.arm_live_cached_sprite_main_cpu_continuation());
    assert!(!state.arm_live_cached_sprite_main_cpu_continuation());
    assert_eq!(state.dungeon_cached_sprite_cpu_interruption_pending,
        Some(CachedSpriteCpuInterruption::ExecutingAfterCoordinates { slot: 3, state: None }));
    assert_eq!(state.dungeon_cached_sprite_cpu_interruption_boundary,
        Some(OriginalTimingBoundary::NmiAccepted));
}

#[test]
fn draw_completely_open_door_uses_inter_room_upnorth_count_before_wall_spirals() {
    let mut state = ZeldaState::new();
    let pos = 0x011eusize;

    state.dungeon_bg2_attributes_mut().fill_bg2_attr_range(
        pos + xy(1, 0),
        xy(1, 3) - xy(1, 0) + 2,
        0xf0,
    );
    state
        .dungeon_stair_lists_mut()
        .set_inter_staircase_pos(0, pos as u16);

    state
        .dungeon_stair_lists_mut()
        .set_stair_list_count(DungeonStairList::InterRoomUpNorth, 0);
    state.dungeon_stair_lists_mut().sync_stair_list_counts(
        &[
            DungeonStairList::WallUpNorthSpiral,
            DungeonStairList::WallUpNorthSpiralBg1,
            DungeonStairList::InterRoomUpNorthStraight,
            DungeonStairList::InterRoomUpSouthStraight,
            DungeonStairList::InterRoomSouthDown,
            DungeonStairList::WallDownNorthSpiral,
            DungeonStairList::WallDownNorthSpiralBg1,
            DungeonStairList::InterPseudoUpNorth,
        ],
        2,
    );

    state.DrawCompletelyOpenDoor();

    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 0), 2),
        &[0x5e, 0x5e]
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 1), 2),
        &[0x30, 0x30]
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 2), 2),
        &[0x00, 0x00]
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 3), 2),
        &[0x00, 0x00]
    );
}

#[test]
fn room_draw_all_objects_clears_width_height_before_terminator() {
    let mut state = ZeldaState::new();
    state
        .dungeon_room_load_mut()
        .set_draw_dimensions_words(0x4000, 0x2000);
    write_le_u16(&mut state.ram, DUNG_LOAD_PTR_OFFS, 0);

    state.RoomData_DrawObjects_from(&[0xff, 0xff]);

    assert_eq!(
        state
            .game_state
            .dungeon
            .room_load
            .draw_width_indicator_word(),
        0
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .room_load
            .draw_height_indicator_word(),
        0
    );
}

#[test]
fn dungeon_load_room_resets_floor_velocity_words() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, DUNG_FLOOR_Y_VEL_DUNGEON, 0x0002);
    write_le_u16(&mut state.ram, DUNG_FLOOR_X_VEL, 0x0001);

    state.dungeon_load_room_reset_floor_velocity();

    assert_eq!(read_le_u16(&state.ram, DUNG_FLOOR_Y_VEL_DUNGEON), 0);
    assert_eq!(read_le_u16(&state.ram, DUNG_FLOOR_X_VEL), 0);
}
