use super::*;

#[test]
fn prefix29_arrghus_states_all_timers_velocity_z_rng_and_garnish_paths_exclude_children() {
    for slot in [0usize,15] {for state in 0usize..6 {for timer in [0,1,16,31,32,48,95,96,97,112,176] {
        for variant in 0..8 {for velocity in [0u8,1,128,255] {
            let mut n=marker29_state();
            if variant&1!=0 {for i in 0..16 {n.write_wram_byte(0xdd0+i,9);n.write_wram_byte(0xe20+i,0x8d);}}
            for(a,v)in[(0xdd0,9),(0xe20,0x8c),(0xd80,state as u8),(0xdf0,timer),(0xd50,velocity),(0xd40,velocity),
                (0xeb0,if variant&2==0{0}else{128}),(0xde0,if variant&2==0{0}else{128}),
                (0xf70,[0,1,95,127,128,223,224,255][variant as usize]),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xb0a,if variant&2==0{0}else{2});n.write_wram_byte(0xb0b,if variant&2==0{0}else{3});
            n.write_wram_byte(0x1a,if variant&2==0{0}else{1});n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0xf359,1);
            for i in 0..30 {n.write_wram_byte(0x1f800+i,if variant&4==0{0}else{1});}
            n.write_wram_word(0x22,0x1000);n.write_wram_word(0x20,0x1000);n.sync_native_game_state_from_ram();
            let owner=[0x1eb593,0x1eb5c8,0x1eb63d,0x1eb4ca,0x1eb4ef,0x1eb532][state];
            let s=marker29_machine(&n,owner,slot).expect("unmodified ROM");
            let random=std::cell::RefCell::new(Vec::new());
            crate::cycle_ledger::assert_rom_marker_owners_work(&[owner,0x1eb5ee],s,
                |s| {
                    let pc=(u32::from(s.cpu.k)<<16)|u32::from(s.cpu.pc);
                    if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                    s.cpu.k==0x1e && match state {0=>(0xb593..0xb5c8).contains(&s.cpu.pc),1=>(0xb5c8..0xb63d).contains(&s.cpu.pc),
                        2=>(0xb63d..0xb674).contains(&s.cpu.pc),3=>(0xb4ca..0xb4ef).contains(&s.cpu.pc),
                        4=>(0xb4ef..0xb532).contains(&s.cpu.pc),_=>(0xb532..0xb593).contains(&s.cpu.pc)}
                },||{n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.arrghus_state(slot);});
        } }
    } } }
}

#[test]
fn prefix29_rupee_pull_all_pause_busy_overlap_and_prize_paths_exclude_children() {
    for slot in [0usize,15] {for state in [8,9] {for busy in [0,1] {for a in [0,1] {
        for pulling in [0,1] {for link in [0x40u16,0x1000] {for flags in [0,1] {
            let mut n=marker29_state();
            for(addr,v)in[(0xdd0,state),(0xe20,0x33),(0xd90,a),(0xf60,flags),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(addr+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x4d,busy);n.write_wram_byte(0x308,pulling);
            n.write_wram_word(0x22,link);n.write_wram_word(0x20,link);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06c017,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x06c017,0x05e24d,0x05e255],s,
                |s|(s.cpu.k==6&&(0xc017..0xc01c).contains(&s.cpu.pc))||(s.cpu.k==5&&(0xe24d..0xe28c).contains(&s.cpu.pc)),
                ||n.sprite_33_rupee_pull(slot));
        } } }
    } } } }
}

#[test]
fn prefix29_bee_dispatch_and_hive_all_states_and_allocation_paths_exclude_children() {
    for slot in [0usize,15] {for ai in 0..3 {for e in [0,1,255] {for free in [false,true] {
        let mut n=marker29_state();
        for i in 0..16 {n.write_wram_byte(0xdd0+i,if free{0}else{9});}
        for(a,v)in[(0xdd0,9),(0xe20,0x79),(0xd80,ai),(0xe90,e),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0xfa0,slot as u8);n.write_wram_word(0x22,0x1000);n.write_wram_word(0x20,0x1000);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x1edc5b,slot).expect("unmodified ROM");
        let random=std::cell::RefCell::new(Vec::new());
        crate::cycle_ledger::assert_rom_marker_owners_work(&[0x1edc5b,0x1edc68,0x008781],s,
            |s| {
                let pc=(u32::from(s.cpu.k)<<16)|u32::from(s.cpu.pc);
                if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                (0x1edc5b..0x1edc62).contains(&pc)||(0x1edc68..0x1edc7b).contains(&pc)||(0x008781..0x00879d).contains(&pc)
            },||{n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_79_bee(slot);});
    } } } }
}

#[test]
fn prefix29_hoarder_covered_all_gates_proximity_lift_and_spawn_paths_exclude_children() {
    for slot in [0usize,15] {for ty in [0x17,0x3e] {for ai in [0,1] {for variant in 0..10 {
        for timer in [0,1] {for link in [0x40u16,0x1000] {for free in [false,true] {
            let mut n=marker29_state();
            if !free {for i in 0..16 {n.write_wram_byte(0xdd0+i,9);}}
            for(a,v)in[(0xdd0,if variant==1{8}else{9}),(0xe20,ty),(0xd80,ai),(0xdf0,timer),(0xe70,u8::from(variant==2)),
                (0xd10,0x40),(0xd00,0x40),(0xcaa,0x80)]{n.write_wram_byte(a+slot,v);}
            if variant==3 {n.write_wram_byte(0x11,1);}
            if variant==4 {n.write_wram_byte(0xfc1,1);}
            n.write_wram_byte(0x372,u8::from(variant==5));n.write_wram_byte(0xf354,u8::from(variant==6));
            n.write_wram_byte(0xf359,1);
            n.write_wram_byte(0x2f4,if variant==7{slot as u8+1}else{0});
            if variant==8 {for a in 0x12000..0x14000 {n.write_wram_byte(a,1);}}
            n.write_wram_byte(0xfa0,slot as u8);n.write_wram_word(0x22,link);n.write_wram_word(0x20,link);
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06a86c,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x06a86c,0x06a874,0x06aa0c,0x06aa14],s,
                |s|s.cpu.k==6&&((0xa86c..0xa91d).contains(&s.cpu.pc)||(0xaa0c..0xaa18).contains(&s.cpu.pc)),
                ||n.sprite_17_hoarder(slot));
        } } }
    } } } }
}

#[test]
fn prefix29_give_damage_all_classes_special_types_and_stun_paths_exclude_children() {
    for slot in [0usize,15] {for ty in [9,0x11,0x18,0x1b,0x53,0x92,0x9a] {
        for dmg in [0,1,0xef,0xf0,0xf9,0xfa,0xfe,0xff] {for variant in 0..8 {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,if variant&1==0{9}else{11}),(0xe20,ty),(0xb6b,variant),
                (0xbe0,if variant&2==0{0}else{0x10}),(0xdb0,if variant&4==0{0}else{3}),
                (0xce2,if variant&2==0{0}else{0xf0}),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xcf2,[0,10,13,14][variant as usize&3]);n.sync_native_game_state_from_ram();
            let mut s=marker29_machine(&n,0x06edc5,slot).expect("unmodified ROM");s.cpu.a=u16::from(dmg);s.ram[0]=0x9d;
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x06edc5,0x06ee9f],s,
                |s|s.cpu.k==6&&((0xedc5..0xedcb).contains(&s.cpu.pc)||(0xede2..0xeec1).contains(&s.cpu.pc)
                    ||(!matches!(dmg,249|250)&&(0xeec1..0xeec8).contains(&s.cpu.pc))),
                ||n.sprite_give_damage(slot,dmg,0x9d));
        } }
    } }
}

#[test]
fn prefix29_throw_all_return_gate_button_and_facing_paths_exclude_sound() {
    for slot in [0usize,15] {for module in [7,14] {for pit in [0,2] {for gate in 0..6 {
        for held in [0,3] {for button in [0,0x80] {for facing in [0,2,4,6] {
            let mut n=marker29_state();n.write_wram_byte(0x10,module);n.write_wram_byte(0x5b,pit);
            n.write_wram_byte(0xe20+slot,0xec);n.write_wram_byte(0x1fa1c+slot,held);n.write_wram_byte(0x1fa2c+slot,6);
            for(a,v)in[(0x4d,u8::from(gate==1)),(0x345,u8::from(gate==2)),(0x2e0,u8::from(gate==3)),
                (0x2da,u8::from(gate==4)),(0x46,u8::from(gate==5)),(0x37b,0),(0xf6,button),(0x2f,facing)]{n.write_wram_byte(a,v);}
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06df6d,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_work(0x06df6d,s,
                |s|s.cpu.k==6&&((0xdf6d..0xdff2).contains(&s.cpu.pc)||s.cpu.pc==0xdf5d),||n.carried_sprite_check_for_throw(slot));
        } } } }
    } } }
}

#[test]
fn prefix29_thrown_collision_all_e_flags_and_single_hit_paths_exclude_children() {
    for slot in [0usize,15] {for other in [1usize,14] {for ty in [0xec,0x11] {for target in [0x3f,0x11] {
        for indoors in [0,1] {for c in [0,2] {for x in [0x40u16,0x1000] {for z in [0,1,255] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,ty),(0xdb0,c),(0xf70,z),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
            for(a,v)in[(0xdd0,9),(0xe20,target),(0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,0x40)]{n.write_wram_byte(a+other,v);}
            n.write_wram_byte(0x1b,indoors);n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
            let mut s=marker29_machine(&n,0x06e1b2,slot).expect("unmodified ROM");s.cpu.y=other as u16;
            crate::cycle_ledger::assert_rom_marker_work(0x06e1b2,s,
                |s|s.cpu.k==6&&((0xe1b2..0xe229).contains(&s.cpu.pc)||s.cpu.pc==0xe1b1),
                ||n.thrown_sprite_check_damage_to_single_sprite(slot,other as i32));
        } } } }
    } } } }
    for slot in [0usize,15] {for e in [0,1,255] {
        let mut n=marker29_state();n.write_wram_byte(0xe90+slot,e);n.write_wram_byte(0xe20+slot,0xec);
        n.write_wram_byte(0xdd0+slot,9);n.write_wram_byte(0xf70+slot,16);n.write_wram_byte(0xfa0,slot as u8);n.sync_native_game_state_from_ram();
        let s=marker29_machine(&n,0x06e164,slot).expect("unmodified ROM");
        assert_eq!(n.sprite_slot_view(slot).e(),s.ram[0xe90+slot],"slot {slot} e {e}");
        crate::cycle_ledger::assert_rom_marker_work(0x06e164,s,
            |s|s.cpu.k==6&&(0xe164..0xe172).contains(&s.cpu.pc),||n.throwable_scenery_interact_with_sprites_and_tiles(slot));
    } }
}

#[test]
fn prefix29_damage_dispatch_all_types_faces_deflections_and_timer_paths_exclude_children() {
    for slot in [0usize,15] {for ty in 0u8..243 {for deflect in [0,4] {
        for timer in [0,1] {for variant in 0..4 {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,ty),(0xcaa,deflect),(0xd90,variant&1),(0xdb0,if variant&1==0{0}else{3}),
                (0xde0,variant),(0xe60,0x40),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x47,timer);n.write_wram_byte(0x3c,if variant&1==0{0}else{9});
            n.write_wram_byte(0x2f,if variant&1==0{[4,6,0,2][variant as usize]}else{4});n.write_wram_byte(0xfac,1);
            n.write_wram_word(0x22,0x40);n.write_wram_word(0x20,0x40);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06f314,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_work(0x06f314,s,
                |s|s.cpu.k==6&&((0xf314..0xf3ca).contains(&s.cpu.pc)||(0xf460..0xf46d).contains(&s.cpu.pc))
                    && !(ty==9 && variant&1==0 && s.cpu.sp==0x1f8 && (0xf3c7..0xf3ca).contains(&s.cpu.pc)),
                ||n.sprite_damage_type_dispatch_markers(slot));
        } } }
    } }
}

#[test]
fn prefix29_weapon_tink_busy_and_coordinate_carry_paths_exclude_pan() {
    for timer in [0,1,255] {for x in [0u16,127,255,256,0xffff] {for y in [0u16,255] {
        for offset in [0,1,127,255] {
            let mut n=marker29_state();n.write_wram_byte(0xfac,timer);
            n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);n.write_wram_byte(0x45,offset);n.write_wram_byte(0x44,offset);
            n.sync_native_game_state_from_ram();
            let mut s=marker29_machine(&n,0x06f69f,0).expect("unmodified ROM");s.cpu.c=false;
            crate::cycle_ledger::assert_rom_marker_work(0x06f69f,s,
                |s|s.cpu.k==6&&(0xf69f..0xf6ca).contains(&s.cpu.pc),||n.link_place_weapon_tink());
        }
    } } }
}

#[test]
fn prefix29_sword_all_flags_running_frames_items_and_damage_lookup_paths_exclude_give_damage() {
    for slot in [0usize,15] {for ty in [0x11,0x83,0xd7,0xd8,0xeb] {for flags in [0,0x40] {
        for running in [0,1] {for frames in [0,8,9,127,128,255] {for sword in 1..=4 {for hand in [0,2,8,10] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,ty),(0xe60,flags)]{n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x372,running);n.write_wram_byte(0x3c,frames);n.write_wram_byte(0xf359,sword);n.write_wram_byte(0x301,hand);
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06ed3f,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x06ed3f,0x06ed89],s,
                |s|s.cpu.k==6&&(0xed3f..0xedc5).contains(&s.cpu.pc),||n.sprite_calculate_sword_damage(slot));
        } } } }
    } } }
}

#[test]
fn prefix29_zap_all_types_electric_state_blink_and_sword_paths_exclude_children() {
    for slot in [0usize,15] {for ty in [0x7a,0x0d,0x24,0x23,0x11] {for state in [8,9] {
        for timer in [0,1] {for blink in [0,1] {for frames in [0,8,9,127,128,255] {for sword in [1,3,4] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,state),(0xe20,ty),(0xdf0,timer),(0xe60,0x40),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x31f,blink);n.write_wram_byte(0x3c,frames);n.write_wram_byte(0xf359,sword);
            n.write_wram_word(0x22,0x40);n.write_wram_word(0x20,0x40);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06ec02,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_work(0x06ec02,s,
                |s|s.cpu.k==6&&(0xec02..0xec5c).contains(&s.cpu.pc),||n.sprite_attempt_zap_damage(slot));
        } } } }
    } } }
}

#[test]
fn prefix29_location_predicates_all_signed_borrow_and_wrap_paths() {
    for slot in [0usize,15] { for origin in [0u16,1,127,128,255,256,0x7fff,0x8000,0xffff] {
        for target in [0u16,1,127,128,255,256,0x7fff,0x8000,0xffff] { for vertical in [false,true] {
            let mut n=marker29_state();
            n.write_wram_byte(0xd10+slot,origin as u8);n.write_wram_byte(0xd30+slot,(origin>>8)as u8);
            n.write_wram_byte(0xd00+slot,origin as u8);n.write_wram_byte(0xd20+slot,(origin>>8)as u8);
            n.write_wram_word(if vertical {6}else{4},target);n.sync_native_game_state_from_ram();
            let owner=if vertical {0x06eb1d}else{0x06eb0a};
            let s=marker29_machine(&n,owner,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_work(owner,s,
                |s|s.cpu.k==6 && if vertical {(0xeb1d..0xeb30).contains(&s.cpu.pc)}else{(0xeb0a..0xeb1d).contains(&s.cpu.pc)},
                ||{if vertical {n.sprite_is_below_location(slot,target);}else{n.sprite_is_right_of_location(slot,target);}});
        } }
    } }
}

#[test]
fn prefix29_heart_piece_callers_all_early_bounce_and_pickup_paths_exclude_children() {
    for slot in [0usize,15] {for ai in [0,1] {for timer in [0,1] {for z in [0,1,128,255] {
        for link in [0x40u16,0x1000] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0xeb),(0xd80,ai),(0xdf0,timer),(0xf70,z),(0xd10,0x40),(0xd00,0x40)]{n.write_wram_byte(a+slot,v);}
            n.write_wram_word(0x22,link);n.write_wram_word(0x20,link);n.write_wram_byte(0xfa0,slot as u8);
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06c0a3,slot).expect("unmodified ROM");
            crate::cycle_ledger::assert_rom_marker_owners_work(&[0x06c0a3,0x05f018],s,
                |s|(s.cpu.k==6&&(0xc0a3..0xc0a8).contains(&s.cpu.pc))||(s.cpu.k==5&&(0xf018..0xf020).contains(&s.cpu.pc)),
                ||n.sprite_heart_piece_bounce(slot));
        }
    } } } }
}

#[test]
fn prefix29_sound_pan_all_regions_boundaries_and_busy_channels_exclude_children() {
    for slot in [0usize,15] {for scroll in [0u16,0x100,0xffff] {
        for offset in [0u16,0x4f,0x50,0x51,0x9f,0xa0,0x7fff,0x8000,0xffff] {
            for channel in [2,3] {for busy in [0,1,255] {
                let mut n=marker29_state();let x=scroll.wrapping_add(offset);
                n.write_wram_word(0xe2,scroll);n.write_wram_byte(0xd10+slot,x as u8);n.write_wram_byte(0xd30+slot,(x>>8)as u8);
                n.write_wram_byte(if channel==2{0x12e}else{0x12f},busy);n.sync_native_game_state_from_ram();
                let owner=if channel==2{0x0dbb7c}else{0x0dbb8a};
                let mut s=marker29_machine(&n,owner,slot).expect("unmodified ROM");s.cpu.a=0x21;
                crate::cycle_ledger::assert_rom_marker_owners_work(&[owner,0x0dbb98,0x0dbba1],s,
                    |s|s.cpu.k==0x0d && ((if channel==2{0xbb7c..0xbb8a}else{0xbb8a..0xbb98}).contains(&s.cpu.pc)
                        ||(0xbb98..0xbbc8).contains(&s.cpu.pc)),
                    ||if channel==2{n.sprite_sfx_queue_sfx2_with_pan(slot,0x21)}else{n.sprite_sfx_queue_sfx3_with_pan(slot,0x21)});
            } }
        }
    } }
}
