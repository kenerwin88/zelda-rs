use super::*;
use crate::zelda_rtl::prefix34_tests::{machine, state};
use std::cell::RefCell;
use std::collections::BTreeSet;

fn record_edge(s: &snes::Snes, edges: &RefCell<BTreeSet<(u16, bool)>>) {
    let taken = match s.cart.read(s.cpu.k, s.cpu.pc, 0) {
        0x10 => Some(!s.cpu.n), 0x30 => Some(s.cpu.n),
        0xf0 => Some(s.cpu.z), 0xd0 => Some(!s.cpu.z),
        0x90 => Some(!s.cpu.c), 0xb0 => Some(s.cpu.c), _ => None,
    };
    if let Some(t) = taken { edges.borrow_mut().insert((s.cpu.pc, t)); }
}

fn assert_edges(edges: &RefCell<BTreeSet<(u16, bool)>>, pcs: &[u16]) {
    for &pc in pcs { for t in [false, true] {
        assert!(edges.borrow().contains(&(pc,t)), "branch {pc:x}/{t}");
    }}
}

#[test]
fn prefix35_hookshot_proximity_all_signed_paths_and_wraps_match_rom() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for origin in [0u16, 255, 0xffff] { for dy in [-32768i16, -13,-12,-11,0,11,12,13,32767] {
        for dx in [-32768i16, -13,-12,-11,0,11,12,13,32767] {
            let mut n = base.clone();
            let x = origin.wrapping_add(8).wrapping_sub(4).wrapping_sub(dx as u16);
            let y = origin.wrapping_add(12).wrapping_sub(4).wrapping_sub(dy as u16);
            n.write_wram_word(0x20, origin); n.write_wram_word(0x22, origin);
            n.sync_native_game_state_from_ram();
            let mut s = machine(&n, 0x08f7dc, 0);
            s.ram[0..2].copy_from_slice(&y.to_le_bytes());
            s.ram[2..4].copy_from_slice(&x.to_le_bytes());
            let mut result = false;
            let source = crate::cycle_ledger::assert_rom_marker_work(0x08f7dc, s,
                |s| { let own = s.cpu.k == 8 && (0xf7dc..0xf824).contains(&s.cpu.pc);
                    if own { record_edge(s, &edges); } own },
                || result = n.hookshot_check_proximity_to_link(i32::from(x), i32::from(y)));
            assert_eq!(result, source.snes().cpu.c, "origin {origin}, dy {dy}, dx {dx}");
        }
    }}
    assert_edges(&edges, &[0xf7fa,0xf803,0xf811,0xf81a]);
}

#[test]
fn prefix35_hookshot_bounds_all_indoor_outdoor_axes_edges_and_page_crossings_match_rom() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for k in [0,5,6,9] { for indoors in [0,1] { for direction in 0..4 {
        for area in [0usize,1,31,63] { for position in [0u16,3,4,5,6,7,0x1df,0x1e7,0x1e8,0x1ef,0x1f0,0x1ff,0x200,0x204,0x3ef,0xffff] {
            for player in [0,0x200] {
                let mut n = base.clone();
                n.write_wram_byte(0x1b, indoors); n.write_wram_byte(0xc72+k,direction);
                n.write_wram_word(0x700, area as u16 * 2); n.write_wram_word(0x716,0x1e0);
                let y = if indoors == 0 {position.wrapping_add(ANCILLA_OVERWORLD_AREA_BASE_Y[area])} else {position};
                let x = if indoors == 0 {position.wrapping_add(ANCILLA_OVERWORLD_AREA_BASE_X[area])} else {position};
                n.write_wram_byte(0xbfa+k,y as u8); n.write_wram_byte(0xc0e+k,(y>>8) as u8);
                n.write_wram_byte(0xc04+k,x as u8); n.write_wram_byte(0xc18+k,(x>>8) as u8);
                n.write_wram_word(0x20,player); n.write_wram_word(0x22,player);
                n.sync_native_game_state_from_ram();
                let mut result = false;
                let source = crate::cycle_ledger::assert_rom_marker_work(0x08fa2d, machine(&n,0x08fa2d,k),
                    |s| {let own=s.cpu.k==8 && (0xfa2d..0xfadd).contains(&s.cpu.pc);
                        if own {record_edge(s,&edges);} own},
                    || result=n.hookshot_should_i_even_bother_with_tiles(k));
                assert_eq!(result,source.snes().cpu.c,"slot {k}, indoors {indoors}, dir {direction}, area {area}, position {position}");
            }
        }}
    }}}
    assert_edges(&edges, &[0xfa45,0xfa4f,0xfa5e,0xfa63,0xfa74,0xfa79,0xfa8f,0xfa99,
        0xfa9e,0xfaaa,0xfaaf,0xfac3,0xfad7]);
}

#[test]
fn prefix35_hookshot_drawing_all_directions_lengths_priority_clipping_and_proximity_exclude_children() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for k in [0,5,9] { for direction in 0..4 { for length in 0..=32 { for priority in [0,1] {
        for position in [0u16,0x40,0xee,0xf0,0xf8,0xff,0x100,0xfffc] { for player in [0,0x40] {
            let mut n = base.clone();
            for (a,v) in [(0xbfa,position as u8),(0xc0e,(position>>8) as u8),
                (0xc04,position as u8),(0xc18,(position>>8) as u8),
                (0xc72,direction),(0xc5e,length),(0x385,priority)] {n.write_wram_byte(a+k,v);}
            n.write_wram_word(0x20,player); n.write_wram_word(0x22,player);
            n.write_wram_word(0x90,0x800); n.write_wram_word(0x92,0xa20);
            n.sync_native_game_state_from_ram();
            let source = crate::cycle_ledger::assert_rom_marker_work(0x08bedc,machine(&n,0x08bedc,k),
                |s| {let own=s.cpu.k==8 && (0xbedc..0xc003).contains(&s.cpu.pc);
                    if own {record_edge(s,&edges);} own},||n.hookshot_draw(k));
            let image=n.wram_image();
            for a in 0x800..0xa40 {assert_eq!(image[a],source.snes().ram[a],
                "slot {k}, direction {direction}, length {length}, position {position}, player {player}, address {a:x}");}
        }}
    }}}}
    assert!(edges.borrow().contains(&(0xbf6d,true)));
    assert!(!edges.borrow().contains(&(0xbf6d,false)));
    assert_edges(&edges,&[0xbee2,0xbf0e,0xbf3d,0xbf41,0xbf60,0xbf77,0xbf82,
        0xbf97,0xbfa4,0xbfb5,0xbfc4,0xbfd5,0xbfff]);
}
