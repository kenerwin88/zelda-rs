use super::*;

fn compare_total(machine: snes::Snes, native: impl FnOnce()) {
    use snes::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, SourcePpuReadState};
    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
    let mut source = RomCpuTimingProbe::new(machine, clock, SourcePpuReadState::snes9x_reset()).unwrap();
    let mut expected = 0;
    for _ in 0..1_000_000 {
        let step = source.step().unwrap();
        expected += step.transactions.iter().map(|t|u64::from(t.duration_master_cycles)).sum::<u64>();
        if source.snes().cpu.pc == 0x8000 { break; }
    }
    assert_eq!(source.snes().cpu.pc, 0x8000);
    let before = crate::cycle_ledger::master(); native();
    assert_eq!(crate::cycle_ledger::master() - before, expected);
}

#[test]
fn prefix27_defer_all_relative_branches_charge_each_helper_once() {
    for ty in [0x1e,0xe8,0x28] { for slot in [0usize,15] { for floor in [0,1] { for (x,y) in [
        (0x40u16,0x40u16),(0,0),(0x40,0x10),(0x40,0x50),(0x10,0x40),
        (0x50,0x40),(0x100,0x40),(0x40,0x100),(0xffff,0xffff)] {
        let mut n = marker29_state();
        for (a,v) in [(0xdd0,9),(0xe20,ty),(0xf20,floor),(0xe40,3),
            (0xd10,0x40),(0xd00,0x40)] { n.write_wram_byte(a+slot,v); }
        n.write_wram_word(0x22,x); n.write_wram_word(0x20,y);
        n.sync_native_game_state_from_ram();
        compare_total(marker29_machine(&n,0x06f86c,slot).unwrap(),||n.oam_allocate_defer_to_player(slot));
    } } } }
}

#[test]
fn prefix27_helmasaur_tail_correction_visits_exactly_sixteen_entries() {
    for slot in [0usize,15] { for start in [0,1,8,15,16] { for animation in [0,1] {
        for (x,y) in [(0x40u16,0x40u16),(0xff,0xf0),(0x100,0x100)] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x92),(0xec0,animation),(0xd10,x as u8),
                (0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xb33,start); n.write_wram_byte(0xb37,64);
            n.write_wram_byte(0x31f,1); n.write_wram_word(0xfd8,x);n.write_wram_word(0xfda,y);
            n.write_wram_word(0,x);n.write_wram_word(2,y);n.write_wram_byte(5,0);
            n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x1e8920,slot).unwrap();
            let mut info=PrepOamCoordsRet{x,y,r4:0,flags:0};
            compare_total(s,||n.king_helmasaur_operate_tail(slot,&mut info));
        }
    } } }
}

#[test]
fn prefix27_eye_statue_all_arrow_trigger_paths_exclude_children() {
    for slot in [0usize,15] { for b in [0,1] { for previous in [0,9] { for paused in [0,1] {
        for (x,y) in [(0x40u16,0x20u16),(0x40,0x80),(0,0x40),(0x80,0x40)] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x38),(0xda0,b),(0xbb0,previous),
                (0xd10,0x40),(0xd00,0x40)] { n.write_wram_byte(a+slot,v); }
            n.write_wram_byte(0x11,paused);n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);
            n.sync_native_game_state_from_ram();
            for owner in [0x05e62b,0x05e633] {
                let mut n = n.clone();
                let s = marker29_machine(&n,0x05e62b,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(owner,s,
                    |s|s.cpu.k==5 && if owner==0x05e62b {(0xe62b..0xe633).contains(&s.cpu.pc)}
                        else {(0xe633..0xe656).contains(&s.cpu.pc)},||n.sprite_38_eye_statue(slot));
            }
        }
    } } } }
}

#[test]
fn prefix27_antifairy_all_damage_and_magic_paths_exclude_children() {
    for slot in [0usize,15] { for paused in [0,1] { for delay in [0,1] { for magic in [0,7,8,255] {
        for link in [0x40u16,0x1000] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x15),(0xdf0,delay),(0xd10,0x40),(0xd00,0x40)] { n.write_wram_byte(a+slot,v); }
            n.write_wram_byte(0x1a,slot as u8); n.write_wram_byte(0x11,paused);n.write_wram_byte(0xf36e,magic);
            n.write_wram_word(0x22,link);n.write_wram_word(0x20,link); n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n,0x06a50c,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x06a50c,s,
                |s|s.cpu.k==6 && (0xa50c..0xa540).contains(&s.cpu.pc),||n.sprite_15_antifairy(slot));
        }
    } } } }
}

#[test]
fn prefix27_spike_all_stationary_transient_and_motion_states_exclude_children() {
    let mut base = marker29_state();
    base.set_assets(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat")).unwrap()).unwrap();
    for slot in [0usize,15] { for transient in [0,1] { for ai in 0..3 { for delay in [0,1,2,64] {
        for aux in [0,1] { for direction in 0..4 { for paused in [0,1] {
            let mut n = base.clone();
            for (a,v) in [(0xdd0,9),(0xe20,0x8a),(0xe90,transient),(0xd80,ai),(0xdf0,delay),
                (0xe00,aux),(0xde0,direction),(0xe40,0x80),(0xd10,0x40),(0xd00,0x40),
                (0xd90,0x40),(0xda0,0x40)] { n.write_wram_byte(a+slot,v); }
            n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);
            n.write_wram_word(0x22,0x1000);n.write_wram_word(0x20,0x1000);n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n,0x1ebce8,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1ebce8,s,
                |s|s.cpu.k==0x1e && (0xbce8..0xbe00).contains(&s.cpu.pc),||n.sprite_8_a_spike_block(slot));
        } } }
    } } } }
}

#[test]
fn prefix27_eyegore_drawing_all_graphics_pause_and_clips_exclude_children() {
    for slot in [0usize,15] { for graphics in 0..12 { for pause in [0,1] {
        for (x,y) in [(0x40u16,0x40u16),(0xff,0xf0),(0x1000,0x1000)] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x84),(0xdc0,graphics),(0xf00,pause),
                (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
            n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n,0x1ecacf,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1ecacf,s,
                |s|s.cpu.k==0x1e && (0xcacf..0xcaf4).contains(&s.cpu.pc),||n.eyegore_draw(slot));
        }
    } } }
}

#[test]
fn prefix27_terrorpin_both_states_all_caller_paths_exclude_children() {
    for slot in [0usize,15] { for b in [0,1] { for timer in [0,1,63,64,255] { for z in [0,1,128,255] {
        for velocity in [0,1,0x7f,0x80,0xff] { for paused in [0,1] {
            let mut n = marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x8e),(0xda0,b),(0xf10,timer),(0xf70,z),(0xf80,velocity),
                (0xd50,velocity),(0xd40,velocity),(0xd10,0x40),(0xd00,0x40),(0xe40,0x80)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfa0,slot as u8);n.write_wram_byte(0x11,paused);n.write_wram_byte(0xfa1,timer);
            n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x1eb26f,slot).unwrap();let random=std::cell::RefCell::new(Vec::new());
            crate::cycle_ledger::assert_rom_marker_work(0x1eb26f,s,|s| {
                let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
                if pc==0x0dba7f {random.borrow_mut().push(crate::RomRandomSample::with_carry(0,s.cpu.a as u8,s.cpu.c));}
                (0x1eb26f..0x1eb292).contains(&pc)||(0x1eb2a7..0x1eb39f).contains(&pc)||(0x008784..0x00879c).contains(&pc)
            },|| {n.install_rom_random_replay(random.borrow().clone(),0);n.rom_random_replay.begin_frame();n.sprite_8_e_terrorpin(slot);});
        } }
    } } } }
}

#[test]
fn prefix27_hinox_draw_all_graphics_and_clips_exclude_children() {
    for slot in [0usize,15] { for graphics in 0..12 {
        for (x,y) in [(0x40u16,0x40u16),(0xff,0xf0),(0x1000,0x1000)] {
            let mut n=marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x11),(0xdc0,graphics),
                (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {n.write_wram_byte(a+slot,v);}
            n.sync_native_game_state_from_ram();let s=marker29_machine(&n,0x06a1f9,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x06a1f9,s,
                |s|s.cpu.k==6 && ((0xa1f9..0xa213).contains(&s.cpu.pc)||(0xdc64..0xdc66).contains(&s.cpu.pc)),||n.hinox_draw(slot));
        }
    } }
}

#[test]
fn prefix27_thrown_item_all_sorting_size_state_and_world_paths_exclude_children() {
    for slot in [0usize,15] { for sorting in [0,1] { for floor in [0,1] { for c in 0..8 {
        for chr in [0,3] { for state in [9,10] { for paused in [0,1] { for dark in [0,1] {
            let mut n=marker29_state();
            for (a,v) in [(0xdd0,state),(0xe20,0xec),(0xdb0,c),(0xf20,floor),
                (0xd10,0x40),(0xd00,0x40),(0xe40,0x80)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0xfb3,sorting);n.write_wram_byte(0xfc6,chr);n.write_wram_byte(0xfff,dark);
            n.write_wram_byte(0x11,paused);n.write_wram_byte(0xfa0,slot as u8);
            n.write_wram_word(0x20,0x1000);n.write_wram_word(0x22,0x1000);n.sync_native_game_state_from_ram();
            let s=marker29_machine(&n,0x06aae0,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x06aae0,s,
                |s|s.cpu.k==6 && (0xaae0..0xab5a).contains(&s.cpu.pc),||n.sprite_ec_thrown_item(slot));
        } } } }
    } } } }
}

#[test]
fn prefix27_heart_piece_prefix_all_initialization_active_and_collision_paths_exclude_children() {
    for slot in [0usize,15] { for ai in [0,1] { for paused in [0,1] { for busy in [0,1] { for tile in [0,1] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,9),(0xe20,0xeb),(0xd80,ai),(0xf10,1),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0x11,paused);n.write_wram_byte(0x4d,busy);n.write_wram_byte(0xfa0,slot as u8);
        for a in 0x12000..0x14000 {n.write_wram_byte(a,tile);}
        n.sync_native_game_state_from_ram();let s=marker29_machine(&n,0x05f020,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x05f020,s,
            |s|s.cpu.k==5 && (0xf020..0xf059).contains(&s.cpu.pc),||n.sprite_heart_piece(slot));
    } } } } }
}

#[test]
fn prefix27_terrorpin_hammer_all_guards_overlap_and_toggle_paths_exclude_children() {
    for slot in [0usize,15] { for z in [0,1] { for aux in [0,1] { for floor in [0,1] {
        for disabled in [0,0x80] { for item in [0,2,8] { for b in [0,1] { for link in [0x40u16,0x1000] {
            let mut n=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x8e),(0xf70,z),(0xe10,aux),(0xf20,floor),(0xda0,b),
                (0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
            n.write_wram_byte(0x44,disabled);n.write_wram_byte(0x301,item);n.write_wram_word(0x22,link);n.write_wram_word(0x20,link);
            n.sync_native_game_state_from_ram();let s=marker29_machine(&n,0x1eb3a3,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x1eb3a3,s,
                |s|s.cpu.k==0x1e && (0xb3a3..0xb405).contains(&s.cpu.pc),||n.terrorpin_check_for_hammer(slot));
        } } } }
    } } } }
}

#[test]
fn prefix27_archer_actual_head_and_body_entries_skip_only_the_rom_prefixes() {
    for slot in [0usize,15] { for graphics in 0..24 { for direction in 0..4 {
        for (x,y) in [(0x40u16,0x40u16),(0xff,0xf0),(0x100,0x100)] { for head in [false,true] {
            let mut n=marker29_state();
            for(a,v)in[(0xe20,0x46),(0xdc0,graphics),(0xde0,direction),(0xeb0,direction)] {n.write_wram_byte(a+slot,v);}
            n.sync_native_game_state_from_ram();let entry=if head {0x05c6e0} else {0x05ca10};
            let mut s=marker29_machine(&n,entry,slot).unwrap();s.cpu.y=0;
            s.ram[0..2].copy_from_slice(&x.to_le_bytes());s.ram[2..4].copy_from_slice(&y.to_le_bytes());s.ram[5]=0;
            let info=PrepOamCoordsRet{x,y,r4:0,flags:0};
            compare_total(s,||if head {n.soldier_draw1_ex(slot,0,&info);}else{n.soldier_draw2_ex(slot,0,&info);});
        } }
    } } }
}

#[test]
fn prefix27_npc_and_shop_draw_callers_all_frames_and_clips_exclude_children() {
    for slot in [0usize,15] { for graphics in 0..3 {
        for (x,y) in [(0x40u16,0x40u16),(0xff,0xf0),(0x100,0x100)] {
            for (entry,end) in [(0x05f91b,0x05f93f),(0x0dc3e1,0x0dc401),
                (0x05e3dc,0x05e3f3),(0x1af88c,0x1af954),(0x1ef4ce,0x1ef4f3)] {
                if graphics==2 && entry!=0x0dc3e1 {continue;}
                let mut n=marker29_state();
                for(a,v)in[(0xdd0,9),(0xe20,0xbb),(0xdc0,graphics),(0xe80,7),
                    (0xd10,x as u8),(0xd30,(x>>8)as u8),(0xd00,y as u8),(0xd20,(y>>8)as u8)] {
                    n.write_wram_byte(a+slot,v);
                }
                n.sync_native_game_state_from_ram();
                let s=marker29_machine(&n,entry,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(entry,s,
                    |s|s.cpu.k==((entry>>16)as u8) && (entry as u16..end as u16).contains(&s.cpu.pc),
                    ||match entry {
                        0x05f91b=>n.shopkeeper_draw(slot),
                        0x0dc3e1=>n.flute_boy_father_draw(slot),
                        0x05e3dc=>n.inn_keeper_draw(slot),
                        0x1af88c=>n.drinking_guy_draw(slot),
                        _=>n.sprite_draw_shop_item(slot),
                    });
            }
        }
    } }
}

#[test]
fn prefix27_npc_bank_six_bounces_and_sweeping_wrapper_exclude_children() {
    for slot in [0usize,15] { for paused in [0,1] { for ty in [0x2a,0x3c,0x74] {
        let mut n=marker29_state();
        for(a,v)in[(0xdd0,9),(0xe20,ty),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+slot,v);}
        n.write_wram_byte(0x11,paused);n.sync_native_game_state_from_ram();
        let entry=match ty{0x2a=>0x06c0bc,0x3c=>0x06c04e,_=>0x06c058};
        let s=marker29_machine(&n,entry,slot).unwrap();
        // The bank-6 bounce is caller work in the existing ExecuteSingle scope.
        let _scope=crate::cycle_ledger::routine(entry);
        crate::cycle_ledger::assert_rom_marker_work(entry,s,
            |s|s.cpu.k==6 && (entry as u16..entry as u16+5).contains(&s.cpu.pc),
            ||n.sprite_active_main_dispatch(slot));
        if ty==0x2a {
            let s=marker29_machine(&n,entry,slot).unwrap();
            crate::cycle_ledger::assert_rom_marker_work(0x0dc4a5,s,
                |s|s.cpu.k==0x0d && (0xc4a5..0xc4ad).contains(&s.cpu.pc),
                ||n.sprite_active_main_dispatch(slot));
        }
    } } }
}

#[test]
fn prefix27_guard_walking_all_timer_collision_direction_and_animation_paths_exclude_children() {
    for slot in [0usize,15] { for ty in [0x46,0x48] { for direction in 0..4 {
        for delay in [0,1,160] { for collision in [0,1,15] {
            for subtype in [0,14,15,255] { for a in [0,1] {
                let mut n=marker29_state();
                for (at,v) in [(0xdd0,9),(0xe20,ty),(0xde0,direction),(0xdf0,delay),
                    (0xe70,collision),(0xe80,subtype),(0xd90,a),(0xf00,1)] {n.write_wram_byte(at+slot,v);}
                n.sync_native_game_state_from_ram();
                let s=marker29_machine(&n,0x05ceaa,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(0x05ceaa,s,
                    |s|s.cpu.k==5 && (0xceaa..0xcee2).contains(&s.cpu.pc),||n.archer_walking(slot));
                let mut n=n.clone();
                let s=marker29_machine(&n,0x05cee2,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(0x05cee2,s,
                    |s|s.cpu.k==5 && (0xcee2..0xcf13).contains(&s.cpu.pc),||n.soldier_throwing_agitated_step(slot));
            } }
        } }
    } } }
}
