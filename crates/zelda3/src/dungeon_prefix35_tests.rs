use super::*;
use crate::zelda_rtl::prefix34_tests::{machine, state};

// Every shipped overlay, including the empty stream and every legal resume
// cursor. The cartridge is never patched; the pointer is the ROM table's own.
fn overlays() -> Vec<(usize, Vec<u8>, usize)> {
    let n = state();
    let data = n.asset_raw(48).unwrap();
    (0..n.asset_raw(49).unwrap().len() / 2).map(|index| {
        let offset = usize::from(n.asset_u16(49, index));
        let src = data[offset..].to_vec();
        let count = (0..src.len()).step_by(3)
            .take_while(|&p| src[p..p + 2] != [255, 255]).count();
        (index, src, count)
    }).collect()
}

fn overlay_machine(n: &ZeldaState, entry: u32, index: usize, cursor: usize) -> snes::Snes {
    let mut s = machine(n, entry, 0);
    s.cpu.db = 0; // Dungeon callers retain bank zero's predefined tile data.
    let p = 0xecc0 + index as u16 * 3;
    for i in 0..3 { s.ram[0xb7 + i] = s.cart.read(4, p + i as u16, 0); }
    s.ram[0xba..0xbc].copy_from_slice(&(cursor as u16).to_le_bytes());
    if entry == 0x01f967 { s.cpu.unpack_flags(0); }
    s
}

#[test]
fn prefix35_overlay_draw_loop_all_streams_and_resume_cursors_exclude_object_children() {
    let base = state();
    for (index, src, count) in overlays() { for cursor in (0..=count * 3).step_by(3) {
        let mut n = base.clone();
        n.write_wram_word(0xba, cursor as u16);
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x01f967,
            overlay_machine(&n, 0x01f967, index, cursor),
            |s| s.cpu.k == 1 && (0xf967..0xf980).contains(&s.cpu.pc),
            || n.dungeon_draw_room_overlay(&src));
        let image = n.wram_image();
        for range in [0xb2..0xb6, 0xba..0xbc, 0x2000..0x4000] {
            for a in range { assert_eq!(image[a], source.snes().ram[a],
                "overlay {index}, cursor {cursor}, address {a:x}"); }
        }
    }}
}

#[test]
fn prefix35_overlay_caller_all_streams_fresh_and_resumed_exclude_draw_dma_and_attribute_children() {
    let base = state();
    for (index, _, count) in overlays() { for cursor in (0..=count * 3).step_by(3) {
        let mut n = base.clone();
        n.write_wram_word(0xba, cursor as u16);
        n.write_wram_byte(0x4ba, index as u8);
        n.write_wram_byte(0x11, 3);
        n.sync_native_game_state_from_ram();
        let source = crate::cycle_ledger::assert_rom_marker_work(0x01b83e,
            overlay_machine(&n, 0x01b83e, index, cursor),
            |s| s.cpu.k == 1 && (0xb83e..0xb8a8).contains(&s.cpu.pc),
            || n.module07_03_overlay_change());
        let image = n.wram_image();
        for range in [0x11..0x12, 0x18..0x19, 0xba..0xbc,
            0x1000..0x1800, 0x2000..0x4000, 0x12000..0x13000] {
            for a in range { assert_eq!(image[a], source.snes().ram[a],
                "overlay {index}, cursor {cursor}, address {a:x}"); }
        }
    }}
}
