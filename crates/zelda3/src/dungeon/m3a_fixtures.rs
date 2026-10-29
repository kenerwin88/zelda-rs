//! CPU-only fixtures for equal-cost scalar replacements and entry aliases.
use super::*;

pub(crate) fn source_work(ram: &[u8], entry: u32, rtl: bool, m8: bool, x8: bool,
    a: u16, x: u16, y: u16) -> u64 {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let bank = (entry >> 16) as u8;
    let checkpoint = RomCpuCheckpoint {
        entry_pc: entry, stop_pc: (entry & 0xff0000) | 0x8036,
        a,x,y, sp: if rtl {0x1fb} else {0x1fc}, dp:0,db:0,
        carry:false,zero:false,overflow:false,negative:false,
        interrupt_disable:true,decimal:false,accumulator_is_8_bit:m8,index_is_8_bit:x8,
        emulation:false,waiting:false,stack_address:if rtl {0x1fc} else {0x1fd},
        stack_bytes:if rtl { match bank {
            0=>&[0x35,0x80,0],1=>&[0x35,0x80,1],9=>&[0x35,0x80,9],
            _=>panic!("add a static return-bank fixture for {bank:02x}"),
        } } else { &[0x35,0x80] },
    };
    let mut run = RomCpuTimingRun::new(&rom,ram,&[0;0x2000],
        &snes::PpuState::default(),&snes::DmaState::default(),[0;4],checkpoint).unwrap();
    let mut clocks=0;let mut count=0;
    while !run.is_complete() {
        clocks+=u64::from(run.step().master_cycles);count+=1;
        assert!(count<1_000_000,"fixture did not return at {entry:06x}");
    }
    clocks
}

#[test]
fn m3b_nx4_entry_and_shared_body_match_rom() {
    for (entry,columns) in [(0x0197f0,1),(0x0197f0,2),(0x0197f0,3),(0x0197f0,4),(0x0197f0,16),(0x0197f0,32),(0x0197ed,4)] {
        for dst in [0, 0x100, 0x800] {
            let mut native=ZeldaState::new();
            native.set_assets(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat")).unwrap()).unwrap();
            let mut ram=native.wram_image().into_owned();
            for (i,offset) in [0xbf,0xcb,0xd7,0xda].into_iter().enumerate() {
                let address=0x7e2000u32+i as u32*128;
                ram[offset..offset+3].copy_from_slice(&address.to_le_bytes()[..3]);
            }
            let expected=source_work(&ram,entry,false,false,false,columns,0,dst);
            for lagless in [false,true] {
                native.set_lag_free(lagless);
                let before=crate::cycle_ledger::master();
                if entry==0x0197ed { native.room_draw_4x4_by_jump(0,dst/2); }
                else { native.room_draw_object_nx4(columns,0,dst/2); }
                assert_eq!(crate::cycle_ledger::master()-before,expected);
                if entry==0x0197f0 {
                    let before=crate::cycle_ledger::master();
                    native.room_data_draw_object_nx4(0,dst/2,columns);
                    assert_eq!(crate::cycle_ledger::master()-before,expected);
                }
            }
        }
    }
}

#[test]
fn m3a_fixed_floor_n8_matches_rom_for_both_planes() {
    for plane in [ROOM_BG2_TILEMAP_BASE, ROOM_BG1_TILEMAP_BASE] {
        for dst in [0, 8, 0x800, 0x1000] {
            let mut state=ZeldaState::new();state.set_lag_free(true);
            let mut ram=state.wram_image().into_owned();
            for (i,offset) in [0xbf,0xc2,0xc5,0xc8,0xcb,0xce,0xd1,0xd4].into_iter().enumerate() {
                let address=0x7e0000u32+plane as u32+(i as u32%4)*0x80+(i as u32/4)*2;
                ram[offset..offset+3].copy_from_slice(&address.to_le_bytes()[..3]);
            }
            let expected=source_work(&ram,0x018a44,false,false,false,8,0,dst);
            for lagless in [false,true] {
                let mut state=state.clone();state.set_lag_free(lagless);
                let before=crate::cycle_ledger::master();
                state.room_draw_many_32x32_blocks(plane,0,&[0;32],usize::from(dst));
                assert_eq!(crate::cycle_ledger::master()-before,expected);
            }
            assert_eq!(expected,16_946);
        }
    }
}

#[test]
fn m3a_quadrant_copy_entries_and_layer_widths_match_rom() {
    for entry in [0x00913f,0x0091d3,0x0091c4] {
        for layer in 0..16u16 {
            for quadrant in [0,4,8,12] {
                let mut state=ZeldaState::new();state.set_lag_free(true);
                state.ram[0x418..0x41a].copy_from_slice(&layer.to_le_bytes());
                state.ram[0x45c]=quadrant;
                state.sync_native_game_state_from_ram();
                let expected=source_work(&state.wram_image(),entry,true,true,true,0,0,0);
                for lagless in [false,true] {
                    let mut state=state.clone();state.set_lag_free(lagless);
                    let before=crate::cycle_ledger::master();
                    match entry {
                        0x00913f=>state.dungeon_prepare_next_room_quadrant_upload(),
                        0x0091d3=>state.tile_map_prep_not_water_on_tag(),
                        _=>state.water_flood_build_one_quadrant_for_vram(),
                    }
                    assert_eq!(crate::cycle_ledger::master()-before,expected,
                        "entry={entry:06x} layer={layer} quadrant={quadrant} lagless={lagless}");
                }
            }
        }
    }
}

#[test]
fn system1_table_helper_real_entries_match_rom_and_exclude_fallthrough() {
    for width in 1..=4u16 {
        let mut native=ZeldaState::new();
            native.set_assets(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat")).unwrap()).unwrap();native.set_lag_free(true);
        let mut ram=native.wram_image().into_owned();ram[0xb2..0xb4].copy_from_slice(&width.to_le_bytes());
        for (i,offset) in [0xbf,0xc2,0xc5].into_iter().enumerate() {
            let address=0x7e2000u32+i as u32*2;
            ram[offset..offset+3].copy_from_slice(&address.to_le_bytes()[..3]);
        }
        for entry in [0x0193ff,0x0193f7] {
            let work=source_work(&ram,entry,false,false,false,0,0,0);
            assert_eq!(work,166+u64::from(width)*330+u64::from(width-1)*6+260+if entry==0x193f7 {112}else{0});
            let mut s=snes::Snes::new();
            let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            s.cart.load(snes::CartType::LoRom,&std::fs::read(root.join("saves/zelda3.sfc")).unwrap(),0x2000);
            s.ram.copy_from_slice(&ram);
            s.cpu.e=false;s.cpu.unpack_flags(0);s.cpu.k=1;s.cpu.db=0;
            s.cpu.pc=entry as u16;s.cpu.sp=0x1fd;
            s.ram[0x1fe..0x200].copy_from_slice(&[0xff,0x7f]);
            let before=crate::cycle_ledger::master();
            crate::cycle_ledger::assert_rom_marker_work(entry,s,
                |s| s.cpu.k==1 && (0x93f7..0x9429).contains(&s.cpu.pc),
                || native.object_table_helper_observed(0,0,width,entry));
            assert_eq!(crate::cycle_ledger::master()-before,work);
            for lagless in [false,true] {
                native.set_lag_free(lagless);
                let before=crate::cycle_ledger::master();
                native.object_table_helper(0,0,width);
                assert_eq!(crate::cycle_ledger::master(),before);
                if !lagless {
                    native.object_table_helper_observed(0,0,width,entry);
                    assert_eq!(crate::cycle_ledger::master(),before);
                }
            }
            let before=crate::cycle_ledger::master();
            crate::cycle_ledger::muted(|| native.object_table_helper_observed(0,0,width,entry));
            assert_eq!(crate::cycle_ledger::master(),before);
        }
        for height in 0..4u16 {
            ram[0xb2..0xb4].copy_from_slice(&(width-1).to_le_bytes());
            ram[0xb4..0xb6].copy_from_slice(&height.to_le_bytes());
            let work=source_work(&ram,0x0193dc,false,false,false,0,0,0);
            let loops=u64::from(height)*2+1;
            let helper=166+u64::from(width)*330+u64::from(width-1)*6+260;
            // Caller owns both INX prefixes and the final fallthrough body;
            // the actual child calls are loops+1 FF entries and one F7 entry.
            let children=(loops+2)*helper+112;
            assert_eq!(work,children+helper+162+46+112+loops*116+(loops-1)*6+46+112);
        }
    }
}

#[test]
fn system1_pushable_block_scalar_replacement_matches_rom_self() {
    for dst in [0u16, 0x100, 0x2000, 0x4000] {
        let mut native=ZeldaState::new();
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        native.set_assets(&std::fs::read(root.join("zelda3_assets.dat")).unwrap()).unwrap();
        native.set_lag_free(true);
        let mut s=snes::Snes::new();
        s.cart.load(snes::CartType::LoRom,&std::fs::read(root.join("saves/zelda3.sfc")).unwrap(),0x2000);
        s.ram.copy_from_slice(&native.wram_image());
        for (i,offset) in [0xbf,0xcb,0xc2,0xce].into_iter().enumerate() {
            let address=0x7e2000u32+i as u32*128;
            s.ram[offset..offset+3].copy_from_slice(&address.to_le_bytes()[..3]);
        }
        s.cpu.e=false;s.cpu.unpack_flags(0);s.cpu.k=1;s.cpu.db=1;
        s.cpu.pc=0xb4d6;s.cpu.sp=0x1fd;s.cpu.y=dst;
        s.ram[0x1fe..0x200].copy_from_slice(&[0xff,0x7f]);
        crate::cycle_ledger::assert_rom_marker_work(0x01b4d6,s,
            |s|s.cpu.k==1 && (0xb4d6..0xb509).contains(&s.cpu.pc),
            ||native.draw_objects_pushable_block(dst,0));
    }
}
