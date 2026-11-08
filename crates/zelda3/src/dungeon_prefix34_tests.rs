use crate::zelda_rtl::prefix34_tests::{state, machine};

#[test]
fn prefix34_transition_landing_every_tile_floor_direction_and_wrap_matches_rom() {
    let base = state();
    for tile in 0..=255u8 { for floor in [0,1] { for direction in 0..4 { for xy in [0,0xff,0xffff] {
        let mut n = base.clone();
        n.write_wram_word(0x20,xy); n.write_wram_word(0x22,xy);
        n.write_wram_byte(0xee,floor); n.write_wram_byte(0x418,direction);
        let pos=(((xy.wrapping_add(12)&0x1f8)<<3)|((xy.wrapping_add(8)&0x1f8)>>3)) as usize
            + usize::from(floor)*0x1000;
        n.write_wram_byte(0x12000+pos,tile); n.sync_native_game_state_from_ram();
        for entry in [0x02c1e5,0x02c12c] {
            let mut n=n.clone();
            let source=crate::cycle_ledger::assert_rom_marker_work(entry,machine(&n,entry,0),
                |s|s.cpu.k==2 && if entry==0x02c1e5 {(0xc1e5..0xc23e).contains(&s.cpu.pc)}
                    else {(0xc12c..0xc162).contains(&s.cpu.pc)},
                || if entry==0x02c1e5 {n.CalculateTransitionLanding();} else {n.SubtileTransitionCalculateLanding();});
            for a in [0x20,0x21,0x22,0x23,0x4b,0x4e] {assert_eq!(n.ram[a],source.snes().ram[a]);}
        }
    }}}}
}
