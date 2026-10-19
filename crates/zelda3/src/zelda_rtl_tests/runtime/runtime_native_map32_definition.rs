use crate::cycle_models::{map32_definition::Map32Definition, native_work::compare_instruction_with_devices,
    intro_clear::Access, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[test]
fn map32_definition_assets_match_original_physical_tables() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let mut bytes = 0;
    for (base, asset) in [(0x038000u32,60), (0x03b400,61), (0x048000,62), (0x04b400,63)] {
        for (i, &byte) in state.asset_raw(asset).unwrap().iter().enumerate() {
            let address = base + i as u32;
            assert_eq!(state.native_map32_definition_byte(address), Some(byte));
            let physical = ((address >> 16) as usize) * 0x8000 + (address as usize & 0x7fff);
            assert_eq!(byte, rom[physical], "map32 asset={asset} byte={i}");
            bytes += 1;
        }
    }
    assert!(state.rom.is_empty());
    eprintln!("native map32 immutable original table bytes={bytes}");
}

#[test]
fn map32_definition_cache_hits_and_rebuilds_match_original_instructions() {
    let state = super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let groups = (60..64).map(|i| state.asset_raw(i).unwrap().len()/6).min().unwrap();
    let mut boundaries = 0;
    for case in 0..6144usize {
        let group = (case / 48) * (groups - 1) / 127;
        let selector = (case / 6) % 8;
        let hit = case / 3 % 2 == 0;
        let input = (group as u16 * 8) | selector as u16;
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, v) in machine.ram.iter_mut().enumerate() {
            *v = (i.wrapping_mul(37) ^ (i >> 8) ^ case.wrapping_mul(19)) as u8;
        }
        crate::types::write_le_u16(&mut machine.ram, 0, 0x4000);
        machine.ram[2] = 0x7e;
        crate::types::write_le_u16(&mut machine.ram, 3, 0x4080);
        machine.ram[5] = 0x7e;
        crate::types::write_le_u16(&mut machine.ram, 0x14440, (input & !7) ^ if hit {0} else {8});
        machine.ram[0x1f9..0x1fb].copy_from_slice(&[0x65,0xf8]);
        let mut cpu = NativeNmiContext { a: input, x: 0x81, y: [0,0x7e,0xfe,0xf7c][case % 4],
            d: 0, db: 0x7f, bank: 2, pc: 0xf691, sp: 0x1f8,
            p: [0,1,2,0x43,0x80,0xc3][case % 6] };
        machine.cpu.a=cpu.a; machine.cpu.x=cpu.x; machine.cpu.y=cpu.y;
        machine.cpu.dp=cpu.d; machine.cpu.db=cpu.db; machine.cpu.k=cpu.bank;
        machine.cpu.pc=cpu.pc; machine.cpu.sp=cpu.sp; machine.cpu.e=false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let clock = CpuMasterTimeline::at_raster(case as u64,
            [snes::CpuRasterPosition::new(100,532), snes::CpuRasterPosition::new(224,1338),
             snes::CpuRasterPosition::new(261,1350)][case % 3],
            snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        let mut source = snes::RomCpuTimingProbe::new(machine,clock,
            snes::SourcePpuReadState::snes9x_reset()).unwrap();
        let mut clock = source.timeline().clone();
        let mut work = Map32Definition::default();
        for _ in 0..256 {
            if work.complete() {break;}
            let label = format!("map32 case={case} group={group} selector={selector} hit={hit} pc={:04x}",cpu.pc);
            compare_instruction_with_devices(&label, &mut cpu, &mut clock, &mut source,
                &mut ram, &rom, None, |_, access| match access {
                    Access::Read(a, width) if a < 0x7e0000 && a & 0x8000 != 0 => {
                        let read = |a| state.native_map32_definition_byte(a)
                            .unwrap_or_else(|| panic!("unmapped native map32 data {a:06x}; {label}"));
                        Some(if width==1 {u16::from(read(a))} else {u16::from_le_bytes([read(a),read(a+1)])})
                    }
                    _ => None,
                }, |cpu, clock, bus| {work.step(cpu,clock,bus).unwrap();});
            assert_eq!(ram,source.snes().ram,"{label}");
            boundaries += 1;
        }
        assert!(work.complete());
        assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,0xf866,0x1fa));
    }
    assert!(state.rom.is_empty());
    eprintln!("native map32 Source cases=6144 instruction boundaries={boundaries}");
}

#[test]
fn overworld_map_stream_assets_match_original_tables_and_compressed_bytes() {
    let state=super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom=super::runtime_native_overworld_dungeon_exit::rom();
    let data=state.native_overworld_map_stream_data().unwrap();
    for (&address,&value) in &data {
        let physical=((address>>16) as usize)*0x8000+(address as usize&0x7fff);
        assert_eq!(value,rom[physical],"overworld map stream byte {address:06x}");
    }
    assert!(state.rom.is_empty());
    eprintln!("native overworld map stream original immutable bytes={}",data.len());
}

#[test]
fn overlay_quadrants_compose_real_decompression_interlace_and_definition_frames() {
    use crate::cycle_models::overlay_quadrant::OverlayQuadrant;
    let state=super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom=super::runtime_native_overworld_dungeon_exit::rom();
    let streams=state.native_overworld_map_stream_data().unwrap();
    let mut boundaries=0;
    for screen in 0..160u16 {
        let case=screen as usize;
        let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
        for (i,v) in machine.ram.iter_mut().enumerate() {*v=(i.wrapping_mul(37)^(i>>8)^case.wrapping_mul(19)) as u8;}
        crate::types::write_le_u16(&mut machine.ram,0x8a,screen);
        machine.ram[0x1f7..0x1f9].copy_from_slice(&[0x11,0xfd]);
        let mut cpu=NativeNmiContext {a:0xb701,x:0x95,y:0x9f7c,d:0,
            db:[0,2,0x80,0x82][case%4],bank:2,pc:0xf7cb,sp:0x1f6,
            p:[0,1,2,0x43,0x80,0xc3][case%6]};
        let saved_db=cpu.db;
        machine.cpu.a=cpu.a;machine.cpu.x=cpu.x;machine.cpu.y=cpu.y;
        machine.cpu.dp=cpu.d;machine.cpu.db=cpu.db;machine.cpu.k=cpu.bank;
        machine.cpu.pc=cpu.pc;machine.cpu.sp=cpu.sp;machine.cpu.e=false;machine.cpu.unpack_flags(cpu.p);
        let mut ram=machine.ram.clone();let clock=CpuMasterTimeline::at_raster(case as u64,
            [snes::CpuRasterPosition::new(100,532),snes::CpuRasterPosition::new(224,1338),
             snes::CpuRasterPosition::new(261,1350)][case%3],
            snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        let mut source=snes::RomCpuTimingProbe::new(machine,clock,
            snes::SourcePpuReadState::snes9x_reset()).unwrap();let mut clock=source.timeline().clone();
        let mut work=OverlayQuadrant::default();
        for _ in 0..250000 {
            if work.complete() {break;}
            let label=format!("overlay quadrant screen={screen:02x} pc={:04x}",cpu.pc);
            compare_instruction_with_devices(&label,&mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                |_,access| match access {
                    Access::Read(a,width) if a<0x7e0000 && a&0x8000!=0 => {
                        let read=|a|state.native_map32_definition_byte(a)
                            .or_else(||streams.get(&(a&0x7fffff)).copied())
                            .unwrap_or_else(||panic!("unmapped composed map data {a:06x}; {label}"));
                        Some(if width==1 {u16::from(read(a))} else {u16::from_le_bytes([read(a),read(a+1)])})
                    }
                    _=>None,
                },|cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
            assert_eq!(ram,source.snes().ram,"{label}");boundaries+=1;
        }
        assert!(work.complete());assert_eq!((cpu.bank,cpu.pc,cpu.sp,cpu.db),(2,0xfd12,0x1f8,saved_db));
    }
    assert!(state.rom.is_empty());
    eprintln!("native overlay quadrant Source cases=160 instruction boundaries={boundaries}");
}

#[test]
fn overlay_map8_asset_matches_original_physical_table() {
    let state=super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom=super::runtime_native_overworld_dungeon_exit::rom();
    let bytes=state.asset_raw(70).unwrap();
    for (i,&byte) in bytes.iter().enumerate() {
        let a=0x0f8000+i as u32;
        assert_eq!(state.native_overlay_map8_byte(a),Some(byte));
        assert_eq!(byte,rom[0x78000+i],"map8 table byte={i}");
    }
    for a in 0x02f889..0x02f88d {
        assert_eq!(state.native_overlay_map8_byte(a),Some(rom[0x10000+(a as usize&0x7fff)]));
    }
    assert!(state.rom.is_empty());
    eprintln!("native map8 original immutable bytes={}",bytes.len()+4);
}

#[test]
fn whole_module08_overlay_dispatch_and_publication_match_original_without_native_rom() {
    use crate::cycle_models::overworld_overlays_caller::OverworldOverlaysCaller;
    let state=super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom=super::runtime_native_overworld_dungeon_exit::rom();
    let streams=state.native_overworld_map_stream_data().unwrap();
    let mut boundaries=0;
    for case in 0..48usize {
        let screen=[0u16,3,5,7,0x43,0x70,0x1b,0x5b,0x80,0x81,0x90,0x9f][case%12];
        let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
        for (i,v) in machine.ram.iter_mut().enumerate() {*v=(i.wrapping_mul(37)^(i>>8)^case.wrapping_mul(19)) as u8;}
        crate::types::write_le_u16(&mut machine.ram,0x8a,screen);
        crate::types::write_le_u16(&mut machine.ram,0xa0,[0x180u16,0x181,0x189,0x182][case/12]);
        machine.ram[0xf300]=if case&1==0 {0} else {0x40};
        machine.ram[0xf2f0]=if case&1==0 {0} else {0x20};
        machine.ram[0xf3c5]=(case%3) as u8;
        machine.ram[0x10]=8;
        machine.ram[0x11]=1;
        machine.ram[0x1fd..0x200].copy_from_slice(&[0x59,0x80,0]);
        let mut cpu=NativeNmiContext {a:0xb701,x:0x81,y:0x42,d:0,
            db:[0,2,0x80,0x82][case%4],bank:2,pc:0x83bf,sp:0x1fc,
            p:0x30|[0,1,2,0x43,0x80,0xc3][case%6]};
        machine.cpu.a=cpu.a;machine.cpu.x=cpu.x;machine.cpu.y=cpu.y;
        machine.cpu.dp=cpu.d;machine.cpu.db=cpu.db;machine.cpu.k=cpu.bank;
        machine.cpu.pc=cpu.pc;machine.cpu.sp=cpu.sp;machine.cpu.e=false;machine.cpu.unpack_flags(cpu.p);
        let mut ram=machine.ram.clone();
        let mut published=state.clone();published.ram.copy_from_slice(&ram);published.sync_native_game_state_from_ram();
        let clock=CpuMasterTimeline::at_raster(case as u64,
            [snes::CpuRasterPosition::new(100,532),snes::CpuRasterPosition::new(224,1338),
             snes::CpuRasterPosition::new(261,1350)][case%3],
            snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        let mut source=snes::RomCpuTimingProbe::new(machine,clock,
            snes::SourcePpuReadState::snes9x_reset()).unwrap();let mut clock=source.timeline().clone();
        let mut caller=OverworldOverlaysCaller::default();
        for _ in 0..250000 {
            if caller.complete() {break;}
            let label=format!("whole overlay case={case} screen={screen:02x} pc={:04x}",cpu.pc);
            compare_instruction_with_devices(&label,&mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                |_,access| match access {
                    Access::Write(a,value,width)=>{published.native_overworld_overlays_store(a,value,width)
                        .unwrap_or_else(|offset|panic!("unowned overlay store {offset:05x}; {label}"));None}
                    Access::Read(a,width) if a<0x7e0000 && a&0x8000!=0 => {
                        let read=|a|crate::cycle_models::overworld_properties_caller::table_byte(a&0x7fffff)
                            .or_else(||state.native_map32_definition_byte(a))
                            .or_else(||state.native_overlay_map8_byte(a))
                            .or_else(||streams.get(&(a&0x7fffff)).copied())
                            .unwrap_or_else(||panic!("unmapped whole overlay data {a:06x}; {label}"));
                        Some(if width==1 {u16::from(read(a))} else {u16::from_le_bytes([read(a),read(a+1)])})
                    }
                    _=>None,
                },|cpu,clock,bus| {
                    caller.step(cpu,clock,bus).unwrap();
                });
            assert_eq!(ram,source.snes().ram,"{label}");
            assert_eq!(published.ram,ram,"overlay store mirror; {label}");boundaries+=1;
        }
        assert!(caller.complete());assert_eq!((cpu.bank,cpu.pc,cpu.sp),(0,0x805a,0x1ff));
        let mut expected=crate::game_state::GameState::load_from_ram(&ram);
        // The entrance sequence view does not own graphics scratch at C8.
        expected.world.overworld.entrance.sequence_counter=published.game_state.world.overworld.entrance.sequence_counter;
        assert_eq!(published.game_state.frame,expected.frame);
        assert_eq!(published.game_state.system_signals,expected.system_signals);
        assert_eq!(published.game_state.display,expected.display);
        assert_eq!(published.game_state.world,expected.world);
    }
    assert!(state.rom.is_empty());
    eprintln!("native whole Module08 overlay publication Source cases=48 instruction boundaries={boundaries}");
}

#[test]
fn full_overworld_quadrants_compose_real_decompression_interlace_and_definition_frames() {
    use crate::cycle_models::overworld_quadrants::OverworldQuadrants;
    let state=super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom=super::runtime_native_overworld_dungeon_exit::rom();
    let streams=state.native_overworld_map_stream_data().unwrap();
    let mut boundaries=0;
    for screen in 0..151u16 {
        let case=screen as usize;
        let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
        for (i,v) in machine.ram.iter_mut().enumerate() {*v=(i.wrapping_mul(37)^(i>>8)^case.wrapping_mul(19)) as u8;}
        crate::types::write_le_u16(&mut machine.ram,0x8a,screen);
        machine.ram[0x1f7..0x1f9].copy_from_slice(&[0xc9,0xee]);
        let mut cpu=NativeNmiContext {a:0xb701,x:0x95,y:0x9f7c,d:0,
            db:[0,2,0x80,0x82][case%4],bank:2,pc:0xf54a,sp:0x1f6,
            p:[0,1,2,0x43,0x80,0xc3][case%6]};
        let saved_db=cpu.db;
        machine.cpu.a=cpu.a;machine.cpu.x=cpu.x;machine.cpu.y=cpu.y;
        machine.cpu.dp=cpu.d;machine.cpu.db=cpu.db;machine.cpu.k=cpu.bank;
        machine.cpu.pc=cpu.pc;machine.cpu.sp=cpu.sp;machine.cpu.e=false;machine.cpu.unpack_flags(cpu.p);
        let mut ram=machine.ram.clone();let clock=CpuMasterTimeline::at_raster(case as u64,
            [snes::CpuRasterPosition::new(100,532),snes::CpuRasterPosition::new(224,1338),
             snes::CpuRasterPosition::new(261,1350)][case%3],
            snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        let mut source=snes::RomCpuTimingProbe::new(machine,clock,
            snes::SourcePpuReadState::snes9x_reset()).unwrap();let mut clock=source.timeline().clone();
        let mut work=OverworldQuadrants::default();
        for _ in 0..1000000 {
            if work.complete() {break;}
            let label=format!("full overworld quadrants screen={screen:02x} pc={:04x}",cpu.pc);
            compare_instruction_with_devices(&label,&mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                |_,access| match access {
                    Access::Read(a,width) if a<0x7e0000 && a&0x8000!=0 => {
                        let read=|a|state.native_map32_definition_byte(a)
                            .or_else(||streams.get(&(a&0x7fffff)).copied())
                            .unwrap_or_else(||panic!("unmapped composed map data {a:06x}; {label}"));
                        Some(if width==1 {u16::from(read(a))} else {u16::from_le_bytes([read(a),read(a+1)])})
                    }
                    _=>None,
                },|cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
            assert_eq!(ram,source.snes().ram,"{label}");boundaries+=1;
        }
        assert!(work.complete());assert_eq!((cpu.bank,cpu.pc,cpu.sp,cpu.db),(2,0xeeca,0x1f8,saved_db));
    }
    assert!(state.rom.is_empty());
    eprintln!("native full overworld quadrants Source cases=151 instruction boundaries={boundaries}");
}


#[test]
fn whole_module08_full_screen_build_matches_original_without_native_rom() {
    use crate::cycle_models::overworld_screen_build::OverworldScreenBuild;
    let state=super::runtime_native_overworld_dungeon_exit::data_owner();
    let rom=super::runtime_native_overworld_dungeon_exit::rom();
    let streams=state.native_overworld_map_stream_data().unwrap();
    let mut boundaries=0;
    for case in 0..24usize {
        let screen=[0u16,3,5,7,0x43,0x70,0x1b,0x5b,0x80,0x81,0x90,0x96][case%12];
        let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
        for (i,v) in machine.ram.iter_mut().enumerate() {*v=(i.wrapping_mul(37)^(i>>8)^case.wrapping_mul(19)) as u8;}
        crate::types::write_le_u16(&mut machine.ram,0x8a,screen);
        crate::types::write_le_u16(&mut machine.ram,0xa0,[0x180u16,0x181,0x189,0x182][case/12]);
        machine.ram[0xf300]=if case&1==0 {0} else {0x40};
        machine.ram[0xf2f0]=if case&1==0 {0} else {0x20};
        machine.ram[0xf3c5]=(case%3) as u8;
        crate::types::write_le_u16(&mut machine.ram,0x84,0x410);
        crate::types::write_le_u16(&mut machine.ram,0x86,0);
        crate::types::write_le_u16(&mut machine.ram,0x88,0);
        machine.ram[0x10]=8;
        machine.ram[0x11]=2;
        machine.ram[0xf280+screen as usize]=if case&1==0 || screen>=128 {0} else {2};
        crate::types::write_le_u16(&mut machine.ram,0x696,[0u16,0xffff,0x100,0x8100][case%4]);
        crate::types::write_le_u16(&mut machine.ram,0x4ac,0);
        machine.ram[0x1fd..0x200].copy_from_slice(&[0x59,0x80,0]);
        let mut cpu=NativeNmiContext {a:0xb701,x:0x81,y:0x42,d:0,
            db:[0,2,0x80,0x82][case%4],bank:2,pc:0x83bf,sp:0x1fc,
            p:0x30|[0,1,2,0x43,0x80,0xc3][case%6]};
        machine.cpu.a=cpu.a;machine.cpu.x=cpu.x;machine.cpu.y=cpu.y;
        machine.cpu.dp=cpu.d;machine.cpu.db=cpu.db;machine.cpu.k=cpu.bank;
        machine.cpu.pc=cpu.pc;machine.cpu.sp=cpu.sp;machine.cpu.e=false;machine.cpu.unpack_flags(cpu.p);
        let mut ram=machine.ram.clone();
        let mut published=state.clone();published.ram.copy_from_slice(&ram);published.sync_native_game_state_from_ram();
        
        let clock=CpuMasterTimeline::at_raster(case as u64,
            [snes::CpuRasterPosition::new(100,532),snes::CpuRasterPosition::new(224,1338),
             snes::CpuRasterPosition::new(261,1350)][case%3],
            snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        let mut source=snes::RomCpuTimingProbe::new(machine,clock,
            snes::SourcePpuReadState::snes9x_reset()).unwrap();let mut clock=source.timeline().clone();
        let mut caller=OverworldScreenBuild::default();
        for _ in 0..1000000 {
            if caller.complete() {break;}
            let label=format!("whole screen build case={case} screen={screen:02x} pc={:04x}",cpu.pc);
            compare_instruction_with_devices(&label,&mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                |_,access| match access {
                    Access::Write(a,value,width)=>{published.native_overworld_screen_build_store(a,value,width)
                        .unwrap_or_else(|offset|panic!("unowned screen build store {offset:05x}; {label}"));None}
                    Access::Read(a,width) if a<0x7e0000 && a&0x8000!=0 => {
                        let read=|a|crate::cycle_models::overworld_properties_caller::table_byte(a&0x7fffff)
                            .or_else(||state.native_overworld_screen_build_byte(a))
                            .or_else(||state.native_map32_definition_byte(a))
                            .or_else(||state.native_overlay_map8_byte(a))
                            .or_else(||streams.get(&(a&0x7fffff)).copied())
                            .unwrap_or_else(||panic!("unmapped whole overlay data {a:06x}; {label}"));
                        Some(if width==1 {u16::from(read(a))} else {u16::from_le_bytes([read(a),read(a+1)])})
                    }
                    _=>None,
                },|cpu,clock,bus| {
                    caller.step(cpu,clock,bus).unwrap();
                });
            assert_eq!(ram,source.snes().ram,"{label}");
            assert_eq!(published.ram,ram,"screen build store mirror; {label}");
            boundaries+=1;
        }
        assert!(caller.complete());assert_eq!((cpu.bank,cpu.pc,cpu.sp),(0,0x805a,0x1ff));
        let mut expected=crate::game_state::GameState::load_from_ram(&ram);
        expected.world.overworld.entrance.sequence_counter=published.game_state.world.overworld.entrance.sequence_counter;
        assert_eq!(published.game_state.frame,expected.frame);
        assert_eq!(published.game_state.system_signals,expected.system_signals);
        assert_eq!(published.game_state.display,expected.display);
        assert_eq!(published.game_state.world,expected.world);
        assert_eq!(published.game_state.memorized_tiles,expected.memorized_tiles);
        // Dungeon line pointers at BF..DF are inactive graphics scratch here.
        for index in 0..4096 {
            assert_eq!(published.game_state.dungeon.room_tilemaps.bg1_tile(index),expected.dungeon.room_tilemaps.bg1_tile(index));
            assert_eq!(published.game_state.dungeon.room_tilemaps.bg2_tile(index),expected.dungeon.room_tilemaps.bg2_tile(index));
        }

    }
    assert!(state.rom.is_empty());
    eprintln!("native whole Module08 full screen build Source cases=24 instruction boundaries={boundaries}");
}

