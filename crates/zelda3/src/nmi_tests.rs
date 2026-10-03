use super::*;
use crate::game_state::constants::{ANIMATED_TILE_DATA_SRC, ANIMATED_TILE_VRAM_ADDR};

#[test]
fn native_nmi_header_bus_writes_keep_typed_latches_and_hardware_owned() {
    use crate::cycle_models::nmi_stack::NmiHeaderAccess::*;
    let mut s = ZeldaState::new();
    let mut clock = snes::CpuMasterTimeline::new(0, Default::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
    clock.begin_synchronous_timeline().unwrap();
    s.native_ppu = Some(crate::cycle_models::native_ppu::NativePpuState::reset(&clock));
    s.ram[0x12d] = 0x37;
    s.ram[0x12e] = 0x5a;
    s.ram[0x12f] = 0xa5;
    s.ram[0x13] = 0x0f;
    s.sync_native_game_state_from_ram();
    let mut expected = s.ram.clone();
    s.ppu.forced_blank = false;
    for channel in &mut s.dma.channel { channel.hdma_active = true; }
    assert_eq!(s.native_nmi_header_access(&clock, Read(0x12d)), 0x37);
    for (address, value) in [(0x131, 0x37), (0x12d, 0), (0x12e, 0), (0x12f, 0), (0x12, 1)] {
        s.native_nmi_header_access(&clock, Write(address, value));
        expected[usize::from(address)] = value;
    }
    s.native_nmi_header_access(&clock, WriteWord(0x1f1, 0x8140));
    expected[0x1f1..0x1f3].copy_from_slice(&[0x40, 0x81]);
    s.native_nmi_header_access(&clock, Write(0x2100, 0x80));
    s.native_nmi_header_access(&clock, Write(0x420c, 0));
    assert_eq!(s.ram, expected);
    assert_eq!(s.game_state.system_signals.last_ambient_sound_effect(), 0x37);
    assert_eq!(s.game_state.system_signals.ambient_sound_effect(), 0);
    assert_eq!(s.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(s.game_state.system_signals.sound_effect_2(), 0);
    assert!(s.game_state.display.nmi_update_is_latched());
    assert_eq!(s.game_state.display.screen_brightness, 0x0f);
    assert!(s.ppu.forced_blank);
    assert!(s.dma.channel.iter().all(|channel| !channel.hdma_active));
}

#[test]
fn idle_music_nmi_prices_a_nonzero_echo_from_the_apu_port() {
    assert_eq!(idle_music_echo_master_cycles(0x10, 0x10), 130);
    assert_eq!(idle_music_echo_master_cycles(0, 0x10), 84);
}

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
