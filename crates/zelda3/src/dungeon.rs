//! Dungeon subsystem modules, shared work cursors, and regression tests.
// Methods ported from zelda3/src/dungeon.c and included inside ZeldaState.

use super::*;
#[cfg(test)]
use crate::tile_definition::NativeTile;

#[path = "dungeon/entrance.rs"]
mod entrance;
#[path = "dungeon/room_load.rs"]
mod room_load;
#[path = "dungeon/objects.rs"]
mod objects;
#[path = "dungeon/object_draw.rs"]
mod object_draw;
#[path = "dungeon/doors.rs"]
mod doors;
#[path = "dungeon/attributes.rs"]
mod attributes;
#[path = "dungeon/room_tags.rs"]
mod room_tags;
#[path = "dungeon/interaction.rs"]
mod interaction;
#[path = "dungeon/transitions.rs"]
mod transitions;
#[path = "dungeon/stairs.rs"]
mod stairs;
#[path = "dungeon/effects.rs"]
mod effects;
#[path = "dungeon/runtime.rs"]
mod runtime;

mod dungeon_shared;
use dungeon_shared::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) enum SelectedGameEntranceContinuation {
    BeforeSelection,
    AfterScroll { selection: (usize, bool) },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) enum EarlyDungeonRoomOwner {
    FallingEntrance,
    PreDungeon,
}

/// One translated floor draw, advanced in the ROM's tile-store order. Both
/// layers contain four quadrants of eight rows of eight 4x4 tile blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct DungeonFloorDrawWork {
    room: u16,
    floor_types: u8,
    completed_words: u16,
}

/// The object stream has two sections. Keeping the section in the caller's
/// work item lets the translated parser return between complete objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DungeonObjectStreamProgress {
    in_doors: bool,
}

/// A statement boundary in RoomDraw_Downwards4x2VariableSpacing. The loop
/// counter still includes the current block until its DEC has retired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Dungeon4x2DrawBoundary {
    pub remaining_blocks: u16,
    pub words_in_block: u8,
}

impl Dungeon4x2DrawBoundary {
    pub(super) fn from_source_pc(pc: u32, remaining_blocks: u16) -> Option<Self> {
        let words_in_block = match pc {
            0x01_b222..=0x01_b225 | 0x01_b251..=0x01_b253 => 0,
            0x01_b227..=0x01_b22a => 1,
            0x01_b22c..=0x01_b22f => 2,
            0x01_b231..=0x01_b234 => 3,
            0x01_b236..=0x01_b239 => 4,
            0x01_b23b..=0x01_b23e => 5,
            0x01_b240..=0x01_b243 => 6,
            0x01_b245..=0x01_b248 => 7,
            0x01_b24a..=0x01_b24f => 8,
            _ => return None,
        };
        Some(Self { remaining_blocks, words_in_block })
    }
}

/// Native arguments and publication cursor of the shared 4x2 tail call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Dungeon4x2DrawWork {
    base: usize,
    increment: u16,
    src: usize,
    dsto: u16,
    count: u16,
    completed_words: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct DungeonObjectDrawWork {
    room: u16,
    first_room_stream_offset: u16,
    phase: u8,
    stream: DungeonObjectStreamProgress,
    completed_objects: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct DungeonRoomUploadWork {
    completed_quadrants: u8,
}

/// Native arguments and store cursor of the shared 1024-word quadrant copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonQuadrantUploadWork {
    source_base: usize,
    table_index: usize,
    dst_bias: u8,
    completed_words: u16,
}

impl DungeonQuadrantUploadWork {
    /// Source Y is a destination loop cursor, not a tile value. Between the
    /// four INYs account for only the already-published block's stores.
    pub(super) fn completed_words_at_source_pc(pc: u32, y: u16) -> Option<u16> {
        let (y, words) = match pc {
            0x00_9153 | 0x00_9157 => (y, 0),
            0x00_915a | 0x00_915e => (y, 1),
            0x00_9161 | 0x00_9165 => (y, 2),
            0x00_9168 | 0x00_916c => (y, 3),
            0x00_916f | 0x00_9173 => (y, 4),
            0x00_9176 | 0x00_917a => (y, 5),
            0x00_917d | 0x00_9181 => (y, 6),
            0x00_9184 | 0x00_9188 => (y, 7),
            0x00_918b..=0x00_918f => (y, 8),
            0x00_9190..=0x00_9192 => (y.checked_sub((pc - 0x00_918f) as u16)?, 8),
            0x00_9193 | 0x00_9194 | 0x00_9197 | 0x00_9199 | 0x00_919a
                | 0x00_919b | 0x00_919e | 0x00_919f | 0x00_91a0
                | 0x00_91a1 | 0x00_91a4 | 0x00_91a5 | 0x00_91a8
                | 0x00_91aa => (y, 0),
            // SEP truncates Y after all stores. The index store is still owed.
            0x00_91ab | 0x00_91ad | 0x00_91b0 | 0x00_91b1 | 0x00_91b3 => return Some(1024),
            _ => return None,
        };
        let completed = (y / 256) * 128 + (y % 256 / 4) * 8 + words;
        (completed <= 1024).then_some(completed)
    }
}

impl DungeonRoomUploadWork {
    pub(super) fn completed_quadrants(self) -> u8 {
        self.completed_quadrants
    }
}

impl DungeonObjectDrawWork {
    pub(super) fn finished(self) -> bool {
        self.phase == 4
    }

    pub(super) fn completed_objects(self) -> u16 {
        self.completed_objects
    }
}

impl DungeonFloorDrawWork {
    pub(super) const TOTAL_WORDS: u16 = 8192;

    pub(super) fn completed_words(self) -> u16 {
        self.completed_words
    }

    fn next_tile(self) -> (usize, usize, usize, u32) {
        assert!(self.completed_words < Self::TOTAL_WORDS);
        let ordinal = usize::from(self.completed_words);
        let layer = ordinal / 4096;
        let layer_word = ordinal % 4096;
        let quadrant = layer_word / 1024;
        let row = (layer_word % 1024) / 128;
        let block = (layer_word % 128) / 16;
        let vertical_pass = (layer_word % 16) / 8;
        let y = (layer_word % 8) / 4;
        let x = layer_word % 4;
        let base = if layer == 0 {
            ROOM_BG1_TILEMAP_BASE
        } else {
            ROOM_BG2_TILEMAP_BASE
        };
        let source = if layer == 0 {
            usize::from(self.floor_types & 0xf0)
        } else {
            usize::from((self.floor_types & 0x0f) << 4)
        };
        let byte_offset = DUNGEON_QUADRANT_OFFSETS[quadrant]
            + row * xy(0, 4) * 2
            + block * xy(4, 0) * 2
            + vertical_pass * xy(0, 2) * 2
            + xy(x, y) * 2;
        (base, byte_offset, source + (y * 4 + x) * 2,
            0x7e_0000 + u32::try_from(base + byte_offset).unwrap())
    }
}

#[cfg(test)]
mod floor_draw_tests {
    use super::*;

    #[test]
    fn resumable_floor_stores_reproduce_the_eager_room_floor() {
        let mut eager = ZeldaState::new();
        let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("zelda3_assets.dat");
        eager.assets = Some(AssetPack::parse(&std::fs::read(asset_path).unwrap()).unwrap());
        eager.set_dungeon_room(0x55);
        eager.dungeon_load_header();
        eager.dungeon_load_room_before_floors();
        let mut resumed = eager.clone();
        eager.room_draw_draw_floors_current_room();

        let mut work = resumed.begin_dungeon_floor_draw_current_room().unwrap();
        for ordinal in 1..=DungeonFloorDrawWork::TOTAL_WORDS {
            let (_, _, source_offset, address) = work.next_tile();
            let value = read_word_from_slice(
                resumed.asset_raw(69).unwrap(), source_offset,
            );
            resumed.advance_dungeon_floor_draw_one(&mut work, ordinal, address, value);
        }
        assert_eq!(work.completed_words(), DungeonFloorDrawWork::TOTAL_WORDS);
        assert_eq!(resumed.ram, eager.ram);
        assert_eq!(resumed.game_state.dungeon.room_tilemaps,
            eager.game_state.dungeon.room_tilemaps);
    }

    #[test]
    fn straight_room_initialization_resumes_shared_draw_and_graphics_before_its_tail() {
        let mut eager = ZeldaState::new();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../zelda3_assets.dat");
        eager.assets = Some(AssetPack::parse(&std::fs::read(path).unwrap()).unwrap());
        eager.set_main_module(7);
        eager.set_submodule(0x12);
        eager.set_subsubmodule(2);
        eager.set_indoor_flag(1);
        eager.set_dungeon_room(0x51);
        let torch_init = eager.asset_raw(54).unwrap().to_vec();
        eager.dungeon_torch_mut().copy_torch_data_table(&torch_init);
        let mut staged = eager.clone();
        eager.dungeon_load_room();
        eager.complete_straight_room_initialization_after_room_load();

        staged.begin_live_dungeon_supertile_room_load();
        assert_eq!(read_le_u16(&staged.ram, 0x043e), 0,
            "stair objects have not run at room-draw entry");
        let mut floor = staged.active_early_dungeon_floor_draw.take().unwrap();
        for ordinal in 1..=DungeonFloorDrawWork::TOTAL_WORDS {
            let (_, _, offset, address) = floor.next_tile();
            let value = read_word_from_slice(staged.asset_raw(69).unwrap(), offset);
            staged.advance_dungeon_floor_draw_one(&mut floor, ordinal, address, value);
        }
        staged.active_early_dungeon_floor_draw = Some(floor);
        assert_eq!(read_le_u16(&staged.ram, 0x043e), 0);
        assert_ne!(staged.game_state.dungeon.room_tilemaps, eager.game_state.dungeon.room_tilemaps,
            "completed floors must not publish the later room objects");
        let mut objects = staged.begin_dungeon_object_draw_current_room().unwrap();
        while staged.advance_dungeon_object_draw_one(&mut objects) {}
        staged.active_early_dungeon_object_draw = Some(objects);
        staged.finish_live_dungeon_supertile_room_load();
        staged.dungeon_restore_star_tile_chr();
        let prefix = load_gfx::TransitionAuxGraphicsProgress { sheet: 0, completed_bytes: 0x100 };
        staged.advance_transition_aux_graphics(None, prefix);
        staged.native_transition_aux_graphics_progress = Some(prefix);
        assert_eq!(staged.game_state.frame.subsubmodule, 2,
            "the straight-stair caller cannot advance before initialization returns");
        staged.complete_straight_room_initialization_after_room_load();
        assert!(staged.active_early_dungeon_floor_draw.is_none());
        assert!(staged.native_transition_aux_graphics_progress.is_none());
        assert_eq!(*staged.wram_image(), *eager.wram_image());
        assert_eq!(staged.game_state, eager.game_state);
    }

    #[test]
    fn resumable_object_stream_reproduces_the_eager_room_objects() {
        let mut eager = ZeldaState::new();
        let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("zelda3_assets.dat");
        eager.assets = Some(AssetPack::parse(&std::fs::read(asset_path).unwrap()).unwrap());
        eager.set_dungeon_room(0x55);
        eager.dungeon_load_header();
        eager.dungeon_load_room_before_floors();
        eager.room_draw_draw_floors_current_room();
        let mut resumed = eager.clone();
        eager.dungeon_room_tilemaps_mut()
            .copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        eager.room_draw_draw_all_objects_current_room();

        let mut work = resumed.begin_dungeon_object_draw_current_room().unwrap();
        let mut object_count = 0;
        while resumed.advance_dungeon_object_draw_one(&mut work) {
            object_count += 1;
        }
        assert!(work.finished());
        assert!(object_count > 0);
        assert_eq!(resumed.ram, eager.ram);
        assert_eq!(resumed.game_state.dungeon, eager.game_state.dungeon);
    }

    #[test]
    fn four_by_two_object_retains_each_source_store_and_resumes_once() {
        let mut initial = ZeldaState::new();
        let data: Vec<u8> = (0x1001u16..=0x1008).flat_map(u16::to_le_bytes).collect();
        let mut ranges = vec![(0, 0); 70];
        ranges[69] = (0, data.len());
        initial.assets = Some(AssetPack::from_data_ranges(data, ranges));
        initial.ram[0x2000..0x6000].fill(0xbe);
        initial.sync_native_game_state_from_ram();
        initial.dungeon_room_tilemaps_mut()
            .copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        let mut eager = initial.clone();
        eager.room_draw_downwards4x2_variable_spacing(128, 0, 5, 3);
        for blocks in 0..3 {
            for words in 0..=8 {
                let mut state = initial.clone();
                let boundary = Dungeon4x2DrawBoundary {
                    remaining_blocks: 3 - blocks, words_in_block: words,
                };
                state.pending_dungeon_object_4x2_boundary = Some(boundary);
                state.room_draw_downwards4x2_variable_spacing(128, 0, 5, 3);
                let target = blocks * 8 + u16::from(words);
                for i in 0..24u16 {
                    let field = i % 8;
                    let dst = 5 + (i / 8) * 128 + field % 4 + (field / 4) * 64;
                    assert_eq!(state.room_read_current(dst),
                        if i < target { 0x1001 + field } else { 0xbebe },
                        "uncommitted field at boundary {boundary:?}, word {i}");
                }
                assert_eq!(state.game_state.dungeon.room_load.draw_width_indicator(),
                    (3 - blocks) as u8);
                assert!(!state.paired_resume_cpu_boundary_is_quiescent());
                let mut work = DungeonObjectDrawWork {
                    room: 0, first_room_stream_offset: 0, phase: 0,
                    stream: DungeonObjectStreamProgress::default(), completed_objects: 2,
                };
                let before_repeat = state.ram.clone();
                state.advance_dungeon_object_4x2_prefix(&mut work, 3, boundary);
                assert_eq!(state.ram, before_repeat);
                assert_eq!(work.completed_objects(), 2);
                assert!(state.advance_dungeon_object_draw_one(&mut work));
                assert_eq!(work.completed_objects(), 3);
                assert!(state.active_dungeon_object_4x2.is_none());
                assert_eq!(*state.wram_image(), *eager.wram_image());
                assert_eq!(state.game_state.dungeon, eager.game_state.dungeon);
            }
        }
    }

    #[test]
    fn four_by_two_boundary_distinguishes_pending_store_and_loop_decrement() {
        assert_eq!(Dungeon4x2DrawBoundary::from_source_pc(0x01_b22c, 3),
            Some(Dungeon4x2DrawBoundary { remaining_blocks: 3, words_in_block: 2 }));
        assert_eq!(Dungeon4x2DrawBoundary::from_source_pc(0x01_b24f, 3),
            Some(Dungeon4x2DrawBoundary { remaining_blocks: 3, words_in_block: 8 }));
        assert_eq!(Dungeon4x2DrawBoundary::from_source_pc(0x01_b251, 2),
            Some(Dungeon4x2DrawBoundary { remaining_blocks: 2, words_in_block: 0 }));
        assert_eq!(Dungeon4x2DrawBoundary::from_source_pc(0x01_b254, 0), None);
    }

    #[test]
    fn resumable_quadrant_upload_reproduces_the_eager_room_upload() {
        let mut eager = ZeldaState::new();
        let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("zelda3_assets.dat");
        eager.assets = Some(AssetPack::parse(&std::fs::read(asset_path).unwrap()).unwrap());
        eager.set_dungeon_room(0x55);
        eager.dungeon_load_header();
        eager.dungeon_load_room_before_floors();
        eager.room_draw_draw_floors_current_room();
        let mut resumed = eager.clone();
        eager.dungeon_upload_room_quadrants();
        let mut work = resumed.begin_dungeon_room_upload();
        for count in 1..=4 {
            resumed.advance_dungeon_room_upload_one(&mut work);
            assert_eq!(work.completed_quadrants(), count);
        }
        resumed.finish_dungeon_room_upload(work);
        assert_eq!(resumed.ram, eager.ram);
        assert_eq!(resumed.game_state.dungeon, eager.game_state.dungeon);
        assert_eq!(resumed.ppu.vram, eager.ppu.vram);
    }

    #[test]
    fn quadrant_copy_cursor_matches_rom_stores_and_defers_publication() {
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for quadrant in [0, 4, 8, 12] {
            for direction in 0..4 {
                let mut state = ZeldaState::new();
                for (i, bytes) in state.ram[0x2000..0x6000].chunks_exact_mut(2).enumerate() {
                    bytes.copy_from_slice(&(0x8000 | i as u16).to_le_bytes());
                }
                state.ram[0x1000..0x1800].fill(0x5a);
                state.ram[0x0418] = direction;
                state.ram[0x045c] = quadrant;
                state.sync_native_game_state_from_ram();
                state.set_nmi_load_target_page(0xa5);
                state.set_pending_nmi_subroutine(9);
                state.set_core_update_disable_flag(9);
                let mut eager = state.clone();
                let checkpoint = RomCpuCheckpoint {
                    entry_pc: 0x00_913f, stop_pc: 0x00_91c3, a: 0, x: 0, y: 0,
                    sp: 0x01fc, dp: 0, db: 0, carry: false, zero: false,
                    overflow: false, negative: false, interrupt_disable: true,
                    decimal: false, accumulator_is_8_bit: true, index_is_8_bit: true,
                    emulation: false, waiting: false, stack_address: 0x01fd,
                    stack_bytes: &[0xff, 0x7f, 0],
                };
                let mut cpu = RomCpuTimingRun::new(&rom, &state.wram_image(), &state.sram,
                    &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
                cpu.enable_cpu_write_trace();
                let mut work = state.begin_dungeon_quadrant_upload(ROOM_BG2_TILEMAP_BASE,
                    usize::from(quadrant), 0);
                let mut source_stores = 0;
                for _ in 0..6000 {
                    if let Some(words) = DungeonQuadrantUploadWork::completed_words_at_source_pc(cpu.pc(), cpu.index_y()) {
                        assert_eq!(words, source_stores, "pc={:06x} y={:04x}", cpu.pc(), cpu.index_y());
                        state.advance_dungeon_quadrant_upload(&mut work, words);
                        assert_eq!(&state.ram[0x1000..0x1800], &cpu.ram()[0x1000..0x1800]);
                        assert_eq!(state.ram[0x045c], quadrant);
                        assert_eq!(state.ram[0x0116], 0xa5);
                        assert_eq!(state.ram[0x17], 9);
                        assert_eq!(state.ram[0x0710], 9);
                    }
                    if cpu.is_complete() { break; }
                    cpu.step();
                    source_stores += (cpu.take_cpu_wram_writes().iter()
                        .filter(|(address, _)| (0x1000..0x1800).contains(address)).count() / 2) as u16;
                }
                assert!(cpu.is_complete());
                assert_eq!(source_stores, 1024);
                state.complete_dungeon_quadrant_upload(work);
                assert_eq!(&state.ram[0x1000..0x1800], &cpu.ram()[0x1000..0x1800]);
                for address in [0x045c, 0x0116, 0x17, 0x0710] {
                    assert_eq!(state.ram[address], cpu.ram()[address]);
                }
                eager.dungeon_prepare_next_room_quadrant_upload();
                assert_eq!(*state.wram_image(), *eager.wram_image());
                assert_eq!(state.game_state, eager.game_state);
            }
        }
    }
}

fn clamp_c_int16_to_u16(value: u16, max: u16) -> u16 {
    if (value as i16) < 0 {
        0
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
#[path = "dungeon_tests.rs"]
mod tests;

/// `LoadTransAuxGFX` output ($7E:6000..8FFF) decoded by the translated
/// decompressors, published in store order as the source CPU advances.
#[derive(Clone, Debug)]
pub(super) struct AuxGraphicsPublication {
    bytes: Vec<u8>,
    published: usize,
}

impl AuxGraphicsPublication {
    pub(super) const RANGE: std::ops::Range<usize> = 0x6000..0x9000;

    pub(super) fn publish_through(&mut self, address: usize, ram: &mut [u8]) {
        assert!(Self::RANGE.contains(&address), "aux graphics store outside its buffer");
        let end = address - Self::RANGE.start + 1;
        assert!(end >= self.published, "source aux graphics cursor moved backward");
        ram[Self::RANGE.start + self.published..Self::RANGE.start + end]
            .copy_from_slice(&self.bytes[self.published..end]);
        self.published = end;
    }

    pub(super) fn publish_rest(mut self, ram: &mut [u8]) {
        self.publish_through(Self::RANGE.end - 1, ram);
    }
}

#[cfg(test)]
mod aux_graphics_publication_tests {
    use super::AuxGraphicsPublication;

    #[test]
    fn publishes_translated_bytes_through_the_source_store_cursor_once() {
        let bytes: Vec<u8> = (0..0x3000).map(|i| (i % 251) as u8).collect();
        let mut publication = AuxGraphicsPublication { bytes: bytes.clone(), published: 0 };
        let mut ram = vec![0xa5u8; 0x10000];
        publication.publish_through(0x6120, &mut ram);
        assert_eq!(&ram[0x6000..=0x6120], &bytes[..=0x120]);
        assert!(ram[0x6121..0x9000].iter().all(|&b| b == 0xa5));
        publication.publish_through(0x6120, &mut ram);
        publication.publish_rest(&mut ram);
        assert_eq!(&ram[0x6000..0x9000], bytes.as_slice());
    }
}

#[cfg(test)]
#[path = "dungeon/m3a_fixtures.rs"]
mod m3a_fixtures;
