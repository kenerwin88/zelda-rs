use super::*;
use crate::zelda_rtl::prefix34_tests::{machine, state};

#[test]
fn prefix38_recoil_guard_actual_call_sites_both_outcomes_exclude_nonzero_body() {
    let base = state();
    for (ty,call) in [(0x27,0x94f0),(0x10,0xd86c),(0x19,0x96c8)] {
        for slot in [0,15] { for recoil in [0,1,4,0x80,0x90] {
            let mut n = base.clone();
            for (a,v) in [(0xdd0,9),(0xe20,ty),(0xea0,recoil),(0xd10,0x40),(0xd00,0x40)] {
                n.write_wram_byte(a+slot,v);
            }
            n.write_wram_byte(0xfa0,slot as u8); n.sync_native_game_state_from_ram();
            let s = machine(&n,0x06ff78,slot);
            assert_eq!([s.cart.read(6,call,0),s.cart.read(6,call+1,0),s.cart.read(6,call+2,0)],
                [0x20,0x78,0xff],"unmodified caller {call:x}");
            crate::cycle_ledger::assert_rom_marker_work(0x06ff78,s, |s| s.cpu.k == 6
                && ((0xff78..0xff7d).contains(&s.cpu.pc) || recoil == 0 && s.cpu.pc == 0xfff3),
                || { n.sprite_return_if_recoiling(slot,SpriteRecoilEntry::Rom06FF78); });
        }}
    }
}

#[test]
fn prefix38_recoil_complete_banks_all_guards_masks_collisions_and_double_returns_exclude_children() {
    let base = state();
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for (entry,body,mode,end) in [(0x06ff78,0x06ff7d,SpriteRecoilEntry::Rom06FF78,0xfff8),
        (0x05f971,0x05f976,SpriteRecoilEntry::Rom05F971,0xf9ed)] {
        for slot in [0,15] { for recoil in [0,1,2,5,9,17,24,0x80,0x81,0x90] {
            for frame in [0,1,3,7] { for (xr,yr) in [(0,0),(223,0),(224,0),(31,0),(32,0),
                (0,223),(0,224),(0,31),(0,32)] { for tile in [0,2] { for bump in [0,128] {
                let mut n = base.clone();
                for (a,v) in [(0xdd0,9),(0xe20,0x7a),(0xea0,recoil),(0xf30,yr),(0xf40,xr),
                    (0xcd2,bump),(0xd40,0x37),(0xd50,0x62),(0xd10,0x40),(0xd00,0x40),
                    (0xbe0,128),(0xf60,0),(0xe40,0),(0xf50,0)] { n.write_wram_byte(a+slot,v); }
                n.write_wram_byte(0xfa0,slot as u8); n.write_wram_byte(0x1a,frame);
                n.write_wram_byte(0x1b,1);
                n.write_wram_word(0xfd8,0x40); n.write_wram_word(0xfda,0x40);
                for a in 0x12000..0x14000 { n.write_wram_byte(a,tile); }
                n.sync_native_game_state_from_ram();
                let mut returned = false;
                let source = crate::cycle_ledger::assert_rom_marker_owners_work(&[entry,body],
                    machine(&n,entry,slot), |s| {
                        let own = s.cpu.k == (entry>>16) as u8 && (entry as u16..end).contains(&s.cpu.pc);
                        if own {
                            let taken = match s.cart.read(s.cpu.k,s.cpu.pc,0) {
                                0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0x90=>Some(!s.cpu.c),
                                0xb0=>Some(s.cpu.c),0x30=>Some(s.cpu.n),_=>None,
                            };
                            if let Some(taken) = taken { edges.borrow_mut().insert((s.cpu.k,s.cpu.pc,taken)); }
                        }
                        own
                    }, || returned = n.sprite_return_if_recoiling(slot,mode));
                assert_eq!(returned,source.snes().cpu.sp==0x1fe,
                    "entry {entry:x}, slot {slot}, recoil {recoil}, frame {frame}, recoil XY {xr}/{yr}, tile {tile}, bump {bump}");
                let image = n.wram_image();
                for a in [0xea0,0xf30,0xf40,0xd40,0xd50,0xd00,0xd10,0xd20,0xd30,0xd60,0xd70] {
                    assert_eq!(image[a+slot],source.snes().ram[a+slot],
                        "entry {entry:x}, slot {slot}, recoil {recoil}, frame {frame}, recoil XY {xr}/{yr}, tile {tile}, bump {bump}, address {a:x}");
                }
            }}}}
        }}
    }
    for (bank,pcs) in [(6,&[0xff7b,0xff7f,0xff8c,0xff96,0xffa0,0xffaa,0xffb4,0xffc5,0xffd0,0xffd4][..]),
        (5,&[0xf974,0xf978,0xf985,0xf98f,0xf999,0xf9a3,0xf9ad,0xf9be,0xf9c5,0xf9c9][..])] {
        for &pc in pcs { for taken in [false,true] {
            assert!(edges.borrow().contains(&(bank,pc,taken)),"branch {bank:x}:{pc:x}/{taken}");
        }}
    }
}
