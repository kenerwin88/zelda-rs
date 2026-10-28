// Methods ported from zelda3/src/load_gfx.c and included inside ZeldaState.

use super::*;
use crate::game_state::constants::MENU_PREV_JOYPAD_H;

use crate::chr_source;
use crate::cycle_models::decompress::{
    decompress_cost, decompress_store_timeline, stream_source_offset, DecompressEntry,
    DecompressStore, GET_NEXT_BYTE_MASTER,
};
use crate::game_state::{PaletteSliceSource, PaletteTransform};
use zelda3_palette::{Bank, ChannelReference};

mod load_gfx_shared;
use load_gfx_shared::*;
use load_gfx_shared::{
    aux_tileset as load_gfx_aux_tileset, main_tileset as load_gfx_main_tileset,
    sprite_tileset as load_gfx_sprite_tileset,
    MESSAGING_BUF_LOAD_GFX as LOAD_GFX_MESSAGING_BUF_LOAD_GFX,
};

const LOAD_GFX_FEATURES0_MISC_BUG_FIXES: u32 = 4096;
const LOAD_GFX_FEATURES0_DIM_FLASHES: u32 = 65536;

pub(super) fn native_tileset_table_byte(address: u32) -> Option<u8> {
    if let Some(offset)=address.checked_sub(0xdb57).filter(|&n|n<144*4) {
        Some(SPRITE_TILESETS[offset as usize/4][offset as usize%4])
    } else if let Some(offset)=address.checked_sub(0xdd97).filter(|&n|n<82*4) {
        Some(AUX_TILESETS[offset as usize/4][offset as usize%4])
    } else if let Some(offset)=address.checked_sub(0xe073).filter(|&n|n<37*8) {
        Some(MAIN_TILESETS[offset as usize/8][offset as usize%8])
    } else {None}
}

/// Ordered publication in the two-sheet animated overworld loader. Each
/// conversion cursor counts 16-bit plane stores, not complete tile rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AnimatedOverworldGraphicsProgress {
    FirstSheet(u16),
    FirstConversion(u16),
    SecondSheet(u16),
    SecondConversion(u16),
    Returned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AnimatedDungeonGraphicsProgress {
    FirstSheet(u16),
    FirstConversion(u16),
    SecondSheet(u16),
    SecondConversion(u16),
    Rotation(u16),
    DestinationPublished,
    Returned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AnimatedDungeonGraphicsContinuation {
    pub progress: AnimatedDungeonGraphicsProgress,
    rotation_saved_word: u16,
}

impl Default for AnimatedDungeonGraphicsContinuation {
    fn default() -> Self {
        Self { progress: AnimatedDungeonGraphicsProgress::FirstSheet(0), rotation_saved_word: 0 }
    }
}

/// Publication within LoadTransAuxGFX's four background and four sprite
/// sheets. This cursor owns native decoded bytes across an interrupt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TransitionAuxGraphicsProgress {
    pub sheet: u8,
    pub completed_bytes: u16,
}

/// Native WRAM publication cursor for one graphics sheet. The bytes come from
/// the translated decoder; the instruction-priced ROM model only determines
/// when each store becomes visible to an interrupting CPU read.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct GraphicsDecompressionCursor {
    destination: usize,
    bytes: Vec<u8>,
    stores: Vec<DecompressStore>,
    next_store: usize,
    total_master_cycles: u64,
}

/// The two story sheets and their caller instructions form one synchronous
/// game-thread operation. An interrupting caller retains this object and its
/// own hardware clock; decoded bytes and instruction progress stay native.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct StoryGraphicsDecompression {
    sheets: [GraphicsDecompressionCursor; 2],
    instructions: Vec<StoryGraphicsInstruction>,
    next_instruction: usize,
    elapsed_master_cycles: u64,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct StoryGraphicsInstruction {
    pub master_cycles: u32,
    pub routine: u32,
}

impl StoryGraphicsDecompression {
    pub(super) fn is_complete(&self) -> bool {
        self.next_instruction == self.instructions.len()
    }

    /// Retire a whole instruction, including its native graphics store. The
    /// final store does not finish the call: all loop/return work remains in
    /// `instructions` and must retire before the caller continues.
    pub(super) fn step(&mut self, ram: &mut [u8]) -> Option<StoryGraphicsInstruction> {
        let instruction = *self.instructions.get(self.next_instruction)?;
        self.next_instruction += 1;
        self.elapsed_master_cycles += u64::from(instruction.master_cycles);
        for sheet in &mut self.sheets {
            sheet.publish_through(self.elapsed_master_cycles, ram);
        }
        Some(instruction)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum TilesetGraphicsPublication {
    Entered,
    SpriteSelection { slot: usize, value: u8 },
    Sheet { index: usize, entry_pc: u32, sheet: u16, completed_bytes: u16 },
    Returned,
}

#[derive(Clone, Debug)]
pub(super) struct NativeTilesetGraphicsPublication {
    sprite_sheets: [usize; 4],
    operations: Vec<(u32, usize, GraphicsDecompressionCursor)>,
}

#[cfg(test)]
mod sprite_decompression_cursor_tests {
    use super::*;

    #[test]
    fn initialize_tilesets_composes_native_callers_selections_and_callee_returns() {
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let Some((template, rom)) = loaded_graphics_test_inputs() else { return };
        let mut cases = 0;
        let mut instructions = 0;
        let mut stores = 0;
        let mut pcs = std::collections::BTreeSet::new();
        for main in 0..37u8 {
            let mut native = template.clone();
            for (i, byte) in native.ram.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(37).wrapping_add((i >> 8) as u8);
            }
    { let image = native.ram.clone(); native.import_wram_image(&image); }
            native.write_wram_byte(0x10, if main & 1 == 0 {14} else {1});
            native.ram[0xaa1] = main;
            native.ram[0xaa2] = main.wrapping_mul(7) % 82;
            native.ram[0xaa3] = main.wrapping_mul(11) % 144;
            // This complete-call matrix covers both common branches. The
            // common callee separately covers all twelve raw sheets from
            // identical entry stacks, including mirrored-WRAM sheet 2.
            native.ram[0xaa4] = [0, 1, 5, 11][main as usize % 4];
            native.ram[0xc2fc..0xc300].copy_from_slice(&[0, 8, 0x5e, 0x5f]);
            native.ram[0x1fc..0x1ff].copy_from_slice(&[0x35, 0x80, 0]);
            native.sync_native_game_state_from_ram();
            native.ppu.vram.fill(0x5a5a);
            native.ppu.forced_blank = true;
            let mut atomic = native.clone();
            let mut work = native.native_initialize_tilesets_work().unwrap();
            let checkpoint = RomCpuCheckpoint {
                entry_pc:0xe19b, stop_pc:0x8036, a:0xabcd, x:0x37, y:0x25,
                sp:0x01fb, dp:0, db:0, carry:true, zero:false, overflow:true, negative:false,
                interrupt_disable:true, decimal:false, accumulator_is_8_bit:true,
                index_is_8_bit:true, emulation:false, waiting:false,
                stack_address:0x01fc, stack_bytes:&[0x35,0x80,0],
            };
            let mut run = RomCpuTimingRun::new(&rom, &native.wram_image(), &native.sram,
                &native.ppu, &snes::DmaState::default(), [0;4], checkpoint).unwrap();
            run.enable_cpu_write_trace();
            let mut expected = native.wram_image().into_owned();
            let mut total = 0;
            while !run.is_complete() {
                let pc = run.pc();
                let cycles = run.step().master_cycles;
                let instruction = work.step(|address| native.read_wram_byte(address)).expect("native initializer returned early");
                work.roundtrip_progress();
                assert_eq!(instruction.master_cycles, cycles, "main={main} pc={pc:06x} elapsed={total}");
                if let Some(native_pc) = instruction.pc { assert_eq!(native_pc, pc); }
                native.retire_tileset_instruction(instruction);
                for (address, value) in run.take_cpu_wram_writes() {
                    expected[address] = value;
                    if address >= 0x6000 {
                        assert_eq!(native.ram[address], value, "main={main} pc={pc:06x} address={address:05x}");
                        stores += 1;
                    }
                }
                assert_eq!(&native.ram[..0x10], &expected[..0x10], "caller DP main={main} pc={pc:06x}");
                assert_eq!(&native.ppu.vram[..], run.vram(), "main={main} pc={pc:06x}");
                assert_eq!((native.ppu.vram_pointer, native.ppu.vram_increment, native.ppu.vram_increment_on_high),
                    run.vram_register_state(), "VRAM registers main={main} pc={pc:06x}");
                if matches!(pc, 0xe1ae | 0xe21e | 0xe28a | 0xe2cf) {
                    work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                }
                pcs.insert(pc);
                total += u64::from(cycles);
                instructions += 1;
            }
            assert!(work.step(|address| native.read_wram_byte(address)).is_none());
            assert_eq!(&native.ram[0x6000..], &expected[0x6000..]);
            let before = crate::cycle_ledger::master();
            atomic.initialize_tilesets();
            assert_eq!(crate::cycle_ledger::master() - before, total, "atomic call cost");
            assert_eq!(&atomic.ram[0x6000..], &expected[0x6000..]);
            assert_eq!(&atomic.ppu.vram[..], run.vram());
            cases += 1;
        }
        eprintln!("native tileset initializer: cases={cases} instructions={instructions} graphics_stores={stores} source_pcs={}", pcs.len());
    }

    #[test]
    fn sheet_loader_work_matches_source_instructions_and_live_buffer_publications() {
        use crate::cycle_models::graphics_load::SheetWork;
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let Some((template, rom)) = loaded_graphics_test_inputs() else { return };
        let mut cases = 0;
        let mut instructions = 0;
        let mut stores = 0;
        // Every sprite stream, both destination banks used by real callers,
        // plus all background conversion branches and scratch destinations.
        let cases_to_check = (0..103).map(|sheet| (true, sheet, 0, 0, 0x7800))
            .chain((12..108).map(|sheet| (true, sheet, 0, 0, 0x14000)))
            .chain((0..114).map(|sheet| (false, sheet, if sheet & 1 == 0 {0x1f} else {0x20}, sheet % 8,
                if sheet % 3 == 0 {0x6000} else {0x14000})))
            .chain([0x1f, 0x20].into_iter().flat_map(|theme|
                (0..8).map(move |slot| (false, 0, theme, slot, 0x14000))));
        for (sprite, sheet, theme, slot, destination) in cases_to_check {
            let mut native = template.clone();
            for (i, byte) in native.ram.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(73).wrapping_add((i >> 8) as u8);
            }
    { let image = native.ram.clone(); native.import_wram_image(&image); }
            native.ram[2] = 0x7e + (destination >> 16) as u8;
            native.ram[0xaa1] = theme;
            native.ram[0x0f] = slot;
            native.sync_native_game_state_from_ram();
            native.ppu.vram.fill(0x5a5a);
            native.ppu.vram_pointer = 0x2000;
            native.ppu.vram_increment_on_high = true;
            native.ppu.forced_blank = true;
            let mut atomic = native.clone();
            let stream = if sprite { native.sprite_graphics_decompression_stream(sheet as usize).unwrap() }
                else { native.asset_bytes(65, sheet as usize).unwrap().to_vec() };
            let mut work = if sprite { SheetWork::sprite(sheet, destination, &stream) }
                else { SheetWork::background(sheet, destination, &stream, theme, slot) };
            let checkpoint = RomCpuCheckpoint {
                entry_pc: if sprite {0xe583} else if destination == 0x14000 {0xe609} else {0xe60d},
                stop_pc:0x8036, a:0x7e + (destination >> 16) as u16,
                x:((destination >> 8) & 0xff) as u16, y:u16::from(sheet), sp:0x01fc, dp:0, db:0,
                carry:true, zero:false, overflow:true, negative:false,
                interrupt_disable:true, decimal:false, accumulator_is_8_bit:true,
                index_is_8_bit:true, emulation:false, waiting:false,
                stack_address:0x01fd, stack_bytes:&[0x35,0x80],
            };
            let mut run = RomCpuTimingRun::new(&rom, &native.wram_image(), &native.sram,
                &native.ppu, &snes::DmaState::default(), [0;4], checkpoint).unwrap();
            run.enable_cpu_write_trace();
            let mut expected = native.wram_image().into_owned();
            let mut total = 0;
            while !run.is_complete() {
                let pc = run.pc();
                let cycles = run.step().master_cycles;
                let step = work.step(|address| native.read_wram_byte(address)).expect("native sheet returned before the source");
                work.roundtrip_progress();
                assert_eq!(step.master_cycles, cycles,
                    "sprite={sprite} sheet={sheet:02x} PC={pc:06x} elapsed={total}");
                if let Some(native_pc) = step.pc { assert_eq!(native_pc, pc); }
                native.retire_sheet_load_instruction(0x2000, step);
                for (address, value) in run.take_cpu_wram_writes() {
                    expected[address] = value;
                    if address >= 0x6000 {
                        assert_eq!(native.ram[address], value,
                            "sprite={sprite} sheet={sheet:02x} PC={pc:06x} write={address:05x}");
                        stores += 1;
                    }
                }
                // Decompressor private DP temporaries are outside this
                // work's contract; its native destination and conversion
                // source pointer are publications of the enclosing loader.
                assert_eq!(&native.ram[..3], &expected[..3], "pointer PC={pc:06x}");
                assert_eq!(&native.ppu.vram[..], run.vram(), "sprite={sprite} sheet={sheet:02x} PC={pc:06x}");
                total += u64::from(cycles);
                instructions += 1;
            }
            assert!(work.step(|address| native.read_wram_byte(address)).is_none());
            assert_eq!(&native.ram[0x6000..], &expected[0x6000..], "all graphics WRAM sheet={sheet:02x}");
            let before = crate::cycle_ledger::master();
            if sprite { atomic.load_sprite_graphics(0x2000, sheet as usize, destination); }
            else { atomic.load_background_graphics(0x2000, sheet as usize, slot as usize, destination); }
            assert_eq!(crate::cycle_ledger::master() - before, total, "atomic sheet call cost");
            assert_eq!(&atomic.ram[0x6000..], &expected[0x6000..], "atomic graphics WRAM");
            assert_eq!(&atomic.ppu.vram[..], run.vram(), "atomic graphics VRAM");
            // Checkpoint payload carries the actual instructions and native
            // loaded byte as well as the phase/index cursor.
            let mut restored: SheetWork = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
            assert!(restored.step(|address| native.read_wram_byte(address)).is_none());
            cases += 1;
        }
        eprintln!("native sheet loader: cases={cases} instructions={instructions} graphics_stores={stores}");
    }

    #[test]
    fn common_sprite_conversion_preserves_inline_and_contiguous_call_boundaries() {
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let Some((template, rom)) = loaded_graphics_test_inputs() else { return };
        for module in [0, 1, 14, 20] {
            for pack in 0..12 {
                let mut native = template.clone();
                native.write_wram_byte(0x10, module);
                native.ram[0xaa4] = pack;
                native.ram[0xbd..0xcf].fill(0xa5);
                // Both implementations receive the same caller stack at
                // routine entry. Raw sheet 2 wraps into this mirrored RAM.
                native.ram[0x1fd..0x1ff].copy_from_slice(&[0x35, 0x80]);
                native.sync_native_game_state_from_ram();
                native.ppu.vram.fill(0x5a5a);
                native.ppu.vram_pointer = 0x4400;
                native.ppu.vram_increment_on_high = true;
                native.ppu.forced_blank = true;
                let checkpoint = RomCpuCheckpoint {
                    entry_pc:0x00_e6b7, stop_pc:0x00_8036,
                    a:0xabc1, x:0x7b, y:0x23, sp:0x01fc, dp:0, db:0,
                    carry:true, zero:false, overflow:true, negative:false,
                    interrupt_disable:true, decimal:false, accumulator_is_8_bit:true,
                    index_is_8_bit:true, emulation:false, waiting:false,
                    stack_address:0x01fd, stack_bytes:&[0x35, 0x80],
                };
                let mut run = RomCpuTimingRun::new(&rom, &native.wram_image(), &native.sram,
                    &native.ppu, &snes::DmaState::default(), [0;4], checkpoint).unwrap();
                let mut expected_cycles = 0;
                while !run.is_complete() { expected_cycles += u64::from(run.step().master_cycles); }
                let before = crate::cycle_ledger::master();
                native.load_common_sprites();
                assert_eq!(crate::cycle_ledger::master() - before, expected_cycles, "module={module} pack={pack}");
                assert_eq!(&native.ppu.vram[..], run.vram(), "module={module} pack={pack}");
                assert_eq!(&native.ram[0xbd..0xcf], &run.ram()[0xbd..0xcf]);
            }
        }
    }

    #[test]
    fn tilemap_erase_native_instructions_and_dma_match_all_rom_entries() {
        use crate::cycle_models::tilemap_erase::{Pattern, Step, TilemapEraseWork};
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let Some((template, rom)) = loaded_graphics_test_inputs() else { return };
        for pattern in [Pattern::Normal, Pattern::Triforce, Pattern::DungeonMap] {
            let mut native = template.clone();
            native.set_lag_free(true);
            native.native_begin_main_cost();
            native.ppu.vram.fill(0x5aa5);
            native.ppu.vram_pointer = 0x7312;
            native.ppu.vram_increment = 32;
            native.ppu.vram_increment_on_high = false;
            native.ppu.forced_blank = true;
            let mut work = TilemapEraseWork::new(pattern);
            let checkpoint = RomCpuCheckpoint {
                entry_pc:u32::from(work.entry_pc()), stop_pc:0x8036,
                a:0xabcd, x:0x7b, y:0x23, sp:0x01fc, dp:0, db:0,
                carry:true, zero:false, overflow:true, negative:false,
                interrupt_disable:true, decimal:false, accumulator_is_8_bit:true,
                index_is_8_bit:true, emulation:false, waiting:false,
                stack_address:0x01fd, stack_bytes:&[0x35,0x80,0],
            };
            let mut run = RomCpuTimingRun::new(&rom, &native.wram_image(), &native.sram,
                &native.ppu, &native.dma, [0;4], checkpoint).unwrap();
            let mut instructions = 0;
            let mut bytes = 0;
            let mut transfers = 0;
            while let Some(step) = work.step(&native.ram, &native.dma) {
                let Step::Cpu { pc, master_cycles, .. } = step else { panic!("DMA escaped CPU start"); };
                assert_eq!(u32::from(pc), run.pc(), "{pattern:?}");
                let source = run.step();
                assert_eq!(master_cycles, source.master_cycles, "{pattern:?} PC={pc:04x}");
                native.retire_tilemap_erase_step(step);
                let dma_clocks = run.drain_started_dma_master_cycles();
                if dma_clocks != 0 {
                    transfers += 1;
                    let mut transferred = 0;
                    loop {
                        let byte = work.step(&native.ram, &native.dma).unwrap();
                        let Step::DmaByte { last, .. } = byte else { panic!("CPU advanced during DMA"); };
                        native.retire_tilemap_erase_step(byte);
                        transferred += 1;
                        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                        if last { break; }
                    }
                    assert_eq!(dma_clocks, 26 + transferred * 8);
                    bytes += transferred;
                }
                assert_eq!(&native.ram[..4], &run.ram()[..4], "WRAM PC={pc:04x}");
                assert_eq!(&native.ppu.vram[..], run.vram(), "VRAM PC={pc:04x}");
                assert_eq!((native.ppu.vram_pointer, native.ppu.vram_increment,
                    native.ppu.vram_increment_on_high), run.vram_register_state(), "PPU PC={pc:04x}");
                assert_eq!(native.dma.save_c_saveload(), run.dma_register_state(), "DMA PC={pc:04x}");
                instructions += 1;
                work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
            }
            assert!(run.is_complete());
            assert_eq!(transfers, 4);
            assert_eq!(bytes, 20480);
            assert_eq!(native.native_cost_main_dma(), u64::from(4 * 26 + bytes * 8));
            assert_eq!(instructions, if matches!(pattern, Pattern::Normal) {48} else {49});
        }
    }

    #[test]
    fn tilemap_erase_retains_loaded_word_and_publishes_low_then_high_dma_bytes() {
        use crate::cycle_models::tilemap_erase::{Pattern, Step, TilemapEraseWork};
        let mut state = ZeldaState::new();
        state.ppu.vram.fill(0x5aa5);
        let mut work = TilemapEraseWork::new(Pattern::Normal);
        let mut byte_count = 0;
        while let Some(step) = work.step(&state.ram, &state.dma) {
            state.retire_tilemap_erase_step(step);
            if let Step::DmaByte { .. } = step {
                byte_count += 1;
                if byte_count == 1 {
                    assert_eq!(state.ppu.vram[0], 0x5aec);
                    assert_eq!(state.ppu.vram[1], 0x5aa5);
                    state.ram[0] = 0x34;
                } else if byte_count == 2 {
                    assert_eq!(state.ppu.vram[1], 0x5a34);
                } else if byte_count == 0x4000 {
                    state.ram[2..4].copy_from_slice(&0x5678u16.to_le_bytes());
                }
            }
            if matches!(step, Step::Cpu { pc: 0x8391, .. }) {
                state.ram[2..4].copy_from_slice(&0x9999u16.to_le_bytes());
            }
            work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        }
        assert_eq!(state.ppu.vram[0], 0x01ec);
        assert_eq!(state.ppu.vram[1], 0x0134);
        assert!(state.ppu.vram[0x6000..0x6800].iter().all(|&word| word == 0x5678));
        assert_eq!(&state.ram[..2], &0x5678u16.to_le_bytes());
    }

    #[test]
    fn graphics_conversion_native_work_matches_each_rom_instruction_and_publication() {
        use crate::cycle_models::graphics_convert::{ConversionWork, Publication};
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let Some((template, rom)) = loaded_graphics_test_inputs() else { return };
        let mut cases = 0;
        let mut instructions = 0;
        let mut publications = 0;
        let mut visited = std::collections::BTreeSet::new();
        for high in [false, true] {
            for tiles in [1u8, 2, 64, 128] {
                for pattern in [0u8, 0x55, 0xff, 0xa7] {
                    let source: Vec<u8> = (0..usize::from(tiles) * 24 + 1)
                        .map(|i| if pattern == 0xa7 { (i as u8).wrapping_mul(73).wrapping_add((i / 24) as u8) } else { pattern })
                        .collect();
                    let mut native = template.clone();
                    native.ram[0x14000..0x14000 + source.len()].copy_from_slice(&source);
                    native.ram[0..3].copy_from_slice(&[0, 0x40, 0x7f]);
                    native.ram[0xbd..0xcf].fill(0xa5);
                    native.sync_native_game_state_from_ram();
                    native.ppu.vram.fill(0x5a5a);
                    native.ppu.vram_pointer = 0x2000;
                    native.ppu.vram_increment_on_high = true;
                    native.ppu.forced_blank = true;
                    let mut atomic = native.clone();
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: if high {0x00_e5af} else {0x00_e63c}, stop_pc: 0x00_8036,
                        a:0xabc1, x:0x7b, y:u16::from(tiles - 1), sp:0x01fc, dp:0, db:0,
                        carry:true, zero:false, overflow:true, negative:false,
                        interrupt_disable:true, decimal:false, accumulator_is_8_bit:false,
                        index_is_8_bit:true, emulation:false, waiting:false,
                        stack_address:0x01fd, stack_bytes:&[0x35, 0x80],
                    };
                    let mut run = RomCpuTimingRun::new(&rom, &native.wram_image(), &native.sram,
                        &native.ppu, &snes::DmaState::default(), [0;4], checkpoint).unwrap();
                    run.enable_cpu_write_trace();
                    let mut expected = native.wram_image().into_owned();
                    let mut work = ConversionWork::new(high, &source, tiles);
                    let mut total = 0;
                    let mut final_store_at = 0;
                    while let Some(instruction) = work.step() {
                        work.roundtrip_progress();
                        let pc = run.pc();
                        assert_eq!(instruction.pc, pc, "high={high} tiles={tiles} pattern={pattern}");
                        let cycles = run.step().master_cycles;
                        assert_eq!(instruction.master_cycles, cycles, "PC {pc:06x}");
                        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                        native.retire_graphics_conversion_instruction(0x2000, instruction);
                        if let Some(Publication::SourceOffset(offset)) = instruction.publication {
                            native.ram[..2].copy_from_slice(&(0x4000 + offset).to_le_bytes());
                        }
                        assert_eq!(&native.ram[..3], &expected[..3], "pointer PC {pc:06x}");
                        assert_eq!(&native.ram[0xbd..0xcf], &expected[0xbd..0xcf], "scratch PC {pc:06x}");
                        assert_eq!(&native.ppu.vram[..], run.vram(), "VRAM PC {pc:06x}");
                        instructions += 1;
                        publications += usize::from(instruction.publication.is_some());
                        visited.insert(pc);
                        total += u64::from(cycles);
                        if matches!(instruction.publication, Some(Publication::VramWord { .. })) {
                            final_store_at = total;
                        }
                    }
                    assert!(run.is_complete());
                    assert!(work.step().is_none());
                    assert!(total > final_store_at, "last VRAM store is not caller completion");
                    if tiles == 64 {
                        let before = crate::cycle_ledger::master();
                        atomic.convert_3_to_4_to_vram(0x2000, &source, chr_source::CHR_KIND_BG, 0, high);
                        assert_eq!(crate::cycle_ledger::master() - before, total);
                        assert_eq!(&atomic.ppu.vram[..], run.vram());
                        assert_eq!(&atomic.ram[0xbd..0xcf], &expected[0xbd..0xcf]);
                    }
                    cases += 1;
                }
            }
        }
        eprintln!("native graphics conversion: cases={cases} instructions={instructions} publications={publications} source_pcs={}", visited.len());
        assert_eq!(cases, 32);
    }

    #[test]
    fn half_slot_work_publications_and_bank_crossing_match_rom() {
        use crate::rom_cpu_timing::{lorom_offset, RomCpuCheckpoint, RomCpuTimingRun};
        let Some((template, rom)) = loaded_graphics_test_inputs() else { return };
        for (sheet, &low) in SPRITE_SOURCE_LOW_WORDS.iter().enumerate() {
            let hi = rom[lorom_offset(0x00_d0d2 + sheet as u32).unwrap()];
            let lo = rom[lorom_offset(0x00_d1b1 + sheet as u32).unwrap()];
            assert_eq!(low, u16::from_le_bytes([lo, hi]), "sheet={sheet}");
        }
        for request in 0..=20 {
            for pack in 0..12 {
                for indoors in [0, 1] {
                    for cgram_flag in [0, 255] {
                        let mut state = template.clone();
                        state.ram[0xaaa] = request;
                        state.ram[0xaa4] = pack;
                        state.ram[0x1b] = indoors;
                        state.ram[0x8a] = if pack & 1 == 0 { 0 } else { 0x40 };
                        state.write_wram_byte(0x15, cgram_flag);
                        state.write_wram_byte(0x16, 0xa5);
                        state.ram[0x112] = 0xa5;
                        state.ram[0x116] = 0xa5;
                        state.ram[0x11000..0x11400].fill(0xa5);
                        state.sync_native_game_state_from_ram();
                        let checkpoint = RomCpuCheckpoint {
                            entry_pc: 0x00_e3fa, stop_pc: 0x00_8036,
                            a: 0, x: 0, y: 0, sp: 0x01fb, dp: 0, db: 0,
                            carry: true, zero: false, overflow: true, negative: false,
                            interrupt_disable: false, decimal: false,
                            accumulator_is_8_bit: true, index_is_8_bit: true,
                            emulation: false, waiting: false,
                            stack_address: 0x01fc, stack_bytes: &[0x35, 0x80, 0],
                        };
                        let mut run = RomCpuTimingRun::new(&rom, &state.wram_image(), &[0; 0x2000],
                            &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
                        run.enable_cpu_write_trace();
                        let mut expected = state.wram_image().into_owned();
                        let mut cycles = 0;
                        let mut instructions = 0;
                        while !run.is_complete() {
                            let pc = run.pc();
                            let step = run.step();
                            if pc == 0x00_e3d1 || (0x00_e3fa..=0x00_e4e8).contains(&pc) {
                                cycles += u64::from(step.master_cycles);
                            }
                            for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                            instructions += 1;
                            assert!(instructions < 100_000, "half-slot call did not return");
                        }
                        let actual = crate::cycle_ledger::measure_routine_self_work(0x00_e3fa,
                            || state.graphics_load_chr_half_slot());
                        let label = format!("request={request} pack={pack} indoors={indoors} cgram={cgram_flag}");
                        assert_eq!(actual, cycles, "{label}");
                        for range in [0x15..0x18, 0x112..0x113, 0x116..0x117, 0xaa4..0xaa5,
                            0xaa9..0xaab, 0xab1..0xab2, 0xc300..0xc700, 0x11000..0x11400] {
                            assert_eq!(&state.wram_image()[range.clone()], &expected[range.clone()], "{label} range={range:x?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn story_graphics_native_continuation_matches_every_rom_instruction_and_store() {
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let Some((state, rom)) = loaded_graphics_test_inputs() else { return };
        let mut work = state.story_graphics_decompression().unwrap();
        let mut native_ram = vec![0; 0x20000];
        native_ram[0x14000..0x15000].fill(0xa5);
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x00_d80e, stop_pc: 0x00_8036,
            a: 0, x: 0, y: 0, sp: 0x01fb, dp: 0, db: 0,
            carry: false, zero: false, overflow: false, negative: false,
            interrupt_disable: false, decimal: false,
            accumulator_is_8_bit: true, index_is_8_bit: true,
            emulation: false, waiting: false,
            stack_address: 0x01fc, stack_bytes: &[0x35, 0x80, 0],
        };
        let mut run = RomCpuTimingRun::new(&rom, &native_ram, &[0; 0x2000],
            &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
        run.enable_cpu_write_trace();
        let mut expected_ram = native_ram.clone();
        let mut elapsed = 0;
        while !run.is_complete() {
            let pc = run.pc();
            let expected = run.step().master_cycles;
            let instruction = work.step(&mut native_ram).expect("native call returned early");
            assert_eq!(instruction.master_cycles, expected, "pc={pc:06x} elapsed={elapsed}");
            assert_eq!(instruction.routine, if (0x00_d80e..=0x00_d836).contains(&pc) {
                0x00_d80e
            } else if (0x00_e843..=0x00_e852).contains(&pc) {
                0x00_e843
            } else { 0x00_e772 }, "pc={pc:06x}");
            elapsed += u64::from(expected);
            assert_eq!(work.elapsed_master_cycles, elapsed);
            for (address, value) in run.take_cpu_wram_writes() {
                if (0x14000..0x15000).contains(&address) {
                    expected_ram[address] = value;
                }
            }
            assert_eq!(&native_ram[0x14000..0x15000], &expected_ram[0x14000..0x15000],
                "graphics publication at pc={pc:06x}");
        }
        assert!(work.step(&mut native_ram).is_none());
        assert!(work.sheets.iter().all(GraphicsDecompressionCursor::is_complete));
        let mut atomic = state.clone();
        let before = crate::cycle_ledger::master();
        atomic.attract_decompress_story_gfx();
        assert_eq!(crate::cycle_ledger::master() - before, elapsed);
        assert_eq!(&atomic.ram[0x14000..0x15000], &native_ram[0x14000..0x15000]);
        eprintln!("story graphics: {} instructions, {} stores, {elapsed} CPU master cycles",
            work.next_instruction, work.sheets.iter().map(|sheet| sheet.next_store).sum::<usize>());
    }

    #[test]
    fn story_graphics_resume_preserves_stores_and_return_instructions() {
        let Some((state, _)) = loaded_graphics_test_inputs() else { return };
        let mut work = state.story_graphics_decompression().unwrap();
        let mut expected = state.clone();
        expected.decompress_sprite_graphics_to_buffer(0x14000, 0x67);
        expected.decompress_sprite_graphics_to_buffer(0x14800, 0x68);
        let mut ram = state.ram.clone();
        let mut resumes = 0;
        while work.step(&mut ram).is_some() {
            // Suspend on either side of output and caller boundaries. The
            // retained object must include work with no remaining stores.
            let at_boundary = work.sheets.iter().any(|sheet| {
                sheet.stores.iter().any(|store|
                    store.after_master_cycles == work.elapsed_master_cycles)
                    && (sheet.next_store == 1 || sheet.is_complete())
            }) || work.next_instruction + 1 >= work.instructions.len();
            if at_boundary || work.next_instruction % 4096 == 0 {
                let mut resumed = work.clone();
                let mut resumed_ram = ram.clone();
                while resumed.step(&mut resumed_ram).is_some() {}
                assert_eq!(resumed_ram, expected.ram);
                assert_eq!(resumed.elapsed_master_cycles,
                    work.instructions.iter().map(|step| u64::from(step.master_cycles)).sum::<u64>());
                resumes += 1;
            }
        }
        assert!(resumes > 10);
        eprintln!("story graphics: {resumes} retained progress checkpoints matched completion");
        let mut retained = ram.clone();
        retained[0x14000] ^= 0xff;
        assert!(work.step(&mut retained).is_none());
        assert_ne!(retained[0x14000], ram[0x14000], "a completed call must not publish again");
    }

    #[test]
    fn spotlight_lower_store_resume_keeps_native_value_and_prices_only_remaining_instructions() {
        use crate::SpotlightTableBuildCheckpoint::{AfterUpperTableWrite, BeforeLowerTableStore};
        let mut base = ZeldaState::new();
        base.follower_link_state_mut().set_y(20);
        base.set_spotlight_window_radius(64);
        for completed_iterations in [1, 185, 191] {
            let lower_cursor = 224 - completed_iterations;
            let mut guard = base.clone();
            let start = crate::cycle_ledger::master();
            let guard_build = guard.begin_iris_spotlight_configure_table_at_progress(
                crate::SpotlightTableBuildProgress {
                    completed_iterations, checkpoint: AfterUpperTableWrite { lower_cursor },
                });
            let guard_prefix = guard.ram.clone();
            guard.complete_iris_spotlight_table_projection(guard_build);
            let guard_cycles = crate::cycle_ledger::master() - start;
            let mut store = base.clone();
            let start = crate::cycle_ledger::master();
            let store_build = store.begin_iris_spotlight_configure_table_at_progress(
                crate::SpotlightTableBuildProgress {
                    completed_iterations, checkpoint: BeforeLowerTableStore { lower_cursor },
                });
            assert_eq!(store.ram, guard_prefix);
            store.complete_iris_spotlight_table_projection(store_build);
            // F387..F38D costs86; F38F TAX + F390 LDA costs46. All
            // of those instructions retired before the F392 store boundary.
            assert_eq!(crate::cycle_ledger::master() - start + 132, guard_cycles);
            assert_eq!(store.ram, guard.ram);
            assert_eq!(store.game_state, guard.game_state);
        }
    }

    #[test]
    fn spotlight_projection_checkpoint_keeps_the_completed_dynamic_table() {
        let mut base = ZeldaState::new();
        base.follower_link_state_mut().set_position(120, 200);
        base.set_spotlight_window_radius(77);
        base.set_spotlight_window_state(2);
        let mut atomic = base.clone();
        atomic.iris_spotlight_configure_table();
        assert_eq!(atomic.spotlight_hdma_table_dynamic_entry(168), 0xfc04);
        let mut resumed = base;
        let build = resumed.begin_iris_spotlight_configure_table_at_progress(
            crate::SpotlightTableBuildProgress {
                completed_iterations: 213,
                checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 168 },
            });
        assert_eq!(resumed.game_state.display.spotlight_hdma.window_radius(), 77);
        assert_eq!(&resumed.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 480],
            &atomic.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 480]);
        assert!(resumed.ram[RESERVED_HDMA_TABLE + 336..RESERVED_HDMA_TABLE + 448]
            .iter().all(|&byte| byte == 0));
        resumed.complete_iris_spotlight_configure_table(build);
        assert_eq!(resumed.ram, atomic.ram);
        assert_eq!(resumed.game_state, atomic.game_state);
    }

    #[test]
    fn animated_dungeon_graphics_resume_without_replaying_planes_or_rotation() {
        use AnimatedDungeonGraphicsProgress::*;
        let Some((mut base, _)) = loaded_graphics_test_inputs() else { return };
        base.ram[0x14000..0x14600].fill(0xa5);
        base.ram[0xa680..0xb280].fill(0x5a);
        for pack in [0x5b, 0x5d] {
            let mut atomic = base.clone();
            let start = crate::cycle_ledger::master();
            atomic.decompress_animated_dungeon_tiles(pack);
            let cost = crate::cycle_ledger::master() - start;
            for target in [FirstSheet(0), FirstSheet(1), FirstSheet(859), FirstSheet(1536),
                FirstConversion(0), FirstConversion(1), FirstConversion(767), FirstConversion(768),
                SecondSheet(792), SecondSheet(1536), SecondConversion(1), SecondConversion(768),
                Rotation(0), Rotation(1), Rotation(2), Rotation(3), Rotation(4), Rotation(511),
                Rotation(1023), Rotation(1024), DestinationPublished, Returned] {
                let mut state = base.clone();
                let mut work = AnimatedDungeonGraphicsContinuation::default();
                let start = crate::cycle_ledger::master();
                state.advance_animated_dungeon_graphics(pack, &mut work, target);
                if target == FirstSheet(859) {
                    assert!(state.ram[0x1435b..0x14600].iter().all(|&b| b == 0xa5));
                    assert!(state.ram[0xa680..0xb280].iter().all(|&b| b == 0x5a));
                }
                let ram = state.ram.clone();
                let once = work;
                state.advance_animated_dungeon_graphics(pack, &mut work, target);
                assert_eq!(state.ram, ram);
                assert_eq!(work, once);
                state.native_pre_dungeon_animated_graphics = Some(work);
                state.decompress_animated_dungeon_tiles(pack);
                assert!(state.native_pre_dungeon_animated_graphics.is_none());
                assert_eq!(crate::cycle_ledger::master() - start, cost);
                assert_eq!(*state.wram_image(), *atomic.wram_image(), "pack={pack} target={target:?}");
                assert_eq!(state.game_state, atomic.game_state);
            }
            let mut state = base.clone();
            let mut work = AnimatedDungeonGraphicsContinuation::default();
            state.advance_animated_dungeon_graphics(pack, &mut work, Rotation(0));
            let saved = read_le_u16(&state.ram, 0xa880);
            state.advance_animated_dungeon_graphics(pack, &mut work, Rotation(1));
            write_le_u16(&mut state.ram, 0xa880, 0x1234);
            state.native_pre_dungeon_animated_graphics = Some(work);
            state.decompress_animated_dungeon_tiles(pack);
            assert_eq!(read_le_u16(&state.ram, 0xaa80), saved, "rotation must retain its native stack word");
            assert_eq!(read_le_u16(&state.ram, 0xa880), 0x1234, "completed stores must not replay");
        }
    }

    #[test]
    fn animated_overworld_graphics_resume_at_sheet_and_plane_boundaries() {
        use AnimatedOverworldGraphicsProgress::*;
        let Some((mut base, _)) = loaded_graphics_test_inputs() else { return };
        base.ram[0x14000..0x14600].fill(0xa5);
        base.ram[0xa680..0xb280].fill(0x5a);
        for sheet in [0x5a, 0x5c, 0x5e] {
            let mut atomic = base.clone();
            let atomic_started = crate::cycle_ledger::master();
            atomic.DecompressAnimatedOverworldTiles(sheet);
            let atomic_cost = crate::cycle_ledger::master() - atomic_started;
            for target in [FirstSheet(645), FirstConversion(1), FirstConversion(1023),
                SecondSheet(254), SecondConversion(257), Returned] {
                let mut resumed = base.clone();
                let mut progress = FirstSheet(0);
                let resumed_started = crate::cycle_ledger::master();
                resumed.advance_animated_overworld_graphics(sheet, &mut progress, target);
                if target == FirstSheet(645) {
                    let decoded = resumed.decompressed_background_graphics_data(sheet as usize).unwrap();
                    assert_eq!(&resumed.ram[0x14000..0x14285], &decoded[..645]);
                    assert!(resumed.ram[0x14285..0x14600].iter().all(|&b| b == 0xa5));
                }
                if target == FirstConversion(1) {
                    assert_eq!(&resumed.ram[0xa680..0xa682], &atomic.ram[0xa680..0xa682]);
                    assert_eq!(&resumed.ram[0xa690..0xa692], &[0x5a; 2]);
                }
                let before_repeat = resumed.ram.clone();
                resumed.advance_animated_overworld_graphics(sheet, &mut progress, target);
                assert_eq!(resumed.ram, before_repeat);
                resumed.native_pre_overworld_animated_graphics = Some(progress);
                resumed.complete_pre_overworld_animated_graphics(sheet);
                assert_eq!(crate::cycle_ledger::master() - resumed_started, atomic_cost,
                    "resuming graphics must charge each native operation exactly once");
                assert_eq!(resumed.ram, atomic.ram, "sheet={sheet} target={target:?}");
                assert_eq!(resumed.game_state, atomic.game_state);
                assert_eq!(resumed.animated_tile_pack, atomic.animated_tile_pack);
            }
        }
    }

    #[test]
    fn completed_animated_graphics_do_not_replay_scratch_or_plane_stores() {
        let Some((mut state, _)) = loaded_graphics_test_inputs() else { return };
        let mut progress = AnimatedOverworldGraphicsProgress::FirstSheet(0);
        state.advance_animated_overworld_graphics(0x5a, &mut progress, AnimatedOverworldGraphicsProgress::Returned);
        state.native_pre_overworld_animated_graphics = Some(progress);
        // Later native work owns this scratch and an already-published plane.
        state.ram[0x14000] = 0xf3;
        state.ram[0xa680] = 0x9d;
        state.complete_pre_overworld_animated_graphics(0x5a);
        assert_eq!(state.ram[0x14000], 0xf3);
        assert_eq!(state.ram[0xa680], 0x9d);
        assert!(state.native_pre_overworld_animated_graphics.is_none());
    }

    #[test]
    fn palette_cache_copy_resumes_in_rom_order_and_preserves_provenance() {
        let mut base = ZeldaState::new();
        for index in 0..256 {
            base.set_aux_color_asset(index, (index as u16 * 31) & 0x7fff);
            base.set_main_color_constant(index, 0x1234);
        }
        let mut atomic = base.clone();
        let aux = atomic.game_state.display.palette_buffer.aux_full_slice().to_vec();
        atomic.copy_main_full_from_tagged(&aux, PaletteSliceSource::MirrorBank(Bank::Aux));
        atomic.increment_cgram_update_flag();
        for cut in [0, 1, 7, 8, 61, 255, 256, 257] {
            let mut resumed = base.clone();
            resumed.advance_palette_cache_copy(0, cut);
            for ordinal in 0..256u16 {
                let index = usize::from(ordinal % 8) * 32 + usize::from(ordinal / 8);
                let expected = if ordinal < cut { &atomic.ram } else { &base.ram };
                assert_eq!(&resumed.ram[0xc500 + index * 2..0xc502 + index * 2],
                    &expected[0xc500 + index * 2..0xc502 + index * 2]);
            }
            if cut <= 256 { assert_eq!(resumed.ram[0x15], base.read_wram_byte(0x15)); }
            let before_repeat = resumed.ram.clone();
            resumed.advance_palette_cache_copy(cut, cut);
            assert_eq!(resumed.ram, before_repeat);
            resumed.advance_palette_cache_copy(cut, 257);
            assert_eq!(resumed.ram, atomic.ram);
            assert_eq!(resumed.game_state, atomic.game_state);
            assert_eq!(resumed.palette_mirror_snapshot_bytes(), atomic.palette_mirror_snapshot_bytes());
        }
        let mut resumed = base;
        resumed.advance_palette_cache_copy(0, 1);
        resumed.set_main_color_constant(0, 0x3210);
        resumed.advance_palette_cache_copy(1, 257);
        assert_eq!(&resumed.ram[0xc500..0xc502], &[0x10, 0x32]);
    }

    #[test]
    fn pre_dungeon_palette_cache_does_not_replay_completed_loader_prefix() {
        let Some((mut state, _)) = loaded_graphics_test_inputs() else { return };
        for (slot, sheet) in [97, 98, 99, 80].into_iter().enumerate() {
            state.sprite_workspace_mut().set_graphics_subset(slot, sheet);
        }
        state.native_pre_dungeon_animated_graphics = Some(AnimatedDungeonGraphicsContinuation {
            progress: AnimatedDungeonGraphicsProgress::Returned, ..Default::default()
        });
        let mut atomic = state.clone();
        atomic.complete_module_pre_dungeon_before_palette_cache();
        atomic.overworld_copy_palettes_to_cache();
        state.advance_pre_dungeon_palette_cache(61);
        assert!(state.native_pre_dungeon_animated_graphics.is_none());
        assert!(state.native_pre_dungeon_tileset_publication.is_none());
        assert!(!state.paired_resume_cpu_boundary_is_quiescent());
        state.advance_pre_dungeon_palette_cache(257);
        assert_eq!(*state.wram_image(), *atomic.wram_image());
        assert_eq!(state.game_state, atomic.game_state);
        assert_eq!(state.palette_mirror_snapshot_bytes(), atomic.palette_mirror_snapshot_bytes());
        state.set_aux_color_asset(0, 0x3210);
        state.set_main_color_constant(0, 0x2345);
        let before = state.ram.clone();
        state.advance_pre_dungeon_palette_cache(257);
        assert_eq!(state.ram, before);
    }

    #[test]
    fn pre_dungeon_tileset_prefix_and_partial_sheet_have_separate_owners() {
        let Some((mut state, _)) = loaded_graphics_test_inputs() else { return };
        for (slot, sheet) in [97, 98, 99, 80].into_iter().enumerate() {
            state.sprite_workspace_mut().set_graphics_subset(slot, sheet);
        }
        // The animated loader has returned; this caller now owns attributes
        // and the common-sprite selector, but no tileset sheet bytes yet.
        state.native_pre_dungeon_animated_graphics = Some(AnimatedDungeonGraphicsContinuation {
            progress: AnimatedDungeonGraphicsProgress::Returned,
            ..Default::default()
        });
        state.world_palette_theme_mut().set_misc_sprites_graphics_index(1);
        state.ram[0x7800..0x7e00].fill(0xa5);
        state.advance_pre_dungeon_tileset_publication(&[TilesetGraphicsPublication::Entered]);
        assert_eq!(state.game_state.world.palette_theme.misc_sprites_graphics_index(), 10);
        assert!(state.native_pre_dungeon_animated_graphics.is_none());
        assert!(state.ram[0x7800..0x7e00].iter().all(|&b| b == 0xa5));
        let plan = state.native_tileset_decompression_operations();
        let (entry_pc, sheet, cursor) = &plan.operations[0];
        let events = [
            TilesetGraphicsPublication::SpriteSelection { slot: 0, value: *sheet as u8 },
            TilesetGraphicsPublication::Sheet { index: 0, entry_pc: *entry_pc,
                sheet: *sheet as u16, completed_bytes: 645 },
        ];
        state.advance_pre_dungeon_tileset_publication(&events);
        assert_eq!(&state.ram[0x7800..0x7a85], &cursor.bytes[..645]);
        assert!(state.ram[0x7a85..0x7e00].iter().all(|&b| b == 0xa5));
        // A repeated cursor must not replay the prefix or an earlier store.
        state.ram[0x7800] = 0x32;
        state.world_palette_theme_mut().set_misc_sprites_graphics_index(3);
        state.advance_pre_dungeon_tileset_publication(&events);
        assert_eq!(state.ram[0x7800], 0x32);
        assert_eq!(state.game_state.world.palette_theme.misc_sprites_graphics_index(), 3);
        assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    }

    #[test]
    fn pre_overworld_tileset_publication_keeps_unwritten_sheet_tails() {
        let Some((mut state, _)) = loaded_graphics_test_inputs() else { return };
        for (slot, sheet) in [97, 98, 99, 80].into_iter().enumerate() {
            state.sprite_workspace_mut().set_graphics_subset(slot, sheet);
        }
        let plan = state.native_tileset_decompression_operations();
        for (slot, &sheet) in plan.sprite_sheets.iter().enumerate() {
            state.advance_pre_overworld_tileset_publication(&[
                TilesetGraphicsPublication::SpriteSelection { slot, value: sheet as u8 },
            ]);
            assert_eq!(state.game_state.sprites.workspace.graphics_subset(slot), sheet as u8);
        }
        for (index, (entry_pc, sheet, cursor)) in plan.operations.iter().enumerate() {
            for end in [1, 645, 1536] {
                let original = state.ram.clone();
                let event = TilesetGraphicsPublication::Sheet {
                    index, entry_pc: *entry_pc, sheet: *sheet as u16, completed_bytes: end,
                };
                state.advance_pre_overworld_tileset_publication(&[event]);
                let start = cursor.destination;
                let end = usize::from(end);
                assert_eq!(&state.ram[start..start + end], &cursor.bytes[..end]);
                assert_eq!(&state.ram[start + end..start + 1536], &original[start + end..start + 1536]);
                let once = state.ram.clone();
                state.advance_pre_overworld_tileset_publication(&[event]);
                assert_eq!(state.ram, once);
            }
        }
        state.advance_pre_overworld_tileset_publication(&[TilesetGraphicsPublication::Returned]);
    }

    #[test]
    fn transition_aux_sheets_resume_without_publishing_the_unwritten_tail() {
        let Some((base, _)) = loaded_graphics_test_inputs() else { return };
        for (background_theme, sprite_theme) in [(0, 0), (3, 8), (23, 15), (81, 143)] {
            let mut state = base.clone();
            state.ram[crate::game_state::constants::AUX_TILE_THEME_INDEX] = background_theme;
            state.ram[crate::game_state::constants::SPRITE_GRAPHICS_INDEX] = sprite_theme;
            state.ram[0x6000..0x9000].fill(0xa5);
            state.ram[0x412] = 0x55;
            state.sync_native_game_state_from_ram();
            let mut atomic = state.clone();
            atomic.LoadTransAuxGFX();
            let mut completed = None;
            for sheet in 0..8 {
                for completed_bytes in [1, 0x321, 0x600] {
                    let target = TransitionAuxGraphicsProgress { sheet, completed_bytes };
                    state.advance_transition_aux_graphics(completed, target);
                    let end = 0x6000 + usize::from(sheet) * 0x600 + usize::from(completed_bytes);
                    assert_eq!(&state.ram[0x6000..end], &atomic.ram[0x6000..end]);
                    assert!(state.ram[end..0x9000].iter().all(|&byte| byte == 0xa5));
                    assert_eq!(state.ram[0x412], 0x55, "caller suffix must remain pending");
                    let before_repeat = state.ram.clone();
                    state.advance_transition_aux_graphics(Some(target), target);
                    assert_eq!(state.ram, before_repeat);
                    completed = Some(target);
                }
            }
            state.reset_incremental_vram_upload_counter();
            assert_eq!(*state.wram_image(), *atomic.wram_image());
            assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
            assert_eq!(state.game_state.world, atomic.game_state.world);
        }
    }

    #[test]
    fn overworld_aux_caller_consumes_shared_graphics_progress_once() {
        let Some((base, _)) = loaded_graphics_test_inputs() else { return };
        for theme in [0, 32] {
            let mut initial = base.clone();
            initial.ram[crate::game_state::constants::AUX_TILE_THEME_INDEX] = theme;
            initial.set_main_module(9);
            initial.set_submodule(8);
            initial.ram[0x6000..0x9000].fill(0xa5);
            initial.ram[0x412] = 0x55;
            initial.sync_native_game_state_from_ram();
            let mut atomic = initial.clone();
            atomic.complete_module09_load_aux_gfx();
            for sheet in 0..8 {
                for completed_bytes in [0, 1, 0x321, 0x600] {
                    let mut state = initial.clone();
                    let progress = TransitionAuxGraphicsProgress { sheet, completed_bytes };
                    state.advance_transition_aux_graphics(None, progress);
                    state.native_transition_aux_graphics_progress = Some(progress);
                    assert_eq!(state.game_state.frame.submodule, 8);
                    assert_eq!(state.ram[0x412], 0x55);
                    state.complete_module09_load_aux_gfx();
                    assert!(state.native_transition_aux_graphics_progress.is_none());
                    assert_eq!(*state.wram_image(), *atomic.wram_image(), "theme={theme} progress={progress:?}");
                    assert_eq!(state.game_state, atomic.game_state);
                }
            }
            let mut state = initial;
            let progress = TransitionAuxGraphicsProgress { sheet: 7, completed_bytes: 0x600 };
            state.advance_transition_aux_graphics(None, progress);
            state.native_transition_aux_graphics_progress = Some(progress);
            state.ram[0x6000] ^= 0xff;
            let retained = state.ram[0x6000];
            state.LoadTransAuxGFX();
            assert_eq!(state.ram[0x6000], retained, "returned sheets must not be decoded again");
        }
    }

    #[test]
    fn stores_become_visible_only_after_their_cpu_instruction() {
        let stream = [0x02, 1, 2, 3, 0x21, 0xaa, 0x41, 0x55, 0x66, 0x61, 0x77, 0x80, 0, 0, 0xff];
        let expected = decompress_asset(&stream);
        let (cost, stores) = decompress_store_timeline(
            DecompressEntry::Sprite, 97, &stream, 0x8000, 0x7e_7800,
        );
        let mut cursor = GraphicsDecompressionCursor::new(0x7800, expected.clone(), stores, cost.master).unwrap();
        let mut ram = vec![0xa5; 0x10000];
        let first = cursor.stores[0].after_master_cycles;
        cursor.publish_through(first - 1, &mut ram);
        assert!(ram[0x7800..0x7800 + expected.len()].iter().all(|&byte| byte == 0xa5));
        cursor.publish_through(first, &mut ram);
        assert_eq!(ram[0x7800], expected[0]);
        assert_eq!(cursor.next_store, 1);
        cursor.publish_through(cursor.total_master_cycles, &mut ram);
        assert!(cursor.is_complete());
        assert_eq!(&ram[0x7800..0x7800 + expected.len()], expected.as_slice());
    }

    #[test]
    fn dungeon_map_sprite_sheets_match_the_existing_decoder() {
        let path = std::env::var_os("ZELDA3_ASSETS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat"));
        let Ok(assets) = std::fs::read(path) else { return };
        let mut state = ZeldaState::new();
        state.set_assets(&assets).unwrap();
        for (slot, sheet) in [97, 98, 99, 80].into_iter().enumerate() {
            let destination = 0x7800 + slot * 0x600;
            let expected = state.decompressed_sprite_graphics_data(sheet).unwrap();
            let mut cursor = state.sprite_decompression_cursor(sheet, destination).unwrap();
            let mut ram = vec![0xa5; 0x10000];
            cursor.publish_through(cursor.total_master_cycles, &mut ram);
            assert!(cursor.is_complete());
            assert_eq!(&ram[destination..destination + 0x600], expected.as_slice());
        }
    }

    #[test]
    fn dungeon_map_sprite_store_events_match_independent_cpu_execution() {
        let Some((state, rom)) = loaded_graphics_test_inputs() else { return };
        for (slot, sheet) in [97usize, 98, 99, 80].into_iter().enumerate() {
            let destination = 0x7800 + slot * 0x600;
            let cursor = state.sprite_decompression_cursor(sheet, destination).unwrap();
            assert_cursor_matches_shadow(&rom, 0x00_e772, sheet, destination, cursor);
        }
    }

    #[test]
    fn dungeon_map_background_store_events_match_independent_cpu_execution() {
        let Some((state, rom)) = loaded_graphics_test_inputs() else { return };
        let sheets = [58, 59, 60, 61, 55, 56, 62, 91];
        let destinations = [0x14000, 0x14000, 0x14000, 0x6000, 0x6600, 0x6c00, 0x7200, 0x14000];
        for (sheet, destination) in sheets.into_iter().zip(destinations) {
            let cursor = state.background_decompression_cursor(sheet, destination).unwrap();
            assert_cursor_matches_shadow(&rom, 0x00_e78f, sheet, destination, cursor);
        }
    }

    #[test]
    fn background_cursor_publishes_into_bank_7f() {
        let Some((state, _)) = loaded_graphics_test_inputs() else { return };
        let expected = state.decompressed_background_graphics_data(58).unwrap();
        let mut cursor = state.background_decompression_cursor(58, 0x14000).unwrap();
        let mut ram = vec![0xa5; 0x2_0000];
        cursor.publish_through(cursor.total_master_cycles, &mut ram);
        assert!(cursor.is_complete());
        assert_eq!(&ram[0x14000..0x14000 + expected.len()], expected.as_slice());
        assert!(ram[0x4000..0x4000 + expected.len()].iter().all(|&byte| byte == 0xa5));
    }

    fn loaded_graphics_test_inputs() -> Option<(ZeldaState, Vec<u8>)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let asset_path = std::env::var_os("ZELDA3_ASSETS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| root.join("zelda3_assets.dat"));
        let rom_path = std::env::var_os("ZELDA3_ROM")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| root.join("saves/zelda3.sfc"));
        let (Ok(assets), Ok(rom)) = (std::fs::read(asset_path), std::fs::read(rom_path)) else {
            return None;
        };
        let mut state = ZeldaState::new();
        state.set_assets(&assets).unwrap();
        Some((state, rom))
    }

    fn assert_cursor_matches_shadow(
        rom: &[u8], entry_pc: u32, sheet: usize, destination: usize,
        cursor: GraphicsDecompressionCursor,
    ) {
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        const RETURN_TO_MAIN_WAIT: [u8; 2] = [0x35, 0x80];
        let checkpoint = RomCpuCheckpoint {
            entry_pc, stop_pc: 0x00_8036,
            a: 0, x: 0, y: sheet as u16, sp: 0x01fc, dp: 0, db: 0,
            carry: false, zero: false, overflow: false, negative: false,
            interrupt_disable: true, decimal: false,
            accumulator_is_8_bit: true, index_is_8_bit: true,
            emulation: false, waiting: false,
            stack_address: 0x01fd, stack_bytes: &RETURN_TO_MAIN_WAIT,
        };
        let mut ram = vec![0; 0x2_0000];
        ram[0] = destination as u8;
        ram[1] = (destination >> 8) as u8;
        ram[2] = ((0x7e_0000 + destination as u32) >> 16) as u8;
        let mut run = RomCpuTimingRun::new(
            rom, &ram, &[0; 0x2000], &snes::PpuState::default(),
            &snes::DmaState::default(), [0; 4], checkpoint,
        ).unwrap();
        run.enable_cpu_write_trace();
        let mut elapsed = 0u64;
        let mut writes = Vec::new();
        for _ in 0..4_000_000 {
            if run.is_complete() { break; }
            elapsed += u64::from(run.step().master_cycles);
            for (address, value) in run.take_cpu_wram_writes() {
                if (destination..destination + 0x600).contains(&address) {
                    writes.push((elapsed, address - destination, value));
                }
            }
        }
        assert!(run.is_complete(), "sheet {sheet} did not return");
        assert_eq!(writes.len(), cursor.stores.len());
        for (index, (at, offset, value)) in writes.into_iter().enumerate() {
            assert_eq!((at, offset, value), (
                cursor.stores[index].after_master_cycles,
                usize::from(cursor.stores[index].output_offset),
                cursor.bytes[index],
            ), "sheet {sheet} store {index}");
        }
    }
}

impl GraphicsDecompressionCursor {
    fn new(destination: usize, bytes: Vec<u8>, stores: Vec<DecompressStore>, total_master_cycles: u64) -> Option<Self> {
        if bytes.len() != stores.len()
            || stores.iter().enumerate().any(|(index, store)| store.output_offset as usize != index)
        {
            return None;
        }
        Some(Self { destination, bytes, stores, next_store: 0, total_master_cycles })
    }

    fn publish_through(&mut self, elapsed_master_cycles: u64, ram: &mut [u8]) {
        while self.next_store < self.stores.len()
            && self.stores[self.next_store].after_master_cycles <= elapsed_master_cycles
        {
            let offset = self.stores[self.next_store].output_offset as usize;
            ram[self.destination + offset] = self.bytes[self.next_store];
            self.next_store += 1;
        }
    }

    fn is_complete(&self) -> bool {
        self.next_store == self.stores.len()
    }
}

pub(super) const fn animated_sprite_tile_secondary_sheet(tile: u8) -> u8 {
    if tile == 0x23 || tile >= 0x37 {
        0x5d
    } else if tile == 0x0c || tile >= 0x24 {
        0x5c
    } else {
        0x5b
    }
}

pub(crate) fn native_animated_item_table_byte(address: u32) -> Option<u8> {
    let offset = address.checked_sub(0xd469)? as usize;
    let value = *ANIMATED_SPRITE_TILE_SOURCE_OFFSETS.get(offset / 2)? as u16;
    Some(value.to_le_bytes()[offset & 1])
}

const ITEM_ANIMATION_GFX_SOURCE_GROUPS: [usize; 10] = [0, 11, 8, 38, 42, 45, 34, 3, 33, 46];
const FOLLOWER_GFX_DECOMPRESSION_OFFSETS: [usize; 14] = [
    0, 0x600, 0x300, 0x300, 0x300, 0, 0, 0x900, 0x600, 0x600, 0x900, 0x900, 0x600, 0x900,
];
const SWORD_TYPE_TO_GFX_OFFSETS: [usize; 5] = [0, 0, 0x120, 0x120, 0x120];
const SHIELD_TYPE_TO_GFX_OFFSETS: [usize; 4] = [0x660, 0x660, 0x6f0, 0x900];

/// Equipment selectors shared by the semantic and retained graphics callers.
pub(crate) fn native_gear_graphics_table_byte(address: u32) -> Option<u8> {
    let (offset, table): (usize, &[usize]) = match address {
        0x00d2be..=0x00d2c7 => ((address - 0x00d2be) as usize, &SWORD_TYPE_TO_GFX_OFFSETS),
        0x00d300..=0x00d307 => ((address - 0x00d300) as usize, &SHIELD_TYPE_TO_GFX_OFFSETS),
        _ => return None,
    };
    Some((table[offset / 2] as u16).to_le_bytes()[offset & 1])
}

const INCREMENTAL_VRAM_UPLOAD_DESTINATIONS: [u8; 16] = [
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
];
const INCREMENTAL_VRAM_UPLOAD_SOURCES: [u16; 16] = [
    0x0000, 0x0200, 0x0400, 0x0600, 0x0800, 0x0a00, 0x0c00, 0x0e00, 0x1000, 0x1200, 0x1400, 0x1600,
    0x1800, 0x1a00, 0x1c00, 0x1e00,
];

const AGAHNIM_WARP_SHADOW_PALETTE_OFFSETS: [usize; 3] = [0x160, 0x180, 0x1a0];
const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
pub(super) const SPOTLIGHT_RADIUS_STEP: u16 = 7;
const SPOTLIGHT_DELTA_SIZE: [i8; 4] = [-(SPOTLIGHT_RADIUS_STEP as i8), 7, 7, 7];
const SPOTLIGHT_GOAL: [u16; 4] = [0, 126, 35, 126];
/// Cycle ledger: what `IrisSpotlight_ConfigureTable` charges for its
/// `$00:F3A3-$F3B2` V-counter wait loop, the single exiting pass (BCC not
/// taken). Every extra pass the ROM spins before scanline 192 costs 178 more
/// and is not charged: it depends on the raster phase at entry.
const SPOTLIGHT_VCOUNTER_WAIT_FINAL_PASS_MASTER_CYCLES: u64 = 172;
const SPOTLIGHT_CIRCLE_X_RADIUS_CURVE: [u8; 129] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0xfc, 0xfc, 0xfc, 0xfb, 0xfb, 0xfb, 0xfa, 0xfa, 0xf9, 0xf9, 0xf8, 0xf8,
    0xf7, 0xf7, 0xf6, 0xf6, 0xf5, 0xf5, 0xf4, 0xf3, 0xf3, 0xf2, 0xf1, 0xf1, 0xf0, 0xef, 0xee, 0xee,
    0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe9, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xdf, 0xde,
    0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7, 0xd6, 0xd5, 0xd3, 0xd2, 0xd0, 0xcf, 0xcd, 0xcc, 0xca, 0xc9,
    0xc7, 0xc6, 0xc4, 0xc2, 0xc1, 0xbf, 0xbd, 0xbb, 0xb9, 0xb7, 0xb6, 0xb4, 0xb1, 0xaf, 0xad, 0xab,
    0xa9, 0xa7, 0xa4, 0xa2, 0x9f, 0x9d, 0x9a, 0x97, 0x95, 0x92, 0x8f, 0x8c, 0x89, 0x86, 0x82, 0x7f,
    0x7b, 0x78, 0x74, 0x70, 0x6c, 0x67, 0x63, 0x5e, 0x59, 0x53, 0x4d, 0x46, 0x3f, 0x37, 0x2d, 0x1f,
    0,
];

impl ZeldaState {
    pub(super) fn filter_majorly_whiten_bg_marked(&mut self) {
        // $0E:D757..D7FD: eight passes over twelve colors. D7FE's existing
        // scalar color body is unchanged and excluded from these markers.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x0ed757));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x0ed757);
        for i in 0..8 {
            for pc in [0x0ed75b, 0x0ed762, 0x0ed76d, 0x0ed778, 0x0ed783,
                0x0ed78e, 0x0ed799, 0x0ed7a4, 0x0ed7af, 0x0ed7ba, 0x0ed7c5, 0x0ed7d0] { b.block(pc); }
            b.block(0x0ed7db);
            if i == 7 { b.taken(0x0ed7e3); } else { b.block(0x0ed7e5); }
        }
        b.block(0x0ed7e8);
        if self.game_state.display.palette_buffer.aux_color(0) != 0 { b.taken(0x0ed7f3); }
        else { b.block(0x0ed7f5); }
        b.block(0x0ed7f6);
        self.filter_majorly_whiten_bg();
    }


    pub(crate) fn native_overworld_palette_selection_byte(address: u32) -> Option<u8> {
        let address=address&0x7f_ffff;
        if (0x0ed504..0x0ed580).contains(&address) {
            let offset=(address-0x0ed504) as usize;
            return Some(if offset%4==3 {255} else {OW_BG_PAL_INFO[offset/4*3+offset%4] as u8});
        }
        if (0x0ed580..0x0ed5a8).contains(&address) {
            return Some(OW_SPR_PAL_INFO[(address-0x0ed580) as usize] as u8);
        }
        None
    }

    /// Native graphics tables and compressed streams at their data addresses.
    pub(crate) fn native_graphics_asset_byte(&self, address: u32) -> Option<u8> {
        let address=address&0x7f_ffff;
        if let Some(value)=crate::cycle_models::graphics_sources::table_byte(address)
            .or_else(||native_tileset_table_byte(address)) {return Some(value);}
        let offset=|a:u32|((a>>16)&127)*0x8000+(a&0x7fff);
        for &(sprite,sheet,start) in crate::cycle_models::graphics_sources::SOURCES {
            let Some(at)=offset(address).checked_sub(offset(start)) else {continue};
            let Some(data)=self.asset_bytes(if sprite {64}else{65},sheet) else {continue};
            if let Some(&value)=data.get(at as usize) {return Some(value);}
        }
        None
    }

    pub(crate) fn native_spotlight_table_byte(address: u32) -> Option<u8> {
        match address {
            0x00f2fb..=0x00f301 => {
                Some([0xf8, 0, 0x1b, 0xf8, 0xf0, 0x1b, 0][(address - 0xf2fb) as usize])
            }
            0x00f302..=0x00f309 => {
                let offset = (address - 0xf302) as usize;
                Some((SPOTLIGHT_DELTA_SIZE[offset / 2] as i16).to_le_bytes()[offset & 1])
            }
            0x00f30a..=0x00f311 => {
                let offset = (address - 0xf30a) as usize;
                Some(SPOTLIGHT_GOAL[offset / 2].to_le_bytes()[offset & 1])
            }
            0x00f44b..=0x00f4cb => {
                Some(SPOTLIGHT_CIRCLE_X_RADIUS_CURVE[(address - 0xf44b) as usize])
            }
            _ => None,
        }
    }

    fn sprite_gfx_subset(&self, slot: usize) -> u8 {
        self.game_state.sprites.workspace.graphics_subset(slot)
    }

    /// Public read-only accessor for the per-area sprite CHR identity: the 4
    /// `graphics_subset` packs (`slot` 0..4) loaded into VRAM by the sprite
    /// graphics loader. Used by the sprite-sheets probe/dump to key sprite cells
    /// by a stable per-area context (a hash of these 4 values). `slot >= 4`
    /// returns 0.
    pub fn parity_probe_sprite_graphics_subset(&self, slot: usize) -> u8 {
        if slot >= 4 {
            return 0;
        }
        self.sprite_gfx_subset(slot)
    }

    fn set_sprite_gfx_subset(&mut self, slot: usize, pack: u8) {
        self.sprite_workspace_mut().set_graphics_subset(slot, pack);
    }

    pub(super) fn world_map_setup_hdma(&mut self) {
        self.set_bg1_x(0x0080);
        self.set_bg1_y(0x00c8);
        self.set_mode7_center_y(0x01c9);
        self.set_mode7_center_x(0x0100);
        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0);
        self.set_main_screen_window_layers(0);
        self.set_sub_screen_window_layers(0);
        if self.game_state.frame.main_module == 20 {
            self.hdma_setup(0x0abddd, 0x0abddd, 0x42, 0x1b, 0x1e, 0);
            self.set_hdma_enable_mask(0xc0);
        } else if self.game_state.frame.submodule != 10 {
            self.set_mode7_zoom_step_counter(4);
            self.set_mode7_zoom_timer(12);
            self.set_overworld_map_flags(1);
            let y = self.game_state.player.special_exit_position.map_zoom_y();
            self.set_bg1_y(y);
            self.set_mode7_center_y(y.wrapping_add(0x100));
            let t0 = self
                .game_state
                .player
                .special_exit_position
                .map_zoom_x_offset();
            let abs_t0 = if (t0 as i16) < 0 {
                0u16.wrapping_sub(t0)
            } else {
                t0
            };
            let t1 = abs_t0.wrapping_mul(5) >> 1;
            let t2 = if (t0 as i16) < 0 {
                0u16.wrapping_sub(t1)
            } else {
                t1
            };
            self.set_bg1_x(t2.wrapping_add(0x80) & !1);
            self.overworld_map_setup_hdma();
            self.set_hdma_enable_mask(0xc0);
        } else {
            self.set_mode7_zoom_step_counter(4);
            self.set_mode7_zoom_timer(33);
            self.set_overworld_map_flags(0);
            self.hdma_setup(0x0abdcf, 0x0abdcf, 0x42, 0x1b, 0x1e, 10);
            self.set_hdma_enable_mask(0xc0);
        }
    }

    pub(super) fn load_overworld_map_palette(&mut self) {
        if let Some(palette) = self.asset_raw(93).map(Vec::from) {
            let offset = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
                0x100
            } else {
                0
            };
            let len = 0x100.min(palette.len().saturating_sub(offset));
            self.copy_main_palette_bytes_tagged(
                &palette[offset..offset + len],
                len,
                PaletteSliceSource::AssetBytes,
            );
        }
    }

    pub(super) fn load_actual_gear_palettes(&mut self) {
        let inventory = &self.game_state.inventory.items;
        self.load_gear_palettes(
            inventory.sword_type(),
            inventory.shield_type(),
            inventory.armor(),
        );
        if self
            .game_state
            .enhanced_features
            .has(LOAD_GFX_FEATURES0_MISC_BUG_FIXES)
        {
            self.palette_update_gloves_color();
        }
    }

    pub(super) fn palette_electro_themed_gear(&mut self) {
        self.load_gear_palettes(2, 2, 4);
    }

    /// ROM: `LoadGearPalettes_bunny`
    pub(super) fn load_gear_palettes_bunny(&mut self) {
        self.load_gear_palettes(
            self.game_state.inventory.items.sword_type(),
            self.game_state.inventory.items.shield_type(),
            3,
        );
    }

    pub(super) fn load_gear_palettes(&mut self, sword: u8, shield: u8, armor: u8) {
        let sword_index = if sword != 0 && sword != 0xff {
            sword.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(82, sword_index * 3, 0x1b2, 2);

        let shield_index = if shield != 0 {
            shield.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(83, shield_index * 4, 0x1b8, 3);

        self.load_gear_palette_from_asset(81, armor as usize * 15, 0x1e2, 14);
        self.increment_cgram_update_flag();
    }

    /// ROM: `Palette_UpdateGlovesColor`
    pub(super) fn palette_update_gloves_color(&mut self) {
        let gloves = self.game_state.inventory.items.gloves();
        if gloves != 0 {
            let color = self.gloves_color(gloves.wrapping_sub(1) as usize);
            self.set_aux_color_asset(0xfd, color);
            self.set_main_color_asset(0xfd, color);
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_load_for_file_select(&mut self) {
        for slot in 0..3 {
            let src = slot * 0x500;
            let k = slot * 0x20;
            self.palette_load_for_file_select_armor(
                k,
                self.sram[src + KSRM_OFFS_ARMOR],
                self.sram[src + KSRM_OFFS_GLOVES],
            );
            self.palette_load_for_file_select_sword(k, self.sram[src + KSRM_OFFS_SWORD]);
            self.palette_load_for_file_select_shield(k, self.sram[src + KSRM_OFFS_SHIELD]);
        }

        let Some(main_sprite_palette) = self.asset_raw(80).map(Vec::from) else {
            return;
        };
        for i in 0..7 {
            let color = read_word_from_slice(&main_sprite_palette, (7 + i) * 2);
            self.set_aux_color_asset(0xe8 + i, color);
            self.set_main_color_asset(0xe8 + i, color);

            let color = read_word_from_slice(&main_sprite_palette, (15 + 7 + i) * 2);
            self.set_aux_color_asset(0xf8 + i, color);
            self.set_main_color_asset(0xf8 + i, color);
        }
    }

    pub(super) fn palette_load_for_file_select_armor(&mut self, k: usize, armor: u8, gloves: u8) {
        let Some(palette) = self.asset_raw(81).map(Vec::from) else {
            return;
        };
        let src = armor as usize * 15 * 2;
        for i in 0..15 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color_asset(k + 0x81 + i, color);
            self.set_main_color_asset(k + 0x81 + i, color);
        }
        if gloves != 0 {
            let color = self.gloves_color(gloves.wrapping_sub(1) as usize);
            self.set_aux_color_asset(k + 0x8d, color);
            self.set_main_color_asset(k + 0x8d, color);
        }
    }

    fn gloves_color(&self, glove_index: usize) -> u16 {
        self.replay_gloves_color(glove_index)
    }

    pub(super) fn palette_load_for_file_select_sword(&mut self, k: usize, sword: u8) {
        let Some(palette) = self.asset_raw(82).map(Vec::from) else {
            return;
        };
        let sword = if sword != 0 { sword.wrapping_sub(1) } else { 0 } as usize;
        let src = sword * 3 * 2;
        for i in 0..3 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color_asset(k + 0x99 + i, color);
            self.set_main_color_asset(k + 0x99 + i, color);
        }
    }

    pub(super) fn palette_load_for_file_select_shield(&mut self, k: usize, shield: u8) {
        let Some(palette) = self.asset_raw(83).map(Vec::from) else {
            return;
        };
        let shield = if shield != 0 {
            shield.wrapping_sub(1)
        } else {
            0
        } as usize;
        let src = shield * 4 * 2;
        for i in 0..4 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color_asset(k + 0x9c + i, color);
            self.set_main_color_asset(k + 0x9c + i, color);
        }
    }

    pub(super) fn load_gear_palette_from_asset(
        &mut self,
        asset: usize,
        color_offset: usize,
        dst: usize,
        x_ents: usize,
    ) {
        let Some(palette) = self.asset_raw(asset).map(Vec::from) else {
            return;
        };
        let base = color_offset * 2;
        let dst_index = dst >> 1;
        for i in 0..=x_ents {
            let color = read_word_from_slice(&palette, base + i * 2);
            self.set_aux_color_asset(dst_index + i, color);
            self.set_main_color_asset(dst_index + i, color);
        }
    }

    pub(super) fn reset_hud_palettes_4_and_5(&mut self) {
        for instruction in 0..16 {
            self.retire_hud_palette_reset_instruction(instruction);
        }
    }

    /// $00:EB29..EB5D. The initializer retains the instruction index across
    /// suspension; other callers drain the same owner synchronously.
    pub(super) fn retire_hud_palette_reset_instruction(&mut self, instruction: u8) -> u32 {
        let cost = [22, 24, 48, 48, 48, 48, 48, 48, 48, 48, 48, 24, 48, 22, 38, 44]
            [usize::from(instruction)];
        crate::cycle_ledger::charge_routine(0x00_eb29, u64::from(cost));
        match instruction {
            2..=9 => self.set_main_color_constant(16 + usize::from(instruction - 2), 0),
            10 => self.set_countdown_word(0),
            12 => self.set_darkening_or_lightening_screen_word(2),
            14 => { self.increment_cgram_update_flag(); }
            _ => {}
        }
        cost
    }

    pub(super) fn palette_filter_history(&mut self) {
        self.palette_filter_range(0x10, 0x18);
        self.palette_filter_incr_countdown();
    }

    pub(super) fn palette_filter_range(&mut self, from: usize, to: usize) {
        let countdown = self.game_state.display.palette_filter.countdown_word();
        let darkening = self.game_state.display.palette_filter.is_darkening();
        self.transform_main_range(
            from,
            to,
            PaletteTransform::FilterRangeStep {
                countdown,
                darkening,
            },
        );
    }

    pub(super) fn palette_filter_incr_countdown(&mut self) {
        let countdown = self
            .game_state
            .display
            .palette_filter
            .countdown_word()
            .wrapping_add(1);
        if countdown == 0x1f {
            self.set_countdown_word(0);
            let mode = self
                .game_state
                .display
                .palette_filter
                .darkening_or_lightening_screen_word()
                ^ 2;
            self.set_darkening_or_lightening_screen_word(mode);
            if mode != 0 {
                // $00:EBBE `INC $27` (16-bit): the attract legend flag in
                // module $14, Link's actual Y/X velocity word elsewhere.
                if self.game_state.frame.main_module == 0x14 {
                    self.attract_scene_mut().increment_legend_flag();
                } else {
                    let link = self.game_state.player.follower_link;
                    let [y, x] = u16::from_le_bytes([link.actual_y_velocity(), link.actual_x_velocity()])
                        .wrapping_add(1)
                        .to_le_bytes();
                    self.follower_link_state_mut().set_actual_velocity_xy(x, y);
                }
            }
        } else {
            self.set_countdown_word(countdown);
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_fade_intro_one_step(&mut self) {
        self.palette_filter_restore_additive(0x100, 0x1a0);
        self.palette_filter_restore_additive(0x0c0, 0x100);
        self.decrement_countdown();
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_fade_intro2(&mut self) {
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.decrement_countdown();
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_RestoreAdditive`
    pub(super) fn palette_filter_restore_additive(&mut self, from: usize, to: usize) {
        // $00:EDCA (JSR target): per colour 378, each channel that differs
        // from the target steps ($EDED/$EDFE/$EE0F, 66; BEQ taken, 6, when
        // equal), 138 for the store and BNE taken back on all but the last;
        // 32 entry, 42 RTS.
        let _scope = crate::cycle_ledger::routine(0x00_edca);
        let palette = &self.game_state.display.palette_buffer;
        let mut cost = 32 + 42;
        for i in from >> 1..to >> 1 {
            let (c, d) = (palette.main_color(i), palette.aux_color(i));
            cost += 378 + 120 + 120 + 138 + 6;
            for mask in [0x001f, 0x03e0, 0x7c00] {
                cost += if c & mask != d & mask { 66 } else { 6 };
            }
        }
        crate::cycle_ledger::charge(cost - 6);
        self.transform_main_range(from >> 1, to >> 1, PaletteTransform::RestoreAdditiveStep);
    }

    pub(super) fn filter_majorly_whiten_bg(&mut self) {
        if self.lag_free {
            // $0E:D7FE..D839, M16/X8: 572 clocks of straight-line self
            // work per color. Each channel takes BCC (+6) when below its
            // saturation value, otherwise executes LDA #clamp (24).
            // Price the ROM's +14 branches from each input color; the
            // optional dim-flashes output transform does not change them.
            for i in 32..128 {
                let color = self.game_state.display.palette_buffer.aux_color(i);
                let mut cost = 572;
                for shift in [0, 5, 10] {
                    cost += if ((color >> shift) & 31) + 14 < 31 { 6 } else { 24 };
                }
                crate::cycle_ledger::charge_routine(0x0e_d7fe, cost);
            }
        }
        let amount = if self
            .game_state
            .enhanced_features
            .has(LOAD_GFX_FEATURES0_DIM_FLASHES)
        {
            3
        } else {
            14
        };
        self.transform_main_range(32, 128, PaletteTransform::WhitenStep { amount });
        if self.game_state.display.palette_buffer.aux_color(0) != 0 {
            self.copy_color((Bank::Main, 32), (Bank::Main, 0));
        } else {
            self.set_main_color_constant(0, 0);
        }
    }

    pub(super) fn palette_restore_bg_from_flash(&mut self) {
        // $08:AAD5 and $1D:E9D3 JSL $0E:D83A; neither caller prices this body.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x0ed83a));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x0ed83a);
        // D83E has twelve word copies at each of eight source X offsets.
        for offset in 0..8 {
            b.block(0x0ed83e);
            for column in 0..12 {
                let i = 32 + column * 8 + offset;
                self.copy_color((Bank::Aux, i), (Bank::Main, i));
            }
            if offset != 7 { b.taken(0x0ed8a2); }
        }
        b.block(0x0ed8a4);
        self.copy_color((Bank::Main, 32), (Bank::Main, 0));
        if self.game_state.world.location.is_indoors() { b.taken(0x0ed8b0); }
        else {
            b.block(0x0ed8b2);
            let screen = self.game_state.world.location.overworld_screen_index();
            if screen == 0 { b.taken(0x0ed8c6); }
            else {
                b.block(0x0ed8c8);
                if screen == 0x40 { b.taken(0x0ed8ca); }
                else {
                    b.block(0x0ed8cc);
                    if screen == 0x5b { b.taken(0x0ed8ce); b.block(0x0ed8f4); }
                    else {
                        b.block(0x0ed8d0);
                        let matched = if screen == 3 { b.taken(0x0ed8d8); true }
                        else {
                            b.block(0x0ed8da);
                            if screen == 5 { b.taken(0x0ed8dc); true }
                            else {
                                b.block(0x0ed8de);
                                if screen == 7 { b.taken(0x0ed8e0); true }
                                else {
                                    b.block(0x0ed8e2);
                                    if screen == 0x43 { b.taken(0x0ed8ea); true }
                                    else {
                                        b.block(0x0ed8ec);
                                        if screen == 0x45 { b.taken(0x0ed8ee); true }
                                        else {
                                            b.block(0x0ed8f0);
                                            if screen != 0x47 { b.taken(0x0ed8f2); false } else { true }
                                        }
                                    }
                                }
                            }
                        };
                        if matched { b.block(0x0ed8f4); }
                    }
                }
            }
        }
        self.palette_restore_coldata();
        // D8F8 SEP #$10 has width-independent work at either incoming X width.
        b.block(0x0ed8f8);
    }

    /// ROM: `Palette_Restore_BG_And_HUD`
    pub(super) fn palette_restore_bg_and_hud(&mut self) {
        let src = self.game_state.display.palette_buffer.aux_full_slice()[..0x100].to_vec();
        self.copy_main_palette_bytes_tagged(&src, 0x100, PaletteSliceSource::MirrorBank(Bank::Aux));
        self.increment_cgram_update_flag();
        self.palette_restore_coldata();
    }

    pub(super) fn palette_restore_coldata(&mut self) {
        if self.game_state.world.location.is_indoors() {
            return;
        }
        let rgb = match self.game_state.world.location.overworld_screen_index() {
            3 | 5 | 7 => 0x8c4c26u32,
            0x43 | 0x45 | 0x47 => 0x874a26,
            0x5b => 0x894f33,
            _ => 0x804020,
        };
        self.set_fixed_color_red(rgb as u8);
        self.set_fixed_color_green((rgb >> 8) as u8);
        self.set_fixed_color_blue((rgb >> 16) as u8);
    }

    pub(super) fn overworld_load_all_palettes(&mut self) {
        self.clear_aux_sprite_subpalettes();
        self.clear_main_full();
        self.overworld_load_all_palettes_after_clear();
    }

    pub(super) fn overworld_load_all_palettes_after_clear(&mut self) {
        self.set_overworld_palette_mode(5);
        self.world_palette_theme_mut()
            .set_overworld_palette_aux1_hi(3);
        self.set_overworld_palette_aux2_hi(3);
        self.set_overworld_palette_aux3_lo(0);
        self.set_sp6r_indoors(5);
        self.set_sp0l(11);
        self.follower_state_mut().set_palette_swap_flag(0);
        self.clear_overworld_aux_or_main_offset();

        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_ow_bg_main();
        self.palette_load_ow_bg1();
        self.palette_load_ow_bg2();
        self.palette_load_ow_bg3();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();

        for i in 0..8 {
            self.copy_color((Bank::Aux, 0xe8 + i), (Bank::Main, 0xd8 + i));
        }
    }

    /// ROM: `Dungeon_LoadPalettes`
    pub(super) fn dungeon_load_palettes(&mut self) {
        self.dungeon_load_palettes_before_cache();
        self.overworld_copy_palettes_to_cache();
    }

    pub(super) fn dungeon_load_palettes_before_cache(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x02_c630));
        if self.lag_free {
            // $02:C630..C68B: caller stores, eleven JSLs and shared backup
            // setup. $C769's cache loop and every palette child own their work.
            crate::cycle_ledger::charge(1192);
        }
        self.clear_overworld_aux_or_main_offset();
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_sprite_environment();
        self.palette_load_link_armor_and_gloves();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.overworld_load_palettes_before_cache();
    }

    /// ROM: `Overworld_LoadPalettesInner`
    pub(super) fn overworld_load_palettes_inner(&mut self) {
        self.overworld_load_palettes_before_cache();
        self.overworld_copy_palettes_to_cache();
    }

    fn overworld_load_palettes_before_cache(&mut self) {
        let main_indoors = self
            .game_state
            .display
            .palette_buffer
            .main_palette_indoors();
        let aux3_lo = self
            .game_state
            .display
            .palette_buffer
            .overworld_palette_aux3_lo();
        let main_indoors_copy = self
            .game_state
            .world
            .palette_theme
            .palette_main_indoors_copy();
        self.set_overworld_main_indoors_palette_backup(main_indoors);
        self.set_overworld_aux3_bg_palette_7_backup(aux3_lo);
        self.set_overworld_main_indoors_copy_palette_backup(main_indoors_copy);
        self.set_darkening_or_lightening_screen_word(2);
        self.set_countdown_word(0);
        self.clear_mosaic_target_level_word();
    }

    /// ROM: `Palette_Load_Sp0L`
    pub(super) fn palette_load_sp0l(&mut self) {
        let src = PALETTE_SPRITE_AUX3_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_0_left() as u32
                * 7
                * 2;
        let dst = if self.palette_swap_enabled() {
            0x1e2
        } else {
            0x102
        };
        self.palette_load_single(src, dst, 6);
    }

    pub(super) fn palette_load_sp5l(&mut self) {
        let src = PALETTE_SPRITE_AUX1_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_5_left() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0x1a2, 6);
    }

    pub(super) fn palette_load_sp6l(&mut self) {
        let src = PALETTE_SPRITE_AUX1_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_6_left() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0x1c2, 6);
    }

    /// ROM: `Palette_Load_Sword`
    pub(super) fn palette_load_sword(&mut self) {
        let sword = self.game_state.inventory.items.sword_type();
        let sword_index = if (sword as i8) > 0 {
            sword.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(82, sword_index * 3, 0x1b2, 2);
        self.increment_cgram_update_flag();
    }

    /// ROM: `Palette_Load_Shield`
    pub(super) fn palette_load_shield(&mut self) {
        let shield = self.game_state.inventory.items.shield_type();
        let shield_index = if shield != 0 {
            shield.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(83, shield_index * 4, 0x1b8, 3);
        self.increment_cgram_update_flag();
    }

    /// ROM: `Palette_Load_SpriteMain`
    pub(super) fn palette_load_sprite_main(&mut self) {
        let offset = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            60 * 2
        } else {
            0
        };
        self.palette_load_multiple(PALETTE_MAIN_SPRITE_SNES_ADDR + offset, 0x122, 14, 3);
    }

    pub(super) fn palette_load_ow_bg_main(&mut self) {
        let src = PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_mode() as u32
                * 35
                * 2;
        self.palette_load_multiple(src, 0x42, 6, 4);
    }

    pub(super) fn palette_load_ow_bg1(&mut self) {
        let src = PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR
            + self
                .game_state
                .world
                .palette_theme
                .overworld_palette_aux1_hi() as u32
                * 21
                * 2;
        self.palette_load_multiple(src, 0x52, 6, 2);
    }

    pub(super) fn palette_load_ow_bg2(&mut self) {
        let src = PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_aux2_hi() as u32
                * 21
                * 2;
        self.palette_load_multiple(src, 0xb2, 6, 2);
    }

    pub(super) fn palette_load_ow_bg2_word_range(&mut self, start: usize, end: usize) {
        let src = PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_aux2_hi() as u32
                * 21
                * 2;
        self.palette_load_multiple_word_range(src, 0xb2, 6, 2, start, end);
    }

    pub(super) fn palette_load_ow_bg3(&mut self) {
        let src = PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_aux3_lo() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0xe2, 6);
    }

    pub(super) fn palette_load_sprite_environment_dungeon(&mut self) {
        let src = PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_6_right_indoors() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0x1d2, 6);
    }

    pub(super) fn palette_load_sprite_environment(&mut self) {
        if self.game_state.world.location.is_indoors() {
            self.palette_load_sprite_environment_dungeon();
        } else {
            self.palette_misc_sprite_outdoors();
        }
    }

    pub(super) fn palette_misc_sprite_outdoors(&mut self) {
        let t = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            9
        } else {
            7
        };
        let src = PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR + t * 7 * 2;
        let dst = if self.palette_swap_enabled() {
            0x1f2
        } else {
            0x112
        };
        self.palette_load_single(src, dst, 6);
        self.palette_load_single(src - 7 * 2, 0x1d2, 6);
    }

    /// ROM: `Palette_Load_HUD`
    pub(super) fn palette_load_hud(&mut self) {
        let src = HUD_PALETTE_SNES_ADDR
            + self.game_state.display.palette_buffer.hud_palette() as u32 * 32 * 2;
        self.palette_load_multiple(src, 0, 15, 1);
    }

    pub(super) fn palette_load_multiple_arbitrary_snes(
        &mut self,
        src: u32,
        dst: usize,
        x_ents: usize,
    ) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1b_ef7b));
        if self.lag_free {
            // $1B:EF7B..EF95, M16/X16: setup, paired palette stores,
            // actual DEY/BPL passes, RTS. This entry has no callees.
            crate::cycle_ledger::charge(84 + (x_ents as u64 + 1) * 318
                + x_ents as u64 * 6 + 42);
        }
        let dst_index = dst >> 1;
        for i in 0..=x_ents {
            let Some(color) = self.rom_or_asset_word_snes(src + i as u32 * 2) else {
                return;
            };
            self.set_aux_color_asset(dst_index + i, color);
            self.set_main_color_asset(dst_index + i, color);
        }
    }

    pub(super) fn palette_load_dungeon_set(&mut self) {
        let src = PALETTE_DUNGEON_BG_MAIN_SNES_ADDR
            + (self
                .game_state
                .display
                .palette_buffer
                .main_palette_indoors()
                >> 1) as u32
                * 90
                * 2;
        self.palette_load_multiple(src, 0x42, 14, 5);
        let dst = if self.palette_swap_enabled() {
            0x1e2
        } else {
            0x112
        };
        self.palette_load_single(src, dst, 6);
    }

    /// ROM: `Palette_Load_LinkArmorAndGloves`
    pub(super) fn palette_load_link_armor_and_gloves(&mut self) {
        let armor = self.game_state.inventory.items.armor() as usize;
        let Some(palette) = self.asset_raw(81).map(Vec::from) else {
            return;
        };
        let src = armor * 15 * 2;
        for i in 0..15 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color_asset((0x1e2 >> 1) + i, color);
            self.set_main_color_asset((0x1e2 >> 1) + i, color);
        }
        self.palette_update_gloves_color();
    }

    pub(super) fn palette_load_single(&mut self, src: u32, dst: usize, x_ents: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1b_ef30));
        if self.lag_free {
            // $1B:EF30..EF4A, M16/X16: setup and one auxiliary-bank copy.
            crate::cycle_ledger::charge(138 + (x_ents as u64 + 1) * 270
                + x_ents as u64 * 6 + 42);
        }
        let base_byte = ((dst
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_aux_or_main_offset() as usize)
            >> 1)
            * 2;
        let base_idx = base_byte >> 1;
        for i in 0..=x_ents {
            let Some(color) = self.rom_or_asset_word_snes(src + i as u32 * 2) else {
                return;
            };
            self.set_aux_color_asset(base_idx + i, color);
        }
    }

    pub(super) fn palette_load_multiple(
        &mut self,
        src: u32,
        dst: usize,
        x_ents: usize,
        y_pals: usize,
    ) {
        let count = x_ents + 1;
        self.palette_load_multiple_word_range(src, dst, x_ents, y_pals, 0, count * (y_pals + 1));
    }

    fn palette_load_multiple_word_range(
        &mut self,
        src: u32,
        dst: usize,
        x_ents: usize,
        y_pals: usize,
        start: usize,
        end: usize,
    ) {
        let row_words = x_ents + 1;
        let total_words = row_words * (y_pals + 1);
        assert!(start <= end && end <= total_words);
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1b_ef4b));
        if self.lag_free && start == 0 && end != 0 {
            // $1B:EF4B..EF54, M16/X16. A partial palette continuation
            // retains this entry; only the first nonempty slice owns it.
            crate::cycle_ledger::charge(152);
        }
        let palette_offset = self
            .game_state
            .display
            .palette_buffer
            .overworld_aux_or_main_offset() as usize;
        for ordinal in start..end {
            let row = ordinal / row_words;
            let column = ordinal % row_words;
            let Some(color) = self.rom_or_asset_word_snes(src + ordinal as u32 * 2) else {
                return;
            };
            if self.lag_free {
                // $1B:EF56..EF7A: row prefix, each actual word's copy and
                // BPL, row suffix/BPL, and final RTS, each owned once even
                // when the caller suspends in the middle of a row.
                crate::cycle_ledger::charge(if column == 0 { 132 } else { 0 }
                    + 270 + if column + 1 < row_words { 6 } else {
                        172 + if row < y_pals { 6 } else { 42 }
                    });
            }
            let byte_offset = dst + row * 32 + palette_offset + column * 2;
            self.set_aux_color_asset(byte_offset >> 1, color);
        }
    }

    pub(super) fn load_item_gfx_into_wram_4bpp_buffer(&mut self) {
        let mut dst = 0x9480;
        dst = self.load_item_animation_gfx_one(dst, 7, 0, false);
        dst = self.load_item_animation_gfx_one(dst, 7, 1, false);
        dst = self.load_item_animation_gfx_one(dst, 3, 2, false);

        self.decompress_sprite_graphics_fixed_buffer(95, false);
        dst = self.load_item_animation_gfx_one(dst, 4, 3, true);
        dst = self.load_item_animation_gfx_one(dst, 3, 4, true);
        dst = self.load_item_animation_gfx_one(dst, 1, 5, true);
        dst = self.load_item_animation_gfx_one(dst, 4, 6, false);

        self.decompress_sprite_graphics_fixed_buffer(96, false);
        dst = self.load_item_animation_gfx_one(dst, 14, 7, true);
        dst = self.load_item_animation_gfx_one(dst, 7, 8, true);

        self.decompress_sprite_graphics_fixed_buffer(95, false);
        self.load_item_animation_gfx_one(dst, 2, 9, true);
        self.decompress_sprite_graphics_fixed_buffer(84, false);

        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.expand3_to_4_high_from_slice(0xa480, &tmp, 0, 0, 8);
        self.expand3_to_4_high_from_slice(0xa580, &tmp, 0x180, 0, 8);

        self.decompress_sprite_graphics_fixed_buffer(96, false);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.expand3_to_4_high_from_slice(0xb280, &tmp, 0, 0, 3);
        self.expand3_to_4_high_from_slice(0xb2e0, &tmp, 0x180, 0, 3);

        self.load_item_gfx_auxiliary();
    }

    pub(super) fn load_item_animation_gfx_one(
        &mut self,
        dst: usize,
        num: usize,
        r12: usize,
        from_temp: bool,
    ) -> usize {
        let Some(source) = (if from_temp {
            Some(self.graphics_primary_decompression_buffer(0x600))
        } else {
            self.asset_bytes(64, 0).map(Vec::from)
        }) else {
            return dst + 0x40 * num;
        };

        let src = ITEM_ANIMATION_GFX_SOURCE_GROUPS[r12] * 24;
        self.expand3_to_4_high_from_slice(dst, &source, src, 0, num);
        self.expand3_to_4_high_from_slice(dst + 0x20 * num, &source, src + 0x180, 0, num);
        dst + 0x40 * num
    }

    pub(super) fn load_item_gfx_auxiliary(&mut self) {
        self.decompress_background_graphics_fixed_buffer(0x0f);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(PEG_TILE_GFX_BUFFER, &tmp, 0, 16);

        self.decompress_sprite_graphics_fixed_buffer(0x58, false);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(0xb540, &tmp, 0, 32);

        self.decompress_background_graphics_fixed_buffer(0x05);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(0xbdc0, &tmp, 0x480, 2);
    }

    pub(super) fn load_follower_graphics(&mut self) {
        let first_sheet = self.follower_graphics_first_sheet();
        self.decompress_sprite_graphics_fixed_buffer(first_sheet, true);
        self.decompress_sprite_graphics_fixed_buffer(0x65, false);
        self.finish_follower_graphics_conversion();
    }

    fn follower_graphics_first_sheet(&self) -> usize {
        let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
        let mut yv = 0x64;
        if follower != 1 {
            yv = 0x66;
            if follower >= 9 {
                yv = 0x59;
                if follower >= 12 {
                    yv = 0x58;
                }
            }
        }
        yv
    }

    /// Commit one exact source-order prefix slice of either decompressed
    /// follower sheet without touching the uncommitted tail of its WRAM
    /// scratch buffer.
    pub(super) fn apply_follower_graphics_decompression_slice(
        &mut self,
        first_sheet: bool,
        start: u16,
        end: u16,
    ) {
        const SHEET_BYTES: u16 = 0x0600;
        assert!(start <= end && end <= SHEET_BYTES);
        let pack = if first_sheet {
            self.follower_graphics_first_sheet()
        } else {
            0x65
        };
        let data = self
            .decompressed_sprite_graphics_data(pack)
            .expect("rescued-maiden follower graphics require their source sheet");
        assert_eq!(
            data.len(),
            usize::from(SHEET_BYTES),
            "rescued-maiden follower sheet ${pack:02x} must expand to 1536 bytes",
        );
        let dst = if first_sheet {
            GraphicsDecompressionScratch::secondary_buffer_offset()
        } else {
            GraphicsDecompressionScratch::primary_buffer_offset()
        };
        let start = usize::from(start);
        let end = usize::from(end);
        self.copy_decompressed_graphics_to(dst + start, &data[start..end]);
    }

    /// Finish the source-order remainder of a `LoadFollowerGraphics` call
    /// whose two-sheet decompressor stopped at `stage`.
    pub(super) fn complete_follower_graphics_decompression(
        &mut self,
        stage: crate::RescuedMaidenInitializationStage,
    ) {
        use crate::RescuedMaidenInitializationStage::{
            Conversion, FirstFollowerSheet, SecondFollowerSheet,
        };
        match stage {
            FirstFollowerSheet { completed_bytes } => {
                self.apply_follower_graphics_decompression_slice(true, completed_bytes, 0x0600);
                self.apply_follower_graphics_decompression_slice(false, 0, 0x0600);
                self.apply_follower_graphics_conversion_slice(0, 512);
            }
            SecondFollowerSheet { completed_bytes } => {
                self.apply_follower_graphics_decompression_slice(false, completed_bytes, 0x0600);
                self.apply_follower_graphics_conversion_slice(0, 512);
            }
            Conversion { completed_stores } => {
                self.apply_follower_graphics_conversion_slice(completed_stores, 512);
            }
        }
    }

    /// Commit an exact source-order slice of the 3bpp-to-4bpp conversion at
    /// the tail of `LoadFollowerGraphics`. Each source store publishes one
    /// 16-bit plane word; keeping that granularity preserves boundaries which
    /// split a row between its low/high and upper-plane writes.
    pub(super) fn apply_follower_graphics_conversion_slice(
        &mut self,
        start_store: u16,
        end_store: u16,
    ) {
        const TOTAL_STORES: u16 = 32 * 8 * 2;
        assert!(start_store <= end_store && end_store <= TOTAL_STORES);
        let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
        let source = self.graphics_combined_decompression_buffers();
        let source_base = FOLLOWER_GFX_DECOMPRESSION_OFFSETS
            .get(follower)
            .copied()
            .unwrap_or(0);
        for store in start_store..end_store {
            let row_index = usize::from(store / 2);
            let tile = row_index / 8;
            let row = row_index % 8;
            let src = source_base + tile * 24;
            let dst = 0xb940 + tile * 32 + row * 2;
            if store & 1 == 0 {
                self.ram[dst] = source.get(src + row * 2).copied().unwrap_or(0);
                self.ram[dst + 1] = source.get(src + row * 2 + 1).copied().unwrap_or(0);
            } else {
                self.ram[dst + 0x10] = source.get(src + 0x10 + row).copied().unwrap_or(0);
                self.ram[dst + 0x11] = 0;
            }
        }
    }

    pub(super) fn finish_follower_graphics_conversion(&mut self) {
        let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
        let tmp = self.graphics_combined_decompression_buffers();
        let offset = FOLLOWER_GFX_DECOMPRESSION_OFFSETS
            .get(follower)
            .copied()
            .unwrap_or(0);
        self.do3_to_4_low_animated_from_slice(0xb940, &tmp, offset, 0x20);
    }

    pub(super) fn decompress_sword_graphics(&mut self) {
        self.replay_trace_ram_watch("loadgfx-before-decompress-sword");
        self.decompress_sprite_graphics_fixed_buffer(0x5f, true);
        self.decompress_sprite_graphics_fixed_buffer(0x5e, false);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        let sword = self.game_state.inventory.items.sword_type() as usize;
        let src = SWORD_TYPE_TO_GFX_OFFSETS.get(sword).copied().unwrap_or(0);
        self.expand3_to_4_high_from_slice(0x9000, &tmp, src, 0, 12);
        self.expand3_to_4_high_from_slice(0x9180, &tmp, src + 0x180, 0, 12);
        self.replay_trace_ram_watch("loadgfx-after-decompress-sword");
    }

    pub(super) fn decompress_shield_graphics(&mut self) {
        self.replay_trace_ram_watch("loadgfx-before-decompress-shield");
        self.decompress_sprite_graphics_fixed_buffer(0x5f, true);
        self.decompress_sprite_graphics_fixed_buffer(0x5e, false);
        let tmp = self.graphics_combined_decompression_buffers();
        let shield = self.game_state.inventory.items.shield_type() as usize;
        let src = SHIELD_TYPE_TO_GFX_OFFSETS
            .get(shield)
            .copied()
            .unwrap_or(0x660);
        self.expand3_to_4_high_from_slice(0x9300, &tmp, src, 0, 6);
        self.expand3_to_4_high_from_slice(0x93c0, &tmp, src + 0x180, 0, 6);
        self.replay_trace_ram_watch("loadgfx-after-decompress-shield");
    }

    /// ROM: `EraseTileMaps_normal`
    pub(super) fn erase_tile_maps_normal(&mut self) {
        self.erase_tile_maps(crate::cycle_models::tilemap_erase::Pattern::Normal);
    }

    pub(super) fn erase_tile_maps_triforce(&mut self) {
        self.erase_tile_maps(crate::cycle_models::tilemap_erase::Pattern::Triforce);
    }

    fn erase_tile_maps(&mut self, pattern: crate::cycle_models::tilemap_erase::Pattern) {
        let mut work = crate::cycle_models::tilemap_erase::TilemapEraseWork::new(pattern);
        while let Some(step) = work.step(&self.ram, &self.dma) {
            self.retire_tilemap_erase_step(step);
        }
    }

    pub(super) fn retire_tilemap_erase_step(&mut self, step: crate::cycle_models::tilemap_erase::Step) {
        use crate::cycle_models::tilemap_erase::Step;
        match step {
            Step::Cpu { master_cycles, write, pc } => {
                crate::cycle_ledger::charge_routine(u32::from(pc), u64::from(master_cycles));
                if let Some((address, value, word)) = write {
                    self.write_tilemap_erase_bus_byte(address, value as u8);
                    if word { self.write_tilemap_erase_bus_byte(address + 1, (value >> 8) as u8); }
                }
            }
            Step::DmaByte { source, register, last } => {
                // Fixed A-bus mode samples the live byte each transfer; a
                // retained continuation must not replace this with a fill.
                self.ppu.write(register, self.ram[usize::from(source)]);
                let channel = &mut self.dma.channel[1];
                channel.size = channel.size.wrapping_sub(1);
                if last {
                    assert_eq!(channel.size, 0);
                    channel.dma_active = false;
                    self.dma.dma_busy = false;
                }
            }
        }
    }

    fn write_tilemap_erase_bus_byte(&mut self, address: u16, value: u8) {
        match address {
            0..=3 => self.ram[usize::from(address)] = value,
            0x2115..=0x2117 => self.ppu.write(address as u8, value),
            0x420b => {
                assert_eq!(value, 2);
                self.account_nmi_dma_transfer(usize::from(self.dma.channel[1].size));
                self.dma.channel[1].dma_active = true;
                self.dma.channel[1].off_index = 0;
                self.dma.dma_busy = true;
                self.dma.dma_timer = 0;
            }
            0x4310 => {
                assert_eq!(value, 8);
                let channel = &mut self.dma.channel[1];
                channel.mode = 0;
                channel.fixed = true;
                channel.decrement = false;
                channel.indirect = false;
                channel.from_b = false;
                channel.unused_bit = false;
            }
            0x4311 => self.dma.channel[1].b_adr = value,
            0x4312 => self.dma.channel[1].a_adr = (self.dma.channel[1].a_adr & 0xff00) | u16::from(value),
            0x4313 => self.dma.channel[1].a_adr = (self.dma.channel[1].a_adr & 0xff) | (u16::from(value) << 8),
            0x4314 => self.dma.channel[1].a_bank = value,
            0x4315 => self.dma.channel[1].size = (self.dma.channel[1].size & 0xff00) | u16::from(value),
            0x4316 => self.dma.channel[1].size = (self.dma.channel[1].size & 0xff) | (u16::from(value) << 8),
            _ => panic!("unexpected tilemap erase bus address {address:04x}"),
        }
    }

    pub(super) fn load_default_graphics(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_e2d0));
        let Some(source) = self.asset_bytes(64, 0).map(Vec::from) else {
            return;
        };
        if self.lag_free {
            // $00:E2D0..E33A uses its own raw-plane CPU copy, not E5AF's
            // generic conversion. Replace that child price, never add both.
            // 64 tiles: eight paired-plane and eight opaque-plane stores.
            crate::cycle_ledger::charge(384 + 64 * (16 + 8 * 382 + 7 * 6
                + 16 + 8 * 336 + 7 * 6 + 30) + 63 * 6 + 340);
            crate::cycle_ledger::muted(|| self.do3_to_4_high_to_vram(
                0x4000, &source, chr_source::CHR_KIND_SPRITE, 0));
        } else {
            self.do3_to_4_high_to_vram(0x4000, &source, chr_source::CHR_KIND_SPRITE, 0);
        }
        if crate::debug_env::var_os!("ZELDA3_DEBUG_DEFAULT_GFX").is_some() {
            eprintln!(
                "default_gfx source_len={} vram40b0={:04x} vram40b1={:04x} vram40b2={:04x} vram40b3={:04x}",
                source.len(),
                self.ppu.vram[0x40b0],
                self.ppu.vram[0x40b1],
                self.ppu.vram[0x40b2],
                self.ppu.vram[0x40b3],
            );
        }
        self.decompress_and_upload_2bpp(0x7000, 0x6a);
        self.decompress_and_upload_2bpp(0x7400, 0x6b);
        self.decompress_and_upload_2bpp(0x7800, 0x69);
    }

    pub(super) fn transfer_font_to_vram(&mut self) {
        let Some(font_pack) = self.asset_memblk(95, 0) else {
            return;
        };
        let font = find_index_in_memblk(font_pack, 0).ptr.to_vec();
        for i in 0..0x800 {
            self.ppu.vram[0x7000 + i] = read_word_from_slice(&font, i * 2);
        }
    }

    pub(super) fn decompress_and_upload_2bpp(&mut self, dst: usize, pack: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_e33b));
        let Some(data) = self.decompressed_sprite_graphics_data(pack) else {
            return;
        };
        if self.lag_free {
            // $00:E33B..E36C: JSR, width/index setup, 256 four-word CPU
            // stores and pointer increments, taken BPLs and return.
            crate::cycle_ledger::charge(92 + 256 * 830 + 255 * 6 + 64);
            let _child = crate::cycle_ledger::routine(0x00_e766);
            // $00:E766..E770 destination setup; E772 decoder owns its bytes.
            crate::cycle_ledger::charge(128);
            self.charge_sheet_decompression(DecompressEntry::Sprite, 64, pack,
                GraphicsDecompressionScratch::primary_buffer_offset());
        }
        self.copy_to_primary_decompression_buffer(&data);
        for i in 0..0x400 {
            self.ppu.vram[dst + i] = read_word_from_slice(&data, i * 2);
        }
    }

    fn native_tileset_decompression_operations(&self) -> NativeTilesetGraphicsPublication {
        let main = load_gfx_main_tileset(
            self.game_state.world.palette_theme.main_tile_theme_index() as usize,
        );
        let aux = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        );
        let sprite = load_gfx_sprite_tileset(
            self.game_state.sprites.system.graphics_index() as usize,
        );
        let sprite_sheets: [usize; 4] = std::array::from_fn(|slot| usize::from(
            if sprite[slot] != 0 { sprite[slot] }
            else { self.game_state.sprites.workspace.graphics_subset(slot) }
        ));
        let aux_sheets: [usize; 4] = std::array::from_fn(|slot| usize::from(
            if aux[slot] != 0 { aux[slot] } else { main[slot + 3] }
        ));
        let mut operations: Vec<(u32, usize, GraphicsDecompressionCursor)> = Vec::new();
        for (slot, sheet) in sprite_sheets.into_iter().enumerate() {
            let destination = 0x7800 + slot * 0x600;
            operations.push((0x00_e772, sheet,
                self.sprite_decompression_cursor(sheet, destination)
                    .expect("map sprite sheet requires a timed translated decoder")));
        }
        for (slot, sheet) in [
            usize::from(main[0]), usize::from(main[1]), usize::from(main[2]),
            aux_sheets[0], aux_sheets[1], aux_sheets[2], aux_sheets[3],
            usize::from(main[7]),
        ].into_iter().enumerate() {
            let destination = match slot {
                3..=6 => 0x6000 + (slot - 3) * 0x600,
                _ => GraphicsDecompressionScratch::primary_buffer_offset(),
            };
            operations.push((0x00_e78f, sheet,
                self.background_decompression_cursor(sheet, destination)
                    .expect("map background sheet requires a timed translated decoder")));
        }
        NativeTilesetGraphicsPublication { sprite_sheets, operations }
    }

    pub(super) fn advance_pre_overworld_tileset_publication(
        &mut self,
        events: &[TilesetGraphicsPublication],
    ) {
        if events.is_empty() { return; }
        let publication = self.native_pre_overworld_tileset_publication.take();
        self.native_pre_overworld_tileset_publication =
            Some(self.advance_tileset_publication(publication, events));
    }

    pub(super) fn advance_pre_dungeon_tileset_publication(
        &mut self,
        events: &[TilesetGraphicsPublication],
    ) {
        if events.is_empty() { return; }
        let publication = self.native_pre_dungeon_tileset_publication.take();
        if publication.is_none() {
            self.complete_module_pre_dungeon_before_tilesets();
        }
        self.native_pre_dungeon_tileset_publication =
            Some(self.advance_tileset_publication(publication, events));
    }

    fn advance_tileset_publication(
        &mut self,
        publication: Option<NativeTilesetGraphicsPublication>,
        events: &[TilesetGraphicsPublication],
    ) -> NativeTilesetGraphicsPublication {
        let mut publication = publication
            .unwrap_or_else(|| self.native_tileset_decompression_operations());
        for event in events {
            match *event {
                TilesetGraphicsPublication::Entered => {}
                TilesetGraphicsPublication::SpriteSelection { slot, value } => {
                    let native = publication.sprite_sheets[slot] as u8;
                    assert_eq!(native, value, "source tileset selection disagrees with native theme");
                    self.sprite_workspace_mut().set_graphics_subset(slot, native);
                }
                TilesetGraphicsPublication::Sheet { index, entry_pc, sheet, completed_bytes } => {
                    let (native_entry, native_sheet, cursor) = &mut publication.operations[index];
                    assert_eq!(*native_entry, entry_pc);
                    assert_eq!(*native_sheet, usize::from(sheet), "source tileset sheet disagrees with native selection");
                    let end = usize::from(completed_bytes);
                    assert!(cursor.next_store <= end && end <= cursor.bytes.len());
                    self.copy_decompressed_graphics_to(
                        cursor.destination + cursor.next_store,
                        &cursor.bytes[cursor.next_store..end],
                    );
                    cursor.next_store = end;
                }
                TilesetGraphicsPublication::Returned => {
                    assert!(publication.operations.iter().all(|(_, _, cursor)| cursor.next_store == cursor.bytes.len()),
                        "tileset return preceded native decompression completion");
                }
            }
        }
        publication
    }

    /// Pair measured CPU addresses with translated graphics bytes. The CPU
    /// timing run never supplies a gameplay value to this plan.
    pub(super) fn build_native_map_graphics_write_plan(
        &self, timing: NativeMapGraphicsTiming,
    ) -> NativeMapGraphicsWritePlan {
        let NativeTilesetGraphicsPublication { sprite_sheets, mut operations } =
            self.native_tileset_decompression_operations();
        assert_eq!(timing.decompressions.len(), operations.len());
        for ((observed_pc, observed_sheet), (pc, sheet, _)) in
            timing.decompressions.iter().zip(&operations)
        {
            assert_eq!(*observed_pc, *pc, "map decompressor entry differs");
            assert_eq!(usize::from(*observed_sheet), *sheet, "map sheet choice differs");
        }
        assert_eq!(timing.writes_by_slice.len(), timing.quadrant_cursor_writes_by_slice.len());
        let mut writes_by_slice = Vec::with_capacity(timing.writes_by_slice.len());
        for slice in timing.writes_by_slice {
            let mut writes = Vec::with_capacity(slice.len());
            for (operation, address) in slice {
                if (0xc2fc..0xc300).contains(&address) {
                    let slot = address - 0xc2fc;
                    writes.push((address, sprite_sheets[slot] as u8));
                    continue;
                }
                let (_, _, cursor) = &mut operations[usize::from(operation)];
                let offset = cursor.next_store;
                assert!(offset < cursor.bytes.len(), "map decompression writes exceeded decoder output");
                assert_eq!(address, cursor.destination + offset,
                    "map timed store address differs from translated decoder");
                writes.push((address, cursor.bytes[offset]));
                cursor.next_store += 1;
            }
            writes_by_slice.push(writes);
        }
        for (_, sheet, cursor) in operations {
            assert_eq!(cursor.next_store, cursor.bytes.len(),
                "map sheet {sheet} timing omitted translated bytes");
        }
        NativeMapGraphicsWritePlan {
            writes_by_slice,
            quadrant_cursor_writes_by_slice: timing.quadrant_cursor_writes_by_slice,
            next_quadrant_write: 0,
            next_slice: 0,
        }
    }

    pub(super) fn advance_native_map_graphics_write_plan(&mut self) {
        let Some(mut plan) = self.native_map_graphics_write_plan.take() else { return };
        for (address, value) in &plan.writes_by_slice[plan.next_slice] {
            if (0xc2fc..0xc300).contains(address) {
                self.sprite_workspace_mut().set_graphics_subset(*address - 0xc2fc, *value);
            } else {
                self.ram[*address] = *value;
            }
        }
        for _ in 0..plan.quadrant_cursor_writes_by_slice[plan.next_slice] {
            if plan.next_quadrant_write == 0 {
                assert!(self.native_map_recovery_quadrants_started.is_none());
                self.clear_screen_transition();
                self.dungeon_room_load_mut().clear_quadrant_upload_index();
                self.native_map_recovery_quadrants_started = Some(0);
            } else {
                assert_eq!(self.native_map_recovery_quadrants_started,
                    Some(plan.next_quadrant_write - 1));
                self.water_flood_build_one_quadrant_for_vram();
                self.upload_tilemap_now();
                self.dungeon_prepare_next_room_quadrant_upload();
                self.upload_tilemap_now();
                self.native_map_recovery_quadrants_started = Some(plan.next_quadrant_write);
            }
            plan.next_quadrant_write += 1;
        }
        plan.next_slice += 1;
        if plan.next_slice < plan.writes_by_slice.len() {
            self.native_map_graphics_write_plan = Some(plan);
        }
    }

    pub(super) fn native_initialize_tilesets_work(&self) -> Option<crate::cycle_models::tilesets::TilesetWork> {
        use crate::cycle_models::tilesets::{Inputs, TilesetWork};
        let theme = &self.game_state.world.palette_theme;
        TilesetWork::initialize(Inputs {
            module: self.game_state.frame.main_module,
            misc: theme.misc_sprites_graphics_index(),
            main: load_gfx_main_tileset(theme.main_tile_theme_index() as usize),
            aux: load_gfx_aux_tileset(theme.aux_tile_theme_index() as usize),
            sprites: load_gfx_sprite_tileset(self.game_state.sprites.system.graphics_index() as usize),
            previous_sprites: std::array::from_fn(|i| self.game_state.sprites.workspace.graphics_subset(i)),
            main_index: theme.main_tile_theme_index(),
        }, |sprite, sheet| {
            if sprite { self.sprite_graphics_decompression_stream(sheet as usize) }
            else { self.asset_bytes(65, sheet as usize).map(Vec::from) }
        })
    }

    pub(super) fn initialize_tilesets(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_e19b);
        let Some(mut work) = self.native_initialize_tilesets_work() else { return };
        while let Some(instruction) = work.step(|address| self.read_wram_byte(address)) {
            let path = work.profile_call_path(&instruction);
            crate::cycle_ledger::with_profile_call_path(&path, || self.retire_tileset_instruction(instruction));
        }
        self.record_common_sprite_provenance();
        for slot in 0..4 {
            let pack = self.game_state.sprites.workspace.graphics_subset(slot);
            self.record_tileset_provenance(0x5000 + slot * 0x400, chr_source::CHR_KIND_SPRITE, pack);
        }
        let main = load_gfx_main_tileset(self.game_state.world.palette_theme.main_tile_theme_index() as usize);
        for slot in 0..8 {
            let pack = if (3..7).contains(&slot) { self.game_state.world.palette_theme.aux_bg_subset(slot - 3) } else { main[slot] };
            self.record_tileset_provenance(0x2000 + slot * 0x400, chr_source::CHR_KIND_BG, pack);
        }
    }

    pub(super) fn load_common_sprites(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_e6b7);
        let module = self.game_state.frame.main_module;
        let misc = self.game_state.world.palette_theme.misc_sprites_graphics_index();
        let Some(mut work) = crate::cycle_models::tilesets::TilesetWork::common(module, misc,
            |sheet| self.sprite_graphics_decompression_stream(sheet as usize)) else { return };
        while let Some(instruction) = work.step(|address| self.read_wram_byte(address)) {
            let path = work.profile_call_path(&instruction);
            crate::cycle_ledger::with_profile_call_path(&path, || self.retire_tileset_instruction(instruction));
        }
        self.record_common_sprite_provenance();
    }

    fn record_tileset_provenance(&mut self, destination: usize, kind: u8, pack: u8) {
        self.vram_chr_source.record_tiles(destination, 64, kind, u16::from(pack));
        self.vram_chr_preview_source.record_tiles(destination, 64, kind, u16::from(pack));
        self.tag_stream_content_hash(destination, 64);
    }

    fn record_common_sprite_provenance(&mut self) {
        self.record_tileset_provenance(0x4400, chr_source::CHR_KIND_SPRITE,
            self.game_state.world.palette_theme.misc_sprites_graphics_index());
        let first = if self.game_state.frame.main_module == 1 {94} else {6};
        for (destination, pack) in [(0x4800, first), (0x4c00, first + 1)] {
            self.record_tileset_provenance(destination, chr_source::CHR_KIND_SPRITE, pack);
        }
    }

    pub(super) fn retire_tileset_instruction(&mut self, instruction: crate::cycle_models::tilesets::Instruction) {
        use crate::cycle_models::tilesets::Publication;
        if let Some(Publication::Graphics { destination, publication }) = instruction.publication {
            self.retire_sheet_load_instruction(destination, crate::cycle_models::graphics_load::Instruction {
                master_cycles: instruction.master_cycles, routine: instruction.routine,
                publication: Some(publication), #[cfg(any(test, feature = "rom-shadow-audit"))] pc: instruction.pc,
            });
            if let crate::cycle_models::graphics_load::Publication::Conversion(
                crate::cycle_models::graphics_convert::Publication::VramWord { offset, .. }
            ) = publication {
                self.ppu.vram_pointer = (destination as u16).wrapping_add(offset).wrapping_add(1);
            }
            return;
        }
        #[cfg(feature = "rom-shadow-audit")]
        if crate::rom_shadow_audit::observing() {
            crate::rom_shadow_audit::model(vec![(instruction.pc, instruction.master_cycles,
                instruction.routine == 0xe843)]);
        }
        crate::cycle_ledger::charge_in_routine(instruction.routine, u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::PpuByte { register, value }) => self.ppu.write(register, value),
            Some(Publication::VramAddress(address)) => {
                self.ppu.write(0x16, address as u8);
                self.ppu.write(0x17, (address >> 8) as u8);
            }
            Some(Publication::SpriteSubset { slot, value }) => self.sprite_workspace_mut().set_graphics_subset(usize::from(slot), value),
            Some(Publication::BackgroundSubset { slot, value }) => self.world_palette_theme_mut().set_aux_bg_subset(usize::from(slot), value),
            Some(Publication::Graphics { .. }) => unreachable!(),
            None => {}
        }
    }

    pub(super) fn load_sprite_graphics(
        &mut self,
        dst: usize,
        gfx_pack: usize,
        decompression_buffer_offset: usize,
    ) {
        let _scope = crate::cycle_ledger::routine(0x00_e583);
        let Some(stream) = self.sprite_graphics_decompression_stream(gfx_pack) else { return };
        let mut work = crate::cycle_models::graphics_load::SheetWork::sprite(
            gfx_pack as u8, decompression_buffer_offset, &stream,
        );
        self.finish_sheet_load(dst, gfx_pack, chr_source::CHR_KIND_SPRITE, &mut work);
    }

    /// The first twelve sheets are contiguous raw 3bpp art. When a caller
    /// passes one to Decomp_spr, it is interpreted as a command stream and
    /// may read into the following raw sheet before reaching the terminator.
    /// Reassemble that native asset window instead of truncating at a tile
    /// sheet boundary or consulting the ROM at runtime.
    fn sprite_graphics_decompression_stream(&self, sheet: usize) -> Option<Vec<u8>> {
        let mut stream = self.asset_bytes(64, sheet)?.to_vec();
        if sheet < 12 {
            for next in sheet + 1..12 {
                stream.extend_from_slice(self.asset_bytes(64, next)?);
            }
        }
        Some(stream)
    }

    pub(super) fn load_background_graphics(
        &mut self,
        dst: usize,
        gfx_pack: usize,
        slot: usize,
        decompression_buffer_offset: usize,
    ) {
        let _scope = crate::cycle_ledger::routine(
            if decompression_buffer_offset == 0x14000 { 0x00_e609 } else { 0x00_e60d });
        let Some(stream) = self.asset_bytes(65, gfx_pack) else { return };
        let mut work = crate::cycle_models::graphics_load::SheetWork::background(
            gfx_pack as u8, decompression_buffer_offset, stream,
            self.game_state.world.palette_theme.main_tile_theme_index(), slot as u8,
        );
        self.finish_sheet_load(dst, gfx_pack, chr_source::CHR_KIND_BG, &mut work);
    }

    fn finish_sheet_load(
        &mut self, dst: usize, pack: usize, kind: u8,
        work: &mut crate::cycle_models::graphics_load::SheetWork,
    ) {
        self.vram_chr_source.record_tiles(dst, 64, kind, pack as u16);
        self.vram_chr_preview_source.record_tiles(dst, 64, kind, pack as u16);
        while let Some(instruction) = work.step(|address| self.read_wram_byte(address)) {
            let path = work.profile_call_path(instruction.routine);
            crate::cycle_ledger::with_profile_call_path(&path, || self.retire_sheet_load_instruction(dst, instruction));
        }
        self.tag_stream_content_hash(dst, 64);
    }

    pub(super) fn decompress_animated_dungeon_tiles(&mut self, pack: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_d337));
        if self.lag_free {
            // $00:D337..D393: caller instructions plus 256 four-way plane
            // rotations. E783 decompression and D5CE conversion are children.
            crate::cycle_ledger::charge(522 + 256 * 518 + 255 * 6 + 158);
        }
        if let Some(mut work) = self.native_pre_dungeon_animated_graphics.take() {
            self.advance_animated_dungeon_graphics(pack, &mut work, AnimatedDungeonGraphicsProgress::Returned);
            // Publication advances are unpriced; retain the ordinary caller's
            // aggregate charge without replaying its completed stores.
            for sheet in [pack, 0x5c] {
                let _decomp = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_e783));
                if self.lag_free { crate::cycle_ledger::charge(128); } // $00:E783..E78D.
                self.charge_sheet_decompression(DecompressEntry::BackgroundTables, 65, sheet,
                    GraphicsDecompressionScratch::primary_buffer_offset());
                drop(_decomp);
                let _scope = crate::cycle_ledger::routine(if self.lag_free { 0x00_d5ce } else { 0x00_dfb8 });
                crate::cycle_ledger::charge(32 + 48 * if self.lag_free { 3808 } else { 3736 } + 47 * 6 + 42);
            }
            return;
        }
        self.decompress_background_graphics_fixed_buffer(pack);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(0xa680, &tmp, 0, 48);

        self.decompress_background_graphics_fixed_buffer(0x5c);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(0xac80, &tmp, 0, 48);

        self.rotate_animated_dungeon_tile_planes();

        self.set_animated_tile_vram_destination_address(
            DUNGEON_ANIMATED_BG_VRAM_DESTINATION as u16,
        );
    }

    /// `Decomp_spr` (`$00:E772`). The ROM decompresses sheet `gfx` as
    /// given; the clamp below is the port's guard against the junk sheets
    /// 0..11, whose original streams the asset pack still carries (as the
    /// first 0x600 bytes at their pointers), so the cost is charged over the
    /// sheet the ROM would have read.
    pub(super) fn decompress_sprite_graphics_to_buffer(
        &mut self,
        dst: usize,
        gfx: usize,
    ) -> usize {
        self.decompress_sprite_graphics_at_entry(dst, gfx, 0x00_e772, 0)
    }

    fn decompress_sprite_graphics_fixed_buffer(&mut self, gfx: usize, secondary: bool) -> usize {
        // $00:E766..E770 sets $7F:4000 (128 clocks); $00:E75C..E770
        // sets $7F:4600 using its BRA (150 clocks). The decoder is shared.
        let dst = if secondary { GraphicsDecompressionScratch::secondary_buffer_offset() }
            else { GraphicsDecompressionScratch::primary_buffer_offset() };
        let routine = if self.lag_free { if secondary { 0x00_e75c } else { 0x00_e766 } } else { 0x00_e772 };
        self.decompress_sprite_graphics_at_entry(dst, gfx, routine,
            if self.lag_free { if secondary { 150 } else { 128 } } else { 0 })
    }

    fn decompress_sprite_graphics_at_entry(&mut self, dst: usize, mut gfx: usize, routine: u32, prefix: u64) -> usize {
        let _scope = crate::cycle_ledger::routine(routine);
        crate::cycle_ledger::charge(prefix);
        self.charge_sheet_decompression(DecompressEntry::Sprite, 64, gfx, dst);
        if gfx < 12 {
            gfx = 12;
        }
        let Some(data) = self.decompressed_sprite_graphics_data(gfx) else {
            return 0;
        };
        self.copy_decompressed_graphics_to(dst, &data)
    }

    /// `Decomp_bg` (`$00:E78F`): the table loads and the decompressor, the
    /// destination set by the caller.
    // Fixed-buffer callers use decompress_background_graphics_fixed_buffer
    // for $00:E783; this entry keeps only $00:E78F's table/decoder work.
    pub(super) fn decompress_background_graphics_to_buffer(
        &mut self,
        dst: usize,
        gfx: usize,
    ) -> usize {
        self.decompress_background_graphics_at_entry(dst, gfx, DecompressEntry::BackgroundTables)
    }

    fn decompress_background_graphics_fixed_buffer(&mut self, gfx: usize) -> usize {
        // $00:E783..E78D owns the 128-clock destination setup, then falls
        // through into E78F. Default/ROM-timing retain the prior entry.
        self.decompress_background_graphics_at_entry(GraphicsDecompressionScratch::primary_buffer_offset(), gfx,
            if self.lag_free { DecompressEntry::Background } else { DecompressEntry::BackgroundTables })
    }

    fn decompress_background_graphics_at_entry(&mut self, dst: usize, gfx: usize, entry: DecompressEntry) -> usize {
        let _scope = crate::cycle_ledger::routine(if entry == DecompressEntry::Background { 0x00_e783 } else { 0x00_e78f });
        self.charge_sheet_decompression(entry, 65, gfx, dst);
        let Some(data) = self.decompressed_background_graphics_data(gfx) else {
            return 0;
        };
        self.copy_decompressed_graphics_to(dst, &data)
    }

    /// Charge the ROM's decompression of `sheet` from `asset` (64 = sprite
    /// sheets, 65 = background sheets) to the WRAM offset `dst`. The asset
    /// pack holds every sheet as the cartridge's verbatim compressed stream
    /// (sheets 0..11 of asset 64 as their first 0x600 bytes; sprite sheet 7's
    /// junk stream is longer than that window, so its cost stops short),
    /// which `cycle_models::decompress` prices instruction by instruction.
    /// The entry routine's scope (open in the caller) takes its own
    /// instructions; every `Decompression_GetNextByte` call is recorded as
    /// its own `$00:E843` routine nested inside it, as the profiler frames
    /// it (one call per compressed byte, `$FF` included).
    fn charge_sheet_decompression(&self, entry: DecompressEntry, asset: usize, sheet: usize, dst: usize) {
        let Ok(sheet) = u8::try_from(sheet) else {
            return;
        };
        let Some(stream) = self.asset_bytes(asset, usize::from(sheet)) else {
            return;
        };
        let cost = decompress_cost(
            entry,
            sheet,
            stream,
            stream_source_offset(entry, sheet),
            0x7e_0000 + dst as u32,
        );
        #[cfg(feature = "rom-shadow-audit")]
        let mut children = if crate::rom_shadow_audit::observing() {
            let (witness_cost, _, steps) = crate::cycle_models::decompress::decompress_instruction_timeline(
                entry, sheet, stream, stream_source_offset(entry, sheet), 0x7e_0000 + dst as u32);
            assert_eq!(witness_cost, cost);
            let mut own = Vec::new();
            let mut children = Vec::new();
            let mut child = Vec::new();
            for step in steps {
                let row = (step.pc, step.master_cycles, step.in_get_next_byte);
                if step.in_get_next_byte {
                    child.push(row);
                    if step.pc == Some(0xe851) { children.push(std::mem::take(&mut child)); }
                } else { own.push(row); }
            }
            assert!(child.is_empty());
            assert_eq!(children.len() as u64, cost.get_next_byte_calls);
            crate::rom_shadow_audit::model(own);
            Some(children.into_iter())
        } else { None };
        crate::cycle_ledger::charge(cost.master - cost.get_next_byte_master);
        let Some(last_call) = cost.get_next_byte_calls.checked_sub(1) else {
            return;
        };
        // Every call but one costs GET_NEXT_BYTE_MASTER; the one that crosses
        // a bank end (if any) costs 56 more, booked on the last record here
        // since only the per-host total is compared.
        for _ in 0..last_call {
            #[cfg(feature = "rom-shadow-audit")]
            if let Some(children) = &mut children { crate::rom_shadow_audit::model(children.next().unwrap()); }
            crate::cycle_ledger::charge_routine(0x00_e843, GET_NEXT_BYTE_MASTER);
        }
        #[cfg(feature = "rom-shadow-audit")]
        if let Some(children) = &mut children { crate::rom_shadow_audit::model(children.next().unwrap()); }
        crate::cycle_ledger::charge_routine(
            0x00_e843,
            cost.get_next_byte_master - last_call * GET_NEXT_BYTE_MASTER,
        );
    }

    fn sprite_decompression_cursor(&self, sheet: usize, destination: usize) -> Option<GraphicsDecompressionCursor> {
        // The port's special handling of sheets 0..11 reads stale WRAM during
        // back-references. Those sheets need a live-buffer decoder cursor.
        if sheet < 12 {
            return None;
        }
        let sheet_number = u8::try_from(sheet).ok()?;
        let stream = self.asset_bytes(64, sheet)?;
        let bytes = self.decompressed_sprite_graphics_data(sheet)?;
        let (cost, stores) = decompress_store_timeline(
            DecompressEntry::Sprite,
            sheet_number,
            stream,
            stream_source_offset(DecompressEntry::Sprite, sheet_number),
            0x7e_0000 + destination as u32,
        );
        GraphicsDecompressionCursor::new(destination, bytes, stores, cost.master)
    }

    fn background_decompression_cursor(&self, sheet: usize, destination: usize) -> Option<GraphicsDecompressionCursor> {
        let sheet_number = u8::try_from(sheet).ok()?;
        let stream = self.asset_bytes(65, sheet)?;
        let bytes = self.decompressed_background_graphics_data(sheet)?;
        let (cost, stores) = decompress_store_timeline(
            DecompressEntry::BackgroundTables,
            sheet_number,
            stream,
            stream_source_offset(DecompressEntry::BackgroundTables, sheet_number),
            0x7e_0000 + destination as u32,
        );
        GraphicsDecompressionCursor::new(destination, bytes, stores, cost.master)
    }

    pub(super) fn decompressed_sprite_graphics_data(&self, gfx: usize) -> Option<Vec<u8>> {
        // Sheets 0..11 ship as their original compressed junk streams; the
        // VRAM load path handles them with WRAM-buffer semantics in
        // `load_sprite_graphics`. This plain resolver decompresses them with
        // zero-filled out-of-range copies (tool/preview quality only).
        let data = self.asset_bytes(64, gfx)?;
        if gfx < 103 && data.len() == 0x600 {
            Some(data.to_vec())
        } else {
            Some(decompress_asset(data))
        }
    }

    pub(super) fn decompressed_background_graphics_data(&self, gfx: usize) -> Option<Vec<u8>> {
        Some(decompress_asset(self.asset_bytes(65, gfx)?))
    }

    pub(super) fn expand3_to_4_high_from_slice(
        &mut self,
        mut dst: usize,
        data: &[u8],
        mut src: usize,
        base: usize,
        num: usize,
    ) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_d61c));
        if self.lag_free && num != 0 { crate::cycle_ledger::charge(32); } // $00:D61C STY.
        for tile in 0..num {
            if self.lag_free {
                // $00:D61E..D650, M16/X16: pointer/row setup and eight
                // three-plane/opaque-plane row copies, seven taken BPLs.
                crate::cycle_ledger::charge(126 + 8 * 700 + 7 * 6);
            }
            let src2 = src + 0x10;
            for i in 0..8 {
                let lo = data.get(src + i * 2).copied().unwrap_or(0);
                let hi = data.get(src + i * 2 + 1).copied().unwrap_or(0);
                let u = data.get(src2 + i).copied().unwrap_or(0);
                // Expand3To4High parks the third-plane byte in the $BD scratch
                // word (`LDA [$03] : AND #$00FF : STA $BD`, $00:D63B).
                crate::types::ww_check(MENU_PREV_JOYPAD_H, 2, "Expand3To4High scratch", u32::from(u));
                self.ram[MENU_PREV_JOYPAD_H] = u;
                self.ram[MENU_PREV_JOYPAD_H + 1] = 0;
                self.write_expanded_graphics_tile_row(dst, lo, hi, u, lo | hi | u);
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
            if self.lag_free {
                // $00:D652..D66B: destination/pointer advance, actual
                // source-row wrap at D65D, and outer DEC/BNE.
                crate::cycle_ledger::charge(138
                    + if (src.wrapping_sub(base) & 0x78) == 0 { 102 } else { 6 }
                    + 102 + if tile + 1 < num { 6 } else { 0 });
            }
            if (src.wrapping_sub(base) & 0x78) == 0 {
                src += 0x180;
            }
        }
        if self.lag_free && num != 0 { crate::cycle_ledger::charge(42); } // $00:D66D RTS.
    }

    /// `Do3To4High16Bit` (`$00:DF4F`, m16 x16). Per tile the ROM runs
    /// `$DF51-$DF5A` (126), the two-row block `$DF5C-$DFA9` four times
    /// (`LDY #$0003`; 1,370 each, `BPL` taken three times) and
    /// `$DFAB-$DFB5` (168): 126 + 4 x 1,370 + 3 x 6 + 168 = 5,792, plus the
    /// taken `BNE` back to `$DF51` for every tile but the last. The ROM's
    /// `DEC $0C : BNE` loop assumes `num >= 1`, as every caller passes.
    #[track_caller]
    pub(super) fn do3_to_4_high_16bit_from_slice(
        &mut self,
        mut dst: usize,
        data: &[u8],
        mut src: usize,
        num: usize,
    ) {
        let _scope = crate::cycle_ledger::routine(0x00_df4f);
        // $DF4F: STY $0C.
        crate::cycle_ledger::charge(32);
        for tile in 0..num {
            // $DF51-$DFB5 for one tile (see above).
            crate::cycle_ledger::charge(5_792);
            if tile + 1 < num {
                // $DFB5: BNE $DF51 taken.
                crate::cycle_ledger::charge(6);
            }
            let src2 = src + 0x10;
            for i in 0..8 {
                let lo = data.get(src + i * 2).copied().unwrap_or(0);
                let hi = data.get(src + i * 2 + 1).copied().unwrap_or(0);
                let u = data.get(src2 + i).copied().unwrap_or(0);
                self.write_expanded_graphics_tile_row(dst, lo, hi, u, lo | hi | u);
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
        }
        // $DFB7: RTS.
        crate::cycle_ledger::charge(42);
    }

    /// `Do3To4Low16Bit` (`$00:DFB8`, m16 x16). Per tile the ROM runs
    /// `$DFBA-$DFC4` (126), the four-row block `$DFC5-$E022` twice
    /// (`LDY #$0001`; 1,718 each, `BPL` taken once) and `$E024-$E02E` (168):
    /// 126 + 2 x 1,718 + 6 + 168 = 3,736, plus the taken `BNE` back to
    /// `$DFBA` for every tile but the last. The ROM's `DEC $0C : BNE` loop
    /// assumes `num >= 1`, as every caller passes.
    pub(super) fn do3_to_4_low_16bit_from_slice(
        &mut self, dst: usize, data: &[u8], src: usize, num: usize,
    ) {
        self.convert_low_buffer(dst, data, src, num, false);
    }

    fn do3_to_4_low_animated_from_slice(
        &mut self, dst: usize, data: &[u8], src: usize, num: usize,
    ) {
        self.convert_low_buffer(dst, data, src, num, true);
    }

    fn convert_low_buffer(
        &mut self,
        mut dst: usize,
        data: &[u8],
        mut src: usize,
        num: usize,
        animated: bool,
    ) {
        let animated = animated && self.lag_free;
        let _scope = crate::cycle_ledger::routine(if animated { 0x00_d5ce } else { 0x00_dfb8 });
        let b = crate::cycle_ledger::blocks(self.lag_free);
        // $DFB8: STY $0C.
        if self.lag_free { b.block(if animated { 0x00d5ce } else { 0x00dfb8 }); }
        else { crate::cycle_ledger::charge(32); }
        for tile in 0..num {
            // $DFBA-$E02E for one tile (see above).
            // $00:D5D0..D616: 126 + 4*874 + 3*6 + 168 = 3808.
            // Animated low conversion increments its DP source pointers;
            // $00:DFBA..E02E instead uses the 3736-clock four-row variant.
            if self.lag_free {
                b.block(if animated { 0x00d5d0 } else { 0x00dfba });
                for row in 0..if animated { 4 } else { 2 } {
                    b.block(if animated { 0x00d5db } else { 0x00dfc5 });
                    if row + 1 < if animated { 4 } else { 2 } {
                        b.taken(if animated { 0x00d60a } else { 0x00e022 });
                    }
                }
                b.block(if animated { 0x00d60c } else { 0x00e024 });
            } else { crate::cycle_ledger::charge(if animated { 3_808 } else { 3_736 }); }
            if tile + 1 < num {
                // $E02E: BNE $DFBA taken.
                if self.lag_free { b.taken(if animated { 0x00d616 } else { 0x00e02e }); }
                else { crate::cycle_ledger::charge(6); }
            }
            let src2 = src + 0x10;
            for i in 0..8 {
                self.write_expanded_graphics_tile_row(
                    dst,
                    data.get(src + i * 2).copied().unwrap_or(0),
                    data.get(src + i * 2 + 1).copied().unwrap_or(0),
                    data.get(src2 + i).copied().unwrap_or(0),
                    0,
                );
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
        }
        // $E030 / $D618: RTS.
        if self.lag_free { b.block(if animated { 0x00d618 } else { 0x00e030 }); }
        else { crate::cycle_ledger::charge(42); }
    }

    pub(super) fn do3_to_4_high_to_vram(
        &mut self, dst: usize, data: &[u8], chr_kind: u8, chr_pack: u16,
    ) {
        self.convert_3_to_4_to_vram(dst, data, chr_kind, chr_pack, true);
    }

    pub(super) fn do3_to_4_low_to_vram(
        &mut self, dst: usize, data: &[u8], chr_kind: u8, chr_pack: u16,
    ) {
        self.convert_3_to_4_to_vram(dst, data, chr_kind, chr_pack, false);
    }

    fn convert_3_to_4_to_vram(
        &mut self, dst: usize, data: &[u8], chr_kind: u8, chr_pack: u16, high: bool,
    ) {
        let _scope = crate::cycle_ledger::routine(if high {0x00_e5af} else {0x00_e63c});
        self.vram_chr_source.record_tiles(dst, 64, chr_kind, chr_pack);
        self.vram_chr_preview_source.record_tiles(dst, 64, chr_kind, chr_pack);
        let mut work = crate::cycle_models::graphics_convert::ConversionWork::new(high, data, 64);
        while let Some(instruction) = work.step() {
            self.retire_graphics_conversion_instruction(dst, instruction);
        }
        // Keep raw pack provenance for authoring; rendered BG/sprite sheets
        // use content identities because their pack numbers are non-injective.
        if matches!(chr_kind, chr_source::CHR_KIND_BG | chr_source::CHR_KIND_SPRITE) {
            self.tag_stream_content_hash(dst, 64);
        }
    }

    fn retire_graphics_conversion_instruction(
        &mut self, dst: usize, instruction: crate::cycle_models::graphics_convert::Instruction,
    ) {
        use crate::cycle_models::graphics_convert::Publication;
        #[cfg(feature = "rom-shadow-audit")]
        if crate::rom_shadow_audit::observing() {
            crate::rom_shadow_audit::model(vec![(instruction.audit_pc(), instruction.master_cycles, false)]);
        }
        crate::cycle_ledger::charge(u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::VramWord { offset, value }) => self.ppu.vram[dst + usize::from(offset)] = value,
            Some(Publication::LineWord { row, value }) => self.set_dungeon_line_pointer_row0(usize::from(row), value),
            Some(Publication::ScratchWord(value)) => {
                crate::types::ww_check(MENU_PREV_JOYPAD_H, 2, "graphics conversion scratch", u32::from(value));
                self.ram[MENU_PREV_JOYPAD_H..MENU_PREV_JOYPAD_H + 2].copy_from_slice(&value.to_le_bytes());
            }
            // This slice API has no physical source address. The enclosing
            // loader owns its pointer; the work exposes every increment for
            // that caller without inventing an address for a native slice.
            Some(Publication::SourceOffset(_)) | None => {}
        }
    }

    fn retire_sheet_load_instruction(
        &mut self, dst: usize, instruction: crate::cycle_models::graphics_load::Instruction,
    ) {
        use crate::cycle_models::graphics_load::Publication;
        use crate::cycle_models::graphics_convert::Publication as Conversion;
        #[cfg(feature = "rom-shadow-audit")]
        if crate::rom_shadow_audit::observing() {
            crate::rom_shadow_audit::model(vec![(instruction.pc, instruction.master_cycles,
                instruction.routine == 0xe843)]);
        }
        crate::cycle_ledger::charge_in_routine(instruction.routine, u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::Byte { address, value }) => self.ram[address] = value,
            Some(Publication::SourcePointer(value)) => self.ram[..2].copy_from_slice(&value.to_le_bytes()),
            Some(Publication::Conversion(Conversion::VramWord { offset, value })) =>
                self.ppu.vram[dst + usize::from(offset)] = value,
            Some(Publication::Conversion(Conversion::LineWord { row, value })) =>
                self.set_dungeon_line_pointer_row0(usize::from(row), value),
            Some(Publication::Conversion(Conversion::ScratchWord(value))) => {
                crate::types::ww_check(MENU_PREV_JOYPAD_H, 2, "sheet conversion scratch", u32::from(value));
                self.ram[MENU_PREV_JOYPAD_H..MENU_PREV_JOYPAD_H + 2].copy_from_slice(&value.to_le_bytes());
            }
            Some(Publication::Conversion(Conversion::SourceOffset(_))) => unreachable!("sheet owns its source pointer"),
            None => {}
        }
    }

    /// ROM: `Graphics_LoadChrHalfSlot`
    pub(super) fn graphics_load_chr_half_slot(&mut self) {
        if self.try_native_intro_half_slot() { return; }
        let _scope = crate::cycle_ledger::routine(0x00_e3fa);
        crate::cycle_ledger::charge(48);
        let k = self.game_state.display.chr_halfslot_request;
        if k == 0 {
            crate::cycle_ledger::charge(6 + 44);
            return;
        }

        self.graphics_load_chr_half_slot_after_request(k);
    }

    /// Resume at E3FF after the native request load and branch.
    pub(super) fn graphics_load_chr_half_slot_after_request(&mut self, k:u8) {
        assert_ne!(k,0);
        let _scope = crate::cycle_ledger::routine(0x00_e3fa);
        crate::cycle_ledger::charge(120); // Bank setup, SP6 lookup and BMI.
        let sp6 = GRAPHICS_LOAD_SP6[k as usize - 1];
        if sp6 >= 0 {
            crate::cycle_ledger::charge(64);
            self.set_sp6r_indoors(sp6 as u8);
            self.select_overworld_aux_palette_offset();
            if k == 1 {
                crate::cycle_ledger::charge(158);
                self.set_sp6r_indoors(10);
                self.palette_load_sprite_environment_with_cycle_scope();
                crate::cycle_ledger::charge(60); // INC CGRAM flag : BRA.
            } else {
                crate::cycle_ledger::charge(6 + 110);
                self.palette_load_sprite_environment_dungeon_with_cycle_scope();
                crate::cycle_ledger::charge(38);
            }
            self.increment_cgram_update_flag();
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(204);
        let mut tilebytes = 0x44;
        self.increment_chr_halfslot_request();
        if self.game_state.display.chr_halfslot_request & 1 != 0 {
            crate::cycle_ledger::charge(64);
            self.clear_chr_halfslot_request();
            if k != 18 {
                crate::cycle_ledger::charge(88);
                tilebytes = 0x46;
                if k == 2 {
                    crate::cycle_ledger::charge(32);
                    self.clear_custom_spell_animation();
                } else {
                    crate::cycle_ledger::charge(6);
                }
            } else {
                crate::cycle_ledger::charge(6);
            }
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(136);
        self.set_nmi_load_target_page(tilebytes);
        self.set_pending_nmi_subroutine(11);

        let mut pack = GRAPHICS_HALF_SLOT_PACKS[k as usize - 1] as usize;
        if pack == 1 {
            crate::cycle_ledger::charge(32);
            pack = self
                .game_state
                .world
                .palette_theme
                .misc_sprites_graphics_index() as usize;
        } else {
            crate::cycle_ledger::charge(6);
        }
        let bank_offset = if tilebytes == 0x46 { 0x300 } else { 0 };
        self.load_chr_half_slot_pack(pack, bank_offset);
    }

    pub(super) fn load_chr_half_slot_pack(&mut self, pack: usize, bank_offset: usize) {
        let Some(data) = self.asset_bytes(64, pack) else {
            return;
        };
        let data = data.to_vec();

        let Some(&source_low) = SPRITE_SOURCE_LOW_WORDS.get(pack) else { return };
        let table_page_crossings = [0xf3, 0xd2, 0xb1].into_iter()
            .filter(|low| low + pack >= 0x100).count();
        crate::cycle_ledger::charge(358 + 6 * table_page_crossings as u64);
        let mut plane01 = source_low.wrapping_add(bank_offset as u16);

        let mut dst = NMI_BG_CHAR_HALF_BUFFER;
        let mut src = bank_offset;
        for tile in 0..32 {
            crate::cycle_ledger::charge(86);
            let mut plane2 = plane01.wrapping_add(16);
            if plane2 == 0 {
                crate::cycle_ledger::charge(78);
                plane2 = 0x8000;
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(56);
            let mut sprite_data = [0u8; 24];
            for byte in &mut sprite_data {
                let Some(value) = data.get(src).copied() else {
                    return;
                };
                *byte = value;
                src += 1;
            }

            for i in 0..8 {
                crate::cycle_ledger::charge(360);
                plane01 = plane01.wrapping_add(2);
                if plane01 == 0 {
                    crate::cycle_ledger::charge(196);
                    plane01 = plane2.wrapping_add(1);
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(314);
                plane2 = plane2.wrapping_add(1);
                if plane2 == 0 {
                    crate::cycle_ledger::charge(220);
                    plane01 = 0x8000;
                    plane2 = 0x8010;
                } else {
                    crate::cycle_ledger::charge(6);
                }
                let lo = sprite_data[i * 2];
                let hi = sprite_data[i * 2 + 1];
                let mask = sprite_data[16 + i];
                self.write_expanded_graphics_tile_row(dst, lo, hi, mask, lo | hi | mask);
                dst += 2;
                crate::cycle_ledger::charge(58 + if i != 7 { 6 } else { 0 });
            }
            dst += 16;
            crate::cycle_ledger::charge(168 + if tile != 31 { 6 } else { 0 });
            plane01 = plane2;
        }
        crate::cycle_ledger::charge(94); // SEP : PLB : RTL.
    }

    pub(super) fn snes_divide(&self, dividend: u16, divisor: u8) -> u16 {
        if divisor == 0 {
            0xffff
        } else {
            dividend / divisor as u16
        }
    }

    pub(super) fn RecoverPegGFXFromMapping(&mut self) {
        if self.lag_free {
            // $02:9761: index setup/barrier test; optional index swap.
            crate::cycle_ledger::charge(126 + 0
                + if self.game_state.dungeon.environment.orange_blue_barrier_state() == 0 { 6 } else { 38 });
        }
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.dungeon_update_peg_gfx_buffer(0x180, 0);
        } else {
            self.dungeon_update_peg_gfx_buffer(0, 0x180);
        }
    }

    /// ROM: `EraseTileMaps_dungeonmap`
    pub(super) fn erase_tile_maps_dungeonmap(&mut self) {
        self.erase_tile_maps(crate::cycle_models::tilemap_erase::Pattern::DungeonMap);
    }

    pub(super) fn EnableForceBlank(&mut self) {
        self.enable_force_blank();
    }

    pub(super) fn enable_force_blank_during_active_scanout(&mut self, scanline: u8) {
        self.enable_force_blank();
        self.active_display_force_blank_event = Some(scanline);
    }

    pub(super) fn DecompressSwordGraphics(&mut self) {
        self.decompress_sword_graphics();
    }

    pub(super) fn DecompressShieldGraphics(&mut self) {
        self.decompress_shield_graphics();
    }

    pub(super) fn DecompressAnimatedOverworldTiles(&mut self, a: u8) {
        self.decompress_background_graphics_fixed_buffer(a as usize);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(0xa680, &tmp, 0, 64);

        self.decompress_background_graphics_fixed_buffer(a.wrapping_add(1) as usize);
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_animated_from_slice(0xae80, &tmp, 0, 32);
        self.set_animated_tile_vram_destination_address(
            OVERWORLD_ANIMATED_BG_VRAM_DESTINATION as u16,
        );
        // Remember the pack filling the animated-tile buffer so the per-frame
        // animated-tile DMA can tag its VRAM slots injectively (CHR_KIND_BG_ANIM).
        self.animated_tile_pack = a as u16;
    }

    fn publish_animated_graphics_plane_stores(&mut self, base: usize, start: u16, end: u16) {
        let source = self.graphics_primary_decompression_buffer(0x600);
        for store in start..end {
            let row = usize::from(store / 2);
            let tile = row / 8;
            let y = row % 8;
            let dst = base + tile * 32 + y * 2;
            let src = tile * 24;
            if store & 1 == 0 {
                self.ram[dst..dst + 2].copy_from_slice(&source[src + y * 2..src + y * 2 + 2]);
            } else {
                self.ram[dst + 16] = source[src + 16 + y];
                self.ram[dst + 17] = 0;
            }
        }
    }

    pub(super) fn advance_animated_dungeon_graphics(
        &mut self,
        pack: usize,
        work: &mut AnimatedDungeonGraphicsContinuation,
        target: AnimatedDungeonGraphicsProgress,
    ) {
        use AnimatedDungeonGraphicsProgress::*;
        assert!(work.progress <= target, "animated dungeon graphics moved backward");
        while work.progress != target {
            match work.progress {
                FirstSheet(start) | SecondSheet(start) => {
                    let first = matches!(work.progress, FirstSheet(_));
                    let end = match (first, target) {
                        (true, FirstSheet(end)) | (false, SecondSheet(end)) => end,
                        _ => 1536,
                    };
                    assert!(start <= end && end <= 1536);
                    let bytes = self.decompressed_background_graphics_data(if first { pack } else { 0x5c })
                        .expect("animated dungeon graphics require native decoded sheets");
                    assert_eq!(bytes.len(), 1536);
                    self.copy_decompressed_graphics_to(0x14000 + usize::from(start),
                        &bytes[usize::from(start)..usize::from(end)]);
                    work.progress = if first { FirstSheet(end) } else { SecondSheet(end) };
                    if work.progress != target {
                        work.progress = if first { FirstConversion(0) } else { SecondConversion(0) };
                    }
                }
                FirstConversion(start) | SecondConversion(start) => {
                    let first = matches!(work.progress, FirstConversion(_));
                    let end = match (first, target) {
                        (true, FirstConversion(end)) | (false, SecondConversion(end)) => end,
                        _ => 768,
                    };
                    assert!(start <= end && end <= 768);
                    self.publish_animated_graphics_plane_stores(if first { 0xa680 } else { 0xac80 }, start, end);
                    work.progress = if first { FirstConversion(end) } else { SecondConversion(end) };
                    if work.progress != target {
                        work.progress = if first { SecondSheet(0) } else { Rotation(0) };
                    }
                }
                Rotation(start) => {
                    let end = if let Rotation(end) = target { end } else { 1024 };
                    assert!(start <= end && end <= 1024);
                    const ROTATION_BLOCKS: [usize; 4] = [0xa880, 0xac80, 0xae80, 0xaa80];
                    for store in start..end {
                        let offset = usize::from(store / 4) * 2;
                        let (dst, value) = match store % 4 {
                            0 => {
                                work.rotation_saved_word = read_le_u16(&self.ram, ROTATION_BLOCKS[0] + offset);
                                (0xa880, read_le_u16(&self.ram, ROTATION_BLOCKS[1] + offset))
                            }
                            1 => (0xac80, read_le_u16(&self.ram, ROTATION_BLOCKS[2] + offset)),
                            2 => (0xae80, read_le_u16(&self.ram, ROTATION_BLOCKS[3] + offset)),
                            _ => (0xaa80, work.rotation_saved_word),
                        };
                        write_le_u16(&mut self.ram, dst + offset, value);
                    }
                    work.progress = Rotation(end);
                    if work.progress != target {
                        self.set_animated_tile_vram_destination_address(DUNGEON_ANIMATED_BG_VRAM_DESTINATION as u16);
                        work.progress = DestinationPublished;
                    }
                }
                DestinationPublished => work.progress = Returned,
                Returned => unreachable!("animated dungeon graphics advanced past return"),
            }
        }
    }

    pub(super) fn advance_animated_overworld_graphics(
        &mut self,
        sheet: u8,
        progress: &mut AnimatedOverworldGraphicsProgress,
        target: AnimatedOverworldGraphicsProgress,
    ) {
        use AnimatedOverworldGraphicsProgress::*;
        assert!(*progress <= target, "animated overworld graphics moved backward");
        while *progress != target {
            match *progress {
                FirstSheet(start) | SecondSheet(start) => {
                    let first = matches!(*progress, FirstSheet(_));
                    let end = match (first, target) {
                        (true, FirstSheet(end)) | (false, SecondSheet(end)) => end,
                        _ => 1536,
                    };
                    assert!(start <= end && end <= 1536);
                    let data = self.decompressed_background_graphics_data(
                        usize::from(sheet.wrapping_add(u8::from(!first))),
                    ).expect("animated overworld graphics require their native sheet");
                    assert_eq!(data.len(), 1536);
                    self.copy_decompressed_graphics_to(
                        GraphicsDecompressionScratch::primary_buffer_offset() + usize::from(start),
                        &data[usize::from(start)..usize::from(end)],
                    );
                    *progress = if first { FirstSheet(end) } else { SecondSheet(end) };
                    if *progress != target {
                        *progress = if first { FirstConversion(0) } else { SecondConversion(0) };
                    }
                }
                FirstConversion(start) | SecondConversion(start) => {
                    let first = matches!(*progress, FirstConversion(_));
                    let total = if first { 1024 } else { 512 };
                    let end = match (first, target) {
                        (true, FirstConversion(end)) | (false, SecondConversion(end)) => end,
                        _ => total,
                    };
                    assert!(start <= end && end <= total);
                    let base = if first { 0xa680 } else { 0xae80 };
                    self.publish_animated_graphics_plane_stores(base, start, end);
                    *progress = if first { FirstConversion(end) } else { SecondConversion(end) };
                    if *progress != target {
                        *progress = if first { SecondSheet(0) } else {
                            self.set_animated_tile_vram_destination_address(
                                OVERWORLD_ANIMATED_BG_VRAM_DESTINATION as u16,
                            );
                            self.animated_tile_pack = u16::from(sheet);
                            Returned
                        };
                    }
                }
                Returned => unreachable!("animated graphics cannot advance past return"),
            }
        }
    }

    pub(super) fn complete_pre_overworld_animated_graphics(&mut self, sheet: u8) {
        let Some(mut progress) = self.native_pre_overworld_animated_graphics.take() else {
            self.DecompressAnimatedOverworldTiles(sheet);
            return;
        };
        self.advance_animated_overworld_graphics(sheet, &mut progress, AnimatedOverworldGraphicsProgress::Returned);
        // Prefix publication is unpriced, like the other semantic progress
        // receipts. Preserve this aggregate caller's original ledger cost
        // when it retires, without replaying its completed memory stores.
        for (pack, tiles) in [(sheet, 64u64), (sheet.wrapping_add(1), 32)] {
            self.charge_sheet_decompression(
                DecompressEntry::BackgroundTables, 65, usize::from(pack),
                GraphicsDecompressionScratch::primary_buffer_offset(),
            );
            let _scope = crate::cycle_ledger::routine(0x00_dfb8);
            crate::cycle_ledger::charge(32 + tiles * 3736 + (tiles - 1) * 6 + 42);
        }
    }

    pub(super) fn LoadFollowerGraphics(&mut self) {
        self.load_follower_graphics();
    }

    /// ROM: `WriteTo4BPPBuffer_at_7F4000`
    pub(super) fn write_to4_bpp_buffer_at_7f4000(&mut self, a: u8) {
        let src = ANIMATED_SPRITE_TILE_SOURCE_OFFSETS
            .get(a as usize)
            .copied()
            .unwrap_or(0);
        let tmp = self.graphics_combined_decompression_buffers();
        self.expand3_to_4_high_from_slice(0xbd40, &tmp, src, 0, 2);
        self.expand3_to_4_high_from_slice(0xbd80, &tmp, src + 0x180, 0, 2);
    }

    /// ROM: `DecodeAnimatedSpriteTile_variable`
    pub(super) fn decode_animated_sprite_tile_variable(&mut self, a: u8) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00d4ed));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x00d4ed);
        if a == 0x23 { b.taken(0x00d4f5); }
        else {
            b.block(0x00d4f7);
            if a >= 0x37 { b.taken(0x00d4f9); }
            else {
                b.block(0x00d4fb);
                if a == 0x0c { b.taken(0x00d4ff); }
                else {
                    b.block(0x00d501);
                    if a >= 0x24 { b.taken(0x00d503); }
                    else { b.block(0x00d505); }
                }
            }
        }
        self.replay_trace_ram_watch("loadgfx-before-decode-animated-sprite-tile");
        let secondary_sheet = animated_sprite_tile_secondary_sheet(a);
        b.block(0x00d507);
        self.decompress_sprite_graphics_fixed_buffer(secondary_sheet.into(), true);
        b.block(0x00d50a);
        self.decompress_sprite_graphics_fixed_buffer(0x5a, false);
        if self.lag_free {
            b.block(0x00d50f);
            if (0xd469u16 & 255) + u16::from(a.wrapping_mul(2)) > 255 { b.crossing(0x00d519); }
            // D4ED inlines D4DB's conversions; price its caller instructions
            // on the actual sides of those child calls.
            let src = ANIMATED_SPRITE_TILE_SOURCE_OFFSETS.get(a as usize).copied().unwrap_or(0);
            let tmp = self.graphics_combined_decompression_buffers();
            self.expand3_to_4_high_from_slice(0xbd40, &tmp, src, 0, 2);
            b.block(0x00d528);
            self.expand3_to_4_high_from_slice(0xbd80, &tmp, src + 0x180, 0, 2);
            b.block(0x00d533);
        } else { self.write_to4_bpp_buffer_at_7f4000(a); }
        self.replay_trace_ram_watch("loadgfx-after-decode-animated-sprite-tile");
    }

    /// Commit a source-order prefix of the shared two-sheet animated sprite
    /// loader. The exact CPU supplies only the output cursor; the bytes come
    /// from this runtime's independently decoded graphics asset.
    pub(super) fn apply_animated_sprite_tile_sheet_slice(
        &mut self,
        tile: u8,
        first_sheet: bool,
        start: u16,
        end: u16,
    ) {
        assert!(start <= end && end <= 0x0600);
        let sheet = if first_sheet {
            animated_sprite_tile_secondary_sheet(tile)
        } else {
            0x5a
        };
        let data = self
            .decompressed_sprite_graphics_data(usize::from(sheet))
            .expect("animated sprite loader requires its own decoded sheet");
        assert_eq!(data.len(), 0x0600);
        let base = if first_sheet {
            GraphicsDecompressionScratch::secondary_buffer_offset()
        } else {
            GraphicsDecompressionScratch::primary_buffer_offset()
        };
        self.copy_decompressed_graphics_to(
            base + usize::from(start),
            &data[usize::from(start)..usize::from(end)],
        );
    }

    /// Publish only the completed sheets and the current sheet's written
    /// prefix. The call's counter-reset suffix remains with its caller.
    pub(super) fn advance_transition_aux_graphics(
        &mut self,
        completed: Option<TransitionAuxGraphicsProgress>,
        target: TransitionAuxGraphicsProgress,
    ) {
        assert!(target.sheet < 8 && target.completed_bytes <= 0x600);
        let first = completed.map_or(0, |progress| {
            assert!((progress.sheet, progress.completed_bytes)
                <= (target.sheet, target.completed_bytes), "aux graphics cursor moved backward");
            progress.sheet
        });
        let backgrounds = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize);
        let sprites = load_gfx_sprite_tileset(
            self.game_state.sprites.system.graphics_index() as usize);
        for sheet in first..=target.sheet {
            let start = completed.filter(|progress| progress.sheet == sheet)
                .map_or(0, |progress| progress.completed_bytes);
            let end = if sheet == target.sheet { target.completed_bytes } else { 0x600 };
            if start == end { continue; }
            let data = if sheet < 4 {
                let pack = backgrounds[usize::from(sheet)];
                if pack == 0 { continue; }
                if start == 0 { self.set_aux_bg_subset_pack(usize::from(sheet), pack); }
                self.decompressed_background_graphics_data(usize::from(pack))
            } else {
                let slot = usize::from(sheet - 4);
                let pack = sprites[slot];
                if start == 0 && pack != 0 { self.set_sprite_gfx_subset(slot, pack); }
                self.decompressed_sprite_graphics_data(usize::from(self.sprite_gfx_subset(slot)).max(12))
            }.expect("transition graphics require native decoded assets");
            assert_eq!(data.len(), 0x600);
            self.copy_decompressed_graphics_to(
                0x6000 + usize::from(sheet) * 0x600 + usize::from(start),
                &data[usize::from(start)..usize::from(end)],
            );
        }
    }

    /// `LoadTransAuxGFX` (`$00:D66E`, entered m8; `REP #$30` at `$D67B`).
    pub(super) fn LoadTransAuxGFX(&mut self) {
        if let Some(progress) = self.native_transition_aux_graphics_progress.take() {
            self.advance_transition_aux_graphics(Some(progress), TransitionAuxGraphicsProgress {
                sheet: 7, completed_bytes: 0x600,
            });
            self.reset_incremental_vram_upload_counter();
            return;
        }
        let _scope = crate::cycle_ledger::routine(0x00_d66e);
        let p = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize
        );
        for (i, pack) in p.iter().copied().enumerate() {
            // i = 0: $D66E-$D68E (PHB : PHK : PLB, destination $7E:6000 in
            // $00-$02, theme index x4 in $0E, LDA kAuxTilesets,X : BEQ).
            // i = 1..3: $D69A-$D6AA / $D6B6-$D6C6 / $D6D2-$D6E2 (SEP #$10,
            // $01 += 6, REP #$10, next kAuxTilesets byte : BEQ).
            crate::cycle_ledger::charge(if i == 0 { 430 } else { 208 });
            if pack != 0 {
                // $D690-$D697 / $D6AC-$D6B3 / $D6C8-$D6CF / $D6E4-$D6EB:
                // STA long aux_bg_subset_i : SEP #$10 : TAY : JSR Decomp_bg
                // (the JSR only; Decomp_bg charges itself).
                crate::cycle_ledger::charge(122);
                self.set_aux_bg_subset_pack(i, pack);
                assert_eq!(
                    self.decompress_background_graphics_to_buffer(
                        0x6000 + 0x600 * i,
                        pack as usize
                    ),
                    0x600
                );
            } else {
                // BEQ taken past the sheet.
                crate::cycle_ledger::charge(6);
            }
        }
        // $D6EE-$D6F7: SEP #$10, $01 += 6, BRA Gfx_LoadSpritesInner.
        crate::cycle_ledger::charge(122);
        self.gfx_load_sprites_inner(0x7800);
    }

    pub(super) fn overworld_aux_graphics_workload(&self) -> OverworldAuxGraphicsWorkload {
        let packs = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        );
        OverworldAuxGraphicsWorkload {
            background_packs_to_decompress: packs.into_iter().filter(|&pack| pack != 0).count(),
        }
    }

    /// `LoadTransAuxGFX_sprite` (`$00:D6F9`, entered m8): the prologue,
    /// then it runs straight into `Gfx_LoadSpritesInner`.
    /// ROM: `LoadTransAuxGFX_sprite`
    pub(super) fn load_trans_aux_gfx_sprite(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_d6f9);
        // $D6F9-$D705: PHB : PHK : PLB, destination $7E:7800 in $00-$02.
        crate::cycle_ledger::charge(176);
        self.gfx_load_sprites_inner(0x7800);
    }

    /// `Gfx_LoadSpritesInner` (`$00:D706`) is the shared tail of
    /// `LoadTransAuxGFX` and `LoadTransAuxGFX_sprite` (reached by `BRA` and
    /// by falling through, never called), so it has no routine scope of its
    /// own: its blocks charge to the caller's.
    /// ROM: `Gfx_LoadSpritesInner`
    pub(super) fn gfx_load_sprites_inner(&mut self, dst: usize) {
        let p = load_gfx_sprite_tileset(self.game_state.sprites.system.graphics_index() as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            // i = 0: $D706-$D719 (REP #$30, graphics index x4 in $0E,
            // LDA kSpriteTilesets,X : BEQ).
            // i = 1..3: $D71F-$D737 / $D73D-$D755 / $D75B-$D773 (SEP #$10,
            // LDA long sprite_gfx_subset_{i-1} : TAY : JSR Decomp_spr (the
            // JSR only; Decomp_spr charges itself), $01 += 6, REP #$10, next
            // kSpriteTilesets byte : BEQ).
            crate::cycle_ledger::charge(if i == 0 { 254 } else { 308 });
            if pack != 0 {
                // $D71B / $D739 / $D757 / $D775: STA long sprite_gfx_subset_i.
                crate::cycle_ledger::charge(40);
                self.set_sprite_gfx_subset(i, pack);
            } else {
                // BEQ taken past the store.
                crate::cycle_ledger::charge(6);
            }
            assert_eq!(
                self.decompress_sprite_graphics_to_buffer(
                    dst + 0x600 * i,
                    self.sprite_gfx_subset(i) as usize
                ),
                0x600
            );
        }
        // $D779-$D787: SEP #$10, LDA long sprite_gfx_subset_3 : TAY :
        // JSR Decomp_spr (JSR only), STZ $0412, PLB, RTL.
        crate::cycle_ledger::charge(226);
        self.reset_incremental_vram_upload_counter();
    }

    pub(super) fn ReloadPreviouslyLoadedSheets(&mut self) {
        self.decompress_background_graphics_to_buffer(
            0x6000,
            self.game_state.world.palette_theme.aux_bg_subset(0) as usize,
        );
        self.decompress_background_graphics_to_buffer(
            0x6600,
            self.game_state.world.palette_theme.aux_bg_subset(1) as usize,
        );
        self.decompress_background_graphics_to_buffer(
            0x6c00,
            self.game_state.world.palette_theme.aux_bg_subset(2) as usize,
        );
        self.decompress_background_graphics_to_buffer(
            0x7200,
            self.game_state.world.palette_theme.aux_bg_subset(3) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x7800,
            self.game_state.sprites.workspace.graphics_subset(0) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x7e00,
            self.game_state.sprites.workspace.graphics_subset(1) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x8400,
            self.game_state.sprites.workspace.graphics_subset(2) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x8a00,
            self.game_state.sprites.workspace.graphics_subset(3) as usize,
        );
        self.reset_incremental_vram_upload_counter();
    }

    /// `Attract_DecompressStoryGFX` (`$00:D80E`, entered m8).
    /// ROM: `Attract_DecompressStoryGFX`
    pub(super) fn attract_decompress_story_gfx(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_d80e);
        if let Some(mut work) = self.story_graphics_decompression() {
            // This entry runs synchronously. The same work object can be
            // retained by an interruptible caller without decoding again or
            // rounding its progress to an output byte. Charge each contiguous
            // callee run once, preserving the ledger's routine ownership.
            let mut pending = StoryGraphicsInstruction { master_cycles: 0, routine: 0x00_d80e };
            let charge = |instruction: StoryGraphicsInstruction| {
                if instruction.routine == 0x00_d80e {
                    crate::cycle_ledger::charge(u64::from(instruction.master_cycles));
                } else {
                    crate::cycle_ledger::charge_routine(instruction.routine,
                        u64::from(instruction.master_cycles));
                }
            };
            while let Some(instruction) = work.step(&mut self.ram) {
                if instruction.routine != pending.routine {
                    charge(pending);
                    pending = instruction;
                } else {
                    pending.master_cycles += instruction.master_cycles;
                }
            }
            charge(pending);
            return;
        }
        // $D80E-$D81F: PHB : PHK : PLB, destination $7F:4000 in $00-$02/$05,
        // sheet $67 in $0E.
        crate::cycle_ledger::charge(240);
        // $D821-$D833, first pass: LDY $0E : JSR Decomp_spr (JSR only;
        // Decomp_spr charges itself), $01 += 8, INC $0E, CMP #$69 : BNE taken.
        crate::cycle_ledger::charge(248);
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x67,
        );
        // $D821-$D833, second pass: BNE not taken.
        crate::cycle_ledger::charge(242);
        self.decompress_sprite_graphics_to_buffer(0x14800, 0x68);
        // $D835-$D836: PLB : RTL.
        crate::cycle_ledger::charge(72);
    }

    pub(super) fn story_graphics_decompression(&self) -> Option<StoryGraphicsDecompression> {
        use crate::cycle_models::decompress::decompress_instruction_timeline;
        let mut sheets = Vec::with_capacity(2);
        let mut instructions = Vec::new();
        let mut elapsed = 0;
        let append = |instructions: &mut Vec<StoryGraphicsInstruction>, elapsed: &mut u64,
                      cycles: &[u32]| {
            for &master_cycles in cycles {
                instructions.push(StoryGraphicsInstruction { master_cycles, routine: 0x00_d80e });
                *elapsed += u64::from(master_cycles);
            }
        };
        // $D80E..$D81F: PHB, PHK, PLB, STZ $00, and the destination/sheet setup.
        append(&mut instructions, &mut elapsed, &[22, 22, 28, 24, 16, 24, 16, 24, 24, 16, 24]);
        for (sheet, destination) in [(0x67, 0x14000), (0x68, 0x14800)] {
            // $D821 LDY $0E : JSR Decomp_spr.
            append(&mut instructions, &mut elapsed, &[24, 46]);
            let bytes = self.decompressed_sprite_graphics_data(sheet)?;
            let (cost, mut stores, steps) = decompress_instruction_timeline(
                DecompressEntry::Sprite, sheet as u8, self.asset_bytes(64, sheet)?,
                stream_source_offset(DecompressEntry::Sprite, sheet as u8),
                0x7e_0000 + destination as u32,
            );
            for store in &mut stores {
                store.after_master_cycles += elapsed;
            }
            sheets.push(GraphicsDecompressionCursor::new(destination, bytes, stores,
                elapsed + cost.master)?);
            instructions.extend(steps.into_iter().map(|step| StoryGraphicsInstruction {
                master_cycles: step.master_cycles,
                routine: if step.in_get_next_byte { 0x00_e843 } else { 0x00_e772 },
            }));
            elapsed += cost.master;
            // $D826..$D833: destination += $800, next sheet, loop decision.
            append(&mut instructions, &mut elapsed,
                &[24, 14, 16, 24, 38, 24, 16, if sheet == 0x67 { 22 } else { 16 }]);
        }
        // $D835 PLB : RTL. These instructions still belong to this call.
        append(&mut instructions, &mut elapsed, &[28, 44]);
        Some(StoryGraphicsDecompression {
            sheets: sheets.try_into().ok()?, instructions,
            next_instruction: 0, elapsed_master_cycles: 0,
        })
    }

    pub(super) fn AnimateMirrorWarp(&mut self) {
        let st = self.overworld_map_state() as usize;
        self.increment_overworld_map_state();
        let nmi = MIRROR_WARP_LOAD_NEXT_NMI_LOAD.get(st).copied().unwrap_or(0);
        self.set_pending_nmi_subroutine(nmi);
        self.set_core_update_disable_flag(nmi);
        let xt = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            8
        } else {
            0
        };

        match st {
            0 => {
                if self.mirror_warp_scratch_mut().increment_load_step_counter() != 32 {
                    self.set_overworld_map_state(0);
                } else {
                    self.SetTargetOverworldWarpToPyramid();
                }
            }
            1 => {
                self.animate_mirror_warp_decompress_new_tile_sets();
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt] as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 1] as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            2 => {
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 2] as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 3] as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                self.do3_to_4_high_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            3 => {
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    self.game_state.world.palette_theme.aux_bg_subset(1) as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    self.game_state.world.palette_theme.aux_bg_subset(2) as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
            }
            4 => {
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 4] as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 5] as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
            }
            5 => {
                self.pre_overworld_load_overlays();
                if matches!(
                    self.game_state.world.location.overworld_screen_index(),
                    27 | 91
                ) {
                    self.set_sub_screen_layers(1);
                }
                self.decrement_submodule();
                self.set_pending_nmi_subroutine(12);
                self.set_core_update_disable_flag(12);
            }
            6 | 9 => {
                self.set_pending_nmi_subroutine(13);
                self.set_core_update_disable_flag(13);
            }
            7 => {
                self.overworld_draw_screen_at_current_mirror_position();
                self.increment_core_update_disable_flag();
            }
            8 => {
                self.follower_link_state_mut().set_blink_countdown(0x90);
                if self.rom_startup_timing() {
                    // The Map16ToMap8 conversion spans the wire's held
                    // vblanks; the sprite reload and this step's NMI
                    // subroutine follow at the load's completion.
                    let step = if self.game_state.frame.main_module == 0x15 {
                        crate::zelda_rtl::Module09LongLoadStep::Module15MirrorWarpSpriteLoadMap16
                    } else {
                        crate::zelda_rtl::Module09LongLoadStep::MirrorWarpSpriteLoadMap16
                    };
                    self.game_execution_scheduler
                        .schedule_work(GameWorkContinuation::FinishModule09LongLoad { step }, 3);
                    return;
                }
                self.mirror_warp_load_sprites_and_colors_after_blink();
                self.set_pending_nmi_subroutine(12);
                self.set_core_update_disable_flag(12);
            }
            10 => {
                let t = self.game_state.world.location.overworld_screen_index() & 0xbf;
                self.DecompressAnimatedOverworldTiles(if matches!(t, 3 | 5 | 7) {
                    0x58
                } else {
                    0x5a
                });
            }
            11 => {
                let t = self.game_state.world.location.overworld_screen_index();
                self.set_sub_screen_layers(matches!(
                    t,
                    0 | 0x70 | 0x40 | 0x5b | 3 | 5 | 7 | 0x43 | 0x45 | 0x47
                ) as u8);
                let Some(data) = self
                    .asset_bytes(64, MIRROR_WARP_BG_PACK_SEQUENCE[xt + 6] as usize)
                    .map(Vec::from)
                else {
                    return;
                };
                self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &data, 0, 64);
            }
            12 => {
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(0) as usize,
                );
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(1) as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                if matches!(
                    self.game_state.sprites.workspace.graphics_subset(0),
                    0x52 | 0x53 | 0x5a | 0x5b
                ) {
                    self.do3_to_4_high_16bit_from_slice(
                        LOAD_GFX_MESSAGING_BUF_LOAD_GFX,
                        &tmp,
                        0,
                        64,
                    );
                } else {
                    self.do3_to_4_low_16bit_from_slice(
                        LOAD_GFX_MESSAGING_BUF_LOAD_GFX,
                        &tmp,
                        0,
                        64,
                    );
                }
                self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            13 => {
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(2) as usize,
                );
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(3) as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
                self.handle_followers_after_mirroring();
            }
            14 => {
                self.set_overworld_map_state(14);
            }
            _ => {}
        }
    }

    /// ROM: `AnimateMirrorWarp_DecompressNewTileSets`
    pub(super) fn animate_mirror_warp_decompress_new_tile_sets(&mut self) {
        let main_tile_set = load_gfx_main_tileset(
            self.game_state.world.palette_theme.main_tile_theme_index() as usize,
        );
        let aux_tile_set = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            0,
            if aux_tile_set[0] != 0 {
                aux_tile_set[0]
            } else {
                main_tile_set[3]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            1,
            if aux_tile_set[1] != 0 {
                aux_tile_set[1]
            } else {
                main_tile_set[4]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            2,
            if aux_tile_set[2] != 0 {
                aux_tile_set[2]
            } else {
                main_tile_set[5]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            3,
            if aux_tile_set[3] != 0 {
                aux_tile_set[3]
            } else {
                main_tile_set[6]
            },
        );

        let p = load_gfx_sprite_tileset(self.game_state.sprites.system.graphics_index() as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.set_sprite_gfx_subset(i, pack);
            }
        }
    }

    /// ROM: `Graphics_IncrementalVRAMUpload`
    pub(super) fn graphics_incremental_vram_upload(&mut self) {
        let k = self
            .game_state
            .display
            .incremental_vram_upload_counter_usize();
        if k == 16 {
            return;
        }
        self.queue_tilemap_update(
            INCREMENTAL_VRAM_UPLOAD_DESTINATIONS[k],
            INCREMENTAL_VRAM_UPLOAD_SOURCES[k],
        );
        // Tag the per-frame animated-sprite CHR with an INJECTIVE logical source.
        // This streams the 4 per-area sprite gfx subsets (64 tiles each) to VRAM
        // 0x5000-0x5fff, 16 tiles per frame: chunk `k` fills VRAM page (0x50+k)
        // with subset `k/4`, sub-chunk `k%4`. `initialize_tilesets` already tagged
        // this region once via `load_sprite_graphics`, but with a possibly-stale /
        // zero `graphics_subset` (e.g. pack=0 when a subset was never set), while
        // the real animated content is re-DMA'd here every frame — leaving the
        // logical key (sprite, pack=0, tile_off) non-injective. Re-tag with the
        // LIVE subset pack so `(kind, pack, tile_off)` maps 1:1 to the pixels.
        let subset = k / 4;
        let pack = self.sprite_gfx_subset(subset) as u16;
        let dst_word = (INCREMENTAL_VRAM_UPLOAD_DESTINATIONS[k] as usize) << 8;
        let base_off = ((k % 4) * 16) as u16;
        self.vram_chr_source.record_tiles_from(
            dst_word,
            16,
            chr_source::CHR_KIND_SPRITE,
            pack,
            base_off,
        );
        self.vram_chr_preview_source.record_tiles_from(
            dst_word,
            16,
            chr_source::CHR_KIND_SPRITE,
            pack,
            base_off,
        );
        self.increment_vram_upload_counter();
    }

    pub(super) fn PrepTransAuxGfx(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_df1a));
        if self.lag_free {
            // $00:DF1A..DF4E: setup/calls/return; DF3C..44 only for high packs.
            crate::cycle_ledger::charge(332 + 144
                + if self.game_state.world.palette_theme.aux_tile_theme_index() >= 32 { 126 } else { 6 });
        }
        let tmp = self.staged_bg_and_sprite_decompression_buffers();
        self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 0x40);
        if self.game_state.world.palette_theme.aux_tile_theme_index() >= 32 {
            self.do3_to_4_high_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 0x80);
            self.do3_to_4_low_16bit_from_slice(0x11800, &tmp, 0x1200, 0x40);
        } else {
            self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 0xc0);
        }
    }

    /// `LoadNewSpriteGFXSet` (`$00:E031`, entered m8; `REP #$31` at `$E037`).
    pub(super) fn LoadNewSpriteGFXSet(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_e031);
        let tmp = self.graphics_sprite_decompression_buffer_tail();
        // $E031-$E052: bank $7E in $02/$05, X = 0, A = $7800, Y = $C0,
        // JSR Do3To4Low16Bit (JSR only; the callee charges itself), Y = $40,
        // LDA long sprite_gfx_subset_3 : AND #$00FF : CMP #$0052 : BEQ not
        // taken.
        crate::cycle_ledger::charge(340);
        self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 0xc0);
        // The compare chain $E052-$E061: each CMP : Bcc pair costs 40 when it
        // falls through and 6 more when its branch is taken.
        crate::cycle_ledger::charge(
            match self.game_state.sprites.workspace.graphics_subset(3) {
                0x52 => 6,            // $E052 BEQ taken
                0x53 => 40 + 6,       // $E054-$E057, BEQ taken
                0x5a => 80 + 6,       // ..$E05C, BEQ taken
                0x5b => 120,          // ..$E061, BNE not taken
                _ => 120 + 6,         // ..$E061, BNE taken to $E06B
            },
        );
        if matches!(
            self.game_state.sprites.workspace.graphics_subset(3),
            0x52 | 0x53 | 0x5a | 0x5b
        ) {
            // $E063-$E06A: LDA $03 : JSR Do3To4High16Bit (JSR only) :
            // SEP #$30 : RTL.
            crate::cycle_ledger::charge(144);
            self.do3_to_4_high_16bit_from_slice(0x11800, &tmp, 0x1200, 0x40);
        } else {
            // $E06B-$E072: LDA $03 : JSR Do3To4Low16Bit (JSR only) :
            // SEP #$30 : RTL.
            crate::cycle_ledger::charge(144);
            self.do3_to_4_low_16bit_from_slice(0x11800, &tmp, 0x1200, 0x40);
        }
    }

    pub(super) fn InitializeTilesets(&mut self) {
        self.initialize_tilesets();
    }

    /// ROM: `Attract_LoadBG3GFX`
    pub(super) fn attract_load_bg3_gfx(&mut self) {
        self.decompress_and_upload_2bpp(0x7800, 0x67);
    }

    pub(super) fn TransferFontToVRAM(&mut self) {
        self.transfer_font_to_vram();
    }

    pub(super) fn ResetHUDPalettes4and5(&mut self) {
        self.reset_hud_palettes_4_and_5();
    }

    pub(super) fn PaletteFilterHistory(&mut self) {
        self.palette_filter_history();
    }

    /// ROM: `PaletteFilter_WishPonds`
    pub(super) fn palette_filter_wish_ponds(&mut self) {
        self.set_sub_screen_layers(2);
        self.set_color_math_control(0x30);
        self.palette_filter_wish_ponds_inner();
    }

    /// ROM: `PaletteFilter_Crystal`
    pub(super) fn palette_filter_crystal(&mut self) {
        // $00:EBCF..EBF1: 102, eight colour clears (92, BPL taken seven
        // times) and the 224-cycle tail with its RTL.
        crate::cycle_ledger::charge_routine(0x00_ebcf, 102 + 8 * 92 + 7 * 6 + 224);
        self.set_sub_screen_layers(1);
        self.palette_filter_wish_ponds_inner();
    }

    /// ROM: `PaletteFilter_WishPonds_Inner`
    pub(super) fn palette_filter_wish_ponds_inner(&mut self) {
        for i in 0..8 {
            self.set_main_color_constant(0xd0 + i, 0);
        }
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(2);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_RestoreSP5F`
    pub(super) fn palette_filter_restore_sp5f(&mut self) {
        for i in 0..8 {
            self.copy_color((Bank::Aux, 208 + i), (Bank::Main, 208 + i));
        }
        self.set_sub_screen_layers(0);
        self.set_color_math_control(32);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_SP5F`
    pub(super) fn palette_filter_sp5f(&mut self) {
        for _ in 0..2 {
            self.palette_filter_range(208, 216);
            self.palette_filter_incr_countdown();
            if self.game_state.display.palette_filter.countdown_word() == 0 {
                break;
            }
        }
    }

    /// ROM: `KholdstareShell_PaletteFiltering`
    pub(super) fn kholdstare_shell_palette_filtering(&mut self) {
        let t = if self
            .game_state
            .enhanced_features
            .has(LOAD_GFX_FEATURES0_MISC_BUG_FIXES)
        {
            0x50
        } else {
            0x40
        };
        if self.game_state.frame.subsubmodule == 0 {
            for i in 0..8 {
                self.copy_color((Bank::Aux, t + i), (Bank::Main, t + i));
            }
            self.set_countdown_word(0);
            self.set_darkening_or_lightening_screen_word(0);
            self.increment_cgram_update_flag();
            self.set_subsubmodule(1);
            return;
        }
        for _ in 0..2 {
            self.palette_filter_range(t, t + 8);
            self.palette_filter_incr_countdown();
            if self.game_state.display.palette_filter.countdown_word() == 0 {
                self.set_sub_screen_layers(0);
                break;
            }
        }
    }

    pub(super) fn AgahnimWarpShadowFilter(&mut self, k: usize) {
        let pal_countdown = self.agahnim_palette_word(k);
        let darkening_screen = self.agahnim_palette_word(k + 3);
        self.set_countdown_word(pal_countdown);
        self.set_darkening_or_lightening_screen_word(darkening_screen);
        let t = AGAHNIM_WARP_SHADOW_PALETTE_OFFSETS[k] >> 1;
        for _ in 0..2 {
            self.palette_filter_range(t, t + 8);
            let countdown = self
                .game_state
                .display
                .palette_filter
                .countdown_word()
                .wrapping_add(1);
            self.set_countdown_word(countdown);
            if countdown == 0x1f {
                self.set_countdown_word(0);
                let screen = self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen_word()
                    ^ 2;
                self.set_darkening_or_lightening_screen_word(screen);
                break;
            }
        }
        let pal_countdown = self.game_state.display.palette_filter.countdown_word();
        let darkening_screen = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word();
        self.set_agahnim_palette_word(k, pal_countdown);
        self.set_agahnim_palette_word(k + 3, darkening_screen);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_RestoreSubtractive`
    pub(super) fn palette_filter_restore_subtractive(&mut self, from: usize, to: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_ee21));
        if self.lag_free && from < to {
            // $00:EE21..EE77, M16/X16: three original-channel comparisons,
            // each executed decrement or taken BEQ, the word store/loop,
            // entry and return. No palette child body is included here.
            let palette = &self.game_state.display.palette_buffer;
            let mut clocks = 32 + 42;
            for i in from >> 1..to >> 1 {
                clocks += 378 + 120 + 120 + 138 + 6;
                for mask in [0x001f, 0x03e0, 0x7c00] {
                    clocks += if palette.main_color(i) & mask == palette.aux_color(i) & mask { 6 } else { 66 };
                }
            }
            crate::cycle_ledger::charge(clocks - 6);
        }
        self.transform_main_range(from >> 1, to >> 1, PaletteTransform::RestoreSubtractiveStep);
    }

    /// ROM: `PaletteFilter_InitializeWhiteFilter`
    pub(super) fn palette_filter_initialize_white_filter(&mut self) {
        for i in 0..256 {
            self.set_aux_color_constant(i, 0x7fff);
        }
        self.copy_color((Bank::Main, 0), (Bank::Main, 32));
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(2);
        if self.game_state.world.location.overworld_screen_index() == 27 {
            self.set_aux_color_constant(0, 0);
            self.set_aux_color_constant(32, 0);
            self.set_main_color_constant(0, 0);
            self.set_main_color_constant(32, 0);
        }
        self.mirror_warp_scratch_mut().set_animation_counter(8);
        self.mirror_warp_scratch_mut().reset_load_step_counter();
    }

    /// ROM: `MirrorWarp_RunAnimationSubmodules`
    pub(super) fn mirror_warp_run_animation_submodules(&mut self) {
        // $00:FE64/$FF2F JSL $EEE7, M8/X8. Animation is a child;
        // the filter falls through to its shared return in this source frame.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00eee7));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x00eee7);
        let ctr = self.mirror_warp_scratch_mut().decrement_animation_counter();
        if ctr != 0 {
            b.taken(0x00eeea);
            b.block(0x00eee0);
            self.AnimateMirrorWarp();
            if !self.game_execution_scheduler.work_is_pending() { b.block(0x00eee4); }
            return;
        }
        b.block(0x00eeec);
        self.mirror_warp_scratch_mut().set_animation_counter(2);
        self.palette_filter_blinding_white_body(self.lag_free);
    }

    /// ROM: `PaletteFilter_BlindingWhite`
    pub(super) fn palette_filter_blinding_white(&mut self) {
        self.palette_filter_blinding_white_body(false);
    }

    fn palette_filter_blinding_white_body(&mut self, marked: bool) {
        let b = crate::cycle_ledger::blocks(marked);
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0xff
        {
            b.taken(0x00eefa);
            b.block(0x00eee4);
            return;
        }
        b.block(0x00eefc);
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 2
        {
            b.block(0x00ef01);
            self.palette_filter_restore_additive(0x40, 0x1b0);
            b.block(0x00ef0a);
            self.palette_filter_restore_additive(0x1c0, 0x1e0);
            b.block(0x00ef13);
        } else {
            b.taken(0x00eeff);
            b.block(0x00ef15);
            self.palette_filter_restore_subtractive(0x40, 0x1b0);
            b.block(0x00ef1e);
            self.palette_filter_restore_subtractive(0x1c0, 0x1e0);
        }
        self.palette_filter_start_blinding_white_body(marked);
    }

    /// ROM: `PaletteFilter_StartBlindingWhite`
    pub(super) fn palette_filter_start_blinding_white(&mut self) {
        self.palette_filter_start_blinding_white_body(false);
    }

    fn palette_filter_start_blinding_white_body(&mut self, marked: bool) {
        let b = crate::cycle_ledger::blocks(marked);
        b.block(0x00ef27);
        self.copy_color((Bank::Main, 32), (Bank::Main, 0));
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0
        {
            b.block(0x00ef35);
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 66 {
                b.block(0x00ef43);
                self.set_darkening_or_lightening_screen(0xff);
                self.mirror_warp_scratch_mut().set_animation_counter(32);
            } else { b.taken(0x00ef41); }
        } else {
            b.taken(0x00ef33);
            b.block(0x00ef56);
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 31 {
                b.block(0x00ef64);
                self.xor_darkening_or_lightening_screen(2);
                if self.game_state.frame.main_module != 21 {
                    b.taken(0x00ef75);
                    b.block(0x00ef89);
                    return;
                }
                b.block(0x00ef77);
                self.clear_hdma_enable_mask();
                for i in 0..240 {
                    self.set_spotlight_hdma_table_dynamic_entry(i, 0x0778);
                }
                self.set_hdma_enable_mask(0xc0);
                b.block(0x00ef87);
                b.block(0x00ef89);
                self.increment_cgram_update_flag();
                return;
            } else { b.taken(0x00ef62); }
        }
        b.block(0x00ef51);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_BlindingWhiteTriforce`
    pub(super) fn palette_filter_blinding_white_triforce(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x200);
        self.palette_filter_start_blinding_white();
    }

    /// ROM: `PaletteFilter_WhirlpoolBlue`
    pub(super) fn palette_filter_whirlpool_blue(&mut self) {
        if self.game_state.frame.frame_counter & 1 != 0 {
            self.transform_main_range(
                0x20,
                0x100,
                PaletteTransform::WhirlpoolChannelStep {
                    channel_mask: 0x7c00,
                    step: 0x400,
                    up: true,
                    reference: ChannelReference::Max,
                },
            );
            self.copy_color((Bank::Main, 32), (Bank::Main, 0));
            if self.game_state.display.palette_filter.countdown() & 1 == 0 {
                self.increment_mosaic_level_by(16);
            }
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 31 {
                self.set_countdown(0);
                self.increment_subsubmodule();
                self.set_mosaic_level(0xf0);
            }
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_IsolateWhirlpoolBlue`
    pub(super) fn palette_filter_isolate_whirlpool_blue(&mut self) {
        // Green then red, both stepped toward 0; the two channels are disjoint
        // and non-interacting so two passes equal the original single loop.
        self.transform_main_range(
            0x20,
            0x100,
            PaletteTransform::WhirlpoolChannelStep {
                channel_mask: 0x03e0,
                step: 0x20,
                up: false,
                reference: ChannelReference::Zero,
            },
        );
        self.transform_main_range(
            0x20,
            0x100,
            PaletteTransform::WhirlpoolChannelStep {
                channel_mask: 0x001f,
                step: 1,
                up: false,
                reference: ChannelReference::Zero,
            },
        );
        self.copy_color((Bank::Main, 32), (Bank::Main, 0));
        self.increment_countdown();
        if self.game_state.display.palette_filter.countdown() == 31 {
            self.set_countdown(0);
            self.increment_subsubmodule();
            self.set_mosaic_level(0xf0);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_WhirlpoolRestoreBlue`
    pub(super) fn palette_filter_whirlpool_restore_blue(&mut self) {
        if self.game_state.frame.frame_counter & 1 != 0 {
            self.transform_main_range(
                0x20,
                0x100,
                PaletteTransform::WhirlpoolChannelStep {
                    channel_mask: 0x7c00,
                    step: 0x400,
                    up: false,
                    reference: ChannelReference::Aux,
                },
            );
            self.copy_color((Bank::Main, 32), (Bank::Main, 0));
            if self.game_state.display.palette_filter.countdown() & 1 == 0 {
                self.decrement_mosaic_level_by(16);
            }
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 31 {
                self.set_countdown(0);
                self.increment_subsubmodule();
                self.clear_mosaic_level();
            }
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_WhirlpoolRestoreRedGreen`
    pub(super) fn palette_filter_whirlpool_restore_red_green(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_f0c7));
        if self.lag_free {
            // $00:F0C7..F131: actual two channel comparisons/updates for
            // each of 224 colors, loop branches and countdown completion.
            let palette = &self.game_state.display.palette_buffer;
            let mut clocks = 118 + 223 * 6 + 272 + 104;
            for i in 0x20..0x100 {
                clocks += 342 + 86 + 130;
                for mask in [0x03e0, 0x001f] {
                    clocks += if palette.main_color(i) & mask == palette.aux_color(i) & mask { 6 } else { 66 };
                }
            }
            clocks += if self.game_state.display.palette_filter.countdown().wrapping_add(1) == 31 { 94 } else { 6 };
            crate::cycle_ledger::charge(clocks);
        }
        // Green then red, both stepped toward aux; disjoint channels, so two
        // passes equal the original single loop.
        self.transform_main_range(
            0x20,
            0x100,
            PaletteTransform::WhirlpoolChannelStep {
                channel_mask: 0x03e0,
                step: 0x20,
                up: true,
                reference: ChannelReference::Aux,
            },
        );
        self.transform_main_range(
            0x20,
            0x100,
            PaletteTransform::WhirlpoolChannelStep {
                channel_mask: 0x001f,
                step: 1,
                up: true,
                reference: ChannelReference::Aux,
            },
        );
        self.copy_color((Bank::Main, 32), (Bank::Main, 0));
        self.increment_countdown();
        if self.game_state.display.palette_filter.countdown() == 31 {
            self.set_countdown(0);
            self.increment_subsubmodule();
        }
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_RestoreBGSubstractiveStrict`
    pub(super) fn palette_filter_restore_bg_substractive_strict(&mut self) {
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0xff
        {
            return;
        }
        self.palette_filter_restore_subtractive(0x40, 0x100);
        self.increment_countdown();
        if self.game_state.display.palette_filter.countdown() == 0x20 {
            self.set_darkening_or_lightening_screen(0xff);
            self.clear_sub_screen_layers_word();
        }
        self.increment_cgram_update_flag();
    }

    /// ROM: `PaletteFilter_RestoreBGAdditiveStrict`
    pub(super) fn palette_filter_restore_bg_additive_strict(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x100);
        self.increment_countdown();
        self.increment_cgram_update_flag();
    }

    /// ROM: `Trinexx_FlashShellPalette_Red`
    pub(super) fn trinexx_flash_shell_palette_red(&mut self) {
        if self.game_state.display.trinexx_palette.red_shell_delay == 0 {
            self.transform_main_range(
                0x41,
                0x48,
                PaletteTransform::TrinexxChannelStep {
                    channel_mask: 0x1f,
                    step: 1,
                    up: true,
                    reference: ChannelReference::Max,
                },
            );
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_red_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_red_shell_palette_step(0);
                self.set_trinexx_red_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_red_shell_palette_delay(3);
        }
        self.decrement_trinexx_red_shell_palette_delay();
    }

    /// ROM: `Trinexx_UnflashShellPalette_Red`
    pub(super) fn trinexx_unflash_shell_palette_red(&mut self) {
        if self.game_state.display.trinexx_palette.red_shell_delay == 0 {
            self.transform_main_range(
                0x41,
                0x48,
                PaletteTransform::TrinexxChannelStep {
                    channel_mask: 0x1f,
                    step: 1,
                    up: false,
                    reference: ChannelReference::Aux,
                },
            );
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_red_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_red_shell_palette_step(0);
                self.set_trinexx_red_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_red_shell_palette_delay(3);
        }
        self.decrement_trinexx_red_shell_palette_delay();
    }

    /// ROM: `Trinexx_FlashShellPalette_Blue`
    pub(super) fn trinexx_flash_shell_palette_blue(&mut self) {
        if self.game_state.display.trinexx_palette.blue_shell_delay == 0 {
            self.transform_main_range(
                0x41,
                0x48,
                PaletteTransform::TrinexxChannelStep {
                    channel_mask: 0x7c00,
                    step: 0x400,
                    up: true,
                    reference: ChannelReference::Max,
                },
            );
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_blue_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_blue_shell_palette_step(0);
                self.set_trinexx_blue_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_blue_shell_palette_delay(3);
        }
        self.decrement_trinexx_blue_shell_palette_delay();
    }

    /// ROM: `Trinexx_UnflashShellPalette_Blue`
    pub(super) fn trinexx_unflash_shell_palette_blue(&mut self) {
        if self.game_state.display.trinexx_palette.blue_shell_delay == 0 {
            self.transform_main_range(
                0x41,
                0x48,
                PaletteTransform::TrinexxChannelStep {
                    channel_mask: 0x7c00,
                    step: 0x400,
                    up: false,
                    reference: ChannelReference::Aux,
                },
            );
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_blue_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_blue_shell_palette_step(0);
                self.set_trinexx_blue_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_blue_shell_palette_delay(3);
        }
        self.decrement_trinexx_blue_shell_palette_delay();
    }

    /// ROM: `IrisSpotlight_close`
    pub(super) fn iris_spotlight_close(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00_f28b));
        // $00:F28B..F293: REP #$10, LDY #0, LDX #$7E, BRA $F29D.
        // The shared prefix, table call and return keep their existing prices.
        if self.lag_free { crate::cycle_ledger::charge(22 + 24 + 24 + 22); }
        self.spotlight_internal(0x7e, 0);
    }

    pub(super) fn AdjustWaterHDMAWindow(&mut self) {
        let r10 = self
            .game_state
            .display
            .water_hdma_window
            .window_y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let y_radius = self.game_state.display.water_hdma_window.window_y_radius();
        self.set_spotlight_y_lower(r10.wrapping_sub(y_radius));
        self.set_spotlight_y_upper(r10.wrapping_add(y_radius));
        self.adjust_water_hdma_window_x(r10);
    }

    /// ROM: `AdjustWaterHDMAWindow_X`
    pub(super) fn adjust_water_hdma_window_x(&mut self, r10: u16) {
        let window_x_center = self
            .game_state
            .display
            .water_hdma_window
            .window_x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        self.set_spotlight_window_x_center(window_x_center);
        let r12 = self.game_state.display.water_hdma_window.window_x_radius()
        .saturating_sub(1);
        let r2 = window_x_center.wrapping_add(r12).min(255);
        let r0 = window_x_center.wrapping_sub(r12).min(255);
        let r12_pair = r0 | (r2 << 8);

        let mut r6 = r10.wrapping_mul(2);
        if r6 < 0xe0 {
            r6 = 0xe0;
        }
        let mut r4 = r10.wrapping_mul(2).wrapping_sub(r6);
        loop {
            if (r4 as i16) >= 0 {
                let y_lower = self.game_state.display.spotlight_hdma.y_lower();
                let a = if (y_lower as i16) >= 0 && r4 < y_lower {
                    0x00ff
                } else {
                    r12_pair
                };
                if r4 < 240 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        r4 as usize,
                        if a != 0xffff { a } else { 0x00ff },
                    );
                }
            }

            let y_upper = self.game_state.display.spotlight_hdma.y_upper();
            let a = if r6 >= y_upper {
                0x00ff
            } else {
                if r6 >= 225
                    && self
                        .game_state
                        .display
                        .water_hdma_window
                        .watergate_spotlight_y_upper()
                        != 0
                {
                    self.water_hdma_window_mut()
                        .decrement_watergate_spotlight_y_upper();
                }
                r12_pair
            };
            if r6 < 240 {
                self.set_spotlight_hdma_table_dynamic_entry(
                    r6 as usize,
                    if a != 0xffff { a } else { 0x00ff },
                );
            }
            if r10 == r4 {
                break;
            }
            r6 = r6.wrapping_sub(1);
            r4 = r4.wrapping_add(1);
        }
    }

    /// ROM: `FloodDam_PrepFloodHDMA`
    pub(super) fn flood_dam_prep_flood_hdma(&mut self) {
        let lower = self
            .game_state
            .display
            .water_hdma_window
            .window_y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        self.set_spotlight_y_lower(lower);
        let window_x_center = self
            .game_state
            .display
            .water_hdma_window
            .window_x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        self.set_spotlight_window_x_center(window_x_center);
        let r14 = self.game_state.display.water_hdma_window.window_x_radius() ^ 1;
        let mut r4 = 0usize;
        let upper = self.game_state.display.spotlight_hdma.y_upper() as usize;
        while r4 != upper {
            self.set_spotlight_hdma_table_dynamic_entry(r4, 0xff00);
            r4 += 1;
        }
        let r12w = r14.wrapping_sub(7).wrapping_add(8);
        let r12_pair = (window_x_center.wrapping_add(r12w) << 8)
            | (window_x_center.wrapping_sub(r12w) & 0x00ff);
        let r10 = (self
            .game_state
            .display
            .spotlight_hdma
            .y_upper()
            .wrapping_add(self.game_state.display.water_hdma_window.window_y_radius()))
            ^ 1;
        while r4 < 225 {
            if r4 as u16 >= r10 {
                self.set_spotlight_hdma_table_dynamic_entry(r4, 0x00ff);
            } else {
                let mut a = r4 as u16;
                loop {
                    a = a.wrapping_mul(2);
                    if a < 480 {
                        break;
                    }
                }
                self.set_spotlight_hdma_table_dynamic_entry(
                    a as usize >> 1,
                    if r12_pair == 0xffff { 0x00ff } else { r12_pair },
                );
            }
            r4 += 1;
        }
    }

    pub(super) fn ResetStarTileGraphics(&mut self) {
        // $02:8A5F/$8CE9 JSL $00:FDA4: STZ then fall through $FDA7, not a child call.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00fda4));
        crate::cycle_ledger::blocks(self.lag_free).block(0x00fda4);
        self.dungeon_room_effects_mut().clear_star_tile_phase();
        self.dungeon_restore_star_tile_chr_body();
    }

    /// ROM: `Dungeon_RestoreStarTileChr`
    pub(super) fn dungeon_restore_star_tile_chr(&mut self) {
        // $01:CC54 JSL $00:FDA7; unlike FDA4 this retains the star phase.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x00fda7));
        self.dungeon_restore_star_tile_chr_body();
    }

    fn dungeon_restore_star_tile_chr_body(&mut self) {
        // $00:FDA7..FDED: two sixteen-word CHR copies and the NMI request.
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x00fda7);
        if self.game_state.dungeon.room_effects.star_tile_phase() == 0 {
            b.taken(0x00fdb2);
        } else {
            b.block(0x00fdb4);
        }
        b.block(0x00fdb8);
        let (xx, yy) = self.dungeon_star_tile_restore_source_offsets();
        let src0 = 0xbdc0 + xx;
        let src1 = 0xbdc0 + yy;
        for i in 0..16 {
            b.block(0x00fdc4);
            if i != 15 { b.taken(0x00fdd2); }
        }
        b.block(0x00fdd4);
        for i in 0..16 {
            b.block(0x00fdd6);
            if i != 15 { b.taken(0x00fde4); }
        }
        self.copy_graphics_message_rows(0, src0, src1, 32);
        b.block(0x00fde6);
        self.set_pending_nmi_subroutine(0x18);
    }

    /// ROM: `LinkZap_HandleMosaic`
    pub(super) fn link_zap_handle_mosaic(&mut self) {
        let mut level = self.game_state.display.mosaic_level;
        if self.game_state.display.mosaic_direction == 0 {
            level = level.wrapping_add(0x10);
            if level == 0xc0 {
                self.set_mosaic_direction(1);
            }
        } else {
            level = level.wrapping_sub(0x10);
            if level == 0 {
                self.clear_mosaic_direction();
            }
        }
        self.set_mosaic_level(level);
        self.set_mosaic_copy((level >> 1) | 3);
        self.set_bg_mode(9);
    }

    /// ROM: `Player_SetCustomMosaicLevel`
    pub(super) fn player_set_custom_mosaic_level(&mut self, level: u8) {
        self.clear_mosaic_direction();
        self.set_mosaic_level(level);
        self.set_mosaic_copy((level >> 1) | 3);
        self.set_bg_mode(9);
    }

    /// ROM: `Module07_16_UpdatePegs_Step1`
    pub(super) fn module07_16_update_pegs_step1(&mut self) {
        if self.lag_free {
            // $02:9739: index setup/barrier test; optional index swap.
            crate::cycle_ledger::charge(126 + 22
                + if self.game_state.dungeon.environment.orange_blue_barrier_state() == 0 { 6 } else { 38 });
        }
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        } else {
            self.dungeon_update_peg_gfx_buffer(0x100, 0x80);
        }
    }

    /// ROM: `Module07_16_UpdatePegs_Step2`
    pub(super) fn module07_16_update_pegs_step2(&mut self) {
        if self.lag_free {
            // $02:974D: index setup/barrier test; optional index swap.
            crate::cycle_ledger::charge(126 + 22
                + if self.game_state.dungeon.environment.orange_blue_barrier_state() == 0 { 6 } else { 38 });
        }
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.dungeon_update_peg_gfx_buffer(0x100, 0x80);
        } else {
            self.dungeon_update_peg_gfx_buffer(0x80, 0x100);
        }
    }

    /// ROM: `Dungeon_UpdatePegGFXBuffer`
    pub(super) fn dungeon_update_peg_gfx_buffer(&mut self, x: usize, y: usize) {
        if self.lag_free {
            // $02:9773..97A8, M8/X16 on entry (PHA, not a wide immediate):
            // two 64-word copies, 63 taken branches each, NMI request/return.
            crate::cycle_ledger::charge(166 + 2 * (64 * 190 + 63 * 6) + 32 + 132);
        }
        self.copy_peg_tile_graphics_to_message_buffer(x, y);
        self.set_pending_nmi_subroutine(23);
    }

    /// ROM: `Dungeon_HandleTranslucencyAndPalette`
    pub(super) fn dungeon_handle_translucency_and_palette(&mut self) {
        if self.palette_swap_enabled() {
            self.palette_revert_translucency_swap();
        }

        self.set_color_window_selection(2);
        self.set_color_math_control(0xb3);

        let mut torch = self.game_state.dungeon.torch.lit_torches();
        if self.game_state.dungeon.torch.wants_lights_out() == 0 {
            let mut a = 0x20;
            if self.game_state.dungeon.room_load.bg2_properties() != 0 {
                a = 0x32;
                if self.game_state.dungeon.room_load.bg2_properties() != 7 {
                    a = 0x62;
                    if self.game_state.dungeon.room_load.bg2_properties() != 4 {
                        a = 0x20;
                        if self.game_state.dungeon.room_load.bg2_properties() == 2 {
                            self.palette_assert_translucency_swap_with_cycle_charge();
                            a = 0x70;
                            if self.game_state.world.location.dungeon_room_index() == 13 {
                                self.clear_agahnim_palette_settings(6);
                                self.palette_load_agahnim();
                                // Palette_LoadAgahnim returns with X = $D0 (its
                                // last copy ends at $01D0, then SEP #$30), and
                                // $02:A251 indexes the lit-torch table with it:
                                // the byte at $02:A2B5 is $85.
                                self.set_color_math_control(a);
                                self.set_overworld_fixed_color_adjustment(0x85);
                                self.finish_dungeon_translucency_and_palette();
                                return;
                            }
                        }
                    }
                }
            }
            self.set_color_math_control(a);
            torch = 3;
        }
        self.set_overworld_fixed_color_adjustment(
            LIT_TORCHES_COLOR_PLUS
                .get(torch as usize)
                .copied()
                .unwrap_or(0),
        );
        self.finish_dungeon_translucency_and_palette();
    }

    fn finish_dungeon_translucency_and_palette(&mut self) {
        self.set_countdown(31);
        self.clear_mosaic_target_level();
        self.set_darkening_or_lightening_screen(2);
        self.clear_overworld_aux_or_main_offset();
        self.palette_load_dungeon_set();
        self.palette_load_sp0l();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.increment_subsubmodule();
    }

    pub(super) fn OverworldLoadScreensPaletteSet(&mut self) {
        let sc = self.game_state.world.location.overworld_screen_index() & 0x3f;
        let mut x = if matches!(sc, 3 | 5 | 7) { 2 } else { 0 };
        if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            x += 1;
        }
        self.overworld_load_area_palettes_ex(x);
    }

    /// ROM: `Overworld_LoadAreaPalettesEx`
    pub(super) fn overworld_load_area_palettes_ex(&mut self, palette: u8) {
        self.set_overworld_palette_mode(palette);
        self.keep_overworld_aux_or_main_low_byte();
        self.palette_load_sprite_main();
        self.palette_load_sprite_environment();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_link_armor_and_gloves();
        let sp0l = if self.game_state.inventory.save_progress.dark_world_state() & 0x40 != 0 {
            3
        } else {
            1
        };
        self.set_sp0l(sp0l);
        self.palette_load_sp0l();
        self.palette_load_hud();
        self.palette_load_ow_bg_main();
    }

    /// ROM: `SpecialOverworld_CopyPalettesToCache`
    pub(super) fn special_overworld_copy_palettes_to_cache(&mut self) {
        for i in 32..(32 * 8) {
            self.set_main_color_constant(i, 0);
        }
        for i in 0..8 {
            for base in [0x00, 0x08, 0x10, 0x18, 0xd8, 0xe8, 0xf0, 0xf8] {
                self.copy_color((Bank::Aux, base + i), (Bank::Main, base + i));
            }
        }
        self.set_mosaic_copy(0xf7);
        self.set_mosaic_level(0xf7);
        self.increment_cgram_update_flag();
    }

    fn overworld_select_palettes(&mut self, bg: u8, spr: u8) {
        self.clear_overworld_aux_or_main_offset();
        let bg_base = bg as usize * 3;
        if OW_BG_PAL_INFO[bg_base] >= 0 {
            self.world_palette_theme_mut()
                .set_overworld_palette_aux1_hi(OW_BG_PAL_INFO[bg_base] as u8);
        }
        if OW_BG_PAL_INFO[bg_base + 1] >= 0 {
            self.set_overworld_palette_aux2_hi(OW_BG_PAL_INFO[bg_base + 1] as u8);
        }
        if OW_BG_PAL_INFO[bg_base + 2] >= 0 {
            self.set_overworld_palette_aux3_lo(OW_BG_PAL_INFO[bg_base + 2] as u8);
        }
        let spr_base = spr as usize * 2;
        if OW_SPR_PAL_INFO[spr_base] >= 0 {
            self.set_sp5l(OW_SPR_PAL_INFO[spr_base] as u8);
        }
        if OW_SPR_PAL_INFO[spr_base + 1] >= 0 {
            self.set_sp6l(OW_SPR_PAL_INFO[spr_base + 1] as u8);
        }
    }

    /// ROM: `Overworld_LoadPalettes`
    pub(super) fn overworld_load_palettes(&mut self, bg: u8, spr: u8) {
        self.overworld_select_palettes(bg, spr);
        self.palette_load_ow_bg1();
        self.palette_load_ow_bg2();
        self.palette_load_ow_bg3();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
    }

    pub(super) fn overworld_load_palettes_through_ow_bg2_words(
        &mut self,
        bg: u8,
        spr: u8,
        completed_ow_bg2_words: usize,
    ) {
        assert!(completed_ow_bg2_words <= 21);
        self.overworld_select_palettes(bg, spr);
        self.palette_load_ow_bg1();
        self.palette_load_ow_bg2_word_range(0, completed_ow_bg2_words);
    }

    pub(super) fn overworld_load_palettes_after_ow_bg2_words(
        &mut self,
        completed_ow_bg2_words: usize,
    ) {
        assert!(completed_ow_bg2_words <= 21);
        self.palette_load_ow_bg2_word_range(completed_ow_bg2_words, 21);
        self.palette_load_ow_bg3();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
    }

    /// ROM: `Palette_SetOwBgColor`
    pub(super) fn palette_set_ow_bg_color(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.palette_set_bg_and_fixed_color(color);
    }

    /// ROM: `Palette_SpecialOw`
    pub(super) fn palette_special_ow(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.set_aux_color_constant(0, color);
        self.set_aux_color_constant(32, color);
        self.set_backdrop_color_black();
    }

    /// ROM: `Palette_AssertTranslucencySwap`
    pub(super) fn palette_assert_translucency_swap_with_cycle_charge(&mut self) {
        // $0E:D657..D6B8: 86, eight passes of the three-band swap (1,122,
        // BNE taken seven times) and the 104-cycle tail with its RTL.
        crate::cycle_ledger::charge_routine(0x0e_d657, 86 + 8 * 1_122 + 7 * 6 + 104);
        self.palette_set_translucency_swap(true);
    }

    pub(super) fn LoadActualGearPalettes(&mut self) {
        self.load_actual_gear_palettes();
    }

    /// ROM: `Palette_Load_SpriteEnvironment`
    pub(super) fn palette_load_sprite_environment_with_cycle_scope(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x1b_ed6e);
        self.palette_load_sprite_environment();
    }

    /// ROM: `Palette_Load_SpriteEnvironment_Dungeon`
    pub(super) fn palette_load_sprite_environment_dungeon_with_cycle_scope(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x1b_ed72);
        self.palette_load_sprite_environment_dungeon();
    }

    /// ROM: `Palette_Load_DungeonMapSprite`
    pub(super) fn palette_load_dungeon_map_sprite(&mut self) {
        if let Some(palette) = self.asset_raw(91).map(Vec::from) {
            let aux_or_main = self
                .game_state
                .display
                .palette_buffer
                .overworld_aux_or_main_offset() as usize;
            for pal in 0..=2 {
                for i in 0..=6 {
                    let color = read_word_from_slice(&palette, (pal * 7 + i) * 2);
                    let idx = ((0x182 + aux_or_main) >> 1) + pal * 0x10 + i;
                    self.set_aux_color_asset(idx, color);
                }
            }
        }
    }

    /// ROM: `Palette_Load_DungeonMapBG`
    pub(super) fn palette_load_dungeon_map_bg(&mut self) {
        if let Some(palette) = self.asset_raw(90).map(Vec::from) {
            let aux_or_main = self
                .game_state
                .display
                .palette_buffer
                .overworld_aux_or_main_offset() as usize;
            for pal in 0..=5 {
                for i in 0..=15 {
                    let color = read_word_from_slice(&palette, (pal * 16 + i) * 2);
                    let idx = ((0x40 + aux_or_main) >> 1) + pal * 0x10 + i;
                    self.set_aux_color_asset(idx, color);
                }
            }
        }
    }

    /// ROM: `Palette_Load_OWBGMain`
    pub(super) fn palette_load_owbg_main(&mut self) {
        self.palette_load_ow_bg_main();
    }

    /// ROM: `Palette_LoadAgahnim`
    pub(super) fn palette_load_agahnim(&mut self) {
        let src = PALETTE_SPRITE_AUX1_SNES_ADDR + 14 * 7 * 2;
        self.palette_load_multiple_arbitrary_snes(src, 0x162, 6);
        self.palette_load_multiple_arbitrary_snes(src, 0x182, 6);
        self.palette_load_multiple_arbitrary_snes(src, 0x1a2, 6);
        self.palette_load_multiple_arbitrary_snes(
            PALETTE_SPRITE_AUX1_SNES_ADDR + 21 * 7 * 2,
            0x1c2,
            6,
        );
        self.increment_cgram_update_flag();
    }

    pub(super) fn asset_bytes(&self, asset: usize, index: usize) -> Option<&[u8]> {
        Some(self.asset_memblk(asset, index)?.ptr)
    }
}

impl ZeldaState {
    pub(super) fn finish_native_screen_flash(
        &mut self,
        tail: crate::cycle_models::dungeon_garnish::GarnishTail,
    ) {
        use crate::cycle_models::dungeon_garnish::GarnishTail;
        let _scope = crate::cycle_ledger::routine(0x1d_e9b6);
        match tail {
            GarnishTail::PaletteRestore => {
                self.palette_restore_bg_and_hud();
                crate::cycle_ledger::charge(44);
            }
            GarnishTail::PaletteWhiten => {
                self.filter_majorly_whiten_bg();
                crate::cycle_ledger::charge(22 + 38 + 44);
                self.increment_cgram_update_flag();
            }
            GarnishTail::PaletteFromFlash => {
                self.palette_restore_bg_from_flash();
                crate::cycle_ledger::charge(38 + 44);
                self.increment_cgram_update_flag();
            }
            _ => unreachable!("flash resumes only at a selected palette child"),
        }
    }

    pub(super) fn handle_screen_flash(&mut self) {
        // Cycle ledger: HandleScreenFlash $1D:E9B6 (m8 x8, JSL target).
        let _scope = crate::cycle_ledger::routine(0x1d_e9b6);
        let flash = self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count();
        // $1D:E9B6-E9B9 LDA $0ff9 : BEQ (48), then $1D:E9BB-E9BD LDY $11 :
        // BNE (40); either taken branch (+6) lands on $1D:E9D9 RTL (44).
        if flash == 0 {
            crate::cycle_ledger::charge(48 + 6 + 44);
        } else if self.game_state.frame.submodule != 0 {
            crate::cycle_ledger::charge(48 + 40 + 6 + 44);
        } else {
            crate::cycle_ledger::charge(48 + 40);
        }
        if flash == 0 || self.game_state.frame.submodule != 0 {
            return;
        }
        // $1D:E9BF-E9C2 DEC $0ff9 : BNE (62): a zero result falls into
        // $1D:E9C4 JSL Palette_Restore_BG_And_HUD : RTL (106).
        self.attract_scene_mut()
            .set_intro_palette_flash_count(flash.wrapping_sub(1));
        if self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count()
            == 0
        {
            crate::cycle_ledger::charge(62 + 106);
            self.palette_restore_bg_and_hud();
            return;
        }
        // BNE taken (+6) into $1D:E9C9-E9CB AND #$01 : BEQ (32) on the
        // pre-decrement count: odd runs $1D:E9CD JSL Filter_Majorly_Whiten_Bg
        // : BRA (84), even takes the BEQ (+6) into $1D:E9D3 JSL
        // Palette_Restore_BG_From_Flash (62); then $1D:E9D7 INC $15 (38) and
        // the RTL (44). The callees charge their own bodies.
        if flash & 1 != 0 {
            crate::cycle_ledger::charge(62 + 6 + 32 + 84 + 38 + 44);
            // ROM $1D:E9CD
            self.filter_majorly_whiten_bg_marked();
        } else {
            crate::cycle_ledger::charge(62 + 6 + 32 + 6 + 62 + 38 + 44);
            self.palette_restore_bg_from_flash();
        }
        self.increment_cgram_update_flag();
    }

    /// `ApplyPaletteFilter` (`$00:E914`) cycle ledger, entry through the
    /// `JSR FilterColors`: `$e914-$e920` (134; `BCC $e924` taken +6 while
    /// the countdown is below `$10`, else `$e922-$e923` `INX INX` 28 selects
    /// the second filtering-bits row) and `$e924-$e94a` (600: pointer, mask
    /// and delta setup, `LDX #$0040`, `JSR $e9e4`; the first 600 cycles of
    /// the listing's 896-cycle block, whose remaining 296 are color 0 after
    /// the call, see `charge_apply_palette_filter_color0`).
    fn charge_apply_palette_filter_entry(&self) {
        let countdown = self.game_state.display.palette_filter.countdown_word();
        if countdown < 0x10 {
            crate::cycle_ledger::charge(134 + 6);
        } else {
            crate::cycle_ledger::charge(134 + 28);
        }
        crate::cycle_ledger::charge(600);
    }

    /// `ApplyPaletteFilter` cycle ledger for color 0, which the ROM filters
    /// inline after `FilterColors` returns (no zero-aux shortcut):
    /// `$e94d-$e961` (296, the tail of the listing's 896-cycle block: load
    /// color 0, red lookup, `BNE $e969`), then per channel either the add
    /// block or the taken branch that skips it (red `$e963-$e967` 96; green
    /// lookup `$e969-$e978` 230 and add `$e97a-$e97e` 96; blue lookup
    /// `$e980-$e98c` 188 and add `$e98e-$e993` 110). Priced from the same
    /// table lookups `PaletteTransform::FilterRangeStep` applies.
    fn charge_apply_palette_filter_color0(&self) {
        let countdown = self.game_state.display.palette_filter.countdown_word();
        let aux = self.game_state.display.palette_buffer.aux_color(0);
        let work = zelda3_palette::filter_range_step_work(aux, countdown);
        crate::cycle_ledger::charge(296 + if work.red_steps { 96 } else { 6 });
        crate::cycle_ledger::charge(230 + if work.green_steps { 96 } else { 6 });
        crate::cycle_ledger::charge(188 + if work.blue_steps { 110 } else { 6 });
    }

    /// `FilterColors` (`$00:E9E4`) cycle ledger for colors `from..to` of the
    /// ROM loop (`X` = index * 2 from `$40` to `$1B0`, then `$1C0` to `$1E0`),
    /// priced per color from the data the ROM branches on (the aux word and
    /// the filter countdown) through the same table lookups
    /// `PaletteTransform::FilterRangeStep` applies. The caller owns the
    /// `$00:E9E4` routine scope; `from..to` must lie inside the loop's two
    /// index ranges.
    fn charge_filter_colors_range(&self, from: usize, to: usize) {
        let countdown = self.game_state.display.palette_filter.countdown_word();
        for index in from..to {
            let aux = self.game_state.display.palette_buffer.aux_color(index);
            let work = zelda3_palette::filter_range_step_work(aux, countdown);
            if work.aux_nonzero {
                // $e9e4-$e9ee: load main and aux, `BEQ $ea34` not taken.
                crate::cycle_ledger::charge(144);
                // $e9f0-$e9fa red lookup (168); `BNE $ea02` taken (+6) skips
                // the $e9fc-$ea00 add (96).
                crate::cycle_ledger::charge(168 + if work.red_steps { 96 } else { 6 });
                // $ea02-$ea11 green lookup (230); $ea13-$ea17 add (96).
                crate::cycle_ledger::charge(230 + if work.green_steps { 96 } else { 6 });
                // $ea19-$ea25 blue lookup (188); $ea27-$ea2c add (110).
                crate::cycle_ledger::charge(188 + if work.blue_steps { 110 } else { 6 });
                // $ea2e-$ea30: store the color.
                crate::cycle_ledger::charge(80);
            } else {
                // $e9e4-$e9ee with `BEQ $ea34` taken: a zero aux word is skipped.
                crate::cycle_ledger::charge(144 + 6);
            }
            // $ea34-$ea39: INX INX, CPX #$01b0, BCC (68).
            match index {
                // `BCC $e9e4` taken back to the loop while X < $1b0.
                0x20..=0xd6 => crate::cycle_ledger::charge(68 + 6),
                // X == $1b0: `BNE $ea43` not taken (16), $ea3d-$ea42 steps X
                // to $1c0 (66), $ea43-$ea46 CPX #$01e0 with BNE taken (40 + 6).
                0xd7 => crate::cycle_ledger::charge(68 + 16 + 66 + 40 + 6),
                // X > $1b0: `BNE $ea43` taken (16 + 6), $ea43-$ea46 CPX #$01e0
                // with BNE taken (40 + 6).
                0xe0..=0xee => crate::cycle_ledger::charge(68 + 22 + 46),
                // X == $1e0: `BNE $ea43` taken (22), CPX #$01e0 with BNE not
                // taken (40), `RTS` (42).
                0xef => crate::cycle_ledger::charge(68 + 22 + 40 + 42),
                _ => unreachable!("FilterColors filters colors $20-$D7 and $E0-$EF"),
            }
        }
    }

    /// ROM: `ApplyPaletteFilter_bounce`
    pub(super) fn apply_palette_filter_bounce(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_e914);
        self.charge_apply_palette_filter_entry();
        self.palette_filter_range(0, 1);
        // `JSR $e9e4 FilterColors` covers colors $20-$D7 and $E0-$EF.
        let filter_colors = crate::cycle_ledger::routine(0x00_e9e4);
        self.charge_filter_colors_range(0x20, 0xd8);
        self.palette_filter_range(0x20, 0xd8);
        self.charge_filter_colors_range(0xe0, 0xf0);
        self.palette_filter_range(0xe0, 0xf0);
        drop(filter_colors);
        // $e94d-$e993: color 0, filtered after `FilterColors` returns.
        self.charge_apply_palette_filter_color0();
        self.complete_apply_palette_filter_bounce_after_ranges();
    }

    pub(super) fn apply_palette_filter_bounce_through_direction_toggle(&mut self) {
        // One `ApplyPaletteFilter` entry: the prefix body runs inside this
        // scope so the host records a single call.
        let _scope = crate::cycle_ledger::routine(0x00_e914);
        self.apply_palette_filter_bounce_prefix_slices(0xf0);
        // $e94d-$e993: color 0, filtered after `FilterColors` returns.
        self.charge_apply_palette_filter_color0();
        let countdown = self.game_state.display.palette_filter.countdown_word();
        let direction = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word();
        let target = u16::from(self.game_state.display.mosaic_target_level);
        if direction == 0 {
            // $e995-$e9a0 (172, `BNE $e9cc` not taken) and $e9a2-$e9af (174,
            // `BNE $e9c7` not taken: the countdown reaches its target).
            crate::cycle_ledger::charge(172 + 174);
            let next = countdown.wrapping_add(1);
            assert_eq!(
                next, target,
                "source palette direction toggle requires its terminal countdown"
            );
            self.set_countdown_word(next);
        } else {
            // $e995-$e9a0 with `BNE $e9cc` taken (172 + 6) and $e9cc-$e9d4 with
            // `BEQ $e9b1` taken (112 + 6).
            crate::cycle_ledger::charge(172 + 6 + 112 + 6);
            assert_eq!(
                countdown, target,
                "source palette direction toggle requires its terminal countdown"
            );
        }
        // $e9b1-$e9b8: direction toggle load, EOR, store (120). The NMI
        // interrupts before the countdown reset, which the completion charges.
        crate::cycle_ledger::charge(120);
        self.set_darkening_or_lightening_screen_word(direction ^ 2);
    }

    pub(super) fn complete_apply_palette_filter_bounce_after_direction_toggle(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_e914);
        // $e9bc-$e9c5: LDA #0, countdown store, SEP #$20, INC $b0 (132) and
        // $e9c7-$e9cb: SEP #$30, INC $15, RTL (104).
        crate::cycle_ledger::charge(132 + 104);
        self.set_countdown_word(0);
        self.increment_subsubmodule();
        self.increment_cgram_update_flag();
    }

    const fn valid_apply_palette_filter_next_color(next_color: u8) -> bool {
        matches!(next_color, 0 | 1)
            || (next_color >= 0x20 && next_color <= 0xd8)
            || (next_color >= 0xe0 && next_color <= 0xf0)
    }

    pub(super) fn apply_palette_filter_bounce_prefix(&mut self, next_color: u8) {
        let _scope = crate::cycle_ledger::routine(0x00_e914);
        self.apply_palette_filter_bounce_prefix_slices(next_color);
    }

    /// The prefix body without its `ApplyPaletteFilter` ledger scope, for the
    /// caller that continues the same source call in this host
    /// (`apply_palette_filter_bounce_through_direction_toggle`).
    fn apply_palette_filter_bounce_prefix_slices(&mut self, next_color: u8) {
        assert!(Self::valid_apply_palette_filter_next_color(next_color));
        self.charge_apply_palette_filter_entry();
        // The ROM filters color 0 after `FilterColors` returns, so its
        // $e94d-$e993 cost belongs to the completion, whichever slice
        // computes the color here.
        if next_color > 0 {
            self.palette_filter_range(0, 1);
        }
        // `JSR $e9e4 FilterColors` is entered once the prefix reaches color $20.
        let _filter_colors =
            (next_color > 0x20).then(|| crate::cycle_ledger::routine(0x00_e9e4));
        if next_color > 0x20 {
            self.charge_filter_colors_range(0x20, usize::from(next_color.min(0xd8)));
            self.palette_filter_range(0x20, usize::from(next_color.min(0xd8)));
        }
        if next_color > 0xe0 {
            self.charge_filter_colors_range(0xe0, usize::from(next_color.min(0xf0)));
            self.palette_filter_range(0xe0, usize::from(next_color.min(0xf0)));
        }
    }

    pub(super) fn complete_apply_palette_filter_bounce_from(&mut self, next_color: u8) {
        assert!(Self::valid_apply_palette_filter_next_color(next_color));
        // The resumed `ApplyPaletteFilter` call; the profile counts the
        // resumed tail as its own call on this host.
        let _scope = crate::cycle_ledger::routine(0x00_e914);
        if next_color == 0 {
            self.palette_filter_range(0, 1);
        }
        // The resumed `FilterColors` loop, when colors below $F0 remain.
        let filter_colors =
            (next_color < 0xf0).then(|| crate::cycle_ledger::routine(0x00_e9e4));
        if next_color <= 0xd8 {
            self.charge_filter_colors_range(usize::from(next_color.max(0x20)), 0xd8);
            self.palette_filter_range(usize::from(next_color.max(0x20)), 0xd8);
        }
        if next_color <= 0xf0 {
            self.charge_filter_colors_range(usize::from(next_color.max(0xe0)), 0xf0);
            self.palette_filter_range(usize::from(next_color.max(0xe0)), 0xf0);
        }
        drop(filter_colors);
        // $e94d-$e993: color 0, filtered after `FilterColors` returns.
        self.charge_apply_palette_filter_color0();
        self.complete_apply_palette_filter_bounce_after_ranges();
    }

    fn complete_apply_palette_filter_bounce_after_ranges(&mut self) {
        self.increment_cgram_update_flag();

        let countdown = self.game_state.display.palette_filter.countdown_word();
        let target = self.game_state.display.mosaic_target_level as u16;
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word()
            == 0
        {
            // $e995-$e9a0: store color 0, PLB, direction test with `BNE $e9cc`
            // not taken (172); $e9a2-$e9af: increment the countdown and
            // compare it with the target (174).
            crate::cycle_ledger::charge(172 + 174);
            let next = countdown.wrapping_add(1);
            self.set_countdown_word(next);
            if next != target {
                // `BNE $e9c7` taken (+6); $e9c7-$e9cb: SEP #$30, INC $15, RTL (104).
                crate::cycle_ledger::charge(6 + 104);
                return;
            }
        } else {
            // $e995-$e9a0 with `BNE $e9cc` taken (172 + 6); $e9cc-$e9d4:
            // compare the countdown with the target (112).
            crate::cycle_ledger::charge(172 + 6 + 112);
            self.set_countdown_word(countdown.wrapping_sub(1));
            if countdown != target {
                // $e9d6-$e9e3: decrement the countdown, SEP #$30, INC $15, RTL (214).
                crate::cycle_ledger::charge(214);
                return;
            }
            // `BEQ $e9b1` taken (+6).
            crate::cycle_ledger::charge(6);
        }
        // $e9b1-$e9c5: direction toggle, countdown reset, SEP #$20, INC $b0
        // (252); $e9c7-$e9cb: SEP #$30, INC $15, RTL (104).
        crate::cycle_ledger::charge(252 + 104);
        let mode = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word()
            ^ 2;
        self.set_darkening_or_lightening_screen_word(mode);
        self.set_countdown_word(0);
        self.increment_subsubmodule();
    }

    /// ROM: `Spotlight_open`
    pub(super) fn spotlight_open(&mut self) {
        // Cycle ledger: Spotlight_open ($00:F295). Only $F295-$F29A (REP #$10,
        // LDY #$0002, LDX #$0000 = 22 + 24 + 24) belongs to the entry; the
        // shared SpotlightInternal body from $F29D is charged by
        // `spotlight_internal_before_table` / `spotlight_internal_after_table`.
        let _scope = crate::cycle_ledger::routine(0x00_f295);
        crate::cycle_ledger::charge(70);
        self.spotlight_internal(0, 2);
    }

    pub(super) fn spotlight_internal(&mut self, x: u8, y: u8) {
        self.spotlight_internal_before_table(x, y);
        self.iris_spotlight_configure_table();
        self.spotlight_internal_after_table();
    }

    /// Synchronous prefix of the ROM's `SpotlightInternal`, before the circle
    /// table builder. These display controls are already live when a long
    /// `IrisSpotlight_ConfigureTable` is interrupted by vblank.
    pub(super) fn spotlight_internal_before_table(&mut self, x: u8, y: u8) {
        // Cycle ledger: SpotlightInternal $F29D-$F2DE (STY $67E, STX $67C,
        // STZ $420C, the four channel-7 HDMA register pairs, W12SEL/W34SEL/
        // WOBJSEL copies, TMW/TSW copies, LDA $1B, BNE not taken): 806 for
        // the whole $F295-$F2DE block, less 26 for the nine register writes
        // made with the system data bank, less the 70 of Spotlight_open's
        // own entry = 710.
        crate::cycle_ledger::charge(710);
        self.set_spotlight_window_radius(x as u16);
        self.set_spotlight_window_state(y as u16);
        self.hdma_setup(0x00f2fb, 0x00f2fb, 0x41, 0x26, 0x26, 0);
        self.set_bg12_window_selection(0x33);
        self.set_bg34_window_selection(3);
        self.set_object_color_window_selection(0x33);
        let main_screen = self.game_state.display.main_screen_layers;
        self.set_main_screen_window_layers(main_screen);
        let sub_screen = self.game_state.display.sub_screen_layers;
        self.set_sub_screen_window_layers(sub_screen);
        if self.game_state.world.location.is_outdoors() {
            // $F2E0-$F2EA: the three fixed-colour writes.
            crate::cycle_ledger::charge(120);
            self.set_fixed_color_red(0x20);
            self.set_fixed_color_green(0x40);
            self.set_fixed_color_blue(0x80);
        } else {
            // BNE $F2EC taken.
            crate::cycle_ledger::charge(6);
        }
        // $F2EC-$F2EE: SEP #$10 + JSL IrisSpotlight_ConfigureTable (22 + 62);
        // the callee charges its own body.
        crate::cycle_ledger::charge(84);
    }

    /// Synchronous suffix of the ROM's `SpotlightInternal`, reached only after
    /// `IrisSpotlight_ConfigureTable` returns.
    pub(super) fn spotlight_internal_after_table(&mut self) {
        // $F2F2-$F2FA: LDA #$80, STA $9B, LDA #$0F, STA $13, RTL.
        crate::cycle_ledger::charge(124);
        self.set_hdma_enable_mask(0x80);
        self.set_screen_brightness(0x0f);
        self.stage_spotlight_scanout_for_next_display();
    }

    /// Complete `SpotlightInternal` after its table builder resumed during an
    /// already-active field. The register copies are live at NMI, but SNES
    /// HDMA channel initialization for that field has already happened; the
    /// authored program therefore begins with the following field.
    pub(super) fn spotlight_internal_after_table_during_active_field(&mut self) {
        // $F2F2-$F2FA: LDA #$80, STA $9B, LDA #$0F, STA $13, RTL.
        crate::cycle_ledger::charge(124);
        self.set_hdma_enable_mask(0x80);
        self.set_screen_brightness(0x0f);
        self.stage_spotlight_scanout_after_active_field();
    }

    /// Complete the ROM suffix after an interrupted table build and retain the
    /// exact channel-7 rows measured for the field after the interrupting one.
    pub(super) fn spotlight_internal_after_table_during_active_rom_field(
        &mut self,
        words: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) {
        // $F2F2-$F2FA: LDA #$80, STA $9B, LDA #$0F, STA $13, RTL.
        crate::cycle_ledger::charge(124);
        self.set_hdma_enable_mask(0x80);
        self.set_screen_brightness(0x0f);
        self.stage_rom_spotlight_scanout_after_active_field(words);
    }

    /// ROM: `IrisSpotlight_ConfigureTable`
    pub(super) fn iris_spotlight_configure_table(&mut self) -> bool {
        // Cycle ledger: IrisSpotlight_ConfigureTable ($00:F312) is charged by
        // its pieces, each under its own 0x00_f312 scope, so an interrupted
        // build split across hosts records what each host ran: `begin_`
        // (prologue and row loop), `complete_..._table_projection` (rest of
        // the loop, V-counter wait, table copy), `..._after_projection`
        // (radius step, goal test, inline goal transition) and the deferred
        // goal-transition wrappers. A resumed build's replayed prefix is not
        // priced (`price == false`).
        let build = self.begin_iris_spotlight_configure_table(usize::MAX);
        self.complete_iris_spotlight_configure_table(build)
    }

    pub(super) fn begin_iris_spotlight_configure_table(
        &mut self,
        max_iterations: usize,
    ) -> SpotlightTableBuildContinuation {
        let _scope = crate::cycle_ledger::routine(0x00_f312);
        self.begin_iris_spotlight_configure_table_inner(max_iterations, true)
    }

    /// Rebuild the source loop's lower cursor from translated spotlight
    /// inputs and the number of completed row pairs. The retained CPU's PC
    /// selects a statement boundary; its private WRAM never supplies this
    /// translated cursor.
    pub(super) fn spotlight_lower_cursor_after_completed_rows(&self, rows: u16) -> u16 {
        let center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        center.wrapping_mul(2).max(224).wrapping_sub(rows)
    }

    /// The C prologue and up to `max_iterations` row-loop iterations. With
    /// `price == false` the work is a replay of instructions the ROM executed
    /// before an interruption and charges nothing.
    fn begin_iris_spotlight_configure_table_inner(
        &mut self,
        max_iterations: usize,
        price: bool,
    ) -> SpotlightTableBuildContinuation {
        let charge = |master: u64| {
            if price {
                crate::cycle_ledger::charge(master);
            }
        };
        // $F312-$F34C: PHB/PHK/PLB, REP #$30, the vertical centre, y_lower/
        // y_upper, x centre, spotlight_var4 = var1, r6 = r14 * 2, CMP #$00E0,
        // BCS not taken.
        charge(816);
        let r14 = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let radius = self.game_state.display.spotlight_hdma.window_radius();
        self.set_spotlight_y_lower(r14.wrapping_sub(radius));
        self.set_spotlight_y_upper(r14.wrapping_add(radius));
        let x_center = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
            .wrapping_add(8);
        self.set_spotlight_window_x_center(x_center);
        self.set_spotlight_window_y_buffer(radius);

        let mut r6 = r14.wrapping_mul(2);
        if r6 < 224 {
            // $F34E-$F351: LDA #$00E0, STA $06.
            charge(56);
            r6 = 224;
        } else {
            // BCS $F353 taken.
            charge(6);
        }
        // $F353-$F35F: r4 = r14 - (r6 - r14).
        charge(220);
        let r4 = r14.wrapping_mul(2).wrapping_sub(r6);
        let mut build = SpotlightTableBuildContinuation {
            vertical_center: r14,
            upper_cursor: r4,
            lower_cursor: r6,
            completed: false,
            pending_circle_input: None,
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: false,
            projection_words_copied: 0,
            source_progress: None,
        };
        self.advance_iris_spotlight_configure_table_priced(&mut build, max_iterations, price);
        build
    }

    /// Recreate the exact C table-builder state exposed by a semantic timing
    /// receipt. The current iteration has already loaded its circle input and
    /// conditionally decremented `spotlight_var4`, but has not yet calculated
    /// or stored either table word.
    pub(super) fn begin_iris_spotlight_configure_table_at_progress(
        &mut self,
        progress: crate::SpotlightTableBuildProgress,
    ) -> SpotlightTableBuildContinuation {
        // Cycle ledger: the ROM resumes IrisSpotlight_ConfigureTable at the
        // receipt's checkpoint on this host. The prefix rebuilt below (and
        // each arm's recomputation of the interrupted iteration) replays
        // instructions the ROM ran before the interruption, so it is not
        // priced; only the instructions from the resume point onward are.
        let _scope = crate::cycle_ledger::routine(0x00_f312);
        let mut build = self.begin_iris_spotlight_configure_table_inner(
            usize::from(progress.completed_iterations),
            false,
        );
        match progress.checkpoint {
            crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization => {
                assert!(
                    !build.completed,
                    "spotlight iteration-start progress cannot point beyond the completed C table loop",
                );
            }
            crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input,
            } => {
                assert!(
                    !build.completed,
                    "spotlight circle progress cannot point beyond the completed C table loop",
                );
                assert!(
                    build.lower_cursor < self.game_state.display.spotlight_hdma.y_upper(),
                    "pending spotlight circle calculation requires the lower cursor inside the iris",
                );
                let native_circle_input = self.game_state.display.spotlight_hdma.window_y_buffer_byte();
                assert_eq!(
                    native_circle_input,
                    pending_circle_input,
                    "spotlight receipt circle input disagrees with native C state",
                );
                if self.game_state.display.spotlight_hdma.window_y_buffer() != 0 {
                    self.decrement_spotlight_window_y_buffer();
                }
                build.pending_circle_input = Some(native_circle_input);
            }
            crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                lower_cursor,
                circle_value,
            } => {
                assert!(
                    !build.completed,
                    "spotlight lower-write progress cannot point beyond the completed C table loop",
                );
                assert_eq!(
                    build.lower_cursor, lower_cursor,
                    "spotlight lower-write receipt disagrees with the native C loop cursor",
                );
                let pending_circle_input = self
                    .game_state
                    .display
                    .spotlight_hdma
                    .window_y_buffer_byte();
                if pending_circle_input != 0 {
                    self.decrement_spotlight_window_y_buffer();
                }
                assert_eq!(
                    self.iris_spotlight_calculate_circle_value_priced(pending_circle_input, false),
                    circle_value,
                    "spotlight lower-write receipt disagrees with the native circle value",
                );
                if build.upper_cursor < 224 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        build.upper_cursor as usize,
                        circle_value,
                    );
                }
                build.pending_lower_value = Some(circle_value);
            }
            crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite { lower_cursor }
            | crate::SpotlightTableBuildCheckpoint::BeforeLowerTableStore { lower_cursor } => {
                assert!(
                    !build.completed,
                    "spotlight upper-write progress cannot point beyond the completed C table loop",
                );
                assert_eq!(
                    build.lower_cursor, lower_cursor,
                    "spotlight upper-write receipt disagrees with the native C loop cursor",
                );
                let mut circle_value = 0x00ff;
                if build.lower_cursor < self.game_state.display.spotlight_hdma.y_upper() {
                    let pending_circle_input = self
                        .game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte();
                    if pending_circle_input != 0 {
                        self.decrement_spotlight_window_y_buffer();
                    }
                    circle_value = self.iris_spotlight_calculate_circle_value_priced(pending_circle_input, false);
                }
                if build.upper_cursor < 224 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        build.upper_cursor as usize,
                        circle_value,
                    );
                }
                // The ROM resumes at $F387 (LDA $06, ASL, CMP #$01C0, BCS):
                // the work above replays the interrupted iteration's earlier
                // instructions and is not charged again here.
                let store_only = matches!(progress.checkpoint,
                    crate::SpotlightTableBuildCheckpoint::BeforeLowerTableStore { .. });
                if !store_only {
                    crate::cycle_ledger::charge(86);
                } else {
                    assert!(build.lower_cursor < 224, "lower table store requires an in-range row");
                }
                if build.lower_cursor < 224 {
                    build.pending_lower_value = Some(circle_value);
                } else {
                    // BCS $F396 taken; `advance_` charges the loop test.
                    crate::cycle_ledger::charge(6);
                    build.pending_loop_completion_test = true;
                }
            }
            crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words } => {
                assert!(
                    build.completed,
                    "spotlight projection progress requires the completed C table loop",
                );
                assert!(
                    copied_words <= 224,
                    "spotlight projection progress exceeds the 224-word C memcpy",
                );
                self.clear_spotlight_hdma_table_dynamic_range(224, 16);
                self.project_spotlight_dynamic_hdma_table_range_to_reserved(
                    0,
                    usize::from(copied_words),
                );
                build.projection_tail_cleared = true;
                build.projection_words_copied = copied_words;
            }
            crate::SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                upper_cursor,
                lower_cursor,
            } => {
                assert!(
                    !build.completed,
                    "spotlight loop-test progress cannot point beyond the completed C table loop",
                );
                assert_eq!(
                    (build.upper_cursor, build.lower_cursor),
                    (upper_cursor, lower_cursor),
                    "spotlight loop-test receipt disagrees with the native C loop cursors",
                );
                let mut circle_value = 0x00ff;
                if build.lower_cursor < self.game_state.display.spotlight_hdma.y_upper() {
                    let pending_circle_input = self
                        .game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte();
                    if pending_circle_input != 0 {
                        self.decrement_spotlight_window_y_buffer();
                    }
                    circle_value = self.iris_spotlight_calculate_circle_value_priced(pending_circle_input, false);
                }
                if build.upper_cursor < 224 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        build.upper_cursor as usize,
                        circle_value,
                    );
                }
                if build.lower_cursor < 224 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        build.lower_cursor as usize,
                        circle_value,
                    );
                }
                build.pending_loop_completion_test = true;
            }
            crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                upper_cursor,
                lower_cursor,
            } => {
                assert!(
                    !build.completed,
                    "spotlight lower-cursor progress cannot point beyond the completed C table loop",
                );
                assert_eq!(
                    (build.upper_cursor.wrapping_add(1), build.lower_cursor),
                    (upper_cursor, lower_cursor),
                    "spotlight lower-cursor receipt disagrees with the native C loop cursors",
                );
                assert_ne!(
                    build.upper_cursor, build.vertical_center,
                    "the source increments its upper cursor only after a false loop-completion test",
                );
                let mut circle_value = 0x00ff;
                if build.lower_cursor < self.game_state.display.spotlight_hdma.y_upper() {
                    let pending_circle_input = self
                        .game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte();
                    if pending_circle_input != 0 {
                        self.decrement_spotlight_window_y_buffer();
                    }
                    circle_value = self.iris_spotlight_calculate_circle_value_priced(pending_circle_input, false);
                }
                if build.upper_cursor < 224 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        build.upper_cursor as usize,
                        circle_value,
                    );
                }
                if build.lower_cursor < 224 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        build.lower_cursor as usize,
                        circle_value,
                    );
                }
                build.upper_cursor = upper_cursor;
                build.pending_lower_cursor_decrement = true;
            }
        }
        build.source_progress = Some(progress);
        build
    }

    fn advance_iris_spotlight_configure_table(
        &mut self,
        build: &mut SpotlightTableBuildContinuation,
        max_iterations: usize,
    ) {
        self.advance_iris_spotlight_configure_table_priced(build, max_iterations, true);
    }

    fn advance_iris_spotlight_configure_table_priced(
        &mut self,
        build: &mut SpotlightTableBuildContinuation,
        max_iterations: usize,
        price: bool,
    ) {
        // Cycle ledger: the $F361-$F3A0 scanline loop, one iteration per
        // pass. The pending_* resumptions charge only the instructions from
        // the interrupted iteration's resume point onward. `price == false`
        // replays iterations the ROM completed before an interruption.
        let charge = |master: u64| {
            if price {
                crate::cycle_ledger::charge(master);
            }
        };
        for _ in 0..max_iterations {
            if build.completed {
                return;
            }
            if build.pending_lower_cursor_decrement {
                // Resume at $F39E: DEC $06, JMP $F361.
                charge(78);
                build.pending_lower_cursor_decrement = false;
                build.lower_cursor = build.lower_cursor.wrapping_sub(1);
                continue;
            }
            if build.pending_loop_completion_test {
                // Resume at $F396: LDA $0E, CMP $04, BEQ.
                charge(80);
                build.pending_loop_completion_test = false;
                if build.upper_cursor == build.vertical_center {
                    // BEQ $F3A3 taken.
                    charge(6);
                    build.completed = true;
                    return;
                }
                // $F39C-$F3A0: INC $04, DEC $06, JMP $F361.
                charge(132);
                build.upper_cursor = build.upper_cursor.wrapping_add(1);
                build.lower_cursor = build.lower_cursor.wrapping_sub(1);
                continue;
            }
            if let Some(r8) = build.pending_lower_value.take() {
                // Resume at $F38F after the lower-cursor test passed: TAX,
                // LDA $08, STA $7F7000,X. (Both producers of this pending
                // value have already established lower_cursor < 224.)
                if build.lower_cursor < 224 {
                    charge(if matches!(build.source_progress.map(|p| p.checkpoint),
                        Some(crate::SpotlightTableBuildCheckpoint::BeforeLowerTableStore { .. })) {
                        48 // $F392 STA long,X; TAX and LDA retired before the interruption.
                    } else { 94 });
                    self.set_spotlight_hdma_table_dynamic_entry(build.lower_cursor as usize, r8);
                }
                // $F396-$F39A: LDA $0E, CMP $04, BEQ.
                charge(80);
                if build.upper_cursor == build.vertical_center {
                    // BEQ $F3A3 taken.
                    charge(6);
                    build.completed = true;
                    return;
                }
                // $F39C-$F3A0: INC $04, DEC $06, JMP $F361.
                charge(132);
                build.upper_cursor = build.upper_cursor.wrapping_add(1);
                build.lower_cursor = build.lower_cursor.wrapping_sub(1);
                continue;
            }
            let mut r8 = 0x00ff;
            if let Some(t) = build.pending_circle_input.take() {
                // Resume at $F375: JSR IrisSpotlight_CalculateCircleValue
                // (the callee charges itself).
                charge(46);
                r8 = self.iris_spotlight_calculate_circle_value_priced(t, price);
            } else if build.lower_cursor < self.game_state.display.spotlight_hdma.y_upper() {
                // $F361-$F36B: LDA #$00FF, STA $08, LDA $06, CMP $676, BCS not
                // taken; $F36D-$F370: LDA $67A, BEQ.
                charge(144 + 56);
                let t = self
                    .game_state
                    .display
                    .spotlight_hdma
                    .window_y_buffer_byte();
                if self.game_state.display.spotlight_hdma.window_y_buffer() != 0 {
                    // $F372: DEC $67A.
                    charge(62);
                    self.decrement_spotlight_window_y_buffer();
                } else {
                    // BEQ $F375 taken.
                    charge(6);
                }
                // $F375: JSR IrisSpotlight_CalculateCircleValue (the callee
                // charges itself).
                charge(46);
                r8 = self.iris_spotlight_calculate_circle_value_priced(t, price);
            } else {
                // $F361-$F36B with BCS $F378 taken: the scanline is outside
                // the iris, r8 stays $00FF.
                charge(144 + 6);
            }
            // $F378-$F37E: LDA $04, ASL, CMP #$01C0, BCS.
            charge(86);
            if build.upper_cursor < 224 {
                // $F380-$F383: TAX, LDA $08, STA $7F7000,X.
                charge(94);
                self.set_spotlight_hdma_table_dynamic_entry(build.upper_cursor as usize, r8);
            } else {
                // BCS $F387 taken.
                charge(6);
            }
            // $F387-$F38D: LDA $06, ASL, CMP #$01C0, BCS.
            charge(86);
            if build.lower_cursor < 224 {
                // $F38F-$F392: TAX, LDA $08, STA $7F7000,X.
                charge(94);
                self.set_spotlight_hdma_table_dynamic_entry(build.lower_cursor as usize, r8);
            } else {
                // BCS $F396 taken.
                charge(6);
            }
            // $F396-$F39A: LDA $0E, CMP $04, BEQ.
            charge(80);
            if build.upper_cursor == build.vertical_center {
                // BEQ $F3A3 taken.
                charge(6);
                build.completed = true;
                return;
            }
            // $F39C-$F3A0: INC $04, DEC $06, JMP $F361.
            charge(132);
            build.upper_cursor = build.upper_cursor.wrapping_add(1);
            build.lower_cursor = build.lower_cursor.wrapping_sub(1);
        }
    }

    pub(super) fn complete_iris_spotlight_configure_table(
        &mut self,
        build: SpotlightTableBuildContinuation,
    ) -> bool {
        self.complete_iris_spotlight_table_projection(build);
        self.complete_iris_spotlight_configure_table_after_projection()
    }

    /// Finish the C routine through the 448-byte table copy. Some short
    /// spotlight iterations reach this boundary before vblank even though the
    /// radius/control suffix and caller return remain interrupted afterward.
    pub(super) fn complete_iris_spotlight_table_projection(
        &mut self,
        mut build: SpotlightTableBuildContinuation,
    ) {
        let _scope = crate::cycle_ledger::routine(0x00_f312);
        self.advance_iris_spotlight_configure_table(&mut build, usize::MAX);
        debug_assert!(build.completed);

        if !build.projection_tail_cleared {
            // $F3A3-$F3B2: the V-counter wait (LDA $2137, LDA $213F, LDA
            // $213D, AND #$00FF, CMP #$00C0, BCC) is 172 per pass with the
            // system data bank, 178 when BCC loops. The ROM spins here until
            // scanline 192; that count depends on the raster phase at entry
            // (1 to ~950 passes in the shadow profiles), which the ledger
            // cannot see, so only the final pass is charged
            // (`SPOTLIGHT_VCOUNTER_WAIT_FINAL_PASS_MASTER_CYCLES`): the
            // remaining 178 per extra pass is the raster-dependent residual a
            // timing owner has to supply.
            // $00:F3AC AND #$00FF: wait for (V & 255) >= 192, including
            // the next field when entered during physical lines 256..261.
            // $00:F3A3 LDA $2137 latches after three 8-clock ROM fetches.
            self.native_wait_until_line(192, 0xff, 24, 178);
            crate::cycle_ledger::charge(SPOTLIGHT_VCOUNTER_WAIT_FINAL_PASS_MASTER_CYCLES);
            // $F3B4: LDX #$0000.
            crate::cycle_ledger::charge(24);
            // The C-only clear of words 224..240 has no ROM counterpart.
            self.clear_spotlight_hdma_table_dynamic_range(224, 16);
        }
        let copied_words = usize::from(build.projection_words_copied);
        assert!(
            copied_words <= 224,
            "a saved ProjectionCopy continuation cannot exceed the 224-word C memcpy",
        );
        // $F3B7-$F3C3: the 224-word copy to $1B00, 162 per word (LDA
        // $7F7000,X; STA $1B00,X; INX; INX; CPX #$01C0; BCC) plus 6 for each
        // taken BCC (all but the last word). A ProjectionCopy resume charges
        // only the words still to copy.
        let remaining_words = (224 - copied_words) as u64;
        if remaining_words != 0 {
            crate::cycle_ledger::charge(remaining_words * 162 + (remaining_words - 1) * 6);
        }
        self.project_spotlight_dynamic_hdma_table_range_to_reserved(
            copied_words,
            224 - copied_words,
        );
    }

    /// Resume the C routine immediately after its table copy.
    pub(super) fn complete_iris_spotlight_configure_table_after_projection(&mut self) -> bool {
        self.complete_iris_spotlight_configure_table_after_projection_deferring_goal(false)
    }

    /// Resume the C routine after its table copy. With `defer_goal_transition`
    /// the radius advances and the goal test runs, but a reached goal leaves
    /// the transition (table reset or force blank, module writes, music) to
    /// the caller, which owns an interruption inside it.
    pub(super) fn complete_iris_spotlight_configure_table_after_projection_deferring_goal(
        &mut self,
        defer_goal_transition: bool,
    ) -> bool {
        let _scope = crate::cycle_ledger::routine(0x00_f312);
        // $F3C5-$F3D5: LDX $67E, LDA $67C, CLC, ADC $F302,X, STA $67C, CMP
        // $F30A,X, BNE.
        crate::cycle_ledger::charge(242);
        let idx = (self.game_state.display.spotlight_hdma.window_state() >> 1) as usize;
        let delta = SPOTLIGHT_DELTA_SIZE[idx] as i16 as u16;
        let next = self
            .game_state
            .display
            .spotlight_hdma
            .window_radius()
            .wrapping_add(delta);
        self.set_spotlight_window_radius(next);
        if next != SPOTLIGHT_GOAL[idx] {
            // BNE $F423 taken, then $F423-$F426: SEP #$30, PLB, RTL.
            crate::cycle_ledger::charge(6 + 94);
            return false;
        }
        if defer_goal_transition {
            return true;
        }
        if self.rom_startup_timing()
            && rom_dungeon_landing_goal_transition_waits_for_caller_return(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            )
        {
            self.dungeon_landing_goal_transition_pending = true;
        } else {
            self.complete_iris_spotlight_goal_transition();
        }
        true
    }

    /// A goal transition the caller deferred past the C return: the ROM is
    /// still inside IrisSpotlight_ConfigureTable, so it runs under its own
    /// 0x00_f312 scope (the inline path is covered by the after-projection
    /// scope).
    pub(super) fn complete_deferred_iris_spotlight_goal_transition(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_f312);
        self.complete_iris_spotlight_goal_transition();
    }

    /// The goal transition's tail resumed after vblank interrupted
    /// IrisSpotlight_ResetTable, under its own 0x00_f312 scope.
    pub(super) fn complete_deferred_iris_spotlight_goal_transition_after_reset(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_f312);
        self.complete_iris_spotlight_goal_transition_after_reset();
    }

    pub(super) fn complete_iris_spotlight_goal_transition(&mut self) {
        // $F3D7-$F3DC: SEP #$20, LDA $67E, BNE.
        crate::cycle_ledger::charge(70);
        if self.game_state.display.spotlight_hdma.window_state() == 0 {
            // $F3DE-$F3E5: LDA #$80, STA $13, STA $2100 (system data bank,
            // 30), BRA $F3EB taken (28).
            crate::cycle_ledger::charge(98);
            self.set_screen_brightness(0x80);
        } else {
            // BNE $F3E7 taken, then JSL IrisSpotlight_ResetTable (the callee
            // charges itself when annotated).
            crate::cycle_ledger::charge(6 + 62);
            self.iris_spotlight_reset_table();
        }
        self.complete_iris_spotlight_goal_transition_after_reset();
    }

    /// The goal transition's tail after the force-blank/table-reset branch.
    pub(super) fn complete_iris_spotlight_goal_transition_after_reset(&mut self) {
        // $F3EB-$F3F5: SEP #$30, STZ $B0, STZ $11, LDA $10, CMP #$07, BEQ.
        crate::cycle_ledger::charge(126);
        self.set_subsubmodule(0);
        self.set_submodule(0);
        let main_module = self.game_state.frame.main_module;
        // The ROM tests module 7 (BEQ $F3FB taken) before module $10
        // ($F3F7-$F3F9: CMP #$10, BNE $F416); charge the outcome the single
        // Rust condition merges.
        if main_module == 7 {
            crate::cycle_ledger::charge(6);
        } else if main_module == 16 {
            crate::cycle_ledger::charge(32);
        } else {
            crate::cycle_ledger::charge(32 + 6);
        }
        if main_module == 7 || main_module == 16 {
            // $F3FB-$F3FD: LDA $1B, BNE.
            crate::cycle_ledger::charge(40);
            if self.game_state.world.location.is_outdoors() {
                // $F3FF-$F409: LDX $8A, LDA $7F5B00,X, four LSRs, STA $12D.
                crate::cycle_ledger::charge(152);
                let ambient = self.overworld_config_table().current_music() >> 4;
                self.set_ambient_sound_effect(ambient);
            } else {
                // BNE $F40C taken.
                crate::cycle_ledger::charge(6);
            }
            // $F40C-$F411: LDA $132, CMP #$FF, BEQ.
            crate::cycle_ledger::charge(64);
            if self.game_state.system_signals.queued_music_control() != 0xff {
                // $F413: STA $12C.
                crate::cycle_ledger::charge(32);
                let music = self.game_state.system_signals.queued_music_control();
                self.set_music_control(music);
            } else {
                // BEQ $F416 taken.
                crate::cycle_ledger::charge(6);
            }
        }
        // $F416-$F41D: LDA $10C, STA $10, CMP #$06, BNE.
        crate::cycle_ledger::charge(88);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        if self.game_state.frame.main_module == 6 {
            // $F41F: JSL Sprite_ResetAll (the callee charges itself when
            // annotated).
            crate::cycle_ledger::charge(62);
            self.sprite_reset_all();
        } else {
            // BNE $F423 taken.
            crate::cycle_ledger::charge(6);
        }
        // $F423-$F426: SEP #$30, PLB, RTL.
        crate::cycle_ledger::charge(94);
    }

    /// `IrisSpotlight_CalculateCircleValue`; with `price == false` the call
    /// is a replay or a display derivation the ROM does not execute, and
    /// nothing is charged.
    pub(super) fn iris_spotlight_calculate_circle_value_priced(&self, a: u8, price: bool) -> u16 {
        // Cycle ledger: IrisSpotlight_CalculateCircleValue ($00:F4CC), entered
        // with m16/x16 and the system data bank ($00), so the seven $42xx
        // divider/multiplier accesses cost 2 less per byte.
        let _scope = price.then(|| crate::cycle_ledger::routine(0x00_f4cc));
        let charge = |master: u64| {
            if price {
                crate::cycle_ledger::charge(master);
            }
        };
        // $F4CC-$F508: SEP #$30, the $4204-$4206 divide, six NOPs, REP #$20,
        // LDA $4214, LSR, SEP #$20, TAX, LDY $F44B,X, the $4202/$4203
        // multiply, two NOPs, STZ $01, STZ $0B, LDA $4217, STA $00, REP #$30,
        // ASL $00, LDA $0A, BEQ: 754 - 16 = 738.
        charge(738);
        let radius = self.game_state.display.spotlight_hdma.window_radius() as u8;
        let div = if radius == 0 {
            0xffff
        } else {
            ((a as u16) << 8) / radius as u16
        };
        let t = (div >> 1) as usize;
        // LDY $F44B,X with an 8-bit X costs 6 more when $F44B + X crosses
        // into page $F5 (X >= $B5); X is the low byte of the quotient / 2.
        if (t & 0xff) + 0x4b > 0xff {
            charge(6);
        }
        let r10 = SPOTLIGHT_CIRCLE_X_RADIUS_CURVE[t];
        let p = 2 * ((((r10 as u16) * (radius as u16)) >> 8) as u8 as u16);
        if r10 == 0 {
            // BEQ $F53D taken, then RTS.
            charge(6 + 42);
            return 0x00ff;
        }
        // $F50A-$F51A: r2 = x_center + p, r0 = x_center - p, STZ $00, BMI.
        charge(252);
        let x_center = self.game_state.display.spotlight_hdma.window_x_center();
        let r2 = x_center.wrapping_add(p).min(255);
        let r0_raw = x_center.wrapping_sub(p);
        let r0 = if (r0_raw as i16) < 0 {
            // BMI $F526 taken: r0 stays the zero just stored.
            charge(6);
            0
        } else {
            // $F51C-$F51F: BIT #$FF00, BEQ; a high byte of zero takes the
            // branch to STA $00, otherwise $F521 loads #$00FF first.
            if r0_raw < 256 {
                charge(40 + 6);
            } else {
                charge(40 + 24);
            }
            // $F524: STA $00.
            charge(32);
            r0_raw.min(255)
        };
        // $F526-$F52B: LDA $02, BIT #$FF00, BEQ.
        if x_center.wrapping_add(p) < 256 {
            charge(72 + 6);
        } else {
            // $F52D: LDA #$00FF.
            charge(72 + 24);
        }
        let result = r0 | (r2 << 8);
        // $F530-$F536: XBA, ORA $00, CMP #$FFFF, BNE.
        charge(92);
        if result == 0xffff {
            // $F538: LDA #$00FF; $F53B: STA $08; $F53D: RTS.
            charge(24 + 32 + 42);
            0x00ff
        } else {
            // BNE $F53B taken; STA $08; RTS.
            charge(6 + 32 + 42);
            result
        }
    }

    /// ROM: `IrisSpotlight_ResetTable`
    pub(super) fn iris_spotlight_reset_table(&mut self) {
        self.iris_spotlight_reset_table_stores(0, 224);
        self.complete_iris_spotlight_reset_table_tail();
    }

    /// The ROM's `IrisSpotlight_ResetTable` ($00:F427) clears the 224-word
    /// dynamic table as 32 iterations of seven interleaved stores: iteration
    /// `i` writes word `(31 - i)` of each 32-word stripe `k` (`k` in 0..7).
    /// Publish stores `start..end` of that source order so an NMI accepted
    /// mid-loop observes exactly the ROM's partial table.
    pub(super) fn iris_spotlight_reset_table_stores(&mut self, start: usize, end: usize) {
        debug_assert!(start <= end && end <= 224);
        for store in start..end {
            let iteration = store / 7;
            let stripe = store % 7;
            let index = stripe * 32 + (31 - iteration);
            self.set_spotlight_hdma_table_dynamic_entry(index, 0xff00);
        }
    }

    /// The source port also resets the off-screen words 224..240, which the
    /// ROM loop never reaches; keep the native table's established shape.
    pub(super) fn complete_iris_spotlight_reset_table_tail(&mut self) {
        for i in 224..240 {
            self.set_spotlight_hdma_table_dynamic_entry(i, 0xff00);
        }
    }

    /// ROM: `Overworld_CopyPalettesToCache`
    pub(super) fn overworld_copy_palettes_to_cache(&mut self) {
        // $02:C769: setup, 32 interleaved eight-word copies, loop branches,
        // and the final CGRAM publication/return. Partial-copy callers own
        // their instruction progress separately.
        let _scope = crate::cycle_ledger::routine(0x02_c769);
        crate::cycle_ledger::charge(38 + 32 * 828 + 31 * 6 + 102);
        self.advance_palette_cache_copy(0, 257);
    }

    /// The ROM interleaves eight 32-word palette columns. The final cursor
    /// step publishes the CGRAM update only after all 256 words are stored.
    pub(super) fn advance_palette_cache_copy(&mut self, start: u16, end: u16) {
        assert!(start <= end && end <= 257);
        for ordinal in start..end {
            if ordinal == 256 {
                self.increment_cgram_update_flag();
            } else {
                let index = usize::from(ordinal % 8) * 32 + usize::from(ordinal / 8);
                self.copy_color((Bank::Aux, index), (Bank::Main, index));
            }
        }
    }

    pub(super) fn advance_pre_dungeon_palette_cache(&mut self, target: u16) {
        let start = self.native_pre_dungeon_palette_cache.unwrap_or_else(|| {
            // Reaching the shared cache copy proves this caller's palette
            // loads are complete. Retire that native prefix exactly once.
            self.complete_module_pre_dungeon_before_palette_cache();
            0
        });
        self.advance_palette_cache_copy(start, target);
        self.native_pre_dungeon_palette_cache = Some(target);
    }

    pub(super) fn enable_force_blank(&mut self) {
        if self.game_state.display.is_hdma_channel_enabled(7) && self.dma.channel[7].b_adr == 0x26 {
            // HDMA has completed the prior field before this C call disables
            // channel 7. Its row-224 terminator is therefore the resident WH0/
            // WH1 hardware state used while the ensuing load stays blank.
            let terminal = read_le_u16(
                &self.ram,
                RESERVED_HDMA_TABLE + TRAILING_NMI_FORCE_BLANK_SCANLINE as usize * 2,
            );
            self.ppu.window1_left = terminal as u8;
            self.ppu.window1_right = (terminal >> 8) as u8;
        }
        self.set_screen_brightness(0x80);
        // C's EnableForceBlank is a semantic request for the next complete
        // field to be blank; WritePpuRegisters publishes its INIDISP_copy at
        // the immediately following NMI. Keep that provenance distinct from
        // other main-thread mirror assignments, such as the iris goal write
        // after the current field has already begun scanning out.
        self.active_display_force_blank_event = Some(0);
        self.clear_hdma_enable_mask();
    }

    // ----- Palette backdrop / OW background color helpers ---------------------
    // Port of zelda3/src/load_gfx.c:1820-1856 plus LoadGearPalette at 1913.

    pub(super) fn palette_bg_and_fixed_color_black(&mut self) {
        self.palette_set_bg_and_fixed_color(0);
    }

    /// ROM: `Palette_SetBgAndFixedColor`
    pub(super) fn palette_set_bg_and_fixed_color(&mut self, color: u16) {
        self.set_main_color_constant(0, color);
        self.set_main_color_constant(32, color);
        self.set_aux_color_constant(0, color);
        self.set_aux_color_constant(32, color);
        self.set_backdropcolor_black();
    }

    pub(super) fn set_backdropcolor_black(&mut self) {
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
    }

    /// ROM: `Palette_GetOwBgColor`
    pub(super) fn palette_get_ow_bg_color(&self) -> u16 {
        let ow_screen = u16::from(self.game_state.world.location.overworld_screen_index());
        if ow_screen < 0x80 {
            return if ow_screen & 0x40 != 0 {
                0x2A32
            } else {
                0x2669
            };
        }
        let room = self.game_state.world.location.dungeon_room();
        if room == 0x180 || room == 0x182 || room == 0x183 {
            return 0x19C6;
        }
        0x2669
    }

    pub(super) fn palette_assert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(true);
    }

    /// ROM: `Palette_SetTranslucencySwap`
    pub(super) fn palette_set_translucency_swap(&mut self, v: bool) {
        self.follower_state_mut()
            .set_palette_swap_flag(if v { 1 } else { 0 });
        // Three banded swaps over 8 consecutive entries each: 0x80<->0xF0, 0x88<->0xF8, 0xB8<->0xD8.
        // For each pair, swap the two aux words and mirror each into main (the
        // original reads both aux words up front, then writes the swapped values
        // into both aux and main).
        for i in 0..8 {
            let pairs: [(usize, usize); 3] = [
                (0x80 + i, 0xf0 + i),
                (0x88 + i, 0xf8 + i),
                (0xb8 + i, 0xd8 + i),
            ];
            for (low, high) in pairs {
                self.swap_colors((Bank::Aux, low), (Bank::Aux, high));
                self.copy_color((Bank::Aux, low), (Bank::Main, low));
                self.copy_color((Bank::Aux, high), (Bank::Main, high));
            }
        }
        self.increment_cgram_update_flag();
    }

    /// ROM: `Palette_RevertTranslucencySwap`
    pub(super) fn palette_revert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(false);
    }
}

#[cfg(test)]
#[test]
fn marker38_palette_restore_flash_matches_rom() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
    let Ok(rom) = std::fs::read(path) else { return };
    for (indoors, screen) in [(1u8, 0u8), (0, 0), (0, 0x40), (0, 0x5b),
        (0, 3), (0, 5), (0, 7), (0, 0x43), (0, 0x45), (0, 0x47), (0, 1)] {
        let mut native = ZeldaState::new();
        native.write_wram_byte(0x1b, indoors); native.write_wram_byte(0x8a, screen);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let mut machine = snes::Snes::new(); machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        machine.ram.copy_from_slice(&native.wram_image());
        machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
        machine.cpu.e = false; machine.cpu.unpack_flags(0x30);
        machine.cpu.k = 0x0e; machine.cpu.db = 0x0e; machine.cpu.pc = 0xd83a; machine.cpu.sp = 0x1fc;
        crate::cycle_ledger::assert_rom_marker_work(0x0ed83a, machine,
            |s| s.cpu.k == 0x0e && (0xd83a..0xd8fb).contains(&s.cpu.pc), || {
                native.palette_restore_bg_from_flash();
            });
    }
}

#[cfg(test)]
#[test]
fn m3b_low_conversion_entries_and_widths_match_rom() {
    use crate::zelda_rtl::dungeon::m3a_fixtures::source_work;
    for animated in [false,true] {
        for num in [1,2,8,32] {
            for src in [0x2000,0x20f0] {
                let mut ram=ZeldaState::new().wram_image().into_owned();
                ram[2]=0x7e;ram[5]=0x7e;
                let entry=if animated {0x00d5ce} else {0x00dfb8};
                let expected=source_work(&ram,entry,false,false,false,src,0,num);
                let mut native=ZeldaState::new();native.set_lag_free(true);
                let before=crate::cycle_ledger::master();
                native.convert_low_buffer(0,&vec![0;0x1000],0,usize::from(num),animated);
                assert_eq!(crate::cycle_ledger::master()-before,expected);
                // The preserved scalar path selects the ordinary body.
                native.set_lag_free(false);
                let before=crate::cycle_ledger::master();
                native.convert_low_buffer(0,&vec![0;0x1000],0,usize::from(num),animated);
                assert_eq!(crate::cycle_ledger::master()-before,32+u64::from(num)*3736+u64::from(num-1)*6+42);
            }
        }
    }
}
