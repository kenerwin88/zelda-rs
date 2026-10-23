use super::*;
use super::super::native_bus_tests::ApuFixture;
use snes::{PpuState, Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
use std::collections::{BTreeMap, BTreeSet};

fn data() -> BTreeMap<u32, u8> {
    let mut state = crate::ZeldaState::new();
    state
        .set_assets(
            &std::fs::read(
                std::env::var_os("ZELDA3_ASSETS").expect("room upload tests require native assets"),
            )
            .unwrap(),
        )
        .unwrap();
    state.native_dungeon_upload_data().unwrap()
}
fn rom() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"))
        .unwrap()
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
    Preparation(TilemapPreparation),
    Upload(TilemapUpload),
    Load(DungeonLoadAndDraw),
}
impl Owner {
    fn entry(&self) -> u32 {
        match self {
            Self::Preparation(w) => w.routine(),
            Self::Upload(w) => w.routine(),
            Self::Load(w) => 0x020000 | u32::from(w.pc),
        }
    }
    fn complete(&self) -> bool {
        match self {
            Self::Preparation(w) => w.complete(),
            Self::Upload(w) => w.complete(),
            Self::Load(w) => w.complete(),
        }
    }
    fn dma_end(&mut self) -> Option<u64> {
        match self {
            Self::Preparation(_) => None,
            Self::Upload(w) => w.take_dma_end(),
            Self::Load(w) => w.take_dma_end(),
        }
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) {
        let result = match self {
            Self::Preparation(w) => w.step(cpu, clock, bus),
            Self::Upload(w) => w.step(cpu, clock, dma, bus),
            Self::Load(w) => w.step(cpu, clock, dma, bus),
        };
        result.unwrap();
    }
}
struct Fixture {
    native: crate::ZeldaState,
    source: Snes9xColdCpuExecutor,
    cpu: NativeNmiContext,
    clock: CpuMasterTimeline,
    ram: Vec<u8>,
    ppu: PpuState,
    dma: DmaState,
    apu: ApuFixture,
    instructions: usize,
    transfers: usize,
    dma_vblanks: usize,
    visited: BTreeSet<u32>,
}
impl Fixture {
    fn new(rom: &[u8], owner: &Owner, variant: usize, init: impl FnOnce(&mut [u8])) -> Self {
        // Warm independent devices with interrupts disabled. Only the reset
        // fixture is replaced; every target routine and table remains original.
        let mut boot = rom.to_vec();
        boot[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&boot).unwrap();
        let (v, h) = [(220, 900), (224, 200), (261, 1200), (101, 850)][variant & 3];
        let wanted = v * 1364 + h;
        let mut apu = ApuFixture::new();
        apu.warm_to(&mut source, wanted);
        if source.machine().timestamp().master_cycles() >= 225 * 1364 {
            assert_eq!(
                source
                    .run_until_main_loop_return()
                    .unwrap()
                    .instruction_count,
                0
            );
        }
        let far = matches!(
            owner,
            Owner::Preparation(_) | Owner::Upload(TilemapUpload { far: true, .. })
        );
        let entry = owner.entry();
        let cpu = NativeNmiContext {
            a: 0xab19,
            x: 0x71,
            y: 0x83,
            d: 0,
            db: 0,
            bank: (entry >> 16) as u8,
            pc: entry as u16,
            sp: if far { 0x1fa } else { 0x1fb },
            p: 0x31,
        };
        let mut registers = source.machine().snes().cpu.clone();
        registers.a = cpu.a;
        registers.x = cpu.x;
        registers.y = cpu.y;
        registers.k = cpu.bank;
        registers.pc = cpu.pc;
        registers.sp = cpu.sp;
        registers.db = 0;
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
        let ppu = source.machine().snes().ppu.clone();
        let dma = source.machine().snes().dma.clone();
        let mut native = crate::ZeldaState::new();
        native.import_wram_image(&ram);
        native.sync_native_game_state_from_ram();
        Self {
            native,
            source,
            cpu,
            clock,
            ram,
            ppu,
            dma,
            apu,
            instructions: 0,
            transfers: 0,
            dma_vblanks: 0,
            visited: BTreeSet::new(),
        }
    }
    fn instruction(&mut self, owner: &mut Owner, data: &BTreeMap<u32, u8>, label: &str) {
        let origin = (u32::from(self.cpu.bank) << 16) | u32::from(self.cpu.pc);
        self.visited.insert(origin);
        let receipt = self.source.step().unwrap();
        assert!(receipt.accepted_interrupt.is_none());
        let mut first = true;
        let expected: Vec<_> = receipt
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
        let is_dma = origin == 0x008cd8;
        assert_eq!(
            is_dma,
            expected
                .iter()
                .any(|(_, a, _)| matches!(a, Access::Write(0x420b, _, _)))
        );
        let mut actual = Vec::new();
        let mut dma_reads = Vec::new();
        let mut dma_writes = 0;
        let mut vblanks = Vec::new();
        let upload_active = match owner {
            Owner::Load(work) => work.upload_active(),
            _ => true,
        };
        let room_parent = matches!(owner, Owner::Load(work) if work.room_parent_active());
        let native = &mut self.native;
        let mut published = false;
        let ram = &mut self.ram;
        let ppu = &mut self.ppu;
        owner.step(
            &mut self.cpu,
            &mut self.clock,
            &mut self.dma,
            self.apu.bus(|at, access| {
                let read = |a: u32| {
                    let a = normalize(a);
                    if a < 0x2000 {
                        ram[a as usize]
                    } else if (0x7e0000..0x800000).contains(&a) {
                        ram[(a - 0x7e0000) as usize]
                    } else {
                        *data.get(&a).unwrap_or_else(|| {
                            panic!("{label} pc{origin:06x}: unbound native data {a:06x}")
                        })
                    }
                };
                let (access, value) = match access {
                    Access::Read(a, width) => {
                        let v = if width == 1 {
                            u16::from(read(a))
                        } else {
                            u16::from_le_bytes([read(a), read(a + 1)])
                        };
                        if is_dma {
                            dma_reads.push((at, a, v, width));
                        }
                        (Access::Read(normalize(a), width), v)
                    }
                    Access::Write(a, value, width) => {
                        let a = normalize(a);
                        if a < 0x2000 || (0x7e0000..0x800000).contains(&a) {
                            let i = if a < 0x2000 {
                                a as usize
                            } else {
                                (a - 0x7e0000) as usize
                            };
                            ram[i..i + usize::from(width)]
                                .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            if upload_active && native.native_dungeon_room_upload_store(a, value, width) {
                                published = true;
                            } else if room_parent && native.native_dungeon_room_parent_store(a, value, width) {
                                // The room caller resets the shared quadrant
                                // cursor before the upload caller resumes.
                            } else {
                                if upload_active {
                                    assert!(matches!(a, 0..=0xf | 0x100..=0x1ff),
                                        "{label} pc{origin:06x}: unowned upload store {a:06x}");
                                }
                                // Other children retain their independent native
                                // output; no source RAM is imported on return.
                                native.ram[i..i + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            }
                        } else if (0x2100..0x2140).contains(&a) {
                            assert_eq!(width, 1);
                            ppu.write(a as u8, value as u8);
                            if is_dma {
                                assert_eq!(a, 0x2118 + (dma_writes & 1));
                                dma_writes += 1;
                            }
                        } else {
                            assert!(
                                matches!(a, 0x420b | 0x420c | 0x4300..=0x437f),
                                "{label}: store {a:06x}"
                            );
                        }
                        (Access::Write(a, value, width), 0)
                    }
                    Access::Vblank { field, at } => {
                        // A mid-transfer video boundary observes partial VRAM. It
                        // cannot be delayed until the whole DMA buffer is uploaded.
                        vblanks.push((field, at, dma_writes));
                        if is_dma {
                            assert!(
                                dma_writes > 0 && dma_writes < 2048,
                                "{label}: delayed DMA video publication"
                            );
                        }
                        return 0;
                    }
                };
                if !is_dma || matches!(access, Access::Write(0x420b, _, _)) {
                    actual.push((at, access, value));
                }
                value
            }),
        );
        self.apu.assert_matches(&self.source, label);
        assert_eq!(*self.native.wram_image(), self.ram, "{label}: native publication bytes");
        if published {
            let state = &self.native.game_state;
            assert_eq!(state.display.pending_nmi_subroutine, self.ram[0x17]);
            assert_eq!(state.display.hdma_enable_mask, self.ram[0x9b]);
            assert_eq!(state.display.nmi_load_target_address,
                crate::types::read_le_u16(&self.ram, 0x116));
            assert_eq!(state.display.core_update_disable_flag, self.ram[0x710]);
            assert_eq!(state.world.overworld.transition.screen_transition_word(),
                crate::types::read_le_u16(&self.ram, 0x418));
            assert_eq!(state.dungeon.room_load.quadrant_upload_index(), self.ram[0x45c]);
            assert_eq!(state.frame.subsubmodule, self.ram[0xb0]);
            assert_eq!(self.native.overworld_map_state(), self.ram[0x200]);
        }
        assert_eq!(actual, expected, "{label} pc{origin:06x}");
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
        let p = &self.source.machine().snes().ppu;
        assert_eq!(self.ppu.vram, p.vram, "{label} pc{origin:06x}");
        assert_eq!(
            (
                self.ppu.vram_pointer,
                self.ppu.vram_increment,
                self.ppu.vram_increment_on_high
            ),
            (p.vram_pointer, p.vram_increment, p.vram_increment_on_high),
            "{label} pc{origin:06x}"
        );
        assert_eq!(
            self.dma.save_c_saveload(),
            self.source.machine().snes().dma.save_c_saveload(),
            "{label} pc{origin:06x}"
        );
        for (_, at, _) in &vblanks {
            let host = self.source.run_until_main_loop_return().unwrap();
            assert_eq!(
                host.instruction_count, 0,
                "{label}: video notification missing from source"
            );
            assert_eq!(
                host.vblank_event_at.master_cycles(),
                *at,
                "{label}: video boundary timestamp"
            );
        }
        if is_dma {
            assert_eq!(dma_reads.len(), 2048);
            assert_eq!(dma_writes, 2048);
            for (i, &(_, a, value, width)) in dma_reads.iter().enumerate() {
                assert_eq!(
                    (a, value, width),
                    (0x1000 + i as u32, u16::from(self.ram[0x1000 + i]), 1)
                );
            }
            assert!(dma_reads.windows(2).all(|v| v[0].0 < v[1].0));
            assert!(owner.dma_end().unwrap() < self.clock.clock_master_cycles());
            self.transfers += 1;
            self.dma_vblanks += vblanks.len();
        }
        *owner = owner.clone();
        self.instructions += 1;
        assert!(self.instructions < 250000, "{label} pc{origin:06x}");
    }
    fn run(&mut self, owner: &mut Owner, data: &BTreeMap<u32, u8>, label: &str) {
        let bank = if matches!(owner, Owner::Upload(TilemapUpload { far: false, .. })) {
            0
        } else {
            2
        };
        while !owner.complete() {
            self.instruction(owner, data, label);
        }
        assert_eq!(
            (self.cpu.bank, self.cpu.pc, self.cpu.sp, self.cpu.p & 0x38),
            (bank, 0xc201, 0x1fd, 0x30)
        );
    }
}

#[test]
fn dungeon_upload_native_tables_match_source_selectors() {
    let data = data();
    let rom = rom();
    for (&a, &v) in &data {
        assert_eq!(
            v,
            rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
            "upload data {a:06x}"
        );
    }
    eprintln!(
        "{} complete native room/upload bytes match source",
        data.len()
    );
}
#[test]
fn dungeon_preparation_matches_cold_source_all_selectors_and_live_buffers() {
    let data = data();
    let rom = rom();
    let mut count = 0;
    let mut scanlines = 0;
    for bg1 in [false, true] {
        for selector in 0..16u16 {
            for variant in 0..4 {
                let mut owner = Owner::Preparation(TilemapPreparation::new(bg1));
                let mut fixture = Fixture::new(&rom, &owner, variant, |ram| {
                    ram[0x0418..0x041a].copy_from_slice(&(0xa500 | (selector & 3)).to_le_bytes());
                    ram[0x045c..0x045e].copy_from_slice(&(selector & !3).to_le_bytes());
                });
                fixture.run(
                    &mut owner,
                    &data,
                    &format!("prep bg1={bg1} selector{selector} variant{variant}"),
                );
                assert_eq!(fixture.transfers, 0);
                count += fixture.instructions;
                scanlines += fixture.apu.lines;
            }
        }
    }
    eprintln!("128 room preparation fixtures matched {count} cold source instructions across both live layers and 16 selectors");
    eprintln!("128 room preparation fixtures retained independent APU hardware across {scanlines} scanlines");
}
#[test]
fn dungeon_tilemap_dma_matches_cold_source_returns_live_bytes_and_video_events() {
    let data = data();
    let rom = rom();
    let (mut count, mut events) = (0, 0);
    let mut scanlines = 0;
    for far in [false, true] {
        for selector in 0..35u8 {
            for variant in 0..4 {
                let mut owner = Owner::Upload(TilemapUpload::new(far));
                let mut fixture = Fixture::new(&rom, &owner, variant, |ram| {
                    ram[0x0116] = selector;
                });
                fixture.run(
                    &mut owner,
                    &data,
                    &format!("upload far={far} selector{selector} variant{variant}"),
                );
                assert_eq!(fixture.transfers, 1);
                count += fixture.instructions;
                events += fixture.dma_vblanks;
                scanlines += fixture.apu.lines;
                assert_eq!(&fixture.ram[0x1000..0x1002], &[0, 0]);
                assert_eq!(fixture.ram[0x0710], 0);
            }
        }
    }
    assert!(events > 0);
    eprintln!("280 tilemap upload fixtures matched {count} cold source instructions with 280 DMAs and {events} in-transfer video events");
    eprintln!("280 tilemap upload fixtures retained independent APU hardware across {scanlines} scanlines");
}
#[test]
fn dungeon_load_and_draw_matches_cold_source_all_rooms_uploads_and_mask_restore() {
    load_and_draw_rooms(0..320);
}

#[test]
fn dungeon_upload_publication_retains_caller_state_across_room_and_dma_returns() {
    load_and_draw_rooms(0..4);
}

fn load_and_draw_rooms(rooms: std::ops::Range<u16>) {
    let cases = rooms.len();
    let data = data();
    let rom = rom();
    let (mut count, mut transfers) = (0, 0);
    let mut scanlines = 0;
    let mut visited = BTreeSet::new();
    for room in rooms {
        let variant = usize::from(room & 3);
        let saved_mask = [0, 0x80, 0x55, 0xff][variant];
        let mut owner = Owner::Load(DungeonLoadAndDraw::default());
        let mut fixture = Fixture::new(&rom, &owner, variant, |ram| {
            ram[0x9b] = saved_mask;
            ram[0x11] = [0, 5, 18, 22][variant];
            ram[0x67] = [0, 2, 4, 8][variant];
            for (a, v) in [
                (0xa0, room),
                (0xa2, room.wrapping_add([1, 0xffff, 16, 0xfff0][variant])),
                (0xe2, [0, 0x01ff, 0xfeff, 0x0200][variant]),
                (0xe8, [0xffff, 0x0200, 0x01ff, 0][variant]),
                (0x045a, [0, 2, 3, 0xffff][variant]),
                (0x2f, (variant * 2) as u16),
            ] {
                ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
            }
            for i in 0..320usize {
                let saved = match variant {
                    0 => 0,
                    1 => 0xffff,
                    2 => 0x8000u16 >> (i % 16),
                    _ => (i as u16).wrapping_mul(251) ^ 0xa59a,
                };
                ram[0xf000 + i * 2..0xf002 + i * 2].copy_from_slice(&saved.to_le_bytes());
            }
            for i in 0..99 {
                ram[0xf940 + i * 4..0xf944 + i * 4].copy_from_slice(&[0xff, 0xff, 0, 0]);
            }
            for i in 0..72 {
                ram[0xfb40 + i * 4..0xfb44 + i * 4].copy_from_slice(&[0xfe, 0xff, 0xff, 0xff]);
            }
            if variant != 0 {
                ram[0xf944..0xf946].copy_from_slice(&room.to_le_bytes());
                ram[0xf946..0xf948].copy_from_slice(&0x0604u16.to_le_bytes());
                ram[0xfb40..0xfb42].copy_from_slice(&room.to_le_bytes());
                ram[0xfb42..0xfb44].copy_from_slice(&0x8208u16.to_le_bytes());
                ram[0xfb44..0xfb46].copy_from_slice(&0xffffu16.to_le_bytes());
            }
        });
        fixture.run(
            &mut owner,
            &data,
            &format!("load/draw room{room:03x} variant{variant}"),
        );
        assert_eq!(fixture.transfers, 8);
        assert_eq!(fixture.ram[0x9b], saved_mask);
        assert_eq!(fixture.ram[0x045c], 16);
        for a in [0x17, 0x0200, 0xb0, 0x0710] {
            assert_eq!(fixture.ram[a], 0);
        }
        count += fixture.instructions;
        transfers += fixture.transfers;
        scanlines += fixture.apu.lines;
        visited.extend(fixture.visited);
    }
    for pc in [
        0x02c57e, 0x02c583, 0x02c590, 0x02c594, 0x02c598, 0x02c59c, 0x02c5a5, 0x02c5a7, 0x02c5b1,
        0x01873a, 0x0188e3, 0x0091d3, 0x00913f, 0x0090e3, 0x008cd8, 0x0090e6,
    ] {
        assert!(
            visited.contains(&pc),
            "missing load/draw continuation {pc:06x}"
        );
    }
    eprintln!("{cases} complete load/draw fixtures matched {count} cold source instructions with {transfers} live tilemap DMAs and restored HDMA shadows");
    eprintln!("{cases} complete load/draw fixtures retained independent APU hardware across {scanlines} scanlines");
}
