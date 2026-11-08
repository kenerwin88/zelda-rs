use crate::zelda_rtl::prefix34_tests::{state, machine};

#[test]
fn prefix34_king_zora_all_graphics_splash_and_double_return_paths_match_rom() {
    let base = state();
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for k in [0,15] { for graphics in 0..13 { for ai in [0,1,2] {
        for splash in [0,1,8,15] { for (x,y) in [(0x40,0x40),(0,0),(0x100,0x40),(0x40,0x180)] {
            let mut n = base.clone();
            for (a,v) in [(0xdd0,9),(0xe20,0x52),(0xdc0,graphics),(0xd80,ai),(0xe10,splash)] {
                n.write_wram_byte(a+k,v);
            }
            n.write_wram_byte(0xd10+k,x as u8); n.write_wram_byte(0xd30+k,(x>>8) as u8);
            n.write_wram_byte(0xd00+k,y as u8); n.write_wram_byte(0xd20+k,(y>>8) as u8);
            n.write_wram_word(0x90,0x800); n.write_wram_word(0x92,0xa20);
            n.sync_native_game_state_from_ram();
            crate::cycle_ledger::assert_rom_marker_work(0x059cab,machine(&n,0x059cab,k),
                |s| {
                    if s.cpu.k == 5 {
                        let t = match s.cpu.pc {0x9cb3=>Some(!s.cpu.c),0x9ce5|0x9cf1|0x9d46=>Some(if s.cpu.pc==0x9ce5 {!s.cpu.z} else {!s.cpu.n}),0x9d02=>Some(s.cpu.z),_=>None};
                        if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                    }
                    s.cpu.k==5 && (0x9cab..0x9d4a).contains(&s.cpu.pc)
                },||n.zora_king_draw(k));
        }}
    }}}
    for pc in [0x9cb3,0x9ce5,0x9cf1,0x9d02,0x9d46] {for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}

#[test]
fn prefix34_waterfall_dispatch_all_inactive_contact_screen_and_child_paths_match_rom() {
    let base=state();
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for k in [0,15] { for subtype in [0,1] { for paused in [0,1,2] {
        for screen in [0,0x43] { for contact in [false,true] {
            let mut n=base.clone();
            for (a,v) in [(0xdd0,9),(0xe20,0x37),(0xe80,subtype),(0xd00,0x40),(0xd10,0x40),(0xe40,0x80)] {
                n.write_wram_byte(a+k,v);
            }
            n.write_wram_byte(0xfa0,k as u8); n.write_wram_byte(0x11,u8::from(paused==1));
            n.write_wram_byte(0xfc1,u8::from(paused==2)); n.write_wram_byte(0x8a,screen);
            n.write_wram_word(0x20,if contact {0x30}else{0xc0});
            n.write_wram_word(0x22,if contact {0x38}else{0xc0});
            n.write_wram_word(0xfd8,0x40); n.write_wram_word(0xfda,0x40);
            n.write_wram_word(0x90,0x800);n.write_wram_word(0x92,0xa20);
            n.sync_native_game_state_from_ram();
            let dispatch=std::cell::Cell::new(false);
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1af5a5,0x1af5ad],machine(&n,0x1af5a5,k),
                |s| {
                    if s.cpu.k==0x1a && s.cpu.pc==0xf5b0 {dispatch.set(true);}
                    let local=dispatch.get()&&s.cpu.k==0&&(0x8784..0x879c).contains(&s.cpu.pc);
                    if local&&s.cpu.pc==0x8799 {dispatch.set(false);}
                    if s.cpu.k==0x1a {
                        let t=match s.cpu.pc {0xf5bf=>Some(!s.cpu.c),0xf5c5=>Some(s.cpu.z),_=>None};
                        if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                    }
                    s.cpu.k==0x1a && ((0xf5a5..0xf5b4).contains(&s.cpu.pc)||(0xf5b8..0xf5d5).contains(&s.cpu.pc))
                        || local
                },||n.sprite_37_waterfall(k));
        }}
    }}}
    for pc in [0xf5bf,0xf5c5] {for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}

#[test]
fn prefix34_bomber_draw_every_graphic_and_clip_path_excludes_draw_and_shadow_children() {
    let base=state();
    for k in [0,15] {for graphics in 0..11 {for (x,y) in [(0,0),(0x40,0x40),(0xff,0xef),(0x100,0x180)] {
        let mut n=base.clone();
        for (a,v) in [(0xdd0,9),(0xe20,0xa9),(0xdc0,graphics),(0xd10,x as u8),(0xd30,(x>>8) as u8),
            (0xd00,y as u8),(0xd20,(y>>8) as u8)] {n.write_wram_byte(a+k,v);}
        n.write_wram_word(0x90,0x800);n.write_wram_word(0x92,0xa20);n.sync_native_game_state_from_ram();
        crate::cycle_ledger::assert_rom_marker_work(0x0dd56c,machine(&n,0x0dd56c,k),
            |s|s.cpu.k==0xd&&(0xd56c..0xd58e).contains(&s.cpu.pc),||n.bomber_draw(k));
    }}}
}

#[test]
fn prefix34_pikit_prep_body_tongue_and_loot_all_paths_exclude_shared_children() {
    let base=state();
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for k in [0,15] {for case in 0..256usize {for path in 0..8 {
        let mut n=base.clone();
        for (a,v) in [(0xdd0,9),(0xe20,0xaa),(0xdc0,(case%4) as u8),
            (0xd80,if path==0 {0}else{2}),(0xf00,u8::from(path==1)),
            (0xde0,(case%8) as u8),(0xd90,case as u8),(0xda0,(255-case) as u8),
            (0xed0,if path<4 {0}else{(path-3) as u8}),(0xe30,(case%3) as u8),
            (0xd10,if path==7 {0}else{0x40}),(0xd30,u8::from(path==7)),(0xd00,0x40)] {
            n.write_wram_byte(a+k,v);
        }
        n.write_wram_word(0x90,if case&1==0 {0x800}else{0x8ff});
        n.write_wram_word(0x92,0xa20);n.sync_native_game_state_from_ram();
        crate::cycle_ledger::assert_rom_marker_owners_work(
            &[0x1e8dca,0x0dd6e6,0x0dd74a,0x0dd813,0x0dd858],machine(&n,0x1e8dca,k),
            |s| {
                if s.cpu.k==0xd && (0xd739..0xd8af).contains(&s.cpu.pc) {
                    let t=match s.cpu.pc {
                        0xd74f|0xd754|0xd860=>Some(!s.cpu.z),0xd79a|0xd7a6|0xd803|0xd8a3=>Some(!s.cpu.n),
                        0xd7c6|0xd7e4=>Some(!s.cpu.c),0xd85b=>Some(s.cpu.z),_=>None};
                    if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                }
                s.cpu.k==0x1e&&(0x8dca..0x8dd2).contains(&s.cpu.pc)
                    ||s.cpu.k==0xd&&((0xd6e6..0xd73a).contains(&s.cpu.pc)
                        ||(0xd74a..0xd80f).contains(&s.cpu.pc)||(0xd813..0xd817).contains(&s.cpu.pc)
                        ||(0xd858..0xd8af).contains(&s.cpu.pc))
            },||n.pikit_draw(k));
    }}}
    // Prep recomputes pause. Enter the unmodified child directly to cover
    // its pause guard independently of the earlier coordinate child.
    let mut n=base.clone(); n.write_wram_byte(0xd80,2); n.write_wram_byte(0xf00,1);
    n.sync_native_game_state_from_ram();
    crate::cycle_ledger::assert_rom_marker_work(0x0dd74a,machine(&n,0x0dd74a,0),
        |s| {
            if s.cpu.k==0xd && s.cpu.pc==0xd754 {edges.borrow_mut().insert((s.cpu.pc,!s.cpu.z));}
            s.cpu.k==0xd&&((0xd74a..0xd80f).contains(&s.cpu.pc)||s.cpu.pc==0xd739)
        },||n.sprite_draw_pikit_tongue(0,&crate::zelda_rtl::sprite::PrepOamCoordsRet {x:0,y:0,r4:0,flags:0}));
    for pc in [0xd74f,0xd754,0xd79a,0xd7a6,0xd7c6,0xd7e4,0xd803,0xd85b,0xd860,0xd8a3] {
        for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");}
    }
}

#[test]
fn prefix34_moblin_pikit_zirro_call_prefixes_all_dispatch_and_early_return_paths_exclude_state_bodies() {
    let base=state();
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for entry in [0x0698e4,0x1e8bbf,0x1e8dd2] {for k in [0,15] {for case in 0..96usize {
        let mut n=base.clone();
        let ty=match entry {0x0698e4=>0x12,0x1e8bbf=>0xaa,_=>0xa9};
        let pellet=entry==0x1e8dd2 && case%8==7;
        for (a,v) in [(0xdd0,if case%8==0 {8}else{9}),(0xe20,ty),
            (0xd80,if pellet {(case/8%2) as u8}else{(case/8%3) as u8}),
            (0xd90,u8::from(pellet)),(0xe80,0),(0xde0,(case%4) as u8),(0xeb0,(case%4) as u8),
            (0xdc0,0),(0xdf0,if pellet {[0,1,8,15,19][case/8%5]}else{[0,1,8,15,16,32,64,95][case%8]}),
            (0xe00,[0,1,8][case/8%3]),(0xd00,0x40),(0xd10,0x40),
            (0xea0,if case%8==2 {8}else{0}),(0xf40,16),(0xf30,16)] {n.write_wram_byte(a+k,v);}
        n.write_wram_byte(0x11,u8::from(case%8==1)); n.write_wram_byte(0xfa0,k as u8);
        n.write_wram_word(0x20,0xc0); n.write_wram_word(0x22,0xc0);
        n.write_wram_word(0x90,0x800);n.write_wram_word(0x92,0xa20);n.sync_native_game_state_from_ram();
        let dispatch=std::cell::Cell::new(false);
        let caller_return=std::cell::Cell::new(false);
        crate::cycle_ledger::assert_rom_marker_work(entry,machine(&n,entry,k),
            |s| {
                let pc=u32::from(s.cpu.k)<<16|u32::from(s.cpu.pc);
                if entry==0x1e8dd2 && [0x1e8dda,0x1e8e34,0x1e8e56].contains(&pc) {
                    edges.borrow_mut().insert((pc,if pc==0x1e8e56 {!s.cpu.z}else{s.cpu.z}));
                }
                // The recoil frame's first PLA has closed its own frame.
                // Its second PLA/RTS are the actual returning caller's work.
                if pc==0x1eff18 {caller_return.set(true);}
                let tail=caller_return.get() && [0x1eff18,0x1eff19].contains(&pc);
                if tail&&pc==0x1eff19 {caller_return.set(false);}
                if [0x0698f9,0x1e8bce,0x1e8ddf].contains(&pc) {dispatch.set(true);}
                let local=dispatch.get()&&s.cpu.k==0&&(0x8784..0x879c).contains(&s.cpu.pc);
                if local&&s.cpu.pc==0x8799 {dispatch.set(false);}
                local || tail || match entry {
                    0x0698e4=>(0x0698e4..0x0698fd).contains(&pc),
                    0x1e8bbf=>(0x1e8bbf..0x1e8bd2).contains(&pc),
                    _=>(0x1e8dd2..0x1e8de3).contains(&pc)||(0x1e8e31..0x1e8e5e).contains(&pc),
                }
            },||match entry {0x0698e4=>n.sprite_12_moblin(k),0x1e8bbf=>n.sprite_aa_pikit(k),_=>n.sprite_a8_green_zirro(k)});
    }}}
    for pc in [0x1e8dda,0x1e8e34,0x1e8e56] {for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}

