use super::*;
use crate::game_state::constants::{ANIMATED_TILE_DATA_SRC, ANIMATED_TILE_VRAM_ADDR};

#[test]
fn held_nmi_reported_cycles_match_source_handler_and_rti() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    for held in [false, true] {
        for mode in [1, 9, 7] {
            let mut state = ZeldaState::new();
            state.ram.fill(0);
            state.ram[0x10] = 14;
            state.ram[0x11] = 2;
            state.ram[0x12] = u8::from(held);
            state.ram[0x710] = 2;
            state.ram[0x94] = mode;
            state.sync_native_game_state_from_ram();
            let checkpoint = RomCpuCheckpoint { entry_pc: 0x0080c9, stop_pc: 0x008036,
                a: 0xb700, x: 0, y: 20, sp: 0x1fb, dp: 0, db: 0,
                carry: false, zero: true, overflow: false, negative: false,
                interrupt_disable: true, decimal: false, accumulator_is_8_bit: true,
                index_is_8_bit: true, emulation: false, waiting: false,
                stack_address: 0x1fc, stack_bytes: &[0x32, 0x36, 0x80, 0] };
            let mut source = RomCpuTimingRun::new(&rom, &state.wram_image(), &state.sram,
                &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
            let mut source_cycles = 62u64;
            for _ in 0..2000 {
                if source.is_complete() { break; }
                source_cycles += u64::from(source.step().master_cycles);
                source_cycles += u64::from(source.drain_started_dma_master_cycles());
            }
            assert!(source.is_complete(), "held NMI must consume its real RTI frame");
            state.interrupt_nmi(0, None, false);
            assert_eq!(u64::from(state.last_nmi_handler_master_cycles.unwrap()), source_cycles,
                "NMI held={held} mode={mode}");
        }
    }
}

#[test]
fn link_dma_accounting_matches_source_mdma_groups_and_payloads() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets_path = std::env::var_os("ZELDA3_ASSETS").map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("zelda3_assets.dat"));
    let Ok(assets) = std::fs::read(assets_path) else { return };
    let mut rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for bird in [false, true] {
        for captured in [false, true] {
            let mut native = ZeldaState::new();
            native.set_assets(&assets).unwrap();
            native.ppu.forced_blank = true;
            for (i, byte) in native.ram[0x8000..0xa000].iter_mut().enumerate() {
                *byte = (i ^ (i >> 8)) as u8;
            }
            for (i, address) in (0xac0..0xafa).step_by(2).enumerate() {
                write_le_u16(&mut native.ram, address, 0x9000 + i as u16 * 0x40);
            }
            native.ram[0xaf4] = u8::from(bird);
            native.sync_native_game_state_from_ram();
            let checkpoint = RomCpuCheckpoint {
                entry_pc: 0x00_89e0, stop_pc: 0x00_8b50,
                a: 0, x: 0, y: 0, sp: 0x01fc, dp: 0, db: 0,
                carry: false, zero: false, overflow: false, negative: false,
                interrupt_disable: true, decimal: false,
                accumulator_is_8_bit: true, index_is_8_bit: true,
                emulation: false, waiting: false,
                stack_address: 0x01fd, stack_bytes: &[0x40, 0x81],
            };
            let mut source = RomCpuTimingRun::new(&rom, &native.wram_image(), &native.sram,
                &native.ppu, &native.dma, [0; 4], checkpoint).unwrap();
            let mut runs = 0;
            let mut master = 0u64;
            for _ in 0..1000 {
                if source.is_complete() { break; }
                source.step();
                let dma = source.drain_started_dma_master_cycles();
                runs += u32::from(dma != 0);
                master += u64::from(dma);
            }
            assert!(source.is_complete(), "Link DMA did not reach animated-BG setup");
            let operands = captured.then(|| PreMainLinkDmaOperands::capture(&native.wram_image()));
            if captured {
                // The selected generation owns these operands, including the
                // staged planes, even when the live main thread changes them.
                native.ram[0xac0..0xaf4].fill(0);
                native.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START
                    ..LINK_DMA_EXPANDED_HIGH_PLANES_START + 2 * LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN]
                    .fill(0xa5);
            }
            let prior = NmiDmaAccounting { runs: 1, channels: 1, bytes: 544 };
            native.nmi_dma_accounting = Some(prior);
            native.nmi_core_link_graphics_update(operands);
            let actual = native.nmi_dma_accounting.unwrap();
            assert_eq!(actual.runs - prior.runs, runs);
            assert_eq!(actual.master_cycles() - prior.master_cycles(), master);
            assert_eq!(actual.channels - prior.channels, 22 + 2 * u32::from(bird));
            assert_eq!(actual.bytes - prior.bytes, 1280 + 128 * u64::from(bird));
            assert_eq!(&native.ppu.vram[..], source.vram(), "bird={bird} captured={captured}");
        }
    }
}

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

    s.nmi_copy_packets();
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

    s.nmi_copy_packets();

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
