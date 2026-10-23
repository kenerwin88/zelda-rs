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
    let mut run = RomCpuTimingRun::new(rom, &state.wram_image(), &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = state.wram_image().into_owned();
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
        assert_eq!(state.read_wram_byte(address), expected[address], "{label} address={address:05x}");
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
    template.write_wram_byte(0x10, 7);
    template.write_wram_byte(0x1a, 1); // Timer prefix has its own independent all-values test.
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
                        state.write_wram_byte(0xf36f, scenario & 1);
                        state.write_wram_byte(0xf367, if scenario & 2 != 0 { 0x80 } else { 0 });
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
                    let (cycles, expected) = dungeon_render_source(&rom, &state.wram_image(), entry, slot_word);
                    let actual = crate::cycle_ledger::measure_routine_self_work(entry, || {
                        if variant < 4 { state.sprite_dungeon_draw_all_push_blocks(); }
                        else { state.sprite_dungeon_draw_single_push_block(slot_word as usize); }
                    });
                    let label = format!("mode={mode} coordinate={coordinate} scroll={scroll} variant={variant}");
                    assert_eq!(actual, cycles, "{label}");
                    for range in [0x90..0x94, 0x800..0xaa0, 0xfe0..0xfec] {
                        assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
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
            state.write_wram_byte(0x1a, variation as u8);
            state.write_wram_byte(0x15, if variation & 32 != 0 { 255 } else { 0 });
            state.write_wram_byte(0x16, 0xa5);
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
            let (cycles, expected) = dungeon_render_source(&rom, &state.wram_image(), 0x0a_fe80, 0);
            let actual = crate::cycle_ledger::measure_routine_self_work(0x0a_fe80,
                || state.dungeon_handle_layer_effect());
            assert_eq!(actual, cycles, "effect={effect} variation={variation}");
            for range in [0x15..0x17, 0x1d..0x1e, 0x9a..0x9b, 0xad..0xae,
                0xe0..0xea, 0x310..0x314, 0x41a..0x426, 0x4c5..0x4c7, 0xc300..0xc700] {
                assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()],
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
                            state.write_wram_byte(0x10, 7);
                            state.ram[0x1b] = 1;
                            state.ram[0x112] = gates & 1;
                            state.ram[0x2e4] = gates & 2;
                            state.ram[0xffc] = gates & 4;
                            state.ram[0xf4] = if buttons & 1 != 0 { 0x10 } else { 0 };
                            state.ram[0xf6] = if buttons & 2 != 0 { 0x40 } else { 0 };
                            state.ram[0xf0] = if buttons & 4 != 0 { 0x20 } else { 0 };
                            state.write_wram_byte(0x40c, palace);
                            state.ram[0xa0..0xa2].copy_from_slice(&room.to_le_bytes());
                            state.write_wram_byte(0xf3c5, progress);
                            state.ram[0x200] = map_state;
                            state.ram[0x1ce8] = 3;
                            state.ram[0x1cf4] = 9;
                            state.ram[0x1cf0] = 0x55;
                            state.write_wram_byte(0xb0, 0xa5);
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
                            let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &[0; 0x2000],
                                &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                            run.enable_cpu_write_trace();
                            let mut expected = state.wram_image().into_owned();
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
                                assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
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
                            state.write_wram_byte(0x10, module);
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
                            let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &[0; 0x2000],
                                &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                            run.enable_cpu_write_trace();
                            let mut expected = state.wram_image().into_owned();
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
                                || state.dungeon_try_screen_edge_transition());
                            let label = format!("x={x} y={y} velocities={velocities} module={module} incapacitated={incapacitated} direction={direction}");
                            assert_eq!(actual, cycles, "{label}");
                            for range in [0x10..0x14, 0x20..0x24, 0x30..0x32, 0x46..0x47,
                                0x67..0x68, 0xa0..0xa4, 0xa9..0xab, 0xe2..0xe4, 0xe8..0xea,
                                0x600..0x610] {
                                assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
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
    state.write_wram_byte(0x10, 7);
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
    let mut run = RomCpuTimingRun::new(rom, &state.wram_image(), &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = state.wram_image().into_owned();
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
        || state.dungeon_detect_staircase());
    assert_eq!(actual_cycles, own_cycles, "{label}");
    for range in [0x10..0x14, 0x20..0x24, 0x3a..0x3e, 0x48..0x49, 0x50..0x51,
        0xa0..0xa4, 0xb0..0xb1, 0x12e..0x12f, 0x371..0x372, 0x462..0x465,
        0x48a..0x48b, 0x492..0x493, 0xe2..0xe4, 0xe8..0xea, 0x600..0x610] {
        assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
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
                let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &[0; 0x2000],
                    &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                run.enable_cpu_write_trace();
                let mut expected = state.wram_image().into_owned();
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
                    assert_eq!(&state.wram_image()[range.clone()], &expected[range],
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
                let mut expected = native.wram_image().into_owned();
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
                    assert_eq!(&native.wram_image()[0xe0..0xea], &expected[0xe0..0xea]);
                    assert_eq!(&native.wram_image()[0x11e..0x126], &expected[0x11e..0x126]);
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
        let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &[0; 0x2000],
            &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint)
            .unwrap();
        run.enable_cpu_write_trace();
        let mut source_cycles = 0;
        let mut source_ram = state.wram_image().into_owned();
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
        if skip == 0 { state.clone().dungeon_detect_staircase(); }
        let staircase_cycles = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        state.dungeon_handle_room_tags();
        assert_eq!(crate::cycle_ledger::master() - before - staircase_cycles,
            source_cycles, "skip {skip}");
        assert_eq!(&state.wram_image()[0xe..0x10], &source_ram[0xe..0x10], "skip {skip}");
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
        state.write_wram_byte(0x1a, frame);
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
        let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &[0; 0x2000],
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
                        callees.dungeon_extinguish_torch();
                    }
                }
            }
        }
        let callee_cycles = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        state.dungeon_process_torch_timers();
        assert_eq!(crate::cycle_ledger::master() - before - callee_cycles,
            source_cycles, "case {case}: frame={frame} spell={spell} timers={timers:?}");
        assert_eq!(&state.wram_image()[0x4f0..0x500], &source_timers,
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
        state.game_execution_scheduler.begin_host_frame();
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
        assert_eq!(state.read_wram_byte(crate::game_state::constants::NMI_SUBROUTINE_INDEX), 1);
        assert_eq!(state.read_wram_byte(crate::game_state::constants::NMI_DISABLE_CORE_UPDATES), 1);
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

    state.room_data_draw_objects_from(&[0xff, 0xff]);

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

    assert_eq!(read_le_u16(&state.wram_image(), DUNG_FLOOR_Y_VEL_DUNGEON), 0);
    assert_eq!(read_le_u16(&state.wram_image(), DUNG_FLOOR_X_VEL), 0);
}

#[test]
fn native_interaction_handoffs_skip_owned_timers_probes_and_calls() {
    use crate::cycle_models::{
        dungeon_interaction_caller::{DungeonInteractionCaller, InteractionTail},
        intro_clear::Access,
        nmi_stack::NativeNmiContext,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = std::env::var_os("ZELDA3_ASSETS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("zelda3_assets.dat"));
    let assets = std::fs::read(path).expect("assets required for native interaction handoffs");
    let mut count = 0;
    let mut tails = [0usize; 6];
    for case in 0..192usize {
        let mut expected = ZeldaState::new();
        expected.set_assets(&assets).unwrap();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_interaction_caller::seed_case(&mut image, case); expected.import_wram_image(&image); }
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x1b] = 1;
        expected.ram[0x3c] = if case / 6 & 1 == 0 { 4 } else { 3 };
        expected.ram[0x44] = (case as u8).wrapping_mul(17);
        expected.ram[0x45] = 255;
        for k in 0..16 {
            expected.ram[0x19c0 + k * 2] = (case / 6 % 4) as u8;
            expected.ram[0x1980 + k * 2] = if case / 6 & 1 == 0 { 0x32 } else { 0 };
            expected.ram[0x19a0 + k * 2..0x19a2 + k * 2]
                .copy_from_slice(&(0x800 + k as u16 * 4).to_le_bytes());
        }
        if case % 6 == 4 {
            expected.ram[0x23] = if case / 6 & 1 == 0 { 12 } else { 9 };
            expected.ram[0x68c..0x68e].fill(if case / 6 & 2 != 0 { 255 } else { 0 });
        }
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0x1111,
            x: 255,
            y: 8,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x87c9,
            sp: 0x1fc,
            p: 0x32,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(49, 1298),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonInteractionCaller::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(low) = crate::cycle_models::dungeon_interactions::table_byte(a)
                        {
                            u16::from(low)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::dungeon_interactions::table_byte(
                                            a + 1,
                                        )
                                        .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let offset = if a >= 0x7e0000 {
                                a as usize - 0x7e0000
                            } else {
                                a as usize
                            };
                            u16::from(actual.read_wram_byte(offset))
                                | if width == 2 {
                                    u16::from(actual.ram[offset + 1]) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, value, width) => {
                        let offset = if a >= 0x7e0000 {
                            a as usize - 0x7e0000
                        } else {
                            a as usize
                        };
                        actual.ram[offset..offset + width as usize]
                            .copy_from_slice(&value.to_le_bytes()[..width as usize]);
                        actual.native_dungeon_camera_store(offset, value, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        let tail = work.tail().unwrap();
        tails[match tail {
            InteractionTail::Lamp => 0,
            InteractionTail::Returned => 1,
            InteractionTail::TorchExpired { .. } => 2,
            InteractionTail::KeyDoor { .. } => 3,
            InteractionTail::EyeDoor => 4,
            InteractionTail::Slash => 5,
        }] += 1;
        let before = crate::cycle_ledger::master();
        expected.complete_module07_room_maintenance_after_room_tags();
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.finish_native_dungeon_interactions(tail);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(
            diffs.is_empty(),
            "case={case} tail={tail:?} RAM differences {diffs:x?}"
        );
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(
            diffs.is_empty(),
            "case={case} tail={tail:?} native differences {diffs:x?}"
        );
        assert_eq!(
            full - remaining,
            owned,
            "owned instructions omitted case={case} tail={tail:?}"
        );
        assert!(actual.rom.is_empty());
        count += 1;
    }
    assert_eq!(tails, [32; 6]);
    println!("native interaction handoffs: {count} fixtures, all six tails, completed timers/probes/calls never replayed");
}

#[test]
fn edge_predicate_all_handler_states_and_recoil_publications_match_source() {
    use crate::cycle_models::nmi_stack::CARRY;
    let rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    )
    .unwrap();
    for handler in 0..=255u8 {
        for timer in [0u8, 1, 255] {
            let mut state = ZeldaState::new();
            state.ram[0x5d] = handler;
            state.ram[0x46] = timer;
            state.ram[0x27..0x29].copy_from_slice(&[255, 127]);
            state.ram[0x20..0x24].copy_from_slice(&[7, 2, 233, 4]);
            state.ram[0xfc2..0xfc6].copy_from_slice(&[0xef, 0xbe, 0x23, 1]);
            state.sync_native_game_state_from_ram();
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.ram[0x1f8..0x1fb].copy_from_slice(&[0xa7, 0x88, 2]);
            machine.cpu.k = 7;
            machine.cpu.pc = 0xf439;
            machine.cpu.sp = 0x1f7;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0x30);
            let clock = snes::CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(50, 372),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                clock,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            for _ in 0..100 {
                if (source.snes().cpu.k, source.snes().cpu.pc) == (2, 0x88a8) {
                    break;
                }
                source.step().unwrap();
            }
            assert_eq!((source.snes().cpu.k, source.snes().cpu.pc), (2, 0x88a8));
            let blocked = state.link_check_for_edge_screen_transition();
            assert_eq!(
                blocked,
                source.snes().cpu.pack_flags() & CARRY != 0,
                "handler={handler} timer={timer}"
            );
            let mut projected = state.wram_image().into_owned();
            state
                .game_state
                .player
                .follower_link
                .write_to_ram(&mut projected);
            for (a, b) in [(0x20, 0x24), (0x27, 0x29), (0x2c6, 0x2c7)] {
                assert_eq!(
                    &state.wram_image()[a..b],
                    &source.snes().ram[a..b],
                    "handler={handler} timer={timer}"
                );
                assert_eq!(&projected[a..b], &state.wram_image()[a..b]);
            }
        }
    }
    println!("edge predicate: 768 source fixtures, all handler bytes and zero/nonzero incapacitation timers");
}

#[test]
fn native_edge_handoffs_preserve_remaining_caller_and_do_not_repeat_predicate() {
    use crate::cycle_models::{
        dungeon_edge_caller::{DungeonEdgeCaller, EdgeTail},
        intro_clear::Access,
        nmi_stack::NativeNmiContext,
    };
    let mut tails = [0usize; 3];
    let mut count = 0;
    for case in 0..192usize {
        let kind = case % 6;
        let mut expected = ZeldaState::new();
        for (a, v) in [
            (0x10, 7u8),
            (0x1b, 1),
            (0x454, if kind == 1 { 1 } else { 0 }),
            (0x455, 255),
            (0x67, 15),
            (0x27, 255),
            (0x28, 127),
        ] {
            expected.write_wram_byte(a, v);
        }
        expected.ram[0x20..0x24].copy_from_slice(&[0, 1, 0, 1]);
        for (a, v) in [
            (0x600, 0u16),
            (0x602, 256),
            (0x604, 256),
            (0x606, 512),
            (0x608, 0),
            (0x60a, 256),
            (0x60c, 256),
            (0x60e, 512),
            (0xa0, 0x12),
            (0xa2, 0x12),
        ] {
            expected.ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
        }
        if kind >= 2 {
            let dir = kind - 2;
            if dir < 2 {
                expected.ram[0x31] = 1;
                expected.ram[0x22..0x24]
                    .copy_from_slice(&(if dir == 0 { 489u16 } else { 7 }).to_le_bytes());
            } else {
                expected.ram[0x30] = 1;
                expected.ram[0x20..0x22]
                    .copy_from_slice(&(if dir == 2 { 476u16 } else { 3 }).to_le_bytes());
            }
        }
        expected.ram[0x5d] = if case / 6 % 2 == 0 { 0 } else { 3 };
        expected.ram[0x46] = if case / 6 % 4 == 2 { 1 } else { 0 };
        expected.ram[0xfc2..0xfc6].copy_from_slice(&[0xef, 0xbe, 0x23, 1]);
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0x2100,
            x: 255,
            y: 8,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x87d1,
            sp: 0x1fc,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(50, 372),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonEdgeCaller::default();
        let mut owned_caller = 0;
        while work.tail().is_none() {
            let predicate = work.routine() == 0x07f439;
            let cycles = work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        let a = (a & 65535) as usize;
                        u16::from(actual.read_wram_byte(a))
                            | if width == 2 {
                                u16::from(actual.read_wram_byte(a + 1)) << 8
                            } else {
                                0
                            }
                    }
                    Access::Write(a, value, width) => {
                        let a = (a & 65535) as usize;
                        actual.ram[a..a + width as usize]
                            .copy_from_slice(&value.to_le_bytes()[..width as usize]);
                        actual.native_dungeon_camera_store(a, value, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
            if !predicate {
                owned_caller += cycles;
            }
        }
        let tail = work.tail().unwrap();
        tails[match tail {
            EdgeTail::Lamp => 0,
            EdgeTail::BlastWall => 1,
            EdgeTail::Movement { .. } => 2,
        }] += 1;
        let before = crate::cycle_ledger::master();
        expected.complete_module07_room_maintenance_after_interactions();
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.finish_native_dungeon_edge(tail);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} tail={tail:?} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(
            diffs.is_empty(),
            "case={case} tail={tail:?} native {diffs:x?}"
        );
        assert_eq!(full-remaining,owned_caller,"case={case} tail={tail:?}: caller pricing excludes the previously unpriced predicate body");
        count += 1;
    }
    assert!(tails.iter().all(|n| *n > 0));
    println!("native edge handoffs: {count} fixtures, tails={tails:?}, owned guards/calls and recoil predicate never replayed");
}

#[test]
fn native_lamp_handoff_preserves_sprite_suffix_without_replaying_cone_work() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_lamp_caller::DungeonLampCaller,
        intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..192usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_lamp_caller::seed_case(&mut image, case * 37); expected.import_wram_image(&image); }
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x11, if case % 3 == 0 { 20 } else { 17 });
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x87e1,
            sp: 0x1fc,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(50, 794),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonLampCaller::default();
        let mut owned = 46;
        while !work.complete() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::lamp_cone::table_byte(a) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::lamp_cone::table_byte(a + 1).unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & 65535) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & 65535) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_dungeon_camera_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::AfterLamp);
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(full - remaining, owned, "case={case}: no owned lamp replay");
    }
    println!("native lamp handoff: 192 fixtures, source scrolls and sprite suffix match, guard/call/cone work omitted");
}

#[test]
fn native_sprite_scroll_handoff_preserves_drawing_suffix_without_replaying_offsets() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_sprite_scroll::DungeonSpriteScroll,
        intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..192usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_sprite_scroll::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x11, if case % 3 == 0 { 20 } else { 17 });
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x87e5,
            sp: 0x1fc,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(50, 954),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonSpriteScroll::default();
        let mut owned = 0;
        while work.saved().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        let a = (a & 65535) as usize;
                        u16::from(actual.read_wram_byte(a))
                            | if width == 2 {
                                u16::from(actual.read_wram_byte(a + 1)) << 8
                            } else {
                                0
                            }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & 65535) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_dungeon_camera_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail =
            Some(DungeonAfterPushTail::BeforeDrawing(work.saved().unwrap()));
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::AfterLamp);
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned sprite scroll replay"
        );
    }
    println!("native sprite scroll handoff: 192 fixtures, saved words and drawing suffix match, scroll work omitted");
}

#[test]
fn native_push_drawing_handoff_preserves_sprite_suffix_without_replaying_oam() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_push_drawing::DungeonPushDrawing,
        intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..192usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_push_drawing::seed_case(&mut image, case * 13); expected.import_wram_image(&image); }
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x11, if case % 3 == 0 { 20 } else { 17 });
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x8838,
            sp: 0x1f4,
            p: 0x10,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(51, 478),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonPushDrawing::default();
        let mut owned = 0;
        while !work.complete() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a,width) => {
                        if let Some(v)=crate::cycle_models::push_block_drawing::table_byte(a) {
                            u16::from(v) | if width==2 {u16::from(crate::cycle_models::push_block_drawing::table_byte(a+1).unwrap())<<8} else {0}
                        } else {
                            let a=(a&65535) as usize;
                            u16::from(actual.read_wram_byte(a)) | if width==2 {u16::from(actual.ram[a+1])<<8} else {0}
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & 65535) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::AfterDrawing(saved));
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::BeforeDrawing(saved));
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned push drawing replay"
        );
    }
    println!("native push drawing handoff: 192 fixtures, OAM and sprite suffix match, drawing work omitted");
}

#[test]
fn native_sprite_prefix_handoff_preserves_all_remaining_sprite_work_without_reset_replay() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_sprite_prefix::DungeonSpritePrefix,
        intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..192usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_sprite_prefix::seed_case(&mut image, case * 13); expected.import_wram_image(&image); }
        expected.write_wram_byte(0x10, 7);

        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 0,
            bank: 2,
            pc: 0x883e,
            sp: 0x1f4,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(51, 832),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonSpritePrefix::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & 65535) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & 65535) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: work.tail().unwrap(),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::AfterDrawing(saved));
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned sprite prefix replay"
        );
    }
    println!("native sprite prefix handoff: 192 fixtures, both worlds, remaining sprite work matches, completed reset and calls omitted");
}


#[test]
fn native_garnish_handoff_preserves_remaining_palette_and_slot_work_once() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_garnish::DungeonGarnish,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..528usize {
        let mut expected = ZeldaState::new();
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x11, 0);
        expected.ram[0xfb4] = u8::from(case / 88 != 0);
        expected.ram[0xff9] = [0, 0, 1, 3, 2, 0][case / 88];
        expected.ram[0x1ee..0x1f1].copy_from_slice(&[0x64, 0x83, 6]);
        expected.write_wram_byte(0x10, 7);

        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        if case / 88 == 5 {
            let k = 15 + case % 15;
            let mut slot = expected.garnish_slot_view_mut(k);
            let kind = 1 + (case % 22) as u8;
            slot.set_garnish_type(kind);
            slot.set_countdown(if kind == 16 { 16 } else { [0, 1, 3, 7][case / 22 % 4] });
            slot.set_oam_flags(if kind == 15 { 7 } else { 0 });
            slot.set_x_low(128);
            slot.set_y_low(96);
        }
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 9,
            pc: 0xb06e,
            sp: 0x1ed,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(52, 1166),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonGarnish::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterGarnish(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::Garnish,
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned garnish caller replay"
        );
    }
    println!(
        "native garnish handoff: 528 fixtures, palette and slot suffix match, caller work omitted"
    );
}

#[test]
fn native_follower_handoff_preserves_remaining_trigger_movement_and_dialogue_once() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_follower::DungeonFollower,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_follower::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0xf3cc] = [0, 5, 9, 10, 14, 1, 12, 13][case % 8];
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1; // A live item guard requires a valid rod selection for the remaining player draw.
        expected.ram[0x5d] = [0, 4, 17][case % 3];
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x8365,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(55, 1280),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonFollower::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = ZeldaState::native_follower_table_byte(a) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        ZeldaState::native_follower_table_byte(a + 1).unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterFollower(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterGarnish(
                crate::cycle_models::dungeon_garnish::GarnishTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned follower caller replay"
        );
    }
    println!(
        "native follower handoff: 384 fixtures, trigger movement and dialogue suffix match, caller work omitted"
    );
}

#[test]
fn native_ancilla_handoff_preserves_remaining_spark_effects_and_receipt_once() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_ancilla::DungeonAncilla,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1;
        expected.ram[0x5d] = 0;
        expected.ram[0xc4a..0xc54].fill(0);
        expected.ram[0xfac] = 0;
        if (64..128).contains(&case) {
            expected.ram[0xfac] = [1, 2, 3, 9, 255][case % 5];
            expected.ram[0xfaf] = [0, 1, 128, 255][case % 4];
            expected.ram[0xfad] = 128;
            expected.ram[0xfae] = 96;
            expected.ram[0xb68] = (case % 2) as u8;
        } else if case >= 128 {
            let k = case % 10;
            expected.ram[0xc4a + k] = [3, 6, 0x0a, 0x13, 0x16, 0x20, 0x21, 0x22][case / 32 % 8];
            expected.ram[0xc68 + k] = [0, 1, 3, 7][case % 4];
        }
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x8369,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(56, 946),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonAncilla::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterAncilla(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterFollower(
                crate::cycle_models::dungeon_follower::FollowerTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned ancilla caller replay"
        );
    }
    println!(
        "native ancilla handoff: 384 fixtures, spark effects and receipt suffix match, caller work omitted"
    );
}

#[test]
fn native_ancilla_receipt_drawing_handoff_preserves_complete_sprite_caller_once() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_ancilla::DungeonAncilla,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1;
        expected.ram[0x5d] = 0;
        expected.ram[0xc4a..0xc54].fill(0);
        expected.ram[0xfac] = 0;
        if (64..128).contains(&case) {
            expected.ram[0xfac] = [1, 2, 3, 9, 255][case % 5];
            expected.ram[0xfaf] = [0, 1, 128, 255][case % 4];
            expected.ram[0xfad] = 128;
            expected.ram[0xfae] = 96;
            expected.ram[0xb68] = (case % 2) as u8;
        } else if case >= 128 {
            let k = case % 10;
            expected.ram[0xc4a+k]=0x22;
            expected.ram[0xc54+k]=1;
            expected.ram[0x3b1+k]=56;
            expected.ram[0xc5e+k]=[1,0x20,0x34,0x35,0x36,9,0x17,0x42][case%8];
            expected.ram[0x3a4+k]=(case%3) as u8;
            expected.ram[0x39f+k]=0;
            expected.ram[0xc68 + k] = [0, 1, 3, 7][case % 4];
        }
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x8369,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(56, 946),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonAncilla::default().with_native_receipt(true);
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_receipt_animation_table_byte(a)) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterAncilla(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterFollower(
                crate::cycle_models::dungeon_follower::FollowerTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned ancilla caller replay"
        );
    }
    println!(
        "native ancilla receipt drawing caller: 384 fixtures, selected drawing and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_ancilla_receipt_oam_handoff_preserves_complete_sprite_caller_without_body_replay() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_ancilla::DungeonAncilla,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1;
        expected.ram[0x5d] = 0;
        expected.ram[0xc4a..0xc54].fill(0);
        expected.ram[0xfac] = 0;
        if (64..128).contains(&case) {
            expected.ram[0xfac] = [1, 2, 3, 9, 255][case % 5];
            expected.ram[0xfaf] = [0, 1, 128, 255][case % 4];
            expected.ram[0xfad] = 128;
            expected.ram[0xfae] = 96;
            expected.ram[0xb68] = (case % 2) as u8;
        } else if case >= 128 {
            let k = case % 10;
            expected.ram[0xc4a+k]=0x22;
            expected.ram[0xc54+k]=1;
            expected.ram[0x3b1+k]=56;
            expected.ram[0xc5e+k]=[1,0x20,0x34,0x35,0x36,9,0x17,0x42][case%8];
            expected.ram[0x3a4+k]=(case%3) as u8;
            expected.ram[0x39f+k]=0;
            expected.ram[0xc68 + k] = [0, 1, 3, 7][case % 4];
        }
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        // Slots6..9 retain the incoming drawing pointers rather than allocate them.
        expected.oam_state_mut().set_current_pointer(0x800);
        expected.oam_state_mut().set_current_extended_pointer(0xa20);
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x8369,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(56, 946),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonAncilla::default().with_native_receipt(true).with_native_oam(true);
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_receipt_animation_table_byte(a)).or_else(||crate::zelda_rtl::misc::native_item_receipt_table_byte(a)) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterAncilla(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterFollower(
                crate::cycle_models::dungeon_follower::FollowerTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        // $74 is the Source palette local, verified instruction by instruction.
        let excluded = |i: usize| i < 0x10 || i==0x74 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned ancilla caller replay"
        );
    }
    println!(
        "native ancilla receipt OAM caller: 384 fixtures, selected drawing and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_ancilla_slot_return_preserves_complete_sprite_caller_without_loop_replay() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_ancilla::DungeonAncilla,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1;
        expected.ram[0x5d] = 0;
        expected.ram[0xc4a..0xc54].fill(0);
        expected.ram[0xfac] = 0;
        if (64..128).contains(&case) {
            expected.ram[0xfac] = [1, 2, 3, 9, 255][case % 5];
            expected.ram[0xfaf] = [0, 1, 128, 255][case % 4];
            expected.ram[0xfad] = 128;
            expected.ram[0xfae] = 96;
            expected.ram[0xb68] = (case % 2) as u8;
        } else if case >= 128 {
            let k = case % 10;
            expected.ram[0xc4a+k]=0x22;
            expected.ram[0xc54+k]=1;
            expected.ram[0x3b1+k]=56;
            expected.ram[0xc5e+k]=[1,0x20,0x34,0x35,0x36,9,0x17,0x42][case%8];
            expected.ram[0x3a4+k]=(case%3) as u8;
            expected.ram[0x39f+k]=0;
            expected.ram[0xc68 + k] = [0, 1, 3, 7][case % 4];
        }
        if case >= 128 {
            crate::cycle_models::dungeon_ancilla::seed_slot_return_case(&mut expected.ram, case);
        }
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        // Slots6..9 retain the incoming drawing pointers rather than allocate them.
        expected.oam_state_mut().set_current_pointer(0x800);
        expected.oam_state_mut().set_current_extended_pointer(0xa20);
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x8369,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(56, 946),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonAncilla::default().with_native_receipt(true).with_native_oam(true).with_native_slot_return(true);
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_receipt_animation_table_byte(a)).or_else(||crate::zelda_rtl::misc::native_item_receipt_table_byte(a)) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterAncilla(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterFollower(
                crate::cycle_models::dungeon_follower::FollowerTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        // $74 is the Source palette local, verified instruction by instruction.
        let excluded = |i: usize| i < 0x10 || i==0x74 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned ancilla caller replay"
        );
    }
    println!(
        "native ancilla slot return caller: 384 fixtures, selected drawing and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_dungeon_overlord_handoff_preserves_complete_sprite_caller_without_header_replay() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_overlord::DungeonOverlord,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1;
        expected.ram[0x5d] = 0;
        expected.ram[0xc4a..0xc54].fill(0);
        expected.ram[0xfac] = 0;
        if (64..128).contains(&case) {
            expected.ram[0xfac] = [1, 2, 3, 9, 255][case % 5];
            expected.ram[0xfaf] = [0, 1, 128, 255][case % 4];
            expected.ram[0xfad] = 128;
            expected.ram[0xfae] = 96;
            expected.ram[0xb68] = (case % 2) as u8;
        } else if case >= 128 {
            let k = case % 10;
            expected.ram[0xc4a+k]=0x22;
            expected.ram[0xc54+k]=1;
            expected.ram[0x3b1+k]=56;
            expected.ram[0xc5e+k]=[1,0x20,0x34,0x35,0x36,9,0x17,0x42][case%8];
            expected.ram[0x3a4+k]=(case%3) as u8;
            expected.ram[0x39f+k]=0;
            expected.ram[0xc68 + k] = [0, 1, 3, 7][case % 4];
        }
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_overlord::seed_case(&mut image, case*17); expected.import_wram_image(&image); }
        for k in 0..8 { if expected.ram[0xb00+k]!=0 {expected.ram[0xb00+k]=1;} }
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        // Slots6..9 retain the incoming drawing pointers rather than allocate them.
        expected.oam_state_mut().set_current_pointer(0x800);
        expected.oam_state_mut().set_current_extended_pointer(0xa20);
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x8398,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(68, 1262),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonOverlord::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_receipt_animation_table_byte(a)).or_else(||crate::zelda_rtl::misc::native_item_receipt_table_byte(a)) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterOverlord(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterAncilla(
                crate::cycle_models::dungeon_ancilla::AncillaTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        // CPU scratch and real stacks are verified instruction by instruction.
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned ancilla caller replay"
        );
    }
    println!(
        "native dungeon overlord caller: 384 fixtures, selected slots and boulder and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_dungeon_sprite_slots_handoff_preserves_complete_sprite_caller_without_loop_replay() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_sprite_slots::DungeonSpriteSlots,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..392usize {
        let mut expected = ZeldaState::new();
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); expected.import_wram_image(&image); }
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x10, 7);
        expected.ram[0x307] = 1;
        expected.ram[0x5d] = 0;
        expected.ram[0xc4a..0xc54].fill(0);
        expected.ram[0xfac] = 0;
        if (64..128).contains(&case) {
            expected.ram[0xfac] = [1, 2, 3, 9, 255][case % 5];
            expected.ram[0xfaf] = [0, 1, 128, 255][case % 4];
            expected.ram[0xfad] = 128;
            expected.ram[0xfae] = 96;
            expected.ram[0xb68] = (case % 2) as u8;
        } else if case >= 128 {
            let k = case % 10;
            expected.ram[0xc4a+k]=0x22;
            expected.ram[0xc54+k]=1;
            expected.ram[0x3b1+k]=56;
            expected.ram[0xc5e+k]=[1,0x20,0x34,0x35,0x36,9,0x17,0x42][case%8];
            expected.ram[0x3a4+k]=(case%3) as u8;
            expected.ram[0x39f+k]=0;
            expected.ram[0xc68 + k] = [0, 1, 3, 7][case % 4];
        }
        { let mut image = expected.wram_image().into_owned(); crate::cycle_models::dungeon_sprite_slots::seed_case(&mut image, case); expected.import_wram_image(&image); }
        expected.write_wram_byte(0x11, 17); // Freeze active bodies; all eleven real states still dispatch.
        for k in 0..16 { expected.ram[0xe20+k]=0x73;expected.ram[0xef0+k]=0; }
        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        if case >= 384 {
            expected.ram[0xfb3] = 0;
            expected.ram[0xdd0..0xde0].fill(0);
            expected.write_wram_byte(0xfc1, 1);
            for slot in [0,14,15] {
                expected.ram[0xdd0+slot]=9;
                expected.ram[0xe20+slot]=if slot==15 {0x76} else {0x73};
                expected.ram[0xe90+slot]=if slot==14 {2} else if slot==0 {1} else {0};
                expected.ram[0xe40+slot]=if slot==15 {1} else if slot==14 {3} else {2};
                expected.ram[0xdc0+slot]=(case % 2) as u8;
                expected.ram[0xde0+slot]=(case / 2 % 2) as u8;
            }
        }
        expected.sync_native_game_state_from_ram();
        for k in 0..16 { expected.sprite_slot_view_mut(k).set_delay_main(16); }
        if (352..384).contains(&case) {
            // Enter Module07, then change modules inside the native message
            // callee. The already-entered caller must still finish its suffix.
            expected.set_submodule(0);
            expected.set_modal_pause_flag(0);
            expected.set_dark_world_region_index(0);
            for k in 0..16 { expected.sprite_slot_view_mut(k).set_state(0); }
            let k = case % 16;
            expected.sprite_slot_view_mut(k).set_state(9);
            expected.sprite_slot_view_mut(k).set_sprite_type(0x7a);
            expected.sprite_slot_view_mut(k).set_hit_timer(31);
            expected.sprite_slot_view_mut(k).set_health(0);
            expected.sprite_slot_view_mut(k).set_incoming_damage(10);
        }
        expected.oam_reset_region_bases();
        // Slots6..9 retain the incoming drawing pointers rather than allocate them.
        expected.oam_state_mut().set_current_pointer(0x800);
        expected.oam_state_mut().set_current_extended_pointer(0xa20);
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 15,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x83a1,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(69, 452),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonSpriteSlots::default().with_inactive_slot_bodies(if case >= 384 {(1<<15)|(1<<14)|1} else {0});
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_oam_table_byte(a)).or_else(||ZeldaState::native_receipt_animation_table_byte(a)).or_else(||crate::zelda_rtl::misc::native_item_receipt_table_byte(a)) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1).or_else(||ZeldaState::native_oam_table_byte(a + 1))
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterSlots(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterOverlord(
                crate::cycle_models::dungeon_overlord::OverlordTail::Returned,
            ),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        // CPU scratch and real stacks are verified instruction by instruction.
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned ancilla caller replay"
        );
    }
    println!(
        "native dungeon sprite slots caller: 392 fixtures, inactive and all active states and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_lower_garnish_handoff_preserves_countdown_allocation_and_complete_sprite_suffix_once() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_lower_garnish::DungeonLowerGarnish,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..660usize {
        let mut expected = ZeldaState::new();
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x11, 0);
        expected.ram[0xfb4]=u8::from(case>=60);
        expected.ram[0x1ee..0x1f1].fill(0);
        expected.write_wram_byte(0x10, 7);

        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        if case>=60 {
            let k=case%15;let kind=(case/15%22+1) as u8;
            let mut slot=expected.garnish_slot_view_mut(k);
            slot.set_garnish_type(kind);
            slot.set_countdown(if kind==13 {1}else if kind==16 {16}else{[0,1,3,7][case/330%4]});
            slot.set_oam_flags(if kind==15 {7}else{0});slot.set_x_low(128);slot.set_y_low(96);
        }
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x83aa,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(73, 278),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonLowerGarnish::default();
        let mut owned = 0;
        while work.tail().is_none() {
            owned += work
                .step(&mut cpu, &mut clock, |_, access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1)
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                })
                .unwrap();
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterLowerGarnish(work.tail().unwrap()),
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterSlots(crate::cycle_models::dungeon_sprite_slots::SpriteSlotsTail::LowerGarnish),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned garnish caller replay"
        );
    }
    println!(
        "native lower garnish handoff: 660 fixtures, countdown allocator dispatch and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_cached_sprites_handoff_preserves_cache_execution_restore_and_complete_suffix_once() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_cached_sprites::DungeonCachedSprites,
        dungeon_sprite_prefix::SpritePrefixTail, intro_clear::Access, nmi_stack::NativeNmiContext,
    };
    for case in 0..384usize {
        let mut expected = ZeldaState::new();
        expected.ram[0x1b] = 1;
        expected.write_wram_byte(0x11, [0,1,14,17][case%4]);
        expected.ram[0xffa] = [0,1,255][case/4%3];
        expected.ram[0xfb4]=0;
        expected.ram[0x1f1]=0;
        expected.ram[0x1f2..0x1f5].copy_from_slice(&[0x41,0x88,2]);
        expected.ram[0xaaa]=(case as u8).wrapping_mul(13);
        expected.ram[0x69e..0x6a2].copy_from_slice(&[7,9,11,13]);
        expected.ram[0x1ee..0x1f1].fill(0);
        expected.write_wram_byte(0x10, 7);

        let saved = crate::cycle_models::dungeon_sprite_scroll::SavedSpriteScroll {
            bg2_x: case as u16 * 113,
            bg2_y: case as u16 * 229,
            bg1_x: case as u16 * 71,
            bg1_y: case as u16 * 127,
        };
        for (at, word) in [
            (0x1fb, saved.bg2_x),
            (0x1f9, saved.bg2_y),
            (0x1f7, saved.bg1_x),
            (0x1f5, saved.bg1_y),
        ] {
            expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        for i in 0..256usize {
            let aux = ((i * 97 + case * 31) & 0x7fff) as u16;
            let main = ((i * 211 + case * 13) & 0x7fff) as u16;
            expected.ram[0xc300 + i * 2..0xc302 + i * 2].copy_from_slice(&aux.to_le_bytes());
            expected.ram[0xc500 + i * 2..0xc502 + i * 2].copy_from_slice(&main.to_le_bytes());
        }
        expected.sync_native_game_state_from_ram();
        expected.oam_reset_region_bases();
        // State 3 owns a valid falling sprite body. Pause keeps its countdown
        // stable; varied cached pause flags exercise cache-state clearing.
        let k=case/12%16;
        for (i,&base) in crate::game_state::constants::CACHED_SPRITE_LIVE_FIELDS.iter().enumerate() {
            expected.ram[base+k]=(i as u8).wrapping_mul(17).wrapping_add(case as u8);
        }
        expected.ram[0xdd0+k]=(case%11+1) as u8;
        expected.ram[0x1d00+k]=if case>=360 {0}else{3};
        expected.ram[0x1d10+k]=0;
        expected.ram[0x1d20+k]=128;
        expected.ram[0x1d40+k]=96;
        expected.ram[0xf00+k]=(case/192) as u8;
        expected.sync_native_game_state_from_ram();
        let mut actual = expected.clone();
        actual.rom.clear();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 3,
            y: 7,
            d: 0,
            db: 6,
            bank: 6,
            pc: 0x83ae,
            sp: 0x1f0,
            p: 0x30,
        };
        let mut clock = snes::CpuMasterTimeline::at_raster(
            4586,
            snes::CpuRasterPosition::new(76, 54),
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonCachedSprites::default();
        let mut owned = 0;
        let mut return_work: Option<crate::cycle_models::sprite_main::SpriteMain> = None;
        loop {
            let mut bus = |_:u64, access:Access| match access {
                    Access::Read(a, width) => {
                        if let Some(v) = crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_oam_table_byte(a)) {
                            u16::from(v)
                                | if width == 2 {
                                    u16::from(
                                        crate::cycle_models::sprite_main::table_byte(a + 1).or_else(||ZeldaState::native_oam_table_byte(a + 1))
                                            .unwrap(),
                                    ) << 8
                                } else {
                                    0
                                }
                        } else {
                            let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                                0x1ffff
                            } else {
                                0xffff
                            }) as usize;
                            u16::from(actual.read_wram_byte(a))
                                | if width == 2 {
                                    u16::from(actual.read_wram_byte(a + 1)) << 8
                                } else {
                                    0
                                }
                        }
                    }
                    Access::Write(a, v, width) => {
                        let a = (a & if matches!(a >> 16, 0x7e | 0x7f) {
                            0x1ffff
                        } else {
                            0xffff
                        }) as usize;
                        for (byte, value) in v.to_le_bytes()[..width as usize].iter().copied().enumerate() { actual.write_wram_byte(a + byte, value); }
                        actual.native_push_block_store(a, v, width);
                        0
                    }
                    Access::Vblank { .. } => 0,
                };
            if let Some(ret) = return_work.as_mut() {
                owned += ret.step(&mut cpu,&mut clock,&mut bus).unwrap();
                if ret.complete() {
                    assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,0x8842,0x1f4));
                    break;
                }
            } else {
                owned += work.step(&mut cpu,&mut clock,&mut bus).unwrap();
                match work.tail() {
                    Some(crate::cycle_models::dungeon_cached_sprites::CachedSpritesTail::Returned)=>
                        return_work=Some(crate::cycle_models::sprite_main::SpriteMain::after_cached()),
                    Some(_)=>break,
                    None=>{},
                }
            }
        }
        actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: if return_work.is_some() {SpritePrefixTail::Returned} else {SpritePrefixTail::AfterCachedSprites(work.tail().unwrap())},
        });
        expected.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::SpritePrefix {
            saved,
            tail: SpritePrefixTail::AfterLowerGarnish(crate::cycle_models::dungeon_lower_garnish::LowerGarnishTail::Returned),
        });
        let before = crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_push_block_handler(true);
        let full = crate::cycle_ledger::master() - before;
        let before = crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_push_block_handler(true);
        let remaining = crate::cycle_ledger::master() - before;
        let excluded = |i: usize| i < 0x10 || (0x1c0..0x200).contains(&i);
        let diffs = actual
            .wram_image()
            .iter()
            .zip(expected.wram_image().iter())
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} RAM {diffs:x?}");
        let mut a = vec![0xa5; snes::WRAM_SIZE];
        let mut b = a.clone();
        actual.game_state.write_to_ram(&mut a);
        expected.game_state.write_to_ram(&mut b);
        let diffs = a
            .iter()
            .zip(&b)
            .enumerate()
            .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
            .take(16)
            .collect::<Vec<_>>();
        assert!(diffs.is_empty(), "case={case} native {diffs:x?}");
        assert_eq!(
            full - remaining,
            owned,
            "case={case}: no owned cached caller replay"
        );
    }
    println!(
        "native cached sprites handoff: 384 fixtures, cached execution restore and complete sprite suffix match, caller work omitted"
    );
}

#[test]
fn native_sprite_timers_handoffs_preserve_message_recoil_and_module_change_once() {
    use crate::cycle_models::{sprite_timers::{SpriteTimers,SpriteTimersTail},intro_clear::Access,nmi_stack::NativeNmiContext};
    let mut tails=[0;3];
    for case in 0..384usize {
        let k=case%16;let mode=case/16%3;let st=[9,10,11][case/48%3];
        let mut expected=ZeldaState::new();
        expected.write_wram_byte(0x10, 7);expected.ram[0x1b]=1;expected.ram[0xfa0]=k as u8;
        expected.ram[0xdd0+k]=st;expected.ram[0xe20+k]=if mode==1 {0x7a}else{0x73};
        expected.ram[0xef0+k]=[0,31,24][mode];expected.ram[0xe50+k]=0;expected.ram[0xce2+k]=10;
        expected.ram[0xd10+k]=128;expected.ram[0xd00+k]=96;
        expected.ram[0xfb3]=(case/144%2) as u8;expected.ram[0xf20+k]=(case/72%2) as u8;
        expected.ram[0xee]=if case/96%2==0 {0}else{3};
        expected.ram[0xe40+k]=(case%32) as u8;
        for (i,a) in [0xdf0,0xe00,0xe10,0xee0,0xf10].into_iter().enumerate() {expected.ram[a+k]=[0,1,128,255][(case+i)%4];}
        expected.sync_native_game_state_from_ram();expected.oam_reset_region_bases();
        let mut actual=expected.clone();actual.rom.clear();
        let mut cpu=NativeNmiContext {a:0xab00|u16::from(st),x:k as u16,y:7,d:0,db:6,bank:6,pc:0x84e7,sp:0x1ee,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(76,54),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        clock.begin_synchronous_timeline().unwrap();
        let mut work=SpriteTimers::after_entry(st);let mut owned=0;
        while work.tail().is_none() {
            owned+=work.step(&mut cpu,&mut clock,|_,access|match access {
                Access::Read(a,width)=>{
                    let read=|a:u32|->u8 {
                        crate::cycle_models::sprite_main::table_byte(a).or_else(||ZeldaState::native_oam_table_byte(a)).unwrap_or_else(||actual.read_wram_byte((a&if matches!(a>>16,0x7e|0x7f) {0x1ffff}else{0xffff}) as usize))
                    };
                    u16::from(read(a))|if width==2 {u16::from(read(a+1))<<8}else{0}
                },
                Access::Write(a,v,width)=>{let a=(a&if matches!(a>>16,0x7e|0x7f) {0x1ffff}else{0xffff}) as usize;for byte in 0..width as usize { actual.write_wram_byte(a+byte,v.to_le_bytes()[byte]); }actual.native_push_block_store(a,v,width);0},
                Access::Vblank {..}=>0,
            }).unwrap();
        }
        let tail=work.tail().unwrap();assert_eq!(tail,[SpriteTimersTail::Returned,SpriteTimersTail::Returned,SpriteTimersTail::Recoil(crate::cycle_models::sprite_recoil::RecoilTail::Death)][mode]);tails[mode]+=1;
        if mode != 2 {
            assert_eq!((cpu.bank,cpu.pc,cpu.sp,cpu.db),(6,0x84eb,0x1ed,6));
            assert_eq!(actual.ram[0x1ee],st,"original state remains on the actual caller stack");
        }
        let before=crate::cycle_ledger::master();crate::cycle_ledger::charge(if case%2==0 {68}else{128});expected.sprite_timers_and_oam(k);let full=crate::cycle_ledger::master()-before;
        let before=crate::cycle_ledger::master();if case%2==0 {actual.finish_native_sprite_timers(k,tail);}else{
            actual.native_sprite_timer_handoff=Some((k as u8,st,tail));
            let _scope=actual.sprite_execute_single_lane_scope(k,false);
            actual.sprite_timers_and_oam(k);
            assert!(actual.native_sprite_timer_handoff.is_none());
        }let remaining=crate::cycle_ledger::master()-before;
        let excluded=|i:usize|i<0x10||(0x1c0..0x200).contains(&i);
        for (i,(a,b)) in actual.wram_image().iter().zip(expected.wram_image().iter()).enumerate() {assert!(excluded(i)||a==b,"case={case} RAM {i:05x}: {a:02x}/{b:02x}");}
        let mut a=vec![0xa5;snes::WRAM_SIZE];let mut b=a.clone();actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        for (i,(a,b)) in a.iter().zip(&b).enumerate() {assert!(excluded(i)||a==b,"case={case} native {i:05x}: {a:02x}/{b:02x}");}
        assert_eq!(full-remaining,owned,"case={case} no timer prefix replay");
    }
    println!("native sprite timer helper handoffs:384 fixtures, modes={tails:?},256 actual timer returns including128 message cases,128 retained recoil calls, module changes and all remaining helper work match");
}

#[test]
fn native_recoil_prefix_handoffs_preserve_health_damage_and_remaining_bodies_once() {
    use crate::cycle_models::{sprite_recoil::{SpriteRecoil,RecoilTail},intro_clear::Access,nmi_stack::NativeNmiContext};
    let mut tails=[0;4];
    for case in 0..1728usize {
        let k=case%16;let st=(case/16%12) as u8;
        let dmg=[0,10,250,251,253,254,255,1,252][case/192%9];
        let health=[0,10,255][case%3];
        let mut expected=ZeldaState::new();expected.write_wram_byte(0x10, 7);
        expected.ram[0xdd0+k]=st;expected.ram[0xce2+k]=dmg;expected.ram[0xe50+k]=health;
        expected.ram[0xe20+k]=if case%5==0 {0x23}else{0x73};
        expected.ram[0xd10+k]=128;expected.ram[0xd00+k]=96;
        expected.ram[0xe40+k]=(case%32) as u8;expected.ram[0xef0+k]=24;
        expected.ram[0x1ea..0x1ec].copy_from_slice(&[0x80,0x84]);
        expected.sync_native_game_state_from_ram();let mut actual=expected.clone();actual.rom.clear();
        let mut cpu=NativeNmiContext{a:0xabcd,x:k as u16,y:7,d:0,db:6,bank:6,pc:0xeec8,sp:0x1e9,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(76,1200),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);clock.begin_synchronous_timeline().unwrap();
        let mut work=SpriteRecoil::default();let mut owned=0;
        while work.tail().is_none(){owned+=work.step(&mut cpu,&mut clock,|_,access|match access{
            Access::Read(a,width)=>{let i=(a&0xffff) as usize;u16::from(actual.ram[i])|if width==2{u16::from(actual.ram[i+1])<<8}else{0}},
            Access::Write(a,v,width)=>{let i=(a&0xffff) as usize;actual.ram[i..i+width as usize].copy_from_slice(&v.to_le_bytes()[..width as usize]);actual.native_push_block_store(i,v,width);0},
            Access::Vblank{..}=>0,
        }).unwrap();}
        let tail=work.tail().unwrap();tails[match tail{RecoilTail::Returned=>0,RecoilTail::Burn=>1,RecoilTail::Stun(_)=>2,RecoilTail::Death=>3}]+=1;
        let before=crate::cycle_ledger::master();expected.sprite_mini_moldorm_recoil(k);let full=crate::cycle_ledger::master()-before;
        let before=crate::cycle_ledger::master();if tail!=RecoilTail::Returned{actual.finish_native_sprite_recoil(k,tail);}let remaining=crate::cycle_ledger::master()-before;
        let excluded=|i:usize|i<16||(0x1c0..0x200).contains(&i);
        for(i,(a,b))in actual.wram_image().iter().zip(expected.wram_image().iter()).enumerate(){assert!(excluded(i)||a==b,"recoil case={case} RAM {i:05x}: {a:02x}/{b:02x}");}
        let mut a=vec![0xa5;snes::WRAM_SIZE];let mut b=a.clone();actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        for(i,(a,b))in a.iter().zip(&b).enumerate(){assert!(excluded(i)||a==b,"recoil case={case} native {i:05x}: {a:02x}/{b:02x}");}
        assert_eq!(full-remaining,owned,"recoil case={case} actual owned prefix omitted once");
    }
    assert!(tails.iter().all(|n|*n>0));println!("native recoil semantic handoffs:1728 fixtures, tails={tails:?}, ROM empty, full remaining RAM/native state and owned cycles match");
}
