use super::*;
use super::tests::{marker29_machine,marker29_state};

fn machine(n: &ZeldaState, entry: u32, slot: usize) -> snes::Snes {
    let mut s = marker29_machine(n,entry,slot).unwrap();
    s.ram[0x1fd..0x200].copy_from_slice(&[0xff,0x7f,0]);
    s.cpu.sp=0x1fc;
    s
}

#[test]
fn prefix33_dash_dust_long_draw_children_all_clip_paths_match_rom() {
    for k in [0,5] {
        for timer in [0,1] {
            for frame in 0..3 {
                for facing in [0,2,4,6] {
                    for (x,y) in [(0,0),(255,239),(256,240),(0,255),(0,256),(0xffff,0xffff)] {
                        let mut n=marker29_state();
                        n.ancilla_set_xy(k,x,y);
                        n.write_wram_byte(0x2f,facing);
                        for (a,v) in [(0xc68,timer),(0xc5e,frame),(0xc4a,0x1e)] {n.write_wram_byte(a+k,v);}
                        n.sync_native_game_state_from_ram();
                        crate::cycle_ledger::assert_rom_marker_owners_work(&[0x08f6d9,0x08f6fe,0x08f6e1],machine(&n,0x09adf4,k),
                            |s|s.cpu.k==8 && (0xf6d9..0xf702).contains(&s.cpu.pc),
                            || n.dash_dust_motive(k));
                    }
                }
            }
        }
    }
}

#[test]
fn prefix33_dash_dust_spawn_both_entries_all_allocations_and_facings_match_rom() {
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for occupied in 0..32 {
        for limit in [0,1,4] {
            for facing in [0,2,4,6] {
                for charging in [false,true] {
                    let mut n=marker29_state();
                    for k in 0..5 {n.write_wram_byte(0xc4a+k,if occupied & (1<<k)==0 {0}else{0x1e});}
                    n.write_wram_byte(0x2f,facing);n.sync_native_game_state_from_ram();
                    let owner=if charging {0x098fc1}else{0x098fba};
                    let mut s=machine(&n,owner,0);s.cpu.a=0x1e;s.cpu.y=limit;
                    crate::cycle_ledger::assert_rom_marker_work(owner,s,
                        |s| {
                            if s.cpu.k==9 {
                                let t=match s.cpu.pc {0x8fcb=>Some(s.cpu.c),0x8fe3=>Some(!s.cpu.z),_=>None};
                                if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                            }
                            s.cpu.k==9 && ((0x8fba..0x9011).contains(&s.cpu.pc)||(0x80c3..0x80c8).contains(&s.cpu.pc))
                        }, || if charging {n.ancilla_add_dash_dust_charging(0x1e,limit as u8);}
                            else {n.ancilla_add_dash_dust(0x1e,limit as u8);});
                }
            }
        }
    }
    for pc in [0x8fcb,0x8fe3] {for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");}}
}
