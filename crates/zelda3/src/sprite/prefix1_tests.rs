use super::*;

fn machine(native: &ZeldaState, entry: u32, slot: usize) -> snes::Snes {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).expect("ROM fixture");
    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&native.wram_image());
    machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0x30);
    machine.cpu.k = (entry >> 16) as u8;
    machine.cpu.db = (entry >> 16) as u8;
    machine.cpu.dp = 0;
    machine.cpu.pc = entry as u16;
    machine.cpu.sp = 0x1fc;
    machine.cpu.x = slot as u16;
    machine
}

#[test]
fn prefix1_track_body_markers_match_all_turning_branches() {
    for slot in [0usize, 15] {
        for direction in 0..4u8 {
            for head in 0..4u8 {
                for frame in [0u8, 1, 32, 64] {
                    let mut native = ZeldaState::new();
                    native.write_wram_byte(0x1a, frame);
                    native.sync_native_game_state_from_ram();
                    native.sprite_slot_view_mut(slot).set_direction(direction);
                    native.sprite_slot_view_mut(slot).set_head_direction(head);
                    native.set_lag_free(true);
                    let machine = machine(&native, 0x05dca2, slot);
                    crate::cycle_ledger::assert_rom_marker_work(0x05dca2, machine,
                        |s| s.cpu.k == 5 && (0xdca2..0xdcde).contains(&s.cpu.pc), || {
                            native.sprite_track_body_to_head(slot);
                        });
                }
            }
        }
    }
}

#[test]
fn prefix1_face_link_markers_match_sign_and_axis_branches() {
    for slot in [0usize, 15] {
        for x in [0u16, 0x10, 0x7f, 0x80, 0xff, 0x100, 0xfff0] {
            for y in [0u16, 0x10, 0x80, 0x100, 0xfff0] {
                let mut native = ZeldaState::new();
                native.follower_link_state_mut().set_x(x);
                native.follower_link_state_mut().set_y(y);
                native.sprite_slot_view_mut(slot).set_x(0x40);
                native.sprite_slot_view_mut(slot).set_y(0x40);
                native.sprite_slot_view_mut(slot).set_z(8);
                native.set_lag_free(true);
                let machine = machine(&native, 0x06eaa4, slot);
                crate::cycle_ledger::assert_rom_marker_work(0x06eaa4, machine,
                    |s| s.cpu.k == 6 && (0xeaa4..0xeacd).contains(&s.cpu.pc), || {
                        native.sprite_direction_and_offset_to_face_link(slot);
                    });
            }
        }
    }
}

#[test]
fn prefix1_barrier_markers_match_damage_branch_and_restore() {
    for floor in [0u8, 1] {
        for x in [0u16, 0x100] {
            let mut native = ZeldaState::new();
            native.sprite_slot_view_mut(0).set_x(x);
            native.sprite_slot_view_mut(0).set_y(0x40);
            native.sprite_slot_view_mut(0).set_floor(floor);
            native.sprite_slot_view_mut(0).set_flags4(0x80);
            native.sprite_slot_view_mut(0).set_state(9);
            native.follower_link_state_mut().set_x(0);
            native.follower_link_state_mut().set_y(0x40);
            native.set_lag_free(true);
            let machine = machine(&native, 0x1ef4f3, 0);
            crate::cycle_ledger::assert_rom_marker_work(0x1ef4f3, machine,
                |s| s.cpu.k == 0x1e && (0xf4f3..0xf508).contains(&s.cpu.pc), || {
                    native.sprite_behave_as_barrier(0);
                });
            assert_eq!(native.sprite_slot_view(0).flags4(), 0x80);
        }
    }
}
