use crate::cycle_models::{
    intro_clear::Access,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    overworld_screen_palettes::OverworldScreenPalettes,
    pre_overworld_properties::{PreOverworldProperties, PropertiesChild},
};
use snes::CpuMasterTimeline;
#[test]
fn screen_palette_family_keeps_properties_frames_and_source_palette_buffers() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let mut instructions = 0;
    for case in 0..768usize {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ case.wrapping_mul(37)) as u8;
        }
        let r = &mut machine.ram;
        r[0x10] = 8;
        r[0x1b] = u8::from(case & 1 != 0);
        r[0x8a..0x8c].copy_from_slice(&((case % 256) as u16).to_le_bytes());
        r[0xaa8..0xaaa].fill(0);
        r[0x15] = case as u8;
        r[0xab1] = (case / 11 % 16) as u8;
        r[0xab2] = (case / 16 % 4) as u8;
        r[0xab8] = [0, 1, 3, 11][case / 16 % 4];
        r[0xabd..0xabf].copy_from_slice(&u16::from(case % 3 == 0).to_le_bytes());
        r[0xaad] = (case / 5 % 24) as u8;
        r[0xaae] = (case / 7 % 24) as u8;
        r[0xf3ca] = if case & 2 != 0 { 0x40 } else { 0 };
        r[0xf359] = (case % 5) as u8;
        r[0xf35a] = (case / 5 % 4) as u8;
        r[0xf35b] = (case / 20 % 3) as u8;
        r[0xf354] = (case / 60 % 3) as u8;
        r[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        let mut cpu = NativeNmiContext {
            a: 0xb701,
            x: 2,
            y: 0x60,
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x8496,
            sp: 0x1fa,
            p: 0x30 | (case as u8 & 0xc3),
        };
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.db = cpu.db;
        machine.cpu.k = 2;
        machine.cpu.pc = cpu.pc;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let clock = CpuMasterTimeline::at_raster(
            case as u64,
            [
                snes::CpuRasterPosition::new(100, 532),
                snes::CpuRasterPosition::new(224, 1338),
                snes::CpuRasterPosition::new(261, 1350),
            ][case % 3],
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            snes::RomCpuTimingProbe::new(machine, clock, snes::SourcePpuReadState::snes9x_reset())
                .unwrap();
        let mut clock = source.timeline().clone();
        let mut caller = PreOverworldProperties::at_instruction_for_test(0x8496);
        let mut palettes = OverworldScreenPalettes::default();
        let mut done = false;
        for _ in 0..30000 {
            let child = caller.child();
            if child == Some(PropertiesChild::OverworldPalettes) {
                done = true;
                break;
            }
            let label = format!(
                "screen palettes case={case} pc={:02x}:{:04x}",
                cpu.bank, cpu.pc
            );
            compare_instruction_with_devices(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                None,
                |_, access| match access {
                    Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                        let read = |address| {
                            state
                                .native_overworld_palette_byte(address)
                                .or_else(|| state.native_overworld_exit_table_byte(address))
                                .unwrap_or_else(|| {
                                    panic!("unowned palette data {address:06x}; {label}")
                                })
                        };
                        Some(if width == 1 {
                            u16::from(read(a))
                        } else {
                            u16::from_le_bytes([read(a), read(a + 1)])
                        })
                    }
                    _ => None,
                },
                |cpu, clock, bus| match child {
                    None => {
                        caller.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::ScreenPalette) => {
                        palettes.step(cpu, clock, bus).unwrap();
                    }
                    _ => panic!("unexpected palette child {child:?}"),
                },
            );
            if child == Some(PropertiesChild::ScreenPalette) && palettes.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x8499, 0x1fa));
                caller.return_from_child(&cpu);
            }
            assert_eq!(ram, source.snes().ram, "{label}");
            instructions += 1;
        }
        assert!(done && palettes.complete());
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (14, 0xd5a8, 0x1f7));
        assert_eq!(ram, source.snes().ram);
    }
    assert!(state.rom.is_empty());
    eprintln!("native screen-palette cases=768 Source instruction boundaries={instructions}");
}
