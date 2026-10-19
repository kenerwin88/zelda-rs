use crate::cycle_models::{
    animated_overworld_graphics::AnimatedOverworldGraphics,
    intro_clear::Access,
    native_tilesets::NativeTilesets,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    pre_overworld_properties::{PreOverworldProperties, PropertiesChild},
};
use snes::CpuMasterTimeline;
#[test]
fn properties_graphics_calls_keep_live_frames_clock_assets_and_ppu_writes() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let mut instructions = 0;
    for case in 0..16usize {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 7) ^ case) as u8;
        }
        machine.ram[0x10] = 8;
        machine.ram[0x1b] = 0;
        machine.ram[0xaa1..0xaa5].copy_from_slice(&[
            if case & 1 == 0 { 0x20 } else { 0x21 },
            [0, 3, 5, 11][case / 2 % 4],
            [0, 1, 11, 32][case / 4 % 4],
            if case & 1 == 0 { 1 } else { 11 },
        ]);
        machine.ram[0xc2fc..0xc300].copy_from_slice(&[12, 82, 90, 95]);
        machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        let mut cpu = NativeNmiContext {
            a: 0xb701,
            x: 2,
            y: if case & 2 == 0 { 0x58 } else { 0x5a },
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x848e,
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
        let mut ppu = machine.ppu.clone();
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
        let mut caller = PreOverworldProperties::at_instruction_for_test(0x848e);
        let mut animated = AnimatedOverworldGraphics::default();
        let mut tilesets = NativeTilesets::default();
        let mut done = false;
        for _ in 0..1000000 {
            let child = caller.child();
            if child == Some(PropertiesChild::ScreenPalette) {
                done = true;
                break;
            }
            let label = format!(
                "properties graphics case={case} pc={:02x}:{:04x}",
                cpu.bank, cpu.pc
            );
            compare_instruction_with_devices(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                Some(&mut ppu),
                |_, access| match access {
                    Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                        let read = |address| {
                            state
                                .native_graphics_asset_byte(address)
                                .unwrap_or_else(|| {
                                    panic!("unowned graphics asset {address:06x}; {label}")
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
                    Some(PropertiesChild::AnimatedTiles) => {
                        animated.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::Tilesets) => {
                        tilesets.step(cpu, clock, bus).unwrap();
                    }
                    _ => panic!("unexpected graphics child {child:?}"),
                },
            );
            if child == Some(PropertiesChild::AnimatedTiles) && animated.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x8492, 0x1fa));
                caller.return_from_child(&cpu);
            }
            if child == Some(PropertiesChild::Tilesets) && tilesets.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x8496, 0x1fa));
                caller.return_from_child(&cpu);
            }
            assert_eq!(ram, source.snes().ram, "{label}");
            instructions += 1;
        }
        assert!(done && animated.complete() && tilesets.complete());
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xc692, 0x1f8));
        assert_eq!(ppu.vram, source.snes().ppu.vram);
        assert_eq!(ppu.cgram, source.snes().ppu.cgram);
        assert_eq!(ram, source.snes().ram);
    }
    assert!(state.rom.is_empty());
    eprintln!("native properties graphics cases=16 Source instruction boundaries={instructions}");
}
