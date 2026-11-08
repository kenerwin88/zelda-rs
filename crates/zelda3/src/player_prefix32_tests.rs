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
fn prefix32_y_velocity_rollback_and_falling_snap_match_all_byte_inputs() {
    for velocity in 0..=255u8 {
        for low in 0..8 {
            for entry in [0x07c1e4,0x07c29f] {
                let mut n = state();
                n.write_wram_word(0x20, if low & 1 == 0 {0} else {0xffff});
                n.write_wram_byte(0x30,velocity); n.write_wram_byte(0x51,low);
                n.sync_native_game_state_from_ram();
                let mut s = machine(&n,entry,false); s.cpu.unpack_flags(0x30);
                let source = crate::cycle_ledger::assert_rom_marker_work(entry,s,
                    |s| s.cpu.k == 7 && if entry == 0x07c1e4 {
                        (0xc1e4..0xc1ff).contains(&s.cpu.pc)
                    } else { (0xc29f..0xc2ba).contains(&s.cpu.pc) },
                    || if entry == 0x07c1e4 {n.link_add_in_velocity_y_falling();}
                        else {n.link_add_in_velocity_y();});
                assert_eq!(&n.ram[0x20..0x22],&source.snes().ram[0x20..0x22]);
            }
        }
    }
}

#[test]
fn prefix32_rebound_all_directions_and_swimming_paths_match_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for direction in 0..4 {
        for moving in 0..3 {
            let mut n = state();
            n.write_wram_byte(0x66,direction); n.write_wram_byte(0x34a,moving);
            n.sync_native_game_state_from_ram();
            let mut s = machine(&n,0x079222,false); s.cpu.unpack_flags(0x30);
            crate::cycle_ledger::assert_rom_marker_work(0x079222,s,
                |s| {
                    if s.cpu.k == 7 {
                        let taken = match s.cpu.pc {0x923c=>Some(s.cpu.z),0x9288=>Some(!s.cpu.z),_=>None};
                        if let Some(t)=taken {edges.borrow_mut().insert((s.cpu.pc,t));}
                    }
                    s.cpu.k == 7 && (0x9222..0x9291).contains(&s.cpu.pc)
                }, || n.link_apply_tile_rebound());
        }
    }
    assert_eq!(edges.borrow().len(),4);
}

#[test]
fn prefix32_repel_dash_all_guards_and_sound_branches_exclude_children() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for running in [0,1] {
        for counter in [0,63,64] {
            for sound in [0,27,50,0x5b,0x72] {
                let mut n = state();
                for (a,v) in [(0x372,running),(0x2f1,counter),(0x12f,sound),(0x66,0),(0x1b,1)] {
                    n.write_wram_byte(a,v);
                }
                n.sync_native_game_state_from_ram();
                let mut s = machine(&n,0x0791f1,false); s.cpu.unpack_flags(0x30);
                let early = running == 0 || counter == 64;
                crate::cycle_ledger::assert_rom_marker_work(0x0791f1,s,
                    |s| {
                        if s.cpu.k == 7 {
                            let taken=match s.cpu.pc {0x91f4=>Some(s.cpu.z),0x91fb=>Some(!s.cpu.z),0x9217|0x921b=>Some(s.cpu.z),_=>None};
                            if let Some(t)=taken {edges.borrow_mut().insert((s.cpu.pc,t));}
                        }
                        s.cpu.k == 7 && ((0x91f1..0x9222).contains(&s.cpu.pc) || early && s.cpu.pc == 0x9290)
                    }, || n.repel_dash());
            }
        }
    }
    for pc in [0x91f4,0x91fb,0x9217,0x921b] {
        for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");}
    }
}
