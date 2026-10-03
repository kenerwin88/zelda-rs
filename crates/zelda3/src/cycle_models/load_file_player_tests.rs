use super::*;
use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
use std::collections::{BTreeMap, BTreeSet};

fn rom() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"))
        .unwrap()
}
fn data() -> BTreeMap<u32, u8> {
    (0x02894c..=0x028953)
        .chain(0x099ef8..=0x099efb)
        .chain(0x08806f..=0x0880b2)
        .map(|a| (a, data_byte(a).unwrap()))
        .collect()
}
fn far(entry: u32) -> bool {
    !matches!(entry, 0x028aef | 0x028d71 | 0x09ad06)
}
fn normalize(a: u32) -> u32 {
    if a & 0x400000 == 0 && a & 65535 < 0x6000 {
        a & 65535
    } else {
        a
    }
}
struct Fixture {
    native: crate::ZeldaState,
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
        owner: &LoadFilePlayer,
        variant: usize,
        db: u8,
        init: impl FnOnce(&mut [u8]),
    ) -> Self {
        // Only the reset fixture changes. The routines and immutable tables
        // under comparison remain original. Initial state is shared once.
        let mut boot = rom.to_vec();
        boot[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&boot).unwrap();
        let phase = ((variant & 3) + (variant >> 8)) & 3;
        let (v, h) = [(224, 1338), (261, 1350), (0, 20), (220, 900)][phase];
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
        let far = far(owner.entry);
        let entry = owner.routine();
        let cpu = NativeNmiContext {
            a: 0xab00 | (variant & 255) as u16,
            x: (variant % 10) as u16,
            y: 0x83,
            d: 0,
            db,
            bank: (entry >> 16) as u8,
            pc: entry as u16,
            sp: if far { 0x1fa } else { 0x1fb },
            p: [0x31, 0xf0, 0x30, 0x71][phase],
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
        let mut native = crate::ZeldaState::new();
        native.ram.copy_from_slice(&ram);
        native.sync_native_game_state_from_ram();
        Self {
            native,
            source,
            cpu,
            clock,
            ram,
            instructions: 0,
            visited: BTreeSet::new(),
            writes: BTreeMap::new(),
        }
    }
    fn run(&mut self, owner: &mut LoadFilePlayer, data: &BTreeMap<u32, u8>, label: &str) {
        let original_db = self.cpu.db;
        let far = far(owner.entry);

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
            let mut actual = Vec::new();
            let mut vblanks = Vec::new();
            let ram = &mut self.ram;
            let writes = &mut self.writes;
            let native = &mut self.native;
            let cleanup_active = owner.interactive_cleanup_active();
            let cache_active = owner.room_entry_cache_active();
            let follower_active = owner.follower_initialization_active();
            let mut published = false;
            owner
                .step(&mut self.cpu, &mut self.clock, |at, access| {
                    let read = |a: u32| {
                        let a = normalize(a);
                        if a < 0x2000 {
                            ram[a as usize]
                        } else if (0x7e0000..0x800000).contains(&a) {
                            ram[(a - 0x7e0000) as usize]
                        } else {
                            *data.get(&a).unwrap_or_else(|| {
                                panic!("{label} pc{origin:06x}: unbound native player data {a:06x}")
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
                            if (cleanup_active && native.native_interactive_cleanup_store(a, value, width))
                                || (cache_active && native.native_room_entry_cache_store(a, value, width))
                                || (follower_active && native.native_follower_initialization_store(a, value, width)) {
                                published = true;
                            } else {
                                if cleanup_active || cache_active || follower_active {
                                    assert!(matches!(i, 0..=0xf | 0x100..=0x1ff),
                                        "{label} pc{origin:06x}: unowned interactive store {a:06x}");
                                }
                                native.ram[i..i + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            }
                            *writes.entry(a).or_default() += 1;
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
            assert_eq!(self.native.ram, self.ram, "{label}: native cleanup publications");
            if published {
                let expected = crate::game_state::GameState::load_from_ram(&self.ram);
                assert_eq!(self.native.game_state.player.follower_link, expected.player.follower_link,
                    "{label} pc{origin:06x}: player cleanup owner");
                assert_eq!(self.native.game_state.sprites.ancilla_slots, expected.sprites.ancilla_slots,
                    "{label} pc{origin:06x}: retained ancilla slots");
                assert_eq!(self.native.game_state.sprites.follower_runtime, expected.sprites.follower_runtime);
                assert_eq!(self.native.game_state.minigame, expected.minigame);
                assert_eq!(self.native.game_state.dungeon.environment, expected.dungeon.environment);
                if follower_active {
                    assert_eq!(self.native.game_state.sprites.tagalong_trail, expected.sprites.tagalong_trail,
                        "{label}: initial follower history sample");
                }
                if cache_active {
                    assert_eq!(self.native.game_state.world, expected.world, "{label}: cached world state");
                    assert_eq!(self.native.game_state.display, expected.display, "{label}: cached display state");
                    assert_eq!(self.native.game_state.dungeon, expected.dungeon, "{label}: cached dungeon state");
                    assert_eq!(self.native.game_state.frame, expected.frame, "{label}: room-entry phase");
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
            (
                if far { 2 } else { (owner.entry >> 16) as u8 },
                0xc201,
                0x1fd,
                original_db,
                0x30
            )
        );
    }
}

#[test]
fn load_file_player_native_tables_match_source() {
    let rom = rom();
    let data = data();
    for (&address, &value) in &data {
        assert_eq!(
            value,
            rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)],
            "{address:06x}"
        );
    }
    assert_eq!(data.len(), 80);
    assert_eq!(data_byte(0x099efc), None);
    eprintln!("80 native player background follower and ancilla table bytes match source");
}

fn check(
    rom: &[u8],
    data: &BTreeMap<u32, u8>,
    entry: u32,
    variant: usize,
    init: impl FnOnce(&mut [u8]),
    totals: &mut (usize, usize, BTreeSet<u32>),
) {
    let mut owner = LoadFilePlayer::from_entry(entry);
    let db = [0, 2, 9, 0x7e][variant & 3];
    let mut fixture = Fixture::new(rom, &owner, variant, db, init);
    fixture.run(&mut owner, data, &format!("{entry:06x} variant{variant}"));
    totals.0 += 1;
    totals.1 += fixture.instructions;
    totals.2.extend(fixture.visited);
}

#[test]
fn load_file_player_leaves_match_cold_source_live_state_and_nested_returns() {
    let rom = rom();
    let data = data();
    let mut totals = (0, 0, BTreeSet::new());
    for variant in 0..256 {
        check(&rom, &data, 0x01fec1, variant, |_| {}, &mut totals);
    }
    let mut variant = 0;
    for state in [0, 2, 13, 17, 28, 255] {
        for transformed in [0, 1, 255] {
            for pearl in [0, 1, 255] {
                check(
                    &rom,
                    &data,
                    0x0ffd22,
                    variant,
                    |ram| {
                        ram[0x5d] = state;
                        ram[0x2e0] = transformed;
                        ram[0xf357] = pearl;
                    },
                    &mut totals,
                );
                variant += 1;
            }
        }
    }
    for variant in 0..64 {
        check(&rom, &data, 0x028d71, variant, |_| {}, &mut totals);
    }
    for variant in 0..16 {
        check(&rom, &data, 0x0afd90, variant, |_| {}, &mut totals);
    }
    for variant in 0..128 {
        check(
            &rom,
            &data,
            0x099efc,
            variant,
            |ram| {
                ram[0xee] = (variant / 32) as u8;
                ram[0x2f] = ((variant / 4) % 8) as u8;
            },
            &mut totals,
        );
    }
    for variant in 0..32 {
        check(&rom, &data, 0x079a2c, variant, |_| {}, &mut totals);
    }
    for kind in 0..68 {
        for phase in 0..4 {
            // A is the incoming ancilla type, including its preserved high byte.
            // Timing/DB variation is selected independently in this child fixture.
            let mut owner = LoadFilePlayer::from_entry(0x098091);
            let mut f = Fixture::new(
                &rom,
                &owner,
                kind + 256 * phase,
                [0, 2, 9, 0x7e][phase],
                |_| {},
            );
            f.run(
                &mut owner,
                &data,
                &format!("blanket type{kind} phase{phase}"),
            );
            totals.0 += 1;
            totals.1 += f.instructions;
            totals.2.extend(f.visited);
        }
    }
    let expected: BTreeSet<_> = OWNED_POSITIONS
        .iter()
        .copied()
        .filter(|p| !matches!(*p, 0x028aef..=0x028b2d | 0x0dfa33..=0x0dfa57 | 0x09ac6b..=0x09acf2))
        .collect();
    assert_eq!(totals.2, expected);
    eprintln!(
        "{} load-game player leaf fixtures matched {} cold source instructions across {} positions",
        totals.0,
        totals.1,
        totals.2.len()
    );
}

#[test]
fn load_file_player_reset_matches_cold_source_torches_interactives_and_recoil() {
    let rom = rom();
    let data = data();
    let mut totals = (0, 0, BTreeSet::new());
    for background in 0..8 {
        for variant in 0..128 {
            check(
                &rom,
                &data,
                0x028aef,
                variant,
                |ram| {
                    ram[0x414] = background;
                    ram[0xf34a] = (variant & 1) as u8;
                    ram[0xc005] = (variant & 2) as u8;
                    ram[0x458] = (variant & 4) as u8;
                    ram[0x45a] = (variant & 8) as u8;
                    ram[0x372] = (variant & 16) as u8;
                    ram[0x308] = if variant & 32 == 0 { 0 } else { 0x80 };
                    ram[0x5d] = if variant & 64 == 0 { 0x13 } else { 2 };
                    ram[0x2ec] = (variant % 7) as u8;
                    ram[0x48] = if variant & 16 == 0 { 0x7f } else { 0xff };
                    ram[0x37a] = if variant & 1 == 0 { 4 } else { 0x14 };
                    for slot in 0..6 {
                        ram[0xc4a + slot] = [0x3e, 0x2c, 0, 1, 0x2c, 0x3e][(slot + variant) % 6];
                    }
                },
                &mut totals,
            );
        }
    }
    let expected: BTreeSet<_> = OWNED_POSITIONS
        .iter()
        .copied()
        .filter(|p| matches!(*p, 0x028aef..=0x028b2d | 0x0dfa33..=0x0dfa57 | 0x09ac6b..=0x09acf2))
        .collect();
    assert_eq!(totals.2, expected);
    eprintln!(
        "{} composed player resets matched {} cold source instructions across {} positions",
        totals.0,
        totals.1,
        totals.2.len()
    );
}

#[test]
fn load_file_player_interactive_cleanup_matches_cold_source_pickup_preservation() {
    let rom = rom();
    let data = data();
    let mut totals = (0, 0, BTreeSet::new());
    for variant in 0..168 {
        check(
            &rom,
            &data,
            0x09ac6b,
            variant,
            |ram| {
                ram[0x2ec] = (variant % 7) as u8;
                ram[0x308] = if variant / 7 % 2 == 0 { 0 } else { 0x80 };
                ram[0x48] = if variant / 14 % 2 == 0 { 0x7f } else { 0x80 };
                ram[0x5d] = if variant / 28 % 2 == 0 { 0x13 } else { 0x14 };
                ram[0x37a] = if variant % 2 == 0 { 0x14 } else { 4 };
                for slot in 0..6 {
                    ram[0xc4a + slot] = [0, 0x2c, 0x3e][(slot + variant / 56) % 3];
                }
            },
            &mut totals,
        );
    }
    let expected: BTreeSet<_> = OWNED_POSITIONS
        .iter()
        .copied()
        .filter(|p| matches!(*p, 0x09ac6b..=0x09acf2))
        .collect();
    assert_eq!(totals.2, expected);
    eprintln!(
        "{} interactive cleanup fixtures matched {} cold source instructions across {} positions",
        totals.0,
        totals.1,
        totals.2.len()
    );
}
