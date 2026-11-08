use super::*;

#[test]
fn prefix30_big_fairy_draw_and_far_child_frames_match_rom() {
    for slot in [0usize, 15] {
        for graphics in 0..4 {
            for y in [0x40u16, 0xff, 0x100, 0x180] {
                let mut native = marker29_state();
                native.write_wram_byte(0xdd0 + slot, 9);
                native.write_wram_byte(0xe20 + slot, 0xc8);
                native.write_wram_byte(0xdc0 + slot, graphics);
                native.write_wram_byte(0xd10 + slot, 0x40);
                native.write_wram_byte(0xd00 + slot, y as u8);
                native.write_wram_byte(0xd20 + slot, (y >> 8) as u8);
                native.sync_native_game_state_from_ram();
                let machine = marker29_machine(&native, 0x1dc5d0, slot).expect("unmodified ROM");
                crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1dc5d0, 0x1d8549], machine,
                    |s| s.cpu.k == 0x1d && ((0xc5d0..0xc5ee).contains(&s.cpu.pc)
                        || (0x8549..0x854f).contains(&s.cpu.pc)), || native.big_faerie_draw(slot));
            }
        }
    }
}

#[test]
fn prefix30_big_fairy_all_draw_prefix_branches_match_rom() {
    for slot in [0usize, 15] {
        for delay in [0u8, 1, 2, 3, 0x3f, 0x40, 0xff] {
            for g in [0u8, 1, 0x80, 0x81] {
                for inactive in [false, true] {
                    let mut native = marker29_state();
                    for (address, value) in [(0xdd0, 9), (0xe20, 0xc8), (0xd10, 0x40),
                        (0xd00, 0x40), (0xd80, 1), (0xe10, delay), (0xed0, g)] {
                        native.write_wram_byte(address + slot, value);
                    }
                    native.write_wram_byte(0x11, u8::from(inactive));
                    native.sync_native_game_state_from_ram();
                    let machine = marker29_machine(&native, 0x1dc414, slot).expect("unmodified ROM");
                    crate::cycle_ledger::assert_rom_marker_work(0x1dc222, machine,
                        |s| s.cpu.k == 0x1d && ((0xc414..0xc41c).contains(&s.cpu.pc)
                            || (0xc4bf..0xc4eb).contains(&s.cpu.pc)), || {
                            let _parent = crate::cycle_ledger::routine(0x1dc222);
                            native.sprite_c8_big_fairy(slot);
                        });
                }
            }
        }
    }
}

#[test]
fn prefix30_cloud_entry_and_child_calls_match_rom() {
    for slot in [0usize, 15] {
        for inactive in [false, true] {
            let mut native = marker29_state();
            for (address, value) in [(0xdd0, 9), (0xe20, 0xc8), (0xeb0, 1),
                (0xd90, 0x80), (0xd80, 1), (0xd10, 0x40), (0xd00, 0x40)] {
                native.write_wram_byte(address + slot, value);
            }
            native.write_wram_byte(0x11, u8::from(inactive));
            native.sync_native_game_state_from_ram();
            let machine = marker29_machine(&native, 0x1dc414, slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_work(0x1dc222, machine,
                |s| s.cpu.k == 0x1d && (0xc414..0xc423).contains(&s.cpu.pc), || {
                    let _parent = crate::cycle_ledger::routine(0x1dc222);
                    native.sprite_c8_big_fairy(slot);
                });
        }
    }
}

#[test]
fn prefix30_pond_entry_branches_and_double_return_match_rom() {
    for slot in [0usize, 15] {
        for (a, b, x) in [(0u8, 0u8, 0x40u16), (0, 0, 0x400), (1, 0, 0x40), (0, 1, 0x40)] {
            let mut native = marker29_state();
            for (address, value) in [(0xdd0, 9), (0xe20, 0x72), (0xd90, a), (0xda0, b),
                (0xdb0, 31), (0xd10, x as u8), (0xd30, (x >> 8) as u8), (0xd00, 0x40)] {
                native.write_wram_byte(address + slot, value);
            }
            native.write_wram_word(0xa0, 21);
            native.write_wram_byte(0x11, 1);
            native.write_wram_byte(0x1a, 1);
            native.write_wram_word(0xfd8, x);
            native.write_wram_word(0xfda, 0x40);
            native.write_wram_byte(0xcaa + slot, 0x80);
            native.sync_native_game_state_from_ram();
            let machine = marker29_machine(&native, 0x06c319, slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_work(0x0684e2, machine,
                |s| s.cpu.k == 6 && ((0xc319..0xc329).contains(&s.cpu.pc)
                    || (0xc41d..0xc423).contains(&s.cpu.pc)), || {
                    let _parent = crate::cycle_ledger::routine(0x0684e2);
                    native.sprite_72_fairy_pond(slot);
                });
        }
    }
}

#[test]
fn prefix30_pond_draw_all_item_and_return_paths_match_rom() {
    for slot in [0usize, 15] {
        for room in [0u16, 21] {
            for ai in [0u8, 5, 6, 11, 12, 13] {
                for graphics in [0u8, 1, 3, 7, 8, 0x10, 0x1f] {
                    let mut native = marker29_state();
                    native.write_wram_word(0xa0, room);
                    for (address, value) in [(0xdd0, 9), (0xe20, 0x72), (0xd10, 0x40),
                        (0xd00, 0x40), (0xd80, ai), (0xdc0, graphics)] {
                        native.write_wram_byte(address + slot, value);
                    }
                    native.sync_native_game_state_from_ram();
                    let machine = marker29_machine(&native, 0x06c4b5, slot).expect("unmodified ROM");
                    crate::cycle_ledger::assert_rom_marker_work(0x06c4b5, machine,
                        |s| s.cpu.k == 6 && (0xc4b5..0xc4fd).contains(&s.cpu.pc),
                        || native.wish_pond2_draw(slot));
                }
            }
        }
    }
}
