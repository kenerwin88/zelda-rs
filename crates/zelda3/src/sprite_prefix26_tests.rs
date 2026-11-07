use super::*;

#[test]
fn prefix26_cukeman_all_caller_branches_match_rom() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {
     for head in [0,1] {
      for state in [8,9] {
       for pause in [0,1,2] {
        for (x,y) in [(0x40u16,0x40u16),(0,0),(0x80,0x40),(0x40,0x80),(0xffff,0xffff)] {
         for joy in [0,0x80] {
          let mut n=marker29_state();
          for (a,v) in [(0xdd0,state),(0xe20,0x0d),(0xeb0,head),(0xe30,1),(0xd00,0x40),(0xd10,0x40)] { n.write_wram_byte(a+slot,v); }
          n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,u8::from(pause==1));n.write_wram_byte(0xfc1,u8::from(pause==2));
          n.write_wram_byte(0xf6,joy);n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);n.sync_native_game_state_from_ram();
          let s=marker29_machine(&n,0x1afa0c,slot).unwrap();
          crate::cycle_ledger::assert_rom_marker_work(0x1afa0c,s,|s| {
            let own=s.cpu.k==0x1a&&(0xfa0c..0xfa7e).contains(&s.cpu.pc);
            if own {let t=match s.cart.read(0x1a,s.cpu.pc,0) {0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0xb0=>Some(s.cpu.c),0x10=>Some(!s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
            own
          },||n.sprite_cukeman(slot));
         }
        }
       }
      }
     }
    }
    for pc in [0xfa0f,0xfa16,0xfa1d,0xfa2e,0xfa3d,0xfa43] { for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");} }
}

#[test]
fn prefix26_buzzblob_actual_recoil_entry_matches_rom() {
    for slot in [0usize,15] { for f in [0,1,8,0x80] { for paused in [0,1] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,9),(0xe20,0x0d),(0xdf0,1),(0xea0,f),(0xe40,0x80),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x06d89a,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x06ff78,s,|s| s.cpu.k==6&&((0xff78..0xff7d).contains(&s.cpu.pc)||f==0&&s.cpu.pc==0xfff3),||n.sprite_0_d_buzzblob(slot));
    } } }
}

#[test]
fn prefix26_raven_project_speed_far_wrapper_matches_rom() {
    for slot in [0usize,15] { for speed in [0u8,1,32,48,255] { for (x,y) in [(0u16,0u16),(0x40,0x40),(0xff,0x40),(0x40,0xff)] {
        let mut n=marker29_state();n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);n.sync_native_game_state_from_ram();
        let mut s=marker29_machine(&n,0x06ea1a,slot).unwrap();s.cpu.a=u16::from(speed);
        crate::cycle_ledger::assert_rom_marker_work(0x06ea1a,s,|s|s.cpu.k==6&&(0xea1a..0xea22).contains(&s.cpu.pc),||{n.sprite_project_speed_towards_link_long_marked(slot,speed);});
    } } }
}

#[test]
fn prefix26_apple_seed_handler_all_branches_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] { for ai in [0,1] { for e in [0,1] { for seed in 0..16 { for full in [false,true] {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0xac),(0xd80,ai),(0xe90,e),(0xd90,16),(0xd00,0x40),(0xd10,0x40)] {n.write_wram_byte(a+slot,v);}
        if full {for i in 0..16 {n.write_wram_byte(0xdd0+i,9);}}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0xfa1,seed);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1ef515,slot).unwrap();let random=std::cell::RefCell::new(Vec::new());
        crate::cycle_ledger::assert_rom_marker_work(0x1ef515,s,|s| {
            if s.cpu.k==13&&s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
            let own=s.cpu.k==0x1e&&(0xf515..0xf535).contains(&s.cpu.pc);
            if own {let t=match s.cart.read(0x1e,s.cpu.pc,0){0xd0=>Some(!s.cpu.z),0x10=>Some(!s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
            own
        },||{n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_ac_apple(slot);});
    } } } } }
    for pc in [0xf518,0xf51d,0xf532] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix26_helmasaur_reinitialize_all_table_indices_match_rom() {
    for slot in [0usize,15] {for t in 0..=255 {
        let mut n=marker29_state();n.write_wram_byte(0xe80+slot,t);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1e8019,slot).unwrap();
        let source=crate::cycle_ledger::assert_rom_marker_work(0x1e8019,s,|s|s.cpu.k==0x1e&&(0x8019..0x8039).contains(&s.cpu.pc),||n.helmasaur_king_reinitialize(slot));
        for i in 0..4 {assert_eq!(n.read_wram_byte(0xb08+i),source.snes().ram[0xb08+i]);}
    } }
}

#[test]
fn prefix26_helmasaur_swing_all_branches_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {for anim in 0..4 {for dir in 0..2 {for head in 0..2 {for frame in 0..2 {for g in [0u8,1,15,16,17,0xef,0xf0,0xf1,255] {for aux in [0,1] {for pos in [0u8,4,120,124,128,132,136,252] {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0x92),(0xec0,anim),(0xde0,dir),(0xeb0,head),(0xee0,aux)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0x1a,frame);n.write_wram_byte(0xb30,g);n.write_wram_byte(0xb31,pos);n.write_wram_byte(0xb2e,head);n.write_wram_byte(0xfa0,slot as u8);n.write_wram_word(0x22,if aux==0 {0x20}else{0x80});n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1e82a0,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x1e82a0,s,|s| {
            let own=s.cpu.k==0x1e&&(0x82a0..0x8383).contains(&s.cpu.pc);
            if own {let t=match s.cart.read(0x1e,s.cpu.pc,0){0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0x10=>Some(!s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
            own
        },||n.helmasaur_king_swing_tail(slot));
    } } } } } } } }
    for pc in [0x82ac,0x82b2,0x82c7,0x82d1,0x82e7,0x82ec,0x82f9,0x82fd,0x8305,0x8318,0x8348,0x8374] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix26_helmasaur_mask_all_caller_branches_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {for c in 0..4 {for delay in [0,1] {for kind in [0,7] {for velocity in [0,1] {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0x92),(0xdb0,c),(0xf10,delay)] {n.write_wram_byte(a+slot,v);}
        for i in 0..2 {n.write_wram_byte(0xc4a+i,kind);n.write_wram_byte(0xc2c+i,velocity);}
        n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1e8686,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x1e8686,s,|s| {
            let own=s.cpu.k==0x1e&&(0x8686..0x86e5).contains(&s.cpu.pc);
            if own {let t=match s.cart.read(0x1e,s.cpu.pc,0){0xb0=>Some(s.cpu.c),0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0x10=>Some(!s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}}
            own
        },||{let mut info=PrepOamCoordsRet{x:0x40,y:0x40,r4:0,flags:0};n.king_helmasaur_mask(slot,&mut info);});
    } } } } }
    for pc in [0x86ac,0x86ca,0x86d4,0x86dc,0x86e2] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix26_helmasaur_root_ai_and_child_returns_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {for ai in 0..4 {for c in [0u8,2,3,0x80] {for delay in [0u8,1,8,64,128] {for (aux1,aux2) in [(0u8,0u8),(1,0),(96,0),(0,1),(0,64)] {for subtype in [0u8,3] {for paused in [0u8,1] {for previous in [0u8,3] {for state in [6u8,9] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,state),(0xe20,0x92),(0xd80,ai),(0xdb0,c),(0xe90,previous),(0xef0,previous),(0xdf0,delay),(0xe00,aux1),(0xe10,aux2),(0xe30,subtype),(0xe50,if c==3{8}else{48}),(0xd00,0x40),(0xd10,0x40),(0xe40,0x80),(0xf70,delay)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);n.sync_native_game_state_from_ram();n.helmasaur_king_initialize(slot);
        n.write_wram_byte(0xb33,[0,15,0,16][usize::from(ai)]);n.write_wram_word(0xb2d,[0x1ff,0,0x180,0x80][usize::from(ai)]);n.write_wram_byte(0xec0+slot,ai&1);n.write_wram_byte(0x1a,delay&1);n.write_wram_byte(0xfa1,previous);n.write_wram_byte(0xfa2,aux1);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1e8039,slot).unwrap();let random=std::cell::RefCell::new(Vec::new());let popped=std::cell::Cell::new(false);
        crate::cycle_ledger::assert_rom_marker_work(0x1e8039,s,|s| {
            let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
            if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
            if pc==0x1e825e {popped.set(true);}
            let own=(0x1e8039..0x1e810a).contains(&pc)||(0x1e8117..0x1e8184).contains(&pc)||(0x1e819c..0x1e81e6).contains(&pc)||(0x1e8210..0x1e8253).contains(&pc)||popped.get()&&(0x1e825e..0x1e827c).contains(&pc)||(0x008784..0x00879c).contains(&pc);
            if own {let t=match s.cart.read(s.cpu.k,s.cpu.pc,0){0x10=>Some(!s.cpu.n),0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0xb0=>Some(s.cpu.c),0x90=>Some(!s.cpu.c),_=>None};if let Some(t)=t {edges.borrow_mut().insert((pc,t));}}
            own
        },|| {n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_92_helmasaur_king(slot);});
    } } } } } } } } }
    for pc in [0x1e803c,0x1e8041,0x1e8044,0x1e8057,0x1e806d,0x1e8084,0x1e80a7,0x1e80ac,0x1e80c5,0x1e80c9,0x1e80d0,0x1e80da,0x1e80f2,0x1e8128,0x1e812d,0x1e813a,0x1e8155,0x1e8159,0x1e815f,0x1e8163,0x1e816d,0x1e8172,0x1e819f,0x1e81a4,0x1e81c6,0x1e81db,0x1e8213,0x1e8218,0x1e8227,0x1e8248,0x1e8268] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix26_helmasaur_movement_and_fireball_child_scopes_match_rom() {
    for slot in [0usize,15] {for frame in 0..4 {for c in [0,3] {for sub in 0..16 {for owner in [0x1e81e6,0x1e8200,0x1e8253] {
        let mut n=marker29_state();n.write_wram_byte(0x1a,frame);n.write_wram_byte(0xdb0+slot,c);n.write_wram_byte(0xe80+slot,sub);n.write_wram_byte(0xe30+slot,sub);n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,if owner==0x1e8253{owner}else{0x1e81e6},slot).unwrap();let random=std::cell::RefCell::new(Vec::new());
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
            let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
            if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
            match owner {0x1e81e6=>(0x1e81e6..0x1e8200).contains(&pc),0x1e8200=>(0x1e8200..0x1e8210).contains(&pc),_=>(0x1e8253..0x1e825e).contains(&pc)||sub!=3&&pc==0x1e827b}
        },||{n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();if owner==0x1e8253 {n.helmasaur_king_maybe_fireball(slot);}else{n.helmasaur_king_handle_movement(slot);}});
    } } } } }
}

#[test]
fn prefix26_sword_prop_and_fountain_all_branches_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {for ai in 0..3 {for a in 0..=255u8 {for delay in [0u8,1] {for owner in [0x0589dc,0x058a16,0x058a94,0x058d29] {
        let mut n=marker29_state();for(addr,v)in[(0xdd0,9),(0xe20,0x62),(0xe80,3),(0xd80,ai),(0xd90,a),(0xe90,a),(0xdf0,delay),(0xd50,8),(0xd40,0xf8),(0xdc0,ai),(0xde0,ai),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(addr+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,owner,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
            let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
            let own=match owner {0x0589dc=>(0x0589dc..0x058a16).contains(&pc),0x058a16=>(0x058a16..0x058a34).contains(&pc),0x058a94=>(0x058a94..0x058ab6).contains(&pc),_=>(0x058d29..0x058d3a).contains(&pc)||(0x058d40..0x058d96).contains(&pc)||(0x008784..0x00879c).contains(&pc)};
            if own {let t=match s.cart.read(s.cpu.k,s.cpu.pc,0){0xd0=>Some(!s.cpu.z),0xf0=>Some(s.cpu.z),_=>None};if let Some(t)=t {edges.borrow_mut().insert((pc,t));}}
            own
        },||match owner {0x0589dc=>n.sprite_master_sword_light_fountain(slot),0x058a16=>n.sprite_master_sword_light_well(slot),0x058a94=>n.sprite_draw_light_fountain(slot),_=>n.sprite_master_sword_prop(slot)});
    } } } } }
    for pc in [0x0589e5,0x058a08,0x058a1f,0x058d46,0x058d6e,0x058d80,0x058d90] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix26_sword_dispatch_excludes_all_five_children() {
    for slot in [0usize,15] {for subtype in 0..5 {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0x62),(0xe80,subtype),(0xdf0,1),(0xda0,2),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x0588c5,slot).unwrap();
        let dispatches=std::cell::Cell::new(0usize);
        crate::cycle_ledger::assert_rom_marker_work(0x0588c5,s,|s| {
            let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
            if pc==0x008784 {dispatches.set(dispatches.get()+1);}
            (0x0588c5..0x0588cc).contains(&pc)||dispatches.get()==1&&(0x008784..0x00879c).contains(&pc)
        },||n.sprite_62_master_sword(slot));
    } }
}

#[test]
fn prefix26_lost_woods_callers_all_branches_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {for ai in 0..2 {for velocity in [0u8,0x10,0xef,0xf1,0x80] {for z in [0u8,16,128,255] {for delay in [0u8,1] {for paused in [0u8,1] {for aux in [0u8,1] {for owner in [0x05940e,0x059468] {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,if owner==0x05940e{0x59}else{0x5a}),(0xd80,ai),(0xdf0,delay),(0xe00,aux),(0xd50,velocity),(0xf80,velocity),(0xf70,z),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,owner,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s| {
            let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
            let own=if owner==0x05940e {(0x05940e..0x059436).contains(&pc)||(0x05943a..0x059468).contains(&pc)||(0x008784..0x00879c).contains(&pc)}else{(0x059468..0x0594af).contains(&pc)};
            if own {let t=match s.cart.read(s.cpu.k,s.cpu.pc,0){0xd0=>Some(!s.cpu.z),0xf0=>Some(s.cpu.z),0x30=>Some(s.cpu.n),0x10=>Some(!s.cpu.n),_=>None};if let Some(t)=t {edges.borrow_mut().insert((pc,t));}}
            own
        },||if owner==0x05940e{n.sprite_59_lost_woods_bird(slot)}else{n.sprite_5_a_lost_woods_squirrel(slot)});
    } } } } } } } }
    for pc in [0x059411,0x05941b,0x059446,0x059456,0x05946b,0x059475,0x059494,0x0594a8] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}");}}
}

#[test]
fn prefix26_hog_and_bolt_actual_movement_and_recoil_entries_match_rom() {
    for slot in [0usize,15] {for ty in [0x44u8,0x45] {for collision in [0,1,4,15] {for recoil in [0,1,0x80] {for paused in [0,1] {for owner in [0x05cdd4,0x05f971] {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,ty),(0xe70,collision),(0xea0,recoil),(0xdf0,1),(0xdc0,0),(0xd50,1),(0xd40,0xff),(0xe40,0x80),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.write_wram_byte(0x1a,(slot as u8)^1);n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,if ty==0x44{0x05cc65}else{0x05cbe0},slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s|s.cpu.k==5&&if owner==0x05cdd4{(0xcdd4..0xcddd).contains(&s.cpu.pc)}else{(0xf971..0xf976).contains(&s.cpu.pc)||recoil==0&&s.cpu.pc==0xf9e8},||if ty==0x44 {n.sprite_44_bluesain_bolt(slot)}else{n.sprite_45_hog_spear_man(slot)});
    } } } } } }
}

#[test]
fn prefix26_crystal_switch_actual_same_layer_entry_matches_rom() {
    for slot in [0usize,15] {for floor in [0u8,1] {for f in [0u8,11] {for paused in [0u8,1] {for link in [0x40u16,0x1000] {for owner in [0x06f154,0x06f15c] {
        let mut n=marker29_state();for(a,v)in[(0xdd0,9),(0xe20,0x1e),(0xf20,floor),(0xea0,f),(0xdf0,1),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.write_wram_word(0x20,link);n.write_wram_word(0x22,link);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x06b8d0,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(owner,s,|s|s.cpu.k==6&&if owner==0x06f154{(0xf154..0xf15c).contains(&s.cpu.pc)||floor!=0&&(0xf1d7..0xf1d9).contains(&s.cpu.pc)}else{floor==0&&(0xf15c..0xf1f6).contains(&s.cpu.pc)},||n.sprite_1_e_crystal_switch(slot));
    } } } } } }
}
