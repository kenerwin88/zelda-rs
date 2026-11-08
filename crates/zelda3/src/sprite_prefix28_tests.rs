use super::*;

#[test]
fn prefix28_big_shadow_entry_all_z_and_clip_paths_excludes_draw_and_coordinates() {
    for slot in [0usize,15] {for z in 0..=255 {for x in [0x40u16,0x1000] {
        let mut n=marker29_state();n.write_wram_byte(0xdd0+slot,9);n.write_wram_byte(0xf70+slot,z);
        n.write_wram_word(0xfd8,x);n.write_wram_word(0xfda,x);
        n.write_wram_byte(0xd10+slot,x as u8);n.write_wram_byte(0xd30+slot,(x>>8)as u8);
        n.sync_native_game_state_from_ram();
        let source=marker29_machine(&n,0x1dd1a8,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1dd1a8,0x1d8549,0x0684bd],source,
            |s| (s.cpu.k==0x1d && ((0xd1a8..0xd1af).contains(&s.cpu.pc)||(0xd1bf..0xd1fd).contains(&s.cpu.pc)
                ||(0x8549..0x854e).contains(&s.cpu.pc)))||(s.cpu.k==6 && (0x84bd..0x84c1).contains(&s.cpu.pc)),
            ||n.sprite_draw_big_shadow(slot,0));
    } } }
}

#[test]
fn prefix28_damage_post_overlap_all_action_and_transmutation_paths_exclude_dispatch() {
    let visited=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize,15] {for ty in [0x11,0x83,0x8e,0xd5,0xd6] {for state in [9,11] {
        for item in [0,2,8,10] {for mode in [0,16] {for frames in [0,1,8,9,11] {for draw in [0,1] {
            let mut n=marker29_state();
            for(a,v) in [(0xdd0,state),(0xe20,ty),(0xd10,0x40),(0xd00,0x40),(0xf60,0),
                (0x1fa3c,draw)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x301,item);n.write_wram_byte(0x37a,mode);
            n.write_wram_byte(0x3c,frames);n.write_wram_byte(0xf359,1);n.write_wram_word(0x22,0x40);n.write_wram_word(0x20,0x40);
            n.sync_native_game_state_from_ram();
            let source=marker29_machine(&n,0x06f2b4,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x06f2d3,source,
                |s| {
                    if s.cpu.k==6 {visited.borrow_mut().insert(s.cpu.pc);}
                    s.cpu.k==6 && ((0xf2d3..0xf314).contains(&s.cpu.pc)||s.cpu.pc==0xf2a9)
                },||{n.sprite_check_damage_from_link(slot,crate::zelda_rtl::sprite::SpriteDamageFromEntry::Unpriced);});
        } } } }
    } } }
    for pc in [0xf2a9,0xf2d3,0xf2dd,0xf2e4,0xf2eb,0xf2f2,0xf2f8,0xf312] {
        assert!(visited.borrow().contains(&pc),"uncovered action block {pc:x}");
    }
}

#[test]
fn prefix28_arrghus_prefix_all_states_hit_recoil_and_collision_paths_exclude_children() {
    for slot in [0usize,15] {for ai in 0..6 {for variant in 0..8 {for z in [0,96,160,224] {
        let mut n=marker29_state();
        for(a,v) in [(0xdd0,if variant==1 {8}else{9}),(0xe20,0x8c),(0xd80,ai),(0xf70,z),
            (0xd10,0x40),(0xd00,0x40),(0xe40,0x80),(0xed0,8),(0xd90,0),(0xda0,0),
            (0xea0,if variant==2 {1}else if variant==3 {128}else{0}),
            (0xef0,if variant==4 {2}else{0}),(0xe80,if variant==5 {3}else{0}),(0xdf0,1)] {n.write_wram_byte(a+slot,v);}
        if variant==6 {n.write_wram_byte(0x11,1);}
        if variant==7 {for a in 0x12000..0x14000 {n.write_wram_byte(a,1);}}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_word(0x22,0x1000);n.write_wram_word(0x20,0x1000);
        n.sync_native_game_state_from_ram();
        let source=marker29_machine(&n,0x1eb433,slot).unwrap();let random=std::cell::RefCell::new(Vec::new());
        crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1eb433,0x008781],source,
            |s| {
                let pc=(u32::from(s.cpu.k)<<16)|u32::from(s.cpu.pc);
                if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                (0x1eb433..0x1eb4be).contains(&pc)||(0x008781..0x00879c).contains(&pc)
                    || pc==0x1eff18 || (pc==0x1eff19 && s.cpu.sp==0x1fa)
            },||{n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_8_c_arrghus(slot);});
    } } } }
}

#[test]
fn prefix28_arrghus_hit_transition_all_slots_excludes_sound() {
    for slot in [0usize,15] {
        let mut n=marker29_state();n.sync_native_game_state_from_ram();
        let source=marker29_machine(&n,0x1eb5ee,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x1eb5ee,source,
            |s|s.cpu.k==0x1e && (0xb5ee..0xb5fd).contains(&s.cpu.pc),||n.arrghus_set_state3(slot));
    }
}

#[test]
fn prefix28_fourth_heart_piece_retained_return_prices_only_its_suffix() {
    for slot in [0usize,15] { for indoors in [0,1] { for high in [0,1] {
        let mut n=marker29_state();n.write_wram_byte(0x1b,indoors);n.write_wram_byte(0xd30+slot,high);
        n.sync_native_game_state_from_ram();let mut source=marker29_machine(&n,0x05f0a5,slot).unwrap();
        source.cpu.sp=0x1f9;source.ram[0x1fa]=slot as u8;
        crate::cycle_ledger::assert_rom_marker_work(0x05f020,source,
            |s|s.cpu.k==5 && (0xf0a5..0xf0cd).contains(&s.cpu.pc),||n.complete_heart_piece_item_receipt(slot));
    } } }
}

#[test]
fn prefix28_guard_idle_alert_approach_and_throw_all_paths_exclude_children() {
    for slot in [0usize,15] { for ty in [0x46,0x48] { for ai in [0,1,2,3,4,5] {
        for direction in 0..4 { for delay in [0,1,12,24,39,40,60,160] { for variant in 0..5 {
            let mut n=marker29_state();
            for(a,v) in [(0xdd0,9),(0xe20,ty),(0xd80,ai),(0xde0,direction),(0xeb0,direction),
                (0xdf0,delay),(0xd10,0x40),(0xd00,0x40),(0xe40,0x80),(0xd90,1)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x1a,slot as u8 + if variant==1 {1}else{0});
            if variant==2 {for a in 0x12000..0x14000 {n.write_wram_byte(a,1);}}
            let (x,y)=if variant==3 {(0x40,0x40)}else{(0x1000,0x1000)};
            if variant==4 {n.write_wram_byte(0x11,1);}
            n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);n.sync_native_game_state_from_ram();
            let source=marker29_machine(&n,if ty==0x48 {0x05cde1}else{0x05cdff},slot).unwrap();
            let random=std::cell::RefCell::new(Vec::new());
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x05ce74,0x05cf44,0x05cf85,0x05d045],source,
                |s| {
                    if s.cpu.k==13 && s.cpu.pc==0xba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                    s.cpu.k==5 && ((0xce74..0xceaa).contains(&s.cpu.pc)||(0xcf44..0xcf61).contains(&s.cpu.pc)
                        ||(0xcf85..0xd001).contains(&s.cpu.pc)||(0xd045..0xd08b).contains(&s.cpu.pc)
                        ||(0xc417..0xc425).contains(&s.cpu.pc))
                },|| {
                    n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();
                    if ty==0x48 {n.sprite_48_red_javelin_guard(slot);}else{n.sprite_46_blue_archer(slot);}
                });
        } } }
    } } }
}

#[test]
fn prefix28_arrgi_all_tether_home_damage_and_return_paths_exclude_children() {
    for slot in [0usize,15] { for ai in [0,1] { for tether in [0,1,5] { for attached in [0,1] {
        for delay in [0,1] { for frame in [0,1,3,255] { for home in [0x40u16,0x80,0x1000] { for paused in [0,1] {
            let mut n=marker29_state();
            for(a,v) in [(0xdd0,9),(0xe20,0x8d),(0xd80,ai),(0xda0,tether),(0xdf0,delay),
                (0xd10,0x40),(0xd00,0x40),(0xe40,0x80)] {n.write_wram_byte(a+slot,v);}
            for i in 0..5 {n.write_wram_byte(0xc4a+i,attached);}
            for a in [0xb0f,0xb2f] {n.write_wram_byte(a+slot,home as u8);n.write_wram_byte(a+slot+16,(home>>8)as u8);}
            n.write_wram_byte(0x11,paused);n.write_wram_byte(0x1a,frame);n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x22,0x1000);n.write_wram_word(0x20,0x1000);n.sync_native_game_state_from_ram();
            let source=marker29_machine(&n,0x1eb8c4,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1eb8c4,source,
                |s|s.cpu.k==0x1e && (0xb8c4..0xb9a9).contains(&s.cpu.pc),||n.sprite_8_d_arrghi(slot));
        } } } }
    } } } }
}

#[test]
fn prefix28_arrghus_draw_all_states_waves_clips_and_page_crossings_exclude_children() {
    for slot in [0usize,15] { for ai in 0..6 { for subtype in [0,8] { for z in [0,159,160,255] {
        for oam in [0x800u16,0x8f0,0x8ff] { for x in [0x40u16,0x1000] {
            let mut n=marker29_state();
            for(a,v) in [(0xdd0,9),(0xe20,0x8c),(0xd80,ai),(0xe80,subtype),(0xf70,z),
                (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_word(0x90,oam);n.sync_native_game_state_from_ram();
            let source=marker29_machine(&n,0x1eb840,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1eb840,0x1e87f0],source,
                |s|s.cpu.k==0x1e && ((0xb840..0xb8b4).contains(&s.cpu.pc)||(0x87f0..0x87f5).contains(&s.cpu.pc)),
                ||n.arrghus_draw(slot));
        } }
    } } } }
}

#[test]
fn prefix28_heart_piece_all_bounce_delay_and_pickup_paths_exclude_children() {
    for slot in [0usize, 15] { for z in [0, 1, 127, 128, 255] {
        for velocity in [0u8, 1, 2, 127, 128, 254, 255] { for timer in [0, 1] {
            for link in [0x40u16, 0x1000] { for pieces in 0..3 {
                let mut n = marker29_state();
                for (a, v) in [(0xdd0,9),(0xe20,0xeb),(0xd80,1),(0xf70,z),(0xf80,0xf0),
                    (0xd50,velocity),(0xf10,timer),(0xd10,0x40),(0xd00,0x40)] { n.write_wram_byte(a+slot,v); }
                n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0xf36b,pieces);
                n.write_wram_word(0x22,link); n.write_wram_word(0x20,link);
                n.sync_native_game_state_from_ram();
                let source = marker29_machine(&n,0x05f020,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(0x05f020,source,
                    |s|s.cpu.k==5 && (0xf020..0xf0cd).contains(&s.cpu.pc),||n.sprite_heart_piece(slot));
            } }
        } }
    } }
}

#[test]
fn prefix28_hinox_all_states_timers_random_faces_and_spawn_paths_exclude_children() {
    for slot in [0usize,15] { for ai in 0..3 { for delay in [0,1,16,31,32,33,64] {
        for direction in 0..4 { for variant in 0..7 {
            let mut n=marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x11),(0xd80,ai),(0xdf0,delay),(0xde0,direction),
                (0xd10,0x40),(0xd00,0x40),(0xe40,0x80),(0xdb0,if variant==1 {3}else{0}),
                (0xd90,if variant==1 {1}else{0}),(0xea0,if variant==2 {1}else if variant==3 {128}else{0})] {
                n.write_wram_byte(a+slot,v);
            }
            if variant==4 {n.write_wram_byte(0x11,1);}
            if variant==5 {for a in 0x12000..0x14000 {n.write_wram_byte(a,1);}}
            if variant==6 {for i in 0..16 {n.write_wram_byte(0xdd0+i,9);}}
            n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x22,0x1000);n.write_wram_word(0x20,0x1000);
            n.sync_native_game_state_from_ram();
            let source=marker29_machine(&n,0x069f05,slot).unwrap();
            let random=std::cell::RefCell::new(Vec::new());
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x069f05,0x069fe1,0x06a004,0x008781],source,
                |s| {
                    let pc=(u32::from(s.cpu.k)<<16)|u32::from(s.cpu.pc);
                    if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                    (0x069f05..0x069f2a).contains(&pc)||(0x069f4a..0x069fef).contains(&pc)
                        ||(0x069ff7..0x06a025).contains(&pc)||(0x06a029..0x06a065).contains(&pc)
                        ||(0x008781..0x00879d).contains(&pc)
                },|| {
                    n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();
                    n.sprite_11_hinox(slot);
                });
        } }
    } } }
}

#[test]
fn prefix28_bank1e_recoil_all_returns_masks_and_collision_paths_exclude_children() {
    for slot in [0usize,15] { for ty in [0x7a,0x83,0x84,0x8e] {
        for f in [0,1,2,4,8,12,16,20,24,128,129,255] { for frame in [0,1,3,7,15,255] {
            for (x,y) in [(0,0),(31,32),(32,31),(224,223),(255,128)] { for tile in [0,1] {
                let mut n=marker29_state();
                for(a,v) in [(0xdd0,9),(0xe20,ty),(0xea0,f),(0xf40,x),(0xf30,y),
                    (0xd50,17),(0xd40,23),(0xd10,0x40),(0xd00,0x40),(0xcd2,if tile==0 {0x80}else{0})] {
                    n.write_wram_byte(a+slot,v);
                }
                for a in 0x12000..0x14000 {n.write_wram_byte(a,tile);}
                n.write_wram_byte(0x1a,frame); n.write_wram_byte(0xfa0,slot as u8);
                n.sync_native_game_state_from_ram();
                let source=marker29_machine(&n,0x1efe9b,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1efe9b,0],source,
                    |s|s.cpu.k==0x1e && (0xfe9b..0xff1e).contains(&s.cpu.pc),|| {
                        n.sprite_return_if_recoiling(slot,SpriteRecoilEntry::Rom1EFE9B);
                    });
            } }
        } }
    } }
}

#[test]
fn prefix28_thrown_scatter_all_sizes_slots_allocations_and_clip_paths_exclude_children() {
    for slot in [0usize,15] { for c in [0,1,5,6,7,127,128,255] {
        for free in [0,1,3,15] { for garnish in [0,1,15,29,30] { for x in [0x40u16,0x1000] {
            let mut n=marker29_state();
            for i in 0..16 {n.write_wram_byte(0xdd0+i,if i<free {0}else{9});}
            for i in 0..30 {n.write_wram_byte(0x1f800+i,if i<garnish {0}else{1});}
            for(a,v) in [(0xdd0,6),(0xe20,0xec),(0xdb0,c),(0xd10,x as u8),(0xd30,(x>>8)as u8),
                (0xd00,0x40),(0xe40,0x80)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
            let source=marker29_machine(&n,0x06ac41,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x06ac41,source,
                |s|s.cpu.k==6 && (0xac41..0xad03).contains(&s.cpu.pc),||n.throwable_scenery_scatter_into_debris(slot));
        } } }
    } }
}

#[test]
fn prefix28_breakage_all_flags_and_slots_excludes_callers() {
    for slot in [0usize,15] { for flags in 0..=255 {
        let mut n=marker29_state();n.write_wram_byte(0xe40+slot,flags);n.sync_native_game_state_from_ram();
        let source=marker29_machine(&n,0x06e25a,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x06e25a,source,
            |s|s.cpu.k==6 && (0xe25a..0xe26e).contains(&s.cpu.pc),||n.sprite_schedule_for_breakage(slot));
    } }
}

#[test]
fn prefix28_arrghus_puffs_all_angles_quadrants_waves_and_wraps_exclude_children() {
    for slot in [0usize,15] { for angle in 0u16..512 { for amplitude in [0,64,255] { for frame in [0,3,7,8] {
        let mut n=marker29_state();
        for(a,v) in [(0xdd0,9),(0xe20,0x8c),(0xd10,0xff),(0xd30,0x7f),(0xd00,1),
            (0xd90,12),(0xda0,0)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_word(0xb08,angle);n.write_wram_byte(0xb0a,amplitude);n.write_wram_byte(0xb0c,1);
        n.write_wram_byte(0x1a,frame);n.sync_native_game_state_from_ram();
        let source=marker29_machine(&n,0x1eb6e9,slot).unwrap();
        let source=crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1eb6e9,0x1e8a85],source,
            |s|s.cpu.k==0x1e && ((0xb6e9..0xb818).contains(&s.cpu.pc)||(0x8a85..0x8a8e).contains(&s.cpu.pc)),
            ||n.arrghus_handle_puffs(slot));
        for a in 0xb08..0xb4d {assert_eq!(n.read_wram_byte(a),source.snes().ram[a],"angle={angle} address={a:x}");}
    } } } }
}
