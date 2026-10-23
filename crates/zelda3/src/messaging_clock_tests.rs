//! Exercise the native caller's instruction owners with one uninterrupted
//! hardware budget. Test entry clocks are explicit fixtures, never runtime
//! admission values or a replacement for the missing native clock origin.

use super::*;
use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Caller {
    Initialize,
    RenderReturn,
    InterfaceReturn,
    CallPreparation,
    Preparation,
    Return,
    Wait,
}

#[test]
fn runtime_initializer_keeps_its_clock_and_state_through_held_hosts() {
    let (assets, rom) = fast_forward_cycle_tests::character_decoder_fixture().unwrap();
    for field in 30_000..30_004 {
        let mut state = ZeldaState::new();
        state.set_assets(&assets).unwrap();
        state.write_wram_byte(0x10, 14);
        state.write_wram_byte(0x11, 2);
        state.ram[0x12] = 1;
        state.ram[0x1cf0..0x1cf2].copy_from_slice(&0x159u16.to_le_bytes());
        state.sync_native_game_state_from_ram();
        state.main_loop_caller_phase = crate::cycle_models::main_wait::MainLoopCallerPhase::ModuleRouting;
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x0ec483, stop_pc: 0x008034, sp: 0x01f6, db: 0x0e,
            a: 0, x: 0, y: 0, zero: false, waiting: false,
            stack_address: 0x01f7, stack_bytes: &[0x45, 0xc4, 0, 0x4d, 0xf8, 0, 0x59, 0x80, 0],
            ..super::super::DUNGEON_MAIN_WAIT_CPU_CHECKPOINT
        };
        let mut source = RomCpuTimingRun::new(&rom, &state.wram_image(), &state.sram,
            &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
        source.restore_original_dialogue_pointer_table().unwrap();
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance_in_field(
            field, CpuRasterPosition::new(58, 1358), CpuBusWorkload::with_dynamic_hdma());
        let work = state.native_dialogue_initialization_caller_work().unwrap();
        state.clocked_dialogue_initialization_caller = Some(ClockedDialogueInitializationCaller::new(
            work, NativeDialogueCallerClock::new(budget.clone(), snes::HdmaTimingState::new(&state.dma))));
        state.advance_clocked_dialogue_initialization();
        let mut held_hosts = 0;
        loop {
            for _ in 0..200_000 {
                if super::super::advance_rom_cpu_step(&mut source, &mut budget).reached_boundary().is_some() {
                    break;
                }
            }
            for range in [0x1cd0..0x1cf0, 0x720..0x728, 0x11a..0x126,
                0x10000..0x10800, 0x11200..0x12000, 0x14000..0x15000] {
                for address in range {
                    assert_eq!(state.read_wram_byte(address), source.ram_byte(address), "field={field} address={address:05x}");
                }
            }
            if let Some(owner) = state.clocked_dialogue_initialization_caller.as_ref() {
                assert_eq!(owner.clock.as_ref().unwrap().budget, budget);
                assert_eq!(state.paired_resume_cpu_boundary_blocker(), Some("retained-dialogue-initialization-caller"));
                state.frame_ctr_dbg += 1;
                state.game_execution_scheduler.begin_host_frame();
                assert!(state.resume_clocked_dialogue_initialization(0, None));
                super::super::advance_rom_cpu_through_nmi(&mut source, &mut budget);
                held_hosts += 1;
                assert!(held_hosts < 10);
            } else {
                let phase = state.native_main_wait_cpu_phase.as_ref().unwrap();
                assert_eq!(phase.budget, budget);
                assert_eq!(phase.checkpoint, source.main_wait_checkpoint(0x805d));
                assert_eq!(phase.host, state.frame_ctr_dbg + 1);
                assert!(state.game_execution_scheduler.returned_main_is_waiting_for_nmi());
                break;
            }
        }
        assert!(held_hosts > 1);
    }
}

#[test]
fn scheduled_initialization_retains_native_caller_until_return() {
    let Some((assets, _)) = fast_forward_cycle_tests::character_decoder_fixture() else { return };
    let mut state = ZeldaState::new();
    state.set_assets(&assets).unwrap();
    state.write_wram_byte(0x10, 14);
    state.write_wram_byte(0x11, 2);
    state.ram[0x12] = 1;
    state.ram[0x1cf0..0x1cf2].copy_from_slice(&0x159u16.to_le_bytes());
    state.sync_native_game_state_from_ram();
    state.main_loop_caller_phase = crate::cycle_models::main_wait::MainLoopCallerPhase::ModuleRouting;
    let mut legacy = state.clone();
    legacy.complete_text_initialization_prefix();
    legacy.prepare_text_character_buffer_for_carry();

    state.complete_scheduled_text_initialization_prefix();
    assert_eq!(state.dialogue_initialization_caller.as_ref().unwrap().initialization.phase,
        TextInitializationPhase::CallDecoder);
    state.prepare_text_character_buffer_for_carry();
    assert_eq!(state.dialogue_initialization_caller.as_ref().unwrap().initialization.phase,
        TextInitializationPhase::Tail);
    assert_eq!(*state.wram_image(), *legacy.wram_image(), "held decoder publication");
    assert_eq!(state.paired_resume_cpu_boundary_blocker(),
        Some("retained-dialogue-initialization-caller"));
    state.interrupt_nmi_with_animated_bg_operands(0, None, false, None);
    legacy.interrupt_nmi_with_animated_bg_operands(0, None, false, None);
    assert!(state.dialogue_initialization_caller.is_some());

    legacy.complete_text_initialization_carry_suffix();
    legacy.complete_module0e_interface_after_run();
    legacy.nmi_prepare_sprites();
    legacy.finish_main_loop_caller_return();
    state.complete_post_trailing_nmi_continuation(
        GameWorkContinuation::FinishDialogueInitializationCallerReturn, 0, false, false);
    assert!(state.dialogue_initialization_caller.is_none());
    assert_eq!(*state.wram_image(), *legacy.wram_image(), "completed native caller publication");
    assert_eq!(state.last_nmi_prepare_sprites_master_cycles, legacy.last_nmi_prepare_sprites_master_cycles);
    assert_eq!(state.main_loop_caller_phase,
        crate::cycle_models::main_wait::MainLoopCallerPhase::Waiting);
    state.latch_nmi_update();
    assert!(!state.complete_native_dialogue_initialization_caller());
    assert!(state.game_state.display.nmi_update_is_latched(), "retired caller cleared a later latch");
}

#[test]
fn native_initialization_caller_keeps_one_clock_across_held_nmis() {
    let Some((assets, rom)) = fast_forward_cycle_tests::character_decoder_fixture() else {
        return;
    };
    let mut boundaries = 0;
    let mut interrupts = 0;
    let mut fixtures = 0;
    let mut interrupted_callers = std::collections::BTreeMap::<Caller, usize>::new();
    for field in 30_000..30_004u64 {
        let odd = field & 1 != 0;
        for hdma_enabled in [false, true] {
            for start_at_return in [false, true] {
                for entry in [
                    (224, 1320),
                    (0, 520),
                    (255, 528),
                    (255, 700),
                    (225, 0),
                    (224, 1000),
                    (224, 0),
                ] {
                    fixtures += 1;
                    let mut state = ZeldaState::new();
                    state.set_assets(&assets).unwrap();
                    state.write_wram_byte(0x10, 14);
                    state.write_wram_byte(0x11, 2);
                    state.ram[0x12] = 1;
                    state.ram[0x1cf0..0x1cf2].copy_from_slice(&0x159u16.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    state.main_loop_caller_phase =
                        crate::cycle_models::main_wait::MainLoopCallerPhase::ModuleRouting;
                    if hdma_enabled {
                        state.set_hdma_enable_mask(0x80);
                        let channel = &mut state.dma.channel[7];
                        channel.a_bank = 0x7e;
                        channel.a_adr = 0x2000;
                        channel.mode = 1;
                        channel.indirect = true;
                        // Native WRAM descriptors span two repeated127line transfers.
                        state.ram[0x2000..0x2007]
                            .copy_from_slice(&[0xff, 0, 0x30, 0xff, 0, 0x32, 0]);
                    }
                    let timing_dma = state.dma_with_native_hdma_enable();
                    let hdma = snes::HdmaTimingState::new(&timing_dma);
                    const STACK: [u8; 9] = [0x45, 0xc4, 0, 0x4d, 0xf8, 0, 0x59, 0x80, 0];
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: if start_at_return {
                            0x0e_c446
                        } else {
                            0x0e_c483
                        },
                        stop_pc: 0x00_8034,
                        a: 0,
                        x: 0,
                        y: 0,
                        sp: if start_at_return { 0x01f8 } else { 0x01f6 },
                        dp: 0,
                        db: 0x0e,
                        carry: false,
                        zero: false,
                        overflow: false,
                        negative: false,
                        interrupt_disable: false,
                        decimal: false,
                        accumulator_is_8_bit: true,
                        index_is_8_bit: true,
                        emulation: false,
                        waiting: false,
                        stack_address: if start_at_return { 0x01f9 } else { 0x01f7 },
                        stack_bytes: if start_at_return { &STACK[2..] } else { &STACK },
                    };
                    let mut source = RomCpuTimingRun::new(
                        &rom,
                        &state.wram_image(),
                        &state.sram,
                        &state.ppu,
                        &timing_dma,
                        [0; 4],
                        checkpoint,
                    )
                    .unwrap();
                    source.restore_original_dialogue_pointer_table().unwrap();
                    let budget = CpuCycleBudget::until_next_nmi_acceptance_in_field(
                        field,
                        CpuRasterPosition::new(entry.0, entry.1),
                        CpuBusWorkload::with_dynamic_hdma(),
                    );
                    let mut source_budget = budget.clone();
                    let mut work = state.native_dialogue_initialization_caller_work().unwrap();
                    if start_at_return {
                        work.phase = DialogueInitializationCallerPhase::RenderReturn;
                    }
                    let mut owner = ClockedDialogueInitializationCaller::new(
                        work, NativeDialogueCallerClock::new(budget, hdma));
                    let mut saw_open_nmi = false;
                    for _ in 0..200_000 {
                        let (cycles, native_advance) = owner.step(&mut state);
                        let caller = match owner.work.phase {
                            DialogueInitializationCallerPhase::Initialize => Caller::Initialize,
                            DialogueInitializationCallerPhase::RenderReturn => Caller::RenderReturn,
                            DialogueInitializationCallerPhase::InterfaceReturn => Caller::InterfaceReturn,
                            DialogueInitializationCallerPhase::CallPreparation => Caller::CallPreparation,
                            DialogueInitializationCallerPhase::Preparation => Caller::Preparation,
                            DialogueInitializationCallerPhase::Return => Caller::Return,
                            DialogueInitializationCallerPhase::Complete => Caller::Wait,
                        };
                        let pc = source.pc();
                        let raster = source_budget.raster_position().coordinates();
                        source.set_raster_position(raster.0, raster.1);
                        let actual_cost = source.step().master_cycles;
                        assert_eq!(
                            cycles, actual_cost,
                            "entry={entry:?} odd={odd} caller={caller:?} PC={pc:06x}"
                        );
                        let source_advance = super::super::advance_instruction_with_rom_hdma(
                            &mut source,
                            &mut source_budget,
                            actual_cost,
                        );
                        assert_eq!(native_advance, source_advance);
                        assert_eq!(owner.clock.as_ref().unwrap().budget, source_budget);
                        boundaries += 1;
                        if native_advance.reached_boundary().is_some() {
                            *interrupted_callers.entry(caller).or_default() += 1;
                            assert_eq!(
                                source.ram_byte(0x12),
                                u8::from(state.game_state.display.nmi_update_is_latched())
                            );
                            if !state.game_state.display.nmi_update_is_latched() {
                                use crate::cycle_models::main_wait::MainLoopCallerPhase;
                                let next_pc = match caller {
                                    Caller::Wait => {
                                        assert_eq!(
                                            state.main_loop_caller_phase,
                                            MainLoopCallerPhase::Waiting
                                        );
                                        owner.wait.next_pc()
                                    }
                                    Caller::Return => match state.main_loop_caller_phase {
                                        MainLoopCallerPhase::ReturnBranch => 0x805f,
                                        MainLoopCallerPhase::Waiting => 0x8034,
                                        _ => panic!("unfinished caller opened NMI"),
                                    },
                                    _ => panic!("callee opened NMI before the main caller's STZ"),
                                };
                                assert_eq!(next_pc, source.pc());
                                if state.main_loop_caller_phase == MainLoopCallerPhase::Waiting {
                                    let phase = owner.into_main_wait_phase(&state, 30_002);
                                    assert_eq!(phase.checkpoint, source.main_wait_checkpoint(0x805d));
                                    assert_eq!(phase.budget, source_budget);
                                }
                                for range in [
                                    0x1cd0..0x1cf0,
                                    0x720..0x728,
                                    0x11a..0x126,
                                    0x10000..0x10800,
                                    0x11200..0x12000,
                                    0x14000..0x15000,
                                ] {
                                    for address in range {
                                        assert_eq!(
                                            state.read_wram_byte(address),
                                            source.ram_byte(address),
                                            "caller return address={address:05x}"
                                        );
                                    }
                                }
                                saw_open_nmi = true;
                                break;
                            }
                            // Exercise the runtime owner itself across the held NMI.
                            state.dialogue_initialization_caller = Some(owner.work.clone());
                            assert_eq!(state.paired_resume_cpu_boundary_blocker(),
                                Some("retained-dialogue-initialization-caller"));
                            let serialized = bincode::serialize(
                                state.dialogue_initialization_caller.as_ref().unwrap()).unwrap();
                            state.dialogue_initialization_caller =
                                Some(bincode::deserialize(&serialized).unwrap());
                            // Native handler accounting determines the CPU work. The
                            // same native hardware budget owns refresh and overshoot.
                            // No source cost or clock is copied into this path.
                            let clock = owner.begin_nmi();
                            state.interrupt_nmi_with_animated_bg_operands(0, None, false, None);
                            let native_nmi_work = state.last_nmi_handler_master_cycles.unwrap();
                            assert_eq!(
                                state.last_nmi_dma_master_cycles, 0,
                                "held NMI must not run general DMA"
                            );
                            assert_eq!(
                                clock.advance(&state, native_nmi_work),
                                CpuWorkAdvance::Complete
                            );
                            super::super::advance_rom_cpu_through_nmi(
                                &mut source,
                                &mut source_budget,
                            );
                            assert_eq!(clock.budget, source_budget,
                        "entry={entry:?} odd={odd} caller={caller:?} PC={pc:06x} native NMI work={native_nmi_work}");
                            for range in [
                                0x1cd0..0x1cf0,
                                0x720..0x728,
                                0x10000..0x10800,
                                0x11200..0x12000,
                                0x14000..0x15000,
                            ] {
                                for address in range {
                                    assert_eq!(
                                        state.read_wram_byte(address),
                                        source.ram_byte(address),
                                        "entry={entry:?} NMI address={address:05x}"
                                    );
                                }
                            }
                            owner.work = state.dialogue_initialization_caller.take().unwrap();
                            interrupts += 1;
                        }
                    }
                    assert!(saw_open_nmi, "caller never reached its first open NMI");
                }
            }
        }
    }
    for caller in [
        Caller::Initialize,
        Caller::RenderReturn,
        Caller::InterfaceReturn,
        Caller::Preparation,
        Caller::Wait,
    ] {
        assert!(
            interrupted_callers.contains_key(&caller),
            "missing interruption coverage for {caller:?}"
        );
    }
    eprintln!("native clock chain: {fixtures} fixture entries, {boundaries} instructions, {interrupts} held NMI resumes; boundaries={interrupted_callers:?}");
}

#[test]
fn glyph_return_caller_keeps_source_clock_and_state_through_held_nmi() {
    let (assets,rom) = fast_forward_cycle_tests::character_decoder_fixture().unwrap();
    for field in 30_000..30_004 {
        for entry in [CpuRasterPosition::new(219,1310),CpuRasterPosition::new(224,1308)] {
            let mut state = ZeldaState::new();
            state.set_assets(&assets).unwrap();
            state.write_wram_byte(0x10, 14);
            state.write_wram_byte(0x11, 2);
            state.ram[0x12] = 1;
            state.ram[0xaa2] = 0xa0;
            state.sync_native_game_state_from_ram();
            state.main_loop_caller_phase = crate::cycle_models::main_wait::MainLoopCallerPhase::ModuleRouting;
            state.game_execution_scheduler.begin_host_frame();
            let checkpoint = RomCpuCheckpoint {
                entry_pc: 0x0ec9f5,stop_pc: 0x008034,sp: 0x1f6,db: 0x0e,
                stack_address: 0x1f7,stack_bytes: &[0x45,0xc4,0,0x4d,0xf8,0,0x59,0x80,0],
                waiting: false,..super::super::DUNGEON_MAIN_WAIT_CPU_CHECKPOINT
            };
            let mut source = RomCpuTimingRun::new(&rom,&state.wram_image(),&state.sram,
                &state.ppu,&state.dma,[0;4],checkpoint).unwrap();
            let mut budget = CpuCycleBudget::until_next_nmi_acceptance_in_field(
                field,entry,CpuBusWorkload::with_dynamic_hdma());
            state.clocked_dialogue_return_caller = Some(ClockedDialogueReturnCaller::new(
                DialogueReturnWork::default(),NativeDialogueCallerClock::new(budget.clone(),
                    snes::HdmaTimingState::new(&state.dma))));
            state.advance_clocked_dialogue_return();
            let mut held = 0;
            loop {
                for step in 0..200_000 {
                    if super::super::advance_rom_cpu_step(&mut source,&mut budget).reached_boundary().is_some() { break; }
                    assert!(step < 199_999,"source return failed to reach NMI");
                }
                for range in [0x11a..0x126,0x120..0x130,0x710..0x712,0x800..0xa00,0xa20..0xaa0] {
                    for address in range { assert_eq!(state.read_wram_byte(address),source.ram_byte(address),
                        "field={field} entry={entry:?} address={address:04x}"); }
                }
                if let Some(owner) = state.clocked_dialogue_return_caller.as_ref() {
                    assert_eq!(owner.clock.as_ref().unwrap().budget,budget);
                    assert_eq!(state.paired_resume_cpu_boundary_blocker(),Some("retained-clocked-dialogue-return-caller"));
                    state.frame_ctr_dbg += 1;
                    state.game_execution_scheduler.begin_host_frame();
                    assert!(state.resume_clocked_dialogue_return(0,None));
                    super::super::advance_rom_cpu_through_nmi(&mut source,&mut budget);
                    held += 1;
                    assert!(held < 3);
                } else {
                    let phase = state.native_main_wait_cpu_phase.as_ref().unwrap();
                    assert_eq!(phase.budget,budget);
                    assert_eq!(phase.checkpoint,source.main_wait_checkpoint(0x805d));
                    assert!(state.game_execution_scheduler.returned_main_is_waiting_for_nmi());
                    break;
                }
            }
            assert!(held > 0);
        }
    }
}
