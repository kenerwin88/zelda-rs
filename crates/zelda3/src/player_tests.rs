use super::*;
use crate::game_state::constants::{
    ANCILLA_TYPE, BUTTON_B_FRAMES, LINK_DELAY_TIMER_SPIN_ATTACK, LINK_HANDLER_STATE,
    LINK_IS_BUNNY_MIRROR, LINK_IS_IN_DEEP_WATER,
};

fn player_caller_test_rom() -> Option<Vec<u8>> {
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let mut rom = std::fs::read(path).ok()?;
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    Some(rom)
}

fn player_caller_source(rom: &[u8], ram: &[u8], entry: u32) -> (u64, Vec<u8>) {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let main = entry == 0x07_8000;
    let end = match entry {
        0x07_8000 => 0x07_8020,
        0x07_8689 => 0x07_86b4,
        0x08_ee57 => 0x08_eedd,
        _ => unreachable!(),
    };
    let checkpoint = RomCpuCheckpoint {
        entry_pc: entry, stop_pc: (entry & 0xff0000) | 0x8036,
        a: 0, x: 0, y: 0, sp: if main { 0x01fb } else { 0x01fc }, dp: 0, db: (entry >> 16) as u8,
        carry: true, zero: false, overflow: true, negative: false,
        interrupt_disable: false, decimal: false,
        accumulator_is_8_bit: true, index_is_8_bit: true,
        emulation: false, waiting: false,
        stack_address: if main { 0x01fc } else { 0x01fd },
        stack_bytes: if main { &[0x35, 0x80, 7] } else { &[0x35, 0x80] },
    };
    let mut run = RomCpuTimingRun::new(rom, ram, &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = ram.to_vec();
    let mut cycles = 0;
    let mut instructions = 0;
    while !run.is_complete() {
        let pc = run.pc();
        let step = run.step();
        let own = (entry..=end).contains(&pc);
        if own { cycles += u64::from(step.master_cycles); }
        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
        instructions += 1;
        assert!(instructions < 100_000, "player caller did not return");
    }
    (cycles, expected)
}

#[test]
fn somaria_and_gravestone_scan_work_and_slot_order_match_rom() {
    let Some(rom) = player_caller_test_rom() else { return };
    for indoors in [0, 1] {
        for latch in [0, 1] {
            for submodule in [0, 1] {
                // Every assignment of empty, grave, and Somaria across five
                // slots covers skipped graves and first-match scan termination.
                for pattern in 0..243 {
                    let mut state = ZeldaState::new();
                    state.ram[0x1b] = indoors;
                    state.ram[0x3e9] = latch;
                    state.ram[0x11] = submodule;
                    state.ram[0xfa0] = 0xa5;
                    state.ram[0x2f] = 4;
                    let mut slots = pattern;
                    for k in 0..5 {
                        state.ram[0xc4a + k] = [0, 0x24, 0x2c][slots % 3];
                        slots /= 3;
                        state.ram[0x394 + k] = 1; // Somaria retains its active-object publication.
                        state.ram[0xbfa + k] = 100; // Graves move but do not expire.
                        state.ram[0xc04 + k] = 100;
                    }
                    state.sync_native_game_state_from_ram();
                    let (cycles, expected) = player_caller_source(&rom, &state.ram, 0x07_8689);
                    let actual = crate::cycle_ledger::measure_routine_self_work(0x07_8689,
                        || state.handle_somaria_and_graves());
                    assert_eq!(actual, cycles, "indoors={indoors} latch={latch} submodule={submodule} pattern={pattern}");
                    for range in [0xc4a..0xc4f, 0xbfa..0xbff, 0xc0e..0xc13, 0xc22..0xc27,
                        0xc36..0xc3b, 0x2f..0x30, 0x3e9..0x3ea, 0xfa0..0xfa1] {
                        assert_eq!(&state.ram[range.clone()], &expected[range.clone()],
                            "indoors={indoors} latch={latch} submodule={submodule} pattern={pattern} range={range:x?}");
                    }
                }
            }
        }
    }
}

#[test]
fn link_main_caller_work_and_saved_position_match_rom() {
    let Some(rom) = player_caller_test_rom() else { return };
    for immobilized in 0..=255 {
        for coordinate in [0u16, 0x7fff, 0xffff] {
            let mut state = ZeldaState::new();
            state.ram[0x2e4] = immobilized;
            state.ram[0xfc1] = 17;
            state.ram[0x20..0x22].copy_from_slice(&coordinate.to_le_bytes());
            state.ram[0x22..0x24].copy_from_slice(&coordinate.wrapping_add(1).to_le_bytes());
            state.sync_native_game_state_from_ram();
            let (cycles, expected) = player_caller_source(&rom, &state.ram, 0x07_8000);
            let actual = crate::cycle_ledger::measure_routine_self_work(0x07_8000,
                || state.link_main());
            assert_eq!(actual, cycles, "immobilized={immobilized} coordinate={coordinate}");
            assert_eq!(&state.ram[0xfc1..0xfc6], &expected[0xfc1..0xfc6]);
        }
    }
}

#[test]
fn gravestone_barrier_work_and_drag_state_owner_match_rom() {
    let Some(rom) = player_caller_test_rom() else { return };
    for coordinate in [0u16, 100, 0xfff0] {
        for dx in [-9i16, -8, 0, 15, 16, 23, 24] {
            for dy in [-9i16, -8, 0, 15, 16, 23, 24] {
                for facing in [0, 2, 4, 6] {
                    for vy in [0, 127, 128, 255] {
                        for vx in [0, 1] {
                            for flags in [0, 4, 255] {
                                let mut state = ZeldaState::new();
                                state.ram[0xbfa] = coordinate as u8;
                                state.ram[0xc0e] = (coordinate >> 8) as u8;
                                state.ram[0xc04] = coordinate as u8;
                                state.ram[0xc18] = (coordinate >> 8) as u8;
                                state.ram[0x20..0x22].copy_from_slice(&coordinate.wrapping_add(dy as u16).to_le_bytes());
                                state.ram[0x22..0x24].copy_from_slice(&coordinate.wrapping_add(dx as u16).to_le_bytes());
                                state.ram[0x2f] = facing;
                                state.ram[0x30] = vy;
                                state.ram[0x31] = vx;
                                state.ram[0x48] = flags;
                                state.ram[0x49] = 0xa5;
                                state.sync_native_game_state_from_ram();
                                let (cycles, expected) = player_caller_source(&rom, &state.ram, 0x08_ee57);
                                let before = crate::cycle_ledger::master();
                                state.gravestone_act_as_barrier(0);
                                assert_eq!(crate::cycle_ledger::master() - before, cycles,
                                    "coordinate={coordinate} dx={dx} dy={dy} facing={facing} vy={vy} vx={vx} flags={flags}");
                                for range in [0x20..0x24, 0x2f..0x32, 0x48..0x4a] {
                                    assert_eq!(&state.ram[range.clone()], &expected[range.clone()],
                                        "coordinate={coordinate} dx={dx} dy={dy} facing={facing} vy={vy} vx={vx} flags={flags} range={range:x?}");
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
fn link_splash_upon_landing_suppresses_recoil_other_deep_water_splash() {
    let mut state = ZeldaState::new();
    state.ram[LINK_HANDLER_STATE] = 6;
    state.ram[LINK_IS_IN_DEEP_WATER] = 1;
    state.ram[LINK_IS_BUNNY_MIRROR] = 0;
    state.sync_native_game_state_from_ram();

    state.link_splash_upon_landing();

    assert_eq!(state.ram[LINK_HANDLER_STATE], 4);
    assert_eq!(state.ram[ANCILLA_TYPE + 4], 0);
}

#[test]
fn hookshot_missing_ancilla_preserves_underflowed_spin_timer() {
    let mut state = ZeldaState::new();
    state.ram[LINK_HANDLER_STATE] = 19;
    state.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
    state.ram[BUTTON_B_FRAMES] = 10;
    state.sync_native_game_state_from_ram();

    state.link_state_hookshotting();

    assert_eq!(state.ram[LINK_HANDLER_STATE], 0);
    assert_eq!(state.ram[LINK_DELAY_TIMER_SPIN_ATTACK], 0xff);
    assert_eq!(state.ram[BUTTON_B_FRAMES], 9);
}

#[test]
fn native_shield_dma_word_store_publishes_the_adjacent_staging_byte() {
    let mut state = ZeldaState::new();
    state.rom.clear();
    state.ram[0x107..0x10b].copy_from_slice(&[0x17, 0x28, 0x39, 0x4a]);
    state.sync_native_game_state_from_ram();
    // Live LinkOam $0d:ac39 stores A=$5002 with M=16 at $0108.
    assert!(state.native_player_drawing_store(0x108, 0x5002, 2));
    assert_eq!(&state.ram[0x107..0x10b], &[0x17, 2, 0x50, 0x4a]);
    let mut published = state.ram.clone();
    published[0x108..0x10a].fill(0);
    state.game_state.write_to_ram(&mut published);
    assert_eq!(&published[0x107..0x10b], &[0x17, 2, 0x50, 0x4a]);
    assert!(state.native_player_drawing_store(0x109, 0xc9, 1));
    state.game_state.write_to_ram(&mut published);
    assert_eq!(&published[0x107..0x10b], &[0x17, 2, 0xc9, 0x4a]);
    // Other LinkOam branches store a word at109, including the neighboring
    // game-over byte. Both typed domains must serialize the retired word.
    state.ram[0x10b] = 0x63;
    assert!(state.native_player_drawing_store(0x109, 0xa541, 2));
    state.game_state.write_to_ram(&mut published);
    assert_eq!(&published[0x107..0x10b], &[0x17, 2, 0x41, 0xa5]);
    assert_eq!(state.ram[0x10b], 0x63);
}
