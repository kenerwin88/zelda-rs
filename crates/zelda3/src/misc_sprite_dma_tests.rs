use super::*;
use crate::cycle_models::sprite_dma::{Operand, SourceWords, Table};
use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};

fn original_rom() -> Option<Vec<u8>> {
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")
        });
    let mut rom = std::fs::read(path).ok()?;
    if rom.len() % 0x400 == 0x200 {
        rom.drain(..0x200);
    }
    Some(rom)
}

#[test]
fn sprite_dma_native_data_preserves_every_byte_and_fragment_boundary() {
    let Some(rom) = original_rom() else { return };
    for &(base, words) in LINK_DMA_TABLE_FRAGMENTS {
        for (i, &word) in words.iter().enumerate() {
            let address = usize::from(base) + i * 2;
            assert_eq!(
                word.to_le_bytes(),
                rom[address & 0x7fff..(address & 0x7fff) + 2],
                "{address:04x}"
            );
        }
    }
    // Read every aligned AND unaligned word in each complete native region.
    for (table, end) in [(Table::PushedBlock, 0x85fc), (Table::Body, 0x9888)] {
        for address in table.base()..end - 1 {
            let offset = usize::from(address & 0x7fff);
            assert_eq!(
                link_dma_table_word(table, address - table.base()),
                u16::from_le_bytes([rom[offset], rom[offset + 1]]),
                "{address:04x}"
            );
        }
    }
    // All eight-bit X values, including table overlap and page crossings.
    for table in [
        Table::Sword,
        Table::Shield,
        Table::Auxiliary,
        Table::PushedBlock,
    ] {
        for index in 0..=255 {
            let offset = usize::from((table.base() + index) & 0x7fff);
            assert_eq!(
                link_dma_table_word(table, index),
                u16::from_le_bytes([rom[offset], rom[offset + 1]])
            );
        }
    }
}

#[test]
fn sprite_dma_reader_rejects_missing_native_data() {
    for address in [0x8493, 0x85fc, 0x9395, 0x9888, 0x18494] {
        assert_eq!(link_dma_data_byte(address), None);
    }
    // A known low byte must not synthesize an unknown high byte.
    assert!(std::panic::catch_unwind(|| link_dma_table_word(Table::Hand, 53)).is_err());
    assert!(std::panic::catch_unwind(|| link_dma_table_word(Table::Body, u16::MAX)).is_err());
}

#[test]
fn sprite_dma_source_words_match_source_at_every_instruction() {
    let Some(rom) = original_rom() else { return };
    // Covers every usable graphics byte index through the adjoining head and
    // hand tables, all valid hand byte indexes, and all byte-valued equipment
    // and staging inputs. Adjacent high bytes deliberately remain nonzero.
    for fixture in 0..=658u16 {
        let mut state = ZeldaState::new();
        state.ram[0xac0..0xadc].fill(0xa5);
        for (address, value) in [
            (0x100, fixture),
            (0x102, fixture % 53),
            (0x104, (fixture * 37) % 53),
            (0x109, 0xa500 | u16::from((fixture as u8).rotate_left(3))),
            (0x2c3, 0xca00 | (fixture & 0xff)),
        ] {
            state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
        }
        state.ram[0x107] = fixture as u8;
        state.ram[0x108] = !(fixture as u8);
        state.sync_native_game_state_from_ram();
        let atomic_words = state.nmi_prepare_sprites_source_words();
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x00_865c,
            stop_pc: 0x00_86df,
            a: 0xabcd,
            x: 0x37,
            y: 0xfc,
            sp: 0x01fc,
            dp: 0,
            db: 0,
            carry: true,
            zero: false,
            overflow: true,
            negative: false,
            interrupt_disable: true,
            decimal: false,
            accumulator_is_8_bit: true,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0x01fd,
            stack_bytes: &[0x35, 0x80],
        };
        let mut source = RomCpuTimingRun::new(
            &rom,
            &state.wram_image(),
            &state.sram,
            &snes::PpuState::default(),
            &snes::DmaState::default(),
            [0; 4],
            checkpoint,
        )
        .unwrap();
        source.enable_cpu_write_trace();
        let mut expected = state.wram_image().into_owned();
        let mut work = SourceWords::default();
        let mut stores = Vec::new();
        let mut total = 0;
        for _ in 0..53 {
            work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
            let pc = source.pc();
            assert_eq!(work.next_pc(), pc);
            let instruction = state.step_sprite_dma_source_words(&mut work).unwrap();
            assert_eq!(
                instruction.master_cycles,
                source.step().master_cycles,
                "fixture={fixture} PC={pc:06x}"
            );
            assert_eq!(work.next_pc(), source.pc());
            if let Some((index, value)) = instruction.store {
                state
                    .display_core_mut()
                    .set_sprite_preparation_source_word(index, value);
                stores.push(value);
            }
            for (address, value) in source.take_cpu_wram_writes() {
                expected[address] = value;
            }
            assert_eq!(*state.wram_image(), expected, "fixture={fixture} PC={pc:06x}");
            total += instruction.master_cycles;
        }
        assert!(source.is_complete());
        assert_eq!(state.step_sprite_dma_source_words(&mut work), None);
        assert_eq!(stores, atomic_words);
        assert!((1642..=1660).contains(&total));
    }
}

#[test]
fn sprite_dma_continuation_retains_loaded_operands() {
    let mut work = SourceWords::default();
    let mut graphics = 1;
    let mut staging = 0x88;
    let mut sword = 0x55;
    let mut table_delta = 0;
    let mut words = [0; 14];
    while let Some(instruction) = work.step(
        |operand| match operand {
            Operand::Graphics => graphics,
            Operand::Staging => staging,
            Operand::Sword => sword,
            _ => 0,
        },
        |table, index| table.base().wrapping_add(index).wrapping_add(table_delta),
    ) {
        if let Some((index, value)) = instruction.store {
            words[index] = value;
        }
        match work.next_pc() {
            0x8661 => graphics = 0x100, // LDX retained the old byte offset
            0x8664 => table_delta = 7,  // body LDA retained the old table word
            0x8691 => sword = 0,        // retain eight-bit X across the table read
            0x86b1 => staging = 3,      // first LDA retained group bits from $88
            0x86ba => staging = 0x77,   // second LDA independently retained 3
            _ => {}
        }
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
    }
    assert_eq!(words[0], 0x9397);
    assert_eq!(words[1], 0x9597);
    assert_eq!(words[2], 0x95fc); // retained X, newly read table
    assert_eq!(words[6], 0x84f8); // $849c + $55 + 7
    assert_eq!(words[10], 0x84bf); // second staging load: 3 * 2
    assert_eq!(words[11], 0x0a9a); // $84bf + ($85b2 + $22 + 7), wrapping
}

#[test]
fn sprite_dma_production_call_matches_source_state_and_cost() {
    let Some(rom) = original_rom() else { return };
    for seed in 0..96u16 {
        let mut state = ZeldaState::new();
        for (i, byte) in state.ram[0xa20..0xaa0].iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(37).wrapping_add(seed as u8);
        }
        for (address, value) in [
            (0x100, seed * 5),
            (0x102, seed % 53),
            (0x104, (seed * 7) % 53),
            (0xc00d, seed % 3),
            (0xc00f, (seed % 3) * 0x400),
            (0xc013, (seed / 3) % 3),
            (0xc015, (seed % 6) * 2),
        ] {
            state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
        }
        state.ram[0x8c] = [0, 0xb5, 0xbc][usize::from(seed % 3)];
        state.ram[0x107] = (seed * 7) as u8;
        state.ram[0x108] = (seed * 11) as u8;
        state.ram[0x109] = (seed * 13) as u8;
        state.ram[0x2c3] = seed as u8;
        state.ram[0xae8] = (seed * 17) as u8;
        state.ram[0xaea] = (seed * 19) as u8;
        state.ram[0xaf4] = (seed * 23) as u8;
        state.sync_native_game_state_from_ram();
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x00_85fc,
            stop_pc: 0x00_8036,
            a: 0xabcd,
            x: 0x37,
            y: 0x42,
            sp: 0x01fc,
            dp: 0,
            db: 0,
            carry: true,
            zero: false,
            overflow: true,
            negative: false,
            interrupt_disable: true,
            decimal: false,
            accumulator_is_8_bit: true,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0x01fd,
            stack_bytes: &[0x35, 0x80],
        };
        let mut source = RomCpuTimingRun::new(
            &rom,
            &state.wram_image(),
            &state.sram,
            &snes::PpuState::default(),
            &snes::DmaState::default(),
            [0; 4],
            checkpoint,
        )
        .unwrap();
        source.enable_cpu_write_trace();
        let mut expected = state.wram_image().into_owned();
        let mut source_cost = 0;
        for _ in 0..1000 {
            if source.is_complete() {
                break;
            }
            source_cost += u64::from(source.step().master_cycles);
            for (address, value) in source.take_cpu_wram_writes() {
                expected[address] = value;
            }
        }
        assert!(source.is_complete());
        let before = crate::cycle_ledger::master();
        state.nmi_prepare_sprites();
        assert_eq!(
            crate::cycle_ledger::master() - before,
            source_cost,
            "seed={seed}"
        );
        assert_eq!(*state.wram_image(), expected, "seed={seed}");
    }
}

#[test]
fn sprite_dma_animations_match_source_at_every_instruction() {
    use crate::cycle_models::sprite_dma::Animations;
    let Some(rom) = original_rom() else { return };
    let mut cases = 0;
    let mut boundaries = 0;
    let mut visited = std::collections::BTreeSet::new();
    for background_countdown in [0u16, 1, 2, 0xffff] {
        for link_countdown in [0u16, 1, 0xffff] {
            for overlay in [0, 0xb5, 0xbc, 0xff] {
                for low in (0..=26u16).chain([254, 255]) {
                    for high in [0, 0xab00] {
                        let mut state = ZeldaState::new();
                        let background_offset =
                            [0u16, 0x400, 0x800, 0xfc00, 0xfbff, 0xffff][cases % 6];
                        for (address, value) in [
                            (0xc00d, background_countdown),
                            (0xc00f, background_offset),
                            (0xc013, link_countdown),
                            (0xc015, high | low),
                        ] {
                            state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
                        }
                        state.ram[0x8c] = overlay;
                        state.ram[0xadc..0xade].fill(0xa5);
                        state.ram[0xae0..0xae4].fill(0x5a);
                        state.sync_native_game_state_from_ram();
                        let mut atomic = state.clone();
                        let mut prepaid = state.clone();
                        let checkpoint = RomCpuCheckpoint {
                            entry_pc: 0x00_86df,
                            stop_pc: 0x00_874e,
                            a: 0xabcd,
                            x: 0x37,
                            y: 0x42,
                            sp: 0x01fc,
                            dp: 0,
                            db: 0,
                            carry: true,
                            zero: false,
                            overflow: true,
                            negative: false,
                            interrupt_disable: true,
                            decimal: false,
                            accumulator_is_8_bit: false,
                            index_is_8_bit: true,
                            emulation: false,
                            waiting: false,
                            stack_address: 0x01fd,
                            stack_bytes: &[0x35, 0x80],
                        };
                        let mut source = RomCpuTimingRun::new(
                            &rom,
                            &state.wram_image(),
                            &state.sram,
                            &snes::PpuState::default(),
                            &snes::DmaState::default(),
                            [0; 4],
                            checkpoint,
                        )
                        .unwrap();
                        source.enable_cpu_write_trace();
                        let mut expected = state.wram_image().into_owned();
                        let mut work = Animations::default();
                        let mut total = 0u64;
                        loop {
                            work =
                                bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                            let pc = source.pc();
                            assert_eq!(work.next_pc(), pc);
                            let Some(instruction) = state.step_sprite_dma_animations(&mut work)
                            else {
                                break;
                            };
                            visited.insert(pc);
                            assert_eq!(
                                instruction.master_cycles,
                                source.step().master_cycles,
                                "case={cases} PC={pc:06x}"
                            );
                            assert_eq!(work.next_pc(), source.pc());
                            for (address, value) in source.take_cpu_wram_writes() {
                                expected[address] = value;
                            }
                            assert_eq!(*state.wram_image(), expected, "case={cases} PC={pc:06x}");
                            total += u64::from(instruction.master_cycles);
                            boundaries += 1;
                        }
                        assert!(source.is_complete());
                        let before = crate::cycle_ledger::master();
                        atomic.nmi_prepare_sprites_animations(0);
                        assert_eq!(crate::cycle_ledger::master() - before, total);
                        assert_eq!(*atomic.wram_image(), *state.wram_image());
                        let before = crate::cycle_ledger::master();
                        prepaid.nmi_prepare_sprites_animations(126);
                        assert_eq!(crate::cycle_ledger::master() - before + 126, total);
                        assert_eq!(prepaid.ram, *state.wram_image());
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 2784);
    assert_eq!(visited.len(), 44);
    eprintln!("sprite DMA animation: {cases} cases, {boundaries} serialized source boundaries, all44 instructions visited");
}

#[test]
fn sprite_dma_animation_loads_and_branch_decisions_survive_shared_state_changes() {
    use crate::cycle_models::sprite_dma::{
        AnimationOperand as Read, AnimationStore as Store, Animations,
    };
    let mut state = ZeldaState::new();
    state.reset_bg_tile_animation_countdown(1);
    state.player_state_mut().set_link_dma_source_offset(0x800);
    state.player_state_mut().set_link_dma_countdown(1);
    state.player_state_mut().set_link_dma_tile_offset(0xabfe);
    let mut work = Animations::default();
    let mut stores = Vec::new();
    while let Some(instruction) = state.step_sprite_dma_animations(&mut work) {
        if let Some(store) = instruction.store {
            stores.push(store);
        }
        match work.next_pc() {
            0x86e3 => state.reset_bg_tile_animation_countdown(7), // after LDA
            0x86e8 => state.reset_bg_tile_animation_countdown(9), // after STA, before BNE
            0x8702 => state.player_state_mut().set_link_dma_source_offset(0x400),
            0x8709 => state.player_state_mut().set_link_dma_source_offset(0x1234),
            0x871d => state.player_state_mut().set_link_dma_countdown(7),
            0x8722 => state.player_state_mut().set_link_dma_countdown(9),
            0x8728 => state.player_state_mut().set_link_dma_tile_offset(10),
            0x8732 => state.player_state_mut().set_link_dma_tile_offset(4),
            _ => {}
        }
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
    }
    assert_eq!(
        stores,
        [
            (Store::BackgroundCountdown, 0),
            (Store::BackgroundCountdown, 9),
            (Store::BackgroundOffset, 0),
            (Store::BackgroundSource, 0xa680),
            (Store::LinkCountdown, 0),
            (Store::LinkOffset, 0),
            (Store::LinkCountdown, 14),
            (Store::LinkSourceUpper, 0xb280),
            (Store::LinkSourceLower, 0xb2e0),
        ]
    );
    // Overlay comparison is also latched independently of later input changes.
    let mut overlay = 0xb5;
    let mut work = Animations::default();
    let mut countdowns = Vec::new();
    while let Some(instruction) = work.step(
        |operand| match operand {
            Read::BackgroundCountdown => 1,
            Read::Overlay => overlay,
            _ => 0,
        },
        link_dma_table_word,
    ) {
        if let Some((Store::BackgroundCountdown, value)) = instruction.store {
            countdowns.push(value);
        }
        if work.next_pc() == 0x86ef {
            overlay = 0;
        }
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
    }
    assert_eq!(countdowns, [0, 0x17]);
}

#[test]
fn sprite_dma_pointer_words_match_source_for_every_word_value() {
    use crate::cycle_models::sprite_dma::PointerWords;
    let Some(rom) = original_rom() else { return };
    // Each of the three inputs visits all 65536 words, covering high-byte
    // retention, carry out of ASL, and carry between the two ADCs.
    for seed in 0..=u16::MAX {
        let mut state = ZeldaState::new();
        for (address, value) in [(0xae8, seed), (0xaea, seed.rotate_left(5)), (0xaf4, !seed)] {
            state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
        }
        state.sync_native_game_state_from_ram();
        let atomic_words = state.nmi_prepare_sprites_pointer_words();
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x00_874e,
            stop_pc: 0x00_8036,
            a: 0xabcd,
            x: 0x37,
            y: 0x42,
            sp: 0x01fc,
            dp: 0,
            db: 0,
            carry: true,
            zero: false,
            overflow: true,
            negative: false,
            interrupt_disable: true,
            decimal: false,
            accumulator_is_8_bit: false,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0x01fd,
            stack_bytes: &[0x35, 0x80],
        };
        let mut source = RomCpuTimingRun::new(
            &rom,
            &state.wram_image(),
            &state.sram,
            &snes::PpuState::default(),
            &snes::DmaState::default(),
            [0; 4],
            checkpoint,
        )
        .unwrap();
        source.enable_cpu_write_trace();
        let mut expected = state.wram_image().into_owned();
        let mut work = PointerWords::default();
        let mut stores = Vec::new();
        let mut total = 0;
        for _ in 0..20 {
            work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
            let pc = source.pc();
            assert_eq!(work.next_pc(), pc);
            let instruction = state.step_sprite_dma_pointer_words(&mut work).unwrap();
            assert_eq!(
                instruction.master_cycles,
                source.step().master_cycles,
                "seed={seed} PC={pc:06x}"
            );
            assert_eq!(work.next_pc(), source.pc());
            let mut native_writes = Vec::new();
            if let Some((index, value)) = instruction.store {
                state
                    .display_core_mut()
                    .set_sprite_preparation_pointer_word(index, value);
                let address = [0xaec, 0xaee, 0xaf0, 0xaf2, 0xaf6, 0xaf8][index];
                native_writes.extend([(address, value as u8), (address + 1, (value >> 8) as u8)]);
                stores.push(value);
            }
            let writes = source.take_cpu_wram_writes();
            assert_eq!(native_writes, writes, "seed={seed} PC={pc:06x}");
            for (address, value) in writes {
                expected[address] = value;
            }
            total += instruction.master_cycles;
        }
        assert_eq!(total, 610);
        assert!(source.is_complete());
        assert_eq!(state.step_sprite_dma_pointer_words(&mut work), None);
        assert_eq!(stores, atomic_words);
        assert_eq!(*state.wram_image(), expected, "seed={seed}");
    }
}

#[test]
fn sprite_dma_pointer_load_retains_word_and_carry_across_producer_updates() {
    use crate::cycle_models::sprite_dma::PointerWords;
    let mut state = ZeldaState::new();
    for (address, value) in [(0xae8, 0x8000u16), (0xaea, 0x2360), (0xaf4, 0xffff)] {
        state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
    state.sync_native_game_state_from_ram();
    let mut work = PointerWords::default();
    let mut stores = Vec::new();
    while let Some(instruction) = state.step_sprite_dma_pointer_words(&mut work) {
        if let Some(store) = instruction.store {
            stores.push(store);
        }
        match work.next_pc() {
            0x8751 => state.set_sprite_dma_head_pointer(0xff),
            0x8761 => state.set_sprite_dma_body_pointer(0),
            0x8771 => state.set_travel_bird_tile_offset(0),
            _ => {}
        }
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
    }
    assert_eq!(
        stores,
        [
            (0, 0xb941),
            (1, 0xbb41),
            (2, 0),
            (3, 0x201),
            (4, 0xb53f),
            (5, 0xb740)
        ]
    );
    assert_eq!(
        state.display_core_mut().sprite_preparation_pointer_word(0),
        0x80ff
    );
    assert_eq!(
        state.display_core_mut().sprite_preparation_pointer_word(1),
        0x2300
    );
    assert_eq!(
        state.display_core_mut().sprite_preparation_pointer_word(2),
        0xff00
    );
    assert!(!state.game_state.display.has_travel_bird_tile_upload());
}

#[test]
fn sprite_preparation_composed_call_returns_through_real_main_caller() {
    use crate::cycle_models::main_wait::{MainLoopCallerPhase, MainWait};
    let Some(rom) = original_rom() else { return };
    let mut boundaries = 0;
    for seed in 0..=255u16 {
        let mut state = ZeldaState::new();
        for (i, value) in state.ram[0xa20..0xaa0].iter_mut().enumerate() {
            *value = (i as u8).wrapping_mul(37).wrapping_add(seed as u8);
        }
        for (address, value) in [
            (0x100, seed * 2),
            (0x102, seed % 53),
            (0x104, (seed * 7) % 53),
            (0xc00d, seed % 3),
            (0xc00f, ((seed / 3) % 3) * 0x400),
            (0xc013, (seed / 9) % 3),
            (0xc015, 0xab00 | ((seed % 6) * 2)),
            (0xae8, seed * 257),
            (0xaea, (seed * 257).rotate_left(5)),
            (0xaf4, !(seed * 257)),
        ] {
            state.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
        }
        state.ram[0x8c] = [0, 0xb5, 0xbc][usize::from((seed / 27) % 3)];
        state.ram[0x107] = seed as u8;
        state.ram[0x108] = !(seed as u8);
        state.ram[0x109] = (seed as u8).rotate_left(3);
        state.ram[0x12] = 1;
        state.sync_native_game_state_from_ram();
        state.main_loop_caller_phase = MainLoopCallerPhase::ModuleRouting;
        let mut atomic = state.clone();
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x00_805a,
            stop_pc: 0x00_8038,
            a: 0xabcd,
            x: 0x37,
            y: 0x42,
            sp: 0x01fc,
            dp: 0,
            db: 0,
            carry: true,
            zero: false,
            overflow: true,
            negative: false,
            interrupt_disable: true,
            decimal: false,
            accumulator_is_8_bit: true,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0,
            stack_bytes: &[],
        };
        let mut source = RomCpuTimingRun::new(
            &rom,
            &state.wram_image(),
            &state.sram,
            &snes::PpuState::default(),
            &snes::DmaState::default(),
            [0; 4],
            checkpoint,
        )
        .unwrap();
        source.enable_cpu_write_trace();
        let mut expected = state.wram_image().into_owned();
        let before = crate::cycle_ledger::master();
        state.enter_main_loop_sprite_preparation();
        let mut total = u64::from(source.step().master_cycles);
        assert_eq!(total, 46);
        assert_eq!(crate::cycle_ledger::master() - before, total);
        assert_eq!(source.pc(), 0x85fc);
        // Rust's semantic call stack is not a WRAM replica of the 65816 stack.
        // Verify the source JSR's return bytes, then compare all callee and
        // caller data publications without injecting those bytes into native.
        assert_eq!(
            source.take_cpu_wram_writes(),
            [(0x1fc, 0x80), (0x1fb, 0x5c)]
        );
        let mut work = SpritePreparationWork::default();
        while !work.is_complete() {
            work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
            let pc = source.pc();
            assert_eq!(work.next_pc(), pc);
            let cycles = state.step_sprite_preparation_work(&mut work).unwrap();
            assert_eq!(
                cycles,
                source.step().master_cycles,
                "seed={seed} PC={pc:06x}"
            );
            assert_eq!(work.next_pc(), source.pc(), "seed={seed} PC={pc:06x}");
            for (address, value) in source.take_cpu_wram_writes() {
                expected[address] = value;
            }
            assert_eq!(*state.wram_image(), expected, "seed={seed} PC={pc:06x}");
            total += u64::from(cycles);
            boundaries += 1;
        }
        assert_eq!(source.pc(), 0x805d);
        assert_eq!(state.step_sprite_preparation_work(&mut work), None);
        for (pc, after, cost) in [(0x805d, 0x805f, 24), (0x805f, 0x8034, 22)] {
            state.main_loop_caller_phase =
                bincode::deserialize(&bincode::serialize(&state.main_loop_caller_phase).unwrap())
                    .unwrap();
            assert_eq!(source.pc(), pc);
            assert_eq!(state.step_main_loop_caller_return(), Some(cost));
            assert_eq!(source.step().master_cycles, cost);
            assert_eq!(source.pc(), after);
            for (address, value) in source.take_cpu_wram_writes() {
                expected[address] = value;
            }
            assert_eq!(*state.wram_image(), expected);
            total += u64::from(cost);
            boundaries += 1;
        }
        assert_eq!(state.main_loop_caller_phase, MainLoopCallerPhase::Waiting);
        assert_eq!(state.step_main_loop_caller_return(), None);
        assert_eq!(crate::cycle_ledger::master() - before, total);
        let before = crate::cycle_ledger::master();
        atomic.nmi_prepare_sprites_for_main_loop();
        atomic.finish_main_loop_caller_return();
        assert_eq!(crate::cycle_ledger::master() - before, total);
        assert_eq!(*atomic.wram_image(), *state.wram_image());
        let mut wait = MainWait::default();
        for _ in 0..2 {
            wait = bincode::deserialize(&bincode::serialize(&wait).unwrap()).unwrap();
            assert_eq!(source.pc(), wait.next_pc());
            let instruction = wait.step(state.ram[0x12]);
            assert_eq!(source.step().master_cycles, instruction.master_cycles);
            assert_eq!(source.pc(), wait.next_pc());
            boundaries += 1;
        }
        assert_eq!(wait, MainWait::LoadLatch);
    }
    eprintln!("composed sprite preparation: 256 calls, {boundaries} serialized body/return/wait boundaries, plus256 JSRs");
}

#[test]
fn main_caller_branch_preserves_an_nmi_latch_written_after_its_store() {
    use crate::cycle_models::main_wait::{MainLoopCallerPhase, MainWait};
    let mut state = ZeldaState::new();
    state.main_loop_caller_phase = MainLoopCallerPhase::SpritePreparation;
    state.latch_nmi_update();
    assert_eq!(state.step_main_loop_caller_return(), Some(24));
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state.main_loop_caller_phase,
        MainLoopCallerPhase::ReturnBranch
    );
    assert_eq!(
        state.paired_resume_cpu_boundary_blocker(),
        Some("active-main-loop-caller")
    );
    state.main_loop_caller_phase =
        bincode::deserialize(&bincode::serialize(&state.main_loop_caller_phase).unwrap()).unwrap();
    state.latch_nmi_update(); // NMI after STZ, before BRA
    assert_eq!(state.step_main_loop_caller_return(), Some(22));
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.step_main_loop_caller_return(), None);
    assert!(state.game_state.display.nmi_update_is_latched());
    let mut wait = MainWait::default();
    assert_eq!(wait.step(state.ram[0x12]).master_cycles, 24);
    wait = bincode::deserialize(&bincode::serialize(&wait).unwrap()).unwrap();
    state.clear_nmi_update_latch(); // preserve the value already loaded by LDA
    assert_eq!(wait.step(state.ram[0x12]).master_cycles, 16);
    assert_eq!(wait, MainWait::Complete);
}

#[test]
fn sprite_dma_live_source_words_match_rom_bus_registers_and_clock() {
    use crate::cycle_models::{intro_clear::Access, nmi_stack::NativeNmiContext};
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, Snes, SourceCpuBusAccessKind as Kind, SourcePpuReadState,
    };
    let Some(rom) = original_rom() else { return };
    for fixture in 0..=658u16 {
        for (v, h) in [(211, 90), (224, 1338), (261, 1350)] {
            for odd in [false, true] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram[0xac0..0xadc].fill(0xa5);
                for (address, value) in [
                    (0x100, fixture),
                    (0x102, fixture % 53),
                    (0x104, (fixture * 37) % 53),
                    (0x109, 0xa500 | u16::from((fixture as u8).rotate_left(3))),
                    (0x2c3, 0xca00 | (fixture & 255)),
                ] {
                    machine.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
                }
                machine.ram[0x107] = fixture as u8;
                machine.ram[0x108] = !(fixture as u8);
                let mut cpu = NativeNmiContext {
                    a: 0xab00 | (fixture & 255),
                    x: 0x37,
                    y: 0xfc,
                    d: 0,
                    db: 0,
                    bank: 0,
                    pc: 0x865c,
                    sp: 0x1fd,
                    p: 0x30 | ((fixture as u8) & 0xc7),
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(odd),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = SourceWords::default();
                let mut stores = 0;
                while !work.is_complete() {
                    let pc = cpu.pc;
                    let expected = source
                        .step()
                        .unwrap()
                        .accesses
                        .into_iter()
                        .filter_map(|a| {
                            if a.address >= 0x2000 && link_dma_data_byte(a.address).is_none() {
                                return None;
                            }
                            let (op, value) = match a.kind {
                                Kind::Read { value, width } => {
                                    (Access::Read(a.address, width), value)
                                }
                                Kind::Write { value, width } => {
                                    (Access::Write(a.address, value, width), 0)
                                }
                                _ => return None,
                            };
                            Some((a.timestamp.master_cycles(), op, value))
                        })
                        .collect::<Vec<_>>();
                    let mut actual = Vec::new();
                    work.step_clocked(&mut cpu, &mut clock, |at, op| {
                        let value = match op {
                            Access::Read(address, width) => {
                                let read = |a: u32| {
                                    if a < 0x2000 {
                                        ram[a as usize]
                                    } else {
                                        link_dma_data_byte(a).unwrap()
                                    }
                                };
                                if width == 1 {
                                    u16::from(read(address))
                                } else {
                                    u16::from_le_bytes([read(address), read(address + 1)])
                                }
                            }
                            Access::Write(address, value, width) => {
                                assert_eq!(width, 2);
                                ram[address as usize..address as usize + 2]
                                    .copy_from_slice(&value.to_le_bytes());
                                stores += 1;
                                0
                            }
                            Access::Vblank { .. } => return 0,
                        };
                        actual.push((at, op, value));
                        value
                    })
                    .unwrap();
                    assert_eq!(
                        actual, expected,
                        "fixture={fixture} pc={pc:04x} v={v} h={h} odd={odd}"
                    );
                    let c = &source.snes().cpu;
                    assert_eq!(
                        (cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc, cpu.p),
                        (c.a, c.x, c.y, c.sp, c.pc, c.pack_flags()),
                        "fixture={fixture} pc={pc:04x}"
                    );
                    assert_eq!(clock, *source.timeline(), "fixture={fixture} pc={pc:04x}");
                    work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                }
                assert_eq!((stores, cpu.pc), (14, 0x86df));
                assert_eq!(ram, source.snes().ram);
            }
        }
    }
}

#[test]
fn sprite_dma_live_tail_matches_rom_bus_registers_and_clock() {
    use crate::cycle_models::{
        intro_clear::Access, nmi_stack::NativeNmiContext, sprite_dma::MainSpriteTail,
    };
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, Snes, SourceCpuBusAccessKind as Kind, SourcePpuReadState,
    };
    let Some(rom) = original_rom() else { return };
    let mut visited = std::collections::BTreeSet::new();
    for fixture in 0..768usize {
        for (v, h) in [(212, 408), (224, 1338), (261, 1350)] {
            for odd in [false, true] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let offset =
                    [0, 1, 2, 9, 10, 11, 12, 13, 14, 15, 16, 23, 24, 26, 254, 255][fixture / 48];
                let seed = [
                    0u16, 1, 0x7f, 0xff, 0x100, 0x2345, 0x46bf, 0x7fff, 0x8000, 0xb53f, 0xb93f,
                    0xfdff, 0xfffe, 0xffff,
                ][fixture % 14];
                for (address, value) in [
                    (0xc00d, [0u16, 1, 2, 0xffff][fixture % 4]),
                    (
                        0xc00f,
                        [0, 0x400, 0x800, 0xfc00, 0xfbff, 0xffff][fixture / 4 % 6],
                    ),
                    (0xc013, [0, 1, 0xffff][fixture / 4 % 3]),
                    (0xc015, offset | if fixture & 1 == 0 { 0xab00 } else { 0 }),
                    (0xae8, seed),
                    (0xaea, seed.rotate_left(5)),
                    (0xaf4, !seed),
                    (0x1fe, 0x805c),
                ] {
                    machine.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
                }
                machine.ram[0x8c] = [0, 0xb5, 0xbc, 0xff][fixture / 12 % 4];
                machine.ram[0x12] = 1;
                let mut cpu = NativeNmiContext {
                    a: 0xab00 | fixture as u16,
                    x: 0x37,
                    y: 0xfc,
                    d: 0,
                    db: 0,
                    bank: 0,
                    pc: 0x86df,
                    sp: 0x1fd,
                    p: 0x10 | ((fixture as u8) & 0xc7),
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(odd),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = MainSpriteTail::default();
                while !work.complete() {
                    let pc = cpu.pc;
                    visited.insert(pc);
                    let expected = source
                        .step()
                        .unwrap()
                        .accesses
                        .into_iter()
                        .filter_map(|a| {
                            if a.address >= 0x2000
                                && !(0x7e0000..0x800000).contains(&a.address)
                                && link_dma_data_byte(a.address).is_none()
                            {
                                return None;
                            }
                            let (op, value) = match a.kind {
                                Kind::Read { value, width } => {
                                    (Access::Read(a.address, width), value)
                                }
                                Kind::Write { value, width } => {
                                    (Access::Write(a.address, value, width), 0)
                                }
                                _ => return None,
                            };
                            Some((a.timestamp.master_cycles(), op, value))
                        })
                        .collect::<Vec<_>>();
                    let mut actual = Vec::new();
                    work.step(&mut cpu, &mut clock, |at, op| {
                        let value = match op {
                            Access::Read(address, width) => {
                                let read = |a: u32| {
                                    if a < 0x2000 {
                                        ram[a as usize]
                                    } else if (0x7e0000..0x800000).contains(&a) {
                                        ram[(a - 0x7e0000) as usize]
                                    } else {
                                        link_dma_data_byte(a).unwrap()
                                    }
                                };
                                if width == 1 {
                                    u16::from(read(address))
                                } else {
                                    u16::from_le_bytes([read(address), read(address + 1)])
                                }
                            }
                            Access::Write(address, value, width) => {
                                let offset = if address >= 0x7e0000 {
                                    address - 0x7e0000
                                } else {
                                    address
                                } as usize;
                                ram[offset..offset + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                0
                            }
                            Access::Vblank { .. } => return 0,
                        };
                        actual.push((at, op, value));
                        value
                    })
                    .unwrap();
                    assert_eq!(
                        actual, expected,
                        "fixture={fixture} pc={pc:04x} v={v} h={h} odd={odd}"
                    );
                    let c = &source.snes().cpu;
                    assert_eq!(
                        (cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc, cpu.p),
                        (c.a, c.x, c.y, c.sp, c.pc, c.pack_flags()),
                        "fixture={fixture} pc={pc:04x}"
                    );
                    assert_eq!(clock, *source.timeline(), "fixture={fixture} pc={pc:04x}");
                    work = work.clone();
                }
                assert_eq!((cpu.pc, cpu.sp, ram[0x12]), (0x8034, 0x1ff, 0));
                assert_eq!(ram, source.snes().ram);
            }
        }
    }
    assert_eq!(
        visited.len(),
        66,
        "all animation, pointer, RTS and caller instructions"
    );
}
