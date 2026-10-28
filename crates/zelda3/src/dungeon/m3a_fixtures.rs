//! CPU-only fixtures for equal-cost scalar replacements and entry aliases.
use super::*;

pub(super) fn source_work(ram: &[u8], entry: u32, rtl: bool, m8: bool, x8: bool,
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
        stack_bytes:if rtl { if bank==0 { &[0x35,0x80,0] } else { &[0x35,0x80,1] } } else { &[0x35,0x80] },
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
