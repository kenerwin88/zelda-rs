use super::*;

fn state() -> ZeldaState {
    let mut n = ZeldaState::new();
    n.set_assets(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap()).unwrap();
    n.set_lag_free(true);
    n
}

fn machine(n: &ZeldaState, entry: u32, far: bool) -> snes::Snes {
    let mut s = snes::Snes::new();
    s.cart.load(snes::CartType::LoRom, &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap(), 0x2000);
    s.ram.copy_from_slice(&n.wram_image());
    s.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
    s.ram[0x1ff] = 0;
    s.cpu.e = false; s.cpu.unpack_flags(0); s.cpu.k = (entry >> 16) as u8;
    s.cpu.db = s.cpu.k; s.cpu.pc = entry as u16; s.cpu.sp = 0x1fc;
    assert!(far == (entry == 0x1bc980 || entry == 0x1bc983));
    s
}

#[test]
fn prefix32_map_coordinates_match_all_directions_and_wrapping() {
    for direction in [0, 2, 4, 6] {
        for (x,y) in [(0,0),(0x10,0x20),(0x7f8,0x7f8),(0xfff8,0xfff8)] {
            let mut n = state();
            for (a,v) in [(0x22,x),(0x20,y),(0x708,0x100),(0x70a,0x7f0),(0x70c,0x20),(0x70e,0x7e)] {
                n.write_wram_word(a,v);
            }
            n.write_wram_byte(0x2f,direction); n.sync_native_game_state_from_ram();
            let s = machine(&n,0x1bbf64,false);
            let mut result = (0,0,0);
            let source = crate::cycle_ledger::assert_rom_marker_work(0x1bbf64,s,
                |s| s.cpu.k == 0x1b && (0xbf64..0xbf9d).contains(&s.cpu.pc),
                || result = n.overworld_get_link_map16_coords_result());
            assert_eq!(result.0, source.snes().cpu.x);
            assert_eq!(result.1, u16::from_le_bytes(source.snes().ram[2..4].try_into().unwrap()));
            assert_eq!(result.2, u16::from_le_bytes(source.snes().ram[0..2].try_into().unwrap()));
        }
    }
}

#[test]
fn prefix32_map_vram_address_matches_all_quadrants() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for addr in [0,0x1e,0x20,0x3e,0x7fe,0x800,0x820,0xffe,0x1000,0x1ffe] {
        let mut n = state(); n.write_wram_word(0,addr);
        let s = machine(&n,0x1bca69,false);
        let mut result = 0;
        let source = crate::cycle_ledger::assert_rom_marker_work(0x1bca69,s,
            |s| {
                if s.cpu.k == 0x1b && [0xca73,0xca82].contains(&s.cpu.pc) {
                    edges.borrow_mut().insert((s.cpu.pc,!s.cpu.c));
                }
                s.cpu.k == 0x1b && (0xca69..0xca9f).contains(&s.cpu.pc)
            }, || result = n.overworld_find_map16_vram_address(addr));
        assert_eq!(result,u16::from_le_bytes(source.snes().ram[2..4].try_into().unwrap()));
    }
    assert_eq!(edges.borrow().len(),4);
}

#[test]
fn prefix32_map_packet_direct_and_zero_offset_entries_match_rom() {
    for entry in [0x1bc980,0x1bc983] {
        for pos in [0,0x20,0x780,0x800,0x820,0x1ffe] {
            for tile in [0,0x36,0xda4,0xe40] {
                let mut n = state(); n.write_wram_word(0x1000,0x20); n.sync_native_game_state_from_ram();
                let mut s = machine(&n,entry,true); s.cpu.a = tile;
                s.cpu.x = pos; s.cpu.y = if entry == 0x1bc983 {2} else {0x80};
                let effective_pos = pos.wrapping_add(if entry == 0x1bc983 {2} else {0});
                let source = crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1bc980,0x1bc983,0x1bca69],s,
                    |s| s.cpu.k == 0x1b && ((entry as u16..0xc9de).contains(&s.cpu.pc)
                        || (0xca69..0xca9f).contains(&s.cpu.pc)),
                    || if entry == 0x1bc980 { n.overworld_draw_map16(pos,tile); }
                        else { n.overworld_draw_map16_anywhere(effective_pos,tile); });
                assert_eq!(&n.ram[0x1000..0x1054],&source.snes().ram[0x1000..0x1054]);
            }
        }
    }
}

#[test]
fn prefix32_four_tile_map_update_excludes_draw_children_and_matches_both_steps() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for counter in [0,8,24,32,40,80] {
        for pos in [0,0x20,0x780,0x800,0x820] {
            let mut n = state();
            for (a,v) in [(0x698,pos),(0x692,counter),(0x690,0),(0x4ac,4),(0x1000,0)] {
                n.write_wram_word(a,v);
            }
            n.sync_native_game_state_from_ram();
            let s = machine(&n,0x02ad85,false);
            let source = crate::cycle_ledger::assert_rom_marker_work(0x02ad85,s,
                |s| {
                    if s.cpu.k == 2 && s.cpu.pc == 0xae4e { edges.borrow_mut().insert(!s.cpu.z); }
                    s.cpu.k == 2 && (0xad85..0xae5e).contains(&s.cpu.pc)
                }, || n.overworld_do_map_update32x32());
            for range in [0x690..0x69a,0x1000..0x1044,0xf800..0xfc00] {
                assert_eq!(&n.ram[range.clone()],&source.snes().ram[range]);
            }
        }
    }
    assert_eq!(edges.borrow().len(),2);
}

#[test]
fn prefix32_mosaic_dispatch_all_steps_parities_and_music_paths_match_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for step in 0..3 {
        for countdown in [0,1,2,3] {
            for screen in [0,1,0x40,0x41,0x80] {
                let mut n = state();
                for (a,v) in [(0xb0,step),(0xc007,countdown),(0xc011,0x30),(0x8a,screen)] {
                    n.write_wram_byte(a,v);
                }
                n.sync_native_game_state_from_ram();
                let mut s = machine(&n,0x02b150,false); s.cpu.unpack_flags(0x30);
                crate::cycle_ledger::assert_rom_marker_work(0x02b150,s,
                    |s| {
                        if s.cpu.k == 2 {
                            let taken = match s.cpu.pc {0xb155=>Some(!s.cpu.c),0xb1a2=>Some(s.cpu.c),0xb1ad=>Some(!s.cpu.z),_=>None};
                            if let Some(t)=taken {edges.borrow_mut().insert((s.cpu.pc,t));}
                        }
                        s.cpu.k == 2 && ((0xb150..0xb16b).contains(&s.cpu.pc)
                            || (0xb195..0xb1bb).contains(&s.cpu.pc))
                            || s.cpu.k == 0 && (0x8784..0x879c).contains(&s.cpu.pc)
                    }, || n.overworld_func1c());
            }
        }
    }
    assert_eq!(edges.borrow().len(),6);
}
