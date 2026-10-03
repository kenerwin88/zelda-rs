//! Complete room construction owns each child call and the live stream cursor.
//! Saved objects are drawn only after all layout and room layers have returned.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    dungeon_floors::DungeonFloors,
    dungeon_header::DungeonHeader,
    dungeon_objects::{DungeonObjectTiles, DungeonObjects},
    dungeon_saved_objects::DungeonSavedObjects,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Header(DungeonHeader),
    Floors(DungeonFloors),
    Objects {
        parser: DungeonObjects,
        tiles: Option<DungeonObjectTiles>,
    },
    Saved(DungeonSavedObjects),
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn data() -> BTreeMap<u32, u8> {
        let mut state = crate::ZeldaState::new();
        state
            .set_assets(
                &std::fs::read(
                    std::env::var_os("ZELDA3_ASSETS").expect("room tests require native assets"),
                )
                .unwrap(),
            )
            .unwrap();
        state
            .native_dungeon_room_data()
            .expect("consistent native room streams and headers")
    }
    fn rom() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap()
    }
    #[test]
    fn dungeon_room_native_data_matches_complete_source_records() {
        let data = data();
        let rom = rom();
        for (&a, &v) in &data {
            assert_eq!(
                v,
                rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
                "room data {a:06x}"
            );
        }
        eprintln!(
            "{} complete native room/header/object bytes match source",
            data.len()
        );
    }
    #[test]
    fn dungeon_room_matches_source_all_rooms_layers_saved_objects_and_nested_returns() {
        let data = data();
        let rom = rom();
        let mut visited = BTreeSet::new();
        let mut layouts = BTreeSet::new();
        let (mut cases, mut instructions, mut child_returns) = (0, 0, 0);
        let (mut parent_instructions, mut parent_stores, mut native_child_inputs) = (0, 0, 0);
        let mut parent_domains = BTreeSet::new();
        let mut floor_tile_stores = 0;
        let (mut parser_instructions, mut parser_stores, mut native_draw_inputs) = (0, 0, 0);
        let mut parser_domains = BTreeSet::new();
        let mut parser_positions = BTreeSet::new();
        let mut pending_draw_entries = 0;
        let (mut shared_instructions, mut shared_tile_stores, mut shared_scalar_stores, mut draw_returns) = (0, 0, 0, 0);
        let mut shared_domains = BTreeSet::new();
        let mut saved_stores = 0;
        for room in 0..320u16 {
            for variant in 0..4usize {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
                }
                machine.ram[0x11] = [0, 5, 18, 22][variant];
                machine.ram[0x67] = [0, 2, 4, 8][variant];
                for (a, v) in [
                    (0xa0, room),
                    (0xa2, room.wrapping_add([1, 0xffff, 16, 0xfff0][variant])),
                    (0xe2, [0u16, 0x01ff, 0xfeff, 0x0200][variant]),
                    (0xe8, [0xffff, 0x0200, 0x01ff, 0][variant]),
                    (0x045a, [0, 2, 3, 0xffff][variant]),
                    (0x2f, (variant * 2) as u16),
                ] {
                    machine.ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
                }
                for i in 0..320usize {
                    let saved = match variant {
                        0 => 0,
                        1 => 0xffff,
                        2 => 0x8000u16 >> (i % 16),
                        _ => (i as u16).wrapping_mul(251) ^ 0xa59a,
                    };
                    machine.ram[0xf000 + i * 2..0xf002 + i * 2]
                        .copy_from_slice(&saved.to_le_bytes());
                }
                // Valid saved block and torch lists exercise first/middle/last matches,
                // no match, live background capture and the three-lit-torch threshold.
                for i in 0..99usize {
                    let which = [1, 49, 98].iter().position(|&s| s == i);
                    let matching = variant != 0 && which.is_some();
                    let words = [
                        if matching { room } else { 0xffff },
                        which.map_or(0, |j| [0x0604, 0x4808, 0x0a0c][j]),
                    ];
                    for (j, word) in words.into_iter().enumerate() {
                        machine.ram[0xf940 + i * 4 + j * 2..0xf942 + i * 4 + j * 2]
                            .copy_from_slice(&word.to_le_bytes());
                    }
                }
                let mut torch = Vec::<u16>::new();
                for _ in 0..[72, 0, 35, 69][variant] {
                    torch.extend([0xfffe, 0xffff]);
                }
                if variant != 0 {
                    torch.extend([room, 0x0604, 0x8208, 0x840c, 0x860e, 0xffff]);
                }
                while torch.len() < 144 {
                    torch.extend([0xfffe, 0xffff]);
                }
                assert_eq!(torch.len(), 144);
                for (i, word) in torch.into_iter().enumerate() {
                    machine.ram[0xfb40 + i * 2..0xfb42 + i * 2]
                        .copy_from_slice(&word.to_le_bytes());
                }
                machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x86, 0xc5, 2]);
                let mut cpu = NativeNmiContext {
                    a: 0xab19,
                    x: 0x71,
                    y: 0x83,
                    d: 0,
                    db: 0,
                    bank: 1,
                    pc: 0x873a,
                    sp: 0x1fa,
                    p: 0x31,
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut state = crate::ZeldaState::new();
                state.ram.copy_from_slice(&ram);
                state.sync_native_game_state_from_ram();
                let (v, h) = if variant & 1 == 0 {
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
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = DungeonRoom::default();
                assert_eq!(work.routine(), 0x01873a);
                let mut count = 0;
                while !work.complete() {
                    visited.insert(cpu.pc);
                    if cpu.pc == 0x8826 {
                        layouts.insert(cpu.x / 3);
                    }
                    let had_child = work.child.is_some();
                    let header = work.header_active();
                    let floors = work.floors_active();
                    let parent = work.parent_active();
                    let parser = work.object_parser_active();
                    let shared = work.object_tiles_active();
                    let saved = work.saved_objects_active();
                    // Check the retained owner against the independent source's
                    // parser instruction domain, including the first tail target.
                    let source_cpu = &source.snes().cpu;
                    assert_eq!(parser, source_cpu.k == 1 && (0x88e4..0x89dc).contains(&source_cpu.pc));
                    if let Some(Child::Objects { parser, tiles }) = &work.child {
                        assert_eq!(parser.pending_draw().is_some(), tiles.is_some(),
                            "tail dispatch must retain its drawer before its first instruction");
                    }
                    if parser { parser_positions.insert(cpu.pc); }
                    let mut published = false;
                    let mut wrote = false;
                    let mut tile_write = None;
                    let label = format!("room {room:03x} variant{variant} pc{:04x}", cpu.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| match access {
                            Access::Write(address, value, width) if parent || header || floors || parser || shared || saved => {
                                wrote = true;
                                let handled = if parent {
                                    state.native_dungeon_room_parent_store(address, value, width)
                                } else if floors {
                                    state.native_dungeon_floor_store(address, value, width)
                                } else if parser {
                                    state.native_dungeon_object_parser_store(address, value, width)
                                } else if shared {
                                    state.native_dungeon_object_tile_store(address, value, width)
                                } else if saved {
                                    saved_stores += 1;
                                    state.native_dungeon_saved_object_store(address, value, width)
                                } else {
                                    state.native_dungeon_header_store(address, value, width)
                                };
                                if handled {
                                    if (floors || shared || saved) && (0x7e2000..0x7e6000).contains(&address) {
                                        tile_write = Some(address as usize - 0x7e0000);
                                        if floors { floor_tile_stores += 1; }
                                        else if shared { shared_tile_stores += 1; }
                                    } else {
                                        published = true;
                                        if shared {
                                            shared_scalar_stores += 1;
                                            shared_domains.insert((address, width));
                                        }
                                    }
                                    if parent {
                                        parent_stores += 1;
                                        parent_domains.insert((address, width));
                                    }
                                    if parser {
                                        parser_stores += 1;
                                        parser_domains.insert((address, width));
                                    }
                                } else {
                                    assert!(matches!(address, 0..=0xf | 0x100..=0x1ff),
                                        "{label}: unowned room caller store {address:06x}");
                                    state.ram[address as usize..address as usize + usize::from(width)]
                                        .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                }
                                None
                            }
                            Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                                let mut v = 0;
                                for b in 0..width {
                                    let a = a + u32::from(b);
                                    v |= u16::from(*data.get(&a).unwrap_or_else(|| {
                                        panic!("{label}: unbound room data {a:06x}")
                                    })) << (8 * b);
                                }
                                Some(v)
                            }
                            _ => None,
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    if wrote {
                        assert_eq!(state.ram, ram, "{label}: published native RAM");
                    }
                    if let Some(offset) = tile_write {
                        assert_eq!(state.game_state.dungeon.room_tilemaps.bg2_tile((offset - 0x2000) / 2),
                            crate::types::read_le_u16(&ram, offset), "{label}: floor tile owner");
                    }
                    if published || (floors && work.child.is_none()) {
                        let mut expected = crate::game_state::GameState::load_from_ram(&ram);
                        // R16/R18 alias the active line pointers. Their legacy
                        // scratch view is inactive while the room owns them.
                        expected.dungeon.scratch_word = state.game_state.dungeon.scratch_word;
                        assert!(state.game_state.dungeon == expected.dungeon,
                            "{label}: {:?}", state.game_state.dungeon.report_incoherent_with_ram(&ram));
                        assert_eq!(state.game_state.display, expected.display, "{label}: display owner");
                        assert_eq!(state.game_state.player.pushed_block, expected.player.pushed_block,
                            "{label}: adjacent pushed-block banks");
                        // Tile-detection fields alias this routine's direct-page
                        // scratch. Only the BG1 accumulator is player-owned here.
                        assert_eq!(state.game_state.player.bg1_movement_accumulator,
                            expected.player.bg1_movement_accumulator, "{label}: BG1 movement owner");
                        assert_eq!(state.game_state.player.follower_link.temp_bunny_timer(),
                            expected.player.follower_link.temp_bunny_timer(), "{label}: Somaria word companion");
                    }
                    parent_instructions += usize::from(parent);
                    parser_instructions += usize::from(parser);
                    shared_instructions += usize::from(shared);
                    if parser && matches!(&work.child, Some(Child::Objects { tiles: Some(_), .. })) {
                        pending_draw_entries += 1;
                    }
                    if !parser && work.object_parser_active() {
                        if !parent {
                            draw_returns += 1;
                            // Shared and saved-object drawers return continuously
                            // through their owners, without any RAM import.
                            if !shared && !saved {
                                state.ram.copy_from_slice(&ram);
                                state.sync_native_game_state_from_ram();
                                native_draw_inputs += 1;
                            }
                        }
                    }
                    if had_child && work.child.is_none() {
                        child_returns += 1;
                        if !header && !floors && !parser && !saved {
                            // Remaining specialized children provide their
                            // independently computed native output to this fixture.
                            state.ram.copy_from_slice(&ram);
                            state.sync_native_game_state_from_ram();
                            native_child_inputs += 1;
                        }
                    }
                    work = work.clone();
                    count += 1;
                    assert!(count < 150000, "{label}");
                }
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.p & 0x38),
                    (2, 0xc587, 0x1fd, 0x30)
                );
                assert!(work.child.is_none());
                cases += 1;
                instructions += count;
            }
        }
        for pc in [
            0x873a, 0x880f, 0x8834, 0x8837, 0x884a, 0x885f, 0x8874, 0x888a, 0x889a, 0x88b3, 0x88c5,
            0x88d3, 0x88e3, 0xb4d6, 0xb4ef, 0xb506, 0xb509, 0xb521, 0xb523, 0xb52e, 0xb539, 0x9895,
            0x98ad,
        ] {
            assert!(visited.contains(&pc), "missing room branch {pc:04x}");
        }
        assert_eq!(layouts.len(), 8);
        eprintln!("{cases} complete room loads matched {instructions} instructions and {child_returns} parent child returns across {} default layouts", layouts.len());
        eprintln!("room parent native publications: instructions={parent_instructions} stores={parent_stores} domains={} native_child_inputs={native_child_inputs}", parent_domains.len());
        assert_eq!(floor_tile_stores, cases * 8192);
        eprintln!("room composed floor publications: {floor_tile_stores} tile stores with continuous parent/header/floor ownership");
        assert_eq!(parser_domains, BTreeSet::from([(0xb2, 1), (0xb2, 2), (0xb4, 1), (0xb4, 2), (0xba, 2)]));
        assert_eq!(pending_draw_entries, draw_returns);
        eprintln!("room object parser publications: instructions={parser_instructions} stores={parser_stores} domains={} positions={} native_draw_inputs={native_draw_inputs} pending_draw_entries={pending_draw_entries}", parser_domains.len(), parser_positions.len());
        eprintln!("room shared object publications: instructions={shared_instructions} tiles={shared_tile_stores} scalars={shared_scalar_stores} domains={shared_domains:x?} continuous_returns={}", draw_returns - native_draw_inputs);
        assert!(saved_stores > 0);
        eprintln!("room saved-object publication stores: {saved_stores}");
    }
}
impl Child {
    fn complete(&self) -> bool {
        match self {
            Self::Header(w) => w.complete(),
            Self::Floors(w) => w.complete(),
            Self::Objects { parser, .. } => parser.complete(),
            Self::Saved(w) => w.complete(),
        }
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        match self {
            Self::Header(w) => w.step(cpu, clock, bus),
            Self::Floors(w) => w.step(cpu, clock, bus),
            Self::Saved(w) => w.step(cpu, clock, bus),
            Self::Objects { parser, tiles } => {
                if let Some(draw) = tiles.as_mut() {
                    let cycles = draw.step(cpu, clock, bus);
                    if draw.complete() {
                        *tiles = None;
                        parser.resume_after_draw(cpu);
                    }
                    cycles
                } else {
                    let cycles = parser.step(cpu, clock, bus);
                    // Retain the dispatched child when the parser retires its
                    // jump. Its owner is now available before the next instruction.
                    if let Some(target) = parser.pending_draw() {
                        *tiles = Some(DungeonObjectTiles::new(target).expect("native room drawer"));
                    }
                    cycles
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonRoom {
    pc: u16,
    child: Option<Child>,
    complete: bool,
}
impl Default for DungeonRoom {
    fn default() -> Self {
        Self {
            pc: 0x873a,
            child: None,
            complete: false,
        }
    }
}
impl DungeonRoom {
    pub(crate) fn door_state_active(&self) -> bool {
        matches!(&self.child, Some(Child::Objects { tiles: Some(draw), .. })
            if draw.door_state_active())
    }

    pub(crate) fn saved_objects_active(&self) -> bool {
        matches!(&self.child, Some(Child::Saved(_)))
            || matches!(&self.child, Some(Child::Objects { tiles: Some(draw), .. })
                if draw.saved_objects_active())
    }

    pub(crate) fn object_tiles_active(&self) -> bool {
        matches!(&self.child, Some(Child::Objects { tiles: Some(draw), .. })
            if draw.shared_tiles_active())
    }

    pub(crate) fn object_parser_active(&self) -> bool {
        // The parser's live return remains pending until its retained drawer
        // retires the real RTS, including the drawer's first instruction.
        matches!(&self.child, Some(Child::Objects { parser, tiles })
            if tiles.is_none() && parser.pending_draw().is_none())
    }

    pub(crate) fn floors_active(&self) -> bool {
        matches!(self.child, Some(Child::Floors(_)))
    }

    pub(crate) fn parent_active(&self) -> bool {
        self.child.is_none()
    }

    pub(crate) fn header_active(&self) -> bool {
        matches!(self.child, Some(Child::Header(_)))
    }

    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x01873a
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
        if let Some(child) = self.child.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if child.complete() {
                self.child = None;
            }
            return cycles;
        }
        let byte = matches!(self.pc, 0x873a | 0x873d | 0x8740 | 0x88e3);
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (1, self.pc, 0, 0, if byte { 0x30 } else { 0 })
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x873a | 0x880f | 0x8834 | 0x884a | 0x885f | 0x8874 | 0x888a | 0x88d3 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                let (target, child) = match self.pc {
                    0x873a => (0xb564, Child::Header(DungeonHeader::default())),
                    0x880f => (0x89dc, Child::Floors(DungeonFloors::default())),
                    0x888a => (0xb4d6, Child::Saved(DungeonSavedObjects::new(0xb4d6))),
                    0x88d3 => (0xb509, Child::Saved(DungeonSavedObjects::new(0xb509))),
                    _ => (
                        0x88e4,
                        Child::Objects {
                            parser: DungeonObjects::default(),
                            tiles: None,
                        },
                    ),
                };
                self.child = Some(child);
                target
            }
            0x873d => {
                w.drain(16);
                w.write(0x03f4, 0, 1);
                0x8740
            }
            0x8740 | 0x889a | 0x88e1 => {
                w.width(
                    if self.pc == 0x889a { 0x20 } else { 0x30 },
                    self.pc != 0x88e1,
                );
                self.pc + 2
            }
            0x8742 | 0x883b => {
                w.drain(16);
                w.cpu.x = w.read(0x0110, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x8745 | 0x874b | 0x8826 | 0x882c | 0x883e | 0x8844 | 0x8854 | 0x8869 | 0x887b
            | 0x8883 | 0x88a9 | 0x88b3 | 0x88c9 | 0x88d8 => {
                let base = match self.pc {
                    0x8745 | 0x883e => 0x1f8001,
                    0x874b | 0x8844 => 0x1f8000,
                    0x8826 => 0x04ef30,
                    0x882c => 0x04ef2f,
                    0x8854 => 0x01871a,
                    0x8869 => 0x0186f9,
                    0x887b => 0x7ef940,
                    0x8883 => 0x7ef942,
                    _ => 0x7efb40,
                };
                w.drain(24);
                let v = w.read(base + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0x8749 | 0x874f | 0x881f | 0x882a | 0x8830 | 0x8842 | 0x8848 | 0x8887 | 0x8893
            | 0x88cd => {
                let a = match self.pc {
                    0x8749 | 0x882a | 0x8842 => 0xb8,
                    0x874f | 0x8830 | 0x8848 => 0xb7,
                    0x881f => 0,
                    0x8893 => 0xba,
                    _ => 8,
                };
                w.dp_store(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8751 | 0x888d => {
                let v = w.dp_read(if self.pc == 0x8751 { 0xad } else { 0xba }, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x8753 | 0x8759 | 0x8771 | 0x881a | 0x889f | 0x88a2 => {
                let a = match self.pc {
                    0x8753 => 0x0428,
                    0x8759 => 0x041c,
                    0x8771 => 0x0436,
                    0x881a => 0x040e,
                    0x889f => 0x042e,
                    _ => 0x0478,
                };
                w.drain(16);
                w.write(a, w.cpu.a, 2);
                self.pc + 3
            }
            0x8756 | 0x876e => {
                w.immediate(if self.pc == 0x8756 { 0xff30 } else { 0xffff }, 2);
                self.pc + 3
            }
            0x875c | 0x875f | 0x8762 | 0x8765 | 0x8768 | 0x876b | 0x8774 | 0x8777 | 0x877a
            | 0x877d | 0x8780 | 0x8783 | 0x8788 | 0x878b | 0x878e | 0x8791 | 0x8794 | 0x8797
            | 0x879a | 0x879d | 0x87a0 | 0x87a3 | 0x87a6 | 0x87a9 | 0x87ac | 0x87af | 0x87b2
            | 0x87b5 | 0x87b8 | 0x87bb | 0x87be | 0x87c1 | 0x87c4 | 0x87c7 | 0x87ca | 0x87cd
            | 0x87d0 | 0x87d3 | 0x87d6 | 0x87d9 | 0x87dc | 0x87df | 0x87e2 | 0x87e5 | 0x87e8
            | 0x87eb | 0x87f1 => {
                let a = match self.pc {
                    0x875c => 0x041a,
                    0x875f => 0x0420,
                    0x8762 => 0x0312,
                    0x8765 => 0x0310,
                    0x8768 => 0x0422,
                    0x876b => 0x0424,
                    0x8774 => 0x0452,
                    0x8777 => 0x0454,
                    0x877a => 0x0456,
                    0x877d => 0x068a,
                    0x8780 => 0x044e,
                    0x8783 => 0x0450,
                    0x8788 => 0x045c,
                    0x878b => 0x0438,
                    0x878e => 0x043a,
                    0x8791 => 0x043c,
                    0x8794 => 0x043e,
                    0x8797 => 0x0440,
                    0x879a => 0x0442,
                    0x879d => 0x0444,
                    0x87a0 => 0x0446,
                    0x87a3 => 0x0448,
                    0x87a6 => 0x049a,
                    0x87a9 => 0x049c,
                    0x87ac => 0x049e,
                    0x87af => 0x04ae,
                    0x87b2 => 0x047e,
                    0x87b5 => 0x0480,
                    0x87b8 => 0x0482,
                    0x87bb => 0x0484,
                    0x87be => 0x04a2,
                    0x87c1 => 0x04a4,
                    0x87c4 => 0x04a6,
                    0x87c7 => 0x04a8,
                    0x87ca => 0x19e2,
                    0x87cd => 0x19e4,
                    0x87d0 => 0x19e6,
                    0x87d3 => 0x19e8,
                    0x87d6 => 0x19e0,
                    0x87d9 => 0x0430,
                    0x87dc => 0x0432,
                    0x87df => 0x042c,
                    0x87e2 => 0x042e,
                    0x87e5 => 0x0496,
                    0x87e8 => 0x0498,
                    0x87eb => 0x04b0,
                    _ => 0x0460,
                };
                w.drain(16);
                w.write(a, 0, 2);
                self.pc + 3
            }
            0x8786 | 0x880d | 0x8832 | 0x8877 | 0x88a5 => {
                w.dp_store(if self.pc == 0x8786 { 0xfc } else { 0xba }, 0, 2);
                self.pc + 2
            }
            0x87ee | 0x8851 | 0x8866 => {
                w.drain(16);
                w.cpu.x = 30;
                w.flags(30, 2);
                self.pc + 3
            }
            0x87f4 | 0x87f7 | 0x87fa | 0x87fd | 0x8800 | 0x8803 | 0x8806 => {
                let base: u16 = match self.pc {
                    0x87f4 => 0x19a0,
                    0x87f7 => 0x1980,
                    0x87fa => 0x19c0,
                    0x87fd => 0x04f0,
                    0x8800 => 0x0500,
                    0x8803 => 0x0520,
                    _ => 0x0540,
                };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.x)), 0, 2);
                self.pc + 3
            }
            0x8809 | 0x880a | 0x885a | 0x885b | 0x885c | 0x886f | 0x8870 | 0x8871 => {
                w.decrement_x();
                self.pc + 1
            }
            0x880b | 0x885d | 0x8872 | 0x8881 | 0x8898 | 0x88af | 0x88be | 0x88c3 | 0x88df => {
                let taken = match self.pc {
                    0x880b | 0x885d | 0x8872 => w.cpu.p & NEGATIVE == 0,
                    0x88af => w.cpu.p & ZERO != 0,
                    _ => w.cpu.p & ZERO == 0,
                };
                let target = match self.pc {
                    0x880b => 0x87f4,
                    0x885d => 0x8854,
                    0x8872 => 0x8869,
                    0x8881 => 0x888d,
                    0x8898 => 0x8879,
                    0x88af => 0x88c7,
                    0x88be => 0x88b3,
                    0x88c3 => 0x88a7,
                    _ => 0x88c9,
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x8812 => {
                w.cpu.y = w.dp_read(0xba, 2);
                w.flags(w.cpu.y, 2);
                0x8814
            }
            0x8814 => {
                w.drain(6);
                w.push(w.cpu.y, 2);
                0x8815
            }
            0x8815 => {
                let a = w.indirect_y_address(0xb7);
                let v = w.read(a, 2);
                w.load(v, 2);
                0x8817
            }
            0x8817 => {
                w.mask(255, 2);
                0x881a
            }
            0x881d | 0x881e => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0x8821 => {
                w.shift_left(2);
                0x8822
            }
            0x8822 | 0x888f => {
                w.clear_carry();
                self.pc + 1
            }
            0x8823 => {
                let v = w.dp_read(0, 2);
                w.adc(v);
                0x8825
            }
            0x8825 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                0x8826
            }
            0x8837 => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                0x8838
            }
            0x8838 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.flags(w.cpu.y, 2);
                0x8839
            }
            0x8839 => {
                w.dp_store(0xba, w.cpu.y, 2);
                0x883b
            }
            0x884d | 0x884f | 0x8862 | 0x8864 => {
                w.dp_increment(0xba, 2, true);
                self.pc + 2
            }
            0x8858 | 0x886d => {
                w.operand_then_internal(1);
                w.write(u32::from(0xc0u16.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 2
            }
            0x8879 | 0x88a7 | 0x88d6 => {
                w.cpu.x = w.dp_read(0xba, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0x887f | 0x88ad => {
                let v = w.dp_read(0xa0, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a >= v);
                w.flags(w.cpu.a.wrapping_sub(v), 2);
                self.pc + 2
            }
            0x8889 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0x888a
            }
            0x8890 => {
                w.drain(16);
                w.adc(4);
                0x8893
            }
            0x8895 | 0x88bb | 0x88dc => {
                w.compare_immediate(if self.pc == 0x8895 { 0x018c } else { 0xffff }, 2);
                self.pc + 3
            }
            0x889c => {
                w.drain(16);
                let v = w.read(0x042c, 2);
                w.load(v, 2);
                0x889f
            }
            0x88b1 | 0x88b2 | 0x88b7 | 0x88b8 | 0x88c7 | 0x88c8 | 0x88cf | 0x88d0 => {
                w.increment_x();
                self.pc + 1
            }
            0x88b9 | 0x88d1 => {
                w.dp_store(0xba, w.cpu.x, 2);
                self.pc + 2
            }
            0x88c0 => {
                w.compare_x_immediate(0x0120, 2);
                0x88c3
            }
            0x88c5 => {
                w.branch(true);
                0x88e1
            }
            0x88e3 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("room construction continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
