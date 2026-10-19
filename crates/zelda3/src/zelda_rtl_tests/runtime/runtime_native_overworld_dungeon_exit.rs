use super::*;
use crate::cycle_models::overworld_dungeon_exit::OverworldDungeonExit;
use snes::CpuMasterTimeline;

use crate::cycle_models::{
    hud_update::HudUpdate,
    intro_clear::Access,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    overworld_song_list::OverworldSongList,
    pre_overworld_properties::{BunnyStatus, PreOverworldProperties, PropertiesChild},
};
pub(super) fn data_owner() -> crate::zelda_rtl::ZeldaState {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = std::env::var_os("ZELDA3_ASSETS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("zelda3_assets.dat"));
    let mut state = crate::zelda_rtl::ZeldaState::new();
    state
        .set_assets(&std::fs::read(path).expect("Source exit tests require real assets"))
        .unwrap();
    state.rom.clear();
    state
}
pub(super) fn rom() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"))
        .unwrap()
}
#[test]
fn exit_data_bindings_match_pristine_rom_without_native_rom() {
    let state = data_owner();
    let rom = rom();
    let mut bytes = 0;
    for (base, length) in [
        (0x02dd8a, 158),
        (0x02de28, 79),
        (0x02de77, 158),
        (0x02df15, 158),
        (0x02dfb3, 158),
        (0x02e051, 158),
        (0x02e0ef, 158),
        (0x02e18d, 158),
        (0x02e22b, 158),
        (0x02e2c9, 79),
        (0x02e318, 79),
        (0x02e367, 158),
        (0x02e405, 158),
        (0x00fc9c, 128),
        (0x00fd1c, 136),
        (0x02a9c4, 256),
        (0x02c303, 256),
        (0x02c403, 96),
        (0x02a844, 64),
        (0x02a884, 64),
        (0x02a8c4, 256),
        (0x02bee2, 272),
        (0x00d8f4, 1),
        (0x00d8fc, 1),
    ] {
        for address in base..base + length {
            let offset = ((address >> 16) as usize & 127) * 0x8000 + (address as usize & 0x7fff);
            assert_eq!(
                state.native_overworld_exit_table_byte(address),
                Some(rom[offset]),
                "exit data {address:06x}"
            );
            assert_eq!(
                state.native_overworld_exit_table_byte(address | 0x800000),
                Some(rom[offset])
            );
            bytes += 1;
        }
    }
    assert!(state.rom.is_empty());
    eprintln!("native exit data bytes checked={bytes}");
}
#[test]
fn properties_dungeon_exit_chain_matches_source_for_all_exit_rooms_and_cached_branches() {
    let state = data_owner();
    let rom = rom();
    let table = state.asset_raw(131).unwrap();
    let mut cases = Vec::new();
    for bytes in table.chunks_exact(2) {
        cases.push((u16::from_le_bytes([bytes[0], bytes[1]]), 0, 0));
    }
    for room in 0x100..0x180 {
        if room == 0x104 {
            continue;
        }
        for screen in [
            0, 3, 5, 0x3f, 0x40, 0x47, 0x7f, 0x80, 0x81, 0x88, 0xbf, 0xff,
        ] {
            for door in [0, 0xffff] {
                cases.push((room, screen, door));
            }
        }
    }
    let mut instructions = 0;
    // Exhaust screen-byte data reads across both cached Y-adjustment branches
    // and both door directions, including the special-screen read windows.
    for room in [0x100, 0x123, 0x124, 0x17f] {
        for screen in 0..=255 {
            for door in [0, 0xffff] {
                cases.push((room, screen, door));
            }
        }
    }
    let mut completed = 0;
    for (case, (room, screen, door)) in cases.into_iter().enumerate() {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i.wrapping_mul(13) ^ (i >> 8) ^ case.wrapping_mul(37)) as u8;
        }
        machine.ram[0x10] = 8;
        machine.ram[0xf357] = [0, 1, 255][case % 3];
        machine.ram[0xa0..0xa2].copy_from_slice(&room.to_le_bytes());
        machine.ram[0xc14c..0xc14e].copy_from_slice(&(screen as u16).to_le_bytes());
        machine.ram[0x696..0x698].copy_from_slice(&(door as u16).to_le_bytes());
        machine.ram[0xc148..0xc14a]
            .copy_from_slice(&[0u16, 0x10, 0x20, 65535][case % 4].to_le_bytes());
        machine.ram[0x713] = 0;
        let r = &mut machine.ram;
        r[0x200] = u8::from(case % 32 == 0);
        r[0x202..0x204].copy_from_slice(&((case % 21) as u16).to_le_bytes());
        r[0xf340..0xf354].fill(1);
        r[0xf34f] = 1;
        r[0xf35c] = 1;
        r[0xf340] = (case % 5) as u8;
        r[0xf359] = (case / 5 % 4) as u8;
        r[0xf3c5] = [0, 1, 2, 3, 255][case % 5];
        r[0xf373] = [0, 1, 127, 128][case % 4];
        r[0xf36e] = [0, 1, 126, 127, 128, 129, 254, 255][case / 4 % 8];
        let (current, target): (u16, u16) = [
            (0, 0),
            (0, 1),
            (1, 0),
            (10, 9),
            (998, 999),
            (999, 1000),
            (1000, 999),
            (0, 65535),
        ][case / 32 % 8];
        r[0xf362..0xf364].copy_from_slice(&current.to_le_bytes());
        r[0xf360..0xf362].copy_from_slice(&target.to_le_bytes());
        r[0x1a] = (case / 8) as u8;
        r[0x12e] = if case & 8 != 0 { 5 } else { 0 };
        r[0x12f] = if case & 16 != 0 { 6 } else { 0 };
        r[0xcfd] = (case / 16 % 16) as u8;
        r[0xf375] = (case % 3) as u8;
        r[0xf370] = (case / 8 % 8) as u8;
        let bomb_max = [10, 15, 20, 25, 30, 35, 40, 50][case / 8 % 8];
        r[0xf343] = bomb_max - u8::from(case & 32 == 0);
        r[0xf376] = (case / 3 % 3) as u8;
        r[0xf371] = (case / 16 % 8) as u8;
        let arrow_max = [30, 35, 40, 45, 50, 55, 60, 70][case / 16 % 8];
        r[0xf377] = arrow_max - u8::from(case & 64 == 0);
        let capacity = [8, 24, 72, 80, 88, 152, 160, 160][case / 128 % 8];
        r[0xf36c] = capacity;
        r[0xf36d] = [0, 1, 8, 9, 16, capacity - 1, capacity, capacity + 1][case / 256 % 8];
        r[0xf372] = [0, 8, 16, 160][case / 512 % 4];
        r[0x4ca] = [0, 1, 31, 255][case / 64 % 4];
        r[0x2e4] = u8::from(case % 7 == 0);
        r[0x20a] = u8::from(case & 512 != 0);
        r[0x208] = [0, 1, 2, 7][case / 1024 % 4];
        r[0x209] = (case / 256 % 4) as u8;
        machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        let mut cpu = NativeNmiContext {
            a: 0xb700,
            x: 0xe0,
            y: 8,
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x83c7,
            sp: 0x1fa,
            p: 0x30 | [0, 1, 0x43, 0xc3][case % 4],
        };
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.dp = 0;
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
                snes::CpuRasterPosition::new(224, 1300),
                snes::CpuRasterPosition::new(261, 1340),
            ][case % 3],
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            snes::RomCpuTimingProbe::new(machine, clock, snes::SourcePpuReadState::snes9x_reset())
                .unwrap();
        let mut clock = source.timeline().clone();
        let mut properties = PreOverworldProperties::new(0x83c6, 0x1fc);
        let mut bunny = BunnyStatus::default();
        let mut exit = OverworldDungeonExit::default();
        let mut songs = OverworldSongList::default();
        let mut hud = HudUpdate::default();
        let mut done = false;
        for _ in 0..20000 {
            let which = properties.child();
            if which == Some(PropertiesChild::AnimatedTiles) {
                done = true;
                break;
            }
            let label = format!(
                "properties exit case={case} room={room:04x} pc={:04x}",
                cpu.pc
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
                                .native_overworld_exit_table_byte(address)
                                .or_else(|| crate::cycle_models::hud_update::table_byte(address))
                                .unwrap_or_else(|| panic!("unowned exit asset {address:06x}; {label}"))
                        };
                        Some(if width == 1 {
                            u16::from(read(a))
                        } else {
                            u16::from_le_bytes([read(a), read(a + 1)])
                        })
                    }
                    _ => None,
                },
                |cpu, clock, bus| match which {
                    None => {
                        properties.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::BunnyStatus) => {
                        bunny.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::DungeonExit) => {
                        exit.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::SongList) => {
                        songs.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::HudRefill) => {
                        hud.step(cpu, clock, bus).unwrap();
                    }
                    _ => panic!("unexpected properties child {which:?}"),
                },
            );
            if which == Some(PropertiesChild::BunnyStatus) && bunny.complete() {
                properties.return_from_child(&cpu);
            }
            if which == Some(PropertiesChild::DungeonExit) && exit.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x83db, 0x1fa));
                properties.return_from_child(&cpu);
            }
            if which == Some(PropertiesChild::SongList) && songs.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x83e4, 0x1fa));
                properties.return_from_child(&cpu);
            }
            if which == Some(PropertiesChild::HudRefill) && hud.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x83ee, 0x1fa));
                properties.return_from_child(&cpu);
            }
            assert_eq!(ram, source.snes().ram, "{label}");
            instructions += 1;
        }
        assert!(
            done && exit.complete() && songs.complete() && hud.complete(),
            "exit did not return room={room:04x}"
        );
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0xd394, 0x1f7));
        completed += 1;
    }
    assert_eq!(completed, 79 + 127 * 12 * 2 + 4 * 256 * 2);
    assert!(state.rom.is_empty());
    eprintln!(
        "native composed exit cases={completed} Source instruction boundaries={instructions}"
    );
}
