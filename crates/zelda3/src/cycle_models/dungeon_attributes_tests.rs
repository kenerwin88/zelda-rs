use super::super::dungeon_object_attributes::ObjectAttributes;
use super::*;
use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
use std::collections::{BTreeMap, BTreeSet};

fn rom() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"))
        .unwrap()
}
fn data() -> BTreeMap<u32, u8> {
    let mut state = crate::ZeldaState::new();
    state
        .set_assets(
            &std::fs::read(
                std::env::var_os("ZELDA3_ASSETS").expect("attribute tests require native assets"),
            )
            .unwrap(),
        )
        .unwrap();
    state.native_dungeon_custom_attribute_data().unwrap()
}
fn normalize(a: u32) -> u32 {
    if a & 0x400000 == 0 && a & 65535 < 0x6000 {
        a & 65535
    } else {
        a
    }
}
#[derive(Clone)]
enum Owner {
    Custom(CustomAttributes),
    Basic(BasicAttributes),
    Peg(CrystalPegAttributes),
    Object(ObjectAttributes),
    Door(DoorAttributes),
    Complete(DungeonAttributes),
}
impl Owner {
    fn routine(&self) -> u32 {
        match self {
            Self::Custom(w) => w.routine(),
            Self::Basic(w) => w.routine(),
            Self::Peg(w) => w.routine(),
            Self::Object(w) => w.routine(),
            Self::Door(w) => w.routine(),
            Self::Complete(w) => w.routine(),
        }
    }
    fn complete(&self) -> bool {
        match self {
            Self::Custom(w) => w.complete(),
            Self::Basic(w) => w.complete(),
            Self::Peg(w) => w.complete(),
            Self::Object(w) => w.complete(),
            Self::Door(w) => w.complete(),
            Self::Complete(w) => w.complete(),
        }
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) {
        match self {
            Self::Custom(w) => w.step(cpu, clock, bus),
            Self::Basic(w) => w.step(cpu, clock, bus),
            Self::Peg(w) => w.step(cpu, clock, bus),
            Self::Object(w) => w.step(cpu, clock, bus),
            Self::Door(w) => w.step(cpu, clock, bus),
            Self::Complete(w) => w.step(cpu, clock, bus),
        }
        .unwrap();
    }
}
struct Fixture {
    native: crate::ZeldaState,
    source: Snes9xColdCpuExecutor,
    cpu: NativeNmiContext,
    clock: CpuMasterTimeline,
    ram: Vec<u8>,
    instructions: usize,
    visited: BTreeSet<u16>,
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
        let far = !matches!(owner, Owner::Basic(_) | Owner::Object(_) | Owner::Door(_));
        let entry = owner.routine();
        let mut cpu = NativeNmiContext {
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
        if matches!(
            owner,
            Owner::Basic(BasicAttributes {
                entry: BasicAttributeEntry::Full,
                ..
            })
        ) {
            cpu.p &= !0x20;
        }
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
    fn run(&mut self, owner: &mut Owner, data: &BTreeMap<u32, u8>, label: &str) {
        let original_db = self.cpu.db;
        let basic = matches!(owner, Owner::Basic(_) | Owner::Object(_));
        let near = basic || matches!(owner, Owner::Door(_));
        while !owner.complete() {
            let origin = (u32::from(self.cpu.bank) << 16) | u32::from(self.cpu.pc);
            self.visited.insert(self.cpu.pc);
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
            let mut published = false;
            owner.step(&mut self.cpu, &mut self.clock, |at, access| {
                let read = |a: u32| {
                    let a = normalize(a);
                    if a < 0x2000 {
                        ram[a as usize]
                    } else if (0x7e0000..0x800000).contains(&a) {
                        ram[(a - 0x7e0000) as usize]
                    } else {
                        *data.get(&a).unwrap_or_else(|| {
                            panic!("{label} pc{origin:06x}: unbound native attribute data {a:06x}")
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
                        if native.native_dungeon_attribute_store(a, value, width) {
                            published = true;
                        } else {
                            assert!(matches!(a, 0..=0xf | 0x100..=0x1ff),
                                "{label} pc{origin:06x}: unowned attribute store {a:06x}");
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
            });
            assert_eq!(actual, expected, "{label} pc{origin:06x}");
            assert_eq!(self.native.ram, self.ram, "{label}: native attribute publications");
            if published {
                let expected = crate::game_state::GameState::load_from_ram(&self.ram);
                assert_eq!(self.native.game_state.dungeon, expected.dungeon,
                    "{label} pc{origin:06x}: native attribute state");
                assert_eq!(self.native.overworld_map_state_word(),
                    crate::types::read_le_u16(&self.ram, 0x200), "{label}: attribute phase word");
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
                if near { 1 } else { 2 },
                0xc201,
                0x1fd,
                original_db,
                if basic { 0 } else { 0x30 }
            )
        );
    }
}

#[test]
fn custom_attribute_native_tables_match_source() {
    let data = data();
    let rom = rom();
    for (&a, &v) in &data {
        assert_eq!(
            v,
            rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
            "custom attribute data {a:06x}"
        );
    }
    assert_eq!(data.len(), 1066);
    eprintln!("1066 immutable custom attribute bytes match source");
}

#[test]
fn custom_attributes_match_cold_source_all_themes_banks_and_continuations() {
    let data = data();
    let rom = rom();
    let mut instructions = 0;
    for theme in 0..21u16 {
        for variant in 0..4 {
            let mut owner = Owner::Custom(CustomAttributes::default());
            let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 0x7e, 0x7f][variant], |ram| {
                ram[0x0aa2..0x0aa4].copy_from_slice(&(0xa500 | theme).to_le_bytes());
            });
            f.run(
                &mut owner,
                &data,
                &format!("custom theme{theme} variant{variant}"),
            );
            let selector = 0x0e9000 + u32::from(theme) * 2;
            let offset = u16::from_le_bytes([data[&selector], data[&(selector + 1)]]);
            for i in 0..128 {
                assert_eq!(
                    f.ram[0xff40 + i],
                    data[&(0x0e902a + u32::from(offset) + i as u32)]
                );
            }
            assert_eq!(
                f.writes
                    .range(0x7eff40..0x7effc0)
                    .map(|(_, n)| n)
                    .sum::<usize>(),
                64
            );
            instructions += f.instructions;
        }
    }
    eprintln!("84 custom attribute fixtures matched {instructions} cold source instructions across 21 themes and four saved data banks");
}

fn init_basic(ram: &mut [u8], variant: usize, source: u16, dest: u16, count: u16) {
    for (a, v) in [(0, count), (0xb2, source), (0xb4, dest), (0x0200, 0x7fff)] {
        ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
    }
    for i in 0..8192usize {
        let tile = ((i & 1023) | (((i / 1024 + variant) & 3) << 14)) as u16;
        ram[0x2000 + i * 2..0x2002 + i * 2].copy_from_slice(&tile.to_le_bytes());
    }
    // This table crosses from bank7E into bank7F. Attribute thresholds and
    // tile orientation are live input, including the word look-ahead byte.
    for i in 0..1025 {
        ram[0xfe00 + i] = (i + variant * 13) as u8;
    }
}

#[test]
fn basic_attributes_match_cold_source_full_partial_orientation_and_bank_carry() {
    let rom = rom();
    let data = BTreeMap::new();
    let (mut cases, mut instructions) = (0, 0);
    let mut visited = BTreeSet::new();
    for variant in 0..4 {
        for count in [1, 2, 63, 64, 4096] {
            let mut owner = Owner::Basic(BasicAttributes::new(BasicAttributeEntry::Full));
            let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 0x7e, 0x7f][variant], |ram| {
                init_basic(ram, variant, 0, 0, count)
            });
            f.run(
                &mut owner,
                &data,
                &format!("full attributes count{count} variant{variant}"),
            );
            assert_eq!(u16::from_le_bytes([f.ram[0xb2], f.ram[0xb3]]), count * 4);
            assert_eq!(u16::from_le_bytes([f.ram[0xb4], f.ram[0xb5]]), count * 2);
            assert_eq!(&f.ram[0..2], &[0, 0]);
            assert_eq!(
                f.writes
                    .range(0x7f2000..0x7f4000)
                    .map(|(_, n)| n)
                    .sum::<usize>(),
                usize::from(count)
            );
            assert_eq!(
                u16::from_le_bytes([f.ram[0x200], f.ram[0x201]]),
                if count == 4096 { 0x8000 } else { 0x7fff }
            );
            instructions += f.instructions;
            cases += 1;
            visited.extend(f.visited);
        }
        for slice in 0..64u16 {
            let entry = if slice == 0 {
                BasicAttributeEntry::Initialize
            } else {
                BasicAttributeEntry::Partial
            };
            let mut owner = Owner::Basic(BasicAttributes::new(entry));
            let mut f = Fixture::new(&rom, &owner, variant, 0, |ram| {
                init_basic(ram, variant, slice * 256, slice * 128, 0xffff)
            });
            f.run(
                &mut owner,
                &data,
                &format!("partial attributes slice{slice} variant{variant}"),
            );
            assert_eq!(
                u16::from_le_bytes([f.ram[0xb2], f.ram[0xb3]]),
                (slice + 1) * 256
            );
            assert_eq!(
                u16::from_le_bytes([f.ram[0xb4], f.ram[0xb5]]),
                (slice + 1) * 128
            );
            assert_eq!(
                f.writes
                    .range(0x7f2000..0x7f4000)
                    .map(|(_, n)| n)
                    .sum::<usize>(),
                64
            );
            instructions += f.instructions;
            cases += 1;
            visited.extend(f.visited);
        }
    }
    for pc in [
        0xb8e3, 0xb8e5, 0xb8ec, 0xb8f3, 0xb904, 0xb918, 0xb91c, 0xb91e, 0xb923, 0xb927, 0xb930,
        0xb934, 0xb936, 0xb93b, 0xb93f, 0xb944, 0xb957, 0xb962, 0xb965, 0xb966,
    ] {
        assert!(
            visited.contains(&pc),
            "missing attribute continuation {pc:04x}"
        );
    }
    eprintln!("{cases} basic attribute fixtures matched {instructions} cold source instructions across three entries and 64 partial slices");
}

#[test]
fn crystal_peg_attributes_match_cold_source_all_planes_and_live_values() {
    let rom = rom();
    let data = BTreeMap::new();
    let mut instructions = 0;
    for variant in 0..4 {
        let mut owner = Owner::Peg(CrystalPegAttributes::default());
        let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 0x7e, 0x7f][variant], |ram| {
            for i in 0..8192 {
                ram[0x12000 + i] = match variant {
                    0 => i as u8,
                    1 => 0x66,
                    2 => 0x67,
                    _ => 0x65,
                };
            }
        });
        let before = f.ram[0x12000..0x14000].to_vec();
        f.run(
            &mut owner,
            &data,
            &format!("peg attributes variant{variant}"),
        );
        let mut changed = 0;
        for (i, old) in before.into_iter().enumerate() {
            let expected = if matches!(old, 0x66 | 0x67) {
                changed += 1;
                old ^ 1
            } else {
                old
            };
            assert_eq!(f.ram[0x12000 + i], expected);
        }
        assert_eq!(
            f.writes
                .range(0x7f2000..0x7f4000)
                .map(|(_, n)| n)
                .sum::<usize>(),
            changed
        );
        instructions += f.instructions;
    }
    eprintln!("4 crystal peg fixtures matched {instructions} cold source instructions across four live attribute planes");
}

fn init_object_lists(ram: &mut [u8], scenario: usize, tag: (u8, u8)) {
    ram[0x0400..0x04b0].fill(0);
    let mut put = |a: usize, v: u16| ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
    // Attribute positions occupy both live planes, including odd coordinates.
    for i in 0..48usize {
        put(0x06a0 + i * 2, ((i * 37 + scenario * 17) & 0x7ff) as u16);
    }
    put(0x0432, [0, 2, 4, 8][scenario & 3]);
    let mut end = 0;
    for (i, a) in [
        0x0438, 0x047e, 0x0482, 0x04a2, 0x04a4, 0x043a, 0x0480, 0x0484, 0x04a6, 0x04a8,
    ]
    .into_iter()
    .enumerate()
    {
        if scenario & 1 != 0 || i == scenario % 10 {
            end += 2;
        }
        put(a, end);
    }
    let first = scenario % 4;
    for a in [0x043c, 0x043e, 0x0440, 0x049a, 0x049c, 0x049e] {
        put(a, 0);
    }
    if first < 3 {
        put([0x043c, 0x043e, 0x0440][first], 4);
        put([0x049a, 0x049c, 0x049e][first], 4);
    }
    put(
        0x0448,
        if first < 3 { 4 } else { 0 } + if scenario & 4 == 0 { 0 } else { 2 },
    );
    put(0x0442, if scenario & 8 == 0 { 0 } else { 2 });
    put(
        0x0444,
        if scenario & 8 == 0 { 0 } else { 2 } + if scenario & 16 == 0 { 0 } else { 2 },
    );
    put(0x04ae, if scenario & 32 == 0 { 0 } else { 4 });
    let blocks = [0, 2, 4, 8][scenario & 3];
    put(0x042c, blocks);
    put(0x042e, blocks + if scenario & 4 == 0 { 0 } else { 4 });
    for i in 0..16 {
        put(0x0500 + i * 2, if i & 1 == 0 { 0x0030 } else { 0x0040 });
        put(
            0x0540 + i * 2,
            0x8000 | ((i * 42 + scenario * 7) & 0x1fff) as u16,
        );
    }
    let chests = if scenario & 8 == 0 { 0 } else { 6 };
    put(0x0496, chests);
    put(0x0498, chests + if scenario & 16 == 0 { 0 } else { 2 });
    for (i, v) in [0, 0x0604, 0x8707, 0x0828].into_iter().enumerate() {
        put(0x06e0 + i * 2, v);
    }
    put(0x0200, 0x7fff);
    ram[0xae] = tag.0;
    ram[0xaf] = tag.1;
}

#[test]
fn object_attributes_match_cold_source_lists_flags_and_nested_chest_returns() {
    let rom = rom();
    let data = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for scenario in 0..64usize {
        for phase in 0..4 {
            let mut owner = Owner::Object(ObjectAttributes::default());
            let mut f = Fixture::new(&rom, &owner, phase, 0, |ram| {
                init_object_lists(
                    ram,
                    scenario,
                    [(0, 0), (0x28, 0x33), (0x33, 0), (0x3f, 0x3f)][phase],
                )
            });
            f.run(
                &mut owner,
                &data,
                &format!("object attributes scenario{scenario} phase{phase}"),
            );
            assert_eq!(&f.ram[0x0200..0x0202], &[0, 0x80]);
            if scenario & 4 != 0 {
                assert_eq!(&f.ram[0x042e..0x0430], &[0, 0]);
            }
            if scenario & 8 != 0 {
                assert_eq!(f.ram[0x06e5] & 0x80, 0);
            }
            instructions += f.instructions;
            visited.extend(f.visited);
        }
    }
    // All concrete source positions, including both physical returns. The tag
    // comparison paths join at the same continuations as this list fixture.
    for pc in [
        0xb967, 0xb974, 0xb993, 0xb9bc, 0xb9e6, 0xba10, 0xba3d, 0xba74, 0xba96, 0xbac0, 0xbaea,
        0xbb17, 0xbb4c, 0xbb5b, 0xbb69, 0xbbb7, 0xbbef, 0xbc27, 0xbc56, 0xbc61, 0xbc8e, 0xbcb2,
        0xbcfe, 0xbd06, 0xbd38, 0xbd47, 0xbd55, 0xbdaa, 0xbdda, 0xbddb, 0xbde0, 0xbdf5, 0xbe07,
        0xbe16,
    ] {
        assert!(
            visited.contains(&pc),
            "missing object continuation {pc:04x}"
        );
    }
    assert_eq!(visited.len(), 459);
    eprintln!("256 object attribute fixtures matched {instructions} cold source instructions across {} retained positions and nested chest returns", visited.len());
}

#[test]
fn object_attribute_room_tags_match_cold_source_all_values_in_both_slots() {
    let rom = rom();
    let data = BTreeMap::new();
    let mut instructions = 0;
    for slot in 0..2 {
        for tag in 0..=255u8 {
            let mut owner = Owner::Object(ObjectAttributes::default());
            let mut f = Fixture::new(&rom, &owner, usize::from(tag & 3), 0, |ram| {
                init_object_lists(ram, 63, if slot == 0 { (tag, 0) } else { (0, tag) })
            });
            let chests_before = f.ram[0x6e0..0x6e8].to_vec();
            f.run(
                &mut owner,
                &data,
                &format!("object tag slot{slot} value{tag:02x}"),
            );
            if matches!(tag, 0x27 | 0x29..=0x32 | 0x3c | 0x3e) {
                assert_eq!(&f.ram[0x6e0..0x6e8], chests_before.as_slice());
                assert!(!f.visited.contains(&0xbcfe));
                assert!(!f.visited.contains(&0xbd06));
            } else {
                assert!(f.visited.contains(&0xbcfe));
                assert!(f.visited.contains(&0xbe16));
                assert!(f.visited.contains(&0xbd06));
                assert_eq!(f.ram[0x6e5] & 0x80, 0);
                assert_eq!(f.ram[0x6e7] & 0x80, 0x80);
            }
            instructions += f.instructions;
        }
    }
    eprintln!("512 object tag fixtures matched {instructions} cold source instructions across both live room tags");
}

fn all_data() -> BTreeMap<u32, u8> {
    let mut state = crate::ZeldaState::new();
    state
        .set_assets(&std::fs::read(std::env::var_os("ZELDA3_ASSETS").unwrap()).unwrap())
        .unwrap();
    state.native_dungeon_attribute_data().unwrap()
}
#[test]
fn complete_attribute_native_tables_match_source() {
    let rom = rom();
    let data = all_data();
    assert_eq!(data.len(), 1178);
    for (&a, &v) in &data {
        assert_eq!(
            v,
            rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
            "attribute data {a:06x}"
        );
    }
    eprintln!("1178 immutable complete attribute bytes match source");
}
fn init_doors(ram: &mut [u8], kind: u16, direction: u16, slot: usize, variant: usize) {
    ram[0x1980..0x19ea].fill(0);
    let mut put = |a: usize, v: u16| ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
    let pos = 0x0604 + slot as u16 * 4;
    put(0x1980 + slot * 2, kind);
    put(0x19a0 + slot * 2, pos);
    put(0x19c0 + slot * 2, direction);
    put(0x068c, [0, 0xffff, 0xaaaa, 0x5555][variant]);
    for (i, a) in [0x19e2, 0x19e4, 0x19e6, 0x19e8].into_iter().enumerate() {
        put(a, if i == variant { pos } else { 0xffff });
    }
    put(0x044e, if variant == 0 { 0 } else { 4 });
    put(0x0450, if variant == 1 { 0 } else { 4 });
    for (i, a) in [0x06c0, 0x06c2, 0x06d0, 0x06d2].into_iter().enumerate() {
        let tile = 0x0200 + i as u16 * 0x0100;
        put(a, tile);
        put(
            0x12000 + usize::from(tile),
            if (i + variant) & 1 == 0 {
                0x8080
            } else {
                0x0000
            },
        );
        put(0x13000 + usize::from(tile), 0x8181);
    }
    put(0x04b0, [0, 0x0600, 0x8600, 0x8600][variant]);
    put(0x0402, if variant & 1 == 0 { 0x1000 } else { 0 });
    put(0x0200, 0x7fff);
}
#[test]
fn door_attributes_match_cold_source_types_directions_records_and_children() {
    let rom = rom();
    let data = all_data();
    let mut instructions = 0;
    let mut visited = BTreeSet::new();
    for kind in (0..80u16).step_by(2) {
        for direction in 0..4u16 {
            for slot in [0, 7, 8, 15] {
                for variant in 0..4 {
                    let mut owner = Owner::Door(DoorAttributes::default());
                    let mut f = Fixture::new(&rom, &owner, variant, 0, |ram| {
                        init_doors(ram, kind, direction, slot, variant)
                    });
                    f.run(
                        &mut owner,
                        &data,
                        &format!("door kind{kind:02x} dir{direction} slot{slot} variant{variant}"),
                    );
                    // The switch child restores M8 before the caller increments
                    // only the low byte. The high byte must remain intact.
                    assert_eq!(&f.ram[0x0200..0x0202], &[0, 0x7f]);
                    instructions += f.instructions;
                    visited.extend(f.visited);
                }
            }
        }
    }
    for pc in [
        0xbe17, 0xbe21, 0xbe2b, 0xbe2e, 0xbe31, 0xbe34, 0xbe35, 0xbe9c, 0xbeb7, 0xbebf, 0xbec4,
        0xbee8, 0xbf3e, 0xbf61, 0xbf8b, 0xbfb2, 0xc085, 0xc09c, 0xc0b8, 0xc128, 0xc15e, 0xc18d,
        0xd51f, 0xd536, 0xd54e, 0xd57b, 0xd593, 0xd5a9, 0xc1ba, 0xc1d2, 0xc1db, 0xc200, 0xc201,
        0xc21b,
    ] {
        assert!(visited.contains(&pc), "missing door continuation {pc:04x}");
    }
    assert_eq!(visited.len(), 368);
    eprintln!("2560 door attribute fixtures matched {instructions} cold source instructions across {} retained positions and three child returns", visited.len());
}
#[test]
fn complete_attributes_match_cold_source_live_layers_lists_and_peg_state() {
    let rom = rom();
    let data = all_data();
    let mut instructions = 0;
    let mut visited = BTreeSet::new();
    for scenario in 0..64usize {
        for barrier in 0..2 {
            let mut owner = Owner::Complete(DungeonAttributes::default());
            let mut f = Fixture::new(&rom, &owner, scenario & 3, 0, |ram| {
                init_basic(ram, scenario & 3, 0x7777, 0xeeee, 0xffff);
                init_object_lists(
                    ram,
                    scenario,
                    if scenario & 1 == 0 {
                        (0, 0)
                    } else {
                        (0x2b, 0x3f)
                    },
                );
                init_doors(
                    ram,
                    ((scenario % 40) * 2) as u16,
                    (scenario % 4) as u16,
                    scenario % 16,
                    scenario & 3,
                );
                ram[0xc172] = barrier;
            });
            f.run(
                &mut owner,
                &data,
                &format!("complete attributes scenario{scenario} barrier{barrier}"),
            );
            assert_eq!(&f.ram[0xb2..0xb6], &[0, 0x40, 0, 0x20]);
            assert_eq!(&f.ram[0x0200..0x0202], &[0, 0x80]);
            assert_eq!(f.visited.contains(&0xc22a), barrier != 0);
            assert_eq!(f.visited.contains(&0xc27c), barrier != 0);
            instructions += f.instructions;
            visited.extend(f.visited);
        }
    }
    for pc in [
        0xb8bf, 0xb8ca, 0xb8cd, 0xb8cf, 0xb8d2, 0xb8d5, 0xb8d9, 0xb8db, 0xb8df, 0xb8e2, 0xb8f3,
        0xb966, 0xb967, 0xbdda, 0xbe17, 0xbe34, 0xc22a, 0xc27c,
    ] {
        assert!(
            visited.contains(&pc),
            "missing parent/child continuation {pc:04x}"
        );
    }
    eprintln!("128 complete attribute fixtures matched {instructions} cold source instructions with live basic object door and optional peg children");
}
