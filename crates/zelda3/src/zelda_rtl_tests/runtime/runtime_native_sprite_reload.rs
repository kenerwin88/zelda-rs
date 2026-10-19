use crate::cycle_models::{
    intro_clear::Access,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    overworld_sprite_reload::OverworldSpriteReload,
    pre_overworld_properties::{PreOverworldProperties, PropertiesChild},
};
use snes::CpuMasterTimeline;

#[test]
fn overworld_sprite_reload_keeps_properties_stack_and_live_presence_scan() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let data = state.native_overworld_sprite_data().unwrap();
    for (&address, &value) in &data {
        let offset = ((address as usize >> 16) & 0x7f) * 0x8000 + (address as usize & 0x7fff);
        assert_eq!(value, rom[offset], "sprite data {address:06x}");
    }
    let mut instructions = 0;
    let mut spawned = 0;
    let mut overlords = 0;
    for case in 0..432usize {
        let screen = case % 144;
        let progress = [0, 2, 3][case / 144];
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, byte) in machine.ram.iter_mut().enumerate() {
            *byte = (i ^ (i >> 8) ^ case.wrapping_mul(37)) as u8;
        }
        let ram = &mut machine.ram;
        ram[0x040a..0x040c].copy_from_slice(&(screen as u16).to_le_bytes());
        ram[0xf3c5] = progress;
        ram[0xf3cc] = [0, 5, 13][case % 3];
        let mut x = ((screen & 7) as u16 * 512).wrapping_add([0, 240, 496, 960][case % 4]);
        let mut y = (((screen & 0x3f) >> 2 & 0x0e) as u16 * 256)
            .wrapping_add([0, 256, 512, 768][case / 4 % 4]);
        // The generic camera matrix missed every original overlord record.
        // Center their source coordinates in the scan so their allocation,
        // byte stack transfer and coordinate publications are compared too.
        let table = [0xc881, 0xc901, 0xca21][case / 144];
        let pointer_offset = 9 * 0x8000 + (table & 0x7fff) + screen * 2;
        let pointer = u16::from_le_bytes([rom[pointer_offset], rom[pointer_offset + 1]]);
        let mut record = 9 * 0x8000 + usize::from(pointer & 0x7fff);
        while rom[record] != 0xff {
            if rom[record + 2] == 0xf3 {
                let encoded_y = u16::from(rom[record]);
                let encoded_x = u16::from(rom[record + 1]);
                x = ((screen & 7) as u16 * 512)
                    .wrapping_add(((encoded_x >> 4) << 8) | ((encoded_x & 15) << 4))
                    .wrapping_sub(96);
                y = (((screen & 0x3f) >> 2 & 0x0e) as u16 * 256)
                    .wrapping_add(((encoded_y >> 4) << 8) | ((encoded_y & 15) << 4))
                    .wrapping_sub(96);
                break;
            }
            record += 3;
        }
        ram[0xe2..0xe4].copy_from_slice(&x.to_le_bytes());
        ram[0xe8..0xea].copy_from_slice(&y.to_le_bytes());
        ram[0x069f] = [0, 16, 240, 255][case % 4];
        ram[0x0dd0..0x0de0].fill(8);
        ram[0xc4a..0xc54].fill(0);
        ram[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        let mut cpu = NativeNmiContext {
            a: 0xb701,
            x: 2,
            y: 0x60,
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x84da,
            sp: 0x1fa,
            p: 0x30 | (case as u8 & 0xc3),
        };
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.db = cpu.db;
        machine.cpu.k = cpu.bank;
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
        let mut caller = PreOverworldProperties::at_instruction_for_test(0x84da);
        let mut reload = OverworldSpriteReload::default();
        for _ in 0..100000 {
            let child = caller.child();
            let label = format!(
                "sprite reload case={case} pc={:02x}:{:04x}",
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
                        let address = a & 0x7fffff;
                        let read = |address| {
                            *data.get(&address).unwrap_or_else(|| {
                                panic!("unowned sprite reload data {address:06x}; {label}")
                            })
                        };
                        Some(if width == 1 {
                            u16::from(read(address))
                        } else {
                            u16::from_le_bytes([read(address), read(address + 1)])
                        })
                    }
                    _ => None,
                },
                |cpu, clock, bus| match child {
                    None => {
                        caller.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::SpriteReload) => {
                        reload.step(cpu, clock, bus).unwrap();
                    }
                    _ => panic!("unexpected reload child {child:?}"),
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            instructions += 1;
            if reload.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x84de, 0x1fa));
                caller.return_from_child(&cpu);
                break;
            }
        }
        assert!(
            reload.complete(),
            "sprite reload did not return in case{case}"
        );
        spawned += ram[0xdd0..0xde0].iter().filter(|&&v| v != 0).count();
        overlords += ram[0xb00..0xb08].iter().filter(|&&v| v != 0).count();
    }
    assert!(state.rom.is_empty());
    assert!(
        spawned > 0 && overlords > 0,
        "spawn coverage {spawned}/{overlords}"
    );
    eprintln!("native overworld sprite reload cases=432 Source instruction boundaries={instructions} normal_spawns={spawned} overlord_spawns={overlords} data_bytes={}", data.len());
}
