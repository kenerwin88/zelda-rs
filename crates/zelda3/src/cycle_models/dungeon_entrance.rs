//! Dungeon entrance selection, overworld exit cache and room scratch reset.
//! Each store retires separately; table indices, DB and the return stack stay
//! live through the ordinary and saved-spawn paths and their shared tail.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonEntrance {
    pc: u16,
    complete: bool,
}
impl Default for DungeonEntrance {
    fn default() -> Self {
        Self {
            pc: 0xd8b3,
            complete: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    fn native_tables() -> crate::ZeldaState {
        let mut state = crate::ZeldaState::new();
        let path = std::env::var_os("ZELDA3_ASSETS").expect("entrance test requires native assets");
        state.set_assets(&std::fs::read(path).unwrap()).unwrap();
        state
    }

    #[test]
    fn dungeon_entrance_native_assets_match_source_data() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let state = native_tables();
        let mut bytes = 0;
        for range in [0x02c813..0x02d8b3, 0x02db6e..0x02dc55, 0x04f1de..0x04f4ea] {
            for address in range {
                assert_eq!(
                    state.native_dungeon_entrance_table_byte(address),
                    Some(rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)]),
                    "entrance asset byte {address:06x}"
                );
                bytes += 1;
            }
        }
        assert_eq!(bytes, 5267);
    }

    #[test]
    fn dungeon_entrance_matches_source_normal_spawn_and_reset_paths() {
        let tables = native_tables();
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut fixtures = Vec::new();
        // Selection is live: follower==4 (word), restart!=0 (word), and the
        // game-over byte independently control the two paths and exit backup.
        for entry in 0..133u16 {
            for progress in [0u16, 1, 2, 0x100] {
                for game_over in [0u8, 1] {
                    fixtures.push((entry, 0, progress, game_over, 0u16, 0u16));
                }
            }
        }
        for spawn in 0..7u8 {
            for (follower, restart) in [(4, 0), (0, 1), (0x104, 0x100)] {
                for progress in [0u16, 1, 2, 0x100] {
                    for game_over in [0u8, 255] {
                        fixtures.push((0x84, spawn, progress, game_over, follower, restart));
                    }
                }
            }
        }
        // A nonzero follower high byte must not spuriously select spawn 4.
        fixtures.push((0x43, 6, 2, 0, 0x104, 0));
        let mut total = 0;
        let mut visited = std::collections::BTreeSet::new();
        for (case, (entry, spawn, progress, game_over, follower, restart)) in
            fixtures.iter().copied().enumerate()
        {
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
            }
            machine.ram[0x10a] = game_over;
            machine.ram[0x10b] = 0x5b;
            machine.ram[0x10e..0x110].copy_from_slice(&(0xa500 | entry).to_le_bytes());
            machine.ram[0x4aa..0x4ac].copy_from_slice(&restart.to_le_bytes());
            machine.ram[0xf3cc..0xf3ce].copy_from_slice(&follower.to_le_bytes());
            machine.ram[0xf3c5..0xf3c7].copy_from_slice(&progress.to_le_bytes());
            machine.ram[0xf3c8] = spawn;
            machine.ram[0x1fc..0x1fe].copy_from_slice(&[0x4c, 0x82]);
            let mut cpu = NativeNmiContext {
                a: 0xab19,
                x: 0x71,
                y: 0x83,
                d: 0,
                db: if case & 2 == 0 { 0 } else { 2 },
                bank: 2,
                pc: 0xd8b3,
                sp: 0x1fb,
                p: 0x31,
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
            let mut state = crate::ZeldaState::new();
            state.ram.copy_from_slice(&ram);
            state.sync_native_game_state_from_ram();
            let (v, h) = if case & 1 == 0 {
                (224, 1338)
            } else {
                (261, 1350)
            };
            let mut clock = CpuMasterTimeline::at_raster(
                0,
                CpuRasterPosition::new(v, h),
                CpuBusWorkload::default(),
                CpuFieldTiming::non_interlace(v == 261),
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = DungeonEntrance::default();
            assert_eq!(work.routine(), 0x02d8b3);
            let mut count = 0;
            while !work.complete() {
                visited.insert(cpu.pc);
                let label = format!("entrance case{case} pc{:04x}", cpu.pc);
                super::super::native_work::compare_instruction_with_devices(
                    &label,
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    None,
                    |_, access| match access {
                        Access::Write(address, value, width) => {
                            let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                                address & 65535
                            } else { address };
                            if !state.native_dungeon_entrance_store(address, value, width) {
                                assert!((0x100..=0x1ff).contains(&address), "{label}: unowned entrance store {address:06x}");
                                state.ram[address as usize..address as usize + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            }
                            None
                        }
                        Access::Read(address, width)
                            if address & 0x8000 != 0 && address < 0x7e0000 =>
                        {
                            let mut value = 0;
                            for byte in 0..width {
                                let address = address + u32::from(byte);
                                value |= u16::from(
                                    tables
                                        .native_dungeon_entrance_table_byte(address)
                                        .unwrap_or_else(|| {
                                            panic!("unbound entrance data {address:06x}")
                                        }),
                                ) << (byte * 8);
                            }
                            Some(value)
                        }
                        _ => None,
                    },
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                assert_eq!(ram, source.snes().ram, "{label}");
                assert_eq!(state.ram, ram, "native publication: {label}");
                state.game_state.world.assert_entrance_matches_ram(&ram);
                state.game_state.dungeon.assert_entrance_matches_ram(&ram);
                assert_eq!(state.game_state.player, crate::game_state::PlayerState::load_from_ram(&ram), "{label}");
                assert_eq!(state.game_state.system_signals, crate::game_state::SystemSignalsState::load_from_ram(&ram), "{label}");
                assert_eq!(state.game_state.inventory, crate::game_state::InventoryState::load_from_ram(&ram), "{label}");
                assert_eq!(state.game_state.display.ppu_scroll_copy, crate::game_state::PpuScrollCopyState::load_from_ram(&ram), "{label}");
                for offset in (0..0x100).step_by(2) {
                    assert_eq!(state.game_state.memorized_tiles.entry_addr(offset / 2), crate::types::read_le_u16(&ram, 0xf800 + offset), "{label}");
                }
                work = work.clone();
                count += 1;
                assert!(count < 5000, "{label}");
            }
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                (2, 0x824d, 0x1fd, if case & 2 == 0 { 0 } else { 2 })
            );
            total += count;
        }
        assert_eq!(
            visited.len(),
            392,
            "all reachable native entrance instructions exercised"
        );
        eprintln!("{} dungeon entrance fixtures matched {total} instructions and native publications; {} distinct continuation positions",fixtures.len(),visited.len());
    }
}
impl DungeonEntrance {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x02d8b3
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (2, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let index_mask = if x == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0xd8b3 | 0xd8b4 | 0xdaf0 | 0xdaf3 => {
                w.drain(6);
                w.push(
                    if self.pc == 0xd8b4 {
                        u16::from(w.cpu.bank)
                    } else if self.pc == 0xdaf3 {
                        w.cpu.a & mask
                    } else {
                        u16::from(w.cpu.db)
                    },
                    1,
                );
                self.pc + 1
            }
            0xd8b5 | 0xdaef | 0xdaf4 | 0xdb6c => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xd8c5 | 0xd95e | 0xd97c | 0xda18 | 0xdaed | 0xdaf5 | 0xdb6a | 0xdcda => {
                w.width(
                    if matches!(self.pc, 0xd97c | 0xdaed | 0xdb6a) {
                        0x30
                    } else {
                        0x20
                    },
                    matches!(self.pc, 0xd8c5 | 0xd97c | 0xdaf5),
                );
                self.pc + 2
            }
            0xd8c2 | 0xd995 | 0xdd87 => {
                w.drain(16);
                match self.pc {
                    0xd8c2 => 0xd97c,
                    0xd995 => 0xdc55,
                    _ => 0xdadb,
                }
            }
            0xdb6d => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            0xd8b6 | 0xd9eb | 0xd9f9 | 0xd9ff | 0xda05 | 0xda0b | 0xda50 | 0xda64 | 0xda70
            | 0xda8c | 0xdadb | 0xdae1 | 0xdae7 | 0xdaf1 | 0xdb32 | 0xdcaf | 0xdcba | 0xdcc0
            | 0xdcc6 | 0xdccc | 0xdd12 | 0xdd26 | 0xdd7f => {
                let v = match self.pc {
                    0xd8b6 => 1,
                    0xd9eb | 0xdcaf => 0x1f8,
                    0xd9ff | 0xdcc0 => 0x110,
                    0xda0b | 0xdccc => 0x100,
                    0xda50 | 0xdd12 => 16,
                    0xda64 | 0xdd26 => 2,
                    0xda8c => 18,
                    0xdadb => 0x80,
                    0xdae1 => 15,
                    0xdae7 | 0xdd7f => 255,
                    0xdaf1 => 0x7e,
                    _ => 0,
                };
                w.immediate(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xdaf7 | 0xdb30 => {
                w.drain(8 * u32::from(x));
                w.cpu.x = if self.pc == 0xdb30 { 0x3e } else { 0 };
                w.flags(w.cpu.x, x);
                self.pc + 1 + u16::from(x)
            }
            0xd98b | 0xda80 | 0xda88 => {
                w.compare_immediate(
                    match self.pc {
                        0xd98b => 4,
                        0xda80 => 3,
                        _ => 2,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xda66 | 0xda6b | 0xdad4 | 0xdb2c | 0xdd74 => {
                w.compare_x_immediate(
                    match self.pc {
                        0xda6b => 0x43,
                        0xdad4 => 0x100,
                        0xdb2c => 0x80,
                        _ => 0,
                    },
                    x,
                );
                self.pc + 1 + u16::from(x)
            }
            0xd99b | 0xda14 | 0xdaad | 0xdabe | 0xdace | 0xdc59 | 0xdcd6 | 0xdd49 | 0xdd5a
            | 0xdd6a => {
                w.mask(
                    if matches!(self.pc, 0xd99b | 0xda14 | 0xdc59 | 0xdcd6) {
                        255
                    } else {
                        15
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xd99e | 0xd9a0 | 0xd9a1 | 0xdc5c | 0xdc5e | 0xdc5f => {
                w.shift_left(m);
                self.pc + 1
            }
            0xdaa4..=0xdaa7
            | 0xdab5..=0xdab8
            | 0xdac5..=0xdac8
            | 0xdd40..=0xdd43
            | 0xdd51..=0xdd54
            | 0xdd61..=0xdd64 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | ((w.cpu.a & 1) as u8);
                w.load((w.cpu.a & mask) >> 1, m);
                self.pc + 1
            }
            0xd9db | 0xd9dc | 0xd9e6 | 0xd9e7 | 0xdc9f | 0xdca0 | 0xdcaa | 0xdcab => {
                w.increment_a(m);
                self.pc + 1
            }
            0xd99f | 0xda17 | 0xdc5d | 0xdcd9 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd9a2 | 0xdc60 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xdb2a | 0xdb2b | 0xdb5f | 0xdb60 => {
                w.drain(6);
                w.cpu.x = if self.pc < 0xdb30 {
                    w.cpu.x.wrapping_add(1)
                } else {
                    w.cpu.x.wrapping_sub(1)
                } & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd8bd | 0xd98e | 0xd993 | 0xd9c9 | 0xda69 | 0xda6e | 0xda82 | 0xda8a | 0xdad7
            | 0xdb2e | 0xdb61 | 0xdc8d | 0xdd77 | 0xdd7d => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, to) = match self.pc {
                    0xd8bd => (z, 0xd8c5),
                    0xd98e => (z, 0xd995),
                    0xd993 => (z, 0xd998),
                    0xd9c9 => (z, 0xd9d5),
                    0xda69 | 0xda6e => (z, 0xda72),
                    0xda82 => (!z, 0xda91),
                    0xda8a => (!c, 0xda91),
                    0xdad7 => (!c, 0xdadb),
                    0xdb2e => (!z, 0xdaf9),
                    0xdb61 => (w.cpu.p & NEGATIVE == 0, 0xdb35),
                    0xdc8d => (z, 0xdc99),
                    _ => (!z, 0xdd84),
                };
                if w.branch(take) {
                    to
                } else {
                    self.pc + 2
                }
            }
            0xd8ce | 0xd8d4 | 0xd8da | 0xd8e0 | 0xd8e6 | 0xd8fa | 0xd900 | 0xdad2 => {
                let a = match self.pc {
                    0xd8ce => 0x1c,
                    0xd8d4 => 0xe8,
                    0xd8da => 0xe2,
                    0xd8e0 => 0x20,
                    0xd8e6 => 0x22,
                    0xd8fa => 0x8a,
                    0xd900 => 0x84,
                    _ => 0xa0,
                };
                let v = w.dp_read(a, if self.pc == 0xdad2 { x } else { m });
                if self.pc == 0xdad2 {
                    w.cpu.x = v;
                    w.flags(v, x);
                } else {
                    w.load(v, m);
                }
                self.pc + 2
            }
            0xd8b8 | 0xd9a6 | 0xd9ae | 0xd9b0 | 0xd9bb | 0xd9bd | 0xd9ce | 0xd9d3 | 0xd9ee
            | 0xda72 | 0xda94 | 0xda9f | 0xdaa8 | 0xdab9 | 0xdac0 | 0xdac9 | 0xdad0 | 0xdadd
            | 0xdadf | 0xdae3 | 0xdae5 | 0xdae9 | 0xdaeb | 0xdc6a | 0xdc72 | 0xdc74 | 0xdc7f
            | 0xdc81 | 0xdc92 | 0xdc97 | 0xdcb2 | 0xdd28 | 0xdd33 | 0xdd44 | 0xdd55 | 0xdd5c
            | 0xdd65 | 0xdd6c => {
                let a = match self.pc {
                    0xd8b8 => 0x1b,
                    0xd9a6 | 0xdc6a => 0xa0,
                    0xd9ae | 0xdc72 => 0xe8,
                    0xd9b0 | 0xdc74 => 0xe6,
                    0xd9bb | 0xdc7f => 0xe2,
                    0xd9bd | 0xdc81 => 0xe0,
                    0xd9ce | 0xdc92 => 0x20,
                    0xd9d3 | 0xdc97 => 0x22,
                    0xd9ee | 0xdcb2 => 0xec,
                    0xda72 | 0xdd28 => 0x2f,
                    0xda94 | 0xdd33 => 0xa4,
                    0xda9f => 0x6c,
                    0xdaa8 | 0xdd44 => 0xee,
                    0xdab9 | 0xdd55 => 0xa6,
                    0xdac0 | 0xdd5c => 0xa7,
                    0xdac9 | 0xdd65 => 0xa9,
                    0xdad0 | 0xdd6c => 0xaa,
                    0xdadd => 0x45,
                    0xdadf => 0x44,
                    0xdae3 => 0x42,
                    0xdae5 => 0x43,
                    0xdae9 => 0x24,
                    _ => 0x29,
                };
                w.dp_store(a, w.cpu.a & mask, m);
                self.pc + 2
            }
            0xd906 | 0xd908 | 0xdad9 | 0xdd3b => {
                w.dp_store(
                    match self.pc {
                        0xd906 => 0x8a,
                        0xd908 => 0x8c,
                        0xdad9 => 0xa4,
                        _ => 0x6c,
                    },
                    0,
                    m,
                );
                self.pc + 2
            }
            0xd8ba | 0xd8c7 | 0xd8ec | 0xd8f3 | 0xd90a | 0xd911 | 0xd918 | 0xd91f | 0xd926
            | 0xd92d | 0xd934 | 0xd93b | 0xd942 | 0xd949 | 0xd950 | 0xd957 | 0xd960 | 0xd967
            | 0xd96e | 0xd975 | 0xd990 | 0xd998 | 0xda11 => {
                let a = match self.pc {
                    0xd8ba => 0x10a,
                    0xd8c7 => 0x40a,
                    0xd8ec => 0x618,
                    0xd8f3 => 0x61c,
                    0xd90a => 0x600,
                    0xd911 => 0x602,
                    0xd918 => 0x604,
                    0xd91f => 0x606,
                    0xd926 => 0x610,
                    0xd92d => 0x612,
                    0xd934 => 0x614,
                    0xd93b => 0x616,
                    0xd942 => 0x624,
                    0xd949 => 0x626,
                    0xd950 => 0x628,
                    0xd957 => 0x62a,
                    0xd960 => 0xaa0,
                    0xd967 => 0xaa1,
                    0xd96e => 0xaa2,
                    0xd975 => 0xaa3,
                    0xd990 => 0x4aa,
                    _ => 0x10e,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), m);
                w.load(v, m);
                self.pc + 3
            }
            0xd8bf | 0xd97e | 0xd981 | 0xd984 | 0xd9f6 | 0xda4a | 0xda4d | 0xda58 | 0xda5b
            | 0xda5e | 0xda61 | 0xdb67 | 0xdd0c | 0xdd0f | 0xdd1a | 0xdd1d | 0xdd20 | 0xdd23
            | 0xdd84 => {
                let a = match self.pc {
                    0xd8bf | 0xd984 => 0x10a,
                    0xd97e => 0x11a,
                    0xd981 => 0x11c,
                    0xd9f6 => 0x698,
                    0xda4a | 0xdd0c => 0x600,
                    0xda4d | 0xdd0f => 0x602,
                    0xda58 | 0xdd1a => 0x608,
                    0xda5b | 0xdd1d => 0x60a,
                    0xda5e | 0xdd20 => 0x60c,
                    0xda61 | 0xdd23 => 0x60e,
                    0xdb67 => 0x4bc,
                    _ => 0x4aa,
                };
                w.drain(16);
                w.write(w.absolute(a), 0, m);
                self.pc + 3
            }
            0xd9a8 | 0xd9b2 | 0xd9b5 | 0xd9bf | 0xd9c2 | 0xd9d8 | 0xd9dd | 0xd9e3 | 0xd9e8
            | 0xd9f3 | 0xd9fc | 0xda02 | 0xda08 | 0xda0e | 0xda1d | 0xda23 | 0xda29 | 0xda2f
            | 0xda35 | 0xda3b | 0xda41 | 0xda47 | 0xda52 | 0xda55 | 0xda77 | 0xda7d | 0xda8e
            | 0xda99 | 0xdaaf | 0xdc64 | 0xdc6c | 0xdc76 | 0xdc79 | 0xdc83 | 0xdc86 | 0xdc9c
            | 0xdca1 | 0xdca7 | 0xdcac | 0xdcb7 | 0xdcbd | 0xdcc3 | 0xdcc9 | 0xdccf | 0xdcdf
            | 0xdce5 | 0xdceb | 0xdcf1 | 0xdcf7 | 0xdcfd | 0xdd03 | 0xdd09 | 0xdd14 | 0xdd17
            | 0xdd2d | 0xdd38 | 0xdd4b | 0xdd71 | 0xdd81 => {
                let a = match self.pc {
                    0xd9a8 | 0xdc6c => 0x48e,
                    0xd9b2 | 0xdc76 => 0x122,
                    0xd9b5 | 0xdc79 => 0x124,
                    0xd9bf | 0xdc83 => 0x11e,
                    0xd9c2 | 0xdc86 => 0x120,
                    0xd9d8 | 0xdc9c => 0x618,
                    0xd9dd | 0xdca1 => 0x61a,
                    0xd9e3 | 0xdca7 => 0x61c,
                    0xd9e8 | 0xdcac => 0x61e,
                    0xd9f3 | 0xdcb7 => 0x696,
                    0xd9fc | 0xdcbd => 0x610,
                    0xda02 | 0xdcc3 => 0x612,
                    0xda08 | 0xdcc9 => 0x614,
                    0xda0e | 0xdccf => 0x616,
                    0xda1d | 0xdcdf => 0x601,
                    0xda23 | 0xdce5 => 0x603,
                    0xda29 | 0xdceb => 0x605,
                    0xda2f | 0xdcf1 => 0x607,
                    0xda35 | 0xdcf7 => 0x609,
                    0xda3b | 0xdcfd => 0x60b,
                    0xda41 | 0xdd03 => 0x60d,
                    0xda47 | 0xdd09 => 0x60f,
                    0xda52 | 0xdd14 => 0x604,
                    0xda55 | 0xdd17 => 0x606,
                    0xda77 | 0xdd2d => 0xaa1,
                    0xda7d | 0xda8e | 0xdd71 | 0xdd81 => 0x132,
                    0xda99 | 0xdd38 => 0x40c,
                    0xdaaf | 0xdd4b => 0x476,
                    _ => 0x10e,
                };
                w.drain(16);
                w.write(w.absolute(a), w.cpu.a & mask, m);
                self.pc + 3
            }
            0xd8ca | 0xd8d0 | 0xd8d6 | 0xd8dc | 0xd8e2 | 0xd8e8 | 0xd8ef | 0xd8f6 | 0xd8fc
            | 0xd902 | 0xd90d | 0xd914 | 0xd91b | 0xd922 | 0xd929 | 0xd930 | 0xd937 | 0xd93e
            | 0xd945 | 0xd94c | 0xd953 | 0xd95a | 0xd963 | 0xd96a | 0xd971 | 0xd978 | 0xdb63 => {
                let a = match self.pc {
                    0xd8ca => 0xc140,
                    0xd8d0 => 0xc142,
                    0xd8d6 => 0xc144,
                    0xd8dc => 0xc146,
                    0xd8e2 => 0xc148,
                    0xd8e8 => 0xc14a,
                    0xd8ef => 0xc150,
                    0xd8f6 => 0xc152,
                    0xd8fc => 0xc14c,
                    0xd902 => 0xc14e,
                    0xd90d => 0xc154,
                    0xd914 => 0xc156,
                    0xd91b => 0xc158,
                    0xd922 => 0xc15a,
                    0xd929 => 0xc15c,
                    0xd930 => 0xc15e,
                    0xd937 => 0xc160,
                    0xd93e => 0xc162,
                    0xd945 => 0xc16a,
                    0xd94c => 0xc16c,
                    0xd953 => 0xc16e,
                    0xd95a => 0xc170,
                    0xd963 => 0xc164,
                    0xd96a => 0xc165,
                    0xd971 => 0xc166,
                    0xd978 => 0xc167,
                    _ => 0xc172,
                };
                w.drain(24);
                w.write(0x7e0000 | a, w.cpu.a & mask, m);
                self.pc + 4
            }
            0xd987 | 0xd9c5 | 0xda84 | 0xdc55 | 0xdc89 | 0xdcd2 | 0xdd79 => {
                let a = match self.pc {
                    0xd987 => 0x7ef3cc,
                    0xdc55 | 0xdcd2 => 0x7ef3c8,
                    _ => 0x7ef3c5,
                };
                w.drain(24);
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 4
            }
            0xd9a3 | 0xd9ab | 0xd9b8 | 0xd9cb | 0xd9d0 | 0xd9d5 | 0xd9e0 | 0xd9f0 | 0xda74
            | 0xda7a | 0xda91 | 0xda96 | 0xda9c | 0xdaa1 | 0xdaaa | 0xdab2 | 0xdabb | 0xdac2
            | 0xdacb | 0xdc61 | 0xdc67 | 0xdc6f | 0xdc7c | 0xdc8f | 0xdc94 | 0xdc99 | 0xdca4
            | 0xdcb4 | 0xdd2a | 0xdd30 | 0xdd35 | 0xdd3d | 0xdd46 | 0xdd4e | 0xdd57 | 0xdd5e
            | 0xdd67 | 0xdd6e | 0xda1a | 0xda20 | 0xda26 | 0xda2c | 0xda32 | 0xda38 | 0xda3e
            | 0xda44 | 0xdcdc | 0xdce2 | 0xdce8 | 0xdcee | 0xdcf4 | 0xdcfa | 0xdd00 | 0xdd06 => {
                let a = match self.pc {
                    0xd9a3 => 0xc813,
                    0xd9ab => 0xce4f,
                    0xd9b8 => 0xcd45,
                    0xd9cb => 0xcf59,
                    0xd9d0 => 0xd063,
                    0xd9d5 => 0xd16d,
                    0xd9e0 => 0xd277,
                    0xd9f0 => 0xd724,
                    0xda74 => 0xd381,
                    0xda7a => 0xd82e,
                    0xda91 => 0xd406,
                    0xda96 => 0xd48b,
                    0xda9c => 0xd510,
                    0xdaa1 | 0xdaaa => 0xd595,
                    0xdab2 | 0xdabb => 0xd61a,
                    0xdac2 | 0xdacb => 0xd69f,
                    0xdc61 => 0xdc40,
                    0xdc67 => 0xdb6e,
                    0xdc6f => 0xdbc2,
                    0xdc7c => 0xdbb4,
                    0xdc8f => 0xdbd0,
                    0xdc94 => 0xdbde,
                    0xdc99 => 0xdbec,
                    0xdca4 => 0xdbfa,
                    0xdcb4 => 0xdc32,
                    0xdd2a => 0xdc08,
                    0xdd30 => 0xdc0f,
                    0xdd35 => 0xdc16,
                    0xdd3d | 0xdd46 => 0xdc1d,
                    0xdd4e | 0xdd57 => 0xdc24,
                    0xdd5e | 0xdd67 => 0xdc2b,
                    0xdd6e => 0xdc4e,
                    0xda1a => 0xc91d,
                    0xda20 => 0xc91e,
                    0xda26 => 0xc91f,
                    0xda2c => 0xc920,
                    0xda32 => 0xc921,
                    0xda38 => 0xc922,
                    0xda3e => 0xc923,
                    0xda44 => 0xc924,
                    0xdcdc => 0xdb7c,
                    0xdce2 => 0xdb7d,
                    0xdce8 => 0xdb7e,
                    0xdcee => 0xdb7f,
                    0xdcf4 => 0xdb80,
                    0xdcfa => 0xdb81,
                    0xdd00 => 0xdb82,
                    _ => 0xdb83,
                };
                let i = if matches!(
                    self.pc,
                    0xda1a
                        | 0xda20
                        | 0xda26
                        | 0xda2c
                        | 0xda32
                        | 0xda38
                        | 0xda3e
                        | 0xda44
                        | 0xdcdc
                        | 0xdce2
                        | 0xdce8
                        | 0xdcee
                        | 0xdcf4
                        | 0xdcfa
                        | 0xdd00
                        | 0xdd06
                ) {
                    w.cpu.y
                } else {
                    w.cpu.x
                };
                w.drain(16);
                if x == 2 || (a & 255) + i > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(i), m);
                w.load(v, m);
                self.pc + 3
            }
            0xdaf9 | 0xdb00 | 0xdb07 | 0xdb0e | 0xdb15 | 0xdb1c | 0xdb23 => {
                let a = match self.pc {
                    0xdaf9 => 0x04f1de,
                    0xdb00 => 0x04f25e,
                    0xdb07 => 0x04f2de,
                    0xdb0e => 0x04f35e,
                    0xdb15 => 0x04f36a,
                    0xdb1c => 0x04f3ea,
                    _ => 0x04f46a,
                };
                w.drain(24);
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 4
            }
            0xdafd | 0xdb04 | 0xdb0b | 0xdb12 | 0xdb19 | 0xdb20 | 0xdb27 | 0xdb35 | 0xdb38
            | 0xdb3b | 0xdb3e | 0xdb41 | 0xdb44 | 0xdb47 | 0xdb4a | 0xdb4d | 0xdb50 | 0xdb53
            | 0xdb56 | 0xdb59 | 0xdb5c => {
                let a = match self.pc {
                    0xdafd => 0xf940,
                    0xdb04 => 0xf9c0,
                    0xdb0b => 0xfa40,
                    0xdb12 => 0xfac0,
                    0xdb19 => 0xfb40,
                    0xdb20 => 0xfbc0,
                    0xdb27 => 0xfc40,
                    0xdb35 => 0xf800,
                    0xdb38 => 0xf840,
                    0xdb3b => 0xf880,
                    0xdb3e => 0xf8c0,
                    0xdb41 => 0xf580,
                    0xdb44 => 0xf5c0,
                    0xdb47 => 0xf600,
                    0xdb4a => 0xf640,
                    0xdb4d => 0xf680,
                    0xdb50 => 0xf6c0,
                    0xdb53 => 0xf700,
                    0xdb56 => 0xf740,
                    0xdb59 => 0xf780,
                    _ => 0xf7c0,
                };
                w.operand_then_internal(2);
                w.write(w.absolute(a) + u32::from(w.cpu.x), w.cpu.a & mask, m);
                self.pc + 3
            }
            _ => unreachable!("dungeon entrance continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
