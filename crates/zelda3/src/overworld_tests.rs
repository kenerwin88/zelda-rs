use super::*;

#[test]
fn marker44_overworld_transition_body_matches_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    // Interior, both edges on both axes, mismatched direction, eligibility
    // rejection, music/pearl paths and the two final palette/mosaic returns.
    for (x, y, vx, vy, direction) in [
        (0x100u16, 0x100u16, 0, 0, 0),
        (0x100, 0x100, 8, 8, 15),
        (0, 0x100, 8, 0, 2), (0x204, 0x100, 8, 0, 1),
        (0x100, 0, 0, 8, 8), (0x100, 0x200, 0, 8, 4),
        (0, 0x100, 8, 0, 1), (0x100, 0, 0, 8, 4),
    ] {
        for (screen, dark, pearl, immobilized) in [
            (0u8, 0, 0, 0), (3, 0, 0, 0), (0x2a, 0, 0, 0),
            (3, 0x40, 0, 0), (3, 0x40, 1, 0), (3, 0, 0, 1),
        ] {
            let mut n = ZeldaState::new();
            n.set_assets(&assets).unwrap();
            for (address, value) in [(0x10, 9), (0x8a, screen), (0x30, vy), (0x31, vx),
                (0x67, direction), (0xf3ca, dark), (0xf357, pearl), (0x2e4, immobilized)] {
                n.write_wram_byte(if address >= 0xf000 { address+0x10000 } else { address }, value);
            }
            // A central area keeps every adjacent table index inside the
            // physical world map; a west exit from area zero wraps into code.
            let table_word = |address: usize| {
                let offset = 2*0x8000 + (address & 0x7fff) + 0x22;
                u16::from_le_bytes([rom[offset], rom[offset+1]])
            };
            n.write_wram_word(0x20, table_word(0xa8c4).wrapping_add(y));
            n.write_wram_word(0x22, table_word(0xa944).wrapping_add(x));
            n.write_wram_word(0x700, 0x22);
            n.write_wram_word(0x716, 0x200);
            n.write_wram_word(0x84, 0x882);
            n.sync_native_game_state_from_ram();
            n.set_lag_free(true);
            let mut s = snes::Snes::new();
            s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            s.ram.copy_from_slice(&n.wram_image());
            s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
            s.cpu.e = false;
            s.cpu.unpack_flags(0x30);
            s.cpu.k = 2;
            s.cpu.db = 2;
            s.cpu.pc = 0xa9c4;
            s.cpu.sp = 0x1fa;
            crate::cycle_ledger::assert_rom_marker_work(0x02a9c4, s,
                |s| s.cpu.k == 2 && (0xa9c4..0xab08).contains(&s.cpu.pc), || {
                    n.OverworldHandleTransitions();
                });
        }
    }
}

fn marker44_map_edge_body_matches_rom(entry: u32, end: u32) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    // Rejected edges, zero/one/multiple normalization passes, both map sizes,
    // and two destination offsets. The original stripe children run unchanged.
    for src in [0u16, 2, 0x5e, 0x60, 0x7e, 0x80, 0x882, 0x17fe, 0x1800] {
        for screen in [0u8, 2] {
            for dst in [0usize, 6] {
                let mut n = ZeldaState::new();
                n.set_assets(&assets).unwrap();
                n.write_wram_byte(0x8a, screen);
                n.write_wram_word(0x84, src);
                n.write_wram_word(0x86, 0x1f);
                n.write_wram_word(0x88, 0);
                n.sync_native_game_state_from_ram();
                n.set_lag_free(true);
                let mut s = snes::Snes::new();
                s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                s.ram.copy_from_slice(&n.wram_image());
                s.ram[0x0e..0x10].copy_from_slice(&((dst * 2) as u16).to_le_bytes());
                s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
                s.cpu.e = false;
                s.cpu.unpack_flags(0x30);
                s.cpu.k = 2;
                s.cpu.db = 2;
                s.cpu.pc = entry as u16;
                s.cpu.sp = 0x1fa;
                let source = crate::cycle_ledger::assert_rom_marker_work(entry, s,
                    |s| s.cpu.k == 2 && (entry as u16..end as u16).contains(&s.cpu.pc), || {
                        match entry {
                            0x02f2dd => { n.check_for_newly_loaded_map_areas_north(dst); }
                            0x02f311 => { n.check_for_newly_loaded_map_areas_south(dst); }
                            0x02f345 => { n.check_for_newly_loaded_map_areas_west(dst); }
                            0x02f37f => { n.check_for_newly_loaded_map_areas_east(dst); }
                            _ => unreachable!(),
                        }
                    });
                for address in [0x84, 0x86, 0x88] {
                    assert_eq!(n.read_wram_image_word(address, 2),
                        u16::from_le_bytes([source.snes().ram[address], source.snes().ram[address+1]]),
                        "entry={entry:06x} src={src:04x} screen={screen} dst={dst} address={address:04x}");
                }
            }
        }
    }
}

#[test]
fn marker44_north_map_edge_matches_rom() { marker44_map_edge_body_matches_rom(0x02f2dd, 0x02f311); }
#[test]
fn marker44_south_map_edge_matches_rom() { marker44_map_edge_body_matches_rom(0x02f311, 0x02f345); }
#[test]
fn marker44_west_map_edge_matches_rom() { marker44_map_edge_body_matches_rom(0x02f345, 0x02f37f); }
#[test]
fn marker44_east_map_edge_matches_rom() { marker44_map_edge_body_matches_rom(0x02f37f, 0x02f3b9); }

#[test]
fn load5_conditional_mosaic_markers_match_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    for countdown in [0u8, 1, 2, 255] {
        let mut n = ZeldaState::new();
        n.write_wram_byte(0xc007, countdown);
        n.sync_native_game_state_from_ram();
        n.set_lag_free(true);
        let mut s = snes::Snes::new();
        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
        s.cpu.e = false;
        s.cpu.unpack_flags(0x30);
        s.cpu.k = 2;
        s.cpu.db = 2;
        s.cpu.pc = 0xc2e4;
        s.cpu.sp = 0x1fa;
        crate::cycle_ledger::assert_rom_marker_work(0x02c2e4, s,
            |s| s.cpu.k == 2 && (0xc2e4..0xc303).contains(&s.cpu.pc), || {
                n.ConditionalMosaicControl();
            });
    }
}

#[test]
fn load5_mosaic_start_dispatch_and_tails_match_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    for (phase, screen, area, module) in [
        (0, 1, 1, 9), (0, 0x80, 0, 9), (1, 1, 1, 9),
        (2, 1, 1, 9), (2, 0, 0, 9), (2, 1, 0, 9),
        (2, 1, 1, 11),
    ] {
        let mut n = ZeldaState::new();
        n.set_assets(&assets).unwrap();
        n.write_wram_byte(0x10, module);
        n.write_wram_byte(0x11, if module == 11 { 0x24 } else { 0x0d });
        n.write_wram_byte(0xb0, phase);
        n.write_wram_byte(0x8a, screen);
        n.write_wram_word(0x40a, area);
        n.write_wram_word(0xc007, 2);
        n.sync_native_game_state_from_ram();
        n.set_lag_free(true);
        let mut s = snes::Snes::new();
        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
        s.cpu.e = false;
        s.cpu.unpack_flags(0x30);
        s.cpu.k = 2;
        s.cpu.db = 2;
        s.cpu.pc = 0xae5e;
        s.cpu.sp = 0x1fa;
        crate::cycle_ledger::assert_rom_marker_work(0x02ae5e, s,
            |s| {
                let pc = (u32::from(s.cpu.k) << 16) | u32::from(s.cpu.pc);
                (0x02ae5e..0x02ae67).contains(&pc)
                    || (0x02ae6d..0x02aece).contains(&pc)
                    || (0x028ca9..0x028cac).contains(&pc)
                    || (0x02a2a0..0x02a2a5).contains(&pc)
                    || (0x008784..0x00879c).contains(&pc)
            }, || { n.overworld_start_mosaic_transition(); });
    }
}

#[test]
fn load5_mirror_dispatch_and_selected_tails_match_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    for (phase, screen) in [(0, 0x80), (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (4, 0x40)] {
        let mut n = ZeldaState::new();
        n.set_assets(&assets).unwrap();
        n.write_wram_byte(0x10, 9);
        n.write_wram_byte(0x11, 0x23);
        n.write_wram_byte(0xb0, phase);
        n.write_wram_byte(0x8a, screen);
        n.write_wram_byte(0x1a, 1);
        n.write_wram_byte(0x6bb, 1);
        n.write_wram_word(0xc009, 0xff);
        n.write_wram_word(0xc007, 65);
        n.sync_native_game_state_from_ram();
        n.set_lag_free(true);
        let mut s = snes::Snes::new();
        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.ram[0x1fa..0x1fd].copy_from_slice(&[0xff, 0x7f, 0]);
        s.cpu.e = false;
        s.cpu.unpack_flags(0x30);
        s.cpu.k = 2;
        s.cpu.db = 2;
        s.cpu.pc = 0xb1ff;
        s.cpu.sp = 0x1f9;
        crate::cycle_ledger::assert_rom_marker_work(0x02b1ff, s,
            |s| {
                let pc = (u32::from(s.cpu.k) << 16) | u32::from(s.cpu.pc);
                (0x02b1ff..0x02b208).contains(&pc)
                    || (0x02b217..0x02b2d4).contains(&pc)
                    || (0x00879f..0x0087c0).contains(&pc)
                    || (0x00fe5e..0x00fe64).contains(&pc)
            }, || { n.module09_mirror_warp(); });
    }
}

// CPU-only ROM execution; count bus/internal transactions, excluding raster
// refresh and interrupts. The range selects caller self instructions only.
fn load_price_rom_work(entry: u32, image: &[u8], y: u16, long_return: bool,
    self_range: std::ops::RangeInclusive<u32>) -> (u64, u64) {
    load_price_rom_work_on_bus(entry, image, y, long_return, self_range, snes::Snes::new())
}

fn load_price_rom_work_on_bus(entry: u32, image: &[u8], y: u16, long_return: bool,
    self_range: std::ops::RangeInclusive<u32>, mut machine: snes::Snes) -> (u64, u64) {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, SourcePpuReadState};
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    machine.cart.load(CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(image);
    machine.ram[0x1fc..0x200].copy_from_slice(&[0xff, 0x7f, 0, 0]);
    machine.ppu.forced_blank = true;
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0x30);
    machine.cpu.k = (entry >> 16) as u8;
    machine.cpu.db = (entry >> 16) as u8;
    machine.cpu.y = y;
    machine.cpu.pc = entry as u16;
    machine.cpu.sp = 0x1fb;
    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
    let apu = machine.source_oam_port().map(|_| {
        use snes::apu::{ApuHostPortProbe, ApuHostPortTiming};
        ApuHostPortTiming::new(ApuHostPortProbe::from_snes9x_coroutine(machine.apu.clone())
            .unwrap(), Default::default()).unwrap()
    });
    let mut source = RomCpuTimingProbe::new(machine, clock,
        SourcePpuReadState::snes9x_reset()).unwrap();
    if let Some(apu) = apu { source.attach_apu_port_owner(apu).unwrap(); }
    let (mut total, mut own) = (0, 0);
    for _ in 0..1_000_000 {
        let cpu = &source.snes().cpu;
        let pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
        let step = source.step().unwrap_or_else(|error|
            panic!("entry={entry:06x} pc={pc:06x}: {error:?}"));
        let work = step.transactions.iter()
            .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
        total += work;
        if self_range.contains(&pc) { own += work; }
        if source.snes().cpu.pc == 0x8000 { break; }
    }
    assert_eq!(source.snes().cpu.pc, 0x8000, "entry={entry:06x}");
    assert_eq!(source.snes().cpu.sp, if long_return { 0x1fe } else { 0x1fd });
    (total, own)
}

#[test]
fn cardinal_selection_self_prices_match_executed_rom() {
    use crate::game_state::CollisionAxis;
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for (entry, end, indoor_tail, outdoor_tail, axis, velocity_address, position_address) in [
        (0x07_ba0a, 0x07_ba34, 0xba35u16, 0xbeafu16, CollisionAxis::Vertical, 0x30, 0x20),
        (0x07_c4d4, 0x07_c4fe, 0xc4ffu16, 0xc8e9u16, CollisionAxis::Horizontal, 0x31, 0x22),
    ] {
        for indoors in [0u8, 1] {
            for doorway in [0u8, 1, 2] {
                for velocity in [0u8, 1, 0x7f, 0x80, 0xff] {
                    for position in [0u8, 0x7f, 0x80, 0xff] {
                        let mut native = base.clone();
                        let mut image = native.wram_image().into_owned();
                        image[0x1b] = indoors;
                        image[0x6c] = doorway;
                        image[velocity_address] = velocity;
                        image[position_address] = position;
                        image[0xec..0xee].copy_from_slice(&0x01ffu16.to_le_bytes());
                        native.import_wram_image(&image);
                        native.sync_native_game_state_from_ram();
                        let mut machine = snes::Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        machine.ram.copy_from_slice(&image);
                        machine.ram[0x1fc..0x200].copy_from_slice(&[0xff, 0x7f, 0, 0]);
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(0x30);
                        machine.cpu.k = 7;
                        machine.cpu.db = 7;
                        machine.cpu.pc = entry as u16;
                        machine.cpu.sp = 0x1fb;
                        let clock = CpuMasterTimeline::at_raster(0,
                            CpuRasterPosition::new(0, 0), CpuBusWorkload::default(),
                            CpuFieldTiming::non_interlace(false));
                        let mut source = RomCpuTimingProbe::new(machine, clock,
                            SourcePpuReadState::snes9x_reset()).unwrap();
                        let mut own = 0;
                        let mut completed = false;
                        for _ in 0..100_000 {
                            let cpu = &source.snes().cpu;
                            if cpu.k == 7 && cpu.pc == 0x8000 {
                                completed = true;
                                break;
                            }
                            let pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
                            let step = source.step().unwrap();
                            let response_end = if matches!(axis, CollisionAxis::Vertical) { 0xbaf3 } else { 0xc59d };
                            let outdoor_end = if matches!(axis, CollisionAxis::Vertical) { 0xbef7 } else { 0xc92e };
                            if (entry..=end).contains(&pc)
                                || ((0x07_0000 | u32::from(indoor_tail))..=(0x07_0000 | response_end)).contains(&pc)
                                || ((0x07_0000 | u32::from(outdoor_tail))..=(0x07_0000 | outdoor_end)).contains(&pc) {
                                own += step.transactions.iter()
                                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                            }
                        }
                        assert!(completed);
                        for lagless in [false, true] {
                            let mut state = native.clone();
                            state.set_lag_free(lagless);
                            let actual = crate::cycle_ledger::measure_routine_self_work(entry,
                                || state.start_movement_collision_checks(axis));
                            assert_eq!(actual, if lagless { own } else { 0 },
                                "entry={entry:06x} indoors={indoors} doorway={doorway} velocity={velocity:02x} position={position:02x} lagless={lagless}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn player_probe_and_acceleration_self_prices_match_executed_rom() {
    use crate::game_state::{CollisionAxis, CollisionDirection, MovementProbeKind};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(assets) = std::fs::read(root.join("zelda3_assets.dat")) else { return; };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for (entry, end, axis, kind) in [
        (0x07_cdcb, 0x07_ce29, CollisionAxis::Vertical, MovementProbeKind::Cardinal),
        (0x07_ce2a, 0x07_ce84, CollisionAxis::Horizontal, MovementProbeKind::Cardinal),
        (0x07_ce85, 0x07_cec8, CollisionAxis::Vertical, MovementProbeKind::Slope),
        (0x07_9873, 0x07_9895, CollisionAxis::Vertical, MovementProbeKind::Cardinal),
    ] {
        for direction in 0..4 {
            for floor in [0u8, 1] {
                for mask in [0x01ffu16, 0x01f8] {
                    for position in [0u16, 0x007f, 0x01ff, 0xffff] {
                        let mut native = base.clone();
                        let mut image = native.wram_image().into_owned();
                        image[0x1b] = 1;
                        image[0xee] = floor;
                        image[0x20..0x22].copy_from_slice(&position.to_le_bytes());
                        image[0x22..0x24].copy_from_slice(&position.rotate_left(3).to_le_bytes());
                        image[0xec..0xee].copy_from_slice(&mask.to_le_bytes());
                        image[0x326..0x340].fill(0xa5);
                        native.import_wram_image(&image);
                        native.sync_native_game_state_from_ram();
                        let (_, source_self) = load_price_rom_work(entry, &image,
                            direction * 2, false, entry..=end);
                        for lagless in [false, true] {
                            let mut state = native.clone();
                            state.set_lag_free(lagless);
                            let own = crate::cycle_ledger::measure_routine_self_work(entry, || {
                                if entry == 0x07_9873 { state.reset_all_acceleration(); }
                                else { state.detect_player_movement(axis,
                                    CollisionDirection::from_legacy(direction as u8), kind); }
                            });
                            assert_eq!(own, if lagless { source_self } else { 0 },
                                "entry={entry:06x} dir={direction} floor={floor} mask={mask:04x} pos={position:04x} lagless={lagless}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn camera_prices_match_executed_rom_paths() {
    let mut cases = Vec::new();
    for overlay in [0u8, 0x95, 0x97, 0x9c, 0x9d, 0x9e, 0xb5, 0xbe] {
        for velocity in [0u8, 1, 4, 0xff, 0xfc] {
            for position in [0u16, 0x100, 0x2000] {
                for equal_bound in [false, true] {
                    let mut image = ZeldaState::new().wram_image().into_owned();
                    image[0x8c] = overlay;
                    image[0x30] = velocity;
                    image[0x31] = velocity;
                    for address in [0x20, 0x22] {
                        image[address..address+2].copy_from_slice(&position.to_le_bytes());
                    }
                    for address in [0xe2, 0xe8] {
                        image[address..address+2].copy_from_slice(&0x80u16.to_le_bytes());
                    }
                    for address in (0x600..0x610).step_by(2) {
                        let bound = if equal_bound { 0x80u16 } else { 0x4000 };
                        image[address..address+2].copy_from_slice(&bound.to_le_bytes());
                    }
                    for address in (0x618..0x620).step_by(2) {
                        image[address..address+2].copy_from_slice(&0x100u16.to_le_bytes());
                    }
                    for address in (0x624..0x62c).step_by(2) { image[address] = 15; }
                    cases.push(image);
                }
            }
        }
    }
    for screen in [0u8, 0x1b, 0x47] {
        for height in [0u16, 0xffff, 8] {
            for scroll in [0x5ffu16, 0x600, 0x640, 0x6c0, 0x6c1] {
                let mut image = ZeldaState::new().wram_image().into_owned();
                image[0x8a] = screen;
                image[0x78] = 1;
                image[0x24..0x26].copy_from_slice(&height.to_le_bytes());
                image[0x30] = 4;
                image[0x31] = 0xff;
                image[0x20..0x22].copy_from_slice(&0x200u16.to_le_bytes());
                image[0xe6..0xe8].copy_from_slice(&scroll.to_le_bytes());
                image[0xa0..0xa2].copy_from_slice(&0x181u16.to_le_bytes());
                for address in (0x600..0x610).step_by(2) { image[address+1] = 0x40; }
                cases.push(image);
            }
        }
    }
    for image in cases {
        let mut native = ZeldaState::new();
        native.set_lag_free(true);
        native.import_wram_image(&image);
        native.sync_native_game_state_from_ram();
        let (total, own) = load_price_rom_work(0x02bb90, &native.wram_image(), 0,
            false, 0x02bb90..=0x02bd61);
        let before = crate::cycle_ledger::master();
        let measured = crate::cycle_ledger::measure_routine_self_work(0x02bb90, || {
            native.overworld_operate_camera_scroll();
        });
        assert_eq!(measured, own, "overlay={:x} velocity={} position={} screen={} z={}",
            image[0x8c], image[0x30], u16::from_le_bytes([image[0x20],image[0x21]]),
            image[0x8a], u16::from_le_bytes([image[0x24],image[0x25]]));
        assert_eq!(crate::cycle_ledger::master() - before, total);
    }
    for entry in [0x01ff28u32, 0x07f514, 0x07e9d3, 0x07e8f0] {
        for indoors in [0u8, 1] {
            for x in [0u8, 1, 0xff] {
                for y in [0u8, 1, 0xff] {
                    let mut native = ZeldaState::new();
                    native.set_lag_free(true);
                    let mut image = native.wram_image().into_owned();
                    image[0x1b] = indoors;
                    image[0x23] = x;
                    image[0x21] = y;
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    let end = match entry { 0x01ff28 => 0x01ffb5, 0x07f514 => 0x07f51c,
                        0x07e9d3 => 0x07ea05, _ => 0x07e900 };
                    let (_, own) = load_price_rom_work(entry, &native.wram_image(), 0,
                        matches!(entry, 0x01ff28 | 0x07e9d3), entry..=end);
                    let measured = crate::cycle_ledger::measure_routine_self_work(entry, || {
                        match entry {
                            0x01ff28 => native.cache_camera_properties(),
                            0x07f514 => native.cache_camera_properties_if_outdoors(),
                            0x07e9d3 => native.apply_links_movement_to_camera(),
                            _ => native.handle_indoor_camera_and_doors(),
                        }
                    });
                    assert_eq!(measured, own, "entry={entry:x} indoors={indoors} x={x} y={y}");
                }
            }
        }
    }
}

#[test]
fn ordinary_load_prices_match_executed_rom_paths() {
    let assets = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap();
    for entry in [0x00e2d0u32, 0x00e33b, 0x00d337, 0x02c630] {
        let mut native = ZeldaState::new();
        native.set_assets(&assets).unwrap();
        native.set_lag_free(true);
        let y = if entry == 0xe33b { 0x6a } else { 0x5b };
        let end = match entry { 0xe2d0 => 0xe33a, 0xe33b => 0xe36c,
            0xd337 => 0xd393, _ => 0x02c68b };
        let (total, own) = load_price_rom_work(entry, &native.wram_image(), y,
            matches!(entry, 0xe2d0 | 0xd337), entry..=end);
        let before = crate::cycle_ledger::master();
        let measured = crate::cycle_ledger::measure_routine_self_work(entry, || {
            match entry {
                0xe2d0 => native.load_default_graphics(),
                0xe33b => native.decompress_and_upload_2bpp(0x7000, y as usize),
                0xd337 => native.decompress_animated_dungeon_tiles(y as usize),
                _ => native.dungeon_load_palettes_before_cache(),
            }
        });
        assert_eq!(measured, own, "entry={entry:06x}");
        if matches!(entry, 0xe2d0 | 0xe33b | 0xd337) {
            assert_eq!(crate::cycle_ledger::master() - before, total, "entry={entry:06x}");
        }
    }
}

#[test]
fn dungeon_entrance_prices_match_executed_rom_paths() {
    let assets = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap();
    for (follower, restart, entrance) in [(0u8, 0u8, 0u8), (4, 0, 0),
        (0, 1, 0), (0, 0, 0x43), (0, 0, 0x84)] {
        for start in [0u8, 1, 6] {
            for progress in [0u16, 1, 2, 0x100] {
                for game_over in [0u8, 1] {
                    let mut native = ZeldaState::new();
                    native.set_assets(&assets).unwrap();
                    native.set_lag_free(true);
                    let mut image = native.wram_image().into_owned();
                    image[0x10a] = game_over;
                    image[0x10e] = entrance;
                    image[0x4aa] = restart;
                    image[0xf3cc] = follower;
                    image[0xf3c8] = start;
                    image[0xf3c5..0xf3c7].copy_from_slice(&progress.to_le_bytes());
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    let (total, _) = load_price_rom_work(0x02d8b3, &native.wram_image(), 0,
                        false, 0x02d8b3..=0x02dd87);
                    let before = crate::cycle_ledger::master();
                    native.dungeon_load_entrance();
                    assert_eq!(crate::cycle_ledger::master() - before, total,
                        "follower={follower} restart={restart} entrance={entrance} start={start} progress={progress} game_over={game_over}");
                }
            }
        }
    }
}

#[test]
fn spotlight_exit_caller_prices_match_executed_rom_paths() {
    let assets = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap();
    let mut fixture = ZeldaState::new();
    fixture.set_assets(&assets).unwrap();
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    // The unmodified spotlight child reads $2138 as the high byte of its
    // 16-bit LDA $2137; LDA $213F also reads APUI0 as its high byte.
    // Retain hardware OAM and APU ports, rather than inventing
    // a constant bus response or replacing that child with a stub.
    let mut source_bus = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom)
        .unwrap().machine().snes().clone();
    source_bus.write(0x2100, 0x80);
    let entrances: Vec<_> = [3, 0xf2, 0x12].into_iter().map(|track|
        (0..0x85).find(|&entrance|
            fixture.zelda_get_entrance_music_track(entrance) == track).unwrap()).collect();
    for indoors in [false, true] {
        for &entrance in &entrances {
            for progress in [0u8, 2, 0xf2] {
                for current in [0u8, 12] {
                    let mut native = fixture.clone();
                    native.set_lag_free(true);
                    let mut image = native.wram_image().into_owned();
                    image[0x1b] = u8::from(indoors);
                    image[0x10e] = entrance as u8;
                    image[0xf3c5] = progress;
                    image[0x130] = current;
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    let (_, source) = load_price_rom_work_on_bus(0x0299ca,
                        &native.wram_image(), 0, false, 0x0299ca..=0x029a18,
                        source_bus.clone());
                    crate::cycle_ledger::profile_iteration(true);
                    crate::cycle_ledger::profile_main_begin();
                    native.dungeon_prep_exit_with_spotlight();
                    let rows = crate::cycle_ledger::take_routine_profile();
                    assert_eq!(rows[&0x02_99ca][0], 1);
                    assert_eq!(rows[&0x02_99ca][2], source,
                        "indoors={indoors} entrance={entrance} progress={progress} current={current}");
                    assert_eq!(rows[&0x00_f28b][2], if indoors { 1016 } else { 1130 });
                    crate::cycle_ledger::profile_iteration(false);
                }
            }
        }
    }
}

#[test]
fn whirlpool_restoration_prices_match_executed_rom_paths() {
    for main in [0u16, 1, 0x20, 0x3ff, 0x7fff] {
        for aux in [0u16, 1, 0x20, 0x3ff, 0x7fff] {
            for countdown in [0u8, 29, 30, 255] {
                let mut native = ZeldaState::new();
                native.set_lag_free(true);
                native.set_countdown(countdown);
                for i in 0x20..0x100 {
                    native.set_main_color(i, main);
                    native.set_aux_color(i, aux);
                }
                let (total, _) = load_price_rom_work(0x00f0c7, &native.wram_image(), 0,
                    true, 0x00f0c7..=0x00f131);
                let before = crate::cycle_ledger::master();
                native.palette_filter_whirlpool_restore_red_green();
                assert_eq!(crate::cycle_ledger::master() - before, total,
                    "main={main:04x} aux={aux:04x} countdown={countdown}");
            }
        }
    }
}

#[test]
fn desert_iris_calculation_prices_match_executed_rom_paths() {
    for radius in [0u8, 1, 16, 128, 192] {
        for y_buffer in [0u8, 1, 8, 16, 128, 255] {
            for opening in [0u8, 1] {
                let mut native = ZeldaState::new();
                native.set_lag_free(true);
                native.set_spotlight_window_radius_byte(radius);
                native.set_spotlight_window_y_buffer_byte(y_buffer);
                native.set_spotlight_window_state_byte(opening);
                let (total, _) = load_price_rom_work(0x07ecdc, &native.wram_image(), 0,
                    false, 0x07ecdc..=0x07ed2b);
                let before = crate::cycle_ledger::master();
                native.desert_hdma_calculate_iris_shape_line();
                assert_eq!(crate::cycle_ledger::master() - before, total,
                    "radius={radius} y_buffer={y_buffer} opening={opening}");
            }
        }
    }
}

#[test]
fn desert_iris_builder_prices_match_executed_rom_paths() {
    for radius in [1u8, 16, 64, 128, 184, 192] {
        for (x, y) in [(0u16, 0u16), (128, 100), (511, 240)] {
            for opening in [0u8, 1] {
                for subsub in [2u8, 4] {
                    let mut native = ZeldaState::new();
                    native.set_lag_free(true);
                    let mut image = native.wram_image().into_owned();
                    image[0x20..0x22].copy_from_slice(&y.to_le_bytes());
                    image[0x22..0x24].copy_from_slice(&x.to_le_bytes());
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    native.set_spotlight_window_radius_byte(radius);
                    native.set_spotlight_window_state_byte(opening);
                    native.set_subsubmodule(subsub);
                    let (_, own) = load_price_rom_work(0x07ea27, &native.wram_image(), 0,
                        true, 0x07ea27..=0x07ebd9);
                    let measured = crate::cycle_ledger::measure_routine_self_work(0x07ea27, || {
                        native.desert_prayer_build_complete_iris_hdma_table();
                    });
                    assert_eq!(measured, own,
                        "radius={radius} x={x} y={y} opening={opening} subsub={subsub}");
                }
            }
        }
    }
}

#[test]
fn mode7_transfer_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    for lag_free in [false, true] {
        let mut native = ZeldaState::new();
        native.set_assets(&assets).unwrap();
        native.set_lag_free(lag_free);
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, &rom, 0x2000);
        machine.ram[0x1fc..0x1ff].copy_from_slice(&[0xff, 0x7f, 0]);
        machine.cpu.e = false;
        machine.cpu.unpack_flags(0x30);
        machine.cpu.pc = 0xe399;
        machine.cpu.sp = 0x1fb;
        let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
        let mut source = RomCpuTimingProbe::new(machine, clock,
            SourcePpuReadState::snes9x_reset()).unwrap();
        let mut measured = 0;
        for _ in 0..100000 {
            let step = source.step().unwrap();
            measured += step.transactions.iter()
                .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
            if source.snes().cpu.pc == 0x8000 { break; }
        }
        assert_eq!(source.snes().cpu.pc, 0x8000);
        assert_eq!(measured, 1_696_052);
        let before = crate::cycle_ledger::master();
        native.transfer_mode7_characters();
        assert_eq!(crate::cycle_ledger::master() - before,
            if lag_free { measured } else { 0 });
        assert_eq!(&native.ppu.vram[..0x4000], &source.snes().ppu.vram[..0x4000]);
    }
}

#[test]
fn subtractive_palette_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let Ok(rom) = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")) else { return; };
    for a in [0u16, 1, 0x1f, 0x20, 0x3ff, 0x400, 0x7fff] {
        for b in [0u16, 1, 0x1f, 0x20, 0x3ff, 0x400, 0x7fff] {
            let mut native = ZeldaState::new();
            native.set_lag_free(true);
            native.set_main_color(0, a);
            native.set_aux_color(0, b);
            native.set_main_color(1, b);
            native.set_aux_color(1, a);
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&native.wram_image());
            machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0);
            machine.cpu.a = 4;
            machine.cpu.k = 0;
            machine.cpu.db = 0;
            machine.cpu.pc = 0xee21;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut measured = 0;
            for _ in 0..1000 {
                let step = source.step().unwrap();
                measured += step.transactions.iter()
                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                if source.snes().cpu.pc == 0x8000 { break; }
            }
            assert_eq!(source.snes().cpu.pc, 0x8000);
            let before = crate::cycle_ledger::master();
            native.palette_filter_restore_subtractive(0, 4);
            assert_eq!(crate::cycle_ledger::master() - before, measured, "a={a:04x} b={b:04x}");
        }
    }
}

#[test]
fn hud_rebuild_self_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(assets), Ok(rom)) = (std::fs::read(root.join("zelda3_assets.dat")),
        std::fs::read(root.join("saves/zelda3.sfc"))) else { return; };
    for entry in [0x0dfa70u32, 0x0dfa85] {
        let mut native = ZeldaState::new();
        native.set_assets(&assets).unwrap();
        native.set_lag_free(true);
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, &rom, 0x2000);
        machine.ram.copy_from_slice(&native.wram_image());
        machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
        machine.cpu.e = false;
        machine.cpu.unpack_flags(0x30);
        machine.cpu.k = 13;
        machine.cpu.db = 13;
        machine.cpu.pc = entry as u16;
        machine.cpu.sp = 0x1fc;
        let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
        let mut source = RomCpuTimingProbe::new(machine, clock,
            SourcePpuReadState::snes9x_reset()).unwrap();
        let mut measured = 0;
        for _ in 0..100000 {
            let caller = source.snes().cpu.k == 13 && (0xfa70..=0xfa92).contains(&source.snes().cpu.pc);
            let step = source.step().unwrap();
            if caller {
                measured += step.transactions.iter()
                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
            }
            if source.snes().cpu.pc == 0x8000 { break; }
        }
        assert_eq!(source.snes().cpu.pc, 0x8000);
        let measured_native = crate::cycle_ledger::measure_routine_self_work(entry, || {
            if entry == 0x0dfa70 { native.hud_rebuild(); }
            else { native.hud_update_only(); }
        });
        assert_eq!(measured_native, measured, "entry={entry:06x}");
    }
}

#[test]
fn tile_lookup_prefix_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(assets), Ok(rom)) = (std::fs::read(root.join("zelda3_assets.dat")),
        std::fs::read(root.join("saves/zelda3.sfc"))) else { return; };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    for entry in [0x07d798u32, 0x07d9d8] {
        for indoors in [0u8, 1] {
            for floor in [0u8, 1] {
                for cheat in [0u8, 1] {
                    let mut native = base.clone();
                    let mut image = native.wram_image().into_owned();
                    image[0x1b] = indoors;
                    image[0xee] = floor;
                    image[0x37f] = cheat;
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram.copy_from_slice(&native.wram_image());
                    machine.ram[..4].fill(0);
                    machine.ram[0xa] = 1;
                    machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(0);
                    machine.cpu.k = 7;
                    machine.cpu.db = 7;
                    machine.cpu.pc = entry as u16;
                    machine.cpu.sp = 0x1fc;
                    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                    let mut source = RomCpuTimingProbe::new(machine, clock,
                        SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut measured = 0;
                    for _ in 0..10000 {
                        let step = source.step().unwrap();
                        measured += step.transactions.iter()
                            .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                        if source.snes().cpu.pc == 0x8000 { break; }
                    }
                    let before = crate::cycle_ledger::master();
                    if entry == 0x07d798 { native.tile_detect_reset_state(); }
                    else { native.tile_detection_execute(0, 0, 1); }
                    assert_eq!(crate::cycle_ledger::master() - before, measured,
                        "entry={entry:06x} indoors={indoors} floor={floor} cheat={cheat}");
                }
            }
        }
    }
}

#[test]
fn high_expansion_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(rom) = std::fs::read(root.join("saves/zelda3.sfc")) else { return; };
    let data: Vec<u8> = (0..0x1000).map(|i| (i * 37) as u8).collect();
    for offset in [0usize, 0x68, 0x180, 0x1e8] {
        for tiles in [1usize, 2, 8, 12] {
            let mut native = ZeldaState::new();
            native.set_lag_free(true);
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram[0x4000..0x5000].copy_from_slice(&data);
            machine.ram[2] = 0x7e;
            machine.ram[5] = 0x7e;
            machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0);
            machine.cpu.k = 0;
            machine.cpu.db = 0;
            machine.cpu.a = (0x4000 + offset) as u16;
            machine.cpu.x = 0;
            machine.cpu.y = tiles as u16;
            machine.cpu.pc = 0xd61c;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut measured = 0;
            for _ in 0..20000 {
                let step = source.step().unwrap();
                measured += step.transactions.iter()
                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                if source.snes().cpu.pc == 0x8000 { break; }
            }
            assert_eq!(source.snes().cpu.pc, 0x8000);
            let before = crate::cycle_ledger::master();
            native.expand3_to_4_high_from_slice(0x9000, &data, offset, 0, tiles);
            assert_eq!(crate::cycle_ledger::master() - before, measured,
                "offset={offset:04x} tiles={tiles}");
            assert_eq!(&native.wram_image()[0x9000..0x9000 + tiles * 32],
                &source.snes().ram[0x9000..0x9000 + tiles * 32]);
        }
    }
}

#[test]
fn scroll_stripe_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    // Cover both quadrant branches, rotations at both buffer ends, every
    // direction and the four offset-update callers. No source hold counts.
    for entry in [0x02f3b9, 0x02f482, 0x02f2f1, 0x02f325, 0x02f363, 0x02f39d] {
        for unit in [0, 15, 16, 31] {
            for rotation in [0, 31] {
                for direction in [1, 2, 4, 8] {
                    let mut native = base.clone();
                    native.set_overworld_map16_src_off(0x1000);
                    native.set_overworld_map16_dst_off(if entry == 0x02f482 { rotation } else { unit });
                    native.set_overworld_map16_y_unit(if entry == 0x02f482 { unit } else { rotation });
                    native.set_screen_transition_direction_bits(direction);
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram.copy_from_slice(&native.wram_image());
                    machine.ram[0x0e..0x10].copy_from_slice(&6u16.to_le_bytes());
                    machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(0);
                    machine.cpu.k = 2;
                    machine.cpu.db = 0x7e;
                    machine.cpu.pc = entry as u16;
                    machine.cpu.sp = 0x1fc;
                    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                    let mut source = RomCpuTimingProbe::new(machine, clock,
                        SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut measured = 0;
                    for _ in 0..10000 {
                        let step = source.step().unwrap();
                        measured += step.transactions.iter()
                            .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                        if source.snes().cpu.pc == 0x8000 { break; }
                    }
                    assert_eq!(source.snes().cpu.pc, 0x8000);
                    let before = crate::cycle_ledger::master();
                    match entry {
                        0x02f3b9 => { native.buffer_and_build_map16_stripes_x(3); }
                        0x02f482 => { native.buffer_and_build_map16_stripes_y(3); }
                        0x02f2f1 => { native.build_full_stripe_during_transition_north(3); }
                        0x02f325 => { native.build_full_stripe_during_transition_south(3); }
                        0x02f363 => { native.build_full_stripe_during_transition_west(3); }
                        _ => { native.build_full_stripe_during_transition_east(3); }
                    }
                    assert_eq!(crate::cycle_ledger::master() - before, measured,
                        "entry={entry:06x} unit={unit} rotation={rotation} direction={direction}");
                }
            }
        }
    }
}

#[test]
fn initial_scroll_prices_match_executed_rom_paths_and_slices() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    for small in [false, true] {
        let screen = (0..128).find(|&screen| (base.asset_u8(107, screen) != 0) == small).unwrap();
        for direction in [1, 2, 4, 8] {
            for source_offset in [0, 0x7f, 0x1080] {
                let mut native = base.clone();
                native.set_overworld_screen(screen as u8);
                native.set_screen_transition_direction_bits(direction);
                native.set_overworld_map16_src_off(source_offset);
                native.set_overworld_map16_dst_off(17);
                native.set_overworld_map16_y_unit(13);
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&native.wram_image());
                machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0x30);
                machine.cpu.k = 2;
                machine.cpu.db = 0x7e;
                machine.cpu.pc = 0xf031;
                machine.cpu.sp = 0x1fc;
                let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                    CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                let mut source = RomCpuTimingProbe::new(machine, clock,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                let mut measured = 0;
                for _ in 0..100000 {
                    let step = source.step().unwrap();
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                    if source.snes().cpu.pc == 0x8000 { break; }
                }
                assert_eq!(source.snes().cpu.pc, 0x8000);
                let mut split = native.clone();
                let before = crate::cycle_ledger::master();
                native.CreateInitialNewScreenMapToScroll();
                assert_eq!(crate::cycle_ledger::master() - before, measured,
                    "small={small} direction={direction} source_offset={source_offset:04x}");
                let before = crate::cycle_ledger::master();
                let mut cursor = split.begin_initial_scroll_cursor();
                for target in 1..=cursor.total_steps() {
                    split.advance_initial_scroll_cursor(&mut cursor, target);
                }
                assert_eq!(crate::cycle_ledger::master() - before, measured);
            }
        }
    }
}

#[test]
fn palette_copy_prices_match_executed_rom_paths_and_partial_rows() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(assets), Ok(rom)) = (std::fs::read(root.join("zelda3_assets.dat")),
        std::fs::read(root.join("saves/zelda3.sfc"))) else { return; };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    for entry in [0x1bef30, 0x1bef4b, 0x1bef7b] {
        for words in [1usize, 7, 15, 16] {
            for rows in [1usize, 3] {
                let mut native = base.clone();
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&native.wram_image());
                machine.ram[..2].copy_from_slice(&(PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR as u16).to_le_bytes());
                machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0);
                machine.cpu.k = 0x1b;
                machine.cpu.db = 0x1b;
                machine.cpu.a = 0x52;
                machine.cpu.x = (words - 1) as u16;
                machine.cpu.y = (rows - 1) as u16;
                machine.cpu.pc = entry as u16;
                machine.cpu.sp = 0x1fc;
                let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                    CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                let mut source = RomCpuTimingProbe::new(machine, clock,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                let mut measured = 0;
                for _ in 0..20000 {
                    let step = source.step().unwrap();
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                    if source.snes().cpu.pc == 0x8000 { break; }
                }
                assert_eq!(source.snes().cpu.pc, 0x8000);
                let before = crate::cycle_ledger::master();
                match entry {
                    0x1bef30 => native.palette_load_single(PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 0x52, words - 1),
                    0x1bef4b => native.palette_load_multiple(PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 0x52, words - 1, rows - 1),
                    _ => native.palette_load_multiple_arbitrary_snes(PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 0x52, words - 1),
                }
                assert_eq!(crate::cycle_ledger::master() - before, measured,
                    "entry={entry:06x} words={words} rows={rows}");
                if entry == 0x1bef4b && words == 7 && rows == 3 {
                    for cut in 0..=21 {
                        let mut split = base.clone();
                        let before = crate::cycle_ledger::master();
                        split.palette_load_ow_bg2_word_range(0, cut);
                        split.palette_load_ow_bg2_word_range(cut, 21);
                        split.palette_load_ow_bg2_word_range(21, 21);
                        assert_eq!(crate::cycle_ledger::master() - before, measured, "cut={cut}");
                    }
                }
            }
        }
    }
}

#[test]
fn song_list_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(assets), Ok(rom)) = (std::fs::read(root.join("zelda3_assets.dat")),
        std::fs::read(root.join("saves/zelda3.sfc"))) else { return; };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    for progress in 0..=3u8 {
        for sword in 0..=4u8 {
            let entry = 0x02c463u32;
                let mut native = base.clone();
                let mut image = native.wram_image().into_owned();
                image[0xf3c5] = progress;
                image[0xf359] = sword;
                native.import_wram_image(&image);
                native.sync_native_game_state_from_ram();
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&native.wram_image());
                machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0x30);
                machine.cpu.k = (entry >> 16) as u8;
                machine.cpu.db = 2;
                machine.cpu.dp = 0;
                machine.cpu.pc = entry as u16;
                machine.cpu.sp = 0x1fc;
                let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                    CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                let mut source = RomCpuTimingProbe::new(machine, clock,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                let mut measured = 0;
                for _ in 0..10000 {
                    let step = source.step().unwrap();
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                    if source.snes().cpu.pc == 0x8000 { break; }
                }
                assert_eq!(source.snes().cpu.pc, 0x8000);
                let start = crate::cycle_ledger::master();
                native.overworld_set_song_list();
                assert_eq!(crate::cycle_ledger::master() - start, measured,
                    "progress={progress} sword={sword}");
        }
    }
}

#[test]
fn entrance_and_tile_attribute_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(assets), Ok(rom)) = (std::fs::read(root.join("zelda3_assets.dat")),
        std::fs::read(root.join("saves/zelda3.sfc"))) else { return; };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    for tile in 0..64u16 {
        for facing in [0, 2] {
            for entry in [0x00882e, 0x1bbbf4] {
                let mut native = base.clone();
                let mut image = native.wram_image().into_owned();
                image[0x10] = 9;
                image[0x2f] = facing;
                image[0x2000..0x2002].copy_from_slice(&tile.to_le_bytes());
                image[0x2002..0x2004].copy_from_slice(&tile.wrapping_add(1).to_le_bytes());
                native.import_wram_image(&image);
                native.sync_native_game_state_from_ram();
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&native.wram_image());
                machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0x30);
                machine.cpu.k = (entry >> 16) as u8;
                machine.cpu.db = 0;
                machine.cpu.dp = 0;
                machine.cpu.pc = entry as u16;
                machine.cpu.sp = 0x1fc;
                let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                    CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                let mut source = RomCpuTimingProbe::new(machine, clock,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                let mut measured = 0;
                for _ in 0..10000 {
                    let step = source.step().unwrap();
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                    if source.snes().cpu.pc == 0x8000 { break; }
                }
                assert_eq!(source.snes().cpu.pc, 0x8000);
                let start = crate::cycle_ledger::master();
                if entry == 0x00882e { native.overworld_tile_definition_at_location(0, 0); }
                else { native.overworld_use_entrance(); }
                assert_eq!(crate::cycle_ledger::master() - start, measured,
                    "entry={entry:06x} tile={tile} facing={facing}");
            }
        }
    }
}

#[test]
fn initial_scroll_resumes_without_replaying_stripes_or_coordinate_tails() {
    let path = std::env::var_os("ZELDA3_ASSETS").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat"));
    let Ok(assets) = std::fs::read(path) else { return };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for index in 0..0x3000 {
        write_le_u16(&mut base.ram, 0x2000 + index * 2, ((index * 7) & 0xff) as u16);
    }
    for small in [false, true] {
        let screen = (0..64).find(|&screen| (base.asset_u8(107, screen) != 0) == small).unwrap();
        for direction in [1, 2, 4, 8] {
            let mut initial = base.clone();
            initial.set_overworld_screen_word(screen as u16);
            initial.set_screen_transition_direction_bits(direction);
            initial.set_overworld_map16_src_off(0x0440);
            initial.set_overworld_map16_dst_off(15);
            initial.set_overworld_map16_y_unit(17);
            let mut expected = initial.clone();
            expected.reference_CreateInitialNewScreenMapToScroll();
            let mut probe = initial.clone();
            let total = probe.begin_initial_scroll_cursor().total_steps();
            for cut in [0, 1, 31, 32, 33, 34, 97, 98, 161, 162, 163, 164, 165, 166, 167, total - 2, total - 1, total] {
                let mut resumed = initial.clone();
                let mut cursor = resumed.begin_initial_scroll_cursor();
                let pending = resumed.ram[0x1102..0x1950].to_vec();
                resumed.advance_initial_scroll_cursor(&mut cursor, cut);
                if cut <= 32 {
                    assert_eq!(&resumed.wram_image()[0x1102..0x1950], &pending,
                        "stripe source stores must not publish map8 output");
                }
                let once = resumed.wram_image().into_owned();
                resumed.advance_initial_scroll_cursor(&mut cursor, cut);
                assert_eq!(*resumed.wram_image(), once);
                resumed.native_initial_scroll = Some(cursor);
                resumed.CreateInitialNewScreenMapToScroll();
                assert!(resumed.native_initial_scroll.is_none());
                assert_eq!(*resumed.wram_image(), *expected.wram_image(), "small={small} dir={direction} cut={cut}");
                assert_eq!(resumed.game_state, expected.game_state);
            }
            let mut completed = initial.clone();
            let mut cursor = completed.begin_initial_scroll_cursor();
            completed.advance_initial_scroll_cursor(&mut cursor, total);
            completed.set_overworld_map16_stripe_word(0, 0x1234);
            completed.write_overworld_vram_word(1, 0x9876);
            completed.set_overworld_map16_src_off(0x4321);
            let once = completed.ram.clone();
            completed.native_initial_scroll = Some(cursor);
            completed.CreateInitialNewScreenMapToScroll();
            assert_eq!(completed.ram, once);
            for horizontal in [true, false] {
                let mut actual = initial.clone();
                let mut frozen = initial.clone();
                let (a, b) = if horizontal {
                    (actual.buffer_and_build_map16_stripes_x(7), frozen.reference_BufferAndBuildMap16Stripes_X(7))
                } else {
                    (actual.buffer_and_build_map16_stripes_y(7), frozen.reference_BufferAndBuildMap16Stripes_Y(7))
                };
                assert_eq!(a, b);
                assert_eq!(*actual.wram_image(), frozen.ram);
                assert_eq!(actual.game_state, frozen.game_state);
            }
        }
    }
}

#[test]
fn quadrant_publication_resumes_each_word_without_redrawing_completed_tiles() {
    let path = std::env::var_os("ZELDA3_ASSETS").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat"));
    let Ok(assets) = std::fs::read(path) else { return };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for screen in [0, 0x18, 0x40] {
        base.set_overworld_screen_word(screen);
        for offset in (0..0x2000).step_by(2) {
            base.dungeon_room_tilemaps_mut().set_bg2_tile_by_byte_pos(offset, 0xeeee);
        }
        let mut expected = base.clone();
        // The independent bulk path predates the resumable four-quadrant caller.
        for (dst, area) in [(0x2000, 0), (0x2040, 1), (0x3000, 8), (0x3040, 9)] {
            expected.overworld_decompress_and_draw_one_quadrant(dst, i32::from(screen) + area);
        }
        for cut in [0, 1, 2, 3, 4, 63, 1023, 1024, 1025, 2047, 3073, 4095, 4096] {
            let mut resumed = base.clone();
            let mut cursor = resumed.begin_overworld_quadrant_cursor();
            resumed.advance_overworld_quadrant_cursor(&mut cursor, cut);
            let mut prefix = base.ram[0x2000..0x4000].to_vec();
            for store in 0..usize::from(cut) {
                let quadrant = store / 1024;
                let tile = (store % 1024) / 4;
                let pos = [0, 0x40, 0x1000, 0x1040][quadrant]
                    + (tile / 16) * 256 + (tile % 16) * 4 + [0, 128, 2, 130][store % 4];
                prefix[pos..pos + 2].copy_from_slice(&expected.wram_image()[0x2000 + pos..0x2002 + pos]);
            }
            assert_eq!(&resumed.wram_image()[0x2000..0x4000], &prefix, "screen={screen} cut={cut}");
            let once = resumed.wram_image().into_owned();
            resumed.advance_overworld_quadrant_cursor(&mut cursor, cut);
            assert_eq!(*resumed.wram_image(), once);
            resumed.native_overworld_quadrants = Some(cursor);
            resumed.overworld_decompress_and_draw_all_quadrants();
            assert!(resumed.native_overworld_quadrants.is_none());
            assert_eq!(*resumed.wram_image(), *expected.wram_image(), "screen={screen} cut={cut}");
            assert_eq!(resumed.game_state, expected.game_state);
        }
        let mut cursor = expected.begin_overworld_quadrant_cursor();
        expected.advance_overworld_quadrant_cursor(&mut cursor, 4096);
        expected.dungeon_room_tilemaps_mut().set_bg2_tile_by_byte_pos(0, 0x1234);
        expected.ram[0x14400] = 0x79;
        let completed = expected.wram_image().into_owned();
        expected.native_overworld_quadrants = Some(cursor);
        expected.overworld_decompress_and_draw_all_quadrants();
        assert_eq!(*expected.wram_image(), completed);
    }
}

#[test]
fn pre_overworld_music_selection_preserves_f2_goto_setsong_path() {
    let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0xf2, 2, 0, 0x40, 1);

    assert_eq!(xt, 0xf3);
    assert_eq!(ow_anim_tiles, 0x5a);
}

#[test]
fn pre_overworld_music_selection_applies_darkworld_override_without_f2() {
    let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0x09, 2, 0, 0x40, 1);

    assert_eq!(xt, 9);
    assert_eq!(ow_anim_tiles, 0x5a);
}

#[test]
fn turtle_rock_vram_common_terminates_nmi_upload_data() {
    let mut state = ZeldaState::new();
    state.write_vram_upload_buffer_byte(6, 0);
    state.ram[UVRAM_DATA_OVERWORLD + 6] = 0;

    state.turtle_rock_vram_common(0x10);

    assert_eq!(state.vram_upload_buffer_byte(6), 0xff);
    assert_eq!(state.read_wram_byte(UVRAM_DATA_OVERWORLD + 6), 0);
}

#[test]
fn map8_conversion_resumes_at_stripe_header_plane_and_row_tail_boundaries() {
    use crate::zelda_rtl::overworld::OVERWORLD_MAP8_TOTAL_STEPS;
    let mut base = ZeldaState::new();
    let mut ranges = vec![(0, 0); 71];
    let table: Vec<u8> = (0..0x4000u16).flat_map(|i| (i ^ 0x4321).to_le_bytes()).collect();
    ranges[70] = (0, table.len());
    base.assets = Some(AssetPack::from_data_ranges(table.clone(), ranges));
    for i in 0..0x2000 {
        write_le_u16(&mut base.ram, 0x2000 + i * 2, ((i * 7) & 0xfff) as u16);
    }
    base.ram[0x12000..0x14080].fill(0xa5);
    base.sync_native_game_state_from_ram();
    for (page, page_base, upload_base) in [
        (OverworldMap16SourcePage::Main, 0x2000, 0),
        (OverworldMap16SourcePage::Overlay, 0x4000, 0x1000),
    ] {
        for (source, destination, y_unit) in [(0x003eu16, 31u16, 15u16), (0x1f80, 0, 31), (0xfff0, 63, 0)] {
            let mut initial = base.clone();
            initial.set_overworld_map16_src_off(source);
            initial.set_overworld_map16_dst_off(destination);
            initial.set_overworld_map16_y_unit(y_unit);
            // Frozen bulk specification: derive each stripe from the input
            // page and expand all four quarters without using the new cursor.
            let mut expected = initial.clone();
            let mut src = source.wrapping_add(0x1000);
            let mut y = y_unit;
            for row in 0..32 {
                for i in 0..32 {
                    let offset = src.wrapping_sub(0x410).wrapping_add(i * 2) & 0x1fff;
                    let value = read_le_u16(&initial.wram_image(), page_base + usize::from(offset));
                    expected.set_overworld_map16_stripe_word(usize::from((destination + i) & 31), value);
                }
                for half in 0..2 {
                    let address = ((y & 15) * 64 + if y >= 16 { 0x800 } else { 0 }) + half * 0x400;
                    expected.write_vram_upload_absolute_word(0x14000 + row * 4 + usize::from(half) * 2, address | upload_base);
                    for tile in 0..16 {
                        let k = expected.game_state.world.transient.overworld_map16_stripe_word(usize::from(half) * 16 + tile);
                        for (quarter, offset) in [(0, 0), (1, 2), (2, 64), (3, 66)] {
                            let value = read_le_u16(&table, usize::from(k) * 8 + quarter * 2);
                            expected.dungeon_bg2_attributes_mut().import_aliased_map8_word(
                                row * 256 + usize::from(half) * 128 + tile * 4 + offset, value,
                            );
                        }
                    }
                }
                src = src.wrapping_sub(0x80);
                y = y.wrapping_sub(1) & 31;
            }
            expected.set_overworld_map16_src_off(src);
            expected.set_overworld_map16_y_unit(y);
            for cut in [0, 1, 31, 32, 33, 34, 96, 97, 98, 161, 162, 163, 164, 5000, OVERWORLD_MAP8_TOTAL_STEPS] {
                let mut resumed = initial.clone();
                let mut cursor = resumed.begin_overworld_map8_cursor(page, upload_base);
                resumed.advance_overworld_map8_cursor(&mut cursor, cut);
                if cut <= 32 {
                    assert_eq!(&resumed.wram_image()[0x12000..0x14080], &initial.wram_image()[0x12000..0x14080]);
                }
                let once = resumed.wram_image().into_owned();
                resumed.advance_overworld_map8_cursor(&mut cursor, cut);
                assert_eq!(*resumed.wram_image(), once);
                let mut expected = expected.clone();
                if cut == OVERWORLD_MAP8_TOTAL_STEPS {
                    // The next owner may reuse completed output before this
                    // caller retires. Completion must not redraw that prefix.
                    resumed.set_overworld_map16_stripe_word(0, 0xffff);
                    expected.set_overworld_map16_stripe_word(0, 0xffff);
                    resumed.dungeon_bg2_attributes_mut().set_bg2_attr(0, 0xf3);
                    expected.dungeon_bg2_attributes_mut().set_bg2_attr(0, 0xf3);
                }
                resumed.native_overworld_map8 = Some(cursor);
                resumed.Map16ToMap8(page, i32::from(upload_base));
                assert!(resumed.native_overworld_map8.is_none());
                assert_eq!(*resumed.wram_image(), *expected.wram_image(), "page={page:?} cut={cut}");
                assert_eq!(resumed.game_state, expected.game_state, "page={page:?} cut={cut}");
            }
        }
    }
}

// Frozen bulk translation used to verify resumable scroll publication.
impl ZeldaState {
    pub(super) fn reference_BufferAndBuildMap16Stripes_X(&mut self, mut dst: usize) -> usize {
        let strip = OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION
            [((self.screen_transition_direction_bits() >> 1) & 1) as usize];
        let mut pos = self.overworld_map16_src_off().wrapping_sub(strip);
        let mut y_unit_index = self.overworld_map16_y_unit() as usize & 0x1f;
        for _ in 0..32 {
            let tile = self.overworld_map16_stripe_source_word(pos);
            self.set_overworld_map16_stripe_word(y_unit_index, tile);
            y_unit_index = (y_unit_index + 1) & 0x1f;
            pos = pos.wrapping_add(0x80);
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut dst_unit = self.overworld_map16_dst_off();
        if dst_unit >= 0x10 {
            dst_unit &= 0x0f;
            r0 = 0x400;
        }
        r0 = r0.wrapping_add(dst_unit.wrapping_mul(2));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            self.write_overworld_vram_word(dst + 33, r0.wrapping_add(1));
            dst += 1;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .overworld_map16_stripe_word(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 33, s1);
                self.write_overworld_vram_word(dst + 1, s2);
                self.write_overworld_vram_word(dst + 34, s3);
                dst += 2;
            }
            dst += 33;
            r0 = r0.wrapping_add(0x800);
        }
        dst
    }

    pub(super) fn reference_BufferAndBuildMap16Stripes_Y(&mut self, mut dst: usize) -> usize {
        let strip_index = 1 + ((self.screen_transition_direction_bits() >> 2) & 1) as usize;
        let mut pos = self
            .overworld_map16_src_off()
            .wrapping_sub(OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION[strip_index]);
        let mut dst_unit_index = self.overworld_map16_dst_off() as usize & 0x1f;
        for _ in 0..32 {
            let tile = self.overworld_map16_stripe_source_word(pos);
            self.set_overworld_map16_stripe_word(dst_unit_index, tile);
            pos = pos.wrapping_add(2);
            dst_unit_index = (dst_unit_index + 1) & 0x1f;
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut y_unit = self.overworld_map16_y_unit();
        if y_unit >= 0x10 {
            y_unit &= 0x0f;
            r0 = 0x800;
        }
        r0 = r0.wrapping_add(y_unit.wrapping_mul(64));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            dst += 1;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .overworld_map16_stripe_word(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 32, s2);
                self.write_overworld_vram_word(dst + 1, s1);
                self.write_overworld_vram_word(dst + 33, s3);
                dst += 2;
            }
            dst += 32;
            r0 = r0.wrapping_add(0x400);
        }
        dst
    }

    pub(super) fn reference_TriggerAndFinishMapLoadStripe_Y(&mut self, mut n: i32) {
        self.set_screen_transition_direction_bits(8);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x0080);
        dst += 1;
        while n != 0 {
            dst = self.reference_BufferAndBuildMap16Stripes_Y(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(0x80);
            self.set_overworld_map16_src_off(src);
            let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_y_unit(y_unit);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn reference_TriggerAndFinishMapLoadStripe_X(&mut self, mut n: i32) {
        self.set_screen_transition_direction_bits(2);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x8040);
        dst += 1;
        while n != 0 {
            dst = self.reference_BufferAndBuildMap16Stripes_X(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(2);
            self.set_overworld_map16_src_off(src);
            let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_dst_off(off);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_North(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(0x380);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_y_unit(31);
        self.reference_TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_South(&mut self) {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        self.set_overworld_map16_src_off(pos.wrapping_add(0x780));
        self.set_overworld_map16_y_unit(7);
        self.reference_TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_West(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(14);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(31);
        self.reference_TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_East(&mut self) {
        let src = self
            .overworld_map16_src_off()
            .wrapping_sub(0x60)
            .wrapping_add(0x1e);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(7);
        self.reference_TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_North(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x700),
            dst_off: map16.dst_off,
            y_unit: 10,
        });
        self.set_overworld_map16_src_off(0x1390);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(31);
        self.reference_TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_South(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src & 0xff,
            dst_off: map16.dst_off,
            y_unit: 24,
        });
        self.set_overworld_map16_src_off(0x0790);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(7);
        self.reference_TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_West(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x20),
            dst_off: 8,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x044e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(31);
        self.reference_TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_East(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x60),
            dst_off: 0x18,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x041e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(7);
        self.reference_TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialNewScreenMapToScroll(&mut self) {
        let dir = self.screen_transition_direction_bits();
        if self.overworld_map_is_small() {
            match dir {
                1 => self.reference_CreateInitialOWScreenView_Small_East(),
                2 => self.reference_CreateInitialOWScreenView_Small_West(),
                4 => self.reference_CreateInitialOWScreenView_Small_South(),
                8 => self.reference_CreateInitialOWScreenView_Small_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "reference_CreateInitialNewScreenMapToScroll small invalid direction {}",
                        dir
                    );
                }
            }
        } else {
            match dir {
                1 => self.reference_CreateInitialOWScreenView_Big_East(),
                2 => self.reference_CreateInitialOWScreenView_Big_West(),
                4 => self.reference_CreateInitialOWScreenView_Big_South(),
                8 => self.reference_CreateInitialOWScreenView_Big_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "reference_CreateInitialNewScreenMapToScroll big invalid direction {}",
                        dir
                    );
                }
            }
        }
    }

}

// Execute the real instruction stream, selecting only the requested self PCs.
fn tile_price_source(entry: u16, image: &[u8], y: u16, flags: u8,
    stop_at_response: bool) -> u64 {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, SourcePpuReadState};
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    let mut machine = snes::Snes::new();
    machine.cart.load(CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(image);
    machine.ram[0x1fc..0x200].copy_from_slice(&[0xff, 0x7f, 0, 0]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(flags);
    machine.cpu.k = 7;
    machine.cpu.db = 7;
    machine.cpu.pc = entry;
    machine.cpu.sp = 0x1fb;
    machine.cpu.y = y;
    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
    let mut source = RomCpuTimingProbe::new(machine, clock,
        SourcePpuReadState::snes9x_reset()).unwrap();
    let mut measured = 0;
    for _ in 0..100_000 {
        let cpu = &source.snes().cpu;
        let pc = cpu.pc;
        if cpu.k == 7 && (pc == 0x8000 || (stop_at_response && pc == 0xd104)) {
            return measured;
        }
        let selected = cpu.k == 7 && if stop_at_response {
            (0xd077..=0xd103).contains(&pc) || pc == 0xd2c5
        } else {
            entry != 0xd077 || (0xd077..=0xd2c5).contains(&pc)
        };
        let step = source.step().unwrap();
        if selected { measured += step.transactions.iter()
            .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>(); }
    }
    panic!("entry={entry:04x} did not return");
}

#[test]
fn main_tile_probe_setup_prices_match_executed_rom() {
    let assets = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap();
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for item in 0..=8 {
        for facing in [0u8, 2, 4, 6] {
            for spin in [0u8, 1, 2, 3, 9, 10, 255] {
                let mut native = base.clone();
                let mut image = native.wram_image().into_owned();
                image[0x1b] = 1;
                image[0x20..0x24].copy_from_slice(&[0x80, 0, 0x80, 0]);
                image[0x2f] = facing;
                image[0x31c] = spin;
                image[0xec..0xee].copy_from_slice(&0x01ffu16.to_le_bytes());
                native.import_wram_image(&image);
                native.sync_native_game_state_from_ram();
                let source = tile_price_source(0xd077, &image, u16::from(item), 0x30, true);
                assert_eq!(native.main_tile_probe_setup_master(item), source,
                    "item={item} facing={facing} spin={spin}");
                // $07:D077..D2C5 now includes the marked response suffix.
                // Measure its full self work; classifier/response children
                // retain their own accounting outside this source range.
                if matches!(item, 0 | 4 | 5) {
                    let handler_source = tile_price_source(0xd077, &image,
                        u16::from(item), 0x30, false);
                    for lagless in [false, true] {
                        let mut state = native.clone();
                        state.set_lag_free(lagless);
                        let own = crate::cycle_ledger::measure_routine_self_work(0x07d077,
                            || state.tile_detect_main_handler(item));
                        assert_eq!(own, if lagless { handler_source } else { 0 });
                    }
                }
            }
        }
    }
}

#[test]
fn flat_tile_effect_prices_match_executed_rom() {
    use crate::tile_definition::NativeTile;
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    let mut count = 0;
    for indoors in [false, true] {
        for attribute in 0u8..=255 {
            for platform in [0u8, 1] {
                let mut native = ZeldaState::new();
                let mut image = native.wram_image().into_owned();
                image[0x1b] = u8::from(indoors);
                image[0x2f5] = platform;
                image[6] = attribute;
                image[0xa..0xc].copy_from_slice(&0x0102u16.to_le_bytes());
                native.import_wram_image(&image);
                native.sync_native_game_state_from_ram();
                let Some(price) = native.tile_effect_self_master(
                    NativeTile::from_cartridge(attribute), indoors) else { continue; };
                let pointer = if indoors { 0x7d7d8 } else { 0x7da2a } + u32::from(attribute) * 2;
                let offset = ((pointer >> 16) as usize & 0x7f) * 0x8000
                    + (pointer as usize & 0x7fff);
                let entry = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
                let measured = tile_price_source(entry, &image, 0, 0, false);
                assert_eq!(price, measured,
                    "attribute={attribute:02x} indoors={indoors} platform={platform} entry={entry:04x}");
                count += 1;
            }
        }
    }
    assert!(count > 600, "selected flat effect coverage: {count}");
}

#[test]
fn marker31_map16_link_coordinate_body_matches_rom() {
    use snes::{CartType, Snes};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(rom), Ok(assets)) = (std::fs::read(root.join("saves/zelda3.sfc")),
        std::fs::read(root.join("zelda3_assets.dat"))) else { return };
    let mut native = ZeldaState::new();
    native.set_assets(&assets).unwrap();
    native.set_lag_free(true);
    let mut machine = Snes::new();
    machine.cart.load(CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&native.wram_image());
    machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0);
    machine.cpu.k = 14;
    machine.cpu.db = 14;
    machine.cpu.pc = 0xde9a;
    machine.cpu.sp = 0x1fa;
    crate::cycle_ledger::assert_rom_marker_work(0x0ede9a, machine,
        |s| s.cpu.k == 14 && (0xde9a..0xdece).contains(&s.cpu.pc), || {
            native.overworld_get_map16_of_link_mult8();
        });
}

#[test]
fn marker31_horizontal_slope_setup_matches_rom() {
    use crate::game_state::{CollisionAxis, CollisionDirection, MovementProbeKind};
    use snes::{CartType, Snes};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(rom), Ok(assets)) = (std::fs::read(root.join("saves/zelda3.sfc")),
        std::fs::read(root.join("zelda3_assets.dat"))) else { return };
    for direction in 0..4 {
        let mut native = ZeldaState::new();
        native.set_assets(&assets).unwrap();
        native.write_wram_byte(0x1b, 1);
        native.write_wram_word(0xec, 0x1ff);
        native.sync_native_game_state_from_ram();
        native.set_lag_free(true);
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, &rom, 0x2000);
        machine.ram.copy_from_slice(&native.wram_image());
        machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
        machine.cpu.e = false;
        machine.cpu.unpack_flags(0x30);
        machine.cpu.k = 7;
        machine.cpu.db = 7;
        machine.cpu.pc = 0xcec9;
        machine.cpu.sp = 0x1fa;
        machine.cpu.y = direction * 2;
        crate::cycle_ledger::assert_rom_marker_work(0x07cec9, machine,
            |s| s.cpu.k == 7 && (0xcec9..0xcf0a).contains(&s.cpu.pc), || {
                native.detect_player_movement(CollisionAxis::Horizontal,
                    CollisionDirection::from_legacy(direction as u8), MovementProbeKind::Slope);
            });
    }
}

#[test]
fn load_pacing2_waving_table_body_matches_rom() {
    load_pacing2_mirror_table_body_matches_rom(false);
}

#[test]
fn load_pacing2_dewaving_table_body_matches_rom() {
    load_pacing2_mirror_table_body_matches_rom(true);
}

fn load_pacing2_mirror_table_body_matches_rom(dewaving: bool) {
    use snes::{CartType, Snes};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    // Odd/even, both displacement branches/signs, filter threshold, and both
    // dewaving completion branches. Animation returns via its $FF sentinel.
    for odd in [false, true] {
        for displacement in [0u16, 0x100, 0x7fff, 0xff00] {
            for countdown in [0u16, 0x30] {
                for screen in [0u8, 0x1b] {
                    let mut native = ZeldaState::new();
                    native.set_assets(&assets).unwrap();
                    native.write_wram_byte(0x1a, u8::from(odd));
                    native.write_wram_byte(0x8a, screen);
                    native.write_wram_word(0x6ac, displacement);
                    native.write_wram_word(0x6a2, 0xff00);
                    native.write_wram_word(0x6a4, 0x100);
                    native.write_wram_word(0x6a6, 0xffc0);
                    native.write_wram_word(0x6a8, 0x40);
                    native.write_wram_byte(0x6bb, 1);
                    native.write_wram_word(0xc007, countdown);
                    native.write_wram_word(0xc009, 0xff);
                    native.sync_native_game_state_from_ram();
                    native.set_lag_free(true);
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram.copy_from_slice(&native.wram_image());
                    // Physical source table and native relocated table
                    // share word indices; upper 16 native padding rows
                    // never feed the ROM-owned prefix in a descending copy.
                    for y in 0..240 {
                        let value = if displacement == 0 { 0 } else { y as u16 };
                        machine.ram[0x1b00 + y * 2..0x1b02 + y * 2]
                            .copy_from_slice(&value.to_le_bytes());
                        native.set_spotlight_hdma_table_dynamic_entry(y, value);
                    }
                    machine.ram[0x1fa..0x1fd].copy_from_slice(&[0xff, 0x7f, 0]);
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(0x30);
                    machine.cpu.k = 0;
                    machine.cpu.db = 0;
                    machine.cpu.pc = if dewaving { 0xff2f } else { 0xfe64 };
                    machine.cpu.sp = 0x1f9;
                    let entry = u32::from(machine.cpu.pc);
                    let source = crate::cycle_ledger::assert_rom_marker_work(entry, machine,
                        |s| s.cpu.k == 0 && if dewaving {
                            (0xff2e..0xffb7).contains(&s.cpu.pc)
                        } else {
                            s.cpu.pc == 0xfe5d || (0xfe64..0xff2f).contains(&s.cpu.pc)
                        }, || {
                            if dewaving { native.mirror_warp_build_dewaving_hdma_table(); }
                            else { native.mirror_warp_build_waving_hdma_table(); }
                        });
                    if !odd {
                        for y in (8..224).step_by(2) {
                            assert_eq!(native.spotlight_hdma_table_dynamic_entry(y),
                                u16::from_le_bytes([source.snes().ram[0x1b00+y*2],
                                    source.snes().ram[0x1b01+y*2]]));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn mirror_initialization_matches_rom_markers_scroll_values() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    for scroll in [0u16, 0x1ff, 0xff00] {
        let mut n = ZeldaState::new();
        n.write_wram_word(0xe2, scroll);
        n.sync_native_game_state_from_ram();
        n.set_lag_free(true);
        let mut s = snes::Snes::new();
        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.cpu.e = false; s.cpu.unpack_flags(0x30);
        s.cpu.k = 0; s.cpu.db = 0; s.cpu.pc = 0xfdee; s.cpu.sp = 0x1fc;
        s.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
        let source = crate::cycle_ledger::assert_rom_marker_work(0x00fdee, s,
            |s| s.cpu.k == 0 && (0xfdee..0xfe5e).contains(&s.cpu.pc),
            || n.InitializeMirrorHDMA());
        for i in 0..224 {
            assert_eq!(n.spotlight_hdma_table_dynamic_entry(i),
                u16::from_le_bytes([source.snes().ram[0x1b00+i*2], source.snes().ram[0x1b01+i*2]]));
        }
    }
}


#[test]
fn load6_fade_mosaic_dispatch_and_tails_match_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    for (phase, screen, module, countdown) in [
        (0u8, 1u8, 9u8, 2u16), (0, 0x40, 11, 3),
        (1, 1, 9, 2), (1, 1, 11, 3),
        (2, 1, 9, 2), (2, 0x80, 11, 3), (2, 0x2a, 9, 2),
    ] {
        let mut n = ZeldaState::new(); n.set_assets(&assets).unwrap();
        n.write_wram_byte(0x10, module); n.write_wram_byte(0x11, 0x16);
        n.write_wram_byte(0xb0, phase); n.write_wram_byte(0x8a, screen);
        n.write_wram_word(0xc007, countdown); n.write_wram_word(0xc011, 0x80);
        n.write_wram_word(0xc009, 0xff);
        n.sync_native_game_state_from_ram(); n.set_lag_free(true);
        let mut s = snes::Snes::new();
        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
        s.cpu.e = false; s.cpu.unpack_flags(0x30);
        s.cpu.k = 2; s.cpu.db = 2; s.cpu.pc = 0xb0d2; s.cpu.sp = 0x1fa;
        crate::cycle_ledger::assert_rom_marker_work(0x02b0d2, s,
            |s| {
                let pc = (u32::from(s.cpu.k) << 16) | u32::from(s.cpu.pc);
                (0x02b0d2..0x02b0ed).contains(&pc)
                    || (0x02b0f3..0x02b150).contains(&pc)
                    || (0x02b195..0x02b19e).contains(&pc)
                    || (0x008784..0x00879c).contains(&pc)
            }, || n.module09_fade_back_in_from_mosaic());
    }
}

#[test]
fn load6_mosaic_graphics_caller_markers_match_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    for request in [0u8, 1, 8] {
        let mut n = ZeldaState::new(); n.set_assets(&assets).unwrap();
        n.write_wram_byte(0xa0c, request); n.write_wram_byte(0x8a, 1);
        n.sync_native_game_state_from_ram(); n.set_lag_free(true);
        let mut s = snes::Snes::new(); s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
        s.cpu.e = false; s.cpu.unpack_flags(0x30);
        s.cpu.k = 2; s.cpu.db = 2; s.cpu.pc = 0xb171; s.cpu.sp = 0x1fa;
        crate::cycle_ledger::assert_rom_marker_work(0x02b171, s,
            |s| s.cpu.k == 2 && (0xb171..0xb195).contains(&s.cpu.pc),
            || n.overworld_mosaic_transition_load_sprite_graphics_and_set_mosaic());
    }
}

#[test]
fn load6_mosaic_register_child_markers_match_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    for countdown in [0u16, 1, 2, 255] {
        let mut n = ZeldaState::new(); n.set_assets(&assets).unwrap();
        n.write_wram_byte(0xb0, 2); n.write_wram_byte(0x8a, 0x80);
        n.write_wram_word(0xc007, countdown);
        n.sync_native_game_state_from_ram(); n.set_lag_free(true);
        let mut s = snes::Snes::new(); s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.ram[0x1fb..0x1fd].copy_from_slice(&[0xff, 0x7f]);
        s.cpu.e = false; s.cpu.unpack_flags(0x30);
        s.cpu.k = 2; s.cpu.db = 2; s.cpu.pc = 0xc2f6; s.cpu.sp = 0x1fa;
        crate::cycle_ledger::assert_rom_marker_work(0x02c2f6, s,
            |s| s.cpu.k == 2 && (0xc2f6..0xc303).contains(&s.cpu.pc),
            || n.module09_fade_back_in_from_mosaic());
    }
}
