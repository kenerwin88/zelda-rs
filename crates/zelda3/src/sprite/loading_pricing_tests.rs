use super::*;
use std::collections::BTreeSet;

fn machine(n: &ZeldaState, entry: u32) -> snes::Snes {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut s = snes::Snes::new();
    s.cart.load(snes::CartType::LoRom, &std::fs::read(root.join("saves/zelda3.sfc")).unwrap(), 0x2000);
    s.ram.copy_from_slice(&n.wram_image());
    s.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
    s.cpu.e = false; s.cpu.unpack_flags(0x30);
    s.cpu.k = 9; s.cpu.db = 9; s.cpu.pc = entry as u16; s.cpu.sp = 0x1fc;
    s
}

fn check(entry: u32, s: snes::Snes, edges: &mut BTreeSet<(u32, bool)>, native: impl FnOnce()) {
    use snes::{RomCpuTimingProbe, CpuMasterTimeline, CpuRasterPosition,
        CpuBusWorkload, CpuFieldTiming, SourcePpuReadState};
    let mut source = RomCpuTimingProbe::new(s, CpuMasterTimeline::at_raster(0,
        CpuRasterPosition::new(0, 0), CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false)),
        SourcePpuReadState::snes9x_reset()).unwrap();
    let (lo, hi) = match entry { 0x09c290 => (0xc290, 0xc2d5),
        0x09c327 => (0xc327, 0xc3e8), 0x09c3e8 => (0xc3e8, 0xc44e),
        0x09c770 => (0xc770, 0xc80b), 0x09c80b => (0xc80b, 0xc881),
        0x09c871 => (0xc871, 0xc881), _ => unreachable!() };
    let mut clocks = 0;
    let mut calls = Vec::new();
    let inputs = (source.snes().cpu.y, source.snes().ram[2],
        source.snes().ram[0x1df80..0x1df82].to_vec());
    let mut witness = Vec::new();
    for _ in 0..100_000 {
        let c = &source.snes().cpu;
        let pc = u32::from(c.k) << 16 | u32::from(c.pc);
        // Local PHY/PHX pushes are still caller work. Only JSR/JSL frames
        // exclude children; raw SP equality would drop the death-mask body.
        let owns = c.k == 9 && calls.is_empty() && (lo..hi).contains(&c.pc);
        if owns { witness.push((pc, c.a, c.x, c.y)); }
        let step = source.step().unwrap();
        if owns {
            clocks += step.transactions.iter().map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
            if matches!(step.opcode, 0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xb0 | 0xd0 | 0xf0) {
                edges.insert((pc, source.snes().cpu.pc != (pc + 2) as u16));
            }
        }
        let sp = source.snes().cpu.sp;
        if matches!(step.opcode, 0x20 | 0x22 | 0xfc) { calls.push(sp); }
        while calls.last().is_some_and(|&entered| sp > entered) { calls.pop(); }
        if source.snes().cpu.pc == 0x8000 { break; }
    }
    assert_eq!(source.snes().cpu.pc, 0x8000);
    crate::cycle_ledger::profile_iteration(true);
    crate::cycle_ledger::profile_main_begin();
    native();
    crate::cycle_ledger::profile_main_end();
    let rows = crate::cycle_ledger::take_routine_profile();
    assert_eq!(rows[&entry][2], clocks, "owner={entry:06x} inputs={inputs:x?} source={witness:x?}");
}

fn require_edges(edges: &BTreeSet<(u32, bool)>, branches: &[u32]) {
    for &pc in branches {
        for taken in [false, true] { assert!(edges.contains(&(pc, taken)), "uncovered {pc:06x}/{taken}"); }
    }
}

#[test]
fn prefix8_overworld_record_markers_cover_allocations_and_shared_initialization() {
    let mut edges = BTreeSet::new();
    let mut cases = 0;
    for ty in [1u8, 0x41, 0x53, 0x58, 0xd0, 0xeb, 0xf3, 0xf4, 0xf5, 0xff] {
        for free in [-1i32, 0, 4, 5, 7, 13, 14] {
            for reuse in [0u8, 1, 2] {
                let mut n = ZeldaState::new();
                n.set_lag_free(true);
                for slot in 0..16 {
                    let mut v = n.sprite_slot_view_mut(slot);
                    v.set_state(9); v.set_sprite_type(if reuse == 2 { 0x41 } else { 1 });
                }
                for slot in 0..8 { n.overlord_slot_view_mut(slot).set_overlord_type(1); }
                if free >= 0 {
                    let slot = free as usize;
                    let mut v = n.sprite_slot_view_mut(slot);
                    if reuse == 1 { v.set_sprite_type(0x41); v.set_c(1); }
                    else { v.set_state(0); }
                    if slot < 8 { n.overlord_slot_view_mut(slot).clear(); }
                }
                for entry in [0x09c770, 0x09c80b] {
                    if entry == 0x09c80b && ty < 0xf4 { continue; }
                    let mut s = machine(&n, entry);
                    s.cpu.a = u16::from(ty);
                    s.cpu.y = 3;
                    s.ram[0..2].copy_from_slice(&0x1123u16.to_le_bytes());
                    s.ram[2..5].copy_from_slice(&[1, 0x10, 0]);
                    s.ram[5..8].copy_from_slice(&[0, 0x10, 0]);
                    s.ram[0x1000] = ty;
                    check(entry, s, &mut edges, || {
                        if entry == 0x09c770 {
                            let _scope = crate::cycle_ledger::routine(entry);
                            n.price_overworld_sprite_record(ty);
                        } else { n.price_overworld_overlord_record(ty); }
                    });
                    cases += 1;
                }
            }
        }
    }
    // C78A compares #$58 after C77E already rejected it: taken is unreachable.
    assert!(edges.contains(&(0x09c78a, false)));
    assert!(!edges.contains(&(0x09c78a, true)));
    require_edges(&edges, &[0x09c774, 0x09c77e, 0x09c784, 0x09c78e,
        0x09c792, 0x09c796, 0x09c79d, 0x09c7a4, 0x09c7a9,
        0x09c7ac, 0x09c810, 0x09c813, 0x09c83d]);
    for slot in 0..8 {
        let mut n = ZeldaState::new();
        n.set_lag_free(true);
        let mut s = machine(&n, 0x09c871);
        s.cpu.x = slot;
        check(0x09c871, s, &mut edges, || n.price_overlord_initialization());
        cases += 1;
    }
    println!("prefix8 C770/C80B/C871: {cases} unmodified-ROM cases; all reachable branch pairs");
}

#[test]
fn prefix7_dungeon_record_markers_cover_all_gates_and_cursor_crossings() {
    let mut edges = BTreeSet::new();
    let mut cases = 0;
    for slot in [1usize, 3, 15] {
        for y in [0u8, 0x7f, 0x80, 0xfd, 0xfe] {
            for x in [0x20u8, 0xdf, 0xe0] {
                for ty in [0u8, 1, 3, 9, 10, 11, 0x2f, 0xe4] {
                    for mask in [0u16, 0xffff] {
                        for record in [1u8, 0xfd, 0xfe, 0xff] {
                            let mut n = ZeldaState::new();
                            n.set_indoor_flag(1); // This WRAM region owns death masks indoors.
                            n.set_sprite_where_in_room_mask(0, mask);
                            n.set_lag_free(true);
                            let pointer = 0x10f8u16;
                            let mut s = machine(&n, 0x09c327);
                            s.ram[0..2].copy_from_slice(&pointer.to_le_bytes());
                            s.ram[2] = slot as u8;
                            s.cpu.y = u16::from(record);
                            for (i, value) in [y, x, ty].into_iter().enumerate() {
                                s.ram[usize::from(pointer) + usize::from(record.wrapping_add(i as u8))] = value;
                            }
                            check(0x09c327, s, &mut edges, ||
                                n.price_dungeon_sprite_record(slot, &[y, x, ty], pointer, record));
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    require_edges(&edges, &[0x09c32e, 0x09c338, 0x09c33c, 0x09c355, 0x09c363, 0x09c380]);
    println!("prefix7 C327: {cases} unmodified-ROM cases; all six conditional edges both ways");
}

#[test]
fn prefix7_dungeon_overlord_markers_cover_search_and_type_tails() {
    let mut edges = BTreeSet::new();
    let mut cases = 0;
    for free in [-1i32, 0, 3, 7] {
        for ty in [1u8, 3, 10, 11, 20] {
            for record in [1u8, 0xfd, 0xfe, 0xff] {
                let mut n = ZeldaState::new();
                for slot in 0..8 { n.overlord_slot_view_mut(slot).set_overlord_type(if slot as i32 == free { 0 } else { 1 }); }
                n.set_lag_free(true);
                let pointer = 0x10f8u16;
                let mut s = machine(&n, 0x09c3e8);
                s.ram[0..2].copy_from_slice(&pointer.to_le_bytes());
                s.cpu.y = u16::from(record.wrapping_add(2));
                for (i, v) in [0x83, 0xe4, ty].into_iter().enumerate() {
                    s.ram[usize::from(pointer) + usize::from(record.wrapping_add(i as u8))] = v;
                }
                check(0x09c3e8, s, &mut edges, || n.price_dungeon_overlord_record(&[0x83, 0xe4, ty], pointer, record));
                cases += 1;
            }
        }
    }
    require_edges(&edges, &[0x09c3ed, 0x09c3f0, 0x09c42f, 0x09c433, 0x09c437]);
    println!("prefix7 C3E8: {cases} unmodified-ROM cases; all five conditional edges both ways");
}

#[test]
fn prefix7_dungeon_list_markers_match_every_cartridge_room_excluding_children() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets = std::fs::read(root.join("zelda3_assets.dat")).unwrap();
    let mut edges = BTreeSet::new();
    for room in 0..crate::cycle_models::sprite_reset_data::ROOM_POINTERS.len() {
        for mask in [0u16, 0xffff] {
            let mut n = ZeldaState::new();
            n.set_assets(&assets).unwrap();
            n.set_indoor_flag(1);
            n.write_wram_byte(0x48e, room as u8);
            n.write_wram_byte(0x48f, (room >> 8) as u8);
            n.sync_native_game_state_from_ram();
            n.set_sprite_where_in_room_mask(room as u16, mask);
            n.set_lag_free(true);
            let s = machine(&n, 0x09c290);
            check(0x09c290, s, &mut edges, || n.dungeon_load_sprites());
        }
    }
    require_edges(&edges, &[0x09c2c5]);
    println!("prefix7 C290: every cartridge room and both death masks; both terminator edges");
}
