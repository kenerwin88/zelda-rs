use crate::zelda_rtl::prefix34_tests::{machine, state};
use std::cell::RefCell;
use std::collections::BTreeSet;

fn edge(s: &snes::Snes, edges: &RefCell<BTreeSet<(u16, bool)>>) {
    let taken = match s.cart.read(s.cpu.k, s.cpu.pc, 0) {
        0xf0 => Some(s.cpu.z), 0xd0 => Some(!s.cpu.z),
        0x90 => Some(!s.cpu.c), 0xb0 => Some(s.cpu.c), _ => None,
    };
    if let Some(taken) = taken { edges.borrow_mut().insert((s.cpu.pc, taken)); }
}

fn both(edges: &RefCell<BTreeSet<(u16, bool)>>, pcs: &[u16]) {
    for &pc in pcs { for taken in [false, true] {
        assert!(edges.borrow().contains(&(pc, taken)), "branch {pc:x}/{taken}");
    }}
}

#[test]
fn prefix38_layout_all_layouts_quadrants_blast_walls_and_overrides_exclude_quadrant_child() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for layout in (0..32).step_by(4) { for x in [0,1] { for y in [0,2] {
        for blast in 0..4 { for reset in [0u16,0x0033,0x4400,0x4433] {
            let mut n = base.clone();
            for (a,v) in [(0x40e,layout),(0xa9,x),(0xaa,y),
                (0x452,blast&1),(0x453,blast>>1)] { n.write_wram_byte(a,v); }
            n.write_wram_word(0xfc,reset); n.sync_native_game_state_from_ram();
            let source = crate::cycle_ledger::assert_rom_marker_work(0x02b5dc,
                machine(&n,0x02b5dc,0), |s| {
                    let own = s.cpu.k == 2 && (0xb5dc..0xb62e).contains(&s.cpu.pc);
                    if own { edge(s,&edges); } own
                }, || n.dungeon_adjust_for_room_layout());
            for a in [0xa6,0xa7,0xa8] { assert_eq!(n.ram[a],source.snes().ram[a],
                "layout {layout}, quadrants {x}/{y}, blast {blast}, reset {reset:x}, address {a:x}"); }
        }}
    }}}
    both(&edges,&[0xb5ea,0xb5f3,0xb5fc,0xb608,0xb611,0xb61a,0xb622,0xb628]);
}

#[test]
fn prefix38_quadrant_key_all_layout_bytes_and_quadrants_match_rom() {
    let base = state();
    for layout in 0..=255 { for x in [0,1] { for y in [0,2] {
        let mut n = base.clone();
        for (a,v) in [(0x40e,layout),(0xa9,x),(0xaa,y)] { n.write_wram_byte(a,v); }
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x02ba27,
            machine(&n,0x02ba27,0), |s| s.cpu.k == 2 && (0xba27..0xba31).contains(&s.cpu.pc),
            || n.dungeon_adjust_quadrant());
        assert_eq!(n.ram[0xa8],source.snes().ram[0xa8]);
    }}}
}

#[test]
fn prefix38_scroll_mirror_all_byte_carries_match_rom() {
    let base = state();
    for h in [0u16,255,256,65535] { for v in [0u16,255,256,65535] {
        let mut n = base.clone();
        n.write_wram_word(0xe2,h); n.write_wram_word(0xe8,v);
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x02bb7b,
            machine(&n,0x02bb7b,0), |s| s.cpu.k == 2 && (0xbb7b..0xbb88).contains(&s.cpu.pc),
            || n.MirrorBg1Bg2Offs());
        for a in [0xe0,0xe1,0xe6,0xe7] { assert_eq!(n.ram[a],source.snes().ram[a]); }
    }}
}

#[test]
fn prefix38_movement_far_entry_all_caller_banks_and_paths_exclude_movement_children() {
    let base = state();
    for bank in [0,2,7,8,255] { for handler in [0,10] { for platform in [0,2] {
        for airborne in [0,1] { for velocity in [0,127,128,255] {
            let mut n = base.clone();
            for (a,v) in [(0x5d,handler),(0x2f5,platform),(0x4d,airborne),
                (0x27,velocity),(0x28,velocity),(0x29,velocity)] { n.write_wram_byte(a,v); }
            n.sync_native_game_state_from_ram();
            let mut s = machine(&n,0x07e370,0); s.cpu.db = bank;
            let source = crate::cycle_ledger::assert_rom_marker_work(0x07e370,s,
                |s| s.cpu.k == 7 && (0xe370..0xe373).contains(&s.cpu.pc),
                || n.link_move_position_long_entry());
            assert_eq!(source.snes().cpu.db,bank);
        }}
    }}}
}
