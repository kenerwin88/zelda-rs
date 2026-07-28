use super::*;
use crate::game_state::constants::{ANIMATED_TILE_DATA_SRC, ANIMATED_TILE_VRAM_ADDR};

#[test]
fn nmi_core_update_copies_animated_tiles_even_from_zero_source() {
    let mut s = ZeldaState::new();
    write_le_u16(&mut s.ram, ANIMATED_TILE_DATA_SRC, 0);
    write_le_u16(&mut s.ram, ANIMATED_TILE_VRAM_ADDR, 0x6000);
    write_le_u16(&mut s.ram, 0, 0x1234);
    write_le_u16(&mut s.ram, 2, 0x5678);
    s.sync_native_game_state_from_ram();

    s.nmi_do_updates();

    assert_eq!(s.ppu.vram[0x6000], 0x1234);
    assert_eq!(s.ppu.vram[0x6001], 0x5678);
}

#[test]
#[should_panic(expected = "invalid nmi_subroutine_index")]
fn nmi_do_updates_panics_on_invalid_subroutine_like_c_table_index() {
    let mut s = ZeldaState::new();
    s.set_core_update_disable_flag(1);
    s.set_pending_nmi_subroutine(25);

    s.nmi_do_updates();
}

#[test]
#[should_panic(expected = "invalid NMI packet vmain")]
fn nmi_copy_packets_panics_on_invalid_vmain_like_c_assert() {
    let mut s = ZeldaState::new();
    write_le_u16(&mut s.ram, 0x1100, 0x2000);
    s.ram[0x1102] = 0x82;
    s.ram[0x1103] = 0;

    s.NMI_CopyPackets();
}

#[test]
fn nmi_copy_packets_reads_uvram_data_not_uvram_header() {
    let mut s = ZeldaState::new();
    write_le_u16(&mut s.ram, 0x1000, 0);
    write_le_u16(&mut s.ram, 0x1002, 0x207f);
    write_le_u16(&mut s.ram, 0x1100, 0x2000);
    write_le_u16(&mut s.ram, 0x1102, 0x0480);
    s.ram[0x1104..0x1108].copy_from_slice(&[0x34, 0x12, 0x78, 0x56]);
    write_le_u16(&mut s.ram, 0x1108, 0xffff);

    s.NMI_CopyPackets();

    assert_eq!(s.ppu.vram[0x2000], 0x1234);
    assert_eq!(s.ppu.vram[0x2001], 0x5678);
}

#[test]
fn nmi_copy_packet_decoder_shares_horizontal_and_vertical_packet_semantics() {
    let data = [
        0x00, 0x20, 0x80, 0x04, 0x34, 0x12, 0x78, 0x56, 0x10, 0x20, 0x81, 0x02, 0xbc, 0x9a, 0xff,
        0xff,
    ];

    let packets = nmi_vram_copy_packets(&data);

    assert_eq!(
        packets,
        vec![
            NmiVramCopyPacket {
                destination: 0x2000,
                direction: NmiVramCopyDirection::Horizontal,
                data: &[0x34, 0x12, 0x78, 0x56],
            },
            NmiVramCopyPacket {
                destination: 0x2010,
                direction: NmiVramCopyDirection::Vertical,
                data: &[0xbc, 0x9a],
            },
        ]
    );
}
