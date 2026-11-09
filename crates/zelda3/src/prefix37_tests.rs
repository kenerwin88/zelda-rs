use super::prefix34_tests::{machine, state};
use std::cell::RefCell;
use std::collections::BTreeSet;

fn edge(s: &snes::Snes, edges: &RefCell<BTreeSet<(u16, bool)>>) {
    let take = match s.cart.read(s.cpu.k, s.cpu.pc, 0) {
        0xf0 => Some(s.cpu.z), 0xd0 => Some(!s.cpu.z), _ => None,
    };
    if let Some(take) = take { edges.borrow_mut().insert((s.cpu.pc, take)); }
}

#[test]
fn prefix37_item_halt_all_collision_platform_guards_match_rom() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for collision in [0,1,2,3,255] { for layer in 0..8 { for platform in [0,1,255] {
        let mut n = base.clone();
        for (a,v) in [(0xad,collision),(0x322,layer),(0x2f5,platform),
            (0x30,0xa5),(0x31,0x5a),(0x67,0x81),(0x2a,0xff),(0x2b,1),(0x6b,0x80)] {
            n.write_wram_byte(a,v);
        }
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x07ae65,
            machine(&n,0x07ae65,0), |s| {
                let own = s.cpu.k == 7 && (0xae65..0xae88).contains(&s.cpu.pc);
                if own { edge(s,&edges); } own
            }, || n.halt_link_when_using_items());
        for a in [0x30,0x31,0x67,0x2a,0x2b,0x6b] {
            assert_eq!(n.read_wram_byte(a),source.snes().ram[a],
                "collision {collision}, layer {layer}, platform {platform}, address {a:x}");
        }
    }}}
    for pc in [0xae69,0xae72,0xae83] { for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}

#[test]
fn prefix37_hookshot_tile_wrapper_all_layers_directions_slots_and_wraps_exclude_children() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for k in [0,5,9] { for work in [0,1] { for stairs in [0,1] { for collision in [0,2] {
        for dir in 0..4 { for xy in [0u16,7,255,65535] { for lower in [0,1] {
            let mut n = base.clone();
            for (a,v) in [(0x1b,1),(0x3a4+k,work),(0x44a,stairs),(0x46c,collision),
                (0xc72+k,dir),(0xee,lower)] { n.write_wram_byte(a,v); }
            for (a,v) in [(0xec,0x1ff),(0xa0,0x12f8),(0xe0,0x180),(0xe2,0x100),
                (0xe6,0x100),(0xe8,0x180)] { n.write_wram_word(a,v); }
            for (lo,hi) in [(0xbfa,0xc0e),(0xc04,0xc18)] {
                n.write_wram_byte(lo+k,xy as u8); n.write_wram_byte(hi+k,(xy>>8) as u8);
            }
            n.sync_native_game_state_from_ram();
            let mut s = machine(&n,0x07d576,k);
            // $07:D576 is a long entry; all child calls use the unmodified ROM.
            s.cpu.db = 8;
            let source = crate::cycle_ledger::assert_rom_marker_owners_work(
                &[0x07d576,0x07d607], s, |s| {
                    let own = s.cpu.k == 7 && (0xd576..0xd657).contains(&s.cpu.pc);
                    if own { edge(s,&edges); } own
                }, || n.hookshot_check_tile_collision(k as i32));
            for a in [0xa0,0xa1,0xee,0x59] {
                assert_eq!(n.read_wram_byte(a),source.snes().ram[a],
                    "slot {k}, work {work}, stairs {stairs}, collision {collision}, dir {dir}, xy {xy}, lower {lower}, address {a:x}");
            }
            assert_eq!(source.snes().cpu.db,8);
        }}}
    }}}}
    for pc in [0xd582,0xd587,0xd5c0] { for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}
