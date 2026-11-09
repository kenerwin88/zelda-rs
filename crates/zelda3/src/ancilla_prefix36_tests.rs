use crate::zelda_rtl::prefix34_tests::{machine, state};
use std::cell::RefCell;
use std::collections::BTreeSet;

#[test]
fn prefix36_hookshot_caller_prefix_all_guards_retraction_extension_bounds_and_hits_exclude_children() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for k in [0,5,9] { for path in 0..12 { for length in [0u8,1,3,4,30,31,32,127,128,255] {
        let mut n = base.clone();
        let position = if path == 6 {0xfff0u16} else {0x40};
        for (a,v) in [(0xc4a,0x1f),(0xc54,u8::from(path == 3 || path == 4)),
            (0xc5e,length),(0xc68,if path & 1 == 0 {0}else{7}),
            (0x385,u8::from(path == 7)),(0xc72,0),
            (0xbfa,position as u8),(0xc0e,(position>>8) as u8),
            (0xc04,position as u8),(0xc18,(position>>8) as u8)] { n.write_wram_byte(a+k,v); }
        n.write_wram_byte(0x11,u8::from(path == 0));
        n.write_wram_byte(0x37e,u8::from(path == 1));
        n.write_wram_byte(0x1b,u8::from(path != 5));
        n.write_wram_word(0x716,0x1e0);
        n.write_wram_word(0x90,0x800); n.write_wram_word(0x92,0xa20);
        // Ancilla_MoveX restores X through the dispatcher's live slot byte.
        n.write_wram_byte(0xfa0,k as u8);
        // A real eligible sprite gives both carry outcomes of BDD7's child.
        if path >= 8 {
            for (a,v) in [(0xdd0,9),(0xe20,0x8d),(0xd10,0x40),(0xd00,0x40),
                (0xcaa,2),(0xf50,8)] { n.write_wram_byte(a,v); }
        }
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x08bd74,
            machine(&n,0x08bd74,k),
            |s| {
                let own = s.cpu.k == 8 && (0xbd74..0xbe01).contains(&s.cpu.pc);
                if own {
                    let taken = match s.cart.read(8,s.cpu.pc,0) {
                        0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),
                        0x90=>Some(!s.cpu.c),0x30=>Some(s.cpu.n),_=>None,
                    };
                    if let Some(t) = taken { edges.borrow_mut().insert((s.cpu.pc,t)); }
                }
                own
            }, || n.ancilla1_f_hookshot(k));
        let image = n.wram_image();
        for a in [0xc4a+k,0xc54+k,0xc5e+k,0xc68+k,0xc22+k,0xc2c+k,
            0xbfa+k,0xc0e+k,0xc04+k,0xc18+k] {
            assert_eq!(image[a],source.snes().ram[a],
                "slot {k}, path {path}, length {length}, address {a:x}");
        }
    }}}
    for pc in [0xbd76,0xbd7b,0xbd8a,0xbd95,0xbd9a,0xbdac,0xbdc8,0xbdd0,0xbdd5,0xbdda] {
        for taken in [false,true] {
            assert!(edges.borrow().contains(&(pc,taken)),"branch {pc:x}/{taken}");
        }
    }
    // The hookshot-specific collision child never stores step ($0C54,X):
    // the preceding BDD5 already established zero, so BDDF cannot be taken.
    assert!(edges.borrow().contains(&(0xbddf,false)));
    assert!(!edges.borrow().contains(&(0xbddf,true)));
}
