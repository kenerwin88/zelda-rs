use super::*;
use crate::game_state::constants::{
    ANCILLA_TYPE, BUTTON_B_FRAMES, LINK_DELAY_TIMER_SPIN_ATTACK, LINK_HANDLER_STATE,
    LINK_IS_BUNNY_MIRROR, LINK_IS_IN_DEEP_WATER,
};

#[test]
fn prefix11_lamp_caller_all_paths_match_rom() {
    for door in [0, 1] {
        for incapacitated in [0, 1] {
            for pressed in [0, 0x40] {
                for mask in [0, 0x40] {
                    for torch in [0, 1] {
                        for magic in [0, 1, 128] {
                            let mut n = ZeldaState::new();
                            for (a, v) in [(0x6c, door), (0x46, incapacitated),
                                (0xf4, pressed), (0x3a, mask), (0x303, 9), (0x304, 9),
                                (0xf34a, torch), (0xf36e, magic)] {
                                n.write_wram_byte(a, v);
                            }
                            n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                            let s = marker36_player_machine(&n, 0x07a24d).expect("unmodified ROM");
                            crate::cycle_ledger::assert_rom_marker_work(0x079b0e, s,
                                |s| s.cpu.k == 7 && (0xa24d..0xa289).contains(&s.cpu.pc), || {
                                    let _scope = crate::cycle_ledger::routine(0x079b0e);
                                    n.link_item_lamp();
                                });
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn prefix11_sword_swing_caller_all_paths_match_rom() {
    for action in [0, 0x10] {
        for mask in [0, 1, 0x80, 0x81] {
            for pressed in [0, 0x80] {
                for down in [0, 0x80] {
                    for timer in [0, 1, 0x80, 0xff] {
                        for frames in [0, 3, 4, 8, 9] {
                            let mut n = ZeldaState::new();
                            for (a, v) in [(0x3b, action), (0x3a, mask), (0xf4, pressed),
                                (0xf0, down), (0x3d, timer), (0x3c, frames), (0x1b, 1)] {
                                n.write_wram_byte(a, v);
                            }
                            n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                            let s = marker36_player_machine(&n, 0x079cd9).expect("unmodified ROM");
                            let sfx_tail = std::cell::Cell::new(false);
                            crate::cycle_ledger::assert_rom_marker_work(0x079cd9, s,
                                |s| {
                                    if s.cpu.k == 7 && s.cpu.sp == 0x1fa && s.cpu.pc == 0x9c66 { sfx_tail.set(true); }
                                    s.cpu.k == 7 && s.cpu.sp == 0x1fa
                                        && ((s.cpu.pc == 0x9cbe && !sfx_tail.get())
                                            || (0x9cd9..0x9d72).contains(&s.cpu.pc))
                                }, || n.link_check_for_sword_swing());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn prefix11_z_acceleration_all_paths_match_rom() {
    for state in [0, 2, 5, 19] {
        for velocity in [0, 1, 0x7f, 0x80, 0xff] {
            for z in [0, 1, 0x7f, 0x80, 0xff] {
                let mut n = ZeldaState::new();
                for (a, v) in [(0x5d, state), (0x29, velocity), (0x24, z), (0x25, 0x12)] {
                    n.write_wram_byte(a, v);
                }
                n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                let s = marker36_player_machine(&n, 0x078926).expect("unmodified ROM");
                let source = crate::cycle_ledger::assert_rom_marker_work(0x078926, s,
                    |s| s.cpu.k == 7 && (0x8926..0x894e).contains(&s.cpu.pc),
                    || n.link_handle_change_in_z_velocity());
                for a in [0x24, 0x25, 0x29] { assert_eq!(n.wram_image()[a], source.snes().ram[a]); }
            }
        }
    }
}

#[test]
fn prefix11_direction_and_recoil_all_paths_match_rom() {
    for y in [0, 1, 0x7f, 0x80, 0xff] {
        for x in [0, 1, 0x7f, 0x80, 0xff] {
            for diagonal in [0, 1, 4, 5, 15] {
                for state in [0, 2] {
                    for door in [0, 1, 2, 3] {
                        for entry in [0x0792a0, 0x07e1be] {
                            let mut n = ZeldaState::new();
                            for (a, v) in [(0x27, y), (0x28, x), (0x6b, diagonal),
                                (0x5d, state), (0x6c, door)] { n.write_wram_byte(a, v); }
                            n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                            let s = marker36_player_machine(&n, entry).expect("unmodified ROM");
                            let source = crate::cycle_ledger::assert_rom_marker_work(entry, s,
                                |s| s.cpu.k == 7 && s.cpu.sp == 0x1fa && if entry == 0x0792a0 {
                                    (0x92a0..0x92bf).contains(&s.cpu.pc)
                                } else { (0xe1be..0xe227).contains(&s.cpu.pc) }, || {
                                    if entry == 0x0792a0 { n.flag67_with_directions(); }
                                    else { n.link_handle_recoiling(); }
                                });
                            for a in [0x26, 0x27, 0x28, 0x67] {
                                assert_eq!(n.wram_image()[a], source.snes().ram[a], "entry={entry:x} addr={a:x}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn coverage3_toss_and_y_press_all_paths_match_rom() {
    for entry in [0x07b5d6, 0x07b073] {
        for first in [0, 0x40, 0x80, 0xff] {
            for second in [0, 1, 0x40, 0x80, 0xff] {
                for third in [0, 1, 0x40, 0x80, 0xff] {
                    for item in [0, 1, 2] {
                        let mut native = ZeldaState::new();
                        let fields = if entry == 0x07b5d6 { [0x3b, 0xf6, 0x309] }
                            else { [0x3a, 0x46, 0xf4] };
                        for (a, v) in fields.into_iter().zip([first, second, third]) {
                            native.write_wram_byte(a, v);
                        }
                        native.write_wram_byte(0x305, item);
                        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
                        let machine = marker36_player_machine(&native, entry).expect("unmodified ROM required");
                        crate::cycle_ledger::assert_rom_marker_work(entry, machine,
                            |s| s.cpu.k == 7 && if entry == 0x07b5d6 {
                                (0xb5d6..0xb609).contains(&s.cpu.pc)
                            } else { (0xb073..0xb087).contains(&s.cpu.pc) }, || {
                                if entry == 0x07b5d6 { native.link_handle_toss(); }
                                else { native.check_y_button_press(); }
                            });
                    }
                }
            }
        }
    }
}

#[test]
fn coverage3_bunny_all_paths_match_rom() {
    for timer in [0u16, 1, 0x100] {
        for poof in [0, 1] {
            for handler in [0, 23, 28] {
                for picking in [0, 2] {
                    for countdown in [0, 1, 0x80, 0x81] {
                        let mut native = ZeldaState::new();
                        native.write_wram_word(0x3f5, timer);
                        for (a, v) in [(0x3f7, poof), (0x5d, handler), (0x309, picking),
                            (0x2e2, countdown), (0xc4a, 0x30), (0xc4b, 0x31), (0xc4c, 1)] {
                            native.write_wram_byte(a, v);
                        }
                        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
                        let machine = marker36_player_machine(&native, 0x0782da).expect("unmodified ROM required");
                        crate::cycle_ledger::assert_rom_marker_work(0x0782da, machine,
                            |s| s.cpu.k == 7 && (0x82d8..0x8365).contains(&s.cpu.pc), || {
                                native.link_handle_bunny_transformation();
                            });
                    }
                }
            }
        }
    }
}

#[test]
fn coverage3_cape_all_paths_match_rom() {
    for cape in [0, 1] {
        for active in [0, 19] {
            for selected in [0, 19] {
                for countdown in [0, 1, 2] {
                    for magic in [0, 1, 2] {
                        for level in 0..3 {
                            let mut native = ZeldaState::new();
                            for (a, v) in [(0x55, cape), (0x304, active), (0x303, selected),
                                (0x4c, countdown), (0xf36e, magic), (0xf37b, level)] {
                                native.write_wram_byte(a, v);
                            }
                            native.sync_native_game_state_from_ram(); native.set_lag_free(true);
                            let machine = marker36_player_machine(&native, 0x07ae8f).expect("unmodified ROM required");
                            crate::cycle_ledger::assert_rom_marker_work(0x07ae8f, machine,
                                |s| s.cpu.k == 7 && (s.cpu.pc == 0xae87 || (0xae8f..0xaec0).contains(&s.cpu.pc)), || {
                                    native.player_check_handle_cape_stuff();
                                });
                        }
                    }
                }
            }
        }
    }
    for lifting in [0, 0x80] {
        let mut native = ZeldaState::new();
        native.write_wram_byte(0x308, lifting);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let machine = marker36_player_machine(&native, 0x07ae88).expect("unmodified ROM required");
        crate::cycle_ledger::assert_rom_marker_work(0x07ae88, machine,
            |s| s.cpu.k == 7 && ((0xae88..0xae8f).contains(&s.cpu.pc) || (lifting == 0 && s.cpu.pc == 0xae87)), || {
                native.link_handle_cape_passive_lift_check();
            });
    }
}

fn marker36_player_machine(native: &ZeldaState, entry: u32) -> Option<snes::Snes> {
    let rom = player_caller_test_rom()?;
    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&native.wram_image());
    machine.ram[0x1fb..0x200].copy_from_slice(&[0xff, 0x7f, 0xff, 0x7f, 0]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0x30);
    machine.cpu.k = (entry >> 16) as u8;
    machine.cpu.db = (entry >> 16) as u8;
    machine.cpu.pc = entry as u16;
    machine.cpu.sp = 0x1fa;
    Some(machine)
}

#[test]
fn marker36_moving_background_velocity_matches_rom() {
    for collision in [0u8, 1, 2] {
        for direction in [0u8, 1, 15] {
            for (x, y) in [(0u16, 0xffffu16), (0xff, 0x100), (0x1234, 0x5678)] {
                let mut native = ZeldaState::new();
                native.write_wram_byte(0x46c, collision);
                native.write_wram_byte(0x67, direction);
                native.write_wram_word(0x22, x); native.write_wram_word(0x20, y);
                native.write_wram_word(0x31a, 0xff); native.write_wram_word(0x318, 0x100);
                native.write_wram_word(0xe0, 0xfff0); native.write_wram_word(0xe2, 0x100);
                native.write_wram_word(0xe6, 0x7fff); native.write_wram_word(0xe8, 0x8000);
                native.sync_native_game_state_from_ram(); native.set_lag_free(true);
                let Some(machine) = marker36_player_machine(&native, 0x07b9b3) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x07b9b3, machine,
                    |s| s.cpu.k == 7 && (0xb9b3..0xb9f7).contains(&s.cpu.pc), || {
                        native.create_velocity_from_moving_background(MovingBackgroundEntry::Rom07B9B3);
                    });
            }
        }
    }
}

#[test]
fn marker36_sword_cooldown_matches_rom() {
    for timer in [0u8, 1, 0x7f, 0x80, 0xff] {
        for (item, position, frames, running) in [(0u8, 0u8, 0u8, 0u8),
            (1, 0, 0, 0), (0, 1, 0, 0), (0, 0, 8, 1), (0, 0, 9, 0)] {
            let mut native = ZeldaState::new();
            native.write_wram_byte(0x2e3, timer);
            native.write_wram_byte(0x301, item); native.write_wram_byte(0x37a, position);
            native.write_wram_byte(0x3c, frames); native.write_wram_byte(0x372, running);
            native.sync_native_game_state_from_ram(); native.set_lag_free(true);
            let Some(machine) = marker36_player_machine(&native, 0x079ac2) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x079ac2, machine,
                |s| s.cpu.k == 7 && (0x9ac2..0x9ae6).contains(&s.cpu.pc), || {
                    native.link_handle_sword_cooldown();
                });
        }
    }
}

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
                    state.write_wram_byte(0x11, submodule);
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
                    let (cycles, expected) = player_caller_source(&rom, &state.wram_image(), 0x07_8689);
                    let actual = crate::cycle_ledger::measure_routine_self_work(0x07_8689,
                        || state.handle_somaria_and_graves());
                    assert_eq!(actual, cycles, "indoors={indoors} latch={latch} submodule={submodule} pattern={pattern}");
                    for range in [0xc4a..0xc4f, 0xbfa..0xbff, 0xc0e..0xc13, 0xc22..0xc27,
                        0xc36..0xc3b, 0x2f..0x30, 0x3e9..0x3ea, 0xfa0..0xfa1] {
                        assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()],
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
            state.write_wram_byte(0xfc1, 17);
            state.ram[0x20..0x22].copy_from_slice(&coordinate.to_le_bytes());
            state.ram[0x22..0x24].copy_from_slice(&coordinate.wrapping_add(1).to_le_bytes());
            state.sync_native_game_state_from_ram();
            let (cycles, expected) = player_caller_source(&rom, &state.wram_image(), 0x07_8000);
            let actual = crate::cycle_ledger::measure_routine_self_work(0x07_8000,
                || state.link_main());
            assert_eq!(actual, cycles, "immobilized={immobilized} coordinate={coordinate}");
            assert_eq!(&state.wram_image()[0xfc1..0xfc6], &expected[0xfc1..0xfc6]);
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
                                let (cycles, expected) = player_caller_source(&rom, &state.wram_image(), 0x08_ee57);
                                let before = crate::cycle_ledger::master();
                                state.gravestone_act_as_barrier(0);
                                assert_eq!(crate::cycle_ledger::master() - before, cycles,
                                    "coordinate={coordinate} dx={dx} dy={dy} facing={facing} vy={vy} vx={vx} flags={flags}");
                                for range in [0x20..0x24, 0x2f..0x32, 0x48..0x4a] {
                                    assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()],
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

    assert_eq!(state.read_wram_byte(LINK_HANDLER_STATE), 4);
    assert_eq!(state.read_wram_byte(ANCILLA_TYPE + 4), 0);
}

#[test]
fn hookshot_missing_ancilla_preserves_underflowed_spin_timer() {
    let mut state = ZeldaState::new();
    state.ram[LINK_HANDLER_STATE] = 19;
    state.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
    state.ram[BUTTON_B_FRAMES] = 10;
    state.sync_native_game_state_from_ram();

    state.link_state_hookshotting();

    assert_eq!(state.read_wram_byte(LINK_HANDLER_STATE), 0);
    assert_eq!(state.read_wram_byte(LINK_DELAY_TIMER_SPIN_ATTACK), 0xff);
    assert_eq!(state.read_wram_byte(BUTTON_B_FRAMES), 9);
}

#[test]
fn native_shield_dma_word_store_publishes_the_adjacent_staging_byte() {
    let mut state = ZeldaState::new();
    state.rom.clear();
    let mut image = state.wram_image().into_owned();
    image[0x107..0x10b].copy_from_slice(&[0x17, 0x28, 0x39, 0x4a]);
    state.import_wram_image(&image);
    state.ram[0x10a] = 0x5a;
    state.sync_native_game_state_from_ram();
    // Live LinkOam $0d:ac39 stores A=$5002 with M=16 at $0108.
    assert!(state.native_player_drawing_store(0x108, 0x5002, 2));
    assert_eq!(&state.wram_image()[0x107..0x10b], &[0x17, 2, 0x50, 0x4a]);
    let mut published = state.wram_image().into_owned();
    published[0x108..0x10a].fill(0);
    state.game_state.write_to_ram(&mut published);
    state.game_state.system_signals.encode_into_image(&mut published);
    assert_eq!(&published[0x107..0x10b], &[0x17, 2, 0x50, 0x4a]);
    assert!(state.native_player_drawing_store(0x109, 0xc9, 1));
    state.game_state.write_to_ram(&mut published);
    state.game_state.system_signals.encode_into_image(&mut published);
    assert_eq!(&published[0x107..0x10b], &[0x17, 2, 0xc9, 0x4a]);
    // Other LinkOam branches store a word at109, including the neighboring
    // game-over byte. Both typed domains must serialize the retired word.
    state.ram[0x10b] = 0x63;
    assert!(state.native_player_drawing_store(0x109, 0xa541, 2));
    state.game_state.write_to_ram(&mut published);
    state.game_state.system_signals.encode_into_image(&mut published);
    assert_eq!(&published[0x107..0x10b], &[0x17, 2, 0x41, 0xa5]);
    assert_eq!(state.ram[0x10b], 0x63);
    assert_eq!(state.ram[0x10a], 0x5a, "owned game-over byte must stay unpublished");
}

#[test]
fn marker37_y_item_matches_rom() {
    for (frames, bunny, game, item, active, hand, cape, flute) in [
        (1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8), (9,0,0,0,0,0,0,0),
        (0,1,0,1,0,0,0,0), (0,1,0,11,0,0,0,0), (0,1,0,20,0,0,0,0),
        (0,0,1,0,0,0,0,0), (0,0,2,0,0,0,0,0), (0,1,2,11,0,0,0,0),
        (0,0,0,5,8,0,0,2), (0,0,0,6,8,0,0,0), (0,0,0,1,19,0,0,0),
        (0,0,0,0,5,1,0,0), (0,0,0,0,6,1,0,0), (0,0,0,0,0,1,0,0),
    ] {
        let mut native = ZeldaState::new();
        for (a,v) in [(0x3c,frames),(0x2e0,bunny),(0x3fc,game),(0x303,item),
            (0x304,active),(0x301,hand),(0x55,cape),(0xf34c,flute)] {
            native.write_wram_byte(if a==0xf34c {0x1f34c}else{a},v);
        }
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker36_player_machine(&native, 0x079b0e) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x079b0e, machine,
            |s| s.cpu.k==7 && ((0x9b0e..0x9b92).contains(&s.cpu.pc) || s.cpu.pc==0x9ae5), || {
                native.link_handle_y_item_marked();
            });
    }
}

#[test]
fn marker37_a_press_matches_rom() {
    for (hand, position, pose, frames, mask, action_state, grab, pull, statue, pickup, new_button) in [
        (1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8),
        (0,1,0,0,0,0,0,0,0,0,0), (0,0,1,0,0,0,0,0,0,0,0),
        (0,0,0,8,0x80,0,0,0,0,0,0), (0,0,0,9,0x80,0,0,0,0,0,0),
        (0,0,0,0,0,0,0,0,0,0,0), (0,0,0,0,0,1,0,0,0,0,0),
        (0,0,0,0,0,0,1,0,0,0,0), (0,0,0,0,0,0,0,1,0,0,0x80),
        (0,0,0,0,0,0,0,0,1,0,0x80), (0,0,0,0,0,0,0,0,0,1,0x80),
    ] {
        let mut native=ZeldaState::new();
        for (a,v) in [(0x301,hand),(0x37a,position),(0x379,pose),(0x3c,frames),(0x3a,mask),
            (0x308,action_state),(0x376,grab),(0x3f8,pull),(0x2fa,statue),(0x2ec,pickup),(0xf6,new_button)] {
            native.write_wram_byte(a,v);
        }
        native.sync_native_game_state_from_ram();native.set_lag_free(true);
        let Some(machine)=marker36_player_machine(&native,0x079baa)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x079baa,machine,
            |s|s.cpu.k==7 && (0x9baa..0x9c4f).contains(&s.cpu.pc),||{native.link_handle_a_press_marked();});
    }
}

#[test]
fn marker37_swimming_reset_matches_rom() {
    let mut native=ZeldaState::new();
    native.write_wram_byte(0x2cb,0xff); native.write_wram_byte(0x34f,0xff);
    native.write_wram_byte(0x32a,0xff);
    native.sync_native_game_state_from_ram();native.set_lag_free(true);
    let Some(machine)=marker36_player_machine(&native,0x07983a)else{return};
    crate::cycle_ledger::assert_rom_marker_work(0x07983a,machine,
        |s|s.cpu.k==7 && (0x983a..0x984b).contains(&s.cpu.pc),||{native.link_reset_swimming_state_marked();});
}

#[test]
fn marker38_slope_wrappers_match_rom() {
    use crate::game_state::CollisionOrder;
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
    let Ok(rom)=std::fs::read(path)else{return};
    for order in [CollisionOrder::VerticalFirst,CollisionOrder::HorizontalFirst] {
        for flags in [0u8,0x10,0x20,0x30] {
            let mut native=ZeldaState::new();
            for(a,v)in[(0x1b,1),(0x10,7),(0x20,0x40),(0x22,0x40),(0x6b,flags)] {native.write_wram_byte(a,v);}
            native.sync_native_game_state_from_ram();native.set_lag_free(true);
            let entry=if order==CollisionOrder::VerticalFirst{0x07b956}else{0x07b969};
            let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
            machine.ram.copy_from_slice(&native.wram_image());machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
            machine.cpu.e=false;machine.cpu.unpack_flags(0x30);machine.cpu.k=7;machine.cpu.db=7;machine.cpu.pc=entry as u16;machine.cpu.sp=0x1fc;
            crate::cycle_ledger::assert_rom_marker_work(entry,machine,
                |s|s.cpu.k==7 && (entry as u16..entry as u16+19).contains(&s.cpu.pc),|| {
                    native.run_slope_collision_checks_marked(order);
                });
        }
    }
}

#[test]
fn marker38_diagonal_kickback_matches_rom() {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
    let Ok(rom)=std::fs::read(path)else{return};
    for (tile,vx,vy) in [(0u8,0u8,1u8),(0,1,0),(0,1,1),(0x10,1,1),
        (0x10,255,255),(0x11,1,255),(0x12,255,1),(0x13,1,1)] {
        let mut native=ZeldaState::new(); let mut image=native.wram_image().into_owned();
        image[0x12000..0x14000].fill(tile);
        for(a,v)in[(0x1b,1),(0x10,7),(0x20,0x40),(0x22,0x40),(0x30,vy),(0x31,vx)] {image[a]=v;}
        native.import_wram_image(&image);native.sync_native_game_state_from_ram();native.set_lag_free(true);
        let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
        machine.ram.copy_from_slice(&native.wram_image());machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
        machine.cpu.e=false;machine.cpu.unpack_flags(0x30);machine.cpu.k=7;machine.cpu.db=7;machine.cpu.pc=0xccab;machine.cpu.sp=0x1fc;
        crate::cycle_ledger::assert_rom_marker_work(0x07ccab,machine,
            |s|s.cpu.k==7 && (0xccab..0xcd7b).contains(&s.cpu.pc),|| {native.link_handle_diagonal_kickback();});
    }
}

#[test]
fn marker39_ground_control_body_matches_rom() {
    // Branch cases: idle/moving, locked action, pulling, item and incapacitation.
    for changes in [vec![], vec![(0xf0,1)], vec![(0xf0,4),(0x26,4)],
        vec![(0x308,1)], vec![(0x376,1)], vec![(0x377,1)],
        vec![(0x2e1,1)], vec![(0x301,1)], vec![(0x37a,1)],
        vec![(0x308,0x80),(0x309,1)], vec![(0x46,2)],
        vec![(0x46,2),(0x6b,1),(0x3a,0x80)], vec![(0x49,8)]] {
        let mut native = ZeldaState::new();
        native.write_wram_word(0x20,0x80); native.write_wram_word(0x22,0x80);
        for (a,v) in changes { native.write_wram_byte(a,v); }
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine)=marker36_player_machine(&native,0x0781a0) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x07807f,machine,
            |s|s.cpu.k==7 && ((0x81a0..0x82da).contains(&s.cpu.pc)
                || (0x8711..0x8723).contains(&s.cpu.pc)
                || (0x880d..0x8872).contains(&s.cpu.pc)), || {
                let _parent=crate::cycle_ledger::routine(0x07807f);
                native.player_handler_00_ground_3();
            });
    }
}

#[test]
fn marker39_main_tile_response_matches_rom() {
    for changes in [vec![], vec![(0x357,1)], vec![(0x357,1),(0x4d,1)],
        vec![(0x357,0x10),(0x31f,1)], vec![(0x359,1)],
        vec![(0x359,1),(0x345,1),(0x1b,0)],
        vec![(0x341,1),(0x1b,0)], vec![(0x2ee,1)],
        vec![(0x2ee,0x10),(0x55,1)], vec![(0x348,1)],
        vec![(0x348,0x10),(0x67,9)], vec![(0x348,1),(0x34a,1)],
        vec![(0x348,1),(0x34a,1),(0x6a,1)], vec![(0x2e8,0x10)]] {
        let mut native = ZeldaState::new();
        native.write_wram_byte(0x1b,1);
        for (a,v) in changes { native.write_wram_byte(a,v); }
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine)=marker36_player_machine(&native,0x07d104) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x07d077,machine,
            |s|s.cpu.k==7 && (0xd104..0xd2c6).contains(&s.cpu.pc), || {
                let _parent=crate::cycle_ledger::routine(0x07d077);
                native.finish_main_tile_response_body(true);
            });
    }
}

#[test]
fn bypass1_new_a_press_matches_rom_all_branches() {
    for flags in [0,0x10,0x80,0xff] {
        for incapacitated in [0,1,0xff] {
            for joy in [0,0x80,0xff] {
                let mut n=ZeldaState::new(); n.set_lag_free(true);
                n.write_wram_byte(0x3b,flags);
                n.write_wram_byte(0x46,incapacitated);
                n.write_wram_byte(0xf6,joy);
                n.sync_native_game_state_from_ram();
                let Some(s)=marker36_player_machine(&n,0x07b5c0)else{panic!("development ROM required")};
                let mut result=false;
                let source=crate::cycle_ledger::assert_rom_marker_work(0x07b5c0,s,
                    |s|s.cpu.k==7 && (0xb5c0..0xb5d6).contains(&s.cpu.pc),||result=n.link_check_new_a_press());
                assert_eq!(result,source.snes().cpu.c);
                assert_eq!(n.wram_image()[0x3b],source.snes().ram[0x3b]);
            }
        }
    }
}


#[test]
fn prefix11_sword_controls_and_hold_all_paths_match_rom() {
    for defense in [0, 1, 8, 9, 0x80] {
        for damage in [0, 1, 2] {
            for speed in [0, 4, 16] {
                for step in [0, 5, 6, 47, 48, 63, 64] {
                    for obstruction in [0, 0x30, 0x31] {
                        for entry in [0x079d72, 0x079d9f, 0x079d84] {
                            let mut n = ZeldaState::new();
                            for (a, v) in [(0x48, defense), (0x47, damage), (0x5e, speed),
                                (0x79, step), (0xc4a, obstruction), (0xf0, 0x80),
                                (0x3c, 12), (0x1b, 1), (0x2f, 2)] { n.write_wram_byte(a, v); }
                            n.write_wram_byte(0xf359, 1);
                            n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                            let s = marker36_player_machine(&n, entry).expect("unmodified ROM");
                            let reset_tail = std::cell::Cell::new(false);
                            crate::cycle_ledger::assert_rom_marker_work(entry, s,
                                |s| {
                                    if entry != 0x079d84 && s.cpu.k == 7 && s.cpu.sp == 0x1fa && s.cpu.pc == 0x9d84 {
                                        reset_tail.set(true);
                                    }
                                    s.cpu.k == 7 && s.cpu.sp == 0x1fa && match entry {
                                    0x079d72 => (0x9d72..0x9d84).contains(&s.cpu.pc),
                                    0x079d84 => (0x9d84..0x9d9f).contains(&s.cpu.pc) || s.cpu.pc == 0x9e62,
                                    _ => (0x9d9f..0x9e63).contains(&s.cpu.pc)
                                        && !(s.cpu.pc == 0x9e62 && reset_tail.get()),
                                    }
                                }, || match entry {
                                    0x079d72 => n.handle_sword_controls(),
                                    0x079d84 => n.link_reset_sword_and_item_usage(),
                                    _ => n.player_sword_spin_attack_jerks_hold_down(),
                                });
                        }
                    }
                }
            }
        }
    }
}


#[test]
fn prefix11_boomerang_caller_all_paths_match_rom() {
    for mask in [0, 0x40, 0xc0] {
        for hand in [0, 0x80] {
            for door in [0, 1] {
                for pressed in [0, 0x40] {
                    for timer in [0, 1, 0x80] {
                        for step in [0, 1, 2] {
                            let mut n = ZeldaState::new();
                            for (a, v) in [(0x3a, mask), (0x301, hand), (0x6c, door),
                                (0xf4, pressed), (0x3d, timer), (0x300, step)] { n.write_wram_byte(a, v); }
                            n.write_wram_byte(0xf341, 1);
                            n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                            let s = marker36_player_machine(&n, 0x07a0bb).unwrap();
                            crate::cycle_ledger::assert_rom_marker_work(0x079b0e, s,
                                |s| s.cpu.k == 7 && (0xa0ba..0xa138).contains(&s.cpu.pc), || {
                                    let _owner = crate::cycle_ledger::routine(0x079b0e);
                                    n.link_item_boomerang();
                                });
                        }
                    }
                }
            }
        }
    }
}


#[test]
fn prefix11_recoil_caller_admitted_fragments_match_rom() {
    for timer in [2, 32, 33, 34, 127] {
        for camera in [0, 1, 0x80, 0xff] {
            for state in [2, 5, 6] {
                for velocity in [0, 16, 0xf0] {
                    let mut n = ZeldaState::new();
                    for (a, v) in [(0x10, 7), (0x1b, 1), (0x5d, state), (0x46, timer),
                        (0x2c5, camera), (0x27, velocity), (0x28, velocity),
                        (0x29, 2), (0x24, 0x20)] { n.write_wram_byte(a, v); }
                    n.sync_native_game_state_from_ram(); n.set_lag_free(true);
                    let s = marker36_player_machine(&n, 0x0786b5).unwrap();
                    let camera_return = std::cell::Cell::new(false);
                    crate::cycle_ledger::assert_rom_marker_work(0x07807f, s,
                        |s| {
                            if s.cpu.k == 7 && s.cpu.pc == 0x881c && !s.cpu.n { camera_return.set(true); }
                            s.cpu.k == 7 && s.cpu.sp == 0x1fa
                                && ((0x86b5..0x86d5).contains(&s.cpu.pc)
                                    || (0x8711..0x8723).contains(&s.cpu.pc)
                                    || (0x880d..0x881e).contains(&s.cpu.pc)
                                    || if camera_return.get() { s.cpu.pc == 0x8851 }
                                        else { (0x881e..0x8872).contains(&s.cpu.pc) })
                        }, || {
                            let _owner = crate::cycle_ledger::routine(0x07807f);
                            n.link_state_recoil();
                        });
                }
            }
        }
    }
}


#[test]
fn prefix11_sword_control_release_and_activation_match_rom() {
    for step in [0, 47, 48, 63] {
        let mut n = ZeldaState::new();
        n.write_wram_byte(0x79, step); n.write_wram_byte(0xf359, 1);
        n.write_wram_byte(0x10, 7); n.write_wram_byte(0x1b, 1);
        n.sync_native_game_state_from_ram(); n.set_lag_free(true);
        let s = marker36_player_machine(&n, 0x079d72).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x079d72, s,
            |s| s.cpu.k == 7 && (0x9d72..0x9d84).contains(&s.cpu.pc),
            || n.handle_sword_controls());
    }
}
