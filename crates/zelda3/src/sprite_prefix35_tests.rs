use crate::zelda_rtl::prefix34_tests::{state, machine};

#[test]
fn prefix35_zora_main_and_surface_attack_submerge_all_paths_exclude_selector_and_children() {
    let base = state();
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for k in [0,15] { for ai in 0..4 { for timer in 0..=127 { for inactive in [false,true] {
        if ai == 3 && timer >= 24 {continue;}
        let mut n = base.clone();
        for (a,v) in [(0xdd0,if inactive {0} else {9}), (0xe20,0x55), (0xd80,ai),
            (0xdf0,timer),(0xe60,0x40),(0xd10,0x40),(0xd00,0x40)] {n.write_wram_byte(a+k,v);}
        n.write_wram_word(0x20,0x100); n.write_wram_word(0x22,0x100);
        n.write_wram_word(0x90,0x800); n.write_wram_word(0x92,0xa20);
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_owners_work(
            &[0x059736,0x0597c5,0x0597f1,0x059824],machine(&n,0x059725,k),
            |s| {
                let pc=s.cpu.pc;
                let own=s.cpu.k==5 && ((0x9736..0x974a).contains(&pc)
                    || (0x97c5..0x97e9).contains(&pc) || (0x97f1..0x9818).contains(&pc)
                    || (0x9824..0x983f).contains(&pc));
                if own {
                    let taken=match s.cart.read(5,pc,0) {
                        0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),_=>None};
                    if let Some(t)=taken {edges.borrow_mut().insert((pc,t));}
                }
                own
            },||n.sprite_zora_main(k));
        let image=n.wram_image();
        for a in [0xd80,0xdf0,0xdc0,0xe60,0xba0] {assert_eq!(image[a+k],source.snes().ram[a+k],
            "slot {k}, ai {ai}, timer {timer}, inactive {inactive}, address {a:x}");}
    }}}}
    for pc in [0x9739,0x973c,0x973f,0x97cb,0x97f7,0x9804,0x9827] {
        for t in [false,true] {assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");}
    }
}
