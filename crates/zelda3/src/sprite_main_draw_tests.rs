use crate::zelda_rtl::sprite::SpriteDistressEntry;
use crate::zelda_rtl::sprite::{
    SpriteDamageToEntry, SpriteHitBoxEntry, SpriteMoveEntry, SpriteMoveZEntry, SpriteRecoilEntry,
    SpriteShadowEntry,
};
use super::*;

#[test]
fn prefix3_leever_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..14 {
            for y in [0u16, 0x40, 0xf0, 0xf8, 0x100, 0x200] {
                let mut n = marker29_state();
                for (a, v) in [(0xdd0, 9), (0xe20, 0x71), (0xdc0, gfx),
                    (0xd10, 0x40), (0xd00, y as u8), (0xd20, (y >> 8) as u8)] {
                    n.write_wram_byte(a + k, v);
                }
                n.write_wram_byte(0xfa0, k as u8);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x06ce45, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x06ce45, s,
                    |s| s.cpu.k == 6 && (0xce45..0xcec0).contains(&s.cpu.pc),
                    || n.leever_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("prefix3 leever draw: {cases} ROM cases");
}

#[test]
fn marker44_laser_eye_draw_matches_rom() {
    for k in [0usize, 15] {
        for head in [0, 1] {
            for delay in [0, 1] {
                for direction in 0..4 {
                    for y in [0x40u16, 0xf8, 0x200] {
                        let mut n = marker29_state();
                        for (address, value) in [(0xe20, 0x95), (0xdd0, 9),
                            (0xeb0, head), (0xf10, delay), (0xde0, direction),
                            (0xdc0, 1), (0xcaa, 0x80)] {
                            n.write_wram_byte(address+k, value);
                        }
                        n.write_wram_word(0xfda, y);
                        n.sync_native_game_state_from_ram();
                        let Some(s) = marker29_machine(&n, 0x1ea708, k) else { return };
                        crate::cycle_ledger::assert_rom_marker_work(0x1ea708, s,
                            |s| s.cpu.k == 0x1e && ((0xa708..0xa73f).contains(&s.cpu.pc)
                                || (0x87f0..0x87f5).contains(&s.cpu.pc)), || {
                                    n.laser_eye_draw_prologue(k);
                                    n.laser_eye_draw_multiple(k);
                                });
                    }
                }
            }
        }
    }
}
use crate::rom_random::RomRandomResult;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

#[test]
fn zoro_caller_price_matches_executed_rom_paths() {
    use snes::{CartType, Snes};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("saves/zelda3.sfc"));
    let Ok(rom) = std::fs::read(path) else { return };
    let mut cases = 0;
    for slot in [0usize, 15] {
        for subtype in [0u8, 2, 3, 255] {
            for free in [-1i32, 0, 14, 29] {
                for (c, delay, below) in [(1, 1, false), (1, 0, false),
                    (0, 1, false), (0, 1, true)] {
                    let mut native = fresh_state();
                    let mut image = native.wram_image().into_owned();
                    image[0x11] = 0;
                    image[0xfa0] = slot as u8;
                    image[0xdd0 + slot] = 9;
                    image[0xe20 + slot] = 0x9d;
                    image[0xe90 + slot] = 1;
                    image[0xdb0 + slot] = c;
                    image[0xe80 + slot] = subtype;
                    image[0xdf0 + slot] = delay;
                    image[0xd10 + slot] = 0x40;
                    image[0xd00 + slot] = 0x40;
                    image[0x20..0x22].copy_from_slice(&(if below { 0u16 } else { 0x80 }).to_le_bytes());
                    image[0x22..0x24].copy_from_slice(&0x200u16.to_le_bytes());
                    image[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                    image[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                    image[0x1f800..0x1f81e].fill(1);
                    if free >= 0 { image[0x1f800 + free as usize] = 0; }
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    native.set_lag_free(true);
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram.copy_from_slice(&native.wram_image());
                    machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(0x30);
                    machine.cpu.k = 0x1e;
                    machine.cpu.db = 0x1e;
                    machine.cpu.pc = 0x9bc8;
                    machine.cpu.sp = 0x1fc;
                    machine.cpu.x = slot as u16;
                    let mut default = native.clone();
                    let source = crate::cycle_ledger::assert_rom_marker_work(0x1e_8b19, machine,
                        |s| s.cpu.k == 0x1e && (0x9bc8..=0x9c6a).contains(&s.cpu.pc), || {
                            let _parent = crate::cycle_ledger::routine(0x1e_8b19);
                            native.sprite_9_c_zoro(slot);
                        });
                    for a in [0xdd0, 0xdb0, 0xe80, 0xdc0, 0xd50] {
                        assert_eq!(native.read_wram_byte(a + slot), source.snes().ram[a + slot]);
                    }
                    assert_eq!(&native.wram_image()[0x1f800..0x1f986],
                        &source.snes().ram[0x1f800..0x1f986]);
                    default.set_lag_free(false);
                    let actual = crate::cycle_ledger::measure_routine_self_work(0x1e_8b19, || {
                        let _parent = crate::cycle_ledger::routine(0x1e_8b19);
                        default.sprite_9_c_zoro(slot);
                    });
                    assert_eq!(actual, 0);
                    cases += 1;
                }
            }
        }
    }
    eprintln!("Zoro caller: {cases} executed-ROM cases");
}

#[test]
fn paused_rat_animation_and_double_return_cost_match_source() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for animation in 0..8 {
        for (state, submodule, modal_pause) in [(9, 2, 0), (9, 0, 1), (8, 0, 0)] {
            let mut native = cucco_draw_fixture(0);
            native.set_submodule(submodule);
            native.write_wram_byte(0x0fc1, modal_pause);
            native.sync_native_game_state_from_ram();
            {
                let mut slot = native.sprite_slot_view_mut(2);
                slot.set_sprite_type(0x6d);
                slot.set_state(state);
                slot.set_a(animation);
            }
            let checkpoint = RomCpuCheckpoint {
                entry_pc: 0x05_a8b0, stop_pc: 0x05_ffff,
                a: 0, x: 2, y: 0, sp: 0x01fc, dp: 0, db: 5,
                carry: true, zero: false, overflow: false, negative: false,
                interrupt_disable: false, decimal: false,
                accumulator_is_8_bit: true, index_is_8_bit: true,
                emulation: false, waiting: false,
                stack_address: 0x01fd, stack_bytes: &[0xfe, 0xff],
            };
            let mut source = RomCpuTimingRun::new(&rom, &native.wram_image(), &[0; 0x2000],
                &native.ppu, &native.dma, [0; 4], checkpoint).unwrap();
            let mut expected = 0u64;
            for _ in 0..1000 {
                if source.is_complete() { break; }
                expected += u64::from(source.step().master_cycles);
            }
            assert!(source.is_complete(), "paused rat did not return");
            let before = crate::cycle_ledger::master();
            native.sprite_6_d_rat(2);
            assert_eq!(crate::cycle_ledger::master() - before, expected,
                "animation={animation} state={state} submodule={submodule} modal={modal_pause}");
            for address in (0x800..0xaa0).chain([0xdc2, 0xf52]) {
                assert_eq!(native.read_wram_byte(address), source.ram_byte(address),
                    "animation={animation} address={address:04x}");
            }
        }
    }
}

fn cucco_draw_fixture(velocity: u8) -> ZeldaState {
    let mut state = fresh_state();
    state.ram[0x0fd8..0x0fda].copy_from_slice(&0x20u16.to_le_bytes());
    state.ram[0x0fda..0x0fdc].copy_from_slice(&0x30u16.to_le_bytes());
    state.sync_native_game_state_from_ram();
    state.oam_state_mut().set_current_pointer(0x800);
    state.oam_state_mut().set_current_extended_pointer(0xa20);
    let mut slot = state.sprite_slot_view_mut(2);
    slot.set_sprite_type(0x0b);
    slot.set_state(9);
    slot.set_x(0x20);
    slot.set_y(0x30);
    slot.set_x_velocity(velocity);
    slot.set_oam_flags(0x25);
    slot.set_flags2(1);
    slot.set_flags3(0x10);
    state
}

#[test]
fn cucco_draw_from_entry_matches_source_at_each_retained_instruction() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    let mut cases = 0;
    let mut boundaries = 0;
    for velocity in 0..=u8::MAX {
        for flags in [0, 0x25, 0x40, 0xff] {
            for offscreen in [false, true] {
                let mut native = cucco_draw_fixture(velocity);
                native.sprite_slot_view_mut(2).set_oam_flags(flags);
                if offscreen { native.sprite_workspace_mut().set_current_sprite_x(0x130); }
                let checkpoint = RomCpuCheckpoint {
                    entry_pc: 0x06_a5c2, stop_pc: 0x06_a5da,
                    a: 0, x: 2, y: 0, sp: 0x01fc, dp: 0, db: 6,
                    carry: true, zero: false, overflow: false, negative: false,
                    interrupt_disable: false, decimal: false,
                    accumulator_is_8_bit: true, index_is_8_bit: true,
                    emulation: false, waiting: false,
                    stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
                };
                let mut source = RomCpuTimingRun::new(&rom, &native.wram_image(), &[0; 0x2000],
                    &native.ppu, &native.dma, [0; 4], checkpoint).unwrap();
                let initial_ram = native.wram_image().into_owned();
                let before = crate::cycle_ledger::master();
                let mut work = native.begin_cucco_draw(2);
                assert_eq!(*native.wram_image(), initial_ram, "admission cannot publish facing or prep");
                assert_eq!(crate::cycle_ledger::master(), before, "admission cannot consume unbudgeted work");
                let mut cycles = 0u64;
                let mut steps = 0;
                let mut flags_waiting = false;
                while !source.is_complete() {
                    let pc = source.pc();
                    if let Some(instruction) = work.prefix.clone().step(
                        native.sprite_slot_view(2).x_velocity(), native.sprite_slot_view(2).oam_flags())
                    {
                        assert_eq!(instruction.pc, pc);
                    }
                    let source_cycles = source.step().master_cycles;
                    cycles += u64::from(source_cycles);
                    assert_eq!(native.retire_cucco_draw_instruction(&mut work), Some(source_cycles),
                        "case={cases} PC={pc:06x}");
                    if pc == 0x06_a5d4 { flags_waiting = false; }
                    for address in (0..6).chain(0x800..0xaa0)
                        .chain([0xdd2, 0xf02, 0xf52, 0xfa8, 0xfa9, 0xbc4, 0xbc5, 0x1ef80])
                    {
                        if flags_waiting && address == 0xf52 { continue; }
                        assert_eq!(native.read_wram_byte(address), source.ram_byte(address),
                            "case={cases} PC={pc:06x} address={address:05x}");
                    }
                    match pc {
                        // Change native inputs after their source loads. The
                        // branch/sign and final flag store must use the saved
                        // operands, even after a serialized suspension.
                        0x06_a5c2 => native.sprite_slot_view_mut(2).set_x_velocity(velocity ^ 0xff),
                        0x06_a5cc => {
                            native.sprite_slot_view_mut(2).set_oam_flags(flags ^ 0xff);
                            flags_waiting = true;
                        }
                        _ => {}
                    }
                    work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                    steps += 1;
                    boundaries += 1;
                    assert!(steps < 200);
                }
                assert_eq!(native.retire_cucco_draw_instruction(&mut work), None);
                assert!(work.prefix.is_complete() && work.draw.is_none());
                assert_eq!(crate::cycle_ledger::master() - before, cycles);
                cases += 1;
            }
        }
    }
    eprintln!("Cucco entry and draw: {cases} cases, {boundaries} retained source instruction boundaries");
}

#[test]
fn cucco_draw_retains_caller_across_each_native_clock_boundary() {
    use super::super::game_execution_scheduler::{CpuBusWorkload, CpuFieldTiming, CpuRasterPosition};
    let mut native = cucco_draw_fixture(1);
    native.latch_nmi_update();
    let mut work = native.begin_cucco_draw(2);
    let mut expected = native.clone();
    let mut expected_work = work;
    assert_eq!(expected.advance_cucco_draw(&mut expected_work, None), CpuWorkAdvance::Complete);
    let expected_oam = expected.ram[0x800..0xaa0].to_vec();
    let mut boundaries = 0;
    loop {
        // A generic clock deadline immediately before NMI acceptance tests
        // every draw instruction as the interrupted one, without a PC cut.
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            CpuRasterPosition::new(225, 11), CpuBusWorkload::default(),
            CpuFieldTiming::non_interlace(false));
        let advance = native.advance_cucco_draw(&mut work, Some(&mut budget));
        assert_eq!(native.sprite_slot_view(2).sprite_type(), 0x0b,
            "the Cucco suffix must remain pending until its callee returns");
        assert!(native.game_state.display.nmi_update_is_latched());
        if advance == CpuWorkAdvance::Complete { break; }
        assert!(advance.reached_boundary().is_some());
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        // The saved facing and prep coordinates must not be recomputed.
        // The head-direction load is still after the draw and must see this.
        native.sprite_slot_view_mut(2).set_x_velocity(0xff);
        native.sprite_slot_view_mut(2).set_head_direction(1);
        if work.draw.as_ref().is_some_and(|draw| draw.coordinates_prepared()) {
            native.sprite_workspace_mut().set_current_sprite_x(0x70);
            native.sprite_workspace_mut().set_current_sprite_y(0x80);
        }
        boundaries += 1;
        assert!(boundaries < 150);
    }
    assert!(boundaries > 40);
    assert_eq!(&native.wram_image()[0x800..0xaa0], &expected_oam);
    let retired = crate::cycle_ledger::master();
    assert_eq!(native.advance_cucco_draw(&mut work, None), CpuWorkAdvance::Complete);
    assert_eq!(crate::cycle_ledger::master(), retired, "a returned callee has no work left");
    native.complete_cucco_draw(work);
    assert_eq!(native.sprite_slot_view(2).sprite_type(), 0x3d);
    assert_eq!(native.sprite_slot_view(2).delay_main(), 48);
    assert_eq!(&native.wram_image()[0x800..0xaa0], &expected_oam, "caller return must not redraw");
    assert!(native.game_state.display.nmi_update_is_latched(), "Cucco does not own the main caller latch");
}

#[test]
#[should_panic(expected = "Cucco caller resumed before its draw returned")]
fn cucco_caller_rejects_publishing_suffix_before_draw_return() {
    let mut state = cucco_draw_fixture(1);
    let work = state.begin_cucco_draw(2);
    state.complete_cucco_draw(work);
}

#[test]
#[cfg(feature = "rom-timing")]
fn cucco_native_budget_suspends_sprite_main_and_preserves_module09_return() {
    use super::super::game_execution_scheduler::{CpuBusWorkload, CpuFieldTiming, CpuRasterPosition};
    use crate::cycle_models::main_wait::MainLoopCallerPhase;
    for (remaining, x) in [(1u16, 0x20), (80, 0x20), (400, 0x20), (900, 0x20),
        (700, 0x130), (1300, 0x130), (1800, 0x130)] {
        let mut state = cucco_draw_fixture(1);
        state.sprite_workspace_mut().set_current_sprite_x(x);
        state.set_main_module(9);
        state.set_rom_startup_timing(true);
        state.sprite_slot_view_mut(2).set_head_direction(1);
        state.sprite_slot_view_mut(3).set_state(9);
        state.sprite_slot_view_mut(3).set_delay_main(23);
        state.latch_nmi_update();
        state.main_loop_caller_phase = MainLoopCallerPhase::ModuleRouting;
        state.pending_main_loop_common_suffix =
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
        let caller = SpriteMainCpuCaller::Module09 { boundary: OriginalTimingBoundary::NmiAccepted };
        let module_return = Module09ItemReceiptCallerReturn {
            link_oam: None,
            scroll: Module09SpriteMainReturn { bg2_x: 0, bg2_y: 0, bg1_x: 0, bg1_y: 0 },
            rain_already_published: false,
            after_sprite_main: Module09AfterSpriteMain::Ordinary,
        };
        state.active_module09_sprite_main_return = Some(module_return);
        state.game_execution_scheduler.begin_host_frame();
        state.game_execution_scheduler.begin_main_loop_iteration();
        let work = state.begin_cucco_draw(2);
        let mut atomic = state.clone();
        let mut atomic_work = work;
        assert_eq!(atomic.advance_cucco_draw(&mut atomic_work, None), CpuWorkAdvance::Complete);
        atomic.complete_cucco_draw(atomic_work);
        atomic.complete_post_trailing_nmi_continuation(
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::AfterSlot(2), caller,
            }, 0, false, false);

        let clock = 225u32 * 1364 + 12 - u32::from(remaining);
        let entry = CpuRasterPosition::new((clock / 1364) as u16, (clock % 1364) as u16);
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            entry, CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
        assert!(!state.advance_or_suspend_cucco_draw(work, Some(&mut budget), caller));
        assert!(state.game_execution_scheduler.work_suspends_translated_call_stack());
        assert_eq!(state.sprite_slot_view(2).sprite_type(), 0x0b);
        assert_eq!(state.sprite_slot_view(3).delay_main(), 23);
        assert_eq!(state.active_module09_sprite_main_return, Some(module_return));
        assert_eq!(state.main_loop_caller_phase, MainLoopCallerPhase::ModuleRouting);
        assert!(state.game_state.display.nmi_update_is_latched());
        assert!(matches!(state.next_display_obj_scanout_generation,
            Some(ObjScanoutGenerations { oam: OamScanoutSource::RetainResidentPpuOam, .. })));
        let continuation = state.game_execution_scheduler.take_after_current_trailing_nmi()
            .expect("a measured NMI acceptance retains the call behind that NMI");
        let GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::CuccoDraw { work: saved }, caller: saved_caller,
        } = continuation else { panic!("lost the native Cucco caller") };
        assert_eq!(saved.slot(), 2);
        assert_eq!(saved_caller, caller);
        let boundary = SpriteMainCpuBoundary::CuccoDraw { work: saved };
        let boundary = bincode::deserialize(&bincode::serialize(&boundary).unwrap()).unwrap();
        // This is the real shared post-NMI resume dispatcher, including
        // lower slots, saved Module09 return and main-loop caller completion.
        state.complete_post_trailing_nmi_continuation(
            GameWorkContinuation::FinishSpriteMain { boundary, caller: saved_caller },
            0, false, false);
        assert_eq!(*state.wram_image(), *atomic.wram_image(), "remaining={remaining}");
        assert_eq!(state.game_state, atomic.game_state, "remaining={remaining}");
        assert_eq!(state.sprite_slot_view(3).delay_main(), 23, "upper slots must not run again");
        assert!(state.active_module09_sprite_main_return.is_none());
        assert!(state.pending_main_loop_common_suffix.is_none());
        assert_eq!(state.main_loop_caller_phase, MainLoopCallerPhase::Waiting);
        assert!(!state.game_state.display.nmi_update_is_latched());
        assert!(state.game_execution_scheduler.take_after_current_trailing_nmi().is_none());
    }
}

#[test]
fn shadow_draw_extended_oam_tail_resumes_after_ordinary_entry() {
    let mut split = fresh_state();
    split.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    split.oam_state_mut().set_current_extended_pointer(0xa20);
    split.sprite_slot_view_mut(2).set_y(0x0020);
    let mut atomic = split.clone();
    let mut split_info = PrepOamCoordsRet { x: 0x0123, y: 0, r4: 0, flags: 0x20 };
    let mut atomic_info = split_info;

    let pending = split.sprite_draw_shadow_custom_before_extended(2, &mut split_info, 10)
        .expect("visible shadow has an extended-OAM tail");
    assert_eq!(&split.wram_image()[OAM_BUF..OAM_BUF + 4], &[0x23, 0x2a, 0x6c, 0x28]);
    assert_eq!(split.game_state.oam.extended_byte(0), 0);
    split.finish_shadow_extended_oam_write(pending);

    atomic.sprite_draw_shadow_custom(2, &mut atomic_info, 10, SpriteShadowEntry::Unpriced);
    assert_eq!(*split.wram_image(), *atomic.wram_image());
    assert_eq!(split.game_state, atomic.game_state);
    assert_eq!(split_info.y, atomic_info.y);
    assert_eq!(split.game_state.oam.extended_byte(0), 3);
}

#[test]
fn rat_random_run_delay_preserves_rng_carry_through_and_adc() {
    assert_eq!(rat_random_run_delay(RomRandomResult::new(1, false)), 0x41);
    assert_eq!(rat_random_run_delay(RomRandomResult::new(1, true)), 0x42);
}

#[test]
fn rope_random_run_delay_preserves_rng_carry_through_and_adc() {
    assert_eq!(rope_random_run_delay(RomRandomResult::new(4, false)), 0x44);
    assert_eq!(rope_random_run_delay(RomRandomResult::new(4, true)), 0x45);
    assert_eq!(
        rope_random_run_delay(RomRandomResult::new(0x98, true)),
        0x59
    );
}

#[test]
fn altar_zelda_warp_clamps_subdmd_index_via_delay_main_shift() {
    // sprite_delay_main[k] >> 2 picks which 2-entry slice; the call
    // should land without panicking even when shift would otherwise
    // overrun the table (we clamp at the high end).
    let mut s = fresh_state();
    let k = 4;
    s.sprite_slot_view_mut(k).set_delay_main(0xff);
    // The function performs an OAM allocation; ensure it returns
    // without trashing state we care about.
    let oam_cur_before = s.game_state.oam.current_pointer();
    s.sprite_draw_altar_zelda_warp(k);
    // OAM cursor should have moved (the alloc/draw advanced it).
    assert_ne!(s.game_state.oam.current_pointer(), oam_cur_before);
}

#[test]
fn antfairy_cycles_graphics_at_six_and_zeroes_low_bit_subtype2() {
    // Antfairy: increment sprite_subtype2; if (subtype2 & 1) | submodule_index
    // | modal_pause_flag is 0, bump sprite_graphics, wrapping 6 -> 0.
    // (sprite_main.c:18841 SpriteDraw_Antfairy.)
    let mut s = fresh_state();
    let k = 1;
    // Sprite_DrawMultiple ultimately writes via set_oam_helper0_at which
    // computes (oam - OAM_BUF) / 4; a fresh state's OAM_CUR_PTR is 0 and
    // that would underflow. Seed a valid cursor so the OAM write path
    // succeeds and the subtype/graphics side-effects are observable.
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    s.set_submodule(0);
    s.clear_modal_pause_flag();
    s.sprite_slot_view_mut(k).set_subtype2(0x10); // odd-mask passes -> ++subtype2 = 0x11 fails (& 1 == 1).
    s.sprite_slot_view_mut(k).set_graphics(2);
    s.sprite_draw_antfairy(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 0x11);
    // 0x11 & 1 == 1, so the inner branch did NOT run; graphics stays.
    assert_eq!(s.sprite_slot_view(k).graphics(), 2);

    // Even-bit case: subtype2 starts at 0x11 -> becomes 0x12, the
    // condition (subtype2 & 1) | submodule | modal_pause_flag == 0; so
    // graphics increments from 5 -> 6 -> wraps to 0.
    s.sprite_slot_view_mut(k).set_subtype2(0x11);
    s.sprite_slot_view_mut(k).set_graphics(5);
    s.sprite_draw_antfairy(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 0x12);
    assert_eq!(s.sprite_slot_view(k).graphics(), 0);
}

#[test]
fn moldorm_tail_bumps_oam_ptrs_and_sets_oam_flags() {
    // SpriteDraw_Moldorm_Tail: oam_cur_ptr += 4; oam_ext_cur_ptr += 1;
    // sprite_graphics[k]++; sprite_oam_flags[k] = 13.
    let mut s = fresh_state();
    let k = 7;
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    s.sprite_slot_view_mut(k).set_graphics(9);
    s.sprite_draw_moldorm_tail(k);
    assert_eq!(s.game_state.oam.current_pointer(), 0x804);
    assert_eq!(s.game_state.oam.current_extended_pointer(), 0xa21);
    assert_eq!(s.sprite_slot_view(k).graphics(), 10);
    assert_eq!(s.sprite_slot_view(k).oam_flags(), 13);
}

#[test]
fn moldorm_segment_c_zeros_graphics_and_advances_oam() {
    let mut s = fresh_state();
    let k = 3;
    s.sprite_slot_view_mut(k).set_graphics(0x55);
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    s.sprite_draw_moldorm_segment_c(k);
    assert_eq!(s.sprite_slot_view(k).graphics(), 0);
    assert_eq!(s.game_state.oam.current_pointer(), 0x810);
    assert_eq!(s.game_state.oam.current_extended_pointer(), 0xa24);
}

#[test]
fn trinexx_body_uses_head_outparam_flags_for_body_oam() {
    let mut s = fresh_state();
    let k = 0;
    s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    s.oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    s.sprite_slot_view_mut(k).set_x_low(0x78);
    s.sprite_slot_view_mut(k).set_x_high(0x08);
    s.sprite_slot_view_mut(k).set_y_low(0x10);
    s.sprite_slot_view_mut(k).set_y_high(0x16);
    s.sprite_slot_view_mut(k).set_a(0x78);
    s.sprite_slot_view_mut(k).set_c(0x10);
    s.sprite_slot_view_mut(k).set_ai_state(0);
    s.sprite_slot_view_mut(k).set_oam_flags(1);
    s.sprite_slot_view_mut(k).set_graphics(0);
    s.sprite_slot_view_mut(k).set_head_direction(0);

    s.sprite_draw_trinexx_rock_head_and_body(k);

    assert_eq!(s.ram[OAM_BUF + 91 * 4 + 3], 0x21);
}

#[test]
fn trinexx_body_keeps_head_outparam_flags_when_head_is_offscreen() {
    let mut s = fresh_state();
    let k = 0;
    s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    s.oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    s.sprite_workspace_mut().set_current_sprite_x(0x0878);
    s.sprite_workspace_mut().set_current_sprite_y(0x156c);
    s.sprite_slot_view_mut(k).set_x_low(0x78);
    s.sprite_slot_view_mut(k).set_x_high(0x08);
    s.sprite_slot_view_mut(k).set_y_low(0x6c);
    s.sprite_slot_view_mut(k).set_y_high(0x15);
    s.sprite_slot_view_mut(k).set_a(0x78);
    s.sprite_slot_view_mut(k).set_c(0x08);
    s.sprite_slot_view_mut(k).set_ai_state(0);
    s.sprite_slot_view_mut(k).set_oam_flags(1);
    s.sprite_slot_view_mut(k).set_object_priority(0x30);
    s.sprite_slot_view_mut(k).set_graphics(0);
    s.sprite_slot_view_mut(k).set_head_direction(0);

    s.sprite_draw_trinexx_rock_head_and_body(k);

    assert_eq!(s.ram[OAM_BUF + 91 * 4 + 3], 0x21);
}

#[test]
fn big_shadow_offsets_cursor_and_refreshes_cur_sprite_y() {
    // SpriteDraw_BigShadow: cur_sprite_y += sprite_z[k]; oam_cur += 16;
    // oam_ext += 4; then Sprite_Get16BitCoords overwrites cur_sprite_*.
    let mut s = fresh_state();
    let k = 2;
    s.sprite_slot_view_mut(k).set_z(8);
    // Put sprite at a known X/Y so Sprite_Get16BitCoords overwrites the
    // prior cur_sprite_y bump.
    s.sprite_slot_view_mut(k).set_x_low(0x40);
    s.sprite_slot_view_mut(k).set_y_low(0x50);
    s.sprite_slot_view_mut(k).set_x_high(0x00);
    s.sprite_slot_view_mut(k).set_y_high(0x01);
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    s.sprite_draw_big_shadow(k, 0);
    // Cursor + ext both advanced.
    assert_ne!(s.game_state.oam.current_pointer(), 0x800);
    // cur_sprite_y == sprite_get_y(k)  (from Sprite_Get16BitCoords).
    let cy = s.game_state.sprites.workspace.current_sprite_y();
    assert_eq!(cy, 0x0150);
}

#[test]
fn zirro_bomb_clears_state_when_delay_main_zero() {
    // SpriteDraw_ZirroBomb: if (!sprite_delay_main[k]) sprite_state[k] = 0;
    // then call Sprite_DrawMultiple. We verify only the state-side branch.
    // (sprite_main.c:13246 SpriteDraw_ZirroBomb.)
    let mut s = fresh_state();
    let k = 5;
    // Seed OAM cursors so Sprite_DrawMultiple's set_oam_helper0_at path
    // doesn't underflow (oam - OAM_BUF) on a fresh-zero cursor.
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    s.sprite_slot_view_mut(k).set_delay_main(0);
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_draw_zirro_bomb(k);
    assert_eq!(s.sprite_slot_view(k).state(), 0);

    // With non-zero delay, sprite_state must NOT be cleared.
    s.sprite_slot_view_mut(k).set_delay_main(0x10);
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_draw_zirro_bomb(k);
    assert_eq!(s.sprite_slot_view(k).state(), 9);
}

#[test]
fn chain_ball_mult_draw_clamps_at_256() {
    // ChainBallMult returns b when a >= 256; verify both branches.
    assert_eq!(chain_ball_mult(0x100, 0x42), 0x42);
    assert_eq!(chain_ball_mult_draw(0x100, 0x42), 0x42);
    // 200 * 80 = 16000 = 0x3e80 -> p>>8 = 0x3e, p>>7 & 1 =
    // (16000 >> 7) & 1 = 125 & 1 = 1, so result = 0x3e + 1 = 0x3f.
    // (Matches C: sprite_main.c:1397 ChainBallMult.)
    assert_eq!(chain_ball_mult(200, 80), 0x3f);
    assert_eq!(chain_ball_mult_draw(200, 80), 0x3f);
}

#[test]
fn named_fixed_point_mult_aliases_match_chain_ball_mult() {
    assert_eq!(guruguru_bar_mult(128, 10), chain_ball_mult(128, 10));
    assert_eq!(arrgi_mult(200, 80), chain_ball_mult(200, 80));
    assert_eq!(helmasaur_mult(0x100, 0x55), chain_ball_mult(0x100, 0x55));
    assert_eq!(trinexx_head_mult(0x100, 0x66), chain_ball_mult(0x100, 0x66));
}

#[test]
fn named_sin_helpers_flip_sign_on_second_half() {
    let guruguru = guruguru_bar_sin(0x20, 50);
    assert_eq!(guruguru_bar_sin(0x120, 50), (0i8).wrapping_sub(guruguru));

    let arrgi = arrgi_sin(0x30, 70);
    assert_eq!(arrgi_sin(0x130, 70), (0i8).wrapping_sub(arrgi));

    let helmasaur = helmasaur_sin(0x40, 90);
    assert_eq!(helmasaur_sin(0x140, 90), (0i8).wrapping_sub(helmasaur));

    let trinexx_head = trinexx_head_sin(0x50, 110);
    assert_eq!(
        trinexx_head_sin(0x150, 110),
        (0i8).wrapping_sub(trinexx_head)
    );
}

#[test]
fn chain_chomp_one_mult_returns_integer_complement_for_negative_input() {
    assert_eq!(chain_chomp_one_mult(0x08, 0x40), 2);
    assert_eq!(chain_chomp_one_mult(0xf8, 0x40), -3);
    assert_eq!(chain_chomp_one_mult(0x80, 0), -1);
}

#[test]
fn trinexx_mult_draw_handles_signed_input() {
    // a positive small case: (8 * 0x40) >> 8 = 0x02; round bit 7 of p
    // (i.e. (8*0x40)>>7 & 1) = 0. So result == 2.
    assert_eq!(trinexx_mult(8, 0x40), 2);
    assert_eq!(trinexx_mult_draw(8, 0x40), 2);
    // a negative case: a = 0xf8 (-8). at = 8. result = 2, sign negated:
    // 0u8 - 2 == 0xfe.
    assert_eq!(trinexx_mult(0xf8, 0x40), 0xfe);
    assert_eq!(trinexx_mult_draw(0xf8, 0x40), 0xfe);
}

#[test]
fn pikit_loot_skips_when_sprite_g_zero() {
    // SpriteDraw_Pikit_Loot returns early if sprite_G[k] == 0.
    let mut s = fresh_state();
    let k = 11;
    s.sprite_slot_view_mut(k).set_g(0);
    // OAM cursor unchanged baseline.
    let before = s.game_state.oam.current_pointer();
    let info = PrepOamCoordsRet::default();
    s.sprite_draw_pikit_loot(k, &info);
    assert_eq!(s.game_state.oam.current_pointer(), before);
}

#[test]
fn beamos_eyeball_low_d_uses_offset_zero() {
    // sprite_D < 0x20 → n = 0 → oam_cur_ptr unchanged after the n*4 bump.
    let mut s = fresh_state();
    let k = 0;
    s.sprite_slot_view_mut(k).set_direction(0x10);
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    let info = PrepOamCoordsRet {
        x: 0x40,
        y: 0x40,
        r4: 0,
        flags: 0,
    };
    s.sprite_draw_beamos_eyeball(k, &info);
    assert_eq!(s.game_state.oam.current_pointer(), 0x800);
    assert_eq!(s.game_state.oam.current_extended_pointer(), 0xa20);
    let scratch = &s.game_state.sprites.draw_hitbox_work;
    assert_eq!(scratch.x_low(), 5);
    assert_eq!(scratch.y_low(), 0xfd);
}

#[test]
fn beamos_eyeball_high_d_bumps_cursor_by_eight() {
    let mut s = fresh_state();
    let k = 0;
    s.sprite_slot_view_mut(k).set_direction(0x20);
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    let info = PrepOamCoordsRet {
        x: 0x40,
        y: 0x40,
        r4: 0,
        flags: 0,
    };
    s.sprite_draw_beamos_eyeball(k, &info);
    // n = 2 → oam_cur += 8, oam_ext += 2.
    assert_eq!(s.game_state.oam.current_pointer(), 0x808);
    assert_eq!(s.game_state.oam.current_extended_pointer(), 0xa22);
}

#[test]
fn thrown_item_gigantic_writes_oam_flags_from_table() {
    // sprite_oam_flags[k] = kThrowableScenery_DrawLarge_OamFlags[sprite_C[k] - 6];
    // C = 7 → idx = 1 → flags = 0.
    let mut s = fresh_state();
    let k = 6;
    s.sprite_slot_view_mut(k).set_c(7);
    // Force the prep helper to return None by setting BG2HOFS far away.
    // Then verify the OAM flags were written even though we early-return.
    s.set_bg2_x(0x4000);
    s.set_bg2_y(0x4000);
    s.sprite_slot_view_mut(k).set_x_low(0);
    s.sprite_slot_view_mut(k).set_y_low(0);
    s.sprite_draw_thrown_item_gigantic(k);
    assert_eq!(s.sprite_slot_view(k).oam_flags(), 0);

    s.sprite_slot_view_mut(k).set_c(6);
    s.sprite_draw_thrown_item_gigantic(k);
    assert_eq!(s.sprite_slot_view(k).oam_flags(), 0xc);
}

#[test]
fn witch_accept_shroom_sets_room_word_powder_bit() {
    let mut s = fresh_state();
    let k = 2;
    s.inventory_items_mut().set_mushroom(1);
    s.save_progress_mut().set_dungeon_info_word(0x109, 0x0002);

    s.witch_accept_shroom(k);

    assert_eq!(s.game_state.inventory.items.mushroom(), 0);
    assert_eq!(s.saved_room_flags(0x109), 0x0082);
    assert_eq!(s.read_wram_byte(SAVE_DUNG_INFO + 0x109), 0);
}

#[test]
fn movable_statue_far_from_link_does_not_touch_direction_tmp_counter() {
    let mut s = fresh_state();
    let k = 0;
    s.oam_state_mut().set_current_pointer(0x800);
    s.oam_state_mut().set_current_extended_pointer(0xa20);
    s.sprite_slot_view_mut(k).set_sprite_type(0x1c);
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_slot_view_mut(k).set_floor(1);
    s.sprite_slot_view_mut(k).set_x_low(0x40);
    s.sprite_slot_view_mut(k).set_x_high(0x15);
    s.sprite_slot_view_mut(k).set_y_low(0x77);
    s.sprite_slot_view_mut(k).set_y_high(0x08);
    s.follower_link_state_mut().set_x(0x14f8);
    s.follower_link_state_mut().set_y(0x08d8);
    s.follower_link_state_mut().clear_lower_level();
    s.follower_link_state_mut().set_facing(8);
    s.temp_counter_mut().set(0x02);

    s.sprite_1_c_statue(k);

    assert_eq!(s.game_state.scratch_counter.value(), 0x02);
    assert!(!s.game_state.player.follower_link.is_near_moveable_statue());
}

#[test]
fn action_hitbox_and_swamola_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    let mut cases = Vec::new();
    for facing in [0u8, 2, 4, 6] {
        for frames in [0u8, 1, 4, 8, 127, 128, 255] {
            for mode in 0..4 {
                for offset in [0u8, 127, 128, 255] {
                    // Item/position bypasses do not index the sword-frame table.
                    if frames == 127 && mode == 0 { continue; }
                    cases.push((0x06_f5e0, 0usize, vec![(0x2f, facing), (0x3c, frames),
                        (0x372, u8::from(mode == 1)), (0x301, if mode == 2 { 2 } else { 0 }),
                        (0x37a, if mode == 3 { 16 } else { 0 }), (0x45, offset), (0x44, offset),
                        (0x22, 255), (0x23, 255), (0x20, 0), (0x21, 0)]));
                }
            }
        }
    }
    for slot in [0usize, 5] {
        for velocity in [0u8, 1, 127, 128, 255] {
            for frame in [0u8, 31, 255] {
                cases.push((0x1d_9f64, slot, vec![(0xd40 + slot, velocity),
                    (0xd50 + slot, 16), (0xe80 + slot, frame), (0xdd0 + slot, 9),
                    (0xd00 + slot, 128), (0xd10 + slot, 128), (0xe40 + slot, 0)]));
            }
        }
    }
    for (entry, slot, changes) in cases {
        let mut native = ZeldaState::new();
        let mut image = native.wram_image().into_owned();
        for &(address, value) in &changes { image[address] = value; }
        image[0x90..0x92].copy_from_slice(&0x0800u16.to_le_bytes());
        image[0x92..0x94].copy_from_slice(&0x0a20u16.to_le_bytes());
        native.import_wram_image(&image);
        native.sync_native_game_state_from_ram();
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, &rom, 0x2000);
        machine.ram.copy_from_slice(&native.wram_image());
        machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
        machine.cpu.e = false;
        machine.cpu.unpack_flags(0x30);
        machine.cpu.k = (entry >> 16) as u8;
        machine.cpu.db = machine.cpu.k;
        machine.cpu.pc = entry as u16;
        machine.cpu.sp = 0x1fc;
        machine.cpu.x = slot as u16;
        let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
        let mut source = RomCpuTimingProbe::new(machine, clock,
            SourcePpuReadState::snes9x_reset()).unwrap();
        let mut measured = 0;
        for _ in 0..20000 {
            let cpu = &source.snes().cpu;
            let pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
            let owns = if entry == 0x06_f5e0 { (0x06_f594..=0x06_f644).contains(&pc) }
                else { (0x1d_9f64..=0x1d_a03b).contains(&pc) };
            let step = source.step().unwrap();
            if owns { measured += step.transactions.iter()
                .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>(); }
            if source.snes().cpu.pc == 0x8000 { break; }
        }
        assert_eq!(source.snes().cpu.pc, 0x8000, "entry={entry:x} {changes:?}");
        let mut hb = crate::types::SpriteHitBox {
            r0_xlo: 0, r8_xhi: 0, r1_ylo: 0, r9_yhi: 0, r2: 0, r3: 0,
            r4_spr_xlo: 0, r10_spr_xhi: 0, r5_spr_ylo: 0, r11_spr_yhi: 0,
            r6_spr_xsize: 0, r7_spr_ysize: 0,
        };
        for lagless in [true, false] {
            native.set_lag_free(lagless);
            let actual = crate::cycle_ledger::measure_routine_self_work(entry, || {
                if entry == 0x06_f5e0 { native.player_setup_action_hit_box(&mut hb); }
                else { native.swamola_draw(slot); }
            });
            if lagless {
                assert_eq!(actual, measured, "entry={entry:x} {changes:?}");
                if entry == 0x06_f5e0 {
                    assert_eq!(hb.r8_xhi, source.snes().ram[8]);
                    if hb.r8_xhi != 128 {
                        assert_eq!([hb.r0_xlo, hb.r1_ylo, hb.r2, hb.r3, hb.r9_yhi],
                            [source.snes().ram[0], source.snes().ram[1], source.snes().ram[2],
                             source.snes().ram[3], source.snes().ram[9]]);
                    }
                }
            } else {
                // Existing draw children have ROM-timing charges; only the
                // newly introduced caller instructions must be absent.
                assert_eq!(actual, 0, "entry={entry:x}");
            }
        }
    }
}

fn marker29_machine(native: &ZeldaState, entry: u32, slot: usize) -> Option<snes::Snes> {
    use snes::{CartType, Snes};
    let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let rom = std::fs::read(path).ok()?;
    let mut machine = Snes::new();
    machine.cart.load(CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&native.wram_image());
    machine.ram[0x1fb..0x200].copy_from_slice(&[0xff, 0x7f, 0xff, 0x7f, 0]);
    machine.cpu.e = false;
    machine.cpu.unpack_flags(0x30);
    machine.cpu.k = (entry >> 16) as u8;
    machine.cpu.db = (entry >> 16) as u8;
    machine.cpu.pc = entry as u16;
    machine.cpu.sp = 0x1fa;
    machine.cpu.x = slot as u16;
    Some(machine)
}

fn marker29_state() -> ZeldaState {
    let mut native = ZeldaState::new();
    let mut image = native.wram_image().into_owned();
    image[0x10] = 7;
    image[0x11] = 0;
    image[0x1b] = 1;
    image[0xfc1] = 0;
    image[0xfd8..0xfdc].copy_from_slice(&[0x40, 0, 0x40, 0]);
    image[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
    image[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
    native.import_wram_image(&image);
    native.sync_native_game_state_from_ram();
    native.set_lag_free(true);
    native
}

#[test]
fn marker30_bank1e_axis_whole_body_matches_rom() {
    marker30_axis_cases(0x1eff34);
}

#[test]
fn marker30_bank1d_axis_whole_body_matches_rom() {
    marker30_axis_cases(0x1de95d);
}

#[test]
fn marker30_bank1e_xyz_whole_body_matches_rom() { marker30_wrapper_cases(0x1eff1e,0x1eff28,0x1eff34,0); }
#[test]
fn marker30_bank1e_xy_whole_body_matches_rom() { marker30_wrapper_cases(0x1eff21,0x1eff28,0x1eff34,1); }
#[test]
fn marker30_bank1e_x_whole_body_matches_rom() { marker30_wrapper_cases(0x1eff28,0x1eff34,0x1eff34,2); }
#[test]
fn marker30_bank1d_xyz_whole_body_matches_rom() { marker30_wrapper_cases(0x1de948,0x1de952,0x1de95d,0); }
#[test]
fn marker30_bank1d_xy_whole_body_matches_rom() { marker30_wrapper_cases(0x1de94b,0x1de952,0x1de95d,1); }
#[test]
fn marker30_bank1d_x_whole_body_matches_rom() { marker30_wrapper_cases(0x1de952,0x1de95d,0x1de95d,2); }

fn marker30_wrapper_cases(entry:u32,end:u32,axis:u32,kind:u8) {
    for k in [0,15] {
        for velocity in [0,1,15,16,127,128,255] {
            let mut native=marker29_state();
            native.write_wram_byte(0xfa0,k as u8);
            for a in [0xd40,0xd50,0xf80] {native.write_wram_byte(a+k,velocity);}
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,entry,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(entry,machine,
                |s|s.cpu.k==(entry>>16)as u8 && (entry as u16..end as u16).contains(&s.cpu.pc),
                ||match kind {
                    0=>native.sprite_move_xyz(k, match axis { 0x06e93e => SpriteMoveEntry::Rom06E93E, 0x05fa00 => SpriteMoveEntry::Rom05FA00, 0x1eff34 => SpriteMoveEntry::Rom1EFF34, 0x1de95d => SpriteMoveEntry::Rom1DE95D, _ => unreachable!() }),
                    1=>native.sprite_move_xy(k, match axis { 0x06e93e => SpriteMoveEntry::Rom06E93E, 0x05fa00 => SpriteMoveEntry::Rom05FA00, 0x1eff34 => SpriteMoveEntry::Rom1EFF34, 0x1de95d => SpriteMoveEntry::Rom1DE95D, _ => unreachable!() }),
                    _=>native.sprite_move_x(k, match axis { 0x06e93e => SpriteMoveEntry::Rom06E93E, 0x05fa00 => SpriteMoveEntry::Rom05FA00, 0x1eff34 => SpriteMoveEntry::Rom1EFF34, 0x1de95d => SpriteMoveEntry::Rom1DE95D, _ => unreachable!() }),
                });
        }
    }
}

fn marker30_axis_cases(entry: u32) {
    for slot in [0,15] {
        for horizontal in [false,true] {
            for velocity in 0..=255 {
                for (position,fraction) in [(0u16,0u8),(255,255),(0xffff,15)] {
                    let mut native=marker29_state();
                    let offset=if horizontal {16}else{0};
                    for (address,value) in [(0xd40,velocity),(0xd60,fraction),
                        (0xd00,position as u8),(0xd20,(position>>8)as u8)] {
                        native.write_wram_byte(address+offset+slot,value);
                    }
                    native.sync_native_game_state_from_ram();
                    let Some(machine)=marker29_machine(&native,entry,slot+offset)else{return};
                    let source=crate::cycle_ledger::assert_rom_marker_work(entry,machine,
                        |s|s.cpu.k==(entry>>16)as u8 && (entry as u16..entry as u16+0x2e).contains(&s.cpu.pc),
                        ||native.sprite_move_axis(slot,horizontal, match entry { 0x06e93e => SpriteMoveEntry::Rom06E93E, 0x05fa00 => SpriteMoveEntry::Rom05FA00, 0x1eff34 => SpriteMoveEntry::Rom1EFF34, 0x1de95d => SpriteMoveEntry::Rom1DE95D, _ => unreachable!() }));
                    for address in [0xd00,0xd20,0xd60] {
                        assert_eq!(native.read_wram_byte(address+offset+slot),source.snes().ram[address+offset+slot]);
                    }
                }
            }
        }
    }
    // The shared native recoil helper represents these two distinct ROM
    // callers. Check the actual XY call through both unmodified bodies.
    for k in [0,15] {
        for (x,y) in [(16u8,240u8),(240,16)] {
            let mut native=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x99),(0xea0,8),(0xcd2,0x80),
                (0xf40,x),(0xf30,y),(0xd00,0x40),(0xd10,0x40)] {native.write_wram_byte(a+k,v);}
            native.write_wram_byte(0xfa0,k as u8);native.sync_native_game_state_from_ram();
            let caller=if entry==0x1eff34 {0x1efe9b}else{0x1de8c5};
            let Some(machine)=marker29_machine(&native,caller,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(entry,machine,
                |s|s.cpu.k==(entry>>16)as u8 && (entry as u16..entry as u16+0x2e).contains(&s.cpu.pc),
                ||{native.sprite_return_if_recoiling(k, match entry { 0x1eff34 => SpriteRecoilEntry::Rom1EFE9B, 0x1de95d => SpriteRecoilEntry::Rom1DE8C5, _ => unreachable!() });});
        }
    }
}

#[test]
fn marker30_pengator_whole_body_matches_rom() {
    for k in [0, 15] {
        for direction in 0..4 {
            for (state, ai, delay, frame, z, hit, wall) in [
                (8,0,0,0,0,0,0),(8,0,0,0,0,1,0),(8,0,0,0,0,0,1),
                (9,0,0,0,0,0,0),(9,1,0,0,0,0,0),(9,1,0,1,0,0,0),
                (9,2,0,0,0,0,0),(9,2,1,0,0,0,0),(9,2,5,0,0,0,0),
                (9,2,15,0,0,0,0),(9,3,0,0,0,0,0),(9,3,0,1,0,0,0),
                (9,3,0,0,0x80,0,0),(9,0,0,0,0,1,0)] {
                let mut native = marker29_state();
                for (a, v) in [(0xdd0, state), (0xe20, 0x99), (0xde0, direction),
                    (0xd90, 1), (0xea0, hit), (0xe70, wall), (0xd10, 0x40),
                    (0xd00, 0x40),(0xd80,ai),(0xdf0,delay),(0xf70,z)] { native.write_wram_byte(a + k, v); }
                native.write_wram_byte(0x1a,frame);
                native.write_wram_byte(0xfa0,k as u8);
                native.write_wram_word(0x20,0x180);
                native.write_wram_word(0x22,0x180);
                native.sync_native_game_state_from_ram();
                let Some(machine) = marker29_machine(&native, 0x1ea196, k) else { return };
                let source = crate::cycle_ledger::assert_rom_marker_work(0x1e8b19, machine,
                    |s| (s.cpu.k == 0x1e && ([(0xa196,0xa1e2),(0xa1ea,0xa1f5),
                        (0xa1fb,0xa240),(0xa244,0xa261),(0xa271,0xa2b5)].iter()
                        .any(|&(a,b)|(a..b).contains(&s.cpu.pc))
                        // Plain FF19 RTS returns from recoil; after its two
                        // PLAs it returns from this fixture's Pengator frame.
                        || s.cpu.pc==0xff18 || (s.cpu.pc==0xff19 && s.cpu.sp==0x1fa)))
                        || (s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc)), || {
                        let _scope = crate::cycle_ledger::routine(0x1e8b19);
                        native.sprite_99_pengator(k);
                    });
                for a in [0xdc0, 0xd80, 0xd40, 0xd50] {
                    assert_eq!(native.read_wram_byte(a+k), source.snes().ram[a+k]);
                }
            }
        }
    }
}

#[test]
fn marker30_pengator_local_dispatch_matches_rom() {
    let mut native=marker29_state();
    for (a,v) in [(0xdd0,9),(0xe20,0x99),(0xd10,0x40),(0xd00,0x40)] {native.write_wram_byte(a,v);}
    native.write_wram_word(0x20,0x180);native.write_wram_word(0x22,0x180);
    native.sync_native_game_state_from_ram();
    let Some(machine)=marker29_machine(&native,0x1ea196,0)else{return};
    crate::cycle_ledger::assert_rom_marker_work(0x008781,machine,
        |s|s.cpu.k==0 && (0x8781..0x8784).contains(&s.cpu.pc),|| {
            let _scope=crate::cycle_ledger::routine(0x1e8b19);native.sprite_99_pengator(0);
        });
}

#[test]
fn marker29_fall1_tail_matches_rom() {
    for k in [0, 15] {
        for delay in [0, 1, 8, 31] {
            for x in [0x40u16, 0x400] {
                let mut native = marker29_state();
                for (a,v) in [(0xdd0,1),(0xdf0,delay),(0xd10,x as u8),
                    (0xd30,(x>>8) as u8),(0xd00,0x40),(0xcaa,0x80)] {
                    native.write_wram_byte(a+k,v);
                }
                native.write_wram_word(0xfd8,x);
                native.sync_native_game_state_from_ram();
                let Some(machine) = marker29_machine(&native, 0x06852e, k) else { return };
                let source = crate::cycle_ledger::assert_rom_marker_work(0x0684e2, machine,
                    |s| s.cpu.k == 6 && (0x852e..0x8543).contains(&s.cpu.pc), || {
                        let _scope = crate::cycle_ledger::routine(0x0684e2);
                        native.sprite_module_fall1(k);
                    });
                assert_eq!(native.read_wram_byte(0xdd0+k),source.snes().ram[0xdd0+k]);
            }
        }
    }
}

#[test]
fn marker29_absorbable_selection_matches_rom() {
    for k in [0,15] {
        for ty in 0xd8..=0xe6 {
          for transient in [false, true] {
            for (sorting, indoors, chr, delay, e, subtype, stunned) in [
                (0,1,0,0,0,0,0), (1,0,0,1,0,1,0), (0,0,3,0,0,0,0),
                (0,1,0,0,1,0,0), (0,1,0,0,0,0,2),
                (0,1,0,0,0,0,1), (0,1,0,0,0,0,3)] {
                let mut native = marker29_state();
                for (a,v) in [(0xdd0,9),(0xe20,ty),(0xe10,delay),(0xe90,e),
                    (0xe30,subtype),(0xb58,stunned),(0xd10,0x40),(0xd00,0x40)] {
                    native.write_wram_byte(a+k,v);
                }
                for (a,v) in [(0xfb3,sorting),(0x1b,indoors),(0xfc6,chr)] {
                    native.write_wram_byte(a,v);
                }
                native.sync_native_game_state_from_ram();
                let entry = if transient { 0x06d22f } else { 0x06d232 };
                let Some(machine) = marker29_machine(&native,entry,k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(entry,machine,
                    |s| {
                        let numbered=s.cart.read(6,0xd144+u16::from(ty),0)!=0;
                        let size=s.cart.read(6,0xd135+u16::from(ty),0);
                        let new_tail=numbered || size!=2 || (ty==0xe6 && subtype==1);
                        s.cpu.k==6 && (((0xd22f..0xd28a).contains(&s.cpu.pc)
                            && !(e!=0 && chr<3 && s.cpu.pc>=0xd288))
                            || (0xd2fa..0xd363).contains(&s.cpu.pc)
                            || (0xdd40..0xddaf).contains(&s.cpu.pc)
                            || (new_tail && ((0xdc4c..0xdc54).contains(&s.cpu.pc)||s.cpu.pc==0xdc64)))
                    }, || {
                        native.sprite_draw_absorbable_transient(k,transient);
                    });
            }
          }
        }
    }
}

#[test]
fn marker29_far_damage_wrapper_matches_rom() {
    for k in [0,15] {
        for (disabled, hit, floor) in [(1,0,0),(0,1,0),(0,0,1)] {
            let mut native=marker29_state();
            for (a,v) in [(0xef0,hit),(0xf20,floor)] {native.write_wram_byte(a+k,v);}
            native.write_wram_byte(0x37b,disabled);
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x06f121,k) else {return};
            crate::cycle_ledger::assert_rom_marker_work(0x06f121,machine,
                |s|s.cpu.k==6 && (0xf121..0xf129).contains(&s.cpu.pc),||{
                    native.sprite_check_damage_to_link(k, SpriteDamageToEntry::Rom06F121);
                });
        }
    }
}

#[test]
fn marker29_damage_trampoline_matches_rom() {
    let mut native=marker29_state();
    native.write_wram_byte(0x37b,1);
    native.sync_native_game_state_from_ram();
    let Some(machine)=marker29_machine(&native,0x1ec26b,0) else {return};
    crate::cycle_ledger::assert_rom_marker_work(0x1ec26b,machine,
        |s|s.cpu.k==0x1e && (0xc26b..0xc270).contains(&s.cpu.pc),||{
            native.sprite_check_damage_to_link(0, SpriteDamageToEntry::Rom1EC26B);
        });
}

#[test]
fn marker31_tree_top_matches_rom() {
    for k in [0usize, 15] {
        for subtype in 0..=6 {
            let mut native = marker29_state();
            native.write_wram_byte(0xe80 + k, subtype);
            native.write_wram_word(0xfd8, 0x40);
            native.write_wram_word(0xfda, 0x40);
            native.sync_native_game_state_from_ram();
            let Some(machine) = marker29_machine(&native, 0x05fe6f, k) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x05fe6f, machine,
                |s| s.cpu.k == 5 && (0xfe6f..0xff39).contains(&s.cpu.pc), || {
                    native.dash_tree_top_draw(k);
                });
        }
    }
}

#[test]
fn marker35_dynamic_spawn_whole_body_matches_rom() {
    for k in [0, 15] {
        for limit in [7, 13, 15] {
            for free in [-1, 0, 5] {
                for indoors in [0, 1] {
                    let mut native = marker29_state();
                    for j in 0..16 { native.write_wram_byte(0xdd0+j, 9); }
                    if free >= 0 { native.write_wram_byte(0xdd0+free as usize, 0); }
                    native.write_wram_byte(0x1b, indoors);
                    native.sync_native_game_state_from_ram();
                    let Some(mut machine) = marker29_machine(&native,0x1df65f,k) else {return};
                    machine.cpu.a=0xdb; machine.cpu.y=limit as u16;
                    crate::cycle_ledger::assert_rom_marker_work(0x1df65f,machine,
                        |s|s.cpu.k==0x1d && (0xf65f..0xf6cf).contains(&s.cpu.pc),|| {
                            let mut info=super::sprite::SpriteSpawnInfo::default();
                            native.sprite_spawn_dynamically_ex(k,0xdb,&mut info,limit);
                        });
                }
            }
        }
    }
}

#[test]
fn marker35_load_properties_whole_body_matches_rom() {
    for k in [0,15] {
        for ty in [0, 0x25, 0x7f, 0x80, 0x8d, 0xdb, 0xe6, 0xf2] {
            for indoors in [0,1] {
                let mut native=marker29_state();
                native.write_wram_byte(0xe20+k,ty);native.write_wram_byte(0x1b,indoors);
                native.sync_native_game_state_from_ram();
                let Some(machine)=marker29_machine(&native,0x0db818,k)else{return};
                crate::cycle_ledger::assert_rom_marker_work(0x0db818,machine,
                    |s|s.cpu.k==0x0d && (0xb818..0xb871).contains(&s.cpu.pc),|| {
                        native.sprite_prep_load_properties(k);
                    });
            }
        }
    }
}

#[test]
fn marker35_thief_spill_whole_body_matches_rom() {
    for k in [0,15] {
        for resources in [0, 1, 20] {
            for occupied in [0,4,8] {
                let mut native=marker29_state();
                for j in 0..occupied {native.write_wram_byte(0xdd0+j,9);}
                for a in [0xf343,0xf377,0xf360] {native.write_wram_byte(a,resources);}
                native.write_wram_word(0x20,0x40);native.write_wram_word(0x22,0x40);
                native.sync_native_game_state_from_ram();
                let Some(machine)=marker29_machine(&native,0x1dcb30,k)else{return};
                // Give native the unmodified CPU's PPU-dependent RNG results.
                // The fixture compares prices for the same executed branches.
                use snes::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
                    CpuRasterPosition, RomCpuTimingProbe, SourcePpuReadState};
                let clock=CpuMasterTimeline::at_raster(0,CpuRasterPosition::new(0,0),
                    CpuBusWorkload::default(),CpuFieldTiming::non_interlace(false));
                let mut probe=RomCpuTimingProbe::new(machine.clone(),clock,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                let mut samples=Vec::new();
                for _ in 0..100000 {
                    let cpu=&probe.snes().cpu;
                    if cpu.k==0x0d && cpu.pc==0xba7f {
                        samples.push(crate::RomRandomSample::with_carry(0,cpu.a as u8,cpu.c));
                    }
                    probe.step().unwrap();
                    if probe.snes().cpu.pc==0x8000 {break;}
                }
                assert_eq!(probe.snes().cpu.pc,0x8000);
                native.install_rom_random_replay(samples,0);
                native.rom_random_replay.begin_frame();
                crate::cycle_ledger::assert_rom_marker_work(0x1dcb30,machine,
                    |s|s.cpu.k==0x1d && (0xcb30..0xcbd6).contains(&s.cpu.pc),|| {
                        native.thief_spill_items(k);
                    });
            }
        }
    }
}

#[test]
fn marker35_thin_drawing_whole_body_matches_rom() {
    for k in [0,15] {
        for y in [0x40,0xf0,0xf8,0xff,0x100,0xfff0] {
            for shadow in [0,0x10] {
                let mut native=marker29_state();
                for (a,v) in [(0xe20,0xdb),(0xdd0,9),(0xe60,shadow),(0xcaa,0x80)] {
                    native.write_wram_byte(a+k,v);
                }
                native.write_wram_word(0xfda,y);
                native.sync_native_game_state_from_ram();
                let Some(machine)=marker29_machine(&native,0x06dd40,k)else{return};
                crate::cycle_ledger::assert_rom_marker_work(0x06dd40,machine,
                    |s|s.cpu.k==6 && ((0xdd40..0xddaf).contains(&s.cpu.pc)
                        || (0xdc4c..0xdc54).contains(&s.cpu.pc) || s.cpu.pc==0xdc64),|| {
                        let _scope=crate::cycle_ledger::routine(0x06dd40);
                        native.sprite_draw_thin_and_tall(k);
                    });
            }
        }
    }
}

#[test]
fn marker35_numbered_drawing_whole_body_matches_rom() {
    for k in [0,15] {
        for a in [1,2,3,4,5] {
            for head in [0,1] {
                for y in [0x40,0xf8,0xfff0] {
                    let mut native=marker29_state();
                    native.write_wram_byte(0xeb0+k,head);
                    native.write_wram_byte(0xcaa+k,0x80);
                    native.write_wram_word(0xfda,y);
                    native.sync_native_game_state_from_ram();
                    let Some(mut machine)=marker29_machine(&native,0x06d2fa,k)else{return};
                    machine.cpu.a=a;
                    crate::cycle_ledger::assert_rom_marker_work(0x06d2fa,machine,
                        |s|s.cpu.k==6 && ((0xd2fa..0xd363).contains(&s.cpu.pc)||s.cpu.pc==0xdc64),|| {
                            let _scope=crate::cycle_ledger::routine(0x06d2fa);
                            native.sprite_draw_numbered_absorbable(k,i32::from(a));
                        });
                }
            }
        }
    }
}

#[test]
fn marker35_tree_eye_whole_body_matches_rom() {
    for k in [0,15] {
        for head in [0,1] {
            for direction in 0..5 {
                for link_y in [0,0x80] {
                    let mut native=marker29_state();
                    for (a,v) in [(0xe20,0x25),(0xdd0,9),(0xeb0,head),(0xde0,direction),
                        (0xd90,0x40),(0xdb0,0x40)] {native.write_wram_byte(a+k,v);}
                    native.write_wram_word(0x20,link_y);native.write_wram_word(0x22,0x40);
                    native.sync_native_game_state_from_ram();
                    let Some(machine)=marker29_machine(&native,0x1dfb0a,k)else{return};
                    crate::cycle_ledger::assert_rom_marker_work(0x1dfb0a,machine,
                        |s|s.cpu.k==0x1d && (0xfb0a..0xfb86).contains(&s.cpu.pc),|| {
                            native.talking_tree_eye(k);
                        });
                }
            }
        }
    }
}

#[test]
fn marker35_relative_predicate_bodies_match_rom() {
    for k in [0,15] {
        for (link, sprite, height) in [(0u16,0u16,0u8),(0xff,0x100,0),(0xff,0x100,255),
            (0x7fff,0x8000,0),(0xffff,0,255),(0x100,0xff,2)] {
            for entry in [0x06ead1,0x06eae8] {
                let mut native=marker29_state();
                native.write_wram_word(0x20,link);native.write_wram_word(0x22,link);
                for (a,v) in [(0xd10,sprite as u8),(0xd30,(sprite>>8)as u8),
                    (0xd00,sprite as u8),(0xd20,(sprite>>8)as u8),(0xf70,height)] {
                    native.write_wram_byte(a+k,v);
                }
                native.write_wram_byte(0xfa0,k as u8);
                    native.sync_native_game_state_from_ram();
                let Some(machine)=marker29_machine(&native,entry,k)else{return};
                crate::cycle_ledger::assert_rom_marker_work(entry,machine,
                    |s|s.cpu.k==6 && if entry==0x06ead1 {(0xead1..0xeae4).contains(&s.cpu.pc)}
                        else {(0xeae8..0xeb0a).contains(&s.cpu.pc)},|| {
                        if entry==0x06ead1 {native.sprite_is_right_of_link(k);}
                        else {native.sprite_is_below_link(k);}
                    });
            }
        }
    }
}

#[test]
fn marker35_phasing_body_matches_rom() {
    for k in [0,15] {
        for timer in [0,1,2,3,0x27,0x28,255] {
            for frame in [0,1] {
                for submodule in [0,1] {
                    let mut native=marker29_state();
                    for (a,v) in [(0xb58,timer),(0xdd0,9),(0xd10,0x40),(0xd00,0x40)] {
                        native.write_wram_byte(a+k,v);
                    }
                    native.write_wram_byte(0x1a,frame);native.write_wram_byte(0x11,submodule);
                    native.write_wram_byte(0xfa0,k as u8);
                    native.sync_native_game_state_from_ram();
                    let Some(machine)=marker29_machine(&native,0x06d0ed,k)else{return};
                    crate::cycle_ledger::assert_rom_marker_work(0x06d0ed,machine,
                        |s|s.cpu.k==6 && (0xd0ed..0xd116).contains(&s.cpu.pc) && s.cpu.sp<=0x1fa,|| {
                            native.sprite_return_if_phasing_out(k);
                        });
                }
            }
        }
    }
}

#[test]
fn marker35_bounce_body_matches_rom() {
    for k in [0,15] {
        for mask in 0..16 {
            let mut native=marker29_state();
            for (a,v) in [(0xe70,mask),(0xd50,0x80),(0xd40,0x7f)] {native.write_wram_byte(a+k,v);}
            native.write_wram_byte(0xfa0,k as u8);
                    native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x06d9c0,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x06d9c0,machine,
                |s|s.cpu.k==6 && (0xd9c0..0xd9d5).contains(&s.cpu.pc),|| {
                    native.sprite_bounce_off_wall(k);
                });
        }
    }
}

#[test]
fn marker35_absorbable_control_body_matches_rom() {
    for k in [0,15] {
        for (z, velocity, tile, kind, delay) in [(10,0,0,0xdb,0),(0,0xf0,0,0xdb,1),
            (0,0xc0,0,0xe4,1),(0,0xc0,8,0xdb,1),(0,0xc0,9,0xdb,1)] {
            let mut native=marker29_state();
            for j in 0..16 {native.write_wram_byte(0xdd0+j,9);}
            for (a,v) in [(0xd10,0x40),(0xd00,0x40),(0xf70,z),(0xf80,velocity),
                (0xe20,kind),(0xee0,delay),(0xf10,32)] {native.write_wram_byte(a+k,v);}
            native.write_wram_byte(0x1f9c2+k,tile);
            native.write_wram_word(0x20,0x400);native.write_wram_word(0x22,0x400);
            native.write_wram_byte(0xfa0,k as u8);
                    native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x06d051,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x06d051,machine,
                |s|s.cpu.k==6 && (0xd051..0xd0ed).contains(&s.cpu.pc),|| {
                    native.sprite_absorbable_main(k);
                });
        }
    }
}

#[test]
fn marker35_single_layer_body_matches_rom() {
    for k in [0,15] {
        for (flags2, flags4, flags5, height, velocity, collision) in [
            (0,0,0,0,0,0),(0,0x80,0,0,1,0),(0,0,0,0,255,1),
            (0x20,0,0,0,0,0),(0,0,0x80,0,0,0),(0,0,0,1,0,0)] {
            let mut native=marker29_state();
            for (a,v) in [(0xe40,flags2),(0xf60,flags4),(0xbe0,flags5),(0xf70,height),
                (0xd40,velocity),(0xd50,velocity),(0xdd0,9),(0xd10,0x40),(0xd00,0x40)] {
                native.write_wram_byte(a+k,v);
            }
            native.write_wram_byte(0x46c,collision);
            native.write_wram_byte(0xfa0,k as u8);
                    native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x06e4db,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x06e4db,machine,
                |s|s.cpu.k==6 && (0xe4db..0xe5b8).contains(&s.cpu.pc),|| {
                    native.sprite_check_tile_collision_single_layer(k);
                });
        }
    }
}

#[test]
fn marker35_direction_probe_bodies_match_rom() {
    let mut saw_nested_steps=false;
    for k in [0,15] {
        for entry in [0x06e5b8,0x06e5ee] {
            for tile in [0,1,0x20,0x68] {
                for amount in [0,1] {
                  for target in [entry,entry+0x22] {
                    let mut native=marker29_state();
                    for (a,v) in [(0xd10,0x40),(0xd00,0x40),(0xea0,amount),(0xdd0,9)] {
                        native.write_wram_byte(a+k,v);
                    }
                    let mut image=native.wram_image().into_owned();
                    image[0x12000..0x14000].fill(tile);native.import_wram_image(&image);
                    native.write_wram_byte(0xfa0,k as u8);
                    native.sync_native_game_state_from_ram();
                    let Some(mut machine)=marker29_machine(&native,entry,k)else{return};
                    let direction=if entry==0x06e5b8 {2}else{0};machine.cpu.y=direction;
                    let source=crate::cycle_ledger::assert_rom_marker_work(target,machine,
                        |s|s.cpu.k==6 && (entry as u16..entry as u16+0x36).contains(&s.cpu.pc)
                            && if target==entry {s.cpu.sp>=0x1fa}else{s.cpu.sp<0x1fa},|| {
                            if entry==0x06e5b8 {native.sprite_check_for_tile_in_direction_horizontal(k,i32::from(direction));}
                            else {native.sprite_check_for_tile_in_direction_vertical(k,i32::from(direction));}
                        });
                    if amount!=0 && source.snes().ram[0xe70+k]!=0 {saw_nested_steps=true;}
                  }
                }
            }
        }
    }
    assert!(saw_nested_steps,"probe fixtures must execute the two nested step calls");
}

#[test]
fn marker37_lightning_matches_rom() {
    for (slot, countdown, x, y, frame, room, link_x, link_y) in [
        (0usize, 8u8, 40u16, 48u16, 0u8, 0x20u8, 40u16, 48u16),
        (1, 4, 40, 48, 1, 0, 41, 49),
        (2, 0, 40, 48, 2, 0, 200, 48),
        (0, 4, 40, 48, 0, 0, 40, 200),
        (29, 0, 256, 48, 0, 0, 0, 0),
        (0, 0, 40, 256, 0, 0, 0, 0),
        (0, 4, 40, 48, 0x80, 0, 40, 48),
    ] {
        let mut native = fresh_state();
        native.write_wram_word(0x90, 0x800); native.write_wram_word(0x92, 0xa20);
        native.write_wram_byte(0x1f800 + slot, 9);
        native.write_wram_byte(0x1f90e + slot, countdown);
        native.write_wram_byte(0x1f83c + slot, x as u8);
        native.write_wram_byte(0x1f878 + slot, (x >> 8) as u8);
        native.write_wram_byte(0x1f81e + slot, y as u8);
        native.write_wram_byte(0x1f85a + slot, (y >> 8) as u8);
        native.write_wram_byte(0x1f92c + slot, 0);
        native.write_wram_byte(0x1a, frame); native.write_wram_byte(0x48e, room);
        native.write_wram_word(0x22, link_x); native.write_wram_word(0x20, link_y);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native, 0x09b429, slot) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6, machine,
            |s| s.cpu.k == 9 && ((0xb429..0xb496).contains(&s.cpu.pc)
                || (0xb5d6..0xb5de).contains(&s.cpu.pc)
                || (0xb70c..0xb714).contains(&s.cpu.pc)
                || (0xb459..0xb496).contains(&s.cpu.pc)), || {
                let _parent = crate::cycle_ledger::routine(0x09b0b6);
                native.garnish09_lightning_trail(slot);
            });
    }
}

#[test]
fn marker37_kholdstare_matches_rom() {
    for (slot, countdown, x, y, frame, room, link_x, link_y) in [
        (0usize, 8u8, 40u16, 48u16, 0u8, 0x20u8, 40u16, 48u16),
        (1, 4, 40, 48, 1, 0, 41, 49),
        (2, 0, 40, 48, 2, 0, 200, 48),
        (0, 4, 40, 48, 0, 0, 40, 200),
        (29, 0, 256, 48, 0, 0, 0, 0),
        (0, 0, 40, 256, 0, 0, 0, 0),
        (0, 4, 40, 48, 0x80, 0, 40, 48),
    ] {
        let mut native = fresh_state();
        native.write_wram_word(0x90, 0x800); native.write_wram_word(0x92, 0xa20);
        native.write_wram_byte(0x1f800 + slot, 8);
        native.write_wram_byte(0x1f90e + slot, countdown);
        native.write_wram_byte(0x1f83c + slot, x as u8);
        native.write_wram_byte(0x1f878 + slot, (x >> 8) as u8);
        native.write_wram_byte(0x1f81e + slot, y as u8);
        native.write_wram_byte(0x1f85a + slot, (y >> 8) as u8);
        native.write_wram_byte(0x1f92c + slot, 0);
        native.write_wram_byte(0x1a, frame); native.write_wram_byte(0x48e, room);
        native.write_wram_word(0x22, link_x); native.write_wram_word(0x20, link_y);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native, 0x09b4c6, slot) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6, machine,
            |s| s.cpu.k == 9 && ((0xb4c6..0xb4fb).contains(&s.cpu.pc)
                || (0xb5d6..0xb5de).contains(&s.cpu.pc)
                || (0xb70c..0xb714).contains(&s.cpu.pc)
                ), || {
                let _parent = crate::cycle_ledger::routine(0x09b0b6);
                native.garnish08_kholdstare_trail(slot);
            });
    }
}

#[test]
fn marker37_smoke_matches_rom() {
    for (slot, countdown, x, y, frame, room, link_x, link_y) in [
        (0usize, 8u8, 40u16, 48u16, 0u8, 0x20u8, 40u16, 48u16),
        (1, 4, 40, 48, 1, 0, 41, 49),
        (2, 0, 40, 48, 2, 0, 200, 48),
        (0, 4, 40, 48, 0, 0, 40, 200),
        (29, 0, 256, 48, 0, 0, 0, 0),
        (0, 0, 40, 256, 0, 0, 0, 0),
        (0, 4, 40, 48, 0x80, 0, 40, 48),
    ] {
        let mut native = fresh_state();
        native.write_wram_word(0x90, 0x800); native.write_wram_word(0x92, 0xa20);
        native.write_wram_byte(0x1f800 + slot, 10);
        native.write_wram_byte(0x1f90e + slot, countdown);
        native.write_wram_byte(0x1f83c + slot, x as u8);
        native.write_wram_byte(0x1f878 + slot, (x >> 8) as u8);
        native.write_wram_byte(0x1f81e + slot, y as u8);
        native.write_wram_byte(0x1f85a + slot, (y >> 8) as u8);
        native.write_wram_byte(0x1f92c + slot, 0);
        native.write_wram_byte(0x1a, frame); native.write_wram_byte(0x48e, room);
        native.write_wram_word(0x22, link_x); native.write_wram_word(0x20, link_y);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native, 0x09b3ee, slot) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6, machine,
            |s| s.cpu.k == 9 && ((0xb3ee..0xb419).contains(&s.cpu.pc)
                || (0xb5d6..0xb5de).contains(&s.cpu.pc)
                || (0xb70c..0xb714).contains(&s.cpu.pc)
                || (0xb459..0xb496).contains(&s.cpu.pc)), || {
                let _parent = crate::cycle_ledger::routine(0x09b0b6);
                native.garnish0_a_cannon_smoke(slot);
            });
    }
}

#[test]
fn marker37_bat_flame_matches_rom() {
    for (slot, countdown, x, y, frame, room, link_x, link_y) in [
        (0usize, 8u8, 40u16, 48u16, 0u8, 0x20u8, 40u16, 48u16),
        (1, 4, 40, 48, 1, 0, 41, 49),
        (2, 0, 40, 48, 2, 0, 200, 48),
        (0, 4, 40, 48, 0, 0, 40, 200),
        (29, 0, 256, 48, 0, 0, 0, 0),
        (0, 0, 40, 256, 0, 0, 0, 0),
        (0, 4, 40, 48, 0x80, 0, 40, 48),
    ] {
        let countdown = if countdown < 8 { 16 } else { countdown };
        let mut native = fresh_state();
        native.write_wram_word(0x90, 0x800); native.write_wram_word(0x92, 0xa20);
        native.write_wram_byte(0x1f800 + slot, 16);
        native.write_wram_byte(0x1f90e + slot, countdown);
        native.write_wram_byte(0x1f83c + slot, x as u8);
        native.write_wram_byte(0x1f878 + slot, (x >> 8) as u8);
        native.write_wram_byte(0x1f81e + slot, y as u8);
        native.write_wram_byte(0x1f85a + slot, (y >> 8) as u8);
        native.write_wram_byte(0x1f92c + slot, 0);
        native.write_wram_byte(0x1a, frame); native.write_wram_byte(0x48e, room);
        native.write_wram_word(0x22, link_x); native.write_wram_word(0x20, link_y);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native, 0x09b306, slot) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6, machine,
            |s| s.cpu.k == 9 && ((0xb306..0xb33f).contains(&s.cpu.pc)
                || (0xb5d6..0xb5de).contains(&s.cpu.pc)
                || (0xb70c..0xb714).contains(&s.cpu.pc)
                || (0xb459..0xb496).contains(&s.cpu.pc)), || {
                let _parent = crate::cycle_ledger::routine(0x09b0b6);
                native.garnish10_ganon_bat_flame(slot);
            });
    }
}

#[test]
fn marker37_damage_body_matches_rom() {
    // Guards, both hitbox forms, shield outcomes and every special tail.
    for (floor, flags4, flags2, aux, bunny, lifting, shield, flags5, type_, dir, facing, frames, x) in [
        (1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0x95u8,0u8,0u8,0u8,40u16),
        (0,0,0x80,0,0,0,0,0,0x95,0,0,0,40),
        (0,0,0,0,0,0,0,0,0x95,0,0,0,200),
        (0,0,0,1,0,0,0,0,0x95,0,0,0,40),
        (0,0,0,0,1,0,0,0,0x95,0,0,0,40),
        (0,0,0,0,0,0x80,0,0,0x95,0,0,0,40),
        (0,0,0,0,0,0,0,0x20,0x95,0,4,0,40),
        (0,0,0,0,0,0,1,0x20,0x95,0,4,0,40),
        (0,0,0,0,0,0,1,0x20,0x9b,0,4,0,40),
        (0,0,0,0,0,0,1,0x20,0x1b,0,4,0,40),
        (0,0,0,0,0,0,1,0x20,0x0c,0,4,0,40),
        (0,0,0,0,0,0,1,0x20,0x08,0,4,0,40),
        (0,0,0,0,0,0,1,0x20,0x95,1,2,9,40),
        (0,0,0,0,0,0,1,0x20,0x95,0,2,0,40),
        (0,1,0x80,0,0,0,0,0,0x95,0,0,0,40),
        (0,1,0,0,0,0,0,0,0x0c,0,0,0,40),
    ] {
        let mut native = fresh_state();
        let slot = 0;
        for (a,v) in [(0xf20,floor),(0xf60,flags4),(0xe40,flags2),(0x4d,aux),
            (0x2e0,bunny),(0x308,lifting),(0x1f35a,shield),(0xbe0,flags5),
            (0xe20,type_),(0xde0,dir),(0x2f,facing),(0x3c,frames),(0xdd0,9)] {
            native.write_wram_byte(a,v);
        }
        native.write_wram_word(0x22,x); native.write_wram_word(0x20,40);
        native.write_wram_byte(0xd10,40); native.write_wram_byte(0xd00,40);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,40);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x06f154,slot) else {return};
        // The F154 prefix is independently priced from the F15C child.
        let source = crate::cycle_ledger::assert_rom_marker_work(0x06f15c,machine,
            |s|s.cpu.k==6 && ((0xf15c..0xf1f6).contains(&s.cpu.pc) || (0xe25a..0xe26e).contains(&s.cpu.pc))
                && !(floor!=0 && s.cpu.pc==0xf1d7 || floor!=0 && s.cpu.pc==0xf1d8),||{
                native.sprite_check_damage_to_link_same_layer_marked(slot);
            });
        assert_eq!(native.sprite_slot_view(slot).state(),source.snes().ram[0xdd0]);
    }
}

#[test]
fn marker37_fast_hitbox_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,31),(15,32),(0,0x80),(15,0xff)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0xfaa, value); native.write_wram_byte(0xfab, value); native.write_wram_byte(0xe20 + slot, if value == 0 {0x6a} else {1});
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x06f645,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x06f645,machine,
            |s|s.cpu.k==0x6 && (0xf645..0xf688).contains(&s.cpu.pc),||{
                native.sprite_do_hit_boxes_fast_marked(slot, &mut crate::types::SpriteHitBox { r0_xlo:0,r8_xhi:0,r1_ylo:0,r9_yhi:0,r2:0,r3:0,r4_spr_xlo:0,r10_spr_xhi:0,r5_spr_ylo:0,r11_spr_yhi:0,r6_spr_xsize:0,r7_spr_ysize:0 });
            });
    }
}

#[test]
fn marker37_babusu_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,31),(15,32),(0,0x80),(15,0xff)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0xdc0 + slot, if value == 0xff {0xff} else {value % 4});
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x0dbd20,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x0dbd20,machine,
            |s|s.cpu.k==0xd && (0xbd20..0xbd46).contains(&s.cpu.pc),||{
                native.babusu_draw(slot);
            });
    }
}

#[test]
fn marker37_large_shadow_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,31),(15,32),(0,0x80),(15,0xff)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0xf70 + slot, value);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x1dd1af,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x1dd1af,machine,
            |s|s.cpu.k==0x1d && (0xd1af..0xd1fd).contains(&s.cpu.pc),||{
                native.sprite_draw_large_shadow2(slot);
            });
    }
}

#[test]
fn marker37_same_layer_entry_matches_rom() {
    for floor in [0u8,1] {
        let mut native=fresh_state();
        native.write_wram_byte(0xf20,floor);native.write_wram_byte(0xe40,0x80);
        native.sync_native_game_state_from_ram();native.set_lag_free(true);
        let Some(machine)=marker29_machine(&native,0x06f154,0)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x06f154,machine,
            |s|s.cpu.k==6 && ((0xf154..0xf15c).contains(&s.cpu.pc)
                || floor!=0 && (0xf1d7..0xf1d9).contains(&s.cpu.pc)),||{
                    native.sprite_check_damage_to_link_same_layer_marked(0);
                });
    }
}

#[test]
fn marker37_probe_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,2),(15,3),(0,4),(15,0x80)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0x1b, value & 1); native.write_wram_byte(0xf20 + slot, value >> 1);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x0dc26e,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x0dc26e,machine,
            |s|s.cpu.k==0xd && (0xc26e..0xc2d1).contains(&s.cpu.pc),||{
                native.probe_check_tile_solidity(slot);
            });
    }
}

#[test]
fn marker37_statue_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,2),(15,3),(0,4),(15,0x80)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0xfa0, slot as u8); native.write_wram_byte(0x1a, value & 1); native.write_wram_byte(0xe20 + 2, if value==0 {0x1c}else{0}); native.write_wram_byte(0xdd0 + 2, if value==1 {8}else{9}); native.write_wram_byte(0xd10 + 2, if value==2 {40}else{200}); native.write_wram_byte(0xd00 + 2, if value==3 {200}else{48});
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x06c277,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x06c277,machine,
            |s|s.cpu.k==0x6 && (0xc277..0xc2e5).contains(&s.cpu.pc),||{
                native.statue_block_sprites(slot);
            });
    }
}

#[test]
fn marker37_hitbox_wrapper_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,2),(15,3),(0,4),(15,0x80)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0xf70 + slot, value);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x0683ea,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x0683ea,machine,
            |s|s.cpu.k==0x6 && (0x83ea..0x83f2).contains(&s.cpu.pc),||{
                native.sprite_setup_hit_box(slot, &mut crate::types::SpriteHitBox { r0_xlo:0,r8_xhi:0,r1_ylo:0,r9_yhi:0,r2:0,r3:0,r4_spr_xlo:0,r10_spr_xhi:0,r5_spr_ylo:0,r11_spr_yhi:0,r6_spr_xsize:0,r7_spr_ysize:0 }, SpriteHitBoxEntry::Rom0683EA);
            });
    }
}

#[test]
fn marker37_red_bari_matches_rom() {
    for (slot, value) in [(0usize,0u8),(15,1),(0,2),(15,3),(0,4),(15,0x80)] {
        let mut native = fresh_state();
        native.write_wram_word(0x90,0x800); native.write_wram_word(0x92,0xa20);
        native.write_wram_byte(0xd10 + slot,40); native.write_wram_byte(0xd00 + slot,48);
        native.write_wram_word(0xfd8,40); native.write_wram_word(0xfda,48);
        native.write_wram_byte(0xdd0 + slot,9);
        native.write_wram_byte(0xdc0 + slot, value & 1);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let Some(machine) = marker29_machine(&native,0x06a3dc,slot) else {return};
        crate::cycle_ledger::assert_rom_marker_work(0x06a3dc,machine,
            |s|s.cpu.k==0x6 && (0xa3dc..0xa3f9).contains(&s.cpu.pc),||{
                native.red_bari_draw(slot);
            });
    }
}

#[test]
fn marker38_agahnim_draw_matches_rom() {
    for (g, head, direction, submodule, x) in [(0u8,0u8,0u8,0u8,0x40u16),
        (11,1,1,0,0x40), (12,13,0,1,0x40), (14,14,1,0,0x40),
        (17,36,0,0,0x40), (0,0,0,0,0x400)] {
        let mut native = marker29_state();
        for (a,v) in [(0xdd0,9),(0xe20,0x7a),(0xdc0,g),(0xeb0,head),(0xde0,direction),(0x11,submodule)] {
            native.write_wram_byte(a,v);
        }
        native.write_wram_word(0xfd8,x); native.write_wram_word(0xfda,0x40);
        native.write_wram_byte(0xd10,x as u8); native.write_wram_byte(0xd30,(x>>8)as u8);
        native.write_wram_byte(0xd00,0x40); native.sync_native_game_state_from_ram();
        let Some(machine) = marker29_machine(&native,0x1ed978,0) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x1ed978,machine,
            |s| s.cpu.k == 0x1e && (0xd978..0xda42).contains(&s.cpu.pc), || { native.agahnim_draw(0); });
    }
}

#[test]
fn marker38_thrown_debris_matches_rom() {
    for (slot, kind, countdown, indoors, chr, x) in [
        (0usize,0u8,28u8,0u8,0u8,40u16), (1,2,0,0,0,40),
        (2,2,31,1,0,40), (29,4,0,1,0,40), (0,0x80,16,0,0,40),
        (0,3,12,0,0,40), (0,0,12,0,3,40), (0,0,12,0,0,0x400)] {
        let mut native = marker29_state();
        for (a,v) in [(0x1f800+slot,22),(0x1f92c+slot,kind),(0x1f90e+slot,countdown),
            (0x1f83c+slot,x as u8),(0x1f878+slot,(x>>8)as u8),(0x1f81e+slot,48),
            (0x1b,indoors),(0xfc6,chr)] { native.write_wram_byte(a,v); }
        native.sync_native_game_state_from_ram();
        let Some(machine) = marker29_machine(&native,0x09f0cb,slot) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6,machine,
            |s| s.cpu.k==9 && (0xf0cb..0xf15c).contains(&s.cpu.pc), || {
                let _scope=crate::cycle_ledger::routine(0x09b0b6);
                native.garnish16_thrown_item_debris(slot);
            });
    }
}

#[test]
fn marker38_mothula_trail_matches_rom() {
    for (slot, sprite) in [(0usize,0u8),(29,15)] {
        let mut native=marker29_state();
        for(a,v)in[(0x1f83c+slot,40),(0x1f81e+slot,48),(0x1f92c+slot,sprite)] {native.write_wram_byte(a,v);}
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x09b6e1,slot)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6,machine,
            |s|s.cpu.k==9 && (0xb6e1..0xb714).contains(&s.cpu.pc),|| {
                let _scope=crate::cycle_ledger::routine(0x09b0b6);native.garnish02_mothula_beam_trail(slot);
            });
    }
}

#[test]
fn marker38_babasu_flash_matches_rom() {
    for(slot,countdown,x)in[(0usize,0u8,40u16),(29,31,40),(0,8,0x400)] {
        let mut native=marker29_state();
        for(a,v)in[(0x1f83c+slot,x as u8),(0x1f878+slot,(x>>8)as u8),
            (0x1f81e+slot,48),(0x1f90e+slot,countdown)] {native.write_wram_byte(a,v);}
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x09b49e,slot)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x09b0b6,machine,
            |s|s.cpu.k==9 && ((0xb49e..0xb4c0).contains(&s.cpu.pc)||(0xb70c..0xb714).contains(&s.cpu.pc)),|| {
                let _scope=crate::cycle_ledger::routine(0x09b0b6);native.garnish07_babasu_flash(slot);
            });
    }
}

#[test]
fn marker39_helmasaur_tail_matches_rom() {
    for (angle, active, start, blink) in [(0u16,0,16,1),(0x80,1,15,1),
        (0x180,1,0,1),(0x1ff,1,15,0),(0,1,15,0)] {
        let mut native=marker29_state();
        for (a,v) in [(0xec0,active),(0xb33,start),(0xb2f,80),(0x31f,blink),
            (0,0x40),(2,0x40),(5,0x20)] {native.write_wram_byte(a,v);}
        native.write_wram_word(0xb2d,angle); native.write_wram_word(0xb31,0);
        native.write_wram_word(0xfd8,0x40); native.write_wram_word(0xfda,0x40);
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x1e8920,0)else{return};
        let mut info=PrepOamCoordsRet{x:0x40,y:0x40,r4:0,flags:0x20};
        crate::cycle_ledger::assert_rom_marker_work(0x1e8920,machine,
            |s|s.cpu.k==0x1e && (0x8920..0x8a85).contains(&s.cpu.pc),
            ||native.king_helmasaur_operate_tail(0,&mut info));
    }
}

#[test]
fn marker39_helmasaur_multiply_delay_matches_rom() {
    let native=marker29_state();
    let Some(machine)=marker29_machine(&native,0x1e8a85,0)else{return};
    crate::cycle_ledger::assert_rom_marker_work(0x1e8a85,machine,
        |s|s.cpu.k==0x1e && (0x8a85..0x8a8e).contains(&s.cpu.pc),|| {
            let _scope=crate::cycle_ledger::routine(0x1e8a85);
            crate::cycle_ledger::blocks(true).block(0x1e8a85);
        });
}

#[test]
fn marker39_trinexx_head_draw_matches_rom() {
    for (k,count,angle,sub) in [(1,1,0u8,0),(2,2,0x40,1),(1,5,0x80,0),(2,9,0xff,0)] {
        let mut native=marker29_state();
        for (a,v) in [(0xd90,0x40),(0xdb0,0x40),(0xe80,count)] {native.write_wram_byte(a+k,v);}
        native.write_wram_byte(0x11,sub);
        for i in 0..count as usize {
            native.write_wram_byte(0x1d10+k*9+i,angle);
            native.write_wram_byte(0x1d50+k*9+i,16);
        }
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x1dbb70,k)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x1dbb70,machine,
            |s|s.cpu.k==0x1d && (0xbb70..0xbc8c).contains(&s.cpu.pc),
            ||native.trinexx_head_draw(k));
    }
}

#[test]
fn marker39_lanmola_draw_matches_rom() {
    for (k,n,state,ai,delay,reverse) in [(0,0x80,9,0,0,false),(1,0,8,0,0,false),
        (2,7,9,5,0,true),(0,2,9,1,24,false),(1,1,9,2,8,false),(2,1,9,2,8,true)] {
        let mut native=marker29_state();
        for (a,v) in [(0xdd0,state),(0xd80,ai),(0xe00,delay),(0xdf0,24),
            (0xd40,if reverse{0xf0}else{0x10}),(0xd50,0x10)] {native.write_wram_byte(a+k,v);}
        native.write_wram_byte(0x1f81e+k,n);
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x05a64a,k)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x05a64a,machine,
            |s|s.cpu.k==5 && (0xa64a..0xa864).contains(&s.cpu.pc),
            ||native.lanmola_draw(k));
    }
}

#[test]
fn marker39_zazak_whole_body_matches_rom() {
    for (state,ai,delay,aux,b) in [(8,0,0,0,0),(9,0,1,0,0),(9,0,0,0,0),
        (9,1,1,0,0),(9,1,0,0,0),(9,2,0,0,0),(9,2,24,0,0),
        (9,2,1,0,0),(9,0,1,2,0),(9,0,1,1,0),(9,0,1,0,1)] {
        let mut native=marker29_state();
        for (a,v) in [(0xdd0,state),(0xe20,0xa6),(0xd80,ai),(0xdf0,delay),
            (0xee0,aux),(0xda0,b),(0xd10,0x40),(0xd00,0x40)] {native.write_wram_byte(a,v);}
        native.write_wram_word(0x20,0x180);native.write_wram_word(0x22,0x180);
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x1e919f,0)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x1e8b19,machine,
            |s|(s.cpu.k==0x1e && ((0x919f..0x9225).contains(&s.cpu.pc)
                || (0x922b..0x9254).contains(&s.cpu.pc)
                || ((0x925c..0x92e4).contains(&s.cpu.pc)
                    && (!(0x92bd..0x92c4).contains(&s.cpu.pc) || s.cpu.sp==0x1fa))))
                || (s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc)),|| {
                let _parent=crate::cycle_ledger::routine(0x1e8b19);
                native.sprite_zazak_main(0);
            });
    }
}

#[test]
fn marker40_boulder_complete_control_matches_rom() {
    for k in [0usize, 15] {
        for (indoors, frame, chr, z, zv, disabled, near) in [
            (0,0,0,32,0,0,false), (0,1,0,0,0xff,0,false),
            (0,0,0,0,0xff,1,false), (1,0,3,0,0,0,false),
            (1,1,0,0,0,0,false), (1,0,3,0,0,0,true),
            (1,0,3,0,0,1,true)] {
            let mut native = marker29_state();
            for (a,v) in [(0x1b,indoors),(0x1a,frame),(0xfc6,chr),(0x11,disabled)] {
                native.write_wram_byte(a,v);
            }
            for (a,v) in [(0xdd0,9),(0xe20,0xc2),(0xd10,0x40),(0xd00,0x40),
                (0xf70,z),(0xf80,zv),(0xcaa,0x80)] { native.write_wram_byte(a+k,v); }
            native.write_wram_word(0x22,if near {0x40}else{0x100});
            native.write_wram_word(0x20,if near {0x3c}else{0x100});
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x1dcfcb,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x1dc222,machine,
                |s|s.cpu.k==0x1d && (0xcfcb..0xd088).contains(&s.cpu.pc),||{
                    let _scope=crate::cycle_ledger::routine(0x1dc222);
                    native.sprite_c2_boulder(k);
                });
        }
    }
}

#[test]
fn marker40_boulder_draw_matches_rom() {
    for k in [0usize,15] {
        for subtype in [0u8,8,16,24] {
            let mut native=marker29_state();
            native.write_wram_byte(0xdd0+k,9);native.write_wram_byte(0xe20+k,0xc2);
            native.write_wram_byte(0xe80+k,subtype);
            native.write_wram_byte(0xd10+k,0x40);native.write_wram_byte(0xd00+k,0x40);
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x1dd185,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x1dd185,machine,
                |s|s.cpu.k==0x1d && (0xd185..0xd1a8).contains(&s.cpu.pc),||native.boulder_draw(k));
        }
    }
}

#[test]
fn marker40_bank1d_z_matches_rom() {
    for k in [0usize,15] {
        for (v,z,sub) in [(0u8,0u8,0u8),(1,0xff,0xff),(0x7f,0,0),
            (0x80,0,0),(0xff,0xff,0xff)] {
            let mut native=marker29_state();
            for (a,value)in[(0xf80,v),(0xf70,z),(0xf90,sub)]{native.write_wram_byte(a+k,value);}
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x1de98b,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x1de98b,machine,
                |s|s.cpu.k==0x1d && (0xe98b..0xe9ad).contains(&s.cpu.pc),||native.sprite_move_z(k, SpriteMoveZEntry::Rom1DE98B));
        }
    }
}

#[test]
fn marker40_cucco_movement_wrapper_matches_rom() {
    for k in [0usize,15] {
        for (xv,yv) in [(0u8,0u8),(1,0xff),(0x80,0x7f)] {
            let mut native=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x0b),(0xd10,0x40),(0xd00,0x40),
                (0xd50,xv),(0xd40,yv),(0xcaa,0x80)]{native.write_wram_byte(a+k,v);}
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x06a7bb,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x06a7bb,machine,
                |s|s.cpu.k==6 && (0xa7bb..0xa7c3).contains(&s.cpu.pc),||{native.cucco_do_movement_xy(k);});
        }
    }
}

#[test]
fn marker40_guard_map16_attribute_matches_rom() {
    for (x,y) in [(0u16,0u16),(7,8),(0xff,0xff),(0xffff,0x8000)] {
        let mut native=marker29_state();
        for (a,v) in [(0x708,0x123u16),(0x70a,0x1ff),(0x70c,0x40),(0x70e,0x1ff)] {
            native.write_wram_word(a,v);
        }
        native.sync_native_game_state_from_ram();
        let Some(mut machine)=marker29_machine(&native,0x05faa2,0)else{return};
        machine.ram[0..2].copy_from_slice(&y.to_le_bytes());
        machine.ram[2..4].copy_from_slice(&x.to_le_bytes());
        crate::cycle_ledger::assert_rom_marker_work(0x05faa2,machine,
            |s|s.cpu.k==5 && (0xfaa2..0xfaca).contains(&s.cpu.pc),||{
                native.overworld_map16_tile_definition_at_location(x,y);
            });
    }
}

#[test]
fn marker40_lightning_garnish_matches_rom() { marker40_garnish_cases(0x1de475); }

#[test]
fn marker40_laser_garnish_matches_rom() { marker40_garnish_cases(0x1ea488); }

fn marker40_garnish_cases(entry: u32) {
    for (free, counter) in [(Some(29usize),0u8),(Some(15),0),(Some(0),0),
        (None,0),(None,1),(None,29)] {
        let mut native=marker29_state();
        for j in 0..30 { native.write_wram_byte(0x1f800+j,if Some(j)==free {0}else{1}); }
        native.write_wram_byte(0xff8,counter);
        for(a,v)in[(0xd10,0xff),(0xd30,1),(0xd00,0xff),(0xd20,1),
            (0xd90,5),(0xdc0,2),(0xf20,1)]{native.write_wram_byte(a+15,v);}
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,entry,15)else{return};
        let end=if entry==0x1de475 {0xe4c8}else{0xa4e7};
        crate::cycle_ledger::assert_rom_marker_work(entry,machine,
            |s|u32::from(s.cpu.k)==entry>>16 && (entry as u16..end).contains(&s.cpu.pc),||{
                if entry==0x1de475 {native.lightning_spawn_garnish(15);}
                else {native.laser_beam_build_up_garnish(15);}
            });
    }
}

#[test]
fn marker40_deadrock_complete_body_matches_rom() {
    for k in [0usize,15] {
        for (ai,delay,aux1,aux2,frame,b,f) in [(0,1,0,0,0,0,0),(0,0,0,0,0,0,0),
            (0,0,0,0,0,3,0),(1,0,0,0,0,0,0),(1,8,0,0,0,0,0),
            (2,0,0,0,0,0,0),(2,1,32,4,0,0,0),(2,1,1,0,0,0,0),
            (2,1,1,0,1,0,0),(1,8,0,0,0,0,0x80)] {
            let mut native=marker29_state();
            for(a,v)in[(0xdd0,9),(0xe20,0x27),(0xd10,0x40),(0xd00,0x40),
                (0xd80,ai),(0xdf0,delay),(0xe00,aux1),(0xe10,aux2),(0xda0,b),(0xea0,f),
                (0xcaa,0x80),(0xd50,1),(0xd40,0xff)]{native.write_wram_byte(a+k,v);}
            native.write_wram_byte(0x1a,frame);native.write_wram_word(0x22,0x100);
            native.write_wram_word(0x20,0x100);
            native.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&native,0x06948a,k)else{return};
            crate::cycle_ledger::assert_rom_marker_work(0x0684e2,machine,
                |s|(s.cpu.k==6 && ((0x948a..0x94fa).contains(&s.cpu.pc)||(0x9506..0x95c9).contains(&s.cpu.pc)))
                    || (s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc)),||{
                    let _scope=crate::cycle_ledger::routine(0x0684e2);
                    native.sprite_27_deadrock(k);
                });
        }
    }
}

#[test]
fn marker40_stalfos_head_matches_rom() {
    for gfx in [0u8,1,2,3] {
        for (dir,y,c) in [(0u8,0u16,0u8),(3,0x100,0),(1,0xff,0xff)] {
            let mut native=marker29_state();
            for(a,v)in[(0xdc0,gfx),(0xeb0,dir),(0xdb0,c)]{native.write_wram_byte(a+15,v);}
            native.sync_native_game_state_from_ram();
            let Some(mut machine)=marker29_machine(&native,0x1eae4e,15)else{return};
            machine.ram[0..2].copy_from_slice(&0x40u16.to_le_bytes());
            machine.ram[2..4].copy_from_slice(&y.to_le_bytes());
            machine.ram[5]=0x30;
            crate::cycle_ledger::assert_rom_marker_work(0x1eae4e,machine,
                |s|s.cpu.k==0x1e && (0xae4e..0xaea4).contains(&s.cpu.pc),||{
                    native.sprite_draw_stalfos_knight_head(15,&PrepOamCoordsRet{x:0x40,y,r4:0,flags:0x30});
                });
        }
    }
}

#[test]
fn marker40_prepared_draw_loop_matches_rom() {
    for (y,state) in [(0u16,9u8),(0xff,9),(0,11),(0xff,11)] {
        let mut native=marker29_state();native.write_wram_byte(0xdd0,state);
        native.write_wram_byte(0x1fa3c,1);
        native.sync_native_game_state_from_ram();
        let Some(mut machine)=marker29_machine(&native,0x05df78,0)else{return};
        machine.cpu.db=0x1e;
        machine.ram[0xcfe]=u8::from(state==11);
        machine.ram[0..2].copy_from_slice(&0x40u16.to_le_bytes());
        machine.ram[2..4].copy_from_slice(&y.to_le_bytes());
        machine.ram[4..6].copy_from_slice(&0x3000u16.to_le_bytes());
        machine.ram[6..8].copy_from_slice(&5u16.to_le_bytes());
        machine.ram[8..10].copy_from_slice(&0xacecu16.to_le_bytes());
        crate::cycle_ledger::assert_rom_marker_work(0x05df6c,machine,
            |s|s.cpu.k==5 && (0xdf78..0xdfe5).contains(&s.cpu.pc),||{
                native.sprite_draw_multiple_prepared_marked(0,&STALFOS_KNIGHT_DRAW_FRAMES[..5],(0x40,y,0x30));
            });
    }
}

#[test]
fn marker40_stalfos_draw_caller_matches_rom() {
    for gfx in [0u8,2,3] {
        let mut native=marker29_state();
        for(a,v)in[(0xdd0,9),(0xe20,0x91),(0xdc0,gfx),(0xd10,0x40),(0xd00,0x40)]{native.write_wram_byte(a,v);}
        native.sync_native_game_state_from_ram();
        let Some(machine)=marker29_machine(&native,0x1eae04,0)else{return};
        crate::cycle_ledger::assert_rom_marker_work(0x1eae04,machine,
            |s|s.cpu.k==0x1e && (0xae04..0xae46).contains(&s.cpu.pc),||native.stalfos_knight_draw(0));
    }
}

#[test]
fn marker40_prepared_draw_wrapper_matches_rom() {
    let mut native=marker29_state();native.write_wram_byte(0xdd0,9);
    native.sync_native_game_state_from_ram();
    let Some(mut machine)=marker29_machine(&native,0x1e87f0,0)else{return};
    machine.cpu.a=5;
    machine.ram[8..10].copy_from_slice(&0xacecu16.to_le_bytes());
    crate::cycle_ledger::assert_rom_marker_work(0x1e87f0,machine,
        |s|s.cpu.k==0x1e && (0x87f0..0x87f5).contains(&s.cpu.pc),||{
            native.sprite_draw_multiple_prepared_marked(0,&STALFOS_KNIGHT_DRAW_FRAMES[..5],(0x40,0x40,0));
        });
}

#[test]
fn marker41_vitreous_eye_complete_body_matches_rom() {
    for slot in [0usize, 15] {
        for (state, gfx, frame, hit, near_x, near_y) in [
            (0, 0, 0, 0, false, false), (0, 1, 1, 0, true, true),
            (1, 0, 0, 0, false, false), (1, 0, 1, 0, true, false),
            (1, 0, 1, 0, true, true), (1, 0, 0, 1, true, true),
            (2, 0, 0, 0, false, false), (2, 0, 1, 0, true, false),
            (2, 0, 1, 0, true, true),
        ] {
            let mut native = marker29_state();
            native.write_wram_byte(0xfa0, slot as u8);
            for (a, v) in [(0xe20, 0xbe), (0xdd0, 9), (0xd80, state),
                (0xdc0, gfx), (0xe80, 0x20), (0xea0, hit), (0xcaa, 0x80),
                (0xed0, if near_x { 0x40 } else { 0x80 }), (0xeb0, 0),
                (0xec0, if near_y { 0x40 } else { 0x80 }), (0xe30, 0),
                (0xd90, if near_x { 0x40 } else { 0x80 }), (0xda0, 0),
                (0xdb0, if near_y { 0x40 } else { 0x80 }), (0xde0, 0)] {
                native.write_wram_byte(a + slot, v);
            }
            native.write_wram_byte(0x1a, frame);
            native.write_wram_byte(0x4d, 1); // Damage children reject without altering AI.
            native.sync_native_game_state_from_ram();
            let Some(machine) = marker29_machine(&native, 0x1de773, slot) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x1dc222, machine,
                |s| s.cpu.k == 0x1d && (0xe773..0xe893).contains(&s.cpu.pc), || {
                    let _parent = crate::cycle_ledger::routine(0x1dc222);
                    native.sprite_be_vitreous_eye(slot);
                });
        }
    }
}

#[test]
fn marker41_swamola_angle_matches_rom() {
    for x in [0u8, 1, 31, 127, 128, 255] {
        for y in [0u8, 1, 31, 127, 128, 255] {
            let mut native = marker29_state();
            native.write_wram_byte(0, y);
            native.write_wram_byte(1, x);
            let Some(machine) = marker29_machine(&native, 0x1df614, 0) else { return };
            let mut result = 0;
            let source = crate::cycle_ledger::assert_rom_marker_work(0x1df614, machine,
                |s| s.cpu.k == 0x1d && (0xf614..0xf65d).contains(&s.cpu.pc), || {
                    result = ZeldaState::sprite_convert_velocity_to_angle(x, y, true);
                });
            assert_eq!(result, source.snes().cpu.a as u8);
        }
    }
}

#[test]
fn marker41_distress_complete_writer_matches_rom() {
    for time in [0u8, 8, 16, 24] {
        for (x, y) in [(0u16, 0u16), (255, 239), (256, 255), (65535, 65535)] {
            let mut native = marker29_state();
            native.write_wram_word(0, x);
            native.write_wram_word(2, y);
            native.write_wram_byte(6, time);
            let Some(machine) = marker29_machine(&native, 0x06a733, 0) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x06a733, machine,
                |s| s.cpu.k == 6 && (0xa733..0xa78e).contains(&s.cpu.pc), || {
                    native.sprite_draw_distress_custom(x, y, time, SpriteDistressEntry::Rom06A733);
                });
        }
    }
}

#[test]
fn marker41_fireball_complete_body_matches_rom() {
    for slot in [0usize, 15] {
        for (indoors, delay, aux, shield, frame, b_frames, floor) in [
            (0, 0, 0, 0, 1, 0, 0), (1, 1, 0, 0, 1, 0, 0),
            (1, 0, 1, 0, 0, 0, 0), (1, 0, 0, 0, 0, 0, 0),
            (1, 0, 1, 2, 1, 0, 1), (1, 0, 1, 2, 1, 0, 0),
            (1, 0, 1, 2, 1, 1, 0),
        ] {
            let mut native = marker29_state();
            native.write_wram_byte(0xfa0, slot as u8);
            for (a, v) in [(0xe20, 0x55), (0xdd0, 9), (0xdf0, delay),
                (0xe00, aux), (0xdc0, 0), (0xd10, 0x40), (0xd00, 0x40),
                (0xf20, floor), (0xe40, 1), (0xcaa, 0x80)] {
                native.write_wram_byte(a + slot, v);
            }
            for (a, v) in [(0x1b, indoors), (0x1a, frame), (0x3c, b_frames),
                (0xf35a, shield), (0x2f, 0)] {
                native.write_wram_byte(a, v);
            }
            native.sync_native_game_state_from_ram();
            let Some(mut machine) = marker29_machine(&native, 0x059683, slot) else { return };
            machine.cpu.a = 1; // $05:967B supplies sprite E to the fallthrough.
            crate::cycle_ledger::assert_rom_marker_work(0x05b5d3, machine,
                |s| s.cpu.k == 5 && (0x9683..0x9725).contains(&s.cpu.pc), || {
                    let _parent = crate::cycle_ledger::routine(0x05b5d3);
                    native.sprite_fireball(slot);
                });
        }
    }
}

#[test]
fn marker41_zora_writer_matches_rom() {
    for slot in [0usize, 15] {
        for gfx in [0u8, 3, 12] {
            for y in [0u16, 239, 255, 256] {
                let mut native = marker29_state();
                native.write_wram_byte(0xfa0, slot as u8);
                native.write_wram_byte(0xdc0 + slot, gfx);
                native.write_wram_byte(0xdd0 + slot, 9);
                native.write_wram_byte(0xe20 + slot, 0x55);
                native.write_wram_word(0xfda, y);
                native.sync_native_game_state_from_ram();
                let Some(machine) = marker29_machine(&native, 0x0598f5, slot) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x0598f5, machine,
                    |s| s.cpu.k == 5 && (0x98f5..0x995b).contains(&s.cpu.pc), || {
                        native.zora_draw(slot);
                    });
            }
        }
    }
}

#[test]
fn marker41_busy_predicate_matches_rom() {
    for (aux, pose, state, hook) in [(1, 0, 0, -1), (0, 1, 0, -1),
        (0, 0, 0x80, -1), (0, 0, 0, -1), (0, 0, 0, 0), (0, 0, 0, 4)] {
        let mut native = marker29_state();
        for (a, v) in [(0x4d, aux), (0x2da, pose), (0x308, state)] {
            native.write_wram_byte(a, v);
        }
        if hook >= 0 { native.write_wram_byte(0xc4a + hook as usize, 0x27); }
        native.sync_native_game_state_from_ram();
        let Some(machine) = marker29_machine(&native, 0x07f4d0, 0) else { return };
        let mut busy = false;
        let source = crate::cycle_ledger::assert_rom_marker_work(0x07f4d0, machine,
            |s| s.cpu.k == 7 && (0xf4d0..0xf4f1).contains(&s.cpu.pc), || {
                busy = native.sprite_check_if_link_is_busy();
            });
        assert_eq!(busy, source.snes().cpu.c);
    }
}

#[test]
fn marker42_spark_complete_body_matches_rom() {
    for (state, ai, delay, frame) in [(8, 0, 0, 0), (9, 0, 0, 0),
        (9, 1, 0, 1), (9, 1, 6, 0), (9, 1, 5, 1)] {
        for typ in [0x5bu8, 0x5c] {
            let mut n = marker29_state();
            let assets = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../zelda3_assets.dat")).unwrap();
            n.set_assets(&assets).unwrap();
            for (a,v) in [(0xdd0,state),(0xe20,typ),(0xd80,ai),(0xe10,delay),
                (0x1a,frame),(0xfa0,0),(0xcaa,0x80)] { n.write_wram_byte(a,v); }
            n.write_wram_word(0xfd8,0x40); n.write_wram_word(0xfda,0x40);
            n.write_wram_byte(0xd10,0x40); n.write_wram_byte(0xd00,0x40);
            n.sync_native_game_state_from_ram();
            let Some(machine) = marker29_machine(&n,0x05933f,0) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x05b5d3,machine,
                |s| s.cpu.k==5 && (0x933f..0x940e).contains(&s.cpu.pc),|| {
                    let _parent=crate::cycle_ledger::routine(0x05b5d3);
                    n.sprite_5_b_spark_clockwise(0);
                });
        }
    }
}

#[test]
fn marker42_babasu_complete_states_match_rom() {
    for ai in 0..4u8 {
        for delay in [0u8, 1, 31, 32, 55] {
            let mut n=marker29_state();
            let assets=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../zelda3_assets.dat")).unwrap();
            n.set_assets(&assets).unwrap();
            for (a,v) in [(0xdd0,9),(0xe20,0x9c),(0xd80,ai),(0xdf0,delay),
                (0xcaa,0x80),(0xfa0,0),(0xde0,1),(0xd10,0x40),(0xd00,0x40)] {
                n.write_wram_byte(a,v);
            }
            n.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&n,0x1e9c6b,0) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x1e8b19,machine,
                |s| s.cpu.k==0x1e && ((0x9c6b..0x9c79).contains(&s.cpu.pc)
                    || (0x9c81..0x9c9d).contains(&s.cpu.pc)
                    || (0x9cad..0x9ce8).contains(&s.cpu.pc)
                    || (0x9cec..0x9d17).contains(&s.cpu.pc))
                    || s.cpu.k==0 && (0x8784..0x879c).contains(&s.cpu.pc),|| {
                    let _parent=crate::cycle_ledger::routine(0x1e8b19);
                    n.babasu(0);
                });
        }
    }
}

#[test]
fn marker42_pengator_complete_draw_matches_rom() {
    for gfx in [0u8, 1, 14, 19, 21] {
        for y in [0x40u16, 0x400] {
            let mut n=marker29_state();
            for (a,v) in [(0xdd0,9),(0xe20,0x99),(0xdc0,gfx),(0xfa0,0),
                (0xcaa,0x80)] { n.write_wram_byte(a,v); }
            n.write_wram_word(0xfda,y); n.write_wram_byte(0xd00,y as u8);
            n.write_wram_byte(0xd20,(y>>8) as u8);
            n.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&n,0x1ea415,0) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x1ea415,machine,
                |s| s.cpu.k==0x1e && (0xa415..0xa462).contains(&s.cpu.pc),
                || { n.pengator_draw(0); });
        }
    }
}

#[test]
fn marker42_goriya_complete_draw_matches_rom() {
    for direction in 0..4u8 {
        for delay in [0u8, 1] {
            for gfx in [0u8, 3, 7] {
                let mut n=marker29_state();
                for (a,v) in [(0xdd0,9),(0xe20,0x83),(0xdc0,gfx),(0xfa0,0),
                    (0xde0,direction),(0xe00,delay),(0xcaa,0x80)] { n.write_wram_byte(a,v); }
                n.sync_native_game_state_from_ram();
                let Some(machine)=marker29_machine(&n,0x1df589,0) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x1df589,machine,
                    |s| s.cpu.k==0x1d && (0xf589..0xf5d4).contains(&s.cpu.pc),
                    || { n.goriya_draw(0); });
            }
        }
    }
}

#[test]
fn marker42_rabbit_active_complete_body_matches_rom() {
    for (pause,state,delay) in [(0u8,9u8,1u8),(0,9,0),(1,9,0),(0,8,1)] {
        for pointer in [0x800u16,0x8fc] {
            let mut n=marker29_state();
            let assets=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../zelda3_assets.dat")).unwrap();
            n.set_assets(&assets).unwrap();
            for (a,v) in [(0xdd0,state),(0xe20,0x4b),(0xd80,1),(0xdf0,delay),
                (0xf00,pause),(0xfa0,0),(0xcaa,0x80),(0xd10,0x40),(0xd00,0x40)] {
                n.write_wram_byte(a,v);
            }
            n.write_wram_word(0x90,pointer);
            n.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&n,0x1d8600,0) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x1dc222,machine,
                |s| s.cpu.k==0x1d && (0x8600..0x866a).contains(&s.cpu.pc),|| {
                    let _parent=crate::cycle_ledger::routine(0x1dc222);
                    n.sprite_bunny_beam(0);
                });
        }
    }
}

#[test]
fn marker42_knight_crumble_complete_body_matches_rom() {
    for (z,vz) in [(0u8,0u8),(2,0xc0),(0,0xbf)] {
        for delay in [0u8,1,223,224,225,255] {
            let mut n=marker29_state();
            let assets=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../zelda3_assets.dat")).unwrap();
            n.set_assets(&assets).unwrap();
            for (a,v) in [(0xdd0,9),(0xe20,0x91),(0xd80,6),(0xdf0,delay),
                (0xf70,z),(0xf80,vz),(0xfa0,0),(0xcaa,0x80),
                (0xd10,0x40),(0xd00,0x40)] { n.write_wram_byte(a,v); }
            n.sync_native_game_state_from_ram();
            let Some(machine)=marker29_machine(&n,0x1eac77,0) else { return };
            let random = std::cell::Cell::new(None);
            crate::cycle_ledger::assert_rom_marker_work(0x1e8b19,machine,
                |s| {
                    if s.cpu.k==0x1e && s.cpu.pc==0xac9f {
                        random.set(Some((s.cpu.a as u8,s.cpu.c)));
                    }
                    s.cpu.k==0x1e && ((0xac77..0xacd6).contains(&s.cpu.pc)
                        || (0xab46..0xab5a).contains(&s.cpu.pc))
                },|| {
                    // Replay the unmodified ROM child's answer, as logical
                    // replay does; the native fallback RNG is a different stream.
                    if let Some((value,carry))=random.get() {
                        n.install_rom_random_replay(vec![crate::RomRandomSample::with_carry(
                            0,value,carry)],0);
                        n.rom_random_replay.begin_frame();
                    }
                    let _parent=crate::cycle_ledger::routine(0x1e8b19);
                    n.stalfos_knight_crumble(0);
                });
        }
    }
}

#[test]
fn marker43_beamos_body_matches_rom() {
    for k in [0usize, 15] {
        for direction in [0, 31, 32, 63] {
            for (x, y) in [(0x40u16, 0x40u16), (0x100, 0xfff8), (0x400, 0x40)] {
                let mut native = marker29_state();
                for (a, v) in [(0xe20, 0x61), (0xdd0, 9), (0xcaa, 0x80), (0xde0, direction)] {
                    native.write_wram_byte(a + k, v);
                }
                native.write_wram_word(0xfd8, x);
                native.write_wram_word(0xfda, y);
                native.write_wram_byte(0xfa0, k as u8);
                native.sync_native_game_state_from_ram();
                let Some(machine) = marker29_machine(&native, 0x059068, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x059068, machine,
                    |s| s.cpu.k == 5 && (0x9068..0x90d1).contains(&s.cpu.pc), || native.beamos_draw(k));
            }
        }
    }
}

#[test]
fn marker43_beamos_eye_matches_rom() {
    for k in [0usize, 15] {
        for direction in [0, 31, 32, 63] {
            for (x, y) in [(0x40u16, 0x40u16), (0xff, 0xfff8)] {
                let mut native = marker29_state();
                for (a, v) in [(0xe20, 0x61), (0xdd0, 9), (0xcaa, 0x80), (0xde0, direction)] {
                    native.write_wram_byte(a + k, v);
                }
                native.write_wram_word(0, x);
                native.write_wram_word(2, y);
                native.write_wram_byte(5, 0x20);
                native.write_wram_byte(0xfa0, k as u8);
                native.sync_native_game_state_from_ram();
                let Some(machine) = marker29_machine(&native, 0x059151, k) else { return };
                let info = PrepOamCoordsRet { x, y, r4: 0, flags: 0x20 };
                crate::cycle_ledger::assert_rom_marker_work(0x059151, machine,
                    |s| s.cpu.k == 5 && (0x9151..0x91b1).contains(&s.cpu.pc), || {
                        native.sprite_draw_beamos_eyeball(k, &info);
                    });
            }
        }
    }
}

#[test]
fn marker43_thief_target_matches_rom() {
    for k in [0usize, 15] {
        for frame in [0, 1, 2, 3] {
            for item in [0u8, 0xdc, 0xe1, 0xd9, 0x40] {
                let mut native = marker29_state();
                native.write_wram_byte(0x1a, frame);
                native.write_wram_byte(0xfa0, k as u8);
                native.write_wram_byte(0xe20 + k, 0xc4);
                for j in 0..16 {
                    native.write_wram_byte(0xdd0 + j, u8::from(item != 0));
                    native.write_wram_byte(0xe20 + j, item);
                    native.write_wram_byte(0xf10 + j, u8::from(j % 3 == 0));
                    native.write_wram_byte(0xd10 + j, if j % 2 == 0 { 0x40 } else { 0x80 });
                    native.write_wram_byte(0xd00 + j, if j % 2 == 0 { 0x40 } else { 0x80 });
                }
                native.sync_native_game_state_from_ram();
                let Some(mut machine) = marker29_machine(&native, 0x1dca4c, k) else { return };
                machine.cpu.y = 8;
                crate::cycle_ledger::assert_rom_marker_work(0x1dca4c, machine,
                    |s| s.cpu.k == 0x1d && (0xca4c..0xca9e).contains(&s.cpu.pc), || {
                        native.thief_target_booty(k, 8);
                    });
            }
        }
    }
}

#[test]
fn marker43_thief_scan_matches_rom() {
    for k in [0usize, 15] {
        for item in [0u8, 0xdc, 0xe1, 0xd9, 0x40] {
            for j in [0usize, 8, 15] {
                let mut native = marker29_state();
                native.write_wram_byte(0xdd0 + j, u8::from(item != 0));
                native.write_wram_byte(0xe20 + j, item);
                native.write_wram_byte(0xd10 + j, 0x80);
                native.write_wram_byte(0xd00 + j, 0x80);
                native.write_wram_byte(0xfa0, k as u8);
                native.sync_native_game_state_from_ram();
                let Some(machine) = marker29_machine(&native, 0x1dca24, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x1dca24, machine,
                    |s| s.cpu.k == 0x1d && (0xca24..0xca4c).contains(&s.cpu.pc), || {
                        native.thief_scan_for_booty(k);
                    });
            }
        }
    }
}

#[test]
fn marker43_thief_grab_matches_rom() {
    for k in [0usize, 15] {
        for j in [0usize, 8, 15] {
            for item in [0xdc, 0xe1, 0xd9] {
                for (x, y) in [(0x40, 0x40), (0x80, 0x40), (0x40, 0x80)] {
                    let mut native = marker29_state();
                    native.write_wram_byte(0xe20 + j, item);
                    native.write_wram_byte(0xd10 + j, x);
                    native.write_wram_byte(0xd00 + j, y);
                    native.write_wram_byte(0xfa0, k as u8);
                    native.sync_native_game_state_from_ram();
                    let Some(mut machine) = marker29_machine(&native, 0x1dca9e, k) else { return };
                    machine.cpu.y = j as u16;
                    crate::cycle_ledger::assert_rom_marker_work(0x1dca9e, machine,
                        |s| s.cpu.k == 0x1d && (0xca9e..0xcaf2).contains(&s.cpu.pc), || {
                            native.thief_grab_booty(k, j);
                        });
                }
            }
        }
    }
}

#[test]
fn marker43_proximity_dispatch_matches_rom() {
    for submodule in [0u8, 1, 2] {
        for toggler in [0u8, 1, 254, 255] {
            let mut native = marker29_state();
            native.write_wram_byte(0x1b, 0);
            native.write_wram_byte(0x11, submodule);
            native.write_wram_byte(0xfb7, toggler);
            native.sync_native_game_state_from_ram();
            let Some(machine) = marker29_machine(&native, 0x09c58f, 0) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x09c58f, machine,
                |s| s.cpu.k == 9 && (0xc58f..0xc5b7).contains(&s.cpu.pc), || {
                    native.sprite_proximity_activation();
                });
        }
    }
}

#[test]
fn prefix2_stalfos_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for (gfx, pause, delay) in [(0, 0, 0), (7, 0, 0), (8, 0, 0),
            (11, 0, 0), (0, 1, 0), (0, 0, 1)] {
            for head in 0..4 {
                for pointer in [0x800u16, 0x8fe] {
                    let mut n = marker29_state();
                    for (a, v) in [(0xdd0, 9), (0xe20, 0x85), (0xdc0, gfx),
                        (0xf00, pause), (0xe10, delay), (0xeb0, head),
                        (0xd10, 0x40), (0xd00, 0x40)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0xfa0, k as u8);
                    n.write_wram_word(0x90, pointer);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x0dc21c, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(0x0dc21c, s,
                        |s| s.cpu.k == 0x0d && (0xc217..0xc26a).contains(&s.cpu.pc),
                        || n.stalfos_draw(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix2 Stalfos: {cases} ROM cases");
}

fn prefix2_npc_draw_cases(entry: u32) {
    let mut cases = 0;
    for k in [0usize, 15] {
        for direction in 0..4 {
            for gfx in 0..2 {
                for y in [0x40u16, 0x100] {
                    let mut n = marker29_state();
                    for (a, v) in [(0xdd0, 9), (0xde0, direction), (0xdc0, gfx),
                        (0xd10, 0x40), (0xd00, y as u8), (0xd20, (y >> 8) as u8)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0xfa0, k as u8);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, entry, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(entry, s,
                        |s| s.cpu.k == 5 && match entry {
                            0x05ea4d => (0xea4d..0xea71).contains(&s.cpu.pc),
                            _ => (0xffdf..0xffff).contains(&s.cpu.pc),
                        }, || match entry {
                            0x05ea4d => n.running_man_draw(k),
                            _ => n.trough_boy_draw(k),
                        });
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix2 {entry:06x}: {cases} ROM cases");
}

#[test]
fn prefix2_running_man_draw_matches_rom() { prefix2_npc_draw_cases(0x05ea4d); }

#[test]
fn prefix2_trough_boy_draw_matches_rom() { prefix2_npc_draw_cases(0x05ffdf); }

#[test]
fn prefix2_trough_boy_body_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for icons in [0u8, 2, 3, 255] {
            for (state, submodule, modal, pause) in [(9, 0, 0, 0),
                (8, 0, 0, 0), (9, 1, 0, 0), (9, 0, 1, 0), (9, 0, 0, 1)] {
                for direction in 0..4 {
                    let mut n = marker29_state();
                    for (a, v) in [(0xdd0, state), (0xe20, 0xa7), (0xde0, direction),
                        (0xd10, 0x40), (0xd00, 0x40), (0xf00, pause), (0xcaa, 0)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0xfa0, k as u8);
                    n.write_wram_byte(0x11, submodule);
                    n.write_wram_byte(0xfc1, modal);
                    n.write_wram_byte(0xf3c7, icons);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x05ff66, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(0x05ff66, s,
                        |s| s.cpu.k == 5 && (0xff66..0xff9f).contains(&s.cpu.pc),
                        || n.sprite_trough_boy(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix2 trough body: {cases} ROM cases");
}

#[test]
fn prefix2_sprite_disable_complete_entry_matches_rom() {
    let mut cases = 0;
    for indoors in [0u8, 1] {
        for pattern in 0..4 {
            let mut n = marker29_state();
            n.write_wram_byte(0x1b, indoors);
            for k in 0..16 {
                n.write_wram_byte(0xdd0 + k, if pattern == 0 { 0 } else { 9 });
                n.write_wram_byte(0xe20 + k, if pattern == 2 || (pattern == 3 && k & 1 == 0) { 0x6c } else { 1 });
            }
            for a in (0xc4a..0xc54).chain(0xb00..0xb08).chain(0x1f800..0x1f81e) {
                n.write_wram_byte(a, 1);
            }
            n.sync_native_game_state_from_ram();
            let Some(s) = marker29_machine(&n, 0x09c22f, 0) else { return };
            let source = crate::cycle_ledger::assert_rom_marker_work(0x09c22f, s,
                |s| s.cpu.k == 9 && (0xc22f..0xc290).contains(&s.cpu.pc),
                || n.sprite_disable_all());
            for a in (0xdd0..0xde0).chain(0xc4a..0xc54).chain(0xb00..0xb08)
                .chain(0x1f800..0x1f81e) {
                assert_eq!(n.read_wram_byte(a), source.snes().ram[a], "address={a:05x}");
            }
            cases += 1;
        }
    }
    eprintln!("prefix2 disable: {cases} ROM cases");
}

#[test]
fn prefix2_running_man_body_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for (ai, delay, a, route) in [(0, 0, 0, 0), (1, 1, 0, 0),
            (2, 1, 0, 0), (1, 0, 0, 0), (1, 0, 2, 0),
            (2, 0, 2, 0), (2, 0, 0, 0), (2, 0, 0, 1), (2, 0, 0, 2),
            (3, 0, 0, 0)] {
            for (state, submodule) in [(9, 0), (9, 2), (8, 0)] {
                let mut n = marker29_state();
                for (address, value) in [(0xe20, 0x74), (0xdd0, state),
                    (0xd80, ai), (0xdf0, delay), (0xd90, a), (0xda0, route),
                    (0xeb0, 2), (0xde0, 2), (0xd10, 0x40), (0xd00, 0x40),
                    (0xcaa, 0x80)] {
                    n.write_wram_byte(address + k, value);
                }
                n.write_wram_byte(0xfa0, k as u8);
                n.write_wram_byte(0x11, submodule);
                n.write_wram_byte(0x1a, 4);
                n.write_wram_word(0x20, 0x100);
                n.write_wram_word(0x22, 0x100);
                n.sync_native_game_state_from_ram();
                for owner in [0x05e8b2, 0x05e9ac, 0x05e9ba] {
                    let mut candidate = n.clone();
                    let Some(s) = marker29_machine(&candidate, 0x05e8b2, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(owner, s,
                        |s| s.cpu.k == 5 && match owner {
                            0x05e8b2 => (0xe8b2..0xe8ed).contains(&s.cpu.pc)
                                || (0xe8f7..0xe96c).contains(&s.cpu.pc)
                                || (0xe973..0xe9ac).contains(&s.cpu.pc),
                            0x05e9ac => (0xe9ac..0xe9ba).contains(&s.cpu.pc),
                            _ => (0xe9ba..0xe9cd).contains(&s.cpu.pc),
                        }, || candidate.sprite_running_man(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix2 running body: {cases} ROM cases");
}

#[test]
fn prefix2_npc_long_entries_match_rom() {
    for entry in [0x05e8a2u32, 0x05ff5e] {
        for k in [0usize, 15] {
            for state in [0u8, 9] {
                let mut n = marker29_state();
                for (a, v) in [(0xdd0, state), (0xe20, if entry == 0x05e8a2 { 0x74 } else { 0x3c }),
                    (0xd10, 0x40), (0xd00, 0x40)] {
                    n.write_wram_byte(a + k, v);
                }
                n.write_wram_byte(0xfa0, k as u8);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, entry, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(entry, s,
                    |s| s.cpu.k == 5 && (entry as u16..entry as u16 + 8).contains(&s.cpu.pc),
                    || if entry == 0x05e8a2 { n.sprite_running_boy(k); }
                        else { n.sprite_kakariko_boy(k); });
            }
        }
    }
}

#[test]
fn prefix2_desert_barrier_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for delay in [0u8, 1, 2, 255] {
            for (x, y) in [(0x40u16, 0x40u16), (0x20, 0x40), (0x60, 0x40),
                (0x40, 0x20), (0x40, 0x60), (0x100, 0x100)] {
                let mut n = marker29_state();
                for (a, v) in [(0xdd0, 9), (0xe20, 0xf2), (0xdf0, delay),
                    (0xd10, 0x40), (0xd00, 0x40)] {
                    n.write_wram_byte(a + k, v);
                }
                n.write_wram_byte(0xfa0, k as u8);
                n.write_wram_word(0x22, x);
                n.write_wram_word(0x20, y);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x059626, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x059626, s,
                    |s| s.cpu.k == 5 && ((0x9626..0x966a).contains(&s.cpu.pc)
                        || (0x8ab1..0x8ab6).contains(&s.cpu.pc)),
                    || n.desert_barrier_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("prefix2 desert barrier: {cases} ROM cases");
}

fn prefix2_debirando_draw_cases(entry: u32) {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..if entry == 0x0586e4 { 7 } else { 4 } {
            for ai in [0u8, 1] {
                for y in [0x40u16, 0xf8, 0x100, 0x200] {
                    let mut n = marker29_state();
                    for (a, v) in [(0xdd0, 9), (0xe20, 0x6e), (0xdc0, gfx),
                        (0xd80, ai), (0xd10, 0x40), (0xd00, y as u8),
                        (0xd20, (y >> 8) as u8)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0xfa0, k as u8);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, entry, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(entry, s,
                        |s| s.cpu.k == 5 && match entry {
                            0x0586e4 => (0x86e4..0x874d).contains(&s.cpu.pc),
                            _ => (0x8857..0x88c5).contains(&s.cpu.pc),
                        }, || if entry == 0x0586e4 { n.debirando_pit_draw(k); }
                            else { n.debirando_draw(k); });
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix2 {entry:06x}: {cases} ROM cases");
}

#[test]
fn prefix2_debirando_pit_draw_matches_rom() { prefix2_debirando_draw_cases(0x0586e4); }

#[test]
fn prefix2_debirando_draw_matches_rom() { prefix2_debirando_draw_cases(0x058857); }

#[test]
fn prefix2_antfairy_complete_entry_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..6 {
            for subtype in [0u8, 1, 255] {
                for (submodule, modal) in [(0, 0), (1, 0), (0, 1)] {
                    let mut n = marker29_state();
                    for (a, v) in [(0xdd0, 9), (0xe20, 0x15), (0xdc0, gfx),
                        (0xe80, subtype), (0xd10, 0x40), (0xd00, 0x40)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0xfa0, k as u8);
                    n.write_wram_byte(0x11, submodule);
                    n.write_wram_byte(0xfc1, modal);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x1df395, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(0x1df395, s,
                        |s| s.cpu.k == 0x1d && (0xf395..0xf3d4).contains(&s.cpu.pc),
                        || n.sprite_draw_antfairy(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix2 antifairy: {cases} ROM cases");
}

fn prefix3_moldorm_state(k: usize, y: u16) -> ZeldaState {
    let mut n = marker29_state();
    for (a, v) in [(0xdd0, 9), (0xe20, 9), (0xd10, 0x40),
        (0xd00, y as u8), (0xd20, (y >> 8) as u8), (0xcaa, 0x80)] {
        n.write_wram_byte(a + k, v);
    }
    n.write_wram_byte(0xfa0, k as u8);
    for j in 0..128 {
        n.write_wram_byte(0x1fc00 + j, 0x40);
        n.write_wram_byte(0x1fd00 + j, y as u8);
        n.write_wram_byte(0x1fd80 + j, (y >> 8) as u8);
    }
    n.sync_native_game_state_from_ram();
    n
}

#[test]
fn prefix3_archer_weapon_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in [0u8, 13, 14, 15, 19, 24, 25] {
            for direction in 0..4 {
                for y in [0u16, 0x40, 0xf8, 0xfff8] {
                    let mut n = marker29_state();
                    n.write_wram_byte(0xdc0 + k, gfx);
                    n.write_wram_byte(0xde0 + k, direction);
                    n.write_wram_word(0, 0x40);
                    n.write_wram_word(2, y);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x05d4d4, k) else { return };
                    let info = PrepOamCoordsRet { x: 0x40, y, r4: 0, flags: 0 };
                    crate::cycle_ledger::assert_rom_marker_work(0x05d4d4, s,
                        |s| s.cpu.k == 5 && (0xd4d4..0xd53b).contains(&s.cpu.pc),
                        || n.sprite_draw_archer_weapon(k, 0, &info));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix3 archer weapon: {cases} ROM cases");
}

#[test]
fn prefix3_moldorm_eyes_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for direction in 0..16 {
            for f in [0u8, 1] {
                for y in [0u16, 0x40, 0xf8, 0xfff8] {
                    let mut n = prefix3_moldorm_state(k, y);
                    n.write_wram_byte(0xde0 + k, direction);
                    n.write_wram_byte(0xea0 + k, f);
                    n.write_wram_byte(0x1a, 255);
                    n.write_wram_word(0, 0x40);
                    n.write_wram_word(2, y);
                    n.write_wram_byte(5, 0x20);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x1ddb9e, k) else { return };
                    let info = PrepOamCoordsRet { x: 0x40, y, r4: 0, flags: 0x20 };
                    crate::cycle_ledger::assert_rom_marker_work(0x1ddb9e, s,
                        |s| s.cpu.k == 0x1d && (0xdb9e..0xdc11).contains(&s.cpu.pc),
                        || n.sprite_draw_moldorm_eyeballs(k, &info));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix3 moldorm eyes: {cases} ROM cases");
}

fn prefix3_moldorm_body_cases(entry: u32) {
    let mut cases = 0;
    for k in [0usize, 15] {
        for subtype in [0u8, 3, 127, 255] {
            for y in [0x40u16, 0xf8, 0x200] {
                for selector in [0u8, 1, 2, 4] {
                    let mut n = prefix3_moldorm_state(k, y);
                    n.write_wram_byte(0xe80 + k, subtype);
                    n.write_wram_byte(0xda0 + k, selector);
                    n.write_wram_byte(0xe00 + k, selector);
                    n.write_wram_byte(0xe10 + k, selector);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, entry, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(entry, s,
                        |s| s.cpu.k == 0x1d && match entry {
                            0x1dd881 => (0xd881..0xd8f2).contains(&s.cpu.pc),
                            0x1dd993 => (0xd993..0xd9b8).contains(&s.cpu.pc),
                            0x1dd9f8 => (0xd9f8..0xda50).contains(&s.cpu.pc),
                            0x1dda50 => (0xda50..0xda5b).contains(&s.cpu.pc)
                                || (0xda00..0xda50).contains(&s.cpu.pc),
                            0x1dda5f => (0xda5f..0xdaba).contains(&s.cpu.pc),
                            0x1ddaba => (0xdaba..0xdb17).contains(&s.cpu.pc),
                            0x1ddb17 => (0xdb17..0xdb3e).contains(&s.cpu.pc)
                                || (0xda7e..0xdaba).contains(&s.cpu.pc),
                            _ => unreachable!(),
                        }, || match entry {
                            0x1dd881 => n.giant_moldorm_draw(k),
                            0x1dd993 => n.sprite_draw_moldorm_head(k),
                            0x1dd9f8 => n.giant_moldorm_draw_segment_ab(k, 16),
                            0x1dda50 => n.giant_moldorm_draw_segment_ab(k, 28),
                            0x1dda5f => n.sprite_draw_moldorm_segment_c(k),
                            0x1ddaba => n.moldorm_handle_tail(k),
                            0x1ddb17 => n.sprite_draw_moldorm_tail(k),
                            _ => unreachable!(),
                        });
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix3 {entry:06x}: {cases} ROM cases");
}

#[test]
fn prefix3_moldorm_caller_matches_rom() { prefix3_moldorm_body_cases(0x1dd881); }
#[test]
fn prefix3_moldorm_head_matches_rom() { prefix3_moldorm_body_cases(0x1dd993); }
#[test]
fn prefix3_moldorm_segment_a_matches_rom() { prefix3_moldorm_body_cases(0x1dd9f8); }
#[test]
fn prefix3_moldorm_segment_b_matches_rom() { prefix3_moldorm_body_cases(0x1dda50); }
#[test]
fn prefix3_moldorm_segment_c_matches_rom() { prefix3_moldorm_body_cases(0x1dda5f); }
#[test]
fn prefix3_moldorm_tail_handler_matches_rom() { prefix3_moldorm_body_cases(0x1ddaba); }
#[test]
fn prefix3_moldorm_tail_draw_matches_rom() { prefix3_moldorm_body_cases(0x1ddb17); }

#[test]
fn prefix3_moldorm_draw_wrapper_matches_rom() {
    let mut n = prefix3_moldorm_state(0, 0x40);
    let Some(s) = marker29_machine(&n, 0x1dd993, 0) else { return };
    crate::cycle_ledger::assert_rom_marker_work(0x1d8549, s,
        |s| s.cpu.k == 0x1d && (0x8549..0x854e).contains(&s.cpu.pc),
        || n.sprite_draw_moldorm_head(0));
}

#[test]
fn prefix3_bot_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for a in 0..4 {
            for y in [0x40u16, 0xf8, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0x14);
                n.write_wram_byte(0xd90 + k, a);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x05b89a, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x05b89a, s,
                    |s| s.cpu.k == 5 && (0xb89a..0xb8b3).contains(&s.cpu.pc),
                    || n.bot_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("prefix3 bot draw: {cases} ROM cases");
}

#[test]
fn prefix3_fairy_touchable_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for submodule in [0u8, 2, 3] {
            for msg in [0u16, 0xc8, 0xc9, 0xca, 0xcb, 0x1c9] {
                let mut n = marker29_state();
                n.write_wram_byte(0x11, submodule);
                n.write_wram_word(0x1cf0, msg);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x06d011, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x06d011, s,
                    |s| s.cpu.k == 6 && (0xd011..0xd030).contains(&s.cpu.pc),
                    || n.fairy_check_if_touchable(k));
                cases += 1;
            }
        }
    }
    eprintln!("prefix3 fairy: {cases} ROM cases");
}

#[test]
fn prefix5_buzz_blob_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..6 {
            for y in [0u16, 0x40, 0xf8, 0x100, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0x0d);
                n.write_wram_byte(0xdc0 + k, gfx);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x06d953, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x06d953, s,
                    |s| s.cpu.k == 6 && ((0xd953..0xd9c0).contains(&s.cpu.pc)
                        || s.cpu.pc == 0xdc64), || n.buzz_blob_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("prefix5 Buzz Blob: {cases} ROM cases");
}

#[test]
fn prefix5_hard_hat_beetle_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..2 {
            for shadow in [0u8, 0x10] {
                for y in [0u16, 0x40, 0xf8, 0x100, 0x200] {
                    let mut n = prefix3_moldorm_state(k, y);
                    n.write_wram_byte(0xe20 + k, 0x13);
                    n.write_wram_byte(0xdc0 + k, gfx);
                    n.write_wram_byte(0xe60 + k, shadow);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x06a4f2, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(0x06a4f2, s,
                        |s| s.cpu.k == 6 && ((0xa4f2..0xa50c).contains(&s.cpu.pc)
                            || (0xdc4c..0xdc54).contains(&s.cpu.pc) || s.cpu.pc == 0xdc64),
                        || n.hard_hat_beetle_draw(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix5 Hard Hat Beetle: {cases} ROM cases");
}

#[test]
fn prefix5_walking_zora_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..8 {
            for y in [0u16, 0x40, 0xf8, 0x100, 0x200] {
                for ripple in [0u8, 1] {
                    let mut n = prefix3_moldorm_state(k, y);
                    n.write_wram_byte(0xe20 + k, 0x8f);
                    n.write_wram_byte(0xdc0 + k, gfx);
                    n.write_wram_byte(0xeb0 + k, gfx & 3);
                    n.write_wram_byte(0xec0 + k, ripple);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x059f08, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(0x059f08, s,
                        |s| s.cpu.k == 5 && (0x9f08..0x9fac).contains(&s.cpu.pc),
                        || n.walking_zora_draw(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("prefix5 Walking Zora: {cases} ROM cases");
}

#[test]
fn prefix5_witch_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..8 {
            for y in [0x40u16, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0xe9);
                n.write_wram_byte(0xdc0 + k, gfx);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x05e55d, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x05e55d, s,
                    |s| s.cpu.k == 5 && (0xe55d..0xe62b).contains(&s.cpu.pc),
                    || n.witch_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("prefix5 Witch: {cases} ROM cases");
}

#[test]
fn prefix5_mini_moldorm_preparation_matches_rom() {
    for k in 0..4 {
        let mut n = prefix3_moldorm_state(k, 0x1234);
        n.sprite_slot_view_mut(k).set_x(0xabcd);
        let Some(s) = marker29_machine(&n, 0x1df277, k) else { return };
        crate::cycle_ledger::assert_rom_marker_work(0x1df277, s,
            |s| s.cpu.k == 0x1d && (0xf277..0xf2a5).contains(&s.cpu.pc),
            || n.sprite_prep_mini_moldorm_bounce(k));
    }
}

#[test]
fn prefix5_guard_probe_prefix_matches_rom() {
    for k in [0usize, 15] {
        for frame in 0..4u8 {
            for pause in [0u8, 1] {
                let mut n = prefix3_moldorm_state(k, 0x40);
                n.sprite_slot_view_mut(k).set_sprite_type(0x41);
                n.sprite_slot_view_mut(k).set_direction(frame);
                n.sprite_slot_view_mut(k).set_anim_clock(31);
                n.sprite_slot_view_mut(k).set_pause(pause);
                n.set_frame_counter(frame);
                let blocked = (k as u8).wrapping_add(frame) & 3 != 0 || pause != 0;
                let Some(s) = marker29_machine(&n, 0x05c5f2, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x05c5f2, s,
                    |s| s.cpu.k == 5 && ((0xc5f2..0xc612).contains(&s.cpu.pc)
                        || blocked && s.cpu.pc == 0xc66d),
                    || n.sprite_guard_send_out_probe(k));
            }
        }
    }
}

#[test]
fn prefix5_probe_spawn_body_matches_rom() {
    for k in [0usize, 15] {
        for direction in [0u8, 31, 63] {
            for free in [false, true] {
                let mut n = prefix3_moldorm_state(k, 0x40);
                for slot in 0..16 { n.sprite_slot_view_mut(slot).set_state(9); }
                if free { n.sprite_slot_view_mut(10).set_state(0); }
                n.write_wram_byte(0x0f, direction);
                let Some(s) = marker29_machine(&n, 0x05c612, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x05c612, s,
                    |s| s.cpu.k == 5 && (0xc612..0xc66e).contains(&s.cpu.pc),
                    || n.sprite_spawn_probe_always(k, direction));
            }
        }
    }
}

#[test]
fn route1_ropa_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..4 {
            for y in [0x40u16, 0xf8, 0x200] {
                for pause in [0u8, 1] {
                    let mut n = prefix3_moldorm_state(k, y);
                    n.write_wram_byte(0xe20 + k, 0x22);
                    n.write_wram_byte(0xdc0 + k, gfx);
                    n.write_wram_byte(0xf00 + k, pause);
                    n.sync_native_game_state_from_ram();
                    let Some(s) = marker29_machine(&n, 0x069ee5, k) else { return };
                    crate::cycle_ledger::assert_rom_marker_work(0x069ee5, s,
                        |s| s.cpu.k == 6 && ((0x9ee5..0x9f05).contains(&s.cpu.pc)
                            || s.cpu.pc == 0xdc64), || n.ropa_draw(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("route1 Ropa: {cases} ROM cases");
}

#[test]
fn route1_water_ripple_entries_match_rom() {
    let mut cases = 0;
    for entry in [0x059fe0, 0x059fe5, 0x059ffa] {
        for k in [0usize, 15] {
            for frame in [0u8, 4, 8, 12, 255] {
                for y in [0x40u16, 0x200] {
                    for active in [0u8, 1] {
                        let mut n = prefix3_moldorm_state(k, y);
                        n.write_wram_byte(0xe20 + k, 0x8f);
                        n.write_wram_byte(0xec0 + k, active);
                        n.write_wram_byte(0x1a, frame);
                        n.sync_native_game_state_from_ram();
                        let Some(mut s) = marker29_machine(&n, entry, k) else { return };
                        if entry == 0x059ffa {
                            // Far entry: RTL uses the third stack byte.
                            s.cpu.sp = 0x1fc;
                            s.ram[0x1fd] = 0xff;
                            s.ram[0x1fe] = 0x7f;
                            s.ram[0x1ff] = 5;
                        }
                        crate::cycle_ledger::assert_rom_marker_work(entry, s,
                            |s| s.cpu.k == 5 && if entry == 0x059ffa {
                                (0x9ffa..0xa029).contains(&s.cpu.pc)
                            } else { (entry as u16..0x9ffa).contains(&s.cpu.pc) }, || {
                                match entry {
                                    0x059fe0 => n.walking_zora_draw_water_ripples(k),
                                    0x059fe5 => n.sprite_draw_water_ripple_with_oam_adjust(k),
                                    _ => n.sprite_draw_water_ripple(k),
                                }
                            });
                        cases += 1;
                    }
                }
            }
        }
    }
    eprintln!("route1 ripple entries: {cases} ROM cases");
}

#[test]
fn route1_octorok_stone_writer_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for delay in [0u8, 8, 16, 24] {
            for y in [0u16, 0x40, 0xf8, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0x0c);
                n.write_wram_byte(0xdf0 + k, delay);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x06d643, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x06d643, s,
                    |s| s.cpu.k == 6 && (0xd643..0xd6a2).contains(&s.cpu.pc),
                    || n.sprite_draw_octorok_stone_crumbling(k));
                cases += 1;
            }
        }
    }
    eprintln!("route1 Octorok stone: {cases} ROM cases");
}

#[test]
fn route1_falling_humanoid_writer_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..14 {
            for y in [0x40u16, 0xf8, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0x41);
                n.write_wram_byte(0xdc0 + k, gfx);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x06fe5b, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x06fe5b, s,
                    |s| s.cpu.k == 6 && (0xfe5b..0xfeb4).contains(&s.cpu.pc),
                    || n.sprite_draw_falling_humanoid(k));
                cases += 1;
            }
        }
    }
    eprintln!("route1 falling humanoid: {cases} ROM cases");
}

#[test]
fn route1_talking_tree_callers_match_rom_without_body_double_charge() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for subtype in [0u8, 1] {
            for (entry, owner) in [(0x06c0d5, 0x06c0d5),
                (0x1df943, 0x1df943), (0x1df943, 0x1df94b)] {
                let mut n = prefix3_moldorm_state(k, 0x40);
                n.write_wram_byte(0xe20 + k, 0x25);
                n.write_wram_byte(0xe80 + k, subtype);
                n.write_wram_byte(0xdc0 + k, 0);
                n.write_wram_byte(0x11, 1); // Inactive mouth returns before its message states.
                n.sync_native_game_state_from_ram();
                let Some(mut s) = marker29_machine(&n, entry, k) else { return };
                if entry == 0x1df943 {
                    s.cpu.sp = 0x1fc;
                    s.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0x1d]);
                }
                crate::cycle_ledger::assert_rom_marker_work(owner, s,
                    |s| match owner {
                        0x06c0d5 => s.cpu.k == 6 && (0xc0d5..0xc0da).contains(&s.cpu.pc),
                        0x1df943 => s.cpu.k == 0x1d && (0xf943..0xf94b).contains(&s.cpu.pc),
                        _ => (s.cpu.k == 0x1d && (0xf94b..0xf952).contains(&s.cpu.pc))
                            || (s.cpu.k == 0 && (0x8784..0x879c).contains(&s.cpu.pc)),
                    }, || {
                        let _bounce = n.lag_free.then(|| crate::cycle_ledger::routine(0x06c0d5));
                        n.sprite_25_talking_tree(k);
                    });
                cases += 1;
            }
        }
    }
    eprintln!("route1 tree callers: {cases} ROM cases");
}

#[test]
fn route1_moblin_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..12 {
            for (pause, delay, y) in [(0u8, 0u8, 0x40u16), (0, 1, 0xf8),
                (1, 1, 0x40), (0, 1, 0x200)] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0x12);
                n.write_wram_byte(0xdc0 + k, gfx);
                n.write_wram_byte(0xeb0 + k, gfx & 3);
                n.write_wram_byte(0xf00 + k, pause);
                n.write_wram_byte(0xe00 + k, delay);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x069bc4, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x069bc4, s,
                    |s| s.cpu.k == 6 && ((0x9bc4..0x9c20).contains(&s.cpu.pc)
                        || s.cpu.pc == 0xdc64), || n.moblin_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("route1 Moblin: {cases} ROM cases");
}

#[test]
fn route1_crab_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for gfx in 0..2 {
            for y in [0u16, 0x40, 0xf8, 0x100, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                n.write_wram_byte(0xe20 + k, 0x0f);
                n.write_wram_byte(0xdc0 + k, gfx);
                n.sync_native_game_state_from_ram();
                let Some(s) = marker29_machine(&n, 0x059510, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x059510, s,
                    |s| s.cpu.k == 5 && (0x9510..0x956d).contains(&s.cpu.pc),
                    || n.crab_draw(k));
                cases += 1;
            }
        }
    }
    eprintln!("route1 Crab: {cases} ROM cases");
}

#[test]
fn route1_moblin_head_read_page_crossing_matches_rom() {
    let mut cases = 0;
    for pointer in [0x8e0u16, 0x8f4, 0x8f8, 0x8fc] {
        for gfx in 0..12 {
            let k = 0;
            let mut n = prefix3_moldorm_state(k, 0x40);
            n.write_wram_byte(0xe20 + k, 0x12);
            n.write_wram_byte(0xdc0 + k, gfx);
            n.write_wram_byte(0xeb0 + k, gfx & 3);
            n.write_wram_word(0x90, pointer);
            n.write_wram_word(0x92, 0xa20 + (pointer - 0x800) / 4);
            n.sync_native_game_state_from_ram();
            let Some(s) = marker29_machine(&n, 0x069bc4, k) else { return };
            crate::cycle_ledger::assert_rom_marker_work(0x069bc4, s,
                |s| s.cpu.k == 6 && ((0x9bc4..0x9c20).contains(&s.cpu.pc)
                    || s.cpu.pc == 0xdc64), || n.moblin_draw(k));
            cases += 1;
        }
    }
    eprintln!("route1 Moblin page edges: {cases} ROM cases");
}

#[test]
fn route2_octorok_inline_draw_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for direction in 0..4u8 {
            for mouth in 0..3u8 {
                for y in [0u16, 0x40, 0xef, 0xf0, 0xff, 0x100, 0x200] {
                    let mut n = prefix3_moldorm_state(k, y);
                    for (a, v) in [(0xe20, 8), (0xde0, direction), (0xdb0, mouth)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.sync_native_game_state_from_ram();
                    let s = marker29_machine(&n, 0x06d54a, k).expect("ROM fixture");
                    crate::cycle_ledger::assert_rom_marker_work(0x06d54a, s,
                        |s| s.cpu.k == 6 && (0xd54a..0xd5b9).contains(&s.cpu.pc),
                        || n.octorock_draw(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("route2 Octorok draw: {cases} ROM cases");
}

#[test]
fn route2_octorok_control_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for kind in [8u8, 10] {
            for direction in 0..4u8 {
                // Inactive, recoil, walk/stop, RNG, and both shooting bodies.
                for (submodule, recoil, ai, delay, aux, graphics) in [
                    (1, 0, 0, 0, 0, 7), (0, 8, 0, 0, 0, 0),
                    (0, 0, 0, 0, 0, 0), (0, 0, 0, 1, 0, 7),
                    (0, 0, 1, 0, 0, 0), (0, 0, 1, 1, 8, 7),
                    (0, 0, 1, 8, 0, 0), (0, 0, 1, 16, 0, 0),
                    (0, 0, 1, 28, 0, 0), (0, 0, 1, 60, 0, 0),
                    (0, 0, 1, 128, 0, 0), (0, 0, 1, 159, 0, 0),
                ] {
                    if kind == 8 && delay > 60 { continue; }
                    let mut n = prefix3_moldorm_state(k, 0x40);
                    for (a, v) in [(0xe20, kind), (0xde0, direction), (0xdd0, 9),
                        (0xd80, ai), (0xdf0, delay), (0xe00, aux), (0xdc0, graphics),
                        (0xea0, recoil), (0xf50, 0), (0xdb0, 0)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0x11, submodule);
                    n.write_wram_byte(0xfa0, k as u8);
                    n.sync_native_game_state_from_ram();
                    let s = marker29_machine(&n, 0x06d377, k).expect("ROM fixture");
                    let random = std::cell::Cell::new(None);
                    crate::cycle_ledger::assert_rom_marker_work(0x06d377, s,
                        |s| {
                            if s.cpu.k == 6 && s.cpu.pc == 0xd419 {
                                random.set(Some((s.cpu.a as u8, s.cpu.c)));
                            }
                            s.cpu.k == 6 && ((0xd377..0xd44b).contains(&s.cpu.pc)
                            || (0xd46f..0xd486).contains(&s.cpu.pc)
                            || (0xd48a..0xd4b5).contains(&s.cpu.pc))
                        }, || {
                            if let Some((value, carry)) = random.get() {
                                n.install_rom_random_replay(vec![crate::RomRandomSample::with_carry(
                                    0, value, carry)], 0);
                                n.rom_random_replay.begin_frame();
                            }
                            n.sprite_08_octorok(k);
                        });
                    cases += 1;
                }
            }
        }
    }
    eprintln!("route2 Octorok control: {cases} ROM cases");
}

#[test]
fn route2_raven_control_and_state_tails_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for ai in 0..4u8 {
            for frame in [0u8, 1, 2, 3] {
                for (x, y) in [(0x40u16, 0x40u16), (0x500, 0x40), (0x40, 0x500)] {
                    for (delay, world, subtype, submodule, recoil) in [
                        (0u8, 0u8, 0u8, 0u8, 0u8), (1, 0, 0, 0, 0),
                        (0, 1, 0, 0, 0), (0, 1, 1, 0, 0),
                        (0, 0, 0, 1, 0), (0, 0, 0, 0, 8),
                    ] {
                        let mut n = prefix3_moldorm_state(k, 0x40);
                        for (a, v) in [(0xe20, 0), (0xdd0, 9), (0xd80, ai),
                            (0xdf0, delay), (0xd90, subtype), (0xea0, recoil), (0xd50, 0xf0), (0xd40, 0x10)] {
                            n.write_wram_byte(a + k, v);
                        }
                        n.write_wram_byte(0xfff, world);
                        n.write_wram_byte(0x11, submodule);
                        n.write_wram_word(0x22, x);
                        n.write_wram_word(0x20, y);
                        n.write_wram_byte(0x1a, frame);
                        n.sync_native_game_state_from_ram();
                        let s = marker29_machine(&n, 0x1ddd85, k).expect("ROM fixture");
                        crate::cycle_ledger::assert_rom_marker_work(0x1ddd85, s,
                            |s| (s.cpu.k == 0x1d && ((0xdd85..0xdda4).contains(&s.cpu.pc)
                                || (0xddae..0xde82).contains(&s.cpu.pc)))
                                || (s.cpu.k == 0 && (0x8784..0x879c).contains(&s.cpu.pc)),
                            || n.sprite_raven(k));
                        cases += 1;
                    }
                }
            }
        }
    }
    eprintln!("route2 Raven: {cases} ROM cases");
}

#[test]
fn route2_ropa_control_and_state_tails_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for ai in 0..2u8 {
            for delay in [0u8, 1] {
                for z in [0u8, 1, 255] {
                    for submodule in [0u8, 1] {
                        let mut n = prefix3_moldorm_state(k, 0x40);
                        for (a, v) in [(0xe20, 0x22), (0xdd0, 9), (0xd80, ai),
                            (0xdf0, delay), (0xf70, z), (0xf80, 0xf0)] {
                            n.write_wram_byte(a + k, v);
                        }
                        n.write_wram_byte(0x11, submodule);
                        n.sync_native_game_state_from_ram();
                        let s = marker29_machine(&n, 0x069e1f, k).expect("ROM fixture");
                        let random = std::cell::Cell::new(None);
                        crate::cycle_ledger::assert_rom_marker_work(0x069e1f, s,
                            |s| {
                                if s.cpu.k == 6 && s.cpu.pc == 0x9e52 {
                                    random.set(Some((s.cpu.a as u8, s.cpu.c)));
                                }
                                (s.cpu.k == 6 && ((0x9e1f..0x9e40).contains(&s.cpu.pc)
                                    || (0x9e44..0x9e85).contains(&s.cpu.pc)))
                                    || (s.cpu.k == 0 && (0x8784..0x879c).contains(&s.cpu.pc))
                            }, || {
                                if let Some((value, carry)) = random.get() {
                                    n.install_rom_random_replay(vec![crate::RomRandomSample::with_carry(
                                        0, value, carry)], 0);
                                    n.rom_random_replay.begin_frame();
                                }
                                n.sprite_22_ropa(k);
                            });
                        cases += 1;
                    }
                }
            }
        }
    }
    eprintln!("route2 Ropa: {cases} ROM cases");
}

#[test]
fn route3_buzzblob_complete_and_split_caller_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for (submodule, recoil, delay, aux, split) in [
            (1u8, 0u8, 1u8, 0u8, false), (0, 8, 1, 0, false),
            (0, 0, 1, 0, false), (0, 0, 0, 0, false),
            (0, 0, 1, 1, false), (0, 0, 0, 7, false),
            (0, 0, 1, 0, true), (0, 0, 0, 0, true),
        ] {
            for y in [0x40u16, 0x200] {
                let mut n = prefix3_moldorm_state(k, y);
                for (a, v) in [(0xe20, 0x0d), (0xdd0, 9), (0xdf0, delay),
                    (0xe00, aux), (0xea0, recoil), (0xd50, 16), (0xd40, 0xf0)] {
                    n.write_wram_byte(a + k, v);
                }
                n.write_wram_byte(0x11, submodule);
                n.write_wram_byte(0xfa0, k as u8);
                n.sync_native_game_state_from_ram();
                let s = marker29_machine(&n, 0x06d89a, k).expect("ROM fixture");
                let random = std::cell::Cell::new(None);
                crate::cycle_ledger::assert_rom_marker_work(0x06d89a, s,
                    |s| {
                        if s.cpu.k == 6 && s.cpu.pc == 0xd90a {
                            random.set(Some((s.cpu.a as u8, s.cpu.c)));
                        }
                        s.cpu.k == 6 && (0xd89a..0xd8ee).contains(&s.cpu.pc)
                    }, || {
                        if let Some((value, carry)) = random.get() {
                            n.install_rom_random_replay(vec![crate::RomRandomSample::with_carry(
                                0, value, carry)], 0);
                            n.rom_random_replay.begin_frame();
                        }
                        if split {
                            n.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
                                slot: k as u8, pending: None,
                            });
                        }
                        n.sprite_0_d_buzzblob(k);
                        if split {
                            let Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
                                pending: Some((x_low, x_high)), ..
                            }) = n.sprite_main_cpu_boundary else { panic!("missing movement handoff"); };
                            n.complete_sprite_move_x_after_subpixel(k, x_low, x_high);
                            n.sprite_move_y(k, super::sprite::SpriteMoveEntry::Unpriced);
                            n.buzzblob_after_movement(k);
                        }
                    });
                cases += 1;
            }
        }
    }
    eprintln!("route3 Buzzblob caller: {cases} ROM cases");
}

#[test]
fn route3_buzzblob_direction_table_matches_rom() {
    for k in [0usize, 15] {
        for seed in 0..32u8 {
            let mut n = prefix3_moldorm_state(k, 0x40);
            n.write_wram_byte(0xfa0, k as u8);
            n.write_wram_byte(0xfa1, seed);
            n.write_wram_byte(0xfa2, 0xff - seed);
            n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n, 0x06d906, k).expect("ROM fixture");
            let random = std::cell::Cell::new(None);
            crate::cycle_ledger::assert_rom_marker_work(0x06d906, s,
                |s| {
                    if s.cpu.k == 6 && s.cpu.pc == 0xd90a {
                        random.set(Some((s.cpu.a as u8, s.cpu.c)));
                    }
                    s.cpu.k == 6 && (0xd906..0xd920).contains(&s.cpu.pc)
                }, || {
                    let (value, carry) = random.get().expect("source RNG returned");
                    n.install_rom_random_replay(vec![crate::RomRandomSample::with_carry(0, value, carry)], 0);
                    n.rom_random_replay.begin_frame();
                    n.buzzblob_select_new_direction(k);
                });
        }
    }
    eprintln!("route3 Buzzblob direction: 64 ROM cases");
}

#[test]
fn route3_firebar_inline_math_and_hit_scan_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for angle in [0u16, 1, 63, 64, 127, 128, 129, 192, 255, 256, 320, 384, 448, 511] {
            for edge in [false, true] {
            for (submodule, modal, near, y) in [
                (0u8, 0u8, false, 0x40u16), (0, 0, true, 0x40),
                (1, 0, false, 0x40), (0, 1, false, 0x40), (0, 0, true, 0x200),
            ] {
                let mut n = prefix3_moldorm_state(k, y);
                for (a, v) in [(0xe20, 0x7e), (0xdd0, 9), (0xd90, angle as u8),
                    (0xda0, (angle >> 8) as u8), (0xf60, 0)] {
                    n.write_wram_byte(a + k, v);
                }
                if edge {
                    n.write_wram_byte(0xd10 + k, 8);
                    n.write_wram_word(0x90, 0x8fc);
                    n.write_wram_word(0x92, 0xa5f);
                }
                n.write_wram_byte(0x11, submodule);
                n.write_wram_byte(0xfc1, modal);
                n.write_wram_byte(0x1a, k as u8);
                n.write_wram_byte(0xfa0, k as u8);
                n.write_wram_word(0x22, if near { 0x40 } else { 0x500 });
                n.write_wram_word(0x20, if near { y } else { 0x500 });
                n.sync_native_game_state_from_ram();
                let s = marker29_machine(&n, 0x1ed049, k).expect("ROM fixture");
                crate::cycle_ledger::assert_rom_marker_work(0x1ed049, s,
                    |s| s.cpu.k == 0x1e && (0xd049..0xd1ca).contains(&s.cpu.pc),
                    || n.firebar_main(k));
                cases += 1;
            }
            }
        }
    }
    eprintln!("route3 Firebar: {cases} ROM cases");
}

#[test]
fn route3_mimic_caller_and_eyegore_entry_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for kind in [0x83u8, 0x84] {
            for joypad in [0u8, 1, 2, 4, 5, 8, 10] {
                for (submodule, recoil, aux, defense, b, x, y) in [
                    (0u8, 0u8, 0u8, 0u8, 1u8, 0x40u16, 0x40u16),
                    (0, 0, 8, 0, 1, 0x40, 0x500),
                    (0, 0, 1, 0, 1, 0x500, 0x40),
                    (0, 0, 1, 1, 1, 0x500, 0x500),
                    (0, 8, 0, 0, 1, 0x40, 0x40),
                    (1, 0, 0, 0, 1, 0x40, 0x40),
                    (1, 0, 0, 0, 0, 0x40, 0x40),
                ] {
                    let mut n = prefix3_moldorm_state(k, 0x40);
                    for (a, v) in [(0xe20, kind), (0xdd0, 9), (0xda0, b), (0xea0, recoil),
                        (0xe00, aux), (0xd90, joypad), (0xd80, 0), (0xf60, 0), (0xe80, 7)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0x11, submodule);
                    n.write_wram_byte(0x48, defense);
                    n.write_wram_byte(0xf0, joypad);
                    n.write_wram_byte(0xfa0, k as u8);
                    n.write_wram_word(0x22, x);
                    n.write_wram_word(0x20, y);
                    n.sync_native_game_state_from_ram();
                    let s = marker29_machine(&n, 0x1ec79b, k).expect("ROM fixture");
                    crate::cycle_ledger::assert_rom_marker_work(0x1ec79b, s,
                        // Recoil's second PLA/RTS already belongs to this caller.
                        |s| s.cpu.k == 0x1e && ((0xc791..0xc839).contains(&s.cpu.pc)
                            || s.cpu.pc == 0xff18 || (s.cpu.pc == 0xff19 && s.cpu.sp == 0x1fa)),
                        || n.sprite_83_green_eyegore(k));
                    cases += 1;
                }
            }
        }
    }
    eprintln!("route3 Mimic: {cases} ROM cases");
}

#[test]
fn route4_probe_complete_and_split_match_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for parent_kind in [0x41u8, 0xce] {
            for velocity in [0u8, 1, 0xff, 0x80] {
                for (link_x, link_y, parent_ai, floor, cape, x, y) in [
                    (0x40u16, 0x40u16, 0u8, 0u8, 0u8, 0x40u16, 0x40u16),
                    (0x50, 0x40, 3, 0, 0, 0x40, 0x40),
                    (0x40, 0x50, 0, 1, 0, 0x40, 0x40),
                    (0x500, 0x500, 0, 0, 0, 0x40, 0x40),
                    (0x500, 0x500, 0, 0, 1, 0x40, 0x40),
                    (0x500, 0x500, 0, 0, 0, 0x400, 0x400),
                ] {
                    let mut n = marker29_state();
                    for (a, v) in [(0xdd0, 9), (0xdb0, 8), (0xd50, velocity),
                        (0xd40, velocity), (0xf20, floor), (0xd10, x as u8),
                        (0xd30, (x >> 8) as u8), (0xd00, y as u8), (0xd20, (y >> 8) as u8)] {
                        n.write_wram_byte(a + k, v);
                    }
                    n.write_wram_byte(0xe20 + 7, parent_kind);
                    n.write_wram_byte(0xd80 + 7, parent_ai);
                    n.write_wram_byte(0x55, cape);
                    n.write_wram_byte(0xfa0, k as u8);
                    n.write_wram_word(0x22, link_x);
                    n.write_wram_word(0x20, link_y);
                    n.sync_native_game_state_from_ram();
                    let mut split = n.clone();
                    let s = marker29_machine(&n, 0x05c15d, k).expect("ROM fixture");
                    let before = crate::cycle_ledger::master();
                    crate::cycle_ledger::assert_rom_marker_work(0x05c15d, s,
                        |s| s.cpu.k == 5 && (0xc15d..0xc227).contains(&s.cpu.pc),
                        || n.probe(k));
                    let atomic_work = crate::cycle_ledger::master() - before;
                    let before = crate::cycle_ledger::master();
                    if let Some(p) = split.probe_until_after_oam_coordinates(k) {
                        split.complete_probe_after_oam_coordinates(k, p);
                    }
                    assert_eq!(crate::cycle_ledger::master() - before, atomic_work);
                    assert_eq!(n.wram_image(), split.wram_image());
                    cases += 1;
                }
            }
        }
    }
    eprintln!("route4 Probe: {cases} ROM cases");
}

#[test]
fn route4_lightning_caller_matches_rom() {
    let mut cases = 0;
    for k in [0usize, 15] {
        for phase in 0..8u8 {
            for (submodule, delay, room, y) in [
                (0u8, 0u8, 0u16, 0x40u16), (0, 1, 0x20, 0x40),
                (1, 0, 0, 0x40), (0, 0, 0x20, 0xd0), (0, 0, 0, 0x400),
            ] {
                let mut n = prefix3_moldorm_state(k, y);
                for (a, v) in [(0xe20, 0xbf), (0xd90, phase), (0xdf0, delay)] {
                    n.write_wram_byte(a + k, v);
                }
                n.write_wram_byte(0x11, submodule);
                n.write_wram_word(0x48e, room);
                n.sync_native_game_state_from_ram();
                let s = marker29_machine(&n, 0x1de3ed, k).expect("ROM fixture");
                let random = std::cell::Cell::new(None);
                crate::cycle_ledger::assert_rom_marker_work(0x1de3ed, s,
                    |s| {
                        if s.cpu.k == 0x1d && s.cpu.pc == 0xe44a {
                            random.set(Some((s.cpu.a as u8, s.cpu.c)));
                        }
                        s.cpu.k == 0x1d && (0xe3ed..0xe475).contains(&s.cpu.pc)
                    }, || {
                        if let Some((value, carry)) = random.get() {
                            n.install_rom_random_replay(vec![crate::RomRandomSample::with_carry(
                                0, value, carry)], 0);
                            n.rom_random_replay.begin_frame();
                        }
                        n.sprite_bf_lightning(k);
                    });
                cases += 1;
            }
        }
    }
    eprintln!("route4 Lightning: {cases} ROM cases");
}

#[test]
fn route4_energy_ball_caller_and_trail_match_rom() {
    let mut cases = 0;
    for k in [1usize, 15] {
        for (ai, graphics, delay, seeker, reflected, submodule, frame, near, sword) in [
            (1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, false, false),
            (1, 2, 6, 0, 0, 0, 0, false, false),
            (1, 0, 15, 1, 0, 0, 0, false, false),
            (1, 0, 15, 0, 0, 1, 0, false, false),
            (0, 0, 0, 0, 0, 1, 0, false, false),
            (0, 0, 0, 0, 0, 0, 0, false, false),
            (0, 0, 0, 0, 0, 0, k as u8, false, false),
            (0, 0, 0, 0, 1, 0, 0, true, false),
            (0, 0, 0, 0, 1, 0, 0, false, false),
            (0, 0, 1, 1, 0, 0, 0, false, false),
            (0, 0, 0, 0, 0, 0, 0, true, true),
            (0, 0, 0, 1, 0, 0, 0, true, true),
        ] {
            let mut n = prefix3_moldorm_state(k, 0x40);
            for (a, v) in [(0xe20, 0x7b), (0xd80, ai), (0xdc0, graphics),
                (0xdf0, delay), (0xda0, seeker), (0xd90, reflected), (0xf60, 0),
                (0xd50, 0), (0xd40, 0), (0xe40, 0), (0xea0, 0)] {
                n.write_wram_byte(a + k, v);
            }
            n.write_wram_byte(0x11, submodule);
            n.write_wram_byte(0x1a, frame);
            n.write_wram_byte(0xdd0, 9);
            n.write_wram_byte(0xe40, 0);
            n.write_wram_byte(0xd10, if near { 0x40 } else { 0x80 });
            n.write_wram_byte(0xd00, 0x40);
            n.write_wram_byte(0xba0, 0);
            n.write_wram_byte(0x3c, if sword { 9 } else { 0 });
            n.write_wram_word(0x22, if near { 0x40 } else { 0x500 });
            n.write_wram_word(0x20, if near { 0x40 } else { 0x500 });
            n.sync_native_game_state_from_ram();
            let s = marker29_machine(&n, 0x1eda42, k).expect("ROM fixture");
            crate::cycle_ledger::assert_rom_marker_work(0x1eda42, s,
                |s| s.cpu.k == 0x1e && ((0xda42..0xdb44).contains(&s.cpu.pc)
                    || (0xdb54..0xdb8a).contains(&s.cpu.pc)),
                || n.sprite_7_b_agahnim_balls(k));
            cases += 1;
        }
    }
    eprintln!("route4 Energy ball: {cases} ROM cases");
}
