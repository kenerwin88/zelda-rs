use crate::zelda_rtl::prefix34_tests::{machine, state};
use std::cell::RefCell;
use std::collections::BTreeSet;

#[test]
fn prefix36_overlord_activity_all_parities_axes_wraps_and_loaded_masks_match_rom() {
    let base = state();
    let edges = RefCell::new(BTreeSet::new());
    for k in [0,7] { for indoors in [0,1] { for frame in 0..2 {
        for scroll in [0u16,0xff,0x7fff,0xfff0] {
            for dx in [0u16,0x100,0x7fff,0x8000,0xffff] {
                for dy in [0u16,0x100,0x7fff,0x8000,0xffff] {
                    for block in [0u16,1,2,3,4,5,6,7,8,0xfff,0x1000,0x7fff,0x83ff,0x8400,0xfffe,0xffff] {
                        let mut n = base.clone();
                        n.write_wram_byte(0x1b,indoors); n.write_wram_byte(0x1a,frame);
                        n.write_wram_word(0xe2,scroll); n.write_wram_word(0xe8,scroll);
                        let x = scroll.wrapping_add(dx); let y = scroll.wrapping_add(dy);
                        for (a,v) in [(0xb00,1),(0xb08,x as u8),(0xb10,(x>>8) as u8),
                            (0xb18,y as u8),(0xb20,(y>>8) as u8)] { n.write_wram_byte(a+k,v); }
                        n.write_wram_word(0xb48+k*2,block);
                        for a in 0x1ef80..0x1ff80 { n.write_wram_byte(a,0xff); }
                        let destination = 0x10000 + usize::from(0xef80u16.wrapping_add(block >> 3));
                        n.write_wram_byte(destination,0xff);
                        n.sync_native_game_state_from_ram();
                        let source = crate::cycle_ledger::assert_rom_marker_work(0x09c08d,
                            machine(&n,0x09c08d,k),
                            |s| {
                                let own = s.cpu.k == 9 && (0xc08c..0xc114).contains(&s.cpu.pc);
                                if own {
                                    let t = match s.cart.read(9,s.cpu.pc,0) {
                                        0xd0=>Some(!s.cpu.z),0x90=>Some(!s.cpu.c),
                                        0xb0=>Some(s.cpu.c),0x30=>Some(s.cpu.n),
                                        0x10=>Some(!s.cpu.n),_=>None,
                                    };
                                    if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                                }
                                own
                            }, || n.overlord_check_if_active(k));
                        let image = n.wram_image();
                        assert_eq!(image[0xb00+k],source.snes().ram[0xb00+k]);
                        assert_eq!(&image[0x1ef80..0x1ff80],&source.snes().ram[0x1ef80..0x1ff80]);
                        assert_eq!(image[destination],source.snes().ram[destination],
                            "slot {k}, indoors {indoors}, frame {frame}, scroll {scroll}, dx {dx}, dy {dy}, block {block:x}");
                    }
                }
            }
        }
    }}}
    for pc in [0xc08f,0xc0b5,0xc0bd,0xc0da,0xc0e2,0xc101] { for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}
