use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn probe(stream: &[u8], pointer: u32) -> snes::RomCpuTimingProbe {
    use snes::{Snes, CartType, RomCpuTimingProbe, CpuMasterTimeline, CpuRasterPosition,
        CpuBusWorkload, CpuFieldTiming, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let mut s = Snes::new();
    s.cart.load(CartType::LoRom, &rom, 0x2000);
    // Synthetic command data lives in WRAM; every instruction byte remains
    // the unmodified cartridge. The byte reader also exercises bank wrapping.
    for (i, &v) in stream.iter().enumerate() {
        let offset = (pointer & 0x7fff) + i as u32;
        let at = (((pointer >> 16) + offset / 0x8000) << 16) | 0x8000 | (offset & 0x7fff);
        s.ram[(at & 0x1ffff) as usize] = v;
    }
    s.ram[0..3].copy_from_slice(&[0, 0x40, 0x7f]);
    s.ram[0xc8..0xcb].copy_from_slice(&pointer.to_le_bytes()[..3]);
    s.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
    s.cpu.e = false; s.cpu.unpack_flags(0x30);
    s.cpu.k = 2; s.cpu.db = 2; s.cpu.pc = 0xfebb; s.cpu.sp = 0x1fc;
    RomCpuTimingProbe::new(s, CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false)),
        SourcePpuReadState::snes9x_reset()).unwrap()
}

#[test]
fn prefix7_bank02_markers_match_every_command_loop_and_reader_edge() {
    let branches = [0x02fec5, 0x02fed0, 0x02fef4, 0x02fef6, 0x02fef9,
        0x02fefc, 0x02ff08, 0x02ff17, 0x02ff24, 0x02ff36, 0x02ff3d,
        0x02ff5a, 0x02ff64];
    let mut edges = BTreeSet::new();
    let mut cases = 0;
    for command in 0u8..8 {
        for length in [1usize, 2, 3, 31, 32, 33, 256, 257, 768, 1024] {
            for pointer in [0x7e9000u32, 0x7effff] {
                let mut stream = Vec::new();
                if length <= 32 && command < 7 {
                    stream.push(command << 5 | (length - 1) as u8);
                } else {
                    let header = 0xe0 | command << 2 | ((length - 1) >> 8) as u8;
                    if header == 0xff { continue; } // Cartridge terminator, not a command.
                    stream.extend([header, (length - 1) as u8]);
                }
                match command {
                    0 => stream.extend((0..length).map(|i| i as u8)),
                    1 | 3 => stream.push(0x7f),
                    2 => stream.extend([0x3f, 0x80]),
                    _ => stream.extend([0, 0]),
                }
                stream.push(0xff);
                let mut source = probe(&stream, pointer);
                let mut expected = BTreeMap::<u32, u64>::new();
                for _ in 0..100_000 {
                    let pc = 0x020000 | u32::from(source.snes().cpu.pc);
                    let step = source.step().unwrap();
                    let owner = if (0x02ff5f..0x02ff6e).contains(&pc) { 0x02ff5f } else { 0x02febb };
                    *expected.entry(owner).or_default() += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                    if branches.contains(&pc) {
                        edges.insert((pc, u32::from(source.snes().cpu.pc) != ((pc + 2) & 0xffff)));
                    }
                    if source.snes().cpu.pc == 0x8000 { break; }
                }
                assert_eq!(source.snes().cpu.pc, 0x8000);
                let mut decoded = vec![0; 0x1000];
                crate::cycle_ledger::profile_iteration(true);
                crate::cycle_ledger::profile_main_begin();
                let written = ZeldaState::decompress_bank02_into(&mut decoded, &stream, Some(pointer));
                crate::cycle_ledger::profile_main_end();
                let rows = crate::cycle_ledger::take_routine_profile();
                for (&owner, &clocks) in &expected {
                    assert_eq!(rows[&owner][2], clocks, "command={command} length={length} owner={owner:06x}");
                }
                assert_eq!(written, length);
                assert_eq!(&decoded[..written], &source.snes().ram[0x14000..0x14000 + written]);
                assert_eq!(rows.values().map(|r| r[2]).sum::<u64>(), expected.values().sum());
                cases += 1;
            }
        }
    }
    for pc in branches {
        for taken in [false, true] { assert!(edges.contains(&(pc, taken)), "uncovered {pc:06x}/{taken}"); }
    }
    println!("prefix7 bank02: {cases} unmodified-ROM cases; all 13 conditional edges both ways");
}
