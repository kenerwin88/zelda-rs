use super::*;

#[test]
fn prefix24_raven_wrappers_and_recoil_prefix_match_rom() {
    for slot in [0usize, 15] {
      for ai in 0..4 {
       for submodule in [0, 1] {
        for recoil in [0, 1, 8, 0x80] {
         for position in [0x40u16, 0x1000] {
          for owner in [0x06946e, 0x1ddd7b, 0x1de898, 0x06eacd, 0x1de8c5] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0),(0xd80,ai),(0xdf0,1),
                (0xea0,recoil),(0xcd2,0x80),(0xd00,0x40),(0xd10,0x40)] { n.write_wram_byte(a+slot,v); }
            n.write_wram_byte(0x11,submodule); n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x20,position); n.write_wram_word(0x22,position);
            n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n,0x06946e,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
                let pc = u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
                match owner {
                    0x06946e => (0x06946e..0x069473).contains(&pc),
                    0x1ddd7b => (0x1ddd7b..0x1ddd83).contains(&pc),
                    0x1de898 => (0x1de898..0x1de89d).contains(&pc),
                    0x06eacd => (0x06eacd..0x06ead1).contains(&pc),
                    _ => (0x1de8c5..0x1de8ca).contains(&pc) || recoil==0 && pc==0x1de943,
                }
            }, || n.sprite_00_raven_bounce(slot));
          }
         }
        }
       }
      }
    }
}

#[test]
fn prefix24_green_bush_guard_caller_and_states_match_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
      for ai in 0..4 {
       for delay in [0,1,16,32,39,40,48] {
        for direction in 0..4 {
         for submodule in [0,1] {
          for graphics in [0,14] {
           for ty in [0x47,0x49] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,ty),(0xd80,ai),(0xdf0,delay),
                (0xde0,direction),(0xeb0,direction),(0xdc0,graphics),(0xe40,0x80),
                (0xd00,0x40),(0xd10,0x40)] { n.write_wram_byte(a+slot,v); }
            n.write_wram_byte(0x11,submodule); n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x20,0x1000); n.write_wram_word(0x22,0x1000);
            n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n,0x05d1bf,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x05d1bf,s,|s| {
                let pc=s.cpu.pc;
                let own=s.cpu.k==5 && [(0xd1bf,0xd1ed),(0xd1f5,0xd203),
                    (0xd223,0xd252),(0xd277,0xd2be),(0xd2ce,0xd2e9)].iter().any(|&(a,b)|(a..b).contains(&pc));
                if own {
                    let taken=match s.cart.read(5,pc,0) {0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),
                        0x90=>Some(!s.cpu.c),0xb0=>Some(s.cpu.c),_=>None};
                    if let Some(taken)=taken {edges.borrow_mut().insert((pc,taken));}
                }
                own
            }, || n.sprite_47_green_bush_guard(slot));
           }
          }
         }
        }
       }
      }
    }
    for pc in [0xd1c2,0xd1c9,0xd1f8,0xd22a,0xd241,0xd280,0xd291,0xd298,0xd2b1,0xd2d4] {
        for taken in [false,true] { assert!(edges.borrow().contains(&(pc,taken)),"uncovered {pc:x}/{taken}"); }
    }
}

#[test]
fn prefix24_bush_soldier_draw_all_graphics_and_clipping_match_rom() {
    for slot in [0usize,15] {
      for graphics in 0..7 {
       for (x,y) in [(0x40u16,0x40u16),(0x40,0),(0x40,0xf0),(0x40,0x1000),(0x1000,0x40)] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,9),(0xdc0,graphics),(0xd10,x as u8),(0xd30,(x>>8)as u8),
            (0xd00,y as u8),(0xd20,(y>>8)as u8)] { n.write_wram_byte(a+slot,v); }
        n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x05d321,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x05d321,s,
            |s|s.cpu.k==5 && (0xd321..0xd381).contains(&s.cpu.pc),||n.bush_soldier_common_draw(slot));
       }
      }
    }
}

#[test]
fn prefix24_guard_preparation_all_subtypes_and_floors_match_rom() {
    for slot in [0usize,15] {
      for subtype in 0..32 {
       for indoors in [0,1] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,9),(0xe20,0x41),(0xe30,subtype),(0xd00,0x40),(0xd10,0x40),
            (0xdf0,1),(0xf20,indoors)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0x1b,indoors);n.write_wram_byte(0xfa0,slot as u8);
        n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);
        n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x068fd6,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x068fd6,s,
            |s|s.cpu.k==6 && (0x8fcf..0x903b).contains(&s.cpu.pc),|| {
                let _scope=crate::cycle_ledger::routine(0x068fd6);
                n.sprite_prep_standard_guard(slot);
            });
       }
      }
    }
}

#[test]
fn prefix24_guard_parry_call_and_heart_first_call_match_rom() {
    for slot in [0usize,15] {
      for floor in [0,1] {
       for paused in [0,1] {
        for entry in [0x05c25f,0x06cec0] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,if entry==0x06cec0 {0xd8}else{0x41}),
                (0xd00,0x40),(0xd10,0x40),(0xdf0,1),(0xf20,floor)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x11,paused);n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,entry,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(entry,s,
                |s|u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc)==entry,|| {
                    let _scope=crate::cycle_ledger::routine(entry);
                    if entry==0x05c25f {crate::cycle_ledger::blocks(true).block(entry);n.guard_parry_sword_attacks_far(slot);}
                    else {n.sprite_d8_heart(slot);}
                });
        }
       }
      }
    }
}

#[test]
fn prefix24_archer_scan_and_clear_movement_match_all_rom_paths() {
    for slot in [0usize,15] {
      for delay in [0,1,0x7f,0x80,0xff] {
       for direction in 0..4 {
        for wall in [0,1,0xff] {
         for velocity in [0,0x7f,0x80,0xff] {
          for entry in [0x05cf13,0x05cdd4] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x46),(0xdf0,delay),(0xde0,direction),
                (0xe70,wall),(0xd40,velocity),(0xd50,velocity),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,entry,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(entry,s,|s|s.cpu.k==5 &&
                if entry==0x05cf13 {(0xcf13..0xcf44).contains(&s.cpu.pc)}
                else {(0xcdd4..0xcddd).contains(&s.cpu.pc)},|| {
                    if entry==0x05cf13 {n.archer_scanning(slot);}else{n.soldier_move_if_clear(slot);}
                });
          }
         }
        }
       }
      }
    }
}

#[test]
fn prefix24_mini_moldorm_on_screen_body_all_paths_match_rom() {
    // The cartridge has exactly four history-base entries at $1D:F7CF.
    for slot in 0..4usize {
      for direction in 0..16 {
       for phase in [0,1,15,16,31] {
        for (x,y) in [(0x40u16,0x40u16),(0x40,0),(0x40,0xf0),(0x40,0x1000),(0x1000,0x40)] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x18),(0xde0,direction),(0xe80,phase),
                (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x1df822,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1df829,s,
                |s|s.cpu.k==0x1d && (0xf829..0xf943).contains(&s.cpu.pc),||n.moldorm_draw(slot));
        }
       }
      }
    }
}

#[test]
fn prefix24_giant_moldorm_entry_and_coordinate_owners_match_rom() {
    for slot in [0usize,15] {
      for submodule in [0,1] {
       for state in [4,9] {
        for ai in [0,3] {
         for aux in [0,16,79,80] {
          for owner in [0x069469,0x1dd6f6,0x1dd74e,0x0684bd] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,state),(0xe20,9),(0xd80,ai),(0xdf0,1),(0xf10,aux),
                (0xe40,0x80),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x11,submodule);n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_byte(0x46,1);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x069469,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
                let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
                match owner {0x069469=>(0x069469..0x06946e).contains(&pc),
                    0x1dd6f6=>(0x1dd6f6..0x1dd6fe).contains(&pc),
                    0x1dd74e=>(0x1dd74e..0x1dd754).contains(&pc),
                    _=>(0x0684bd..0x0684c1).contains(&pc)}
            },||n.sprite_09_moldorm_bounce(slot));
          }
         }
        }
       }
      }
    }
}

#[test]
fn prefix24_giant_moldorm_segment_explosion_all_gates_match_rom() {
    for slot in [0usize,15] {
      for state in [4,9] {
       for aux in [0,1,16,64,79,80,0xff] {
        for submodule in [0,1] {
         for pause in [0,1] {
            let mut n=marker29_state();
            n.write_wram_byte(0xdd0+slot,state);n.write_wram_byte(0xf10+slot,aux);
            n.write_wram_byte(0x11,submodule);n.write_wram_byte(0xfc1,pause);
            n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x1dd8f2,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1dd8f2,s,
                |s|s.cpu.k==0x1d && (0xd8f2..0xd913).contains(&s.cpu.pc),
                ||n.giant_moldorm_incremental_segment_explosion(slot));
         }
        }
       }
      }
    }
}

#[test]
fn prefix24_walking_zora_dispatch_wait_and_surface_match_rom() {
    for slot in [0usize,15] {
      for phase in 0..4 {
       for recoil in [0,1] {
        for delay in [0,1,63,127] {
         for submodule in [0,1] {
          for direction in 0..4 {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x56),(0xda0,phase),(0xea0,recoil),
                (0xdf0,delay),(0xde0,direction),(0xeb0,direction),(0xed0,128),
                (0xe40,0x80),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x11,submodule);n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x059d4a,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x059d4a,s,
                |s|s.cpu.k==5 && ((0x9d4a..0x9d77).contains(&s.cpu.pc)||(0x9d7f..0x9dd6).contains(&s.cpu.pc))
                    ||s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc),
                ||n.sprite_56_walking_zora(slot));
          }
         }
        }
       }
      }
    }
}
