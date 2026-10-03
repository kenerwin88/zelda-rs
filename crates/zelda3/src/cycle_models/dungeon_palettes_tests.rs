use super::super::dungeon_visited::VisitedFlags;
use super::*;
use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
enum Owner {
    Palette(DungeonPalettes),
    Visited(VisitedFlags),
}
impl Owner {
    fn routine(&self) -> u32 {
        match self {
            Self::Palette(w) => w.routine(),
            Self::Visited(w) => w.routine(),
        }
    }
    fn complete(&self) -> bool {
        match self {
            Self::Palette(w) => w.complete(),
            Self::Visited(w) => w.complete(),
        }
    }
    fn palette_source(&self, cpu: &NativeNmiContext) -> PaletteSource {
        match self {
            Self::Palette(w) => w.palette_source(cpu),
            Self::Visited(_) => PaletteSource::Constant,
        }
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        match self {
            Self::Palette(w) => w.step(cpu, clock, bus),
            Self::Visited(w) => w.step(cpu, clock, bus),
        }
    }
}

fn rom() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"))
        .unwrap()
}
fn data() -> BTreeMap<u32, u8> {
    let mut state = crate::ZeldaState::new();
    state
        .set_assets(
            &std::fs::read(
                std::env::var_os("ZELDA3_ASSETS").expect("palette tests require native assets"),
            )
            .unwrap(),
        )
        .unwrap();
    let mut data = state.native_palette_data().unwrap();
    for address in 0x02b5cc..0x02b5dc {
        data.insert(
            address,
            crate::ZeldaState::native_dungeon_visited_flag_byte(address).unwrap(),
        );
    }
    data
}
fn normalize(a: u32) -> u32 {
    if a & 0x400000 == 0 && a & 65535 < 0x6000 {
        a & 65535
    } else {
        a
    }
}
struct Fixture {
    visited_native: Option<crate::ZeldaState>,
    source: Snes9xColdCpuExecutor,
    cpu: NativeNmiContext,
    clock: CpuMasterTimeline,
    ram: Vec<u8>,
    instructions: usize,
    visited: BTreeSet<u32>,
    writes: BTreeMap<u32, usize>,
}
impl Fixture {
    fn new(
        rom: &[u8],
        owner: &Owner,
        variant: usize,
        db: u8,
        init: impl FnOnce(&mut [u8]),
    ) -> Self {
        // Only the reset fixture changes. The routines and immutable tables
        // under comparison remain original. Initial state is shared once.
        let mut boot = rom.to_vec();
        boot[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&boot).unwrap();
        let (v, h) = [(224, 1338), (261, 1350), (0, 20), (220, 900)][variant & 3];
        while source.machine().timestamp().master_cycles() < v * 1364 + h {
            source.step().unwrap();
        }
        if source.machine().timestamp().master_cycles() >= 225 * 1364 {
            assert_eq!(
                source
                    .run_until_main_loop_return()
                    .unwrap()
                    .instruction_count,
                0
            );
        }
        let far = owner.routine() != 0x02c630;
        let entry = owner.routine();
        let cpu = NativeNmiContext {
            a: 0xab19,
            x: 0x71,
            y: 0x83,
            d: 0,
            db,
            bank: (entry >> 16) as u8,
            pc: entry as u16,
            sp: if far { 0x1fa } else { 0x1fb },
            p: [0x31, 0xf0, 0x30, 0x71][variant & 3],
        };
        let mut registers = source.machine().snes().cpu.clone();
        registers.a = cpu.a;
        registers.x = cpu.x;
        registers.y = cpu.y;
        registers.k = cpu.bank;
        registers.pc = cpu.pc;
        registers.sp = cpu.sp;
        registers.db = cpu.db;
        registers.dp = 0;
        registers.e = false;
        registers.unpack_flags(cpu.p);
        let mut ram = source.machine().snes().ram.clone();
        for (i, b) in ram.iter_mut().enumerate() {
            *b = (i.wrapping_mul(7) ^ (i >> 8) ^ (variant * 53)) as u8;
        }
        init(&mut ram);
        if far {
            ram[0x1fb..0x1fe].copy_from_slice(&[0, 0xc2, 2]);
        } else {
            ram[0x1fc..0x1fe].copy_from_slice(&[0, 0xc2]);
        }
        let sram = source.machine().snes().cart.ram.clone();
        source
            .rebase_private_timing_state_at(source.machine().timestamp(), &registers, &ram, &sram)
            .unwrap();
        let clock = source.machine().timeline().clone();
        let visited_native = matches!(owner, Owner::Visited(_)).then(|| {
            let mut native = crate::ZeldaState::new();
            native.ram.copy_from_slice(&ram);
            native.sync_native_game_state_from_ram();
            native
        });
        Self {
            visited_native,
            source,
            cpu,
            clock,
            ram,
            instructions: 0,
            visited: BTreeSet::new(),
            writes: BTreeMap::new(),
        }
    }
    fn run(&mut self, owner: &mut Owner, data: &BTreeMap<u32, u8>, label: &str) {
        let original_db = self.cpu.db;

        while !owner.complete() {
            let origin = (u32::from(self.cpu.bank) << 16) | u32::from(self.cpu.pc);
            self.visited.insert(origin);
            let source = self.source.step().unwrap();
            assert!(source.accepted_interrupt.is_none());
            let mut first = true;
            let expected: Vec<_> = source
                .accesses
                .into_iter()
                .filter_map(|a| match a.kind {
                    Kind::Read { value, width } => {
                        let operand = first && a.address == origin + 1;
                        first = false;
                        (!operand).then_some((
                            a.timestamp.master_cycles(),
                            Access::Read(normalize(a.address), width),
                            value,
                        ))
                    }
                    Kind::Write { value, width } => Some((
                        a.timestamp.master_cycles(),
                        Access::Write(normalize(a.address), value, width),
                        0,
                    )),
                    _ => None,
                })
                .collect();
            let provenance = owner.palette_source(&self.cpu);
            let mut actual = Vec::new();
            let mut vblanks = Vec::new();
            let ram = &mut self.ram;
            let writes = &mut self.writes;
            let native = &mut self.visited_native;
            let mut published = false;
            owner
                .step(&mut self.cpu, &mut self.clock, |at, access| {
                    let read =
                        |a: u32| {
                            let a = normalize(a);
                            if a < 0x2000 {
                                ram[a as usize]
                            } else if (0x7e0000..0x800000).contains(&a) {
                                ram[(a - 0x7e0000) as usize]
                            } else {
                                *data.get(&a).unwrap_or_else(|| {
                            panic!("{label} pc{origin:06x}: unbound native palette data {a:06x}")
                        })
                            }
                        };
                    let (access, value) = match access {
                        Access::Read(a, width) => {
                            let value = if width == 1 {
                                u16::from(read(a))
                            } else {
                                u16::from_le_bytes([read(a), read(a + 1)])
                            };
                            (Access::Read(normalize(a), width), value)
                        }
                        Access::Write(a, value, width) => {
                            let a = normalize(a);
                            assert!(a < 0x2000 || (0x7e0000..0x800000).contains(&a));
                            let i = if a < 0x2000 {
                                a as usize
                            } else {
                                (a - 0x7e0000) as usize
                            };
                            ram[i..i + usize::from(width)]
                                .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            if let Some(native) = native.as_mut() {
                                if native.native_dungeon_visited_store(a, value, width) {
                                    published = true;
                                } else {
                                    assert!(matches!(a, 0..=0xf | 0x100..=0x1ff),
                                        "{label} pc{origin:06x}: unowned visited store {a:06x}");
                                    native.ram[i..i + usize::from(width)]
                                        .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                }
                            }
                            *writes.entry(a).or_default() += 1;
                            if (0x7ec300..0x7ec700).contains(&a) {
                                if (0x02c771..=0x02c7a9).contains(&origin) {
                                    let source = (a - 0x7ec500) as u16;
                                    assert_eq!(provenance, PaletteSource::AuxCopy(source));
                                } else if !matches!(
                                    origin,
                                    0x0ed5f9 | 0x0ed5fd | 0x0ed601 | 0x0ed605
                                ) {
                                    assert_eq!(
                                        provenance,
                                        PaletteSource::Asset,
                                        "palette writer {origin:06x}"
                                    );
                                }
                            }
                            (Access::Write(a, value, width), 0)
                        }
                        Access::Vblank { at, .. } => {
                            vblanks.push(at);
                            return 0;
                        }
                    };
                    actual.push((at, access, value));
                    value
                })
                .unwrap();
            assert_eq!(actual, expected, "{label} pc{origin:06x}");
            if let Some(native) = self.visited_native.as_ref() {
                assert_eq!(native.ram, self.ram, "{label}: native visited publications");
                if published {
                    let expected = crate::game_state::GameState::load_from_ram(&self.ram);
                    assert_eq!(native.game_state.dungeon.room_load, expected.dungeon.room_load,
                        "{label}: live visited word");
                    assert_eq!(native.game_state.inventory.save_progress, expected.inventory.save_progress,
                        "{label}: saved visited word");
                }
            }
            let c = &self.source.machine().snes().cpu;
            assert_eq!(
                self.cpu,
                NativeNmiContext {
                    a: c.a,
                    x: c.x,
                    y: c.y,
                    d: c.dp,
                    db: c.db,
                    bank: c.k,
                    pc: c.pc,
                    sp: c.sp,
                    p: c.pack_flags()
                },
                "{label} pc{origin:06x}"
            );
            assert_eq!(
                self.clock,
                *self.source.machine().timeline(),
                "{label} pc{origin:06x}"
            );
            assert_eq!(
                self.ram,
                self.source.machine().snes().ram,
                "{label} pc{origin:06x}"
            );
            for at in vblanks {
                let host = self.source.run_until_main_loop_return().unwrap();
                assert_eq!(host.instruction_count, 0);
                assert_eq!(host.vblank_event_at.master_cycles(), at);
            }
            *owner = owner.clone();
            self.instructions += 1;
            assert!(self.instructions < 250000, "{label} pc{origin:06x}");
        }
        assert_eq!(
            (
                self.cpu.bank,
                self.cpu.pc,
                self.cpu.sp,
                self.cpu.db,
                self.cpu.p & 0x38
            ),
            (2, 0xc201, 0x1fd, original_db, 0x30)
        );
    }
}

fn initialize(ram: &mut [u8], variant: usize) {
    // Valid, varied live selectors; the adjacent high bytes intentionally vary.
    ram[0x1b] = 1;
    ram[0xaa8..0xac0].fill(0);
    ram[0xaa8] = (variant & 1) as u8 * 2;
    ram[0xaa9] = 0; // Parent is tested separately with a nonzero high byte.
    ram[0xaab] = (variant % 2) as u8;
    ram[0xaac] = (variant % 16) as u8;
    ram[0xaad] = (variant % 24) as u8;
    ram[0xaae] = ((variant * 7) % 24) as u8;
    ram[0xab1] = (variant % 16) as u8;
    ram[0xab2] = (variant % 2) as u8;
    ram[0xab6] = (variant % 20 * 2) as u8;
    ram[0xab7] = (variant % 2) as u8;
    ram[0xab8] = (variant % 16) as u8;
    ram[0xabd] = (variant & 1) as u8;
    ram[0xf354] = (variant % 3) as u8;
    ram[0xf359] = (variant % 5) as u8;
    ram[0xf35a] = (variant % 4) as u8;
    ram[0xf35b] = (variant % 3) as u8;
}

#[test]
fn dungeon_palette_and_visited_native_data_match_source() {
    let data = data();
    let rom = rom();
    for (&address, &value) in &data {
        assert_eq!(
            value,
            rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)],
            "native palette data {address:06x}"
        );
    }
    eprintln!(
        "{} immutable palette and visiting-flag bytes match source",
        data.len()
    );
}

#[test]
fn dungeon_palette_leaves_match_cold_source_selectors_equipment_and_returns() {
    let data = data();
    let rom = rom();
    let mut count = 0;
    let mut instructions = 0;
    let mut visited = BTreeSet::new();
    for (entry, selectors) in [
        (0x1becc5, 24),
        (0x1bece4, 24),
        (0x1bed03, 5),
        (0x1bed29, 4),
        (0x1bed6e, 20),
        (0x1bedf9, 9),
        (0x0ed6dd, 20),
    ] {
        for selector in 0..selectors {
            for variant in 0..4 {
                let mut owner = Owner::Palette(DungeonPalettes::from_entry(entry));
                let mut f =
                    Fixture::new(&rom, &owner, variant, [0, 2, 0x1b, 0x7e][variant], |ram| {
                        initialize(ram, selector + variant);
                        match entry {
                            0x1becc5 => ram[0xaad] = selector as u8,
                            0x1bece4 => ram[0xaae] = selector as u8,
                            0x1bed03 => ram[0xf359] = selector as u8,
                            0x1bed29 => ram[0xf35a] = selector as u8,
                            0x1bed6e => {
                                ram[0x1b] = u8::from(selector < 16);
                                ram[0xab1] = (selector % 16) as u8;
                                ram[0x8a] = (selector & 1) as u8 * 0x40;
                                ram[0xabd] = ((selector >> 1) & 1) as u8;
                            }
                            0x1bedf9 => {
                                ram[0xf35b] = (selector / 3) as u8;
                                ram[0xf354] = (selector % 3) as u8;
                            }
                            0x0ed6dd => {
                                ram[0xf359] = (selector % 5) as u8;
                                ram[0xf35a] = (selector / 5) as u8;
                            }
                            _ => unreachable!(),
                        }
                    });
                f.run(
                    &mut owner,
                    &data,
                    &format!("palette {entry:06x} selector{selector} variant{variant}"),
                );
                count += 1;
                instructions += f.instructions;
                visited.extend(f.visited);
            }
        }
    }
    eprintln!("{count} palette leaf fixtures matched {instructions} cold source instructions across {} positions", visited.len());
    for &pc in OWNED_POSITIONS.iter().filter(|&&pc| pc >= 0x0e0000) {
        assert!(
            visited.contains(&pc),
            "unvisited palette continuation {pc:06x}"
        );
    }
    // Both environment paths include the reused indoor owner.
    for pc in [
        0x1bed9c, 0x1bedb5, 0x1bee26, 0x1bee35, 0x1bef95, 0x0ed756, 0x1bed90,
    ] {
        assert!(
            visited.contains(&pc),
            "unvisited palette continuation {pc:06x}"
        );
    }
}

#[test]
fn dungeon_palette_parent_matches_cold_source_live_copies_and_cache_publication() {
    let data = data();
    let rom = rom();
    let mut instructions = 0;
    let mut visited = BTreeSet::new();
    for selector in 0..20 {
        for variant in 0..4 {
            let mut owner = Owner::Palette(DungeonPalettes::from_entry(0x02c630));
            let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 0x1b, 0x7e][variant], |ram| {
                initialize(ram, selector + variant);
                ram[0xaa9] = 0xff;
                ram[0xab6] = selector as u8 * 2;
                ram[0x1b] = (variant & 1) as u8;
                ram[0x8a] = ((variant >> 1) & 1) as u8 * 0x40;
            });
            f.run(
                &mut owner,
                &data,
                &format!("palette parent set{selector} variant{variant}"),
            );
            assert_eq!(&f.ram[0xc300..0xc500], &f.ram[0xc500..0xc700]);
            assert_eq!(f.ram[0xaa9], 0);
            assert_eq!(f.ram[0xc20a], f.ram[0xab6]);
            assert_eq!(f.ram[0xc20b], f.ram[0xab8]);
            assert_eq!(f.ram[0xc20c], f.ram[0xab7]);
            instructions += f.instructions;
            visited.extend(f.visited);
        }
    }
    for &pc in OWNED_POSITIONS.iter().filter(|&&pc| pc < 0x0e0000) {
        assert!(visited.contains(&pc));
    }
    eprintln!("80 composed dungeon palette fixtures matched {instructions} cold source instructions across {} positions", visited.len());
}

#[test]
fn dungeon_visited_flags_match_cold_source_all_layouts_and_rooms() {
    let data = data();
    let rom = rom();
    let mut instructions = 0;
    let mut visited = BTreeSet::new();
    for room in 0..320u16 {
        for index in 0..16usize {
            let variant = (usize::from(room) + index) & 3;
            let mut owner = Owner::Visited(VisitedFlags::default());
            let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 0x1b, 0x7e][variant], |ram| {
                ram[0xa0..0xa2].copy_from_slice(&room.to_le_bytes());
                ram[0xa7] = ((index >> 2) & 1) as u8;
                ram[0xa6] = ((index >> 1) & 1) as u8;
                ram[0xaa] = (index & 8) as u8;
                ram[0xa9] = (index & 1) as u8;
            });
            let original = f.ram[0x408];
            let saved = u16::from_le_bytes([
                f.ram[0xf000 + room as usize * 2],
                f.ram[0xf001 + room as usize * 2],
            ]);
            f.run(
                &mut owner,
                &data,
                &format!("visited room{room} layout{index}"),
            );
            assert_eq!(f.ram[0x408], original | data[&(0x02b5cc + index as u32)]);
            let expected = saved | u16::from_le_bytes([f.ram[0x408], f.ram[0x409]]);
            assert_eq!(
                &f.ram[0xf000 + room as usize * 2..0xf002 + room as usize * 2],
                &expected.to_le_bytes()
            );
            instructions += f.instructions;
            visited.extend(f.visited);
        }
    }
    assert_eq!(visited.len(), 22);
    eprintln!("5120 visited-room fixtures matched {instructions} cold source instructions across 22 positions");
}
