use super::*;
use crate::zelda_rtl::prefix34_tests::{machine, state};
use std::cell::RefCell;
use std::collections::BTreeSet;

#[test]
fn prefix37_follower_trigger_proximity_all_signed_guards_edges_and_wraps_match_rom() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    let deltas = [-32768i32,-256,-29,-28,-27,-25,-24,-23,-1,0,1,23,24,25,27,28,29,255,256,32767];
    for x in [0u16,255,65535] { for y in [0u16,255,65535] {
        for dx in deltas { for dy in deltas {
            let info = TagalongMessageInfo { x,y,bit:1,msg:0,tagalong:1 };
            let mut n = base.clone();
            n.write_wram_word(0x22,x.wrapping_add(dx as u16).wrapping_sub(4));
            n.write_wram_word(0x20,y.wrapping_add(dy as u16).wrapping_sub(4));
            n.write_wram_word(0x00,y); n.write_wram_word(0x02,x); n.write_wram_word(0x0a,0);
            n.sync_native_game_state_from_ram();
            let mut hit = false;
            let source = crate::cycle_ledger::assert_rom_marker_work(0x09ac26,
                machine(&n,0x09ac26,0), |s| {
                    let own = s.cpu.k==9 && (0xac26..0xac6b).contains(&s.cpu.pc);
                    if own {
                        let t=match s.cart.read(9,s.cpu.pc,0) {
                            0x10=>Some(!s.cpu.n),0xb0=>Some(s.cpu.c),_=>None};
                        if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                    }
                    own
                },||hit=n.follower_check_for_trigger(&info));
            assert_eq!(hit,source.snes().cpu.c,"anchor {x}/{y}, delta {dx}/{dy}");
        }}
    }}
    for pc in [0xac44,0xac4d,0xac58,0xac61] {for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}

#[test]
fn prefix37_follower_trigger_scan_all_rooms_records_guards_and_hits_exclude_handoffs() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for indoor in [false,true] {
        let rooms = if indoor { &TAGALONG_INDOOR_ROOMS[..] } else { &TAGALONG_OUTDOOR_ROOMS[..] };
        let infos = if indoor { &TAGALONG_INDOOR_INFOS[..] } else { &TAGALONG_OUTDOOR_INFOS[..] };
        for room in rooms.iter().copied().chain([0xffff]) {
            for submodule in [0,1] { for indicator in [0,1,4,6,10,14] {
                for position in std::iter::once((0xffff,0xffff)).chain(
                    infos.iter().filter(|i|i.bit!=0).map(|i|(i.x.wrapping_sub(4),i.y.wrapping_sub(4)))) {
                    let mut n = base.clone();
                    n.write_wram_byte(0x1b,u8::from(indoor)); n.write_wram_byte(0x11,submodule);
                    n.write_wram_word(if indoor {0xa0} else {0x8a},room);
                    n.write_wram_byte(0xf3cc,indicator); n.write_wram_word(0x2f2,0xffff);
                    n.write_wram_word(0x22,position.0); n.write_wram_word(0x20,position.1);
                    n.sync_native_game_state_from_ram();
                    let source = crate::cycle_ledger::assert_rom_marker_work(0x09a59e,
                        machine(&n,0x09a59e,0), |s| {
                            let own = s.cpu.k==9 && ((0xa59e..0xa657).contains(&s.cpu.pc)
                                || (0xa6ca..0xa6cd).contains(&s.cpu.pc));
                            if own {
                                let t=match s.cart.read(9,s.cpu.pc,0) {
                                    0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),
                                    0x10=>Some(!s.cpu.n),0xb0=>Some(s.cpu.c),_=>None};
                                if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                            }
                            own
                        },||n.follower_handle_trigger());
                    for a in [0x2f2,0x2f3,0xf3cc,0x1cf0,0x1cf1] {
                        assert_eq!(n.read_wram_byte(a),source.snes().ram[a],
                            "indoor {indoor}, room {room:x}, sub {submodule}, indicator {indicator}, position {position:x?}, address {a:x}");
                    }
                }
            }}
        }
    }
    for pc in [0xa5a0,0xa5ac,0xa5b7,0xa5bb,0xa5c7,0xa5cb,0xa5e7,0xa602,0xa60f,
        0xa62b,0xa646,0xa653] {for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}
