use super::*;

#[test]
fn prefix31_bonk_wrapper_matches_rom_for_each_selected_child() {
    for slot in [0, 15] {
        for graphics in 0..3 {
            let mut native = marker29_state();
            native.write_wram_byte(0xe20 + slot, 0x3b);
            native.write_wram_byte(0xdc0 + slot, graphics);
            native.write_wram_byte(0xfa0, slot as u8);
            native.sync_native_game_state_from_ram();
            let machine = marker29_machine(&native, 0x05fbef, slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x05fbef, machine,
                |s| s.cpu.k == 5 && (0xfbef..0xfbf7).contains(&s.cpu.pc),
                || native.sprite_active_main_dispatch(slot));
        }
    }
}

#[test]
fn prefix31_bonk_bank6_bounce_matches_rom() {
    for slot in [0,15] {for graphics in 0..3 {
        let mut native=marker29_state();
        native.write_wram_byte(0xe20+slot,0x3b);native.write_wram_byte(0xdc0+slot,graphics);
        native.write_wram_byte(0xfa0,slot as u8);native.sync_native_game_state_from_ram();
        let machine=marker29_machine(&native,0x06c049,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x0684e2,machine,
            |s|s.cpu.k==6 && (0xc049..0xc04e).contains(&s.cpu.pc),
            || {let _parent=crate::cycle_ledger::routine(0x0684e2);native.sprite_active_main_dispatch(slot);});
    }}
}

#[test]
fn prefix31_bonk_key_all_caller_paths_match_rom() {
    let visited = std::cell::RefCell::new(std::collections::BTreeSet::new());
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    let previous = std::cell::Cell::new(None);
    for slot in [0, 15] {
        for ai in 0..4 {
            for (z, velocity) in [(0u8, 0u8), (0, 1), (0, 0x80), (0xff, 0), (0xff, 0xe0)] {
                for path in 0..7 {
                    let mut native = marker29_state();
                    for (a,v) in [(0xdd0,9),(0xe20,0x3b),(0xdc0,1),(0xd80,ai),
                        (0xd10,0x40),(0xd00,0x40),(0xf70,z),(0xf80,velocity)] {
                        native.write_wram_byte(a + slot,v);
                    }
                    native.write_wram_byte(0x11,u8::from(path == 6));
                    native.write_wram_word(0x22,if path == 0 {0x180} else {0x40});
                    native.write_wram_word(0x20,if path == 1 {0x180} else {0x40});
                    native.write_wram_word(0x11a,if path == 2 {0} else {1});
                    native.write_wram_byte(0x44,if path == 3 {0x80} else {0});
                    native.write_wram_byte(0xcba + slot,if path == 4 {2} else {0});
                    native.write_wram_byte(0xee,1); // Exclude damage except the deliberate same-floor case.
                    native.write_wram_byte(0xfa0,slot as u8);
                    if path == 5 { native.write_wram_byte(0xee,0); }
                    native.sync_native_game_state_from_ram();
                    let machine = marker29_machine(&native,0x05fc04,slot).unwrap();
                    previous.set(None);
                    crate::cycle_ledger::assert_rom_marker_work(0x05fbf7,machine,
                        |s| {
                            let pc=(u32::from(s.cpu.k)<<16)|u32::from(s.cpu.pc);
                            if let Some(from)=previous.replace(Some(pc)) { edges.borrow_mut().insert((from,pc)); }
                            visited.borrow_mut().insert(pc);
                            s.cpu.k == 5 && ((0xfc04..0xfc46).contains(&s.cpu.pc)
                                || (0xfc4e..0xfc9e).contains(&s.cpu.pc)
                                || (0xfd1b..0xfd3a).contains(&s.cpu.pc))
                                || s.cpu.k == 0 && (0x8784..0x879c).contains(&s.cpu.pc)
                        }, || { let _parent=crate::cycle_ledger::routine(0x05fbf7); native.sprite_bonk_key(slot); });
                }
            }
        }
    }
    for pc in [0x05fc04,0x05fc08,0x05fc0b,0x05fc0f,0x05fc11,0x05fc16,0x05fc19,
        0x05fc1f,0x05fc24,0x05fc39,0x05fc3f,0x05fc4e,0x05fc5f,0x05fc6e,0x05fc76,
        0x05fc79,0x05fc7e,0x05fc9d,0x05fd1b,0x05fd2e,0x05fd33,0x05fd39] {
        assert!(visited.borrow().contains(&pc),"uncovered block {pc:06x}");
    }
    for pc in [0x05fc0f,0x05fc22,0x05fc37,0x05fc5d,0x05fc6c,0x05fc74,0x05fd31] {
        assert_eq!(edges.borrow().iter().filter(|(from,_)|*from==pc).count(),2,"branch {pc:06x}");
    }
}

#[test]
fn prefix31_book_caller_prefix_and_bounce_paths_match_rom() {
    let branches=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0,15] {
        for (z,velocity) in [(0u8,0u8),(0,1),(0,0x80),(0xff,0),(0xff,0xe0)] {
            for inactive in [false,true] {
                for same_floor in [false,true] {
                let mut native=marker29_state();
                for (a,v) in [(0xdd0,9),(0xe20,0x3b),(0xd10,0x40),(0xd00,0x40),
                    (0xf70,z),(0xf80,velocity)] { native.write_wram_byte(a+slot,v); }
                native.write_wram_byte(0xee,u8::from(!same_floor));
                native.write_wram_word(0x22,0x40);native.write_wram_word(0x20,0x40);
                native.write_wram_byte(0xfa0,slot as u8);
                native.write_wram_byte(0x11,u8::from(inactive));
                native.sync_native_game_state_from_ram();
                let machine=marker29_machine(&native,0x05fc9e,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(0x05fbf7,machine,
                    |s| {
                        if s.cpu.k==5 {let taken=match s.cpu.pc {0xfca9=>Some(!s.cpu.c),0xfcbc=>Some(!s.cpu.n),0xfcd1=>Some(s.cpu.z),_=>None};
                            if let Some(t)=taken {branches.borrow_mut().insert((s.cpu.pc,t));}}
                        s.cpu.k==5 && (0xfc9e..0xfce0).contains(&s.cpu.pc)
                            || s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc)
                    },
                    || { let _parent=crate::cycle_ledger::routine(0x05fbf7);native.book_of_mudora_before_ai(slot); });
                }
            }
        }
    }
    for pc in [0xfca9,0xfcbc,0xfcd1] {for taken in [false,true] {assert!(branches.borrow().contains(&(pc,taken)),"Book branch {pc:x}/{taken}");}}
}

#[test]
fn prefix31_shop_dispatch_and_clerk_item_prefixes_match_rom() {
    for slot in [0,15] {
        for subtype in [0u8,7,8,9,10,11,12,13] {
            for dark in [false,true] {
                for inactive in [false,true] {
                    let mut native=marker29_state();
                    for (a,v) in [(0xdd0,9),(0xe20,0xbb),(0xe80,subtype),(0xd10,0x40),
                        (0xd00,0x40),(0xd80,1)] { native.write_wram_byte(a+slot,v); }
                    native.write_wram_byte(0xfa0,slot as u8);
                    native.write_wram_byte(0x11,u8::from(inactive));
                    native.write_wram_byte(0xfff,u8::from(dark));
                    native.write_wram_word(0x22,0x180);native.write_wram_word(0x20,0x180);
                    native.sync_native_game_state_from_ram();
                    let machine=marker29_machine(&native,0x1eeeef,slot).unwrap();
                    crate::cycle_ledger::assert_rom_marker_work(0x1e8b19,machine,
                        |s| s.cpu.k==0x1e && ((0xeeef..0xeef6).contains(&s.cpu.pc)
                            || (0xef12..0xef22).contains(&s.cpu.pc)
                            || (0xef50..0xef5c).contains(&s.cpu.pc)
                            || [(0xf16e,0xf174),(0xf1f2,0xf1f8),(0xf230,0xf236),
                                (0xf27d,0xf283),(0xf2af,0xf2b5),(0xf2f0,0xf2f6),(0xf322,0xf328)]
                                .iter().any(|&(a,b)|(a..b).contains(&s.cpu.pc)))
                            || s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc),
                        || {let _parent=crate::cycle_ledger::routine(0x1e8b19);native.sprite_bb_shopkeeper(slot);});
                }
            }
        }
    }
}

#[test]
fn prefix31_bomb_shop_dispatch_and_all_item_prefixes_match_rom() {
    for slot in [0,15] {
        for subtype in 0..3 {
            for inactive in [false,true] {
                let mut native=marker29_state();
                for (a,v) in [(0xdd0,9),(0xe20,0xb5),(0xe80,subtype),(0xd10,0x40),
                    (0xd00,0x40),(0xdf0,1)] {native.write_wram_byte(a+slot,v);}
                native.write_wram_byte(0xfa0,slot as u8);
                native.write_wram_byte(0x11,u8::from(inactive));
                native.write_wram_word(0x22,0x180);native.write_wram_word(0x20,0x180);
                native.sync_native_game_state_from_ram();
                let machine=marker29_machine(&native,0x1ee111,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(0x1e8b19,machine,
                    |s| s.cpu.k==0x1e && [(0xe111,0xe118),(0xe134,0xe13a),
                        (0xe190,0xe196),(0xe1df,0xe1e5)].iter().any(|&(a,b)|(a..b).contains(&s.cpu.pc))
                        || s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc),
                    || {let _parent=crate::cycle_ledger::routine(0x1e8b19);native.sprite_b5_bomb_shop(slot);});
            }
        }
    }
}

#[test]
fn prefix31_bomb_shop_draw_all_frames_and_clip_paths_match_rom() {
    for slot in [0,15] {for subtype in 0..3 {for graphics in 0..2 {
        for y in [0x40u16,0xff,0x100,0x180] {
            let mut native=marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0xb5),(0xe80,subtype),(0xdc0,graphics),
                (0xd10,0x40),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {native.write_wram_byte(a+slot,v);}
            native.write_wram_byte(0xfa0,slot as u8);native.sync_native_game_state_from_ram();
            let machine=marker29_machine(&native,0x1ee2c6,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1ee2c6,machine,
                |s|s.cpu.k==0x1e && (0xe2c6..0xe2e9).contains(&s.cpu.pc),
                ||native.bomb_shop_entity_draw(slot));
        }
    }}}
}

#[test]
fn prefix31_boulder_draw_and_large_shadow_child_wrappers_match_rom() {
    for slot in [0,15] {for z in [0u8,1,31,32,0x80,0xff] {for subtype in [0u8,8,16,24,255] {
        let mut native=marker29_state();
        for (a,v) in [(0xdd0,9),(0xe20,0xc2),(0xe80,subtype),(0xf70,z),(0xd10,0x40),(0xd00,0x40)] {
            native.write_wram_byte(a+slot,v);
        }
        native.write_wram_byte(0xfa0,slot as u8);native.sync_native_game_state_from_ram();
        let machine=marker29_machine(&native,0x1dd185,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1d8549,0x0684bd],machine,
            |s|s.cpu.k==0x1d && (0x8549..0x854f).contains(&s.cpu.pc)
                || s.cpu.k==6 && (0x84bd..0x84c1).contains(&s.cpu.pc),
            ||native.boulder_draw(slot));
    }}}
}

#[test]
fn prefix31_crab_entry_and_recoil_prefix_all_paths_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0,15] {for inactive in [false,true] {for recoil in [0u8,1,8,0x80] {
        for direction in 0..4 {for timer in [0u8,1] {for tile in [0u8,1,0x0f,0x40,0x80] {
        let mut native=marker29_state();
        for (a,v) in [(0xdd0,9),(0xe20,0x58),(0xea0,recoil),(0xdf0,1),(0xd10,0x40),(0xd00,0x40)] {
            native.write_wram_byte(a+slot,v);
        }
        native.write_wram_byte(0x11,u8::from(inactive));native.write_wram_byte(0xfa0,slot as u8);
        native.write_wram_byte(0xde0+slot,direction);native.write_wram_byte(0xdf0+slot,timer);
        native.write_wram_byte(0xd50+slot,16);native.write_wram_byte(0xd40+slot,16);
        for a in 0x12000..0x14000 {native.write_wram_byte(a,tile);}
        native.sync_native_game_state_from_ram();
        let machine=marker29_machine(&native,0x0594b5,slot).unwrap();
        let random=std::cell::RefCell::new(Vec::new());
        crate::cycle_ledger::assert_rom_marker_work(0x05b5d3,machine,
            |s| {
                if s.cpu.k==13 && s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                let owns=s.cpu.k==5 && (0x94b5..0x9500).contains(&s.cpu.pc);
                if owns {let outcome=match s.cpu.pc {0x94c7|0x94cc=>Some(!s.cpu.z),0x94f6=>Some(!s.cpu.c),_=>None};
                    if let Some(taken)=outcome {edges.borrow_mut().insert((s.cpu.pc,taken));}}
                owns
            },
            || {let _parent=crate::cycle_ledger::routine(0x05b5d3);
                native.install_rom_random_replay(random.borrow().clone(),0);native.rom_random_replay.begin_frame();native.sprite_58_crab(slot);});
    }}}}}}
    for pc in [0x94c7,0x94cc,0x94f6] {for taken in [false,true] {assert!(edges.borrow().contains(&(pc,taken)),"branch {pc:x}/{taken}");}}
}

#[test]
fn prefix31_ropa_and_snapdragon_movement_children_match_rom() {
    for slot in [0,15] {for kind in [0x0eu8,0x22] {for z in [0u8,1,255] {for velocity in [0u8,16,0x80,255] {
        let mut native=marker29_state();
        for (a,v) in [(0xdd0,9),(0xe20,kind),(0xd80,1),(0xdf0,1),(0xd10,0x40),(0xd00,0x40),
            (0xf70,z),(0xf80,velocity),(0xd50,velocity),(0xd40,velocity)] {native.write_wram_byte(a+slot,v);}
        native.write_wram_byte(0xfa0,slot as u8);native.write_wram_word(0x22,0x180);native.write_wram_word(0x20,0x180);
        native.sync_native_game_state_from_ram();
        let machine=marker29_machine(&native,if kind==0x0e {0x069c24}else{0x069e1f},slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_owners_work(&[0x06e92c,0x06e932,0x06e93e,0x06e96c],machine,
            |s|s.cpu.k==6 && (0xe92c..0xe98e).contains(&s.cpu.pc),
            || if kind==0x0e {native.sprite_0_e_snapdragon(slot);} else {native.sprite_22_ropa(slot);});
    }}}}
}
