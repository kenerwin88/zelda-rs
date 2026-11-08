use super::*;

fn state() -> ZeldaState {
    let mut n = ZeldaState::new();
    n.set_assets(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap()).unwrap();
    n.set_lag_free(true);
    n
}

fn machine(n: &ZeldaState, entry: u32, wide: bool) -> snes::Snes {
    let mut s = snes::Snes::new();
    s.cart.load(snes::CartType::LoRom, &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap(), 0x2000);
    s.ram.copy_from_slice(&n.wram_image());
    s.ram[0x1fd..0x200].copy_from_slice(&[0xff,0x7f,0]);
    s.cpu.e = false; s.cpu.unpack_flags(if wide {0} else {0x30});
    s.cpu.k = (entry >> 16) as u8; s.cpu.db = s.cpu.k;
    s.cpu.pc = entry as u16; s.cpu.sp = 0x1fc;
    s
}

#[test]
fn prefix33_secret_lookup_every_rom_record_and_missing_screen_matches_rom() {
    let mut cases = 0;
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    let base = state();
    let offsets = base.asset_raw(157).unwrap();
    let records = base.asset_raw(158).unwrap();
    for screen in 0..=255u8 {
        let mut positions = vec![0x7ffe];
        if screen < 0x80 {
            let mut ptr = read_word_from_slice(offsets,screen as usize * 2) as usize;
            loop {
                let pos = read_word_from_slice(records,ptr);
                if pos == 0xffff { break; }
                positions.push(pos & 0x7fff); ptr += 3;
            }
        }
        for pos in positions {
            for event in [0,2] {
                for follower in [0,13] {
                    for powder in [0,0x40] {
                        let mut n = state();
                        n.write_wram_word(0x8a,u16::from(screen));
                        n.write_wram_byte(0xf280+screen as usize,event);
                        n.write_wram_byte(0xf3cc,follower);
                        n.write_wram_byte(0x301,powder);
                        n.sync_native_game_state_from_ram();
                        let mut s = machine(&n,0x1bc8a4,true); s.cpu.x = pos;
                        crate::cycle_ledger::assert_rom_marker_work(0x1bc8a4,s,
                            |s| {
                                if s.cpu.k == 0x1b {
                                    let t = match s.cpu.pc {
                                        0xc8b4|0xc8e3 => Some(s.cpu.c),
                                        0xc8ce|0xc8de|0xc8fe => Some(s.cpu.z),
                                        0xc8d5|0xc909|0xc910|0xc91c => Some(!s.cpu.z),
                                        0xc8ee => Some(!s.cpu.c), _ => None,
                                    };
                                    if let Some(t) = t { edges.borrow_mut().insert((s.cpu.pc,t)); }
                                }
                                s.cpu.k == 0x1b && (0xc8a4..0xc943).contains(&s.cpu.pc)
                            }, || { n.overworld_reveal_secret(pos); });
                        cases += 1;
                    }
                }
            }
        }
    }
    for pc in [0xc8b4,0xc8ce,0xc8d5,0xc8e3,0xc8ee,0xc8fe,0xc909,0xc910,0xc91c] {
        for t in [false,true] { assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}"); }
    }
    eprintln!("prefix33 secret lookup: {cases} whole calls; edges {:?}",edges.borrow());
}

#[test]
fn prefix33_powder_adjustment_and_memorized_tile_guards_match_rom() {
    for powder in 0..=255u8 {
        let mut n = state(); n.write_wram_byte(0x301,powder);
        n.sync_native_game_state_from_ram();
        crate::cycle_ledger::assert_rom_marker_work(0x1bc943,machine(&n,0x1bc943,true),
            |s| s.cpu.k == 0x1b && (0xc943..0xc952).contains(&s.cpu.pc),
            || n.adjust_secret_for_powder());
    }
    for value in [0,0xdc5,0xdc7,0xdc8,0xdc9,0xffff] {
        for pos in [0,0x7e,0x1000,0xfffe] {
            for index in [0,2,0x1fe] {
                let mut n = state(); n.write_wram_word(0x4ac,index);
                n.sync_native_game_state_from_ram();
                let mut s = machine(&n,0x0edd40,true); s.cpu.a = value; s.cpu.x = pos;
                let source = crate::cycle_ledger::assert_rom_marker_work(0x0edd40,s,
                    |s| s.cpu.k == 0x0e && (0xdd40..0xdd61).contains(&s.cpu.pc),
                    || n.overworld_memorize_map16_change(pos,value));
                for range in [0x4ac..0x4ae,0xf800..0xfc00] {
                    assert_eq!(&n.ram[range.clone()],&source.snes().ram[range]);
                }
            }
        }
    }
}

#[test]
fn prefix33_tool_mutation_suffix_excludes_prefix_and_all_children() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for attr in [0,0x34,0x71,0x35,0x10d,0x10f,0xe1,0xe2,0xda,0xf8,0x10e,0x37e,0x36,0x72a,0x21b] {
        for hand in [0,2,0x40] {
            for mode in [0,1,2] {
                for (screen,pos) in [(0,0),(0,0x36a),(0x2a,0),(0x2a,0x492)] {
                    let mut n = state();
                    for (a,v) in [(0x708,0),(0x70a,0x3f0),(0x70c,0),(0x70e,0x7e),(0x8a,screen),(0x2000+pos,attr)] {
                        n.write_wram_word(a,v as u16);
                    }
                    n.write_wram_byte(0x301,hand); n.write_wram_byte(0x37a,mode);
                    n.sync_native_game_state_from_ram();
                    let mut s = machine(&n,0x1bbd7a,false);
                    let x = (pos & 0x7e) as u16; let y = (pos >> 3 & 0x3f0) as u16;
                    s.ram[0..2].copy_from_slice(&y.to_le_bytes());
                    s.ram[2..4].copy_from_slice(&x.to_le_bytes());
                    let entered = std::cell::Cell::new(false);
                    crate::cycle_ledger::assert_rom_marker_work(0x1bbe49,s,
                        |s| {
                            if s.cpu.k == 0x1b && [0xbe49,0xbe5d,0xbe8d,0xbec3].contains(&s.cpu.pc) { entered.set(true); }
                            if s.cpu.k == 0x1b && entered.get() {
                                let t = match s.cpu.pc {
                                    0xbe4e|0xbe53|0xbe66|0xbeb5 => Some(!s.cpu.z),
                                    0xbe96|0xbf0e => Some(s.cpu.z),0xbebf => Some(s.cpu.c),_ => None,
                                };
                                if let Some(t) = t {edges.borrow_mut().insert((s.cpu.pc,t));}
                            }
                            s.cpu.k == 0x1b && entered.get() && (0xbe49..0xbf1e).contains(&s.cpu.pc)
                        }, || { n.handle_item_tile_action_overworld(x,y); });
                }
            }
        }
    }
    for pc in [0xbe4e,0xbe53,0xbe66,0xbe96,0xbeb5,0xbebf,0xbf0e] {
        for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");}
    }
}

#[test]
fn prefix33_special_overworld_exit_all_tiles_screens_and_scan_edges_match_rom() {
    let base = state();
    let map8 = base.asset_raw(70).unwrap();
    let mut tiles = Vec::new();
    for wanted in SPECIAL_SWITCH_AREA_B_TILE_IDS.into_iter().map(Some).chain([None]) {
        let tile = map8.chunks_exact(8).position(|t| {
            let id = u16::from_le_bytes([t[0],t[1]]) & 0x1ff;
            wanted.map_or(!SPECIAL_SWITCH_AREA_B_TILE_IDS.contains(&id),|w|id == w)
        }).unwrap() as u16;
        tiles.push(tile);
    }
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for screen in 0..=255u8 {
        for &tile in &tiles {
            let mut n = state();
            n.write_wram_word(0x20,0); n.write_wram_word(0x22,0);
            n.write_wram_word(0x708,0); n.write_wram_word(0x70a,0);
            n.write_wram_word(0x70c,0); n.write_wram_word(0x70e,0);
            n.write_wram_word(0x2000,tile); n.write_wram_byte(0x8a,screen);
            n.sync_native_game_state_from_ram();
            let source = crate::cycle_ledger::assert_rom_marker_work(0x0edee3,machine(&n,0x0edee3,false),
                |s| {
                    if s.cpu.k == 0x0e {
                        let t = match s.cpu.pc {
                            0xdef8 => Some(s.cpu.n),0xdefe|0xdf06 => Some(!s.cpu.z),
                            0xdf14|0xdf20 => Some(!s.cpu.c),_ => None,
                        };
                        if let Some(t) = t {edges.borrow_mut().insert((s.cpu.pc,t));}
                    }
                    s.cpu.k == 0x0e && (0xdee0..0xdf2f).contains(&s.cpu.pc)
                }, || n.overworld_check_special_overworld_exit());
            for a in [0x11,0xb0,0x67,0x418,0x69c] {
                assert_eq!(n.read_wram_byte(a),source.snes().ram[a]);
            }
        }
    }
    for pc in [0xdef8,0xdefe,0xdf06,0xdf14,0xdf20] {
        for t in [false,true] { assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}"); }
    }
}

#[test]
fn prefix33_scroll_exit_wrapper_excludes_scroll_and_exit_children() {
    for direction in [0,1,2,4,8] {
        let mut n = state();
        n.write_wram_byte(0x416,direction);
        n.sync_native_game_state_from_ram();
        crate::cycle_ledger::assert_rom_marker_work(0x02ab7b,machine(&n,0x02ab7b,false),
            |s| s.cpu.k == 2 && (0xab7b..0xab88).contains(&s.cpu.pc),
            || n.ScrollAndCheckForSOWExit());
    }
}
