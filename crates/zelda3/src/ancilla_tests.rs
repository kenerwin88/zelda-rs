use super::*;

fn item_receipt_test_rom() -> Option<Vec<u8>> {
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let mut rom = std::fs::read(path).ok()?;
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    Some(rom)
}

fn run_item_receipt_source(rom: &[u8], ram: &[u8], entry: u32, stop: u32, slot: usize) -> (u64, Vec<u8>) {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let checkpoint = RomCpuCheckpoint {
        entry_pc: entry, stop_pc: stop,
        a: 0, x: slot as u16, y: 0, sp: 0x01fc, dp: 0, db: 8,
        carry: true, zero: false, overflow: true, negative: false,
        interrupt_disable: false, decimal: false,
        accumulator_is_8_bit: true, index_is_8_bit: true,
        emulation: false, waiting: false,
        stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
    };
    let mut run = RomCpuTimingRun::new(rom, ram, &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = ram.to_vec();
    let mut cycles = 0;
    let mut steps = 0;
    while !run.is_complete() {
        cycles += u64::from(run.step().master_cycles);
        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
        steps += 1;
        assert!(steps < 2000, "item receipt did not reach {stop:06x}");
    }
    (cycles, expected)
}

#[test]
fn item_receipt_drawing_matches_rom_for_all_items_and_clipping_branches() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for item in 0..76 {
        for x in [0u16, 255, 256, 0xffff] {
            for y in [0u16, 231, 232, 239, 240, 255, 256, 0xffff] {
                let mut state = ZeldaState::new();
                let slot = 4;
                state.ram[0..2].copy_from_slice(&y.to_le_bytes());
                state.ram[2..4].copy_from_slice(&x.to_le_bytes());
                state.ram[8..10].copy_from_slice(&y.wrapping_add(8).to_le_bytes());
                state.ram[0x90..0x92].copy_from_slice(&0x820u16.to_le_bytes());
                state.ram[0x92..0x94].copy_from_slice(&0xa28u16.to_le_bytes());
                state.ram[0xc5e + slot] = item;
                state.ram[0xbf0 + slot] = 5;
                state.ram[0x820..0x828].fill(0x5a);
                state.sync_native_game_state_from_ram();
                let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c690, 0x08_8036, slot);
                let before = crate::cycle_ledger::master();
                state.ancilla_receive_item_draw(slot, x, y);
                assert_eq!(crate::cycle_ledger::master() - before, cycles, "item={item} x={x} y={y}");
                assert_eq!(&state.ram[0x820..0x828], &expected[0x820..0x828], "item={item} x={x} y={y}");
                assert_eq!(&state.ram[0xa28..0xa2a], &expected[0xa28..0xa2a]);
            }
        }
    }
}

#[test]
fn item_receipt_message_selection_work_matches_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for item in 0..76 {
        for step in [1, 2] {
            for (indoors, room) in [(0, 0), (1, 0x20), (1, 0xff), (1, 0x10f), (1, 0x110), (1, 0x112), (1, 0x11f)] {
                for flags in [0, 7] {
                    for pieces in 0..4 {
                        let mut state = ZeldaState::new();
                        let slot = 4;
                        state.ram[0x10] = 7;
                        state.ram[0x1b] = indoors;
                        state.ram[0xa0..0xa2].copy_from_slice(&(room as u16).to_le_bytes());
                        state.ram[0xc5e + slot] = item;
                        state.ram[0xc54 + slot] = step;
                        state.ram[0xf374] = flags;
                        state.ram[0xf36b] = pieces;
                        state.sync_native_game_state_from_ram();
                        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c546, 0x08_c5f7, slot);
                        let before = crate::cycle_ledger::master();
                        state.ancilla22_item_receipt_show_message(slot);
                        assert_eq!(crate::cycle_ledger::master() - before, cycles,
                            "item={item} step={step} indoors={indoors} room={room:x} flags={flags} pieces={pieces}");
                        for address in [0x10, 0x11, 0x12, 0x012d, 0x1cf0, 0x1cf1] {
                            assert_eq!(state.ram[address], expected[address], "item={item} address={address:x}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn item_receipt_countdown_through_message_matches_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for slot in 0..5 {
        for aux in 1..=24 {
            for submodule in [0, 2, 9, 11, 43] {
                let mut state = ZeldaState::new();
                state.ram[0x10] = 7;
                state.ram[0x11] = submodule;
                state.ram[0x1b] = 1;
                state.ram[0xa0] = 0x20;
                state.ram[0xc5e + slot] = 0x3f;
                state.ram[0xc54 + slot] = 1;
                state.ram[0x3b1 + slot] = aux;
                state.ram[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                state.ram[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                state.sync_native_game_state_from_ram();
                let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c38a, 0x08_8036, slot);
                let before = crate::cycle_ledger::master();
                state.ancilla22_item_receipt(slot);
                assert_eq!(crate::cycle_ledger::master() - before, cycles,
                    "slot={slot} aux={aux} submodule={submodule}");
                for range in [0x10..0x14, 0x3b1..0x3b6, 0xc68..0xc6d, 0x800..0x808, 0xa20..0xa22, 0x1cf0..0x1cf2] {
                    assert_eq!(&state.ram[range.clone()], &expected[range], "slot={slot} aux={aux} submodule={submodule}");
                }
            }
        }
    }
}

#[test]
fn item_receipt_message_entry_and_already_open_return_match_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for module in 0..=27 {
        let mut state = ZeldaState::new();
        state.ram[0x10] = module;
        state.ram[0x11] = 9;
        state.ram[0x223] = 0x5a;
        state.ram[0x1cd8] = 3;
        state.ram[0xc5e + 4] = 0x3f;
        state.ram[0xc54 + 4] = 1;
        state.sync_native_game_state_from_ram();
        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c546, 0x08_c5f7, 4);
        let before = crate::cycle_ledger::master();
        state.ancilla22_item_receipt_show_message(4);
        assert_eq!(crate::cycle_ledger::master() - before, cycles, "module={module}");
        for address in [0x10, 0x11, 0x10c, 0x223, 0x1cd8] {
            assert_eq!(state.ram[address], expected[address], "module={module} address={address:x}");
        }
    }
}

#[test]
fn item_receipt_animation_and_initial_countdown_work_match_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    // Crystal transmutation and milestone graphics reload have separately
    // owned callees; this matrix covers drawing and animation without them.
    for item in (0..76).filter(|&item| item != 0x20) {
        for step in [0, 2, 3] {
            for timer in [15, 17] {
                for frame_timer in [0, 1] {
                    if (0x34..=0x36).contains(&item) && frame_timer == 0 { continue; }
                    for frame in [0, 2] {
                        let mut state = ZeldaState::new();
                        let slot = 4;
                        state.ram[0xc5e + slot] = item;
                        state.ram[0xc54 + slot] = step;
                        state.ram[0xc68 + slot] = timer;
                        state.ram[0x39f + slot] = frame_timer;
                        state.ram[0x3a4 + slot] = frame;
                        state.ram[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                        state.ram[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                        state.sync_native_game_state_from_ram();
                        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c5f7, 0x08_8036, slot);
                        let before = crate::cycle_ledger::master();
                        state.ancilla22_item_receipt_draw_and_update(slot);
                        assert_eq!(crate::cycle_ledger::master() - before, cycles,
                            "item={item} step={step} timer={timer} frame_timer={frame_timer} frame={frame}");
                        for range in [0x39f..0x3a9, 0xbf0..0xbf5, 0x800..0x808, 0xa20..0xa22] {
                            assert_eq!(&state.ram[range.clone()], &expected[range]);
                        }
                    }
                }
            }
        }
    }
    for item in [1, 0x37, 0x38, 0x39, 0x3f] {
        for step in [0, 3] {
            for aux in [2, 3, 25] {
                for timer in [1, 16, 17, 18] {
                    let mut state = ZeldaState::new();
                    let slot = 4;
                    state.ram[0x10] = 7;
                    state.ram[0xc5e + slot] = item;
                    state.ram[0xc54 + slot] = step;
                    state.ram[0xc68 + slot] = timer;
                    state.ram[0x3b1 + slot] = aux;
                    state.ram[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                    state.ram[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c38a, 0x08_8036, slot);
                    let before = crate::cycle_ledger::master();
                    state.ancilla22_item_receipt(slot);
                    assert_eq!(crate::cycle_ledger::master() - before, cycles,
                        "item={item} step={step} aux={aux} timer={timer}");
                    for range in [0x10..0x14, 0x2cd..0x2cf, 0x3b1..0x3b6, 0x800..0x808, 0x1cf0..0x1cf2, 0xf3cc..0xf3cd] {
                        assert_eq!(&state.ram[range.clone()], &expected[range]);
                    }
                }
            }
        }
    }
}

#[test]
fn adjusted_oam_coordinate_work_matches_rom_including_slot_page_crossing() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for slot in 0..10 {
        for floor in 0..4 {
            for x in [0u16, 1, 255, 256, 0x7fff, 0x8000, 0xffff] {
                for y in [0u16, 1, 255, 256, 0x7fff, 0x8000, 0xffff] {
                    let mut state = ZeldaState::new();
                    state.ram[0xc04 + slot] = x as u8;
                    state.ram[0xc18 + slot] = (x >> 8) as u8;
                    state.ram[0xbfa + slot] = y as u8;
                    state.ram[0xc0e + slot] = (y >> 8) as u8;
                    state.ram[0xc7c + slot] = floor;
                    state.ram[0x11e..0x120].copy_from_slice(&0x1ffu16.to_le_bytes());
                    state.ram[0x122..0x124].copy_from_slice(&0xfffeu16.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: 0x08_f6a4, stop_pc: 0x08_8036,
                        a: 0, x: slot as u16, y: 0, sp: 0x01fc, dp: 0, db: 8,
                        carry: true, zero: false, overflow: true, negative: false,
                        interrupt_disable: false, decimal: false,
                        accumulator_is_8_bit: true, index_is_8_bit: true,
                        emulation: false, waiting: false,
                        stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
                    };
                    let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
                        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint)
                        .unwrap();
                    run.enable_cpu_write_trace();
                    let mut expected = state.ram.clone();
                    let mut cycles = 0u64;
                    let mut steps = 0;
                    while !run.is_complete() {
                        cycles += u64::from(run.step().master_cycles);
                        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                        steps += 1;
                        assert!(steps < 100, "adjusted OAM preparation did not return");
                    }
                    let before = crate::cycle_ledger::master();
                    let actual = state.ancilla_prep_adjusted_oam_coord(slot);
                    assert_eq!(crate::cycle_ledger::master() - before, cycles, "slot={slot} floor={floor}");
                    assert_eq!(actual, (u16::from_le_bytes([expected[2], expected[3]]),
                        u16::from_le_bytes([expected[0], expected[1]])), "slot={slot} floor={floor}");
                    assert_eq!(&state.ram[0x64..0x66], &expected[0x64..0x66]);
                }
            }
        }
    }
}

#[test]
fn dash_dust_motive_expires_out_of_range_frame() {
    let mut state = ZeldaState::new();
    state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
    state.ancilla_slot_view_mut(0).set_timer(1);
    state.ancilla_slot_view_mut(0).set_item_to_link(3);

    state.dash_dust_motive(0);

    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
}

#[test]
fn sword_wall_hit_does_not_write_terminal_animation_counter() {
    let mut state = ZeldaState::new();
    state.ancilla_slot_view_mut(4).set_ancilla_type(0x1b);
    state.ancilla_slot_view_mut(4).set_item_to_link(7);
    state.ancilla_slot_view_mut(4).set_aux_timer(0);

    state.ancilla_sword_wall_hit(4);

    let slot = state.ancilla_slot_view(4);
    assert_eq!(slot.ancilla_type(), 0);
    assert_eq!(slot.item_to_link(), 7);
    assert_eq!(slot.aux_timer(), 0xff);
}
