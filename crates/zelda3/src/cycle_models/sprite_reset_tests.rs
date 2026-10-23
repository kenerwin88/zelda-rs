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
                std::env::var_os("ZELDA3_ASSETS")
                    .expect("sprite reset tests require native assets"),
            )
            .unwrap(),
        )
        .unwrap();
    state.native_dungeon_sprite_data().unwrap()
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
    retained_temp_y_reads: usize,
    retained_temp_y_99_reads: usize,
}
impl Fixture {
    fn new(
        rom: &[u8],
        owner: &SpriteReset,
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
        let far = matches!(owner.entry, 0xc114 | 0xc22f | 0xc44e);
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
        let mut native = crate::ZeldaState::new();
        native.import_wram_image(&ram);
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
            retained_temp_y_reads: 0,
            retained_temp_y_99_reads: 0,
        }
    }
    fn run(&mut self, owner: &mut SpriteReset, data: &BTreeMap<u32, u8>, label: &str) {
        let original_db = self.cpu.db;
        let far = matches!(owner.entry, 0xc114 | 0xc22f | 0xc44e);

        while !owner.complete() {
            let origin = (u32::from(self.cpu.bank) << 16) | u32::from(self.cpu.pc);
            self.visited.insert(origin);
            // This is the source read from the historical ownership mismatch.
            // It must observe the live byte published at C38E, not a final subtype.
            if origin == 0x09c3c7 {
                assert!(self.writes.contains_key(&0xfb5) || self.writes.contains_key(&0x7e0fb5));
                self.retained_temp_y_reads += 1;
                self.retained_temp_y_99_reads += usize::from(self.ram[0xfb5] == 0x99);
            }
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
            let records_active = owner.records_active();
            let reset_parent_active = owner.reset_parent_active();
            let cache_active = owner.cache_active();
            let disable_active = owner.disable_active();
            let reset_suffix_active = owner.reset_suffix_active();
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
                                panic!("{label} pc{origin:06x}: unbound native sprite data {a:06x}")
                            })
                        }
                    };
                    let (access, value) = match access {
                        Access::Read(a, width) => {
                            if records_active && normalize(a) == 0xfb5 {
                                assert_eq!(native.game_state.scratch_counter.value(), read(a),
                                    "{label} pc{origin:06x}: source temporary-Y read retains its native last writer");
                            }
                            if records_active && normalize(a) == 0xfb6 {
                                assert_eq!(native.game_state.sprites.workspace.shared_scratch_a(), read(a),
                                    "{label} pc{origin:06x}: source temporary-X read retains its native last writer");
                            }
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
                            if (records_active && native.native_sprite_record_store(a, value, width))
                                || (reset_parent_active && native.native_sprite_reset_parent_store(a, value, width))
                                || (cache_active && native.native_sprite_cache_store(a, value, width))
                                || (disable_active && native.native_sprite_disable_store(a, value, width))
                                || (reset_suffix_active && native.native_sprite_reset_suffix_store(a, value, width)) {
                                published = true;
                            } else {
                                if records_active {
                                    assert!(matches!(i, 0x100..=0x1ff),
                                        "{label} pc{origin:06x}: unowned sprite-record store {a:06x}");
                                } else if reset_parent_active || cache_active || disable_active || reset_suffix_active {
                                    assert!(matches!(i, 0..=0xf | 0x100..=0x1ff),
                                        "{label} pc{origin:06x}: unowned sprite-disable store {a:06x}");
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
            assert_eq!(*self.native.wram_image(), self.ram, "{label}: native sprite-disable publications");
            if published {
                let expected = crate::game_state::GameState::load_from_ram(&self.ram);
                let native = &self.native.game_state;
                assert_eq!(native.scratch_counter, expected.scratch_counter,
                    "{label} pc{origin:06x}: retained record coordinate temporary");
                assert_eq!(native.oam, expected.oam, "{label}: sprite sorting owner");
                if cache_active {
                    assert_eq!(native.sprites.cached_sprites, expected.sprites.cached_sprites,
                        "{label}: retained cache banks and borrowed slot aliases");
                    assert_eq!(native.sprites.system, expected.sprites.system,
                        "{label}: shared cache flags");
                }
                assert_eq!(native.sprites.ancilla_slots, expected.sprites.ancilla_slots, "{label}: ancilla slots");
                assert_eq!(native.sprites.sprite_slots, expected.sprites.sprite_slots, "{label}: live sprite slots");
                assert_eq!(native.sprites.system.limit_instance(), expected.sprites.system.limit_instance());
                assert_eq!(native.sprites.system.chr_halfslot_state(), expected.sprites.system.chr_halfslot_state());
                assert_eq!(native.sprites.garnish_runtime, expected.sprites.garnish_runtime);
                assert_eq!(native.sprites.workspace, expected.sprites.workspace);
                assert_eq!(native.archery_game, expected.archery_game);
                assert_eq!(native.sprite_battle, expected.sprite_battle);
                assert_eq!(native.minigame, expected.minigame);
                assert_eq!(native.dungeon.room_effects, expected.dungeon.room_effects);
                assert_eq!(native.ending.attract_scene.intro_palette_flash_count,
                    expected.ending.attract_scene.intro_palette_flash_count);
                assert_eq!(native.player.follower_link.ancilla_pickup_flag(),
                    expected.player.follower_link.ancilla_pickup_flag());
                assert_eq!(native.player.follower_link.menu_block_flag(),
                    expected.player.follower_link.menu_block_flag());
                if reset_parent_active || reset_suffix_active {
                    assert_eq!(native.sprites.system, expected.sprites.system);
                    assert_eq!(native.sprites.overworld_sprite_presence, expected.sprites.overworld_sprite_presence);
                    assert_eq!(native.sprites.overworld_sprite_loaded, expected.sprites.overworld_sprite_loaded);
                    assert_eq!(native.dungeon.room_tracking, expected.dungeon.room_tracking);
                    assert_eq!(native.oam, expected.oam);
                    assert_eq!(native.display.hud_runtime, expected.display.hud_runtime);
                    assert_eq!(native.messaging.runtime, expected.messaging.runtime);
                    assert_eq!(native.player.follower_link, expected.player.follower_link);
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
            (if far { 2 } else { 9 }, 0xc201, 0x1fd, original_db, 0x30)
        );
    }
}

fn initialize(ram: &mut [u8], room: u16, variant: usize) {
    ram[0x1b] = (variant & 1) as u8;
    ram[0x48e..0x490].copy_from_slice(&room.to_le_bytes());
    ram[0xf3cc] = if variant & 2 == 0 { 13 } else { 1 };
    for i in 0..16 {
        ram[0xdd0 + i] = [0, 4, 10, 8, 9][(variant + i) % 5];
        ram[0xe20 + i] = if i & 1 == 0 { 0x6c } else { 0x18 };
        ram[0xf00 + i] = if (i + variant) % 3 == 0 { 1 } else { 0 };
    }
    for i in 0..320 {
        let mask = match variant & 3 {
            0 => 0,
            1 => 0xffff,
            2 => 0x5555,
            _ => 0xaaaa,
        };
        ram[0x1df80 + i * 2..0x1df82 + i * 2].copy_from_slice(&u16::to_le_bytes(mask));
    }
    for i in 0..4 {
        let previous = if variant & 3 == 0 {
            0xffff
        } else if i == (variant & 3) {
            room
        } else {
            (room + i as u16 + 1) % 320
        };
        ram[0xb80 + i * 2..0xb82 + i * 2].copy_from_slice(&previous.to_le_bytes());
    }
}

#[test]
fn sprite_reset_native_records_pointers_properties_and_masks_match_source() {
    let data = data();
    let rom = rom();
    for (&address, &value) in &data {
        assert_eq!(
            value,
            rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)],
            "native sprite data {address:06x}"
        );
    }
    eprintln!(
        "{} native sprite record pointer property and death-mask bytes match source",
        data.len()
    );
}

#[test]
fn sprite_reset_all_and_cache_match_cold_source_guards_clear_loops_and_returns() {
    let data = data();
    let rom = rom();
    let mut instructions = 0;
    let mut count = 0;
    let mut visited = BTreeSet::new();
    for entry in [0x09c176, 0x09c22f, 0x09c44e] {
        for variant in 0..64 {
            let mut owner = SpriteReset::from_entry(entry);
            let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 9, 0x7e][variant & 3], |ram| {
                initialize(ram, (variant * 3) as u16, variant);
            });
            f.run(
                &mut owner,
                &data,
                &format!("sprite reset {entry:06x} variant{variant}"),
            );
            count += 1;
            instructions += f.instructions;
            visited.extend(f.visited);
            if entry == 0x09c44e {
                assert!(f.ram[0x1df80..0x1f180].iter().all(|&v| v == 0));
                assert_eq!(&f.ram[0xb80..0xb88], &[0xff; 8]);
            }
        }
    }
    for &pc in OWNED_POSITIONS
        .iter()
        .filter(|&&p| (0xc175..=0xc28f).contains(&p) || (0xc44e..=0xc498).contains(&p))
    {
        assert!(
            visited.contains(&(0x090000 | u32::from(pc))),
            "unvisited sprite reset {pc:04x}"
        );
    }
    eprintln!("{count} cache disable and full-reset fixtures matched {instructions} cold source instructions across {} positions", visited.len());
}

#[test]
fn dungeon_sprite_reset_matches_cold_source_all_rooms_histories_and_death_masks() {
    let data = data();
    let rom = rom();
    let mut instructions = 0;
    let mut reads = 0;
    let mut reads_99 = 0;
    let mut visited = BTreeSet::new();
    for room in 0..320u16 {
        for variant in 0..8 {
            let mut owner = SpriteReset::from_entry(0x09c114);
            let mut f = Fixture::new(&rom, &owner, variant, [0, 2, 9, 0x7e][variant & 3], |ram| {
                initialize(ram, room, variant);
                // Also exercise absent history with a real evicted room, plus
                // a match at the first history word (search exits before INX).
                if variant == 4 || variant == 5 {
                    for i in 0..4 {
                        let old = (room + i as u16 + 1) % 320;
                        ram[0xb80 + i * 2..0xb82 + i * 2].copy_from_slice(&old.to_le_bytes());
                    }
                } else if variant >= 6 {
                    ram[0xb80..0xb82].copy_from_slice(&room.to_le_bytes());
                }
            });
            f.run(
                &mut owner,
                &data,
                &format!("dungeon sprite reset room{room} variant{variant}"),
            );
            instructions += f.instructions;
            reads += f.retained_temp_y_reads;
            reads_99 += f.retained_temp_y_99_reads;
            visited.extend(f.visited);
        }
    }
    // Original room streams never exhaust all eight overlord slots after
    // DisableAll. The independent single-record fixture covers that return.
    for &pc in OWNED_POSITIONS
        .iter()
        .filter(|&&p| (p < 0xc44e || p > 0xc498) && p != 0xc3f2)
    {
        assert!(
            visited.contains(&(0x090000 | u32::from(pc))),
            "unvisited dungeon reset {pc:04x}"
        );
    }
    assert!(reads_99 > 0);
    eprintln!("2560 complete dungeon sprite resets matched {instructions} cold source instructions across {} positions and {reads} retained temporary-Y reads; {reads_99} read 0x99 from native records", visited.len());
}

#[test]
fn dungeon_single_sprite_matches_cold_source_markers_overlord_exhaustion_and_page_crossing() {
    let data = data();
    let rom = rom();
    let mut instructions = 0;
    let mut count = 0;
    let mut visited = BTreeSet::new();
    // A WRAM fixture supplies arbitrary record combinations to both machines
    // once. Runtime still reads its pointer and each record byte live.
    for ty in [0, 3, 10, 11, 0x18, 0x6c, 0xe4, 0xf2] {
        for variant in 0..32 {
            let mut owner = SpriteReset::from_entry(0x09c327);
            let mut f = Fixture::new(&rom, &owner, variant, 9, |ram| {
                initialize(ram, 0x104, variant);
                ram[0..2].copy_from_slice(&0x06ffu16.to_le_bytes());
                ram[2] = ((variant >> 1) % 16) as u8;
                ram[3] = 0;
                let y = if ty == 0xe4 && variant % 4 < 2 {
                    [0xfe, 0xfd][variant % 4]
                } else {
                    (variant * 7) as u8
                };
                ram[0x701..0x704].copy_from_slice(&[
                    y,
                    if variant & 1 == 0 { 0xe7 } else { 0x1b },
                    ty,
                ]);
                ram[0xb00..0xb08].fill(if variant & 4 == 0 { 1 } else { 0 });
                if variant & 8 != 0 {
                    ram[0xb03] = 0;
                }
            });
            // Entry Y points at the first byte; include a pointer page crossing.
            f.cpu.y = 2;
            let mut registers = f.source.machine().snes().cpu.clone();
            registers.y = 2;
            let sram = f.source.machine().snes().cart.ram.clone();
            f.source
                .rebase_private_timing_state_at(
                    f.source.machine().timestamp(),
                    &registers,
                    &f.ram,
                    &sram,
                )
                .unwrap();
            f.run(
                &mut owner,
                &data,
                &format!("single sprite ty{ty:02x} variant{variant}"),
            );
            count += 1;
            instructions += f.instructions;
            visited.extend(f.visited);
        }
    }
    for &pc in OWNED_POSITIONS
        .iter()
        .filter(|&&p| (0xc327..0xc44e).contains(&p) || p >= 0xc871)
    {
        assert!(
            visited.contains(&(0x090000 | u32::from(pc))),
            "unvisited single sprite branch {pc:04x}"
        );
    }
    eprintln!("{count} single sprite fixtures matched {instructions} cold source instructions across {} positions", visited.len());
}
