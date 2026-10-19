use crate::cycle_models::{
    intro_clear::Access,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    overworld_area_palettes::OverworldAreaPalettes,
    pre_overworld_properties::{PreOverworldProperties, PropertiesChild},
};
use snes::CpuMasterTimeline;
#[test]
fn area_palette_descriptors_match_cartridge_data() {
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    for a in 0x0ed504..0x0ed5a8 {
        let o = (a >> 16) as usize * 0x8000 + (a as usize & 0x7fff);
        assert_eq!(
            crate::ZeldaState::native_overworld_palette_selection_byte(a),
            Some(rom[o]),
            "palette descriptor {a:06x}"
        );
    }
}
#[test]
fn properties_area_backdrop_and_both_cache_paths_keep_live_clock_and_buffers() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let mut instructions = 0;
    let mut copies = [0usize; 2];
    for case in 0..1240usize {
        let bg = (case / 40) as u8;
        let sprite = (case / 2 % 20) as u8;
        let module = if case & 1 == 0 { 8 } else { 16 };
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ case.wrapping_mul(31)) as u8;
        }
        let r = &mut machine.ram;
        r[0] = sprite;
        r[0x10] = module;
        r[0x8a..0x8c]
            .copy_from_slice(&[0u16, 0x40, 0x7f, 0x80, 0x88, 0xff][case % 6].to_le_bytes());
        r[0xa0..0xa2]
            .copy_from_slice(&[0u16, 0x180, 0x182, 0x183, 0x1ff][case / 6 % 5].to_le_bytes());
        r[0xaa8..0xaaa].fill(0);
        r[0xab4] = 0;
        r[0xab5] = 1;
        r[0xab8] = 7;
        r[0xaad] = 3;
        r[0xaae] = 4;
        r[0x15] = case as u8;
        r[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        let mut cpu = NativeNmiContext {
            a: 0xb700 | u16::from(bg),
            x: 2,
            y: 7,
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x84a5,
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
        let mut caller = PreOverworldProperties::at_instruction_for_test(0x84a5);
        let mut area = OverworldAreaPalettes::from_entry(0x0ed5a8);
        let mut backdrop = OverworldAreaPalettes::from_entry(0x0ed618);
        let mut cache =
            OverworldAreaPalettes::from_entry(if module == 8 { 0x02c65f } else { 0x02c6eb });
        let mut done = false;
        for _ in 0..30000 {
            let child = caller.child();
            if child == Some(PropertiesChild::FixedColorAndScroll) {
                done = true;
                break;
            }
            let label = format!(
                "area palettes bg={bg} sprite={sprite} module={module} pc={:02x}:{:04x}",
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
                            crate::ZeldaState::native_overworld_palette_selection_byte(address)
                                .or_else(|| state.native_overworld_palette_byte(address))
                                .unwrap_or_else(|| {
                                    panic!("unowned area palette data {address:06x}; {label}")
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
                    Some(PropertiesChild::OverworldPalettes) => {
                        area.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::BackgroundColor) => {
                        backdrop.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::PaletteCache) => {
                        cache.step(cpu, clock, bus).unwrap();
                    }
                    _ => panic!("unexpected area child {child:?}"),
                },
            );
            let returned = match child {
                Some(PropertiesChild::OverworldPalettes) if area.complete() => Some(0x84a9),
                Some(PropertiesChild::BackgroundColor) if backdrop.complete() => Some(0x84ad),
                Some(PropertiesChild::PaletteCache) if cache.complete() => {
                    Some(if module == 8 { 0x84b6 } else { 0x84bb })
                }
                _ => None,
            };
            if let Some(pc) = returned {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, pc, 0x1fa));
                caller.return_from_child(&cpu);
            }
            assert_eq!(ram, source.snes().ram, "{label}");
            instructions += 1;
        }
        assert!(done && area.complete() && backdrop.complete() && cache.complete());
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (11, 0xfe70, 0x1f7));
        copies[usize::from(module != 8)] += 1;
    }
    assert_eq!(copies, [620, 620]);
    assert!(state.rom.is_empty());
    eprintln!("native area/backdrop/cache cases=1240 Source instruction boundaries={instructions}");
}
