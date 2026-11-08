use super::*;

#[test]
fn prefix25_helmasaur_draw_eyes_body_legs_mouth_and_correction_match_rom() {
    for slot in [0usize,15] {
     for delay in 0..128 {
      for paused in [0,1] {
       for (x,y) in [(0x40u16,0x40u16),(0,0),(0xff,0xf0),(0x1000,0x40),(0x40,0x1000)] {
        for owner in [0x1e853b,0x1e856b,0x1e87e5,0x1e8805,0x1e88bc,0x06feb4] {
         let mut n=marker29_state();
         for(a,v)in[(0xdd0,9),(0xe20,0x92),(0xe10,delay),(0xdb0,delay&3),(0xec0,delay&3),(0xe80,delay&31),
           (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
         n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.write_wram_byte(0xb0c,delay&31);
         n.write_wram_word(0xfd8,x);n.write_wram_word(0xfda,y);n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);
         n.sync_native_game_state_from_ram();n.helmasaur_king_initialize(slot);
         // Isolate the body entry: the mask also calls its shared 87F0 tail.
         let s=marker29_machine(&n,if owner==0x1e87e5 {owner}else{0x1e853b},slot).unwrap();
         crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
           let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
           match owner {0x1e853b=>(0x1e853b..0x1e855f).contains(&pc),0x1e856b=>(0x1e856b..0x1e85c6).contains(&pc),
             0x1e87e5=>(0x1e87e5..0x1e87f5).contains(&pc),0x1e8805=>(0x1e8805..0x1e889c).contains(&pc),
             0x1e88bc=>(0x1e88bc..0x1e88f0).contains(&pc),_=>(0x06feb4..0x06febc).contains(&pc)}
         },||if owner==0x1e87e5 {
             let mut info=PrepOamCoordsRet{x,y,r4:0,flags:0};n.sprite_draw_king_helmasaur_body(slot,&mut info);
         }else{n.helmasaur_king_draw(slot);});
        }
       }
      }
     }
    }
}

#[test]
fn prefix25_guard_head_and_body_all_graphics_clips_and_crossings_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
     for graphics in 0..24 {
      for head in 0..4 {
       for offset in [0,2,3,6] {
        for (x,y) in [(0u16,0u16),(0x40,0x40),(0xff,0xf0),(0x100,0x100),(0xffff,0xffff)] {
         for owner in [0x05b160,0x05b3cd] {
          let mut n=marker29_state();n.write_wram_byte(0xdc0+slot,graphics);n.write_wram_byte(0xeb0+slot,head);
          n.write_wram_word(0,x);n.write_wram_word(2,y);n.write_wram_byte(5,0x23);n.sync_native_game_state_from_ram();
          let info=PrepOamCoordsRet{x,y,r4:0,flags:0x23};
          let mut s=marker29_machine(&n,owner,slot).unwrap();s.cpu.y=(offset*4)as u16;
          crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
            let own=s.cpu.k==5 && if owner==0x05b160 {(0xb160..0xb1a3).contains(&s.cpu.pc)}else{(0xb3cd..0xb440).contains(&s.cpu.pc)};
            if own {let t=match s.cart.read(5,s.cpu.pc,0){0x90=>Some(!s.cpu.c),0xd0=>Some(!s.cpu.z),0x10=>Some(!s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
            own
          },||if owner==0x05b160 {n.sprite_draw_guard_head(slot,&info,offset);}else{n.sprite_draw_bnc_body(slot,&info,offset);});
         }
        }
       }
      }
     }
    }
    for pc in [0xb180,0xb40c,0xb435,0xb43c] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix25_guard_spear_directions_types_clips_and_crossings_match_rom() {
    for slot in [0usize,15] {
     for direction in 0..4 {
      for a in [0,1,0xfe,0xff] {
       for ty in [0x41,0x45,0x47,0x48,0x49] {
        for offset in [0,1] {
         for (x,y) in [(0u16,0u16),(0x40,0x40),(0xff,0xf0),(0x100,0x100),(0xffff,0xffff)] {
          let mut n=marker29_state();for(addr,v)in[(0xde0,direction),(0xd90,a),(0xe20,ty)] {n.write_wram_byte(addr+slot,v);}
          n.write_wram_word(0,x);n.write_wram_word(2,y);n.write_wram_byte(5,0x23);n.sync_native_game_state_from_ram();
          let info=PrepOamCoordsRet{x,y,r4:0,flags:0x23};let mut s=marker29_machine(&n,0x05cd54,slot).unwrap();s.cpu.y=(offset*4)as u16;s.cpu.a=u16::from(a);
          crate::cycle_ledger::assert_rom_marker_work(0x05cd54,s,|s|s.cpu.k==5&&(0xcd54..0xcdd4).contains(&s.cpu.pc),||n.sprite_draw_guard_spear(slot,&info,offset));
         }
        }
       }
      }
     }
    }
}

#[test]
fn prefix25_javelin_draw_fresh_spear_and_shadow_entries_match_rom() {
    for slot in [0usize,15] {
     for graphics in 0..24 {
      for direction in 0..4 {
       for shadow in [0,0x10] {
        for (x,y) in [(0x40u16,0x40u16),(0,0),(0xff,0xf0),(0x1000,0x40),(0x40,0x1000)] {
         for owner in [0x05d192,0x05cd48,0x06dc5c] {
          let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0x48),(0xdc0,graphics),(0xde0,direction),(0xeb0,direction),(0xe60,shadow),
            (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
          n.write_wram_byte(0xfa0,slot as u8);n.write_wram_word(0xfd8,x);n.write_wram_word(0xfda,y);n.sync_native_game_state_from_ram();
          let s=marker29_machine(&n,0x05d192,slot).unwrap();
          crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
           let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
           match owner {0x05d192=>(0x05d192..0x05d1ac).contains(&pc)||(0x05c68c..0x05c69e).contains(&pc),0x05cd48=>(0x05cd48..0x05cd4f).contains(&pc),_=>(0x06dc5c..0x06dc64).contains(&pc)}
          },||n.javelin_trooper_draw(slot));
         }
        }
       }
      }
     }
    }
}

#[test]
fn prefix25_red_javelin_caller_and_shared_prefix_all_paths_match_rom() {
    for slot in [0usize,15] {
     for ai in 0..6 {
      for delay in [0,1,60] {
       for aux in [0,1] {
        for direction in 0..4 {
         for graphics in [0,19,20,23] {
          for paused in [0,1] {
           let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0x48),(0xd80,ai),(0xdf0,delay),(0xe00,aux),
              (0xde0,direction),(0xeb0,direction),(0xdc0,graphics),(0xe40,0x80),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
           n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);n.sync_native_game_state_from_ram();
           let s=marker29_machine(&n,0x05cde1,slot).unwrap();let random=std::cell::RefCell::new(Vec::new());
           crate::cycle_ledger::assert_rom_marker_work(0x05cde1,s,|s| {
              if s.cpu.k==13&&s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
              s.cpu.k==5 && ((0xcde1..0xcdff).contains(&s.cpu.pc)||(0xce1b..0xce68).contains(&s.cpu.pc))
           },|| {n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_48_red_javelin_guard(slot);});
          }
         }
        }
       }
      }
     }
    }
}

#[test]
fn prefix25_water_terminal_helpers_and_transmute_guard_match_rom() {
    for slot in [0usize,15] {
     for ty in [0x26,0xe8,0xec] {
      for x in [0u16,0x40,0x100,0x1ff] {
       for owner in [0x06e0ab,0x06e0f6,0x06e095,0x06e22f] {
        let mut n=marker29_state();n.write_wram_byte(0xe20+slot,ty);n.write_wram_byte(0xdc0+slot,0);
        n.write_wram_byte(0xd10+slot,x as u8);n.write_wram_byte(0xd30+slot,(x>>8)as u8);
        n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,if owner==0x06e095 {0x06e0f6}else{owner},slot).unwrap();
        let random=std::cell::RefCell::new(Vec::new());
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
            if s.cpu.k==13&&s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
            s.cpu.k==6 && match owner {0x06e0ab=>(0xe0ab..0xe0bf).contains(&s.cpu.pc),0x06e0f6=>(0xe0f6..0xe115).contains(&s.cpu.pc),0x06e095=>(0xe095..0xe09b).contains(&s.cpu.pc),_=>(0xe22f..0xe239).contains(&s.cpu.pc)||(s.cpu.pc==0xe26d && ty!=0xec)}
        },|| {
            n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();
            match owner {0x06e0ab=>n.sprite_func8(slot),0x06e0f6|0x06e095=>n.sprite_func22(slot),_=>n.throwable_scenery_transmute_if_valid(slot)}
        });
       }
      }
     }
    }
}

#[test]
fn prefix25_zora_speed_wrapper_and_child_match_rom() {
    for slot in [0usize,15] {
     for speed in [0u8,1,8,255] {
      for (x,y) in [(0u16,0u16),(0x40,0x40),(0x80,0x20),(0x20,0x80),(0xff,0x40)] {
       for owner in [0x06ea12,0x06ea04] {
        let mut n=marker29_state();n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);n.sync_native_game_state_from_ram();
        let mut s=marker29_machine(&n,0x06ea12,slot).unwrap();s.cpu.a=u16::from(speed);
        let source=crate::cycle_ledger::assert_rom_marker_work(owner,s,|s|s.cpu.k==6 &&
           if owner==0x06ea12 {(0xea12..0xea1a).contains(&s.cpu.pc)}else{(0xea04..0xea12).contains(&s.cpu.pc)},
           ||n.sprite_apply_speed_towards_link_long_marked(slot,speed));
        for a in [0xd40,0xd50] {assert_eq!(n.read_wram_byte(a+slot),source.snes().ram[a+slot]);}
       }
      }
     }
    }
}

#[test]
fn prefix25_walking_zora_walking_all_paths_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
     for frame in [0,1,15,16,31,32] {
      for z in [0,1,127,128,255] {
       for velocity in [0,0xf0,0x80,0x7f] {
        for tile in [0,8,9,0x20] {
         for paused in [0,1] {
          let mut n=marker29_state();
          for(a,v)in[(0xdd0,9),(0xe20,0x56),(0xda0,2),(0xf70,z),(0xf80,velocity),
            (0xde0,frame&3),(0xeb0,frame&3),(0xe40,0x80),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
          n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0x1a,frame);n.write_wram_byte(0x11,paused);
          let mut image=n.wram_image().into_owned(); image[0x12000..0x14000].fill(tile);image[0xfa5]=tile;
          n.import_wram_image(&image);n.sync_native_game_state_from_ram();
          let s=marker29_machine(&n,0x059dd6,slot).unwrap();
          crate::cycle_ledger::assert_rom_marker_work(0x059dd6,s,|s| {
             let own=s.cpu.k==5&&(0x9dd6..0x9e66).contains(&s.cpu.pc);
             if own {let t=match s.cart.read(5,s.cpu.pc,0){0x10=>Some(!s.cpu.n),0xd0=>Some(!s.cpu.z),_=>None};
                if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
             own
          },|| { // Enter at the handler, excluding the dispatcher already proved in hunt 24.
              n.sprite_56_walking_zora(slot);
          });
         }
        }
       }
      }
     }
    }
    for pc in [0x9dff,0x9e06,0x9e16,0x9e24,0x9e3a,0x9e44] {
        for taken in [false,true] {assert!(edges.borrow().contains(&(pc,taken)),"uncovered {pc:x}/{taken}");}
    }
}

#[test]
fn prefix25_walking_zora_thrown_all_paths_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
     for state in [9,10,11] {
      for frame in 0..4 {
       for g in [0,1,47,48,255] {
        for recoil in [0,1,0x80] {
         for z in [0,16] {
          let mut n=marker29_state();
          for(a,v)in[(0xdd0,state),(0xe20,0x56),(0xda0,3),(0xed0,g),(0xea0,recoil),
            (0xf70,z),(0xe40,0x80),(0xd00,0x40),(0xd10,if frame&2==0{0}else{255})] {n.write_wram_byte(a+slot,v);}
          n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x1a,frame);
          n.sync_native_game_state_from_ram();
          let s=marker29_machine(&n,0x059d4a,slot).unwrap();
          let random=std::cell::RefCell::new(Vec::new());
          crate::cycle_ledger::assert_rom_marker_work(0x059e66,s,|s| {
             if s.cpu.k==13&&s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
             let own=s.cpu.k==5&&(0x9e66..0x9edb).contains(&s.cpu.pc);
             if own {let t=match s.cart.read(5,s.cpu.pc,0){0xd0=>Some(!s.cpu.z),0xb0=>Some(s.cpu.c),_=>None};
                if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
             own
          },|| {n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_56_walking_zora(slot);});
         }
        }
       }
      }
     }
    }
    for pc in [0x9e6e,0x9e73,0x9e84,0x9e91,0x9e97] {
        for taken in [false,true] {assert!(edges.borrow().contains(&(pc,taken)),"uncovered {pc:x}/{taken}");}
    }
}

#[test]
fn prefix25_zora_shadow_suffix_all_paths_match_rom() {
    for slot in [0usize,15] {
     for z in [0,1,255] {
      for tile in [0,8,9,0x20] {
       let mut n=marker29_state();n.write_wram_byte(0xf70+slot,z);n.write_wram_byte(0xfa5,tile);n.sync_native_game_state_from_ram();
       let s=marker29_machine(&n,0x059edb,slot).unwrap();
       crate::cycle_ledger::assert_rom_marker_work(0x059edb,s,|s|s.cpu.k==5&&(0x9edb..0x9eee).contains(&s.cpu.pc),||n.walking_zora_adjust_shadow(slot));
      }
     }
    }
}

#[test]
fn prefix25_beetle_and_firebar_preparation_match_rom() {
    for slot in 0..16 {
     for x in 0..=255 {
      for owner in [0x069122,0x0691b4] {
       let mut n=marker29_state();
       n.write_wram_byte(0xd10+slot,x); n.write_wram_byte(0xda0+slot,x);
       n.sync_native_game_state_from_ram();
       let s=marker29_machine(&n,owner,slot).unwrap();
       let source=crate::cycle_ledger::assert_rom_marker_work(owner,s,|s|s.cpu.k==6 &&
          if owner==0x069122 {(0x9122..0x9151).contains(&s.cpu.pc)} else {(0x91b4..0x91ba).contains(&s.cpu.pc)||(0x8ba7..0x8bab).contains(&s.cpu.pc)},
          ||if owner==0x069122 {n.sprite_prep_hardhat_beetle(slot);}else{n.sprite_prep_fire_bar(slot);});
       for a in [0xf50,0xe50,0xd90,0xd80,0xbe0,0xcd2,0xda0,0xba0] {
           assert_eq!(n.read_wram_byte(a+slot),source.snes().ram[a+slot],"{owner:x}/{a:x}");
       }
      }
     }
    }
}

#[test]
fn prefix25_firebar_correction_wrapper_matches_rom() {
    for slot in [0usize,15] {
     for x in [0x40u16,0x1000] {
      for angle in [0u16,127,128,255,256,511] {
       let mut n=marker29_state();
       for(a,v)in[(0xdd0,9),(0xe20,0x7e),(0xd90,angle as u8),(0xda0,(angle>>8)as u8),
          (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
       n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0x11,1); n.sync_native_game_state_from_ram();
       let s=marker29_machine(&n,0x1ed049,slot).unwrap();
       crate::cycle_ledger::assert_rom_marker_work(0x06feb4,s,|s|s.cpu.k==6&&(0xfeb4..0xfebc).contains(&s.cpu.pc),||n.firebar_main(slot));
      }
     }
    }
}

#[test]
fn prefix25_thrown_interaction_prefix_and_peer_suffix_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
     for ty in [0x26,0x7e,0xe8,0x4a,0xd2] {
      for state in [9,11] {
       for wall in [0,1,4,15] {
        for z in [0,1,16,255] {
         for velocity in [0,0xf0,0x80,0x7f] {
          for tile in [0,8,9,0x20] {
          for owner in [0x06e02a,0x06e149,0x06dff2,0x06e07a] {
           let mut n=marker29_state();
           if velocity==0x80 {for j in 0..16 {n.write_wram_byte(0xdd0+j,9);}}
           for(a,v)in[(0xdd0,state),(0xe20,ty),(0xe70,wall),(0xf70,z),(0xf80,velocity),
                (0xd00,0x40),(0xd10,0x40),(0xe90,0)] {n.write_wram_byte(a+slot,v);}
           n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0xfa5,0);
           n.write_wram_byte(0x1fa3c+slot,wall&1);
           n.write_wram_byte(0xb6b+slot,if wall==1 {0x11}else if wall==4 {0x13}else{0});
           n.write_wram_byte(0xd40+slot,if wall==0 {0xff}else{16});
           n.write_wram_byte(0xd50+slot,if wall==0 {0xfe}else{16});
           let mut image=n.wram_image().into_owned();image[0x12000..0x14000].fill(tile);image[0xfa5]=tile;n.import_wram_image(&image);
           n.sync_native_game_state_from_ram();
           let s=marker29_machine(&n,if owner==0x06dff2 {owner}else{0x06e02a},slot).unwrap();
           let random=std::cell::RefCell::new(Vec::new());
           crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
                if s.cpu.k==13 && s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                let own=s.cpu.k==6 && match owner {0x06e02a=>(0xe02a..0xe07a).contains(&s.cpu.pc),0x06e149=>(0xe149..0xe164).contains(&s.cpu.pc),0x06e07a=>[(0xe07a,0xe095),(0xe09b,0xe0ab),(0xe0bf,0xe0f6),(0xe115,0xe149)].iter().any(|&(a,b)|(a..b).contains(&s.cpu.pc)),_=>(0xdff2..0xdffa).contains(&s.cpu.pc)};
                if own {let t=match s.cart.read(6,s.cpu.pc,0){0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0xb0=>Some(s.cpu.c),0x90=>Some(!s.cpu.c),0x10=>Some(!s.cpu.n),0x30=>Some(s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
                own
           },|| {n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();
                if owner==0x06dff2 {n.thrown_sprite_tile_and_sprite_interaction_long(slot);}else{n.thrown_sprite_tile_and_sprite_interaction(slot);}});
          }
          }
         }
        }
       }
      }
     }
    }
    for pc in [0xe02f,0xe039,0xe052,0xe061,0xe075,0xe082,0xe089,0xe0a3,0xe0a9,0xe0c1,0xe0cb,0xe0d3,0xe0e3,0xe0ea,0xe0f1,0xe118,0xe120,0xe133,0xe144,0xe14e,0xe154,0xe15e] {
        for taken in [false,true] {assert!(edges.borrow().contains(&(pc,taken)),"uncovered {pc:x}/{taken}");}
    }
}

#[test]
fn prefix25_keese_caller_all_paths_match_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize, 15] {
      for ai in [0, 1] {
       for delay in [0, 1, 8, 64] {
        for frame in 0..4 {
         for recoil in [0, 1, 8, 0x80] {
          for submodule in [0, 1] {
           for (x,y) in [(0x40,0x40),(0x40,0x90),(0x90,0x40),(0x1000,0x40)] {
            for seed in 0..16 {
             let mut n=marker29_state();
             for(a,v)in[(0xdd0,9),(0xe20,0x6f),(0xd80,ai),(0xdf0,delay),
                (0xea0,recoil),(0xe40,0x80),(0xd90,seed),(0xda0,seed&1),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
             n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0x11,submodule);
             n.write_wram_byte(0x1a,frame); n.write_wram_byte(0xfa1,seed);
             n.write_wram_word(0x20,y); n.write_wram_word(0x22,x);
             n.sync_native_game_state_from_ram();
             let s=marker29_machine(&n,0x05aa8b,slot).unwrap();
             let random=std::cell::RefCell::new(Vec::new());
             crate::cycle_ledger::assert_rom_marker_work(0x05aa8b,s,|s| {
                if s.cpu.k==13 && s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                let owned=s.cpu.k==5 && ((0xaa8b..0xaae2).contains(&s.cpu.pc)||(0xab04..0xab54).contains(&s.cpu.pc));
                if owned { let taken=match s.cart.read(5,s.cpu.pc,0) {0xd0=>Some(!s.cpu.z),0xb0=>Some(s.cpu.c),_=>None};
                    if let Some(t)=taken {edges.borrow_mut().insert((s.cpu.pc,t));} }
                owned
             },|| {n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_6_f_keese(slot);});
            }
           }
          }
         }
        }
       }
      }
    }
    for pc in [0xaaa6,0xaab0,0xaabc,0xaac5,0xab07,0xab1a,0xab32] {
        for taken in [false,true] {assert!(edges.borrow().contains(&(pc,taken)),"uncovered {pc:x}/{taken}");}
    }
}

#[test]
fn prefix25_bumper_draw_and_trampoline_match_rom() {
    for slot in [0usize,15] {
     for delay in 0..4 {
      for (x,y) in [(0x40u16,0x40u16),(0x40,0xf0),(0x1000,0x40),(0x40,0x1000)] {
       for owner in [0x1eaa8b,0x1e87f0] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,9),(0xe20,0x93),(0xdf0,delay),(0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8); n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1eaa8b,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s|s.cpu.k==0x1e &&
            if owner==0x1eaa8b {(0xaa8b..0xaaa7).contains(&s.cpu.pc)} else {(0x87f0..0x87f5).contains(&s.cpu.pc)},||n.bumper_draw(slot));
       }
      }
     }
    }
}

#[test]
fn prefix25_fake_sword_caller_and_draw_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
     for paused in [0,1] {
      for c in [0,1] {
       for z in [0,16] {
        for owner in [0x05eeaf,0x05eee6,0x05eea7] {
         let mut n=marker29_state();
         for(a,v)in[(0xdd0,9),(0xe20,0xe8),(0xdb0,c),(0xf70,z),(0xd00,0x40),(0xd10,0x40),(0xe40,0x80)] {n.write_wram_byte(a+slot,v);}
         n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0xfc1,paused);
         n.write_wram_byte(0x1fa1c+slot,if z==0 {3}else{0});
         n.sync_native_game_state_from_ram();
         let s=marker29_machine(&n,if owner==0x05eea7 {owner}else{0x05eeaf},slot).unwrap();
         crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
            if s.cpu.k==5 && [0xeebb,0xeec0].contains(&s.cpu.pc) {edges.borrow_mut().insert((s.cpu.pc,!s.cpu.z));}
            s.cpu.k==5 && match owner {0x05eeaf=>(0xeeaf..0xeed6).contains(&s.cpu.pc),0x05eee6=>(0xeee6..0xeef9).contains(&s.cpu.pc),_=>(0xeea7..0xeeaf).contains(&s.cpu.pc)}
         },||if owner==0x05eea7 {n.sprite_e8_fake_sword_long(slot);}else{n.sprite_e8_fake_sword(slot);});
        }
       }
      }
     }
    }
    for pc in [0xeebb,0xeec0] {for taken in [false,true] {assert!(edges.borrow().contains(&(pc,taken)),"uncovered {pc:x}/{taken}");}}
}

#[test]
fn prefix25_firebar_caller_all_paths_match_rom() {
    for slot in [0usize,15] {
     for ty in [0x7e,0x7f] {
      for palace in [0,18] {
       for submodule in [0,1] {
        for angle in [0u16,127,128,255,256,511] {
         let mut n=marker29_state();
         for(a,v)in[(0xdd0,9),(0xe20,ty),(0xd90,angle as u8),(0xda0,(angle>>8)as u8),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
         n.write_wram_byte(0x40c,palace); n.write_wram_byte(0x11,submodule); n.write_wram_byte(0xfa0,slot as u8);
         n.write_wram_word(0x20,0x1000); n.write_wram_word(0x22,0x1000); n.sync_native_game_state_from_ram();
         let s=marker29_machine(&n,0x1ed01a,slot).unwrap();
         crate::cycle_ledger::assert_rom_marker_work(0x1ed01a,s,|s|s.cpu.k==0x1e&&(0xd01a..0xd049).contains(&s.cpu.pc),||n.sprite_7_e_firebar_clockwise(slot));
        }
       }
      }
     }
    }
}
