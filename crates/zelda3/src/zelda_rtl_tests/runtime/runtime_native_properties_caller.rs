use crate::cycle_models::{
    hud_update,
    intro_clear::Access,
    load_file_player,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    overworld_properties_caller::{self, OverworldPropertiesCaller},
};
use snes::CpuMasterTimeline;

#[test]
fn module08_properties_composition_consumes_real_near_and_far_frames() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let sprite_data = state.native_overworld_sprite_data().unwrap();
    for address in 0x0283b9..0x0283bf {
        assert_eq!(
            overworld_properties_caller::table_byte(address),
            Some(rom[0x10000 + (address as usize & 0x7fff)])
        );
    }
    let mut instructions = 0;
    let mut unowned_stores = std::collections::BTreeMap::new();
    let mut incoherent_states = std::collections::BTreeSet::new();
    for case in 0..24usize {
        let screen = [3u16, 24, 44, 0, 27, 43][case % 6] | if case >= 12 { 0x40 } else { 0 };
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        let r = &mut machine.ram;
        r[0x10] = 8;
        r[0x11] = 0;
        r[0xa0..0xa2].copy_from_slice(&0x100u16.to_le_bytes());
        r[0xc14c..0xc14e].copy_from_slice(&screen.to_le_bytes());
        r[0xc140..0xc142].copy_from_slice(&screen.to_le_bytes());
        r[0xc142..0xc144].copy_from_slice(&0x0200u16.to_le_bytes());
        r[0xc144..0xc146].copy_from_slice(&0x0300u16.to_le_bytes());
        r[0xc146..0xc148].copy_from_slice(&0x0100u16.to_le_bytes());
        r[0xc164..0xc168].copy_from_slice(&[
            0,
            if case & 1 == 0 { 0x20 } else { 0x21 },
            [0, 3, 5, 11][case / 2 % 4],
            [0, 1, 11, 32][case / 4 % 4],
        ]);
        r[0xaa4] = if case & 1 == 0 { 1 } else { 11 };
        r[0xf357] = u8::from(case % 2 == 0);
        r[0xf3ca] = (screen & 0x40) as u8;
        r[0xf3c5] = [0, 2, 3][case % 3];
        r[0xf340] = 1;
        r[0xf354] = (case % 3) as u8;
        r[0xf359] = (case % 4) as u8;
        r[0xf35a] = (case % 3) as u8;
        r[0xf35b] = (case % 2) as u8;
        r[0xf36c] = 24;
        r[0xf36d] = 24;
        r[0xf36f] = 0xff;
        r[0xab2] = (case % 4) as u8;
        r[0x202..0x204].fill(0);
        r[0xc2fc..0xc300].copy_from_slice(&[12, 82, 90, 95]);
        r[0x136] = 0; // Resident overworld bank: real no-upload tail and near return.
        r[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]); // Main JSL frame.
        let mut cpu = NativeNmiContext {
            a: 0xb701,
            x: 2,
            y: 8,
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x83bf,
            sp: 0x1fc,
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
        let mut published = state.clone();
        published.import_wram_image(&ram);
        published.sync_native_game_state_from_ram();
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
        let mut caller = OverworldPropertiesCaller::default();
        for _ in 0..1000000 {
            if caller.complete() {
                break;
            }
            assert!(
                caller.external_child().is_none(),
                "unowned composed child {:?}; case={case}",
                caller.external_child()
            );
            let label = format!(
                "Module08 properties case={case} screen={screen} pc={:02x}:{:04x} child={:?}",
                cpu.bank,
                cpu.pc,
                caller.active_child()
            );
            let child = caller.active_child();
            compare_instruction_with_devices(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                Some(&mut ppu),
                |_, access| match access {
                    Access::Write(a, value, width) => {
                        if let Err(offset) =
                            published.native_overworld_properties_store(child, a, value, width)
                        {
                            if !unowned_stores.contains_key(&offset) {
                                eprintln!("unowned properties address={offset:05x} child={child:?} value={value:04x} width={width}; {label}");
                            }
                            unowned_stores
                                .entry(offset)
                                .or_insert((child, value, width));
                        }
                        None
                    }
                    Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                        let read = |address| {
                            let address = address & 0x7fffff;
                            overworld_properties_caller::table_byte(address)
                                .or_else(|| state.native_overworld_exit_table_byte(address))
                                .or_else(|| hud_update::table_byte(address))
                                .or_else(|| {
                                    crate::ZeldaState::native_sprite_property_table_byte(address)
                                })
                                .or_else(|| state.native_graphics_asset_byte(address))
                                .or_else(|| {
                                    crate::zelda_rtl::load_gfx::native_animated_item_table_byte(
                                        address,
                                    )
                                })
                                .or_else(|| state.native_overworld_palette_byte(address))
                                .or_else(|| {
                                    crate::ZeldaState::native_overworld_palette_selection_byte(
                                        address,
                                    )
                                })
                                .or_else(|| load_file_player::data_byte(address))
                                .or_else(|| sprite_data.get(&address).copied())
                                .unwrap_or_else(|| {
                                    panic!("unowned composed data {address:06x}; {label}")
                                })
                        };
                        Some(if width == 1 {
                            u16::from(read(a))
                        } else {
                            u16::from_le_bytes([read(a), read(a + 1)])
                        })
                    }
                    Access::Read(0x2000, _) => panic!("invalid graphics pointer; {label}"),
                    _ => None,
                },
                |cpu, clock, bus| {
                    caller
                        .step(cpu, clock, bus)
                        .expect("native properties instruction");
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            assert_eq!(published.game_state.player.follower_link.y(), crate::types::read_le_u16(&ram, 0x20), "typed player Y; {label}");
            assert_eq!(published.game_state.frame.saved_module_for_menu, ram[0x10c], "typed saved module; {label}");
            assert_eq!(published.game_state.sprites.workspace.graphics_subset(1), ram[0xc2fd], "typed graphics subset; {label}");
            if *published.wram_image() != ram {
                let differences: Vec<_> = published
                    .wram_image()
                    .iter()
                    .zip(&ram)
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .take(16)
                    .map(|(address, (a, b))| (address, *a, *b))
                    .collect();
                panic!("typed properties store mirror; {label}; differences={differences:04x?}");
            }
            instructions += 1;
        }
        assert!(
            caller.complete(),
            "Module08 properties did not return in case{case}"
        );
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0x805a, 0x1ff));
        assert_eq!(ppu.vram, source.snes().ppu.vram);
        assert_eq!(ppu.cgram, source.snes().ppu.cgram);
        assert_properties_owners(&published, &ram, case);
        incoherent_states.extend(published.game_state.report_incoherent_with_ram(&ram));
        if case == 0 {
            let fresh = crate::game_state::GameState::load_from_ram(&ram);
            eprintln!("properties active leaves: follower_link={} sprite_slots={} sprite_system={} palette_buffer={} world_location={} world_region={} map16={} config={} screen_size={}",
                published.game_state.player.follower_link == fresh.player.follower_link,
                published.game_state.sprites.sprite_slots == fresh.sprites.sprite_slots,
                published.game_state.sprites.system == fresh.sprites.system,
                published.game_state.display.palette_buffer == fresh.display.palette_buffer,
                published.game_state.world.location == fresh.world.location,
                published.game_state.world.region == fresh.world.region,
                published.game_state.world.overworld.map16 == fresh.world.overworld.map16,
                published.game_state.world.overworld.config_table == fresh.world.overworld.config_table,
                published.game_state.world.overworld.screen_size == fresh.world.overworld.screen_size);
            let mut projected = ram.clone();
            published.game_state.write_to_ram(&mut projected);
            let differences: Vec<_> = projected.iter().zip(&ram).enumerate()
                .filter(|(_, (a, b))| a != b).take(128)
                .map(|(address, (a, b))| (address, *a, *b)).collect();
            eprintln!("properties typed projection differences={differences:04x?}");
            eprintln!("properties typed projection owners={:04x?}", published.game_state.report_scratch_conflicts(&ram));
        }
        eprintln!("properties case={case} complete; instruction boundaries={instructions}; unowned bytes={}; incoherent states={incoherent_states:?}", unowned_stores.len());
    }
    assert!(state.rom.is_empty());
    let mut ranges: Vec<(
        usize,
        usize,
        Option<crate::cycle_models::pre_overworld_properties::PropertiesChild>,
    )> = Vec::new();
    for (&address, &(child, _, _)) in &unowned_stores {
        if let Some(last) = ranges
            .last_mut()
            .filter(|last| last.1 + 1 == address && last.2 == child)
        {
            last.1 = address;
        } else {
            ranges.push((address, address, child));
        }
    }
    assert!(
        unowned_stores.is_empty(),
        "unowned typed properties ranges: {ranges:04x?}; incoherent states: {incoherent_states:?}"
    );
    // These inactive views borrow the graphics scratch/staging addresses.
    // The native caller never projects them or imports a whole state at return.
    // Verify its active owners directly above, and reject any new drift here.
    let inactive_views = ["dungeon.room_tilemaps", "dungeon.stair_lists",
        "dungeon_map_display.scroll_draw_offset", "dungeon_map_display.scroll_input",
        "ending", "intro_sword", "messaging", "player", "save_load_transfer",
        "sprites.workspace", "world.overworld"];
    let unexpected: Vec<_> = incoherent_states.iter()
        .filter(|name| !inactive_views.contains(name)).collect();
    assert!(unexpected.is_empty(), "unexpected properties typed state drift: {unexpected:?}");
    eprintln!("native Module08 properties whole composition cases=24 Source instruction boundaries={instructions}");
}

fn assert_properties_owners(published: &crate::ZeldaState, ram: &[u8], case: usize) {
    let expected = crate::game_state::GameState::load_from_ram(ram);
    let actual = &published.game_state;
    macro_rules! check {
        ($($field:ident).+) => {
            assert_eq!(actual.$($field).+, expected.$($field).+,
                "properties owner {}; case={case}", stringify!($($field).+));
        };
    }
    check!(frame);
    check!(system_signals);
    check!(minigame);
    check!(inventory);
    check!(display);
    check!(memorized_tiles);
    check!(player.follower_link);
    check!(player.special_exit_position);
    check!(player.swim_acceleration);
    check!(player.pushed_block);
    check!(player.bg1_movement_accumulator);
    check!(world.location);
    check!(world.region);
    check!(world.scroll);
    check!(world.camera_boundaries);
    check!(world.room_bounds);
    check!(world.palette_theme);
    check!(world.transient);
    check!(world.overworld.config_table);
    check!(world.overworld.map16);
    check!(world.overworld.screen_size);
    check!(world.overworld.scroll_delta);
    check!(sprites.system);
    check!(sprites.sprite_slots);
    check!(sprites.ancilla_slots);
    check!(sprites.follower_runtime);
    check!(sprites.garnish_runtime);
    check!(sprites.overworld_sprite_presence);
    check!(sprites.overworld_sprite_loaded);
    // These two owners also expose temporary zero-page operands. Compare
    // every projected byte except that explicitly shared scratch, rather than
    // accepting a difference anywhere in their otherwise active fields.
    macro_rules! check_projection {
        ($field:expr, $scratch:expr, $name:literal) => {{
            let mut projection = ram.to_vec();
            $field.write_to_ram(&mut projection);
            let differences: Vec<_> = projection.iter().zip(ram).enumerate()
                .filter(|(at, (a, b))| a != b && !$scratch.contains(at))
                .map(|(at, (a, b))| (at, *a, *b)).collect();
            assert!(differences.is_empty(), "properties owner {}; case={case}; differences={differences:04x?}", $name);
        }};
    }
    check_projection!(actual.player.tile_detection, 0..=0xf, "tile_detection");
    check_projection!(actual.sprites.workspace, 0..=3, "sprite workspace");
    let mut overworld = expected.world.overworld;
    // $C8 is not projected by this view; graphics owns it during this call.
    overworld.entrance.sequence_counter = actual.world.overworld.entrance.sequence_counter;
    assert_eq!(actual.world.overworld, overworld, "properties overworld owner; case={case}");
}
