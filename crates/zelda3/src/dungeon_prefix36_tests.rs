use crate::zelda_rtl::prefix34_tests::{machine, state};

#[test]
fn prefix36_spiral_wall_scan_all_table_slots_layers_and_skip_paths_match_rom() {
    let base = state();
    // The physical table has four words. Every search termination, both
    // staircase directions, both tilemap layers and the five-column loop.
    for slot in 0..4 { for index in 0..8 { for layer in [0u16, 0x1000] {
        let mut n = base.clone();
        let target = layer + 0x184;
        n.write_wram_byte(0x462, index);
        n.write_wram_word(0x48c, target.wrapping_sub(4).wrapping_mul(2));
        for j in 0..4 { n.write_wram_word(0x6b0 + j*2,
            if j == slot {target} else {layer + 0x190 + j as u16}); }
        for a in (0x2000..0x6000).step_by(2) { n.write_wram_word(a, a as u16 ^ 0xa55a); }
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x0292b1,
            machine(&n,0x0292b1,0),
            |s| s.cpu.k == 2 && (0x92b1..0x9319).contains(&s.cpu.pc),
            || n.spiral_stairs_make_nearby_walls_high_priority_exiting());
        let image = n.wram_image();
        for range in [0x48c..0x48e, 0x2000..0x6000] {
            assert_eq!(&image[range.clone()], &source.snes().ram[range],
                "slot {slot}, index {index}, layer {layer}");
        }
    }}}
}

#[test]
fn prefix36_spiral_room_adjustment_all_room_axis_deltas_and_byte_wraps_match_rom() {
    let base = state();
    // Each axis is independent straight-line M8 code. Exercise all 16x16
    // room-nibble pairs on both axes, retaining unrelated high room bytes.
    for axis in 0..2 { for from in 0..16u16 { for to in 0..16u16 {
        for high in [0u16, 1, 0x7f, 0xff] {
            let mut n = base.clone();
            let shift = axis*4;
            n.write_wram_word(0xa2, 0xab00 | from << shift);
            n.write_wram_word(0xa0, 0xcd00 | to << shift);
            for a in [0x20,0x22,0xe2,0xe8,0x600,0x602,0x604,0x606,
                0x608,0x60a,0x60c,0x60e] { n.write_wram_word(a, high << 8 | 0x5a); }
            n.sync_native_game_state_from_ram();
            let source = crate::cycle_ledger::assert_rom_marker_work(0x02a2f0,
                machine(&n,0x02a2f0,0),
                |s| s.cpu.k == 2 && (0xa2f0..0xa37c).contains(&s.cpu.pc),
                || n.dungeon_adjust_after_spiral_stairs());
            let image = n.wram_image();
            for range in [0x20..0x24,0xe2..0xe4,0xe8..0xea,0x600..0x610] {
                assert_eq!(&image[range.clone()], &source.snes().ram[range],
                    "axis {axis}, from {from}, to {to}, high {high}");
            }
        }
    }}}
}
