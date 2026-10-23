use super::*;
use crate::rom_random::RomRandomResult;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
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

    atomic.sprite_draw_shadow_custom(2, &mut atomic_info, 10);
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
