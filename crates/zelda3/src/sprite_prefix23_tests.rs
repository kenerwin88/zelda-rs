use super::*;

#[test]
fn prefix23_dungeon_sprite_entries_cover_all_prefix_paths_in_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    let mut cases = 0;
    for entry in [0x06cba2, 0x058090, 0x058531, 0x05874d] {
      for slot in [0usize, 15] {
       for submodule in [0, 2] {
        for ai in 0..4 {
         for pause in [0, 1] {
          for direction in 0..4 {
           for (x, y) in [(0x40u16, 0x40u16), (0x80, 0x40), (0x40, 0x80), (0x1000, 0x1000)] {
            let owners: &[u32] = if entry == 0x058531 { &[entry, 0x05f93f, 0x06eaa0] } else { &[entry] };
            for &owner in owners {
                let mut n = marker29_state();
                let ty = match entry { 0x06cba2 => 0x71, 0x058531 => 0x63, 0x05874d => 0x64, _ => 0x66 };
                for (a, v) in [(0xdd0, 9), (0xe20, ty), (0xd80, ai), (0xdf0, 1),
                    (0xf00, pause), (0xde0, direction), (0xeb0, 1), (0xd10, x as u8),
                    (0xd30, (x >> 8) as u8), (0xd00, y as u8), (0xd20, (y >> 8) as u8)] {
                    n.write_wram_byte(a + slot, v);
                }
                n.write_wram_byte(0xdd1, 6); // Pit's unpriced suffix returns at its child-state gate.
                n.write_wram_word(0xfd8, x); n.write_wram_word(0xfda, y);
                n.write_wram_byte(0x11, submodule); n.write_wram_byte(0xfa0, slot as u8);
                n.write_wram_word(0x20, 0x40); n.write_wram_word(0x22, 0x40);
                n.sync_native_game_state_from_ram();
                let s = marker29_machine(&n, entry, slot).expect("unmodified ROM");
                crate::cycle_ledger::assert_rom_marker_work(owner, s,
                    |s| {
                        let pc = (u32::from(s.cpu.k) << 16) | u32::from(s.cpu.pc);
                        let taken = match pc {
                            0x06cba5 | 0x06cbb2 => Some(s.cpu.z),
                            0x05853b | 0x058544 => Some(s.cpu.c), _ => None,
                        };
                        if let Some(taken) = taken { edges.borrow_mut().insert((pc, taken)); }
                        match owner {
                            0x06cba2 => (0x06cba2..0x06cbbc).contains(&pc),
                            0x058090 => (0x058090..0x0580b2).contains(&pc),
                            0x058531 => (0x058531..0x058552).contains(&pc),
                            0x05874d => (0x05874d..0x058753).contains(&pc),
                            0x05f93f => (0x05f93f..0x05f944).contains(&pc),
                            _ => (0x06eaa0..0x06eaa4).contains(&pc),
                        }
                    }, || {
                        let _root = crate::cycle_ledger::routine(entry);
                        match entry {
                            0x06cba2 => n.sprite_71_leever(slot),
                            0x058090 => n.sprite_66_wall_cannon_vertical_left(slot),
                            0x05874d => n.sprite_64_debirando(slot),
                            _ => n.sprite_63_debirando_pit(slot),
                        }
                    });
                cases += 1;
            }
           }
          }
         }
        }
       }
      }
    }
    for pc in [0x06cba5, 0x06cbb2, 0x05853b, 0x058544] {
        for taken in [false, true] { assert!(edges.borrow().contains(&(pc, taken)), "uncovered {pc:x}/{taken}"); }
    }
    eprintln!("prefix23 dungeon entries: {cases} unmodified-ROM fixtures; all prefix branch pairs");
}

#[test]
fn prefix23_plaque_two_call_prefix_matches_active_and_unwinding_paths() {
    for slot in [0usize, 15] {
        for submodule in [0, 1] {
            for position in [0x40u16, 0x1000] {
                let mut n = marker29_state();
                for (a, v) in [(0xdd0, 9), (0xe20, 0xb3), (0xd10, position as u8),
                    (0xd30, (position >> 8) as u8), (0xd00, 0x40)] { n.write_wram_byte(a + slot, v); }
                n.write_wram_byte(0x11, submodule); n.write_wram_byte(0xfa0, slot as u8);
                n.write_wram_byte(0x2f, 2); // The unpriced interaction suffix returns on facing.
                n.sync_native_game_state_from_ram();
                let s = marker29_machine(&n, 0x1ee044, slot).expect("unmodified ROM");
                crate::cycle_ledger::assert_rom_marker_work(0x1ee044, s,
                    |s| s.cpu.k == 0x1e && (0xe044..0xe04b).contains(&s.cpu.pc), || {
                        let _scope = crate::cycle_ledger::routine(0x1ee044);
                        n.sprite_b3_pedestal_plaque(slot);
                    });
            }
        }
    }
}

#[test]
fn prefix23_vulture_wrappers_draw_and_inactive_prefix_match_rom() {
    let mut cases = 0;
    for slot in [0usize, 15] {
        for submodule in [0, 1] {
            for ai in [0, 1] {
                for graphics in 0..4 {
                    for position in [0x40u16, 0x1000] {
                        for owner in [0x069473, 0x1ddc72, 0x1ddc7a, 0x1ddd5e, 0x1d8549] {
                            let mut n = marker29_state();
                            for (a, v) in [(0xdd0, 9), (0xe20, 1), (0xd80, ai),
                                (0xdc0, graphics), (0xd10, position as u8),
                                (0xd30, (position >> 8) as u8), (0xd00, 0x40)] {
                                n.write_wram_byte(a + slot, v);
                            }
                            n.write_wram_byte(0x11, submodule);
                            n.write_wram_byte(0xfa0, slot as u8);
                            n.write_wram_word(0x20, 0x40); n.write_wram_word(0x22, 0x40);
                            n.sync_native_game_state_from_ram();
                            let s = marker29_machine(&n, 0x069473, slot).expect("unmodified ROM");
                            crate::cycle_ledger::assert_rom_marker_work(owner, s,
                                |s| match owner {
                                    0x069473 => s.cpu.k == 6 && (0x9473..0x9478).contains(&s.cpu.pc),
                                    0x1ddc72 => s.cpu.k == 0x1d && (0xdc72..0xdc7a).contains(&s.cpu.pc),
                                    0x1ddc7a => s.cpu.k == 0x1d && (0xdc7a..0xdc88).contains(&s.cpu.pc),
                                    0x1ddd5e => s.cpu.k == 0x1d && (0xdd5e..0xdd7b).contains(&s.cpu.pc),
                                    _ => s.cpu.k == 0x1d && (0x8549..0x854e).contains(&s.cpu.pc),
                                }, || {
                                    let _root = crate::cycle_ledger::routine(0x069473);
                                    n.sprite_01_vulture_bounce(slot);
                                });
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    eprintln!("prefix23 vulture: {cases} unmodified-ROM owner fixtures; active AI remains unpriced");
}

#[test]
fn prefix23_desert_statue_caller_all_paths_match_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    let mut cases = 0;
    for slot in [0usize, 15] {
      for delay in [0, 1] {
       for ai in [0, 1, 128] {
        for cache in [0, 1] {
         for direction in 0..4 {
          for subtype in [0, 1] {
           for g in [128, 129] {
            for incap in [0, 1] {
             for position in [0x40u16, 0x80, 0x1000] {
              for tiles in [0, 1] {
                let mut n = marker29_state();
                n.ram[0x12000..0x14000].fill(tiles);
                for (a, v) in [(0xdd0, 9), (0xe20, 0x57), (0xdf0, delay),
                    (0xd80, ai), (0xde0, direction), (0xe80, subtype), (0xed0, g),
                    (0xd10, position as u8), (0xd30, (position >> 8) as u8), (0xd00, 0x40)] {
                    n.write_wram_byte(a + slot, v);
                }
                n.write_wram_byte(0x2f0, cache); n.write_wram_byte(0x46, incap);
                n.write_wram_word(0xfd8, position); n.write_wram_word(0xfda, 0x40);
                n.write_wram_byte(0xf20 + slot, tiles);
                // Negative flags2 preserves overlap carry without applying
                // damage: the caller can then reach its zero-incapacity path.
                n.write_wram_byte(0xe40 + slot, 0x80);
                n.write_wram_byte(0xfa0, slot as u8);
                n.write_wram_word(0x20, 0x40); n.write_wram_word(0x22, 0x40);
                n.sync_native_game_state_from_ram();
                let s = marker29_machine(&n, 0x05956d, slot).expect("unmodified ROM");
                crate::cycle_ledger::assert_rom_marker_work(0x05956d, s,
                    |s| {
                        if s.cpu.k == 5 {
                            let taken = match s.cpu.pc {
                                0x9578 | 0x95a2 => Some(!s.cpu.c),
                                0x9586 | 0x958d | 0x9592 | 0x95a6 | 0x95e7 | 0x95f1 => Some(!s.cpu.z),
                                0x958b => Some(s.cpu.n), 0x95cf => Some(s.cpu.z), _ => None,
                            };
                            if let Some(taken) = taken { edges.borrow_mut().insert((s.cpu.pc, taken)); }
                        }
                        s.cpu.k == 5 && (0x956d..0x95fc).contains(&s.cpu.pc)
                    }, || n.sprite_57_desert_statue(slot));
                cases += 1;
              }
             }
            }
           }
          }
         }
        }
       }
      }
    }
    for pc in [0x9578, 0x9586, 0x958b, 0x958d, 0x9592, 0x95a2,
        0x95a6, 0x95cf, 0x95e7, 0x95f1] {
        for taken in [false, true] { assert!(edges.borrow().contains(&(pc, taken)), "uncovered {pc:x}/{taken}"); }
    }
    eprintln!("prefix23 statue: {cases} unmodified-ROM fixtures; every conditional both ways");
}
