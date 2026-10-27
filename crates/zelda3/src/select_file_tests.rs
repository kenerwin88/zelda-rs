use super::*;

fn prefix1_machine(native: &ZeldaState, entry: u32) -> snes::Snes {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).expect("ROM fixture");
    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
    machine.cart.ram.copy_from_slice(&native.sram);
    machine.ram.copy_from_slice(&native.wram_image());
    machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0x30);
    machine.cpu.k = (entry >> 16) as u8;
    machine.cpu.db = 0;
    machine.cpu.dp = 0;
    machine.cpu.pc = entry as u16;
    machine.cpu.sp = 0x1fc;
    machine
}

#[test]
fn prefix1_sram_validation_markers_match_valid_backup_and_erase_paths() {
    for primary in [false, true] {
        for backup in [false, true] {
            let mut native = ZeldaState::new();
            native.sram.fill(0);
            for slot in 0..3 {
                if primary { native.sram[slot * 0x500..slot * 0x500 + 2]
                    .copy_from_slice(&0x5a5au16.to_le_bytes()); }
                if backup { native.sram[slot * 0x500 + 0xf00..slot * 0x500 + 0xf02]
                    .copy_from_slice(&0x5a5au16.to_le_bytes()); }
            }
            native.set_lag_free(true);
            let machine = prefix1_machine(&native, 0x028054);
            let source = crate::cycle_ledger::assert_rom_marker_work(0x028054, machine,
                |s| s.cpu.k == 2 && (0x8054..0x8116).contains(&s.cpu.pc), || {
                    native.intro_validate_sram();
                });
            assert_eq!(native.sram, source.snes().cart.ram);
        }
    }
}

#[test]
fn prefix1_file_graphics_caller_markers_exclude_conversion_and_decoder_children() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets = std::fs::read(root.join("zelda3_assets.dat")).expect("asset fixture");
    let mut native = ZeldaState::new();
    native.set_assets(&assets).unwrap();
    native.set_lag_free(true);
    let machine = prefix1_machine(&native, 0x00e4e9);
    crate::cycle_ledger::assert_rom_marker_work(0x00e4e9, machine,
        |s| s.cpu.k == 0 && (0xe4e9..0xe556).contains(&s.cpu.pc), || {
            native.load_file_select_graphics();
        });
}

#[test]
fn prefix1_checkerboard_markers_match_rom_and_upload_words() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).expect("ROM fixture");
    let mut native = ZeldaState::new();
    native.set_lag_free(true);
    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&native.wram_image());
    machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0);
    machine.cpu.k = 0x0c;
    machine.cpu.db = 0x0c;
    machine.cpu.dp = 0;
    machine.cpu.pc = 0xce1b;
    machine.cpu.sp = 0x1fc;
    let source = crate::cycle_ledger::assert_rom_marker_work(0x0cce1b, machine,
        |s| s.cpu.k == 0x0c && (0xce1b..0xce53).contains(&s.cpu.pc), || {
            assert_eq!(native.select_file_func1(), 0x1806);
        });
    assert_eq!(&native.wram_image()[0x1002..0x1806],
        &source.snes().ram[0x1002..0x1806]);
}

#[test]
fn name_stripes_match_the_pre_cleanup_upload_bytes() {
    // Frozen from file_select_trigger_name_stripes_and_advance at a44598b3.
    // Compare the full stream, including byte order, row lengths and terminator.
    const EXPECTED: [u8; 253] = [
        0x61, 0x29, 0x00, 0x25, 0xe7, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0x49, 0x00,
        0x25, 0xf7, 0x18, 0x91, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0xa9, 0x00, 0x25, 0xe8, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0x61, 0xc9, 0x00, 0x25, 0xf8, 0x18, 0x91, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0x62, 0x29, 0x00, 0x25, 0xe9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0x62, 0x49, 0x00, 0x25, 0xf9, 0x18, 0x91, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9,
        0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18,
        0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xa9, 0x18, 0xff,
    ];
    assert_eq!(file_select_name_stripes(), EXPECTED);
}

#[test]
fn name_entry_vertical_scroll_down_from_top_row_matches_c() {
    let mut state = ZeldaState::new();
    state.set_select_file_name_row(0);
    state.follower_link_state_mut().set_joypad1h_last(0x04);

    state.name_file_check_for_scroll_input_y();

    assert_eq!(state.game_state.messaging.select_file_menu.name_row(), 1);
    assert_eq!(
        state
            .game_state
            .messaging
            .select_file_menu
            .name_scroll_y_step(),
        1
    );
    assert_eq!(state.game_state.messaging.select_file_menu.choice(1), 1);
}

#[test]
fn name_entry_vertical_scroll_down_released_settles_on_next_row() {
    let mut state = ZeldaState::new();
    state.set_select_file_name_row(0);
    state.set_select_file_name_cursor_y(0x83);
    state.follower_link_state_mut().set_joypad1h_last(0x04);

    state.name_file_check_for_scroll_input_y();
    state.follower_link_state_mut().set_joypad1h_last(0);

    for _ in 0..9 {
        state.name_file_do_the_naming();
    }

    assert_eq!(state.game_state.messaging.select_file_menu.name_row(), 1);
    assert_eq!(
        state.game_state.messaging.select_file_menu.name_cursor_y(),
        0x93
    );
    assert_eq!(
        state
            .game_state
            .messaging
            .select_file_menu
            .name_scroll_y_step(),
        0
    );
}
