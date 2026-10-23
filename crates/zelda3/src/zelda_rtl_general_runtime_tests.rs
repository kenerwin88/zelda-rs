use super::*;
use crate::dialogue_ir::{
    DialogueIrKind, TEXT_CMD_2, TEXT_CMD_COLOR, TEXT_CMD_END_MESSAGE, TEXT_CMD_NAME,
    TEXT_CMD_NUMBER, TEXT_CMD_WAIT, TEXT_COMMAND_START_US,
};
use crate::game_state::constants::{
    ANCILLA_ALLOC_ROTATE, ANCILLA_TYPE, ANCILLA_X_LO, ANCILLA_X_VELOCITY, ANIMATED_TILE_DATA_SRC,
    BG2_X_SCROLL, BG_TILE_ANIMATION_COUNTDOWN, DIALOGUE_MESSAGE_INDEX, DMA_SOURCE_ADDR_0,
    DMA_SOURCE_ADDR_1, DMA_SOURCE_ADDR_10, DMA_SOURCE_ADDR_11, DMA_SOURCE_ADDR_12,
    DMA_SOURCE_ADDR_13, DMA_SOURCE_ADDR_14, DMA_SOURCE_ADDR_15, DMA_SOURCE_ADDR_16,
    DMA_SOURCE_ADDR_17, DMA_SOURCE_ADDR_18, DMA_SOURCE_ADDR_19, DMA_SOURCE_ADDR_2,
    DMA_SOURCE_ADDR_20, DMA_SOURCE_ADDR_21, DMA_SOURCE_ADDR_3, DMA_SOURCE_ADDR_4,
    DMA_SOURCE_ADDR_5, DMA_SOURCE_ADDR_6, DMA_SOURCE_ADDR_7, DMA_SOURCE_ADDR_8, DMA_SOURCE_ADDR_9,
    DUNG_BG1, DUNG_BG2, HDMAEN_COPY, INIDISP_COPY, MOSAIC_TARGET_LEVEL, PALETTE_FILTER_COUNTDOWN,
    SUBSUBMODULE, TM_COPY, TS_COPY,
};
use crate::game_state::constants::{FLAG_IS_ANCILLA_TO_PICK_UP, SPRITE_LIMIT_INSTANCE};
use crate::game_state::constants::{MAP16_LOAD_DST_OFF, MAP16_LOAD_SRC_OFF, MAP16_LOAD_Y_UNIT};
use crate::game_state::CachedSpriteCacheField;
use crate::game_state::FrameState;
use crate::{JoypadPublication, NmiUpdateGate};

#[test]
fn native_cold_display_preserves_hardware_before_memory_initialization() {
    let Some(path) = std::env::var_os("ZELDA3_ASSETS") else { return };
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_assets(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(state.read_wram_byte(crate::game_state::constants::NMI_COPY_PACKETS_FLAG), 0x55);
    for host in 0..=80 {
        state.frame_ctr_dbg = host + 1;
        assert_eq!(state.advance_native_startup_host(0), Some(false));
        state.capture_display_snapshot();
        let uninitialized = state.native_startup_cpu.as_ref().unwrap().startup.memory_initialization_pending();
        let ram = state.wram_image().into_owned();
        let ppu = bincode::serialize(&state.ppu).unwrap();
        for _ in 0..2 {
            let shown = state.with_display_snapshot(|game| bincode::serialize(&game.ppu).unwrap());
            if uninitialized { assert_eq!(shown, ppu, "pre-initializer display has no game-authored DMA requests"); }
            assert_eq!(*state.wram_image(), ram, "capture cannot author reset RAM");
            assert_eq!(bincode::serialize(&state.ppu).unwrap(), ppu);
        }
    }
    assert!(!state.native_startup_cpu.as_ref().unwrap().startup.memory_initialization_pending());
}

#[test]
fn native_startup_host_continuation_enters_its_module_once_after_rti() {
    let Some(path) = std::env::var_os("ZELDA3_ASSETS") else { return };
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_assets(&std::fs::read(path).unwrap()).unwrap();
    state.ram[0x100] = 0xa5;
    for host in 0..=81 {
        state.frame_ctr_dbg = host + 1;
        let completed_nmi = state.advance_native_startup_host(0x0841);
        assert_eq!(completed_nmi, Some(host == 81));
        if host == 79 {
            // The first initializer slice cannot publish stores belonging to
            // its following host. The loop is still above this lower word.
            assert_eq!(state.ram[0x100], 0xa5);
        }
        if host == 80 {
            assert_eq!(state.ram[0x100], 0);
            assert_eq!(state.ram[0x15], 1);
            // Initialization does not write the animated BG source. It must
            // remain a real DMA operand, not a boot-completion sentinel.
            assert!(!state.game_state.display.has_animated_tile_data_source());
            assert_eq!(state.paired_resume_cpu_boundary_blocker(), Some("retained-native-bootstrap-clock"));
            // The native header must queue these sampled effects even after
            // its physical STZs clear their gameplay latches.
            state.system_signals_mut().set_sound_effect_1_word(0x0201);
            state.follower_link_state_mut().set_joypad1h_last2(0x80);
            state.follower_link_state_mut().set_joypad1l_last2(0x11);
            for (bg, base) in state.ppu.bg_layer.iter_mut().zip([0x1000, 0x3000, 0x5000, 0x6000]) {
                bg.tile_adr = base;
            }
            for index in 0..128 {
                state.oam_state_mut().write_entry(0x800 + index * 4, index as u8, 17, 0x23, 0x31);
            }
        }
    }
    // Zero-SRAM startup accepts at 28_896_206, then the independently tested
    // handler executes RTI to8036. Its saved BEQ still sees zero, so the
    // wait costs62 (taken BEQ, LDA, untaken BEQ) before reaching8038.
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert_eq!(cpu.timeline.clock_master_cycles(), 28_933_252);
    assert_eq!(cpu.timeline.raster_position(), snes::CpuRasterPosition::new(252, 244));
    let caller = cpu.startup.main_caller_context().unwrap();
    assert_eq!((caller.bank, caller.pc, caller.sp, caller.a), (0, 0x8038, 0x1ff, 1));
    // The native tail does not perform the semantic helper's extra CHR-base stores.
    assert_eq!(state.ppu.bg_layer.map(|bg| bg.tile_adr), [0x1000, 0x3000, 0x5000, 0x6000]);
    assert_eq!(&state.wram_image()[0..2], &[0x10, 0x82]);
    let input = state.game_state.player.follower_link;
    assert_eq!((input.joypad1l_last(), input.joypad1h_last()), (0x10, 0x82));
    assert_eq!((input.joypad1l_last2(), input.joypad1h_last2()), (0x10, 0x82));
    assert_eq!((input.filtered_joypad_l(), input.filtered_joypad_h()), (0, 2));
    // Seven core MDMAEN writes select 23 channels / 2304 bytes. The
    // following palette and OAM each add one run and one channel.
    assert_eq!(state.last_nmi_dma_master_cycles, 9 * 18 + 25 * 8 + (2304 + 512 + 544) * 8);
    let oam = state.dma.channel[0];
    let palette = state.dma.channel[1];
    assert_eq!((oam.mode, oam.b_adr, oam.a_bank, oam.a_adr, oam.size), (0, 4, 0, 0xa20, 0));
    assert_eq!((palette.mode, palette.b_adr, palette.a_bank, palette.a_adr, palette.size),
        (0, 0x22, 0x7e, 0xc700, 0));
    assert_eq!(&state.wram_image()[0x15..0x17], &[0, 0]);
    assert!(!state.game_state.system_signals.should_update_hud());
    assert!(!state.game_state.system_signals.should_update_cgram());
    assert!(!cpu.startup.pending());
    assert_eq!(state.game_state.frame.frame_counter, 0);
    assert_eq!(state.ppu.vram[0x4000], 0x8000);
    assert_eq!(state.ppu.vram[0x4009], 0x8001);
    assert_eq!(state.ppu.vram[0], 0x8000);
    assert_eq!(state.ppu.vram[9], 0x8001);
    assert_eq!(&state.zelda_debug_apu_write_ports()[2..], &[1, 2]);
    assert_eq!(&state.wram_image()[0x12e..0x130], &[0, 0]);
    assert_eq!(state.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);
    assert_eq!(state.advance_native_startup_host(0), None);
    assert_eq!(state.paired_resume_cpu_boundary_blocker(), Some("retained-native-apu-port-access"));
    let presented_oam = state.ppu.oam.clone();
    assert!(state.try_native_startup_main_entry());
    assert_eq!(state.game_state.frame.frame_counter, 1);
    for index in 0..128 {
        assert_eq!(&state.wram_image()[0x800 + index * 4..0x804 + index * 4], &[index as u8, 0xf0, 0x23, 0x31]);
    }
    assert_eq!(state.ppu.oam, presented_oam, "CPU buffer clear cannot republish OAM");
    assert_eq!(&state.wram_image()[3..6], &[0x20, 0xc1, 0x0c]);
    assert_eq!(&state.wram_image()[0x1fd..0x200], &[0x59, 0x80, 0]);
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert!(cpu.startup.main_caller_context().is_none());
    let module = cpu.startup.entered_module_context().unwrap();
    assert_eq!((module.bank, module.pc, module.sp, module.a, module.x, module.y),
        (0x0c, 0xc120, 0x1fc, 0x0c, 0xe0, 0));
    assert_eq!(cpu.timeline.clock_master_cycles(), 28_939_174);
    assert_eq!(cpu.timeline.raster_position(), snes::CpuRasterPosition::new(256, 710));
    let clock = cpu.timeline.clone();
    let ram = state.wram_image().into_owned();
    assert!(!state.try_native_startup_main_entry(), "native caller work must execute once");
    assert_eq!(*state.wram_image(), ram);
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline, clock);
    state.ppu.forced_blank = false;
    state.ppu.brightness = 7;
    state.ppu.obj_tile_adr1 = 0;
    assert!(state.try_native_intro_setup_entry());
    assert!(state.ppu.forced_blank, "setup must write INIDISP before its graphics callee");
    assert_eq!(state.ppu.brightness, 0);
    assert_eq!(state.ppu.obj_tile_adr1, 0x4000);
    assert_eq!(state.ppu.bg_layer.map(|bg| bg.tile_adr), [0x2000, 0x2000, 0x7000, 0]);
    assert_eq!((state.game_state.display.screen_brightness, state.game_state.display.chr_halfslot_request), (0x80, 20));
    assert_eq!(&state.wram_image()[0..3], &[0x5d, 0xc1, 0x0c]);
    assert_eq!(&state.wram_image()[0x1f7..0x200], &[0x23, 0x80, 2, 0x60, 0xc1, 12, 0x59, 0x80, 0]);
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert!(cpu.startup.entered_module_context().is_none());
    let graphics = cpu.startup.intro_half_slot_context().unwrap();
    assert_eq!((graphics.bank, graphics.pc, graphics.sp, graphics.a, graphics.x, graphics.y),
        (0, 0xe3fa, 0x1f6, 0x0c14, 0xe0, 0));
    assert!(cpu.timeline.clock_master_cycles() > clock.clock_master_cycles());
    let clock = cpu.timeline.clone();
    let ram = state.wram_image().into_owned();
    let ppu = bincode::serialize(&state.ppu).unwrap();
    assert!(!state.try_native_intro_setup_entry(), "native setup must execute once");
    assert_eq!(*state.wram_image(), ram);
    assert_eq!(bincode::serialize(&state.ppu).unwrap(), ppu);
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline, clock);
    let rom_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
    let rom = std::fs::read(rom_path).unwrap();
    assert_eq!(state.asset_bytes(64, 8).unwrap(), &rom[0x8a000..0x8a600]);
    assert_eq!(state.asset_bytes(64, 9).unwrap()[0], rom[0x8a600]);
    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&state.wram_image());
    machine.ppu = state.ppu.clone();
    machine.cpu.a = graphics.a; machine.cpu.x = graphics.x; machine.cpu.y = graphics.y;
    machine.cpu.sp = graphics.sp; machine.cpu.pc = graphics.pc;
    machine.cpu.k = graphics.bank; machine.cpu.db = graphics.db; machine.cpu.dp = graphics.d;
    machine.cpu.e = false; machine.cpu.unpack_flags(graphics.p);
    let mut source = snes::RomCpuTimingProbe::new(machine, clock,
        snes::SourcePpuReadState::snes9x_reset()).unwrap();
    for step in 0..20000 {
        if (source.snes().cpu.k, source.snes().cpu.pc) == (2, 0x8024) { break; }
        assert!(step < 19999);
        source.step().unwrap();
    }
    assert!(state.try_native_intro_half_slot());
    assert_eq!(*state.wram_image(), source.snes().ram, "native asset bus and typed publications must match ROM");
    assert_eq!(bincode::serialize(&state.ppu).unwrap(), ppu, "conversion writes WRAM, not PPU memory");
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert_eq!(cpu.timeline, *source.timeline());
    assert!(cpu.startup.intro_half_slot_context().is_none());
    let returned = cpu.startup.intro_setup_after_half_slot_context().unwrap();
    let expected = &source.snes().cpu;
    assert_eq!((returned.a, returned.x, returned.y, returned.sp, returned.pc, returned.bank, returned.db, returned.d, returned.p),
        (expected.a, expected.x, expected.y, expected.sp, expected.pc, expected.k, expected.db, expected.dp, expected.pack_flags()));
    assert_eq!(state.game_state.display.chr_halfslot_request, 0);
    let clock = cpu.timeline.clone();
    let ram = state.wram_image().into_owned();
    assert!(!state.try_native_intro_half_slot(), "native half-slot must execute once");
    assert_eq!(*state.wram_image(), ram);
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline, clock);
    for step in 0..256 {
        if (source.snes().cpu.k, source.snes().cpu.pc) == (12, 0xc161) { break; }
        assert!(step < 255);
        source.step().unwrap();
    }
    assert!(state.try_native_intro_setup_return());
    assert_eq!(*state.wram_image(), source.snes().ram, "setup typed palette and scratch publications must match ROM");
    assert_eq!(bincode::serialize(&state.ppu).unwrap(), bincode::serialize(&source.snes().ppu).unwrap());
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert_eq!(cpu.timeline, *source.timeline());
    assert!(cpu.startup.intro_setup_after_half_slot_context().is_none());
    let returned = cpu.startup.intro_after_setup_context().unwrap();
    let expected = &source.snes().cpu;
    assert_eq!((returned.a, returned.x, returned.y, returned.sp, returned.pc, returned.bank, returned.db, returned.d, returned.p),
        (expected.a, expected.x, expected.y, expected.sp, expected.pc, expected.k, expected.db, expected.dp, expected.pack_flags()));
    assert_eq!(&state.ppu.vram[0x27f0..0x2801], &[0; 17]);
    let clock = cpu.timeline.clone();
    let ram = state.wram_image().into_owned();
    let ppu = bincode::serialize(&state.ppu).unwrap();
    assert!(!state.try_native_intro_setup_return(), "native setup return must execute once");
    assert_eq!(*state.wram_image(), ram);
    assert_eq!(bincode::serialize(&state.ppu).unwrap(), ppu);
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline, clock);

    for step in 0..512 {
        if (source.snes().cpu.k, source.snes().cpu.pc) == (12, 0xc1a0) { break; }
        assert!(step < 511);
        source.step().unwrap();
    }
    assert!(state.try_native_intro_first_clear_entry());
    assert_eq!(*state.wram_image(), source.snes().ram, "typed intro/OAM stores must match source before the first clear");
    assert_eq!(bincode::serialize(&state.ppu).unwrap(), ppu, "logo writes CPU OAM buffers only");
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert_eq!(cpu.timeline, *source.timeline());
    assert!(cpu.startup.intro_after_setup_context().is_none());
    let entered = cpu.startup.intro_first_clear_context().unwrap();
    let expected = &source.snes().cpu;
    assert_eq!((entered.a, entered.x, entered.y, entered.sp, entered.pc, entered.bank, entered.db, entered.d, entered.p),
        (expected.a, expected.x, expected.y, expected.sp, expected.pc, expected.k, expected.db, expected.dp, expected.pack_flags()));
    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.frame.subsubmodule, 1);
    assert_eq!(state.game_state.display.screen_brightness, 15);
    assert!(state.game_state.system_signals.should_update_cgram());
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 10);
    let clock = cpu.timeline.clone();
    let ram = state.wram_image().into_owned();
    assert!(!state.try_native_intro_first_clear_entry());
    assert_eq!(*state.wram_image(), ram);
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline, clock);

    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
    machine.ram.copy_from_slice(&state.wram_image());
    machine.cpu = source.snes().cpu.clone();
    machine.ppu = state.ppu.clone();
    machine.nmi_enabled = true;
    let mut clear_source = snes::RomCpuTimingProbe::new(machine, clock,
        snes::SourcePpuReadState::snes9x_reset()).unwrap();
    // The source NMI samples real synchronized ports. Clone the complete SMP
    // continuation and clock, never just its currently visible output bytes.
    clear_source.attach_apu_port_owner(state.native_apu_port_owner_for_test()).unwrap();
    for instruction in 0..20000 {
        if clear_source.pending_nmi_acceptance().is_some() { break; }
        assert!(instruction < 19999);
        clear_source.step().unwrap();
    }
    let accepts_before_host_return = clear_source.pending_nmi_acceptance()
        .is_some_and(|at| clear_source.timeline().timestamp() >= at);
    if accepts_before_host_return { clear_source.accept_native_nmi().unwrap(); }
    state.begin_native_intro_clear();
    assert!(state.native_intro_clear_pending());
    assert_eq!(state.intro_initialization_work_frames_pending, 0, "native work cannot use a host-count hold");
    assert_eq!(*state.wram_image(), clear_source.snes().ram, "only stores before hardware NMI acceptance may be published");
    assert_eq!(state.ram[0x3c00], ram[0x3c00], "future clear column must remain untouched");
    assert_eq!(bincode::serialize(&state.ppu).unwrap(), ppu);
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert_eq!(cpu.timeline, *clear_source.timeline());
    assert_eq!(cpu.startup.intro_clear_accepted_context().is_some(), accepts_before_host_return);
    let accepted = cpu.startup.intro_clear_context().unwrap();
    let expected = &clear_source.snes().cpu;
    assert_eq!((accepted.a, accepted.x, accepted.y, accepted.sp, accepted.pc, accepted.bank, accepted.db, accepted.d, accepted.p),
        (expected.a, expected.x, expected.y, expected.sp, expected.pc, expected.k, expected.db, expected.dp, expected.pack_flags()));

    if !accepts_before_host_return {
        while clear_source.pending_nmi_acceptance().is_some_and(|at| clear_source.timeline().timestamp() < at) {
            clear_source.step().unwrap();
        }
        clear_source.accept_native_nmi().unwrap();
        let mut accepted_probe = state.clone();
        assert!(!accepted_probe.run_native_intro_clear_until_boundary());
        assert_eq!(*accepted_probe.wram_image(), clear_source.snes().ram);
        assert_eq!(accepted_probe.native_startup_cpu.as_ref().unwrap().timeline, *clear_source.timeline());
    }
    state.frame_ctr_dbg += 1;
    state.main_loop_caller_phase = crate::cycle_models::main_wait::MainLoopCallerPhase::ModuleRouting;
    state.pending_main_loop_common_suffix = Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.resume_native_intro_clear_host(0x0841);
    assert!(!state.native_intro_clear_pending(), "late acceptance must not add a held host");
    let returned = state.native_startup_cpu.as_ref().unwrap().startup.intro_tail_returned_context().unwrap();
    assert_eq!((returned.bank, returned.pc, returned.sp), (0, 0x8034, 0x1ff));
    for instruction in 0..100000 {
        if clear_source.snes().cpu.pc == 0x8034 && clear_source.snes().cpu.k == 0 { break; }
        assert!(instruction < 99999);
        clear_source.step().unwrap();
    }
    let expected = &clear_source.snes().cpu;
    assert_eq!((returned.a, returned.x, returned.y, returned.sp, returned.pc, returned.p),
        (expected.a, expected.x, expected.y, expected.sp, expected.pc, expected.pack_flags()));
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline, *clear_source.timeline());
    assert_eq!(&state.wram_image()[0xa00..0xa20], &clear_source.snes().ram[0xa00..0xa20]);
    assert_eq!(&state.wram_image()[0xac0..0xadc], &clear_source.snes().ram[0xac0..0xadc]);
    assert_eq!(*state.wram_image(),clear_source.snes().ram,"complete sprite caller RAM ownership");
    assert_eq!(&state.wram_image()[0x1fe..0x200], &clear_source.snes().ram[0x1fe..0x200]);
    assert_eq!(state.intro_initialization_work_frames_pending, 0);
    assert!(state.pending_main_loop_common_suffix.is_none());
    for block in 1..=15 {
        assert!(state.ram[block*0x2000+0x1c00..(block+1)*0x2000].iter().all(|&b| b==0));
    }
    // The same returned caller must own the remaining wait in this host,
    // then the next actual NMI and RTI. No post-NMI register/clock seed.
    assert_eq!(state.advance_native_startup_host(0), Some(false));
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    for step in 0..2000 {
        if clear_source.timeline().clock_master_cycles() >= cpu.timeline.clock_master_cycles() { break; }
        assert!(step < 1999);
        clear_source.step().unwrap();
    }
    assert_eq!(cpu.timeline, *clear_source.timeline());
    let waiting = cpu.startup.waiting_context().unwrap();
    let expected = &clear_source.snes().cpu;
    assert_eq!((waiting.a, waiting.pc, waiting.p, waiting.sp),
        (expected.a, expected.pc, expected.pack_flags(), expected.sp));
    assert!(cpu.startup.pending());
    // This probe retains an APU access as well; that guard has priority over
    // the live bootstrap clock. Neither owner can be checkpointed here.
    assert_eq!(state.paired_resume_cpu_boundary_blocker(), Some("retained-native-apu-port-access"));

}

#[test]
fn native_next_intro_nmi_matches_full_cold_source_owner() {
    let Some(path) = std::env::var_os("ZELDA3_ASSETS") else { return };
    let Ok(rom) = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")) else { return };
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_assets(&std::fs::read(path).unwrap()).unwrap();
    let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&state.sram)).unwrap();
    for host in 0..=82 {
        source.set_joypad_serial_state(0, 0);
        source.run_until_main_loop_return().unwrap();
        // Drive the core host entry, excluding frontend recorder patches
        // such as its RAM_APUI00 music-status byte. Keep every core RAM byte
        // in the comparison; the host ordinal is external input to both CPUs.
        state.frame_ctr_dbg = host + 1;
        state.run_frame_internal(0, 1);
    }
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    assert_eq!(cpu.timeline, *source.machine().timeline());
    assert_eq!(*state.wram_image(), source.machine().snes().ram, "cold RAM at wait host return");
    for step in 0..20000 {
        let cpu = &source.machine().snes().cpu;
        if cpu.k == 0 && cpu.pc == 0x8038 { break; }
        assert!(step < 19999);
        source.step().unwrap();
    }
    state.frame_ctr_dbg += 1;
    assert_eq!(state.advance_native_startup_host(0), Some(true));
    let cpu = state.native_startup_cpu.as_ref().unwrap();
    let returned = cpu.startup.main_caller_context().unwrap();
    let expected = &source.machine().snes().cpu;
    assert_eq!((returned.a, returned.x, returned.y, returned.sp, returned.pc, returned.p),
        (expected.a, expected.x, expected.y, expected.sp, expected.pc, expected.pack_flags()));
    assert_eq!(cpu.timeline, *source.machine().timeline());
    assert_eq!(*state.wram_image(), source.machine().snes().ram, "next cold native NMI RAM ownership");
    assert_eq!(state.ppu.cgram, source.machine().snes().ppu.cgram, "next NMI palette DMA");
    assert_eq!(state.ppu.vram, source.machine().snes().ppu.vram, "next NMI graphics DMA");
    state.native_ppu.as_mut().unwrap().assert_matches_source(&cpu.timeline, source.machine());
    state.zelda_run_game_loop();
    assert!(state.native_startup_cpu.as_ref().unwrap().startup.entered_module_context().is_none(),
        "the following clear must claim its module entry");
    assert!(state.native_intro_clear_pending());
}

#[test]
fn native_repeated_intro_clears_match_full_cold_source_hosts() {
    let Some(path) = std::env::var_os("ZELDA3_ASSETS") else { return };
    let Ok(rom) = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")) else { return };
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_assets(&std::fs::read(path).unwrap()).unwrap();
    let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&state.sram)).unwrap();
    for host in 0..128 {
        source.set_joypad_serial_state(0, 0);
        source.run_until_main_loop_return().unwrap();
        state.frame_ctr_dbg = host + 1;
        state.run_frame_internal(0, 1);
        let samples = source.take_dsp_samples().samples.len();
        let mut output = vec![0; samples * 2];
        state.zelda_render_audio(&mut output, samples as i32, 2);
        if host < 82 { continue; }
        let cpu = state.native_startup_cpu.as_ref().unwrap();
        assert_eq!(cpu.timeline, *source.machine().timeline(), "host={host}");
        let diffs: Vec<_> = state.wram_image().iter().zip(&source.machine().snes().ram).enumerate()
            .filter(|(_, (a,b))| a != b).map(|(i,(a,b))| (i,*a,*b)).collect();
        assert!(diffs.is_empty(), "host={host} RAM differences={} first={:?}",diffs.len(),&diffs[..diffs.len().min(8)]);
        assert_eq!(state.ppu.vram, source.machine().snes().ppu.vram, "VRAM host={host}");
        assert_eq!(state.ppu.cgram, source.machine().snes().ppu.cgram, "CGRAM host={host}");
        assert_eq!(state.ppu.oam, source.machine().snes().ppu.oam, "OAM host={host}");
        assert_eq!(state.intro_initialization_work_frames_pending, 0);
        if state.game_state.frame.subsubmodule == 8
            && state.main_loop_caller_phase == crate::cycle_models::main_wait::MainLoopCallerPhase::Waiting {
            assert!(cpu.startup.pending(), "the final wait or accepted NMI must retain its clock");
            return;
        }
    }
    panic!("native clear sequence did not finish");
}

#[test]
fn native_startup_retains_derived_clock_and_memory_ownership() {
    let Some(path) = std::env::var_os("ZELDA3_ASSETS") else { return };
    let assets = std::fs::read(path).unwrap();
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_assets(&assets).unwrap();
    assert!(state.wram_image().iter().all(|&byte| byte == 0x55), "native cold reset must initialize the full WRAM bank pair");
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline.clock_master_cycles(), 28_396_112);
    assert_eq!(state.paired_resume_cpu_boundary_blocker(), Some("retained-native-bootstrap-clock"));
    // The initializer owns its store footprint, not every byte of low WRAM.
    write_le_u16(&mut state.ram, 0x400, 0xbeef);
    let before_reload = state.wram_image().into_owned();
    state.set_assets(&assets).unwrap();
    assert_eq!(*state.wram_image(), before_reload, "asset reload must not repeat cold WRAM reset");
    state.zelda_initialization_code();
    assert_eq!(state.native_startup_cpu.as_ref().unwrap().timeline.clock_master_cycles(), 28_617_174);
    assert_ne!(state.paired_resume_cpu_boundary_blocker(), Some("retained-native-bootstrap-clock"));
    assert_eq!(read_le_u16(&state.wram_image(), 0x400), 0xbeef);
    assert_eq!(&state.wram_image()[..3], &[0, 0x80, 0]);
    assert_eq!(read_le_u16(&state.wram_image(), 0x1fe), 0x802e);
    assert_eq!(state.ram[0x13], 0x80);
    assert_eq!(state.ram[0x15], 1);
    assert_eq!(read_le_u16(&state.wram_image(), 0xc00d), 0x5555);
    assert_eq!(read_le_u16(&state.wram_image(), 0xc013), 0x5555);
    for word in 0..256 {
        let value = if word == 0 { 0 } else { 0x5555 };
        assert_eq!(state.game_state.display.palette_provenance.0.main[word],
            zelda3_palette::MirrorWord::Known(value, zelda3_palette::SourceTag::Constant));
    }
    let mut progressed = state.clone();
    progressed.frame_ctr_dbg = 1;
    progressed.clear_spc_driver_clock();
    let before_reload = progressed.wram_image().into_owned();
    progressed.set_assets(&assets).unwrap();
    assert_eq!(*progressed.wram_image(), before_reload, "progressed states cannot acquire cold reset values");
    state.set_rom_startup_timing(false);
    assert!(state.native_startup_cpu.is_none());
}

fn dungeon_reset_progress_receipt(
    progress: DungeonResetSpritesCpuProgress,
    boundary: OriginalTimingBoundary,
) -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(DungeonResetSpritesProgressReceipt {
        progress,
        boundary,
    })
}

fn cached_sprite_progress_receipt(
    progress: crate::CachedSpriteExecutionProgress,
    boundary: OriginalTimingBoundary,
) -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
        crate::CachedSpriteExecutionProgressReceipt { progress, boundary },
    )
}

fn test_sync_all(state: &mut ZeldaState) {
    state.ram[0x42] = state.ram[0x42].wrapping_add(1);
}

fn test_run_frame_callback(state: &mut ZeldaState, input: u16, run_what: i32) {
    state.ram[0x43] = state.ram[0x43].wrapping_add(1);
    state.ram[0x44] = input as u8;
    state.ram[0x45] = run_what as u8;
}

fn test_run_frame_callback_calls_internal(state: &mut ZeldaState, input: u16, _: i32) {
    state.ram[0x43] = state.ram[0x43].wrapping_add(1);
    state.run_frame_internal(input, 0);
}

fn test_run_frame_callback_observes_snes9x_semantic_receipts(
    state: &mut ZeldaState,
    input: u16,
    _: i32,
) {
    assert!(state.original_timing_host_dispatch_active);
    assert!(matches!(
        state.original_timing_owner,
        OriginalTimingOwnerState::Live
    ));
    let receipts = state
        .original_timing_semantic_receipts
        .as_ref()
        .expect("oracle semantic receipts remain visible during this host dispatch");
    assert_eq!(receipts.input_state, input);
    assert_eq!(
        receipts.semantic(),
        &[
            dungeon_reset_progress_receipt(
                sprite::DungeonResetSpritesCpuProgress::Load(
                    sprite::DungeonLoadSpritesCpuProgress {
                        normal_load_ordinal: 1,
                        slot: 1,
                        checkpoint: sprite::DungeonSpriteLoadCheckpoint::YHigh,
                    },
                ),
                OriginalTimingBoundary::NmiAccepted,
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ]
    );
    assert!(state
        .take_original_timing_dungeon_reset_sprites_progress()
        .is_some());
    state.ram[0x43] = state.ram[0x43].wrapping_add(1);
}

fn exact_timing_test_rom(opcode: u8) -> Vec<u8> {
    let mut rom = vec![opcode; 0x8000];
    rom[0x7ffc] = 0x00;
    rom[0x7ffd] = 0x80;
    rom
}

fn link_test_byte(state: &ZeldaState, addr: usize) -> u8 {
    state.read_wram_byte(addr)
}

fn set_link_test_byte(state: &mut ZeldaState, addr: usize, value: u8) {
    state.write_wram_byte(addr, value);
}

fn link_test_word(state: &ZeldaState, addr: usize) -> u16 {
    state.read_wram_image_word(addr, 2)
}

fn set_link_test_word(state: &mut ZeldaState, addr: usize, value: u16) {
    state.write_wram_word(addr, value);
}

fn put_test_asset(data: &mut Vec<u8>, ranges: &mut [(usize, usize)], index: usize, bytes: Vec<u8>) {
    let start = data.len();
    data.extend(bytes);
    ranges[index] = (start, data.len());
}

fn pack_test_memblk_arrays(items: &[Vec<u8>]) -> Vec<u8> {
    assert!(!items.is_empty());
    let payload_before_last = items[..items.len() - 1].iter().map(Vec::len).sum::<usize>();
    let wide_offsets = payload_before_last >= 65536;
    let mut data = Vec::new();
    let mut offset = 0usize;
    for item in &items[..items.len() - 1] {
        offset += item.len();
        if wide_offsets {
            data.extend_from_slice(&(offset as u32).to_le_bytes());
        } else {
            data.extend_from_slice(&(offset as u16).to_le_bytes());
        }
    }
    for item in items {
        data.extend_from_slice(item);
    }
    let marker = if wide_offsets {
        8192 + items.len() - 1
    } else {
        items.len() - 1
    };
    data.extend_from_slice(&(marker as u16).to_le_bytes());
    data
}

fn dialogue_source_sidecar_asset(messages: &[Vec<u8>]) -> Vec<u8> {
    let state = ZeldaState::new();
    let table = messages
        .iter()
        .map(|message| state.dialogue_ir_for_decoded_bytes(message))
        .collect::<Vec<_>>();
    let mut asset = DIALOGUE_SOURCE_SIDECAR_MAGIC.to_vec();
    asset.extend(bincode::serialize(&table).unwrap());
    asset
}

fn asset_pack_with_named_assets(
    data: Vec<u8>,
    ranges: Vec<(usize, usize)>,
    named_assets: &[(usize, &str)],
) -> AssetPack {
    let mut names = vec![String::new(); ranges.len()];
    for &(index, name) in named_assets {
        names[index] = name.to_string();
    }
    AssetPack::from_named_data_ranges(data, ranges, names)
}

fn test_asset_pack_bytes(named_assets: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut key_signature = Vec::new();
    for &(name, _) in named_assets {
        key_signature.extend_from_slice(name.as_bytes());
        key_signature.push(0);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ASSET_SIGNATURE_PREFIX);
    bytes.extend_from_slice(&[0; 64]);
    bytes.extend_from_slice(&(named_assets.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(key_signature.len() as u32).to_le_bytes());
    for (_, payload) in named_assets {
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    }
    bytes.extend_from_slice(&key_signature);
    for (_, payload) in named_assets {
        while bytes.len() & 3 != 0 {
            bytes.push(0);
        }
        bytes.extend_from_slice(payload);
    }
    bytes
}

fn probe_entrance_asset_pack(entrance_index: usize, room: u16) -> AssetPack {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 56];
    let byte_len = entrance_index + 1;
    let word_len = (entrance_index + 1) * 2;

    let mut rooms = vec![0; word_len];
    write_le_u16(&mut rooms, entrance_index * 2, room);
    put_test_asset(&mut data, &mut ranges, 11, rooms);

    put_test_asset(&mut data, &mut ranges, 12, vec![0; byte_len * 8]);
    for asset in [13, 14, 15, 16, 17, 18, 26] {
        put_test_asset(&mut data, &mut ranges, asset, vec![0; word_len]);
    }
    for asset in [19, 20, 21, 22, 23, 24, 25, 27] {
        put_test_asset(&mut data, &mut ranges, asset, vec![0; byte_len]);
    }
    put_test_asset(&mut data, &mut ranges, 53, Vec::new());
    put_test_asset(&mut data, &mut ranges, 54, vec![0; 116]);
    put_test_asset(&mut data, &mut ranges, 55, Vec::new());

    AssetPack::from_data_ranges(data, ranges)
}

fn probe_overworld_asset_pack(screen: usize) -> AssetPack {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 109];
    put_test_asset(&mut data, &mut ranges, 107, vec![1; screen + 1]);
    put_test_asset(&mut data, &mut ranges, 108, vec![0; screen + 1]);
    AssetPack::from_data_ranges(data, ranges)
}

fn run4782_spotlight_progress_receipt() -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
        crate::SpotlightTableBuildProgressReceipt {
            progress: crate::SpotlightTableBuildProgress {
                completed_iterations: 216,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 23,
                },
            },
            boundary: OriginalTimingBoundary::NmiAccepted,
        },
    )
}

fn run4782_spotlight_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 4,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        run4782_spotlight_progress_receipt(),
    ]
}

fn run4784_terminal_spotlight_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

fn run4784_terminal_spotlight_state() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(176);
    state.set_indoor_flag(0);
    state.set_animated_tile_data_source_address(1);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.schedule_spotlight_iteration_return(SpotlightIteration::closing(
        SpotlightIterationPhase::CloseEntryBeforeTablePublication,
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_784,
            0,
            run4784_terminal_spotlight_semantic(),
        ))
        .unwrap();
    state
}

fn terminal_recurring_spotlight_state(iteration: SpotlightIteration) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(177);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(0);
    state.set_animated_tile_data_source_address(1);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.schedule_dungeon_exit_spotlight_link_oam(iteration);
    let work = GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration };
    assert_eq!(state.game_execution_scheduler.current_work(), Some(work));
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(0),
    );
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_787,
            0,
            run4784_terminal_spotlight_semantic(),
        ))
        .unwrap();
    state
}

fn run4787_terminal_recurring_spotlight_state() -> ZeldaState {
    terminal_recurring_spotlight_state(SpotlightIteration::closing(
        SpotlightIterationPhase::WholeTable,
    ))
}

fn run4786_spotlight_build_link_oam_state() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(177);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(119);
    state.set_spotlight_window_state(0);
    state.set_animated_tile_data_source_address(1);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    assert!(state.begin_dungeon_exit_spotlight_build(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 105,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        }),
        iteration,
    ));
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. })
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_786,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ))
        .unwrap();
    state
}

fn run4788_spotlight_progress_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 4,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            crate::SpotlightTableBuildProgressReceipt {
                progress: crate::SpotlightTableBuildProgress {
                    completed_iterations: 224,
                    checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                        pending_circle_input: 15,
                    },
                },
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        ),
    ]
}

fn run4789_terminal_build_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

fn run4790_spotlight_progress_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 0,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            crate::SpotlightTableBuildProgressReceipt {
                progress: crate::SpotlightTableBuildProgress {
                    completed_iterations: 229,
                    checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                        pending_circle_input: 10,
                    },
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        ),
    ]
}

fn run4791_terminal_build_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

/// Execute run4788's exact fresh-iteration checkpoint and install run4789's
/// carried-Held terminal return without consuming it.
fn run4789_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = run4787_terminal_recurring_spotlight_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 112);

    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    let receipts = OriginalTimingHostReceipts::new(4_788, 0, run4788_spotlight_progress_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4788 source receipts"),
    )
    .expect("deserialize run4788 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4788_spotlight_progress_semantic(),
        "run4788 wire order must retain Held acceptance before its spotlight claim",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("run4788 lost its suspended recurring spotlight Build")
    };
    assert_eq!(
        table_build,
        SpotlightTableBuildContinuation {
            vertical_center: 238,
            upper_cursor: 224,
            lower_cursor: 252,
            completed: false,
            pending_circle_input: Some(15),
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: false,
            projection_words_copied: 0,
            source_progress: Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 224,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 15,
                },
            }),
        },
    );
    assert!(!projection_completed);
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1),
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);
    assert!(state.original_timing_semantic_receipts.is_none());

    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_789,
            0,
            run4789_terminal_build_semantic(),
        ))
        .unwrap();
    state
}

/// Execute run4789's terminal Build, then run4790's exact HostReturn table
/// checkpoint and install run4791's same-host Held/trailing-Open return.
fn run4791_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = run4789_terminal_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert!(!state.original_timing_nmi_publication_pending);

    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    let receipts = OriginalTimingHostReceipts::new(4_790, 0, run4790_spotlight_progress_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4790 source receipts"),
    )
    .expect("deserialize run4790 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4790_spotlight_progress_semantic(),
        "run4790 wire order must retain the HostReturn spotlight checkpoint",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("run4790 lost its suspended recurring spotlight Build")
    };
    assert_eq!(
        table_build,
        SpotlightTableBuildContinuation {
            vertical_center: 238,
            upper_cursor: 229,
            lower_cursor: 247,
            completed: false,
            pending_circle_input: Some(10),
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: false,
            projection_words_copied: 0,
            source_progress: Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 229,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 10,
                },
            }),
        },
    );
    assert!(!projection_completed);
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.deferred_display_snapshot.is_none());
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);

    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    let receipts = OriginalTimingHostReceipts::new(4_791, 0, run4791_terminal_build_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4791 source receipts"),
    )
    .expect("deserialize run4791 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4791_terminal_build_semantic(),
        "run4791 wire order must keep the caller-return token after its trailing Open acceptance",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    state
}

/// One spotlight-close fresh-iteration host from the run4792..run4818 window:
/// an optional leading Open acceptance, the shared handler/joypad/iteration
/// prefix, then the host-specific tail (a table checkpoint, a LinkOam
/// interruption, or nothing when the module body finished inside the host).
fn run48xx_spotlight_close_fresh_semantic(
    leading_open_acceptance: bool,
    tail: &[OriginalTimingSemanticReceipt],
) -> Vec<OriginalTimingSemanticReceipt> {
    let mut semantic = Vec::new();
    if leading_open_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        ));
    }
    semantic.extend([
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 4,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
    ]);
    semantic.extend_from_slice(tail);
    semantic
}

fn run48xx_spotlight_projection_claim(
    copied_words: u16,
    boundary: OriginalTimingBoundary,
) -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
        crate::SpotlightTableBuildProgressReceipt {
            progress: crate::SpotlightTableBuildProgress {
                completed_iterations: 239,
                checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words },
            },
            boundary,
        },
    )
}

/// One module-qualified spotlight-close terminal return from the
/// run4793..run4817 window: a carried or same-host Held handler, the shared
/// continued-return/suffix pair, an optional trailing Open acceptance, and
/// the wire-last caller-return token.
fn run48xx_spotlight_terminal_semantic(
    same_host_held: bool,
    trailing_open: bool,
) -> Vec<OriginalTimingSemanticReceipt> {
    let mut semantic = Vec::new();
    if same_host_held {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.extend([
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
    ]);
    if trailing_open {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait);
    semantic
}

/// run4799: the leading vblank re-checkpoints the saved ProjectionCopy at its
/// acceptance boundary, then the resumed copy completes and returns.
fn run4799_recheckpointed_terminal_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        run48xx_spotlight_projection_claim(117, OriginalTimingBoundary::NmiAccepted),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

/// run4819: the final suffix-only continuation returns after its module body
/// already advanced the frame out of Module0F, so the adapter proves the
/// return without the module-qualified caller-return token.
fn run4819_tokenless_final_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
    ]
}

fn install_run48xx_host(
    state: &mut ZeldaState,
    host_call: u64,
    semantic: Vec<OriginalTimingSemanticReceipt>,
) {
    let receipts = OriginalTimingHostReceipts::new(host_call, 0, semantic.clone());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run48xx source receipts"),
    )
    .expect("deserialize run48xx source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        semantic,
        "run{host_call} wire order must survive serialization unchanged",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
}

fn assert_run48xx_scheduled_flagged_projection_build(
    state: &ZeldaState,
    copied_words: u16,
    label: &str,
) {
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("{label} lost its suspended ProjectionCopy spotlight Build")
    };
    assert_eq!(
        table_build,
        SpotlightTableBuildContinuation {
            vertical_center: 238,
            upper_cursor: 238,
            lower_cursor: 238,
            completed: true,
            pending_circle_input: None,
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: true,
            projection_words_copied: copied_words,
            source_progress: Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 239,
                checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words },
            }),
        },
        "{label}",
    );
    assert!(!projection_completed, "{label}");
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .with_main_loop_sprite_preparation_before_second_nmi(),
        "{label}: an interrupted ProjectionCopy predicts its same-host suffix on the saved call",
    );
}

/// The live trace holds the follower and BG2 scroll fixed through the whole
/// close window (the exit walk is already over), so every fresh host enters
/// its module body from the same retained pair. Re-pin them where the
/// synthesized lineage would otherwise let the module body drift the iris
/// vertical center away from the trace.
fn pin_live_spotlight_follower(state: &mut ZeldaState) {
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_v_copy2(8466);
}

/// A synthesized fresh-host entry into the run-67132 calibration lineage's
/// ProjectionCopy window, mirroring `terminal_recurring_spotlight_state`'s
/// idiom: the frame, follower, scroll, and radius values retained from the
/// live trace (the shared follower/scroll pair keeps the iris vertical
/// center at 238 for every remaining close iteration), with the previous
/// terminal's completed common suffix and released latch.
fn spotlight_close_projection_window_state(radius: u16, frame_counter: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(frame_counter);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(radius);
    state.set_spotlight_window_state(0);
    state.set_animated_tile_data_source_address(1);
    state.main_loop_sprite_preparation_completed = true;
    state
}

/// Enter the ProjectionCopy window at run4794 and prove its
/// mid-ProjectionCopy interrupt schedules the sprite-preparation estimate,
/// then install run4795's carried-Held terminal return without consuming it.
fn run4795_flagged_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = spotlight_close_projection_window_state(91, 180);

    install_run48xx_host(
        &mut state,
        4_794,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                run48xx_spotlight_projection_claim(2, OriginalTimingBoundary::NmiAccepted),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 2, "run4794");
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);

    install_run48xx_host(
        &mut state,
        4_795,
        run48xx_spotlight_terminal_semantic(false, false),
    );
    state
}

/// Execute run4795's flagged terminal Build, then run4796's second
/// mid-ProjectionCopy interrupt, and install run4797's flagged terminal with
/// its trailing Open acceptance.
fn run4797_flagged_trailing_open_spotlight_build_state() -> ZeldaState {
    let mut state = run4795_flagged_terminal_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 84);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_796,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                run48xx_spotlight_projection_claim(60, OriginalTimingBoundary::NmiAccepted),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 60, "run4796");

    install_run48xx_host(
        &mut state,
        4_797,
        run48xx_spotlight_terminal_semantic(false, true),
    );
    state
}

/// Execute run4797, then run4798's HostReturn ProjectionCopy checkpoint (the
/// host budget expires inside the memcpy with no vblank), and install
/// run4799's re-checkpointed terminal return.
fn run4799_recheckpointed_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = run4797_flagged_trailing_open_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 77);
    assert!(state.original_timing_nmi_publication_pending);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_798,
        run48xx_spotlight_close_fresh_semantic(
            false,
            &[run48xx_spotlight_projection_claim(
                117,
                OriginalTimingBoundary::HostReturn,
            )],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 117, "run4798");
    assert!(
        !state.original_timing_nmi_publication_pending,
        "run4798 ends inside the ProjectionCopy without accepting a vblank",
    );

    install_run48xx_host(
        &mut state,
        4_799,
        run4799_recheckpointed_terminal_semantic(),
    );
    state
}

/// Execute run4799..run4804 (the final table upload and the first LinkOam
/// interruptions of the tail phase), and install run4805's carried-Held
/// suffix-only terminal return.
fn run4805_carried_suffix_only_spotlight_state() -> ZeldaState {
    let mut state = run4799_recheckpointed_terminal_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 70);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_800,
        run48xx_spotlight_close_fresh_semantic(
            false,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                run48xx_spotlight_projection_claim(174, OriginalTimingBoundary::NmiAccepted),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 174, "run4800");

    install_run48xx_host(
        &mut state,
        4_801,
        run48xx_spotlight_terminal_semantic(false, false),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 63);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_802,
        run48xx_spotlight_close_fresh_semantic(true, &[]),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));

    install_run48xx_host(
        &mut state,
        4_803,
        run48xx_spotlight_terminal_semantic(true, false),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_804,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    let Some(GameWorkContinuation::FinishSpotlightIteration { iteration }) =
        state.game_execution_scheduler.current_work()
    else {
        panic!("run4804 lost its suspended LinkOam spotlight suffix")
    };
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn),
    );
    assert!(
        state.original_timing_nmi_publication_pending,
        "run4804's LinkOam interruption carries its Held handler into run4805",
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    install_run48xx_host(
        &mut state,
        4_805,
        run48xx_spotlight_terminal_semantic(false, false),
    );
    state
}

/// Execute run4805, then run4806's budget-expiry LinkOam interruption, and
/// install run4807's same-host suffix-only terminal with its trailing Open.
fn run4807_trailing_open_suffix_only_spotlight_state() -> ZeldaState {
    let mut state = run4805_carried_suffix_only_spotlight_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 49);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_806,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            )],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert!(
        !state.original_timing_nmi_publication_pending,
        "run4806 expires inside LinkOam without accepting a vblank",
    );

    install_run48xx_host(
        &mut state,
        4_807,
        run48xx_spotlight_terminal_semantic(true, true),
    );
    state
}

/// Execute run4807..run4818 (the remaining LinkOam tail iterations, whose
/// final module body advances the frame out of Module0F), and install
/// run4819's tokenless final suffix-only return.
fn run4819_tokenless_final_spotlight_state() -> ZeldaState {
    let mut state = run4807_trailing_open_suffix_only_spotlight_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 42);

    for (host_call, fresh, terminal) in [
        (4_808u64, (false, true), (false, false)),
        (4_810, (true, true), (false, false)),
        (4_812, (true, false), (true, true)),
        (4_814, (false, true), (false, true)),
        (4_816, (false, false), (true, true)),
    ] {
        let (leading_open, latch_before_interruption) = fresh;
        let mut tail = Vec::new();
        if latch_before_interruption {
            tail.push(OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld,
            ));
        }
        tail.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        ));
        pin_live_spotlight_follower(&mut state);
        install_run48xx_host(
            &mut state,
            host_call,
            run48xx_spotlight_close_fresh_semantic(leading_open, &tail),
        );
        state.run_frame_internal(0, crate::RUN_MAIN);
        assert!(
            matches!(
                state.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishSpotlightIteration { .. })
            ),
            "run{host_call} lost its suspended LinkOam spotlight suffix",
        );

        let (same_host_held, trailing_open) = terminal;
        install_run48xx_host(
            &mut state,
            host_call + 1,
            run48xx_spotlight_terminal_semantic(same_host_held, trailing_open),
        );
        state.run_frame_internal(0, crate::RUN_MAIN);
        assert!(
            state.game_execution_scheduler.is_idle(),
            "run{} left successor scheduled work",
            host_call + 1,
        );
    }
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 7);
    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.game_state.frame.submodule, 1);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_818,
        run48xx_spotlight_close_fresh_semantic(
            false,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0,
        "run4818's module body writes the final zero radius",
    );
    assert_ne!(
        (
            state.game_state.frame.main_module,
            state.game_state.frame.submodule,
        ),
        (0x0f, 1),
        "run4818's module body advances the frame out of Module0F before its caller returns",
    );
    assert!(state.original_timing_nmi_publication_pending);

    install_run48xx_host(&mut state, 4_819, run4819_tokenless_final_semantic());
    state
}

fn live_dungeon_spotlight_caller_before_terminal_return(remaining_slices: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    // Keep this non-ROM lifecycle fixture outside the Module-7 CPU timing
    // states. The continuation and typed return receipt own the generic
    // handler/caller/suffix/carry ordering tested below; pinned-ROM coverage
    // separately owns instruction timing for submodule $0f itself.
    state.set_submodule(14);
    state.set_subsubmodule(1);
    state.set_frame_counter(125);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.dungeon_landing_entry_started_after_leading_nmi = true;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        remaining_slices,
    );
    state
}

fn dungeon_spotlight_terminal_return_receipts(host_call: u64) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host_call,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    )
}

fn dungeon_spotlight_terminal_return_without_trailing_nmi_receipts(
    host_call: u64,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host_call,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    )
}

fn dungeon_spotlight_carried_terminal_return_receipts(
    host_call: u64,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host_call,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    )
}

fn carry_dungeon_spotlight_nmi_from_acceptance_host(
    state: &mut ZeldaState,
    host_call: u64,
    gate: NmiUpdateGate,
) {
    match gate {
        NmiUpdateGate::Open => state.clear_nmi_update_latch(),
        NmiUpdateGate::LatchHeld => state.latch_nmi_update(),
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            host_call,
            0,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(gate)],
        ))
        .unwrap();
    let owns_dispatch = state.begin_original_timing_host_dispatch(0);
    assert!(owns_dispatch);
    let phases = state.take_original_timing_nmi_phases();
    assert_eq!(phases, [OriginalTimingNmiPhase::Accepted(gate)]);
    let classification = classify_original_timing_nmi_phases_with_ownership(false, &phases);
    assert_eq!(
        classification.handler_completion,
        OriginalTimingNmiHandlerCompletionOwner::None,
    );
    assert!(classification.publication_pending_at_exit);
    state.capture_and_carry_original_timing_nmi_publication_at_host_return();
    state.finish_original_timing_host_dispatch(owns_dispatch);
    assert!(state.original_timing_semantic_receipts.is_none());
}

fn live_dialogue_terminal_return_state(endpoint: u16) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_bg2_h_copy2(0x1010);
    state.set_bg2_v_copy2(0x2020);
    state.set_bg1_h_copy2(0x3030);
    state.set_bg1_v_copy2(0x4040);
    state.set_bg1_x_offset(3);
    state.set_bg1_y_offset(5);
    state
        .messaging_state_mut()
        .set_dialogue_msg_read_pos(endpoint);
    // The initializer's source caller has already selected the active
    // RenderText state by the time its final character-buffer suffix resumes.
    state.messaging_state_mut().set_module(1);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.ppu.vram[0x5a20] = 0x1357;
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    state.ppu.vram[0x5a20] = 0x2468;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.set_pending_nmi_subroutine(0x77);
    state.set_core_update_disable_flag(4);
    state.set_ambient_sound_effect(3);
    state.follower_link_state_mut().set_joypad1h_last(0xa5);
    state
        .game_execution_scheduler
        .schedule_after_current_trailing_nmi(
            GameWorkContinuation::FinishDialogueInitializationCallerReturn,
        );
    state
}

fn dialogue_terminal_return_receipts(host: u64) -> OriginalTimingHostReceipts {
    dialogue_terminal_return_receipts_for_nmi_cell(host, true, false)
}

fn dialogue_terminal_return_receipts_for_nmi_cell(
    host: u64,
    publication_pending_at_entry: bool,
    trailing_open_acceptance: bool,
) -> OriginalTimingHostReceipts {
    // `$0e:c58b` is outside the VWF endpoint range. The terminal host therefore
    // publishes one Held completion, the continued call stack, and the exact
    // ZeldaRunGameLoop common suffix. Raster position decides whether the Held
    // acceptance belongs to the preceding host or this host, and whether an
    // Open acceptance follows the suffix.
    let mut semantic = Vec::new();
    if !publication_pending_at_entry {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.extend([
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
    ]);
    if trailing_open_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        ));
    }
    OriginalTimingHostReceipts::new(host, 0, semantic)
}

fn ordinary_dialogue_iteration_receipts(host: u64) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0x40,
                low: 0x02,
                high_filtered: 0x40,
                low_filtered: 0x02,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    )
}

fn live_idle_dialogue_main_loop_state() -> ZeldaState {
    let mut state = live_dialogue_terminal_return_state(0x37);
    state.game_execution_scheduler = GameExecutionScheduler::default();
    state.pending_main_loop_common_suffix = None;
    state.main_loop_sprite_preparation_completed = false;
    state.original_timing_nmi_publication_pending = false;
    state.original_timing_pending_nmi_update_gate = None;
    state.original_timing_expected_nmi_update_gates.clear();
    state.original_timing_semantic_receipts = None;
    state.original_timing_last_oracle_host_call = None;
    state.set_pending_nmi_subroutine(0);
    state.set_core_update_disable_flag(0);
    state.clear_nmi_update_latch();
    state
}

fn live_idle_terminal_dialogue_endpoint_state(endpoint: u16) -> ZeldaState {
    let mut state = live_idle_dialogue_main_loop_state();
    state.messaging_state_mut().set_text_render_state(3);
    state
        .messaging_state_mut()
        .set_dialogue_msg_read_pos(endpoint);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.dialogue_fast_forward_hold_pending = false;
    state
}

fn idle_terminal_dialogue_endpoint_receipts(
    host: u64,
    endpoint: u16,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: endpoint,
                },
            ),
        ],
    )
}

fn idle_terminal_dialogue_endpoint_with_trailing_open_receipts(
    host: u64,
    endpoint: u16,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: endpoint,
                },
            ),
        ],
    )
}

fn idle_terminal_suspended_vwf_receipts(host: u64) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    )
}

fn live_idle_terminal_suspended_vwf_state() -> ZeldaState {
    let mut state = live_idle_terminal_dialogue_endpoint_state(25);
    state.messaging_text_mut().load_decoded_dialogue(&[0; 27]);
    state.messaging_state_mut().set_vwf_line_speed_cur(3);
    state.dialogue_fast_forward_hold_active = true;
    state.capture_display_snapshot();
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state
        .install_original_timing_host_receipts(idle_terminal_suspended_vwf_receipts(3_823))
        .unwrap();
    state
}

fn intro_memory_darken_suspended_state(
    pending_at_entry: bool,
    trailing_acceptance: bool,
    host_call: u64,
) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(26);
    state.intro_memory_darken_frame_delay = 1;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    if pending_at_entry {
        state.ppu.vram[0] = host_call as u16;
        state.capture_display_snapshot();
        state.original_timing_nmi_publication_pending = true;
        state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    }
    let mut semantic = Vec::new();
    if !pending_at_entry {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    if trailing_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
        crate::MainLoopProgress::CallStackContinued,
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            host_call, 0, semantic,
        ))
        .unwrap();
    state
}

fn injected_dungeon_cpu_schedule(
    submodule_nmis: u8,
    caller_nmis: u8,
) -> DungeonSubmoduleCpuSchedule {
    DungeonSubmoduleCpuSchedule {
        reset_progress: None,
        submodule_nmis,
        caller_nmis,
        caller_sprite_main_nmis: 0,
        caller_suffix_nmis: caller_nmis,
        caller_first_nmi_phase: None,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
        reenters_main_loop_before_nmi: false,
    }
}

fn spiral_cpu_test_state(subsubmodule: u8, schedule: DungeonSubmoduleCpuSchedule) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(subsubmodule);
    state.dungeon_submodule_cpu_schedule = Some(schedule);
    state
}

fn live_terminal_ground_item_receipt_state_with_gfx(gfx: u8) -> ZeldaState {
    const ANIMATED_SOURCE: usize = 0xa680;
    const ANIMATED_DESTINATION: usize = 0x3b00;

    let item = match gfx {
        0x06 => 0x00,
        0x14 => 0x12,
        0x21 => 0x33,
        0x22 => 0x32,
        _ => panic!("terminal ground-item test needs a source-coherent item for gfx {gfx:02x}"),
    };
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.set_frame_counter(0xfa);
    state.set_animated_tile_data_source_address(ANIMATED_SOURCE as u16);
    state.set_animated_tile_vram_destination_address(ANIMATED_DESTINATION as u16);
    state.ram[ANIMATED_SOURCE..ANIMATED_SOURCE + 0x400].fill(0x5a);
    state.ppu.vram[ANIMATED_DESTINATION] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[ANIMATED_DESTINATION] = 0;
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx,
                ground_apress_tail: Some(ItemReceiptReturn {
                    ancilla_slot: 4,
                    item,
                    chest_position: 0x0182,
                }),
            },
        },
        1,
    );
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_586,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();
    state
}

fn live_terminal_ground_item_receipt_state() -> ZeldaState {
    live_terminal_ground_item_receipt_state_with_gfx(0x14)
}

fn live_file_select_graphics_state_before_slice(completed_waiting_slices: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(1);
    state.set_submodule(1);
    state.set_subsubmodule(249);
    state
        .game_execution_scheduler
        .schedule_file_select_graphics();
    for _ in 0..completed_waiting_slices {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::FileSelectWaiting),
        );
    }
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
}

fn install_zero_file_select_test_assets(state: &mut ZeldaState) {
    // Sheets below 103 are stored raw by the Rust asset pack. The remaining
    // file-select sheets use two LC-LZ2 fill commands for exactly 0x600 zero
    // bytes, followed by the source terminator.
    let compressed_zero_sheet = vec![0xe7, 0xff, 0, 0xe5, 0xff, 0, 0xff];
    let sprite_packs = (0..=0x6b)
        .map(|gfx| {
            if gfx < 103 {
                vec![0; 0x600]
            } else {
                compressed_zero_sheet.clone()
            }
        })
        .collect::<Vec<_>>();
    let mut asset_data = Vec::new();
    let mut asset_ranges = vec![(0, 0); 65];
    put_test_asset(&mut asset_data, &mut asset_ranges, 56, vec![0; 0x800]);
    put_test_asset(
        &mut asset_data,
        &mut asset_ranges,
        64,
        pack_test_memblk_arrays(&sprite_packs),
    );
    state.assets = Some(AssetPack::from_data_ranges(asset_data, asset_ranges));
}

fn live_file_select_waiting_state(
    pending_at_entry: bool,
    trailing_acceptance: bool,
    completed_waiting_slices: u8,
    host_call: u64,
) -> ZeldaState {
    let mut state = live_file_select_graphics_state_before_slice(completed_waiting_slices);
    if pending_at_entry {
        state.ppu.vram[0] = host_call as u16;
        state.capture_display_snapshot();
        state.original_timing_nmi_publication_pending = true;
        state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    }
    let mut semantic = Vec::new();
    if !pending_at_entry {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    if trailing_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
        crate::MainLoopProgress::CallStackContinued,
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            host_call, 0, semantic,
        ))
        .unwrap();
    state
}

fn live_selected_game_load_state_before_terminal_hosts() -> ZeldaState {
    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::parse(&assets).unwrap());
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(5);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    for _ in 0..SELECTED_GAME_LOAD_NMI_SLICES - 2 {
        assert!(state
            .game_execution_scheduler
            .advance_startup_sequence()
            .is_some());
    }
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        2,
    );
    state.enable_force_blank();
    state.begin_selected_game_load_pre_dungeon_audio(SelectedGameLoadDestination::Dungeon);
    state.complete_pending_selected_game_load_entry_room_load();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state
}

fn live_selected_game_load_state_before_pre_dungeon_audio() -> ZeldaState {
    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::parse(&assets).unwrap());
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(5);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting),
        );
    }
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
}

fn mark_selected_game_load_sprite_disable_all_completed(state: &mut ZeldaState) {
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_selected_game_load_from_source(
                Some(crate::SpriteResetAllProgress::SpriteDisableAllCompleted),
                false,
            ),
        Some(StartupSequenceStep::SelectedGameLoadWaiting),
    );
}

fn early_dialogue_completion_state(pending_subroutine: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(5);
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0);
    }
    state.clear_nmi_update_latch();
    state.set_pending_nmi_subroutine(pending_subroutine);
    state.set_core_update_disable_flag(2);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state
        .install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                1,
                0,
                vec![
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                        high: 0,
                        low: 0,
                        high_filtered: 0,
                        low_filtered: 0,
                    }),
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                    OriginalTimingSemanticReceipt::SpriteMainReturned,
                ],
            )
            .with_dialogue_scroll_progress(vec![
                crate::DialogueScrollProgressReceipt {
                    entered: false,
                    completed_pixel_passes: 3,
                    returned: true,
                },
            ]),
        )
        .unwrap();
    state
}

fn publish_staged_dialogue_text_dma(state: &mut ZeldaState) {
    let staged = state
        .dialogue_scroll_completion_staged
        .clone()
        .expect("test dialogue publication requires a staged text generation");
    if !state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts)
    {
        state.capture_display_snapshot();
    }
    for (offset, &word) in staged.vram.iter().enumerate() {
        write_le_u16(&mut state.ram, 0x10000 + offset * 2, word);
    }
    state.bg3_vwf_glyph_runs = staged.glyph_runs.clone();
    state.bg3_vwf_glyph_run_dialogue_offsets = staged.glyph_run_dialogue_offsets.clone();
    state.bg3_vwf_glyph_run_dialogue_message_id = staged.dialogue_message_id;
    state
        .messaging_state_mut()
        .set_dialogue_msg_read_pos(staged.dialogue_msg_read_pos);
    state.clear_nmi_update_latch();
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    assert!(
        state
            .interrupt_nmi_for_active_scanout(0, None, false)
            .is_none(),
        "a staged BG3 text DMA must consume its linear evidence at NMI completion",
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
}

fn return_only_dialogue_state_before_run2508() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(3);
    state.messaging_state_mut().set_dialogue_scroll_speed(4);
    state
        .messaging_text_mut()
        .load_decoded_dialogue(&[TEXT_COMMAND_START_US + 12]);

    state.ppu.vram[0x7c00..0x7ff0].fill(0x1111);
    state.capture_display_snapshot();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x3000 | index as u16);
    }
    state.bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: 0,
        width: 3,
    }];
    state.bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.bg3_vwf_glyph_run_dialogue_message_id = 32;
    state.messaging_state_mut().set_dialogue_msg_read_pos(0);

    // Run 2508 enters in the Held handler accepted by run 2507. The outgoing
    // acceptance-host image is still receptive because that handler has not
    // yet returned.
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.latch_nmi_update();
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    state.frame_ctr_dbg = 2508;
    state
}

fn run2508_dialogue_receipts() -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        2508,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    )
}

fn run2509_dialogue_receipts() -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        2509,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        ],
    )
    .with_dialogue_scroll_progress(vec![crate::DialogueScrollProgressReceipt {
        entered: true,
        completed_pixel_passes: 2,
        returned: false,
    }])
}

fn staged_dialogue_carry_state(gate: NmiUpdateGate) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(14);
    state.set_submodule(2);
    state.capture_display_snapshot();
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x4444);
    }
    state.bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 0x42,
        origin_tile_number: 0x181,
        x: 7,
        y: 0,
        width: 4,
    }];
    state.bg3_vwf_glyph_run_dialogue_offsets = vec![0x31];
    state.bg3_vwf_glyph_run_dialogue_message_id = 32;
    state.messaging_state_mut().set_dialogue_msg_read_pos(0x31);
    let staged = state.dialogue_text_scanout_from_render_buffer();
    state.stage_dialogue_scroll_completion_after_return(staged);
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(gate);
    if gate == NmiUpdateGate::LatchHeld {
        state.latch_nmi_update();
    } else {
        state.clear_nmi_update_latch();
    }
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    let mut semantic = vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted];
    if gate == NmiUpdateGate::Open {
        semantic.push(OriginalTimingSemanticReceipt::JoypadPublication(
            JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            },
        ));
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(1, 0, semantic))
        .unwrap();
    assert_eq!(
        state.take_original_timing_nmi_phases(),
        [OriginalTimingNmiPhase::HandlerCompleted],
    );
    state.begin_original_timing_host_dispatch(0);
    state
}

// Topical test files split out of this hub (see the declarations' paths).
#[path = "zelda_rtl_tests/runtime/runtime_audio.rs"]
mod runtime_audio;
#[path = "zelda_rtl_tests/runtime/runtime_dialogue.rs"]
mod runtime_dialogue;
#[path = "zelda_rtl_tests/runtime/runtime_display.rs"]
mod runtime_display;
#[path = "zelda_rtl_tests/runtime/runtime_dungeon.rs"]
mod runtime_dungeon;
#[path = "zelda_rtl_tests/runtime/runtime_dungeon_roles.rs"]
mod runtime_dungeon_roles;
#[path = "zelda_rtl_tests/runtime/runtime_entity_tiles.rs"]
mod runtime_entity_tiles;
#[path = "zelda_rtl_tests/runtime/runtime_entrance_backup.rs"]
mod runtime_entrance_backup;
#[path = "zelda_rtl_tests/runtime/runtime_native_overworld_dungeon_exit.rs"]
mod runtime_native_overworld_dungeon_exit;
#[path = "zelda_rtl_tests/runtime/runtime_native_properties_graphics.rs"]
mod runtime_native_properties_graphics;
#[path = "zelda_rtl_tests/runtime/runtime_native_screen_palettes.rs"]
mod runtime_native_screen_palettes;
#[path = "zelda_rtl_tests/runtime/runtime_native_area_palettes.rs"]
mod runtime_native_area_palettes;
#[path = "zelda_rtl_tests/runtime/runtime_native_sprite_reload.rs"]
mod runtime_native_sprite_reload;
#[path = "zelda_rtl_tests/runtime/runtime_native_properties_player.rs"]
mod runtime_native_properties_player;
#[path = "zelda_rtl_tests/runtime/runtime_native_properties_caller.rs"]
mod runtime_native_properties_caller;
#[path = "zelda_rtl_tests/runtime/runtime_native_map32_definition.rs"]
mod runtime_native_map32_definition;
#[path = "zelda_rtl_tests/runtime/runtime_native_mirror_portal.rs"]
mod runtime_native_mirror_portal;
#[path = "zelda_rtl_tests/runtime/runtime_boss_prize_countdown.rs"]
mod runtime_boss_prize_countdown;
#[path = "zelda_rtl_tests/runtime/runtime_water_hdma_window.rs"]
mod runtime_water_hdma_window;
#[path = "zelda_rtl_tests/runtime/runtime_door_step_owner.rs"]
mod runtime_door_step_owner;
#[path = "zelda_rtl_tests/runtime/runtime_blast_wall_owner.rs"]
mod runtime_blast_wall_owner;
#[path = "zelda_rtl_tests/runtime/runtime_spawn_record_carry.rs"]
mod runtime_spawn_record_carry;
#[path = "zelda_rtl_tests/runtime/runtime_boss_prize_retry.rs"]
mod runtime_boss_prize_retry;
#[path = "zelda_rtl_tests/runtime/runtime_items.rs"]
mod runtime_items;
#[path = "zelda_rtl_tests/runtime/runtime_misc.rs"]
mod runtime_misc;
#[path = "zelda_rtl_tests/runtime/runtime_object_records.rs"]
mod runtime_object_records;
#[path = "zelda_rtl_tests/runtime/runtime_overworld.rs"]
mod runtime_overworld;
#[path = "zelda_rtl_tests/runtime/runtime_palette_backup.rs"]
mod runtime_palette_backup;
#[path = "zelda_rtl_tests/runtime/runtime_player_collision.rs"]
mod runtime_player_collision;
#[path = "zelda_rtl_tests/runtime/runtime_player_transitions.rs"]
mod runtime_player_transitions;
#[path = "zelda_rtl_tests/runtime/runtime_scroll_bounds.rs"]
mod runtime_scroll_bounds;
#[path = "zelda_rtl_tests/runtime/runtime_spotlight.rs"]
mod runtime_spotlight;
#[path = "zelda_rtl_tests/runtime/runtime_sprites.rs"]
mod runtime_sprites;
#[path = "zelda_rtl_tests/runtime/runtime_startup.rs"]
mod runtime_startup;
#[path = "zelda_rtl_tests/runtime/runtime_tile_behavior.rs"]
mod runtime_tile_behavior;
#[path = "zelda_rtl_tests/runtime/runtime_tile_definitions.rs"]
mod runtime_tile_definitions;
#[path = "zelda_rtl_tests/runtime/runtime_timing.rs"]
mod runtime_timing;
#[path = "zelda_rtl_tests/runtime/runtime_world_transient_owners.rs"]
mod runtime_world_transient_owners;

    #[test]
    fn encoded_asset_streams_reproduce_source_message_bytes() {
        let Some(path) = std::env::var_os("ZELDA3_ASSETS") else {
            return;
        };
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        let mut state = crate::ZeldaState::new();
        state.set_assets(&std::fs::read(path).unwrap()).unwrap();
        let pack = state.asset_memblk(94, 0).unwrap();
        let dialogue = find_index_in_memblk(pack, 1);
        let messages: Vec<_> = (0..397)
            .map(|i| find_index_in_memblk(dialogue, i).ptr)
            .collect();
        let streams = crate::cycle_models::text_pointers::message_streams(&messages).unwrap();
        for (stream, start) in streams.iter().zip([0xe0000, 0x75f40]) {
            assert_eq!(stream, &rom[start..start + stream.len()]);
        }
    }

#[test]
fn native_intro_through_poly_worker_irq_matches_full_cold_source_hosts() {
    let Some(path)=std::env::var_os("ZELDA3_ASSETS") else {return};
    let Ok(rom)=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")) else {return};
    let mut state=ZeldaState::new();state.set_rom_startup_timing(true);
    state.set_assets(&std::fs::read(path).unwrap()).unwrap();
    let mut source=snes::Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom,Some(&state.sram)).unwrap();
    let mut scan_hosts=0;
    for host in 0..228 {
        source.set_joypad_serial_state(0,0);source.run_until_main_loop_return().unwrap();
        state.frame_ctr_dbg=host+1;state.run_frame_internal(0,1);
        let samples=source.take_dsp_samples().samples.len();
        state.zelda_render_audio(&mut vec![0;samples*2],samples as i32,2);
        if scan_hosts==0 && !matches!(state.game_state.frame.subsubmodule,9..=11) {
            continue;
        }
        for index in 0..398 {
            let at=0x171c0+index*3;
            let pointer=u32::from(state.ram[at]) | (u32::from(state.ram[at+1])<<8) | (u32::from(state.ram[at+2])<<16);
            assert_eq!(state.game_state.messaging.dialogue_pointers.pointer(index),pointer,
                "typed pointer publication host={host} index={index}");
        }
        let cpu=state.native_startup_cpu.as_ref().unwrap();
        if !cpu.startup.owns_cpu_clock() {
            panic!("intro caller must retain its clock through the common suffix host={host}");
        }
        assert_eq!(cpu.timeline,*source.machine().timeline(),"host={host}");
        state.native_ppu.as_mut().unwrap().assert_matches_source(&cpu.timeline, source.machine());
        let differences:Vec<_>=state.wram_image().iter().zip(&source.machine().snes().ram).enumerate()
            .filter(|(_, (a,b))|a!=b).map(|(i,(a,b))|(i,*a,*b)).collect();
        assert!(differences.is_empty(),"host={host} RAM count={} first={:?}",differences.len(),&differences[..differences.len().min(8)]);
        assert_eq!(state.ppu.vram,source.machine().snes().ppu.vram,"VRAM host={host}");
        assert_eq!(state.ppu.cgram,source.machine().snes().ppu.cgram,"CGRAM host={host}");
        assert_eq!(state.ppu.oam,source.machine().snes().ppu.oam,"OAM host={host}");
        assert_eq!(state.intro_initialization_work_frames_pending,0);
        scan_hosts+=1;
        if state.game_state.frame.submodule==3 {
            assert!(scan_hosts>100);
            let mut scheduled=state.clone();
            // Continue the native handler from its own retained hardware
            // acceptance. No source registers or WRAM are copied into it.
            // Continue through rendering and the actual IRQ stack exchange.
            let mut cpu=state.native_startup_cpu.take().unwrap();
            let mut context=cpu.startup.wait_handler_context();
            let mut source_programming=crate::cycle_models::poly_interrupt::PolyIrqProgramming::default();
            for count in 0..50000 {
                let c=&source.machine().snes().cpu;
                if (c.k,c.pc)==(9,0xf81d) {break;}
                assert!(count<49999,"source did not reach poly thread entry");
                for access in source.step().unwrap().accesses {
                    if let snes::SourceCpuBusAccessKind::Write {value,width:1}=access.kind {
                        match access.address {
                            0x4209=>source_programming.vertical_timer=(source_programming.vertical_timer&0x100)|value,
                            0x420a=>source_programming.vertical_timer=(source_programming.vertical_timer&255)|((value&1)<<8),
                            0x4200=>{source_programming.nmitimen=value as u8;source_programming.enabled_at=access.timestamp.master_cycles();}
                            _=>{},
                        }
                    }
                }
            }
            let programming=state.interrupt_accepted_poly_nmi_with_native_context(0,&mut context,&mut cpu.timeline);
            assert_eq!((programming.vertical_timer,programming.nmitimen,programming.enabled_at),
                (source_programming.vertical_timer,source_programming.nmitimen,source_programming.enabled_at));
            assert_eq!((programming.vertical_timer,programming.nmitimen),(0x90,0xa1));
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let differences:Vec<_>=state.wram_image().iter().zip(&source.machine().snes().ram).enumerate()
                .filter(|(_, (a,b))|a!=b).map(|(i,(a,b))|(i,*a,*b)).collect();
            assert!(differences.is_empty(),"poly NMI RAM count={} first={:?}",differences.len(),&differences[..differences.len().min(8)]);
            assert_eq!(state.ppu.vram,source.machine().snes().ppu.vram);
            assert_eq!(state.ppu.cgram,source.machine().snes().ppu.cgram);
            assert_eq!(state.ppu.oam,source.machine().snes().ppu.oam);
            assert_eq!(state.game_state.display.nmi_thread_stack_pointer,0x1f2);
            assert_eq!(state.ram[0x1f00],1,"initializer owns the first render request");
            while (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)!=(9,0xf829) {source.step().unwrap();}
            state.run_native_poly_buffer_clear(&mut context,&mut cpu.timeline);
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*state.wram_image(),source.machine().snes().ram);
            assert!(state.ram[0xe800..0xf000].iter().all(|&b|b==0));
            for count in 0..100 {
                if (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)==(9,0xf82c) {break;}
                assert!(count<99,"source did not return from poly shape setup");
                source.step().unwrap();
            }
            state.run_native_poly_shape(&mut context,&mut cpu.timeline);
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*state.wram_image(),source.machine().snes().ram);
            assert_eq!(state.game_state.poly,crate::game_state::PolyState::load_from_ram(&state.wram_image()));
            for count in 0..100 {
                if (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)==(9,0xf82f) {break;}
                assert!(count<99,"source did not return from poly rotation setup");
                source.step().unwrap();
            }
            state.run_native_poly_rotation(&mut context,&mut cpu.timeline);
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*state.wram_image(),source.machine().snes().ram);
            assert_eq!(state.game_state.poly,crate::game_state::PolyState::load_from_ram(&state.wram_image()));
            assert_eq!(state.ppu.m7_matrix,source.machine().snes().ppu.m7_matrix);
            assert_eq!(state.ppu.m7_prev,source.machine().snes().ppu.m7_prev);
            for count in 0..5000 {
                if (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)==(9,0xf832){break;}
                assert!(count<4999,"source did not return from vertex projection");
                source.step().unwrap();
            }
            let divider=state.run_native_poly_vertices(&mut context,&mut cpu.timeline);
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*state.wram_image(),source.machine().snes().ram);
            assert_eq!(state.game_state.poly,crate::game_state::PolyState::load_from_ram(&state.wram_image()));
            assert_eq!(state.ppu.m7_matrix,source.machine().snes().ppu.m7_matrix);
            assert_eq!(state.ppu.m7_prev,source.machine().snes().ppu.m7_prev);
            assert_eq!((divider.dividend,divider.quotient,divider.remainder),
                (source.machine().snes().divide_a,source.machine().snes().divide_result,source.machine().snes().multiply_result));
            for count in 0..5000 {
                if (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)==(9,0xfd1e){break;}
                assert!(count<4999,"source did not call the first visible face rasterizer");
                source.step().unwrap();
            }
            let faces=state.run_native_poly_faces_to_raster(&mut context,&mut cpu.timeline);
            assert!(faces.raster_pending());
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*state.wram_image(),source.machine().snes().ram);
            assert_eq!(state.game_state.poly,crate::game_state::PolyState::load_from_ram(&state.wram_image()));
            assert_eq!(state.ppu.m7_matrix,source.machine().snes().ppu.m7_matrix);
            assert_eq!(state.ppu.m7_prev,source.machine().snes().ppu.m7_prev);
            assert!(cpu.timeline.clock_master_cycles()<programming.irq_deadline.unwrap());
            let mut worker=crate::cycle_models::poly_worker::PolyWorker::from_pending_faces(faces);
            for count in 0..100000 {
                let irq_return=(source.machine().snes().cpu.k,source.machine().snes().cpu.pc)==(0,0x8332);
                assert!(count<99999,"source did not return from the polygon V-IRQ");
                source.step().unwrap();
                if irq_return {break;}
            }
            state.run_native_poly_worker_to_irq(&mut worker,&mut context,&mut cpu.timeline,programming);
            assert_eq!(cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(context,crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*state.wram_image(),source.machine().snes().ram);
            assert_eq!(state.ppu.m7_matrix,source.machine().snes().ppu.m7_matrix);
            assert_eq!(state.ppu.m7_prev,source.machine().snes().ppu.m7_prev);
            assert_eq!((state.native_cpu_math.dividend,state.native_cpu_math.quotient,state.native_cpu_math.remainder),
                (source.machine().snes().divide_a,source.machine().snes().divide_result,source.machine().snes().multiply_result));
            // The same chain is now reachable through the default scheduler.
            // Compare that entry independently from the component calls above.
            for count in 0..10 {
                if (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)==(0,0x8038){break;}
                assert!(count<9,"source did not leave the released main wait");
                source.step().unwrap();
            }
            scheduled.frame_ctr_dbg+=1;
            assert_eq!(scheduled.advance_native_startup_host(0),Some(true));
            let scheduled_cpu=scheduled.native_startup_cpu.as_ref().unwrap();
            assert_eq!(scheduled_cpu.timeline,*source.machine().timeline());
            let c=&source.machine().snes().cpu;
            assert_eq!(scheduled_cpu.startup.main_caller_context().unwrap(),crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
            assert_eq!(*scheduled.wram_image(),source.machine().snes().ram);
            assert_eq!(scheduled_cpu.poly_worker.as_ref(),Some(&worker));
            return;
        }
    }
    panic!("native intro initializer did not return");
}

#[test]
fn native_idle_poly_irq_matches_hardware_acceptance_and_main_restore() {
    use crate::cycle_models::{nmi_stack::NativeNmiContext,poly_interrupt::PolyIrqProgramming};
    let mut rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
    // Fixture-only setup programs the real V timer before parking. The
    // original IRQ vector/prologue/return remain unchanged.
    let code=[0x18,0xfb,0xa9,0x80,0x8d,0,0x21,0xa9,0x90,0x8d,9,0x42,
        0x9c,10,0x42,0xa9,0xa1,0x8d,0,0x42,0x80,0xfe];
    rom[..code.len()].copy_from_slice(&code);
    for status in [0x30,0x31] {
        let mut source=snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        while source.machine().timestamp().master_cycles()<100*1364+510 {source.step().unwrap();}
        let mut state=ZeldaState::new();
        for (i,b) in state.ram.iter_mut().enumerate(){*b=(i^(i>>7)) as u8;}
    { let image = state.ram.clone(); state.import_wram_image(&image); }
        state.ram[0x12a]=1;state.ram[0x1f00]=0;state.ram[0x1f0c]=0;
        state.ram[0x1f0a..0x1f0c].copy_from_slice(&0x1f2u16.to_le_bytes());
        state.ram[0x1f3..0x200].copy_from_slice(&[0,0,0,0x78,0x56,0x34,0x12,0xef,0xcd,0,0x34,0x80,0]);
        state.sync_native_game_state_from_ram();
        let mut context=NativeNmiContext {a:0xab00,x:0x45,y:0x67,d:0x1f00,db:9,bank:9,pc:0xf81d,sp:0x1f3e,p:status};
        let mut registers=source.machine().snes().cpu.clone();
        registers.a=context.a;registers.x=context.x;registers.y=context.y;registers.dp=context.d;
        registers.db=context.db;registers.k=context.bank;registers.pc=context.pc;registers.sp=context.sp;
        registers.e=false;registers.unpack_flags(context.p);
        let sram=source.machine().snes().cart.ram.clone();
        source.rebase_private_timing_state_at(source.machine().timestamp(),&registers,&state.wram_image(),&sram).unwrap();
        let mut clock=source.machine().timeline().clone();
        let programming=PolyIrqProgramming {vertical_timer:0x90,nmitimen:0xa1,enabled_at:0,
            irq_deadline:snes::cpu_timeline::vertical_irq_deadline(&clock,0x90,false).map(|at|at.master_cycles())};
        while (source.machine().snes().cpu.k,source.machine().snes().cpu.pc)!=(0,0x8034) {source.step().unwrap();}
        state.run_native_poly_idle_to_irq(&mut context,&mut clock,programming);
        let c=&source.machine().snes().cpu;
        assert_eq!(context,NativeNmiContext {a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
        assert_eq!(clock,*source.machine().timeline());
        assert_eq!(*state.wram_image(),source.machine().snes().ram);
        assert_eq!(state.game_state.display.nmi_thread_stack_pointer,0x1f31);
    }
}

#[test]
fn native_intro_animation_matches_independent_cold_source_hosts() {
    compare_native_intro_animation_cold_hosts(false,5);
}

#[test]
fn public_frame_dispatch_matches_independent_cold_intro_animation_hosts() {
    compare_native_intro_animation_cold_hosts(true,5);
}

#[test]
fn title_fade_and_sword_match_independent_cold_source_hosts() {
    compare_native_intro_animation_cold_hosts(true,7);
}

#[test]
fn background_fade_matches_independent_cold_source_hosts() {
    compare_native_intro_animation_cold_hosts(true,8);
}

#[test]
fn title_start_exits_match_independent_cold_source_hosts() {
    for submodule in [7,8] {
        compare_native_intro_input_cold_hosts(true,(1,0),Some(submodule));
    }
}

#[test]
fn title_wait_timeout_matches_independent_cold_source_hosts() {
    compare_native_intro_input_cold_hosts(true,(20,0),None);
}

#[test]
fn file_select_initialization_matches_independent_cold_source_hosts() {
    for submodule in [7,8] {
        compare_native_intro_input_cold_hosts(true,(1,1),Some(submodule));
    }
}

#[test]
fn file_select_erase_and_palettes_match_independent_cold_source_hosts() {
    for submodule in [7,8] {
        compare_native_intro_input_cold_hosts(true,(1,2),Some(submodule));
    }
}

#[test]
fn file_select_stripes_match_independent_cold_source_hosts() {
    for submodule in [7,8] {
        compare_native_intro_input_cold_hosts(true,(1,5),Some(submodule));
    }
}

#[test]
fn file_select_active_menu_matches_independent_cold_source_hosts() {
    for submodule in [7,8] {
        compare_native_intro_input_cold_hosts_after(true,(1,5),Some(submodule),16);
    }
}

#[test]
fn file_select_valid_saves_and_confirmation_match_independent_cold_source_hosts() {
    compare_native_file_menu_cold_hosts(true,(1,5),Some(7),16,true,&[]);
    for slot in 0..3 {
        let mut input=vec![0;2];
        for _ in 0..slot {input.extend([0x20,0]);}
        input.push(8); // Start, the same controller bit used to leave the title.
        compare_native_file_menu_cold_hosts(true,(5,0),Some(7),0,true,&input);
    }
}

#[test]
fn file_select_menu_choices_match_independent_cold_source_hosts() {
    compare_native_file_menu_cold_hosts(true,(4,0),Some(7),0,false,&[0,0,8]);
    compare_native_file_menu_cold_hosts(true,(2,0),Some(7),0,true,&[0,0,0x10,0,0x10,0,8]);
    compare_native_file_menu_cold_hosts(true,(3,0),Some(7),0,true,&[0,0,0x10,0,8]);
    compare_native_file_menu_cold_hosts(true,(1,5),Some(7),16,false,&[0,0,0x10,0,8,0,0x10,0,8,0,0x20,0,0x20]);
}

#[test]
fn kill_file_setup_retains_clock_through_shared_erase_and_stripes() {
    for stop in [(3,1),(3,2),(3,3)] {
        compare_native_file_menu_cold_hosts(true,stop,Some(7),0,true,&[0,0,0x10,0,8]);
    }
}

#[test]
fn kill_file_selection_retains_clock_through_live_slot_drawing() {
    compare_native_file_menu_cold_hosts(true,(3,3),Some(7),16,true,&[0,0,0x10,0,8]);
}

#[test]
fn kill_file_confirmation_and_cancel_match_independent_cold_source_hosts() {
    let menu=[0,0,0x10,0,8];
    for slot in 0..3 {
        let mut input=vec![0,0];
        for _ in 0..slot { input.extend([0x20,0]); }
        input.extend([8,0,0,8,0]);
        compare_native_menu_input_cold_hosts(true,(1,1),Some(7),0,true,&menu,&[],&input,Some(slot));
    }
    // Cancel from the selection screen, and from the confirmation screen.
    for input in [&[0,0,0x10,0,8,0][..], &[0,0,8,0,0x20,0,8,0][..]] {
        compare_native_menu_input_cold_hosts(true,(1,1),Some(7),0,true,&menu,&[],input,None);
    }
}

#[test]
fn name_file_initialization_matches_independent_cold_source_hosts() {
    compare_native_file_menu_cold_hosts(true,(4,1),Some(7),0,false,&[0,0,8]);
}

#[test]
fn load_file_entry_matches_independent_cold_source_hosts() {
    // LoadFile returns to the dungeon's entrance-fade state, after publishing
    // its destination at $02:8383/$02:8387. Module 5 is its entry, not its return.
    compare_native_file_menu_cold_hosts(true,(7,15),Some(7),0,true,&[0,0,8]);
}

#[test]
fn dungeon_landing_matches_independent_cold_source_hosts() {
    compare_native_file_menu_cold_hosts(true,(7,15),Some(7),3,true,&[0,0,8]);
}

#[test]
fn dungeon_landing_into_dialogue_matches_independent_cold_source_hosts() {
    compare_native_file_menu_cold_hosts(true,(14,2),Some(7),0,true,&[0,0,8]);
}

#[test]
fn load_file_player_publications_match_source_after_each_instruction() {
    use crate::cycle_models::{intro_clear::Access, load_file_state::LoadFileState, nmi_stack::NativeNmiContext};
    let mut rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
    rom[..9].copy_from_slice(&[0x18,0xfb,0xa9,0x80,0x8d,0,0x21,0x80,0xfe]);
    let mut instructions=0;
    for seed in [0u8,0x55,0xaa,0xff] {
        let mut source=snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        while source.machine().timestamp().master_cycles()<1364 {source.step().unwrap();}
        let mut state=ZeldaState::new();
        for (i,b) in state.ram.iter_mut().enumerate(){*b=(i as u8).wrapping_mul(7)^seed;}
    { let image = state.ram.clone(); state.import_wram_image(&image); }
        state.ram[0x1fb..0x1fe].copy_from_slice(&[0x59,0x80,0]);
        state.sync_native_game_state_from_ram();
        let mut cpu=NativeNmiContext {a:0xab00|u16::from(seed),x:0x71,y:0x83,d:0,db:2,bank:7,pc:0xf13c,sp:0x1fa,p:0x31};
        let mut registers=source.machine().snes().cpu.clone();
        registers.a=cpu.a;registers.x=cpu.x;registers.y=cpu.y;registers.dp=cpu.d;registers.db=cpu.db;
        registers.k=cpu.bank;registers.pc=cpu.pc;registers.sp=cpu.sp;registers.e=false;registers.unpack_flags(cpu.p);
        let sram=source.machine().snes().cart.ram.clone();
        source.rebase_private_timing_state_at(source.machine().timestamp(),&registers,&state.wram_image(),&sram).unwrap();
        let mut clock=source.machine().timeline().clone();
        let mut work=LoadFileState::new(0x07f13c);
        while !work.complete() {
            let pc=cpu.pc;
            assert!(source.step().unwrap().accepted_interrupt.is_none());
            work.step(&mut cpu,&mut clock,|_,access|match access {
                Access::Read(address,width)=>{
                    let address=(address&65535)as usize;
                    assert!(address<0x2000);
                    if width==1 {u16::from(state.ram[address])}else{read_le_u16(&state.wram_image(),address)}
                }
                Access::Write(address,value,width)=>{
                    let address=(address&65535)as usize;
                    if (0x100..0x200).contains(&address) {
                        state.ram[address..address+usize::from(width)].copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                    } else {
                        assert!(state.native_load_player_initialization_store(address,value,width),"unowned player store {address:04x}");
                    }
                    0
                }
                Access::Vblank{..}=>0,
            }).unwrap();
            let c=&source.machine().snes().cpu;
            assert_eq!(cpu,NativeNmiContext{a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()},"CPU after {pc:04x}");
            assert_eq!(clock,*source.machine().timeline(),"clock after {pc:04x}");
            assert_eq!(*state.wram_image(),source.machine().snes().ram,"publication after {pc:04x}");
            let projected=crate::game_state::PlayerState::load_from_ram(&state.wram_image());
            assert_eq!(state.game_state.player.follower_link,projected.follower_link,"native player after {pc:04x}");
            assert_eq!(state.game_state.player.swim_acceleration,projected.swim_acceleration,"native swim after {pc:04x}");
            instructions+=1;
        }
    }
    eprintln!("{instructions} Link initialization instructions preserved partial native player publications");
}

#[test]
fn name_file_keyboard_setup_matches_independent_cold_source_hosts() {
    for title in [7,8] {for slot in 0..3 {
        let mut input=vec![0,0];
        for _ in 0..slot {input.extend([0x20,0]);}
        input.push(8);
        compare_native_file_menu_cold_hosts(true,(4,3),Some(title),0,false,&input);
    }}
}

#[test]
fn name_file_active_matches_independent_cold_source_hosts() {
    for title in [7,8] {
        let mut input=vec![0;4];
        // Reject an empty name, type seven characters (slot wrap), move the
        // keyboard in both directions, then scroll down and back up.
        input.extend([8,0]);
        for button in [1,2,0x100,0x200,1,2,0x100] {input.extend([button,0]);}
        input.extend([0x80;70]);input.extend([0;8]);
        input.extend([0x40;70]);input.extend([0;8]);
        input.extend([0x20;30]);input.extend([0;8]);
        input.extend([0x10;30]);input.extend([0;8]);
        compare_native_name_input_cold_hosts(true,(4,3),Some(title),input.len()+8,false,&[0,0,8],&input);
    }
}

#[test]
fn name_file_confirmation_matches_independent_cold_source_hosts() {
    for slot in 0..3 {
        let mut menu=vec![0,0];
        for _ in 0..slot {menu.extend([0x20,0]);}menu.push(8);
        compare_native_name_input_cold_hosts(true,(1,1),Some(7),0,false,&menu,&[0,0,[1,0x100,0x200][slot],0,0,8,0]);
    }
}

#[test]
fn file_select_retained_data_tables_match_source_bytes() {
    let rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
    for range in [0x0cce0f..0x0cce1b,0x0ce1c8..0x0ce2a8,0x0ce359..0x0ce456,
        0x0ccd78..0x0ccd7d,0x0cd416..0x0cd485,0x0cd597..0x0cd599,0x0ce53f..0x0ce63c,
        0x0cd630..0x0cd63c,0x0cd698..0x0cd6af,0x0cd7a3..0x0cd7a5,0x0cd7cb..0x0cd7db,
        0x0cd935..0x0cda4d,0x0cdd24..0x0cdd30,0x0083f8..0x0083fa,0x04f4c6..0x04f502] {
        for a in range {assert_eq!(select_file::native_file_select_data_byte(a),Some(rom[(a as usize>>16)*0x8000+(a as usize&0x7fff)]),"file table {a:06x}");}
    }
}

fn compare_native_intro_animation_cold_hosts(public_dispatch: bool,stop_submodule:u8) {
    compare_native_intro_input_cold_hosts(public_dispatch,(0,stop_submodule),None);
}

fn compare_native_intro_input_cold_hosts(public_dispatch: bool,stop:(u8,u8),press_start_in:Option<u8>) {
    compare_native_intro_input_cold_hosts_after(public_dispatch,stop,press_start_in,0);
}

fn compare_native_intro_input_cold_hosts_after(public_dispatch: bool,stop:(u8,u8),press_start_in:Option<u8>,active_hosts:usize) {
    compare_native_file_menu_cold_hosts(public_dispatch,stop,press_start_in,active_hosts,false,&[]);
}

fn compare_native_file_menu_cold_hosts(public_dispatch: bool,stop:(u8,u8),press_start_in:Option<u8>,active_hosts:usize,valid_saves:bool,menu_inputs:&[u16]) {
    compare_native_name_input_cold_hosts(public_dispatch,stop,press_start_in,active_hosts,valid_saves,menu_inputs,&[]);
}

fn compare_native_name_input_cold_hosts(public_dispatch: bool,stop:(u8,u8),press_start_in:Option<u8>,active_hosts:usize,valid_saves:bool,menu_inputs:&[u16],name_inputs:&[u16]) {
    compare_native_menu_input_cold_hosts(public_dispatch,stop,press_start_in,active_hosts,valid_saves,menu_inputs,name_inputs,&[],None);
}

fn compare_native_menu_input_cold_hosts(public_dispatch: bool,stop:(u8,u8),press_start_in:Option<u8>,mut active_hosts:usize,valid_saves:bool,menu_inputs:&[u16],name_inputs:&[u16],kill_inputs:&[u16],deleted_slot:Option<usize>) {
    let stop_submodule=if stop.0==0 {stop.1}else{9};
    let load_file_destination = matches!(stop.0, 7 | 14);
    let mut load_file_save_copy_checked = false;
    let mut load_file_destination_writes = Vec::new();
    let path = std::env::var_os("ZELDA3_ASSETS").expect("cold intro test requires embedded assets");
    let rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    )
    .unwrap();
    for address in 0x0cc000..0x0cd000 {
        if let Some(value) = super::ending::native_intro_sprite_table_byte(address) {
            assert_eq!(
                value,
                rom[0x60000 + (address as usize & 0x7fff)],
                "sprite table {address:06x}"
            );
        }
    }
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_assets(&std::fs::read(path).unwrap()).unwrap();
    if valid_saves {
        // Shared initial cartridge data, supplied before either machine boots.
        // Runtime registers, RAM and continuations remain independently cold.
        for slot in 0..3 {
            let s=slot*0x500;
            state.sram[s..s+0x500].fill(0);
            state.sram[s+0x359]=(slot+1)as u8;
            state.sram[s+0x35a]=slot as u8;
            state.sram[s+0x35b]=slot as u8;
            state.sram[s+0x36c]=(3+slot*8)as u8*8;
            state.sram[s+0x3e5..s+0x3e7].copy_from_slice(&0x55aau16.to_le_bytes());
            state.sram[s+0x405..s+0x407].copy_from_slice(&[0u16,42,999][slot].to_le_bytes());
            for i in 0..6 {state.sram[s+0x3d9+i*2..s+0x3db+i*2].copy_from_slice(&((i+slot*6)as u16).to_le_bytes());}
            let sum=state.sram[s..s+0x4fe].chunks_exact(2).fold(0u16,|sum,b|sum.wrapping_add(u16::from_le_bytes([b[0],b[1]])));
            state.sram[s+0x4fe..s+0x500].copy_from_slice(&0x5a5au16.wrapping_sub(sum).to_le_bytes());
        }
    }
    if public_dispatch {
        // An empty input replay has no host-authored recorder patches (such
        // as the live music-status mirror at $0648). Both machines receive
        // only controller input, so their complete CPU-owned RAM is comparable.
        state.state_recorder.replay_mode = true;
        state.state_recorder.total_frames = 4000;
    }
    let mut source =
        snes::Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&state.sram)).unwrap();
    let mut source_window_events = source.machine().timeline().synchronous_event_observer();
    let mut source_window_rows = [[1, 0, 1, 0]; 224];
    // The port relocates its display table out of the ROM's physical WRAM
    // table. Derive that projection independently from source bus stores;
    // the production state never consumes this expected-value buffer.
    let mut source_spotlight_projection = [0u8; 448];
    let mut compared = 0;
    let mut last_source_irq = None;
    let mut menu_host=0;
    let mut naming_irqs=0;
    let mut naming_host=0;
    let mut kill_host=0;
    let mut naming_irq_finish=[false;3]; // final request, acknowledgement, IRQ disable
    for host in 0..4000 {
        let input=if press_start_in.is_some_and(|submodule|state.game_state.frame.main_module==0 && state.game_state.frame.submodule==submodule
            && state.ram[0xc007]==0 && (submodule!=8 || (1..255).contains(&state.ram[0xb0]))) {8}
            else if (state.game_state.frame.main_module,state.game_state.frame.submodule)==(1,5) && !state.native_intro_clear_pending() {
                let input=menu_inputs.get(menu_host).copied().unwrap_or(0);menu_host+=1;input
            }else if (state.game_state.frame.main_module,state.game_state.frame.submodule)==(4,3) {
                let input=name_inputs.get(naming_host).copied().unwrap_or(0);naming_host+=1;input
            }else if state.game_state.frame.main_module==3 && matches!(state.game_state.frame.submodule,3|4)
                && !state.native_intro_clear_pending() {
                let input=kill_inputs.get(kill_host).copied().unwrap_or(0);kill_host+=1;input
            }else{0};
        source.set_joypad_serial_state(input, 0);
        let mut interrupt_events=Vec::new();
        let mut source_scroll_ppu = source.machine().snes().ppu.clone();
        let mut source_scroll_writes = Vec::new();
        source
            .run_until_main_loop_return_with_quiescent_step_hook(|receipt, source| {
                // Independently sample the source device after the instruction
                // that drains RenderLine. Window writes after that event would
                // make this sampler ambiguous, so reject that case explicitly.
                let clock = source.machine().timeline();
                let ppu = &source.machine().snes().ppu;
                clock.observe_processed_synchronous_events(&mut source_window_events, |event| {
                    if let snes::cpu_timeline::CpuSynchronousTimelineEvent::RenderLine {
                        scanline: scanline @ 1..=224, ..
                    } = event {
                        let at = clock.master_cycles_at_raster(clock.field_index(),
                            snes::CpuRasterPosition::new(scanline, snes::cpu_timeline::SNES9X_RENDER_CYCLE as u16));
                        assert!(!receipt.accesses.iter().any(|a| matches!(a.kind, snes::SourceCpuBusAccessKind::Write { .. })
                            && a.address & 0x400000 == 0 && (0x2126..=0x2129).contains(&(a.address & 0xffff))
                            && a.timestamp.master_cycles() >= at), "source window sample crosses a later CPU write");
                        source_window_rows[usize::from(scanline - 1)] =
                            [ppu.window1_left, ppu.window1_right, ppu.window2_left, ppu.window2_right];
                    }
                });
                if receipt.accepted_interrupt.is_some() {interrupt_events.push(format!("{:?} at {:?} pc={:06x}",receipt.accepted_interrupt,receipt.started_at,receipt.origin_pc));}
                for access in &receipt.accesses {
                    if matches!(receipt.origin_pc, 0x00f3bb | 0x00f42f | 0x00f432 | 0x00f435 | 0x00f438 | 0x00f43b | 0x00f43e | 0x00f441) {
                        if let snes::SourceCpuBusAccessKind::Write { value, width } = access.kind {
                            let offset = (access.address & 0xffff) as usize;
                            if (0x1b00..0x1cc0).contains(&offset) {
                                let start = offset - 0x1b00;
                                source_spotlight_projection[start..start + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            }
                        }
                    }
                    if let snes::SourceCpuBusAccessKind::Write{value,..}=access.kind {
                        if access.address & 0x400000 == 0
                            && (0x210d..=0x2114).contains(&(access.address & 0xffff)) {
                            source_scroll_writes.push((access.timestamp, access.address as u8, value as u8));
                        }
                        if matches!(receipt.origin_pc, 0x028383 | 0x028387) {
                            load_file_destination_writes.push((receipt.origin_pc, access.address & 0xffff, value));
                        }
                        match (access.address&0xffff,value) {
                            (0x128,255) if receipt.origin_pc==0x0cdc81=>naming_irq_finish[0]=true,
                            (0x128,0) if naming_irq_finish[0]=>naming_irq_finish[1]=true,
                            (0x4200,0x81) if naming_irq_finish[0]=>naming_irq_finish[2]=true,
                            _=>{}
                        }
                    }
                    if matches!(access.kind,snes::SourceCpuBusAccessKind::Write{..}) && matches!(access.address&0xffff,0x4200|0x12a) {
                        interrupt_events.push(format!("pc={:06x} {access:?}",receipt.origin_pc));
                    }
                }
                if let Some(irq @ snes::SourceCpuAcceptedInterrupt::Irq { .. }) =
                    receipt.accepted_interrupt
                {
                    last_source_irq = Some((host, irq, receipt.origin_pc, receipt.started_at));
                    if state.game_state.frame.main_module==4 && state.ram[0x128]==1 {naming_irqs+=1;}
                }
                Ok::<(), snes::SourceCpuError>(())
            })
            .unwrap();
        if public_dispatch {
            state.zelda_run_frame_with_replay_input_override(i32::from(input), Some(input));
        } else {
            state.frame_ctr_dbg = host + 1;
            state.run_frame_internal(input, 1);
        }
        let samples = source.take_dsp_samples().samples.len();
        state.zelda_render_audio(&mut vec![0; samples * 2], samples as i32, 2);
        if compared == 0 && !matches!(state.game_state.frame.submodule, 3 | 4) {
            continue;
        }
        let cpu = state.native_startup_cpu.as_ref().unwrap();
        assert!(
            cpu.startup.owns_cpu_clock(),
            "lost native animation ownership host={host}"
        );
        assert!(matches!(state.paired_resume_cpu_boundary_blocker(),
            Some("retained-native-apu-port-access" | "retained-native-bootstrap-clock")),
            "checkpoint would discard the retained CPU at host={host}");
        state.native_ppu.as_mut().unwrap().assert_matches_source(&cpu.timeline, source.machine());
        let math = &state.native_cpu_math;
        let source_math = source.machine().snes();
        assert_eq!((math.multiplicand, math.dividend, math.quotient, math.remainder),
            (source_math.multiply_a, source_math.divide_a, source_math.divide_result, source_math.multiply_result),
            "CPU arithmetic hardware host={host}");
        let c = &source.machine().snes().cpu;
        if cpu.timeline != *source.machine().timeline() {
            eprintln!("source interrupt events host={host}: {interrupt_events:?}");
            let differences: Vec<_> = state
                .wram_image()
                .iter()
                .zip(&source.machine().snes().ram)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, (a, b))| (i, *a, *b))
                .take(24)
                .collect();
            eprintln!("clock mismatch host={host} source_irq={last_source_irq:?} native={:?} source={:?} RAM={differences:x?}",cpu.startup.retained_context(),crate::cycle_models::nmi_stack::NativeNmiContext {a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()});
        }
        assert_eq!(
            cpu.timeline,
            *source.machine().timeline(),
            "clock host={host}"
        );
        assert_eq!(
            cpu.startup.retained_context(),
            Some(crate::cycle_models::nmi_stack::NativeNmiContext {
                a: c.a,
                x: c.x,
                y: c.y,
                d: c.dp,
                db: c.db,
                bank: c.k,
                pc: c.pc,
                sp: c.sp,
                p: c.pack_flags()
            }),
            "CPU host={host}"
        );
        let differences: Vec<_> = state
            .wram_image()
            .iter()
            .zip(&source.machine().snes().ram)
            .enumerate()
            .filter_map(|(i, (&actual, &physical))| {
                let base = crate::game_state::constants::HDMA_TABLE_DYNAMIC;
                let expected = if (base..base + source_spotlight_projection.len()).contains(&i) {
                    source_spotlight_projection[i - base]
                } else { physical };
                (actual != expected).then_some((i, actual, expected))
            })
            .collect();
        assert!(
            differences.is_empty(),
            "host={host} source_irq={last_source_irq:?} RAM count={} first={:?}",
            differences.len(),
            &differences[..differences.len().min(12)]
        );
        assert_eq!(state.sram,source.machine().snes().cart.ram,"SRAM host={host}");
        if matches!(state.game_state.frame.main_module,1|3|4) {
            assert_eq!(state.game_state.sprites.sprite_slots,SpriteSlotsState::load_from_ram(&state.wram_image()),"typed sprite slots host={host}");
            assert_eq!(state.game_state.messaging.select_file_menu,crate::game_state::SelectFileMenuState::load_from_ram(&state.wram_image()),"typed file menu host={host}");
        }
        assert_eq!(
            state.ppu.vram,
            source.machine().snes().ppu.vram,
            "VRAM host={host}"
        );
        assert_eq!(
            state.ppu.cgram,
            source.machine().snes().ppu.cgram,
            "CGRAM host={host}"
        );
        assert_eq!(
            state.ppu.oam,
            source.machine().snes().ppu.oam,
            "OAM host={host}"
        );
        for (native, source) in state.ppu.bg_layer.iter().zip(&source.machine().snes().ppu.bg_layer) {
            assert_eq!((native.h_scroll, native.v_scroll), (source.h_scroll, source.v_scroll),
                "BG scroll host={host}");
        }
        assert_eq!((state.ppu.scroll_prev, state.ppu.scroll_prev2),
            (source.machine().snes().ppu.scroll_prev, source.machine().snes().ppu.scroll_prev2),
            "scroll latch host={host}");
        if load_file_destination && matches!(state.game_state.frame.main_module, 7 | 14) {
            let native_rows = state.with_display_snapshot(|state| state.ppu_scanline_windows());
            for (row, expected) in source_window_rows.iter().enumerate() {
                let actual = native_rows[row];
                assert_eq!([actual.0, actual.1, actual.2, actual.3], *expected,
                    "native dungeon window scanout host={host} row={row}");
            }
        }
        if state.native_bg3_scroll_scanout.is_some() {
            // Observe source register writes independently of the native IRQ
            // owner. Sample their retained values at each hardware render event,
            // including the NMI writes before IRQ changes the live end-state.
            let clock = source.machine().timeline();
            let mut writes = source_scroll_writes.iter().copied().peekable();
            let native_rows = state.with_display_snapshot(|state| state.ppu_scanline_windows());
            for row in 0..224 {
                let render_at = clock.master_cycles_at_raster(clock.field_index(),
                    snes::CpuRasterPosition::new(row as u16 + 1, snes::cpu_timeline::SNES9X_RENDER_CYCLE as u16));
                while writes.peek().is_some_and(|(at, _, _)| at.master_cycles() < render_at) {
                    let (_, address, value) = writes.next().unwrap();
                    source_scroll_ppu.write(address, value);
                }
                assert_eq!([native_rows[row].5[2], native_rows[row].6[2]],
                    [source_scroll_ppu.bg_layer[2].h_scroll, source_scroll_ppu.bg_layer[2].v_scroll],
                    "native IRQ scroll scanout host={host} row={row} render_at={render_at} source_writes={source_scroll_writes:?}");
            }
        }
        // Attract/credits coordinates and scratch share player and room bytes
        // (28/29 and CC/CE). Once intro returns, those active owners replace
        // the ending view. CPU, all RAM and PPU checks above remain unconditional.
        if state.game_state.frame.main_module == 0 {
            assert_eq!(
                state.game_state.ending,
                crate::game_state::EndingState::load_from_ram(&state.wram_image()),
                "typed ending host={host}"
            );
        }
        compared += 1;
        if load_file_destination && !load_file_save_copy_checked
            && state.game_state.frame.main_module == 5 {
            let slot = state.selected_save_slot_index();
            assert_eq!(slot, menu_inputs.iter().filter(|&&v| v == 0x20).count());
            assert_eq!(&state.wram_image()[0xf000..0xf500], &state.sram[slot*0x500..slot*0x500+0x500],
                "selected save must be copied before LoadFile mutates live inventory");
            load_file_save_copy_checked = true;
        }
        if state.game_state.frame.main_module==0 && stop_submodule>=7 && state.game_state.frame.submodule>=5 {
            assert_eq!(state.game_state.intro_sword,crate::game_state::IntroSwordState::load_from_ram(&state.wram_image()),"typed sword host={host}");
        }
        if (state.game_state.frame.main_module,state.game_state.frame.submodule)==stop
            && (name_inputs.is_empty() || naming_host>0)
            && (kill_inputs.is_empty() || kill_host>0)
            && !(stop.0!=0 && state.native_intro_clear_pending()) {
            if active_hosts != 0 { active_hosts -= 1; continue; }
            if load_file_destination {
                assert!(load_file_save_copy_checked);
                assert_eq!(load_file_destination_writes, [(0x028383, 0x10, 7), (0x028387, 0x11, 15)],
                    "the source LoadFile caller must publish the requested destination before returning");
            }
            if stop.0==4 && stop.1>=1 {
                assert_eq!(read_le_u16(&state.wram_image(),0x200)as usize,menu_inputs.iter().filter(|&&v|v==0x20).count()*0x500);
                if stop.1>=3 {
                    assert!(naming_irqs>0,"keyboard setup did not exercise its live scanline IRQ");
                    eprintln!("name keyboard setup compared {naming_irqs} IRQ acceptances with the naming request active");
                }
            }
            if valid_saves {
                for slot in 0..3 {
                    if deleted_slot==Some(slot) {
                        assert!(state.sram[slot*0x500..(slot+1)*0x500].iter().all(|&v|v==0));
                        assert!(state.sram[0xf00+slot*0x500..0xf00+(slot+1)*0x500].iter().all(|&v|v==0));
                    } else {
                        assert_eq!(read_le_u16(&state.sram,slot*0x500+0x3e5),0x55aa,"cold fixture must survive source SRAM validation");
                    }
                }
                assert_eq!(state.game_state.inventory.save_progress,crate::game_state::SaveProgressState::load_from_ram(&state.wram_image()));
                if stop.0==5 {
                    let slot=state.selected_save_slot_index();
                    assert_eq!(slot,menu_inputs.iter().filter(|&&v|v==0x20).count());
                    assert_eq!(&state.wram_image()[0xf000..0xf500],&state.sram[slot*0x500..slot*0x500+0x500]);
                }
            }
            if !name_inputs.is_empty() && stop==(1,1) {
                let slot=menu_inputs.iter().filter(|&&v|v==0x20).count();
                assert_eq!(read_le_u16(&state.sram,0x1ffe),2+slot as u16*2);
                assert_eq!(read_le_u16(&state.sram,slot*0x500+0x3e5),0x55aa);
                assert_ne!(read_le_u16(&state.sram,slot*0x500+0x3d9),0xa9);
                let sum=state.sram[slot*0x500..slot*0x500+0x500].chunks_exact(2).fold(0u16,|s,b|s.wrapping_add(u16::from_le_bytes([b[0],b[1]])));
                assert_eq!(sum,0x5a5a);
                // The final host includes the one-shot IRQ after the save
                // store; both independently executed machines have cleared it.
                assert_eq!(naming_irq_finish,[true;3]);assert_eq!(state.ram[0x128],0);
                eprintln!("cold naming saved slot {slot} after {naming_host} naming hosts with {naming_irqs} naming IRQs");
            }
            if !name_inputs.is_empty() && stop==(4,3) {
                for i in 0..6 {assert_eq!(read_le_u16(&state.sram,0x3d9+i*2),6,"cold keyboard input must write each name cell");}
                assert_eq!(state.ram[0xb12],1,"seventh character must wrap the name slot");
            }
            if stop_submodule>=7 {
                eprintln!("{compared} cold animation/title host boundaries through host {host}, reached main={} sub={} Start-in={press_start_in:?}",state.game_state.frame.main_module,state.game_state.frame.submodule);
                if press_start_in.is_none() { return; }
            }
            assert!(compared > 100);
            if stop_submodule<7 {
                eprintln!("{compared} cold animation host boundaries through host {host}, returned to submodule 5");
                assert_eq!(state.ram[0x14], 3);
            }
            // The animation's final live $14 write belongs to the following
            // NMI. Retire that handler, worker and IRQ before handing main to
            // the next module, comparing independently cold state throughout.
            source.set_joypad_serial_state(0,0);
            for instruction in 0..100_000 {
                if (source.machine().snes().cpu.k, source.machine().snes().cpu.pc) == (0, 0x8038) { break; }
                assert!(instruction < 99_999, "source did not release main after stripe upload");
                source.step().unwrap();
            }
            state.frame_ctr_dbg += 1;
            assert_eq!(state.advance_native_startup_host(0), Some(true));
            let cpu = state.native_startup_cpu.as_ref().unwrap();
            let c = &source.machine().snes().cpu;
            assert_eq!(cpu.startup.main_caller_context(), Some(crate::cycle_models::nmi_stack::NativeNmiContext {
                a:c.a,x:c.x,y:c.y,d:c.dp,db:c.db,bank:c.k,pc:c.pc,sp:c.sp,p:c.pack_flags()
            }));
            assert_eq!(cpu.timeline, *source.machine().timeline());
            let mut expected_ram = source.machine().snes().ram.clone();
            let base = crate::game_state::constants::HDMA_TABLE_DYNAMIC;
            expected_ram[base..base + source_spotlight_projection.len()]
                .copy_from_slice(&source_spotlight_projection);
            assert_eq!(state.wram_image().iter().zip(&expected_ram).enumerate()
                .find(|(_, (actual, expected))| actual != expected), None,
                "physical RAM and relocated spotlight publication after following NMI");
            assert_eq!(state.ppu.vram, source.machine().snes().ppu.vram);
            assert_eq!(state.ppu.cgram, source.machine().snes().ppu.cgram);
            assert_eq!(state.ppu.oam, source.machine().snes().ppu.oam);
            assert_eq!(state.ram[0x14], 0);
            eprintln!("following NMI stripe upload and IRQ return match independent cold source Start-in={press_start_in:?}");
            return;
        }
        assert!(
            (state.game_state.frame.main_module==0 && (3..stop_submodule).contains(&state.game_state.frame.submodule))
                || (stop.0!=0 && state.game_state.frame.main_module==1 && state.game_state.frame.submodule<=5)
                || (state.game_state.frame.main_module==stop.0 && stop.0!=0 && state.game_state.frame.submodule<=stop.1)
                || (load_file_destination && state.game_state.frame.main_module==5 && state.game_state.frame.submodule==0)
                || (load_file_destination && state.game_state.frame.main_module==7 && state.game_state.frame.submodule==15)
                || (!name_inputs.is_empty() && state.game_state.frame.main_module==4 && state.game_state.frame.submodule<=3)
                || (!kill_inputs.is_empty() && state.game_state.frame.main_module==3 && state.game_state.frame.submodule<=4),
            "unexpected intro transition at host {host}: native main={} sub={} pending={} source main={} sub={} CPU={:?}",
            state.game_state.frame.main_module, state.game_state.frame.submodule,
            state.native_intro_clear_pending(), source.machine().snes().ram[0x10],
            source.machine().snes().ram[0x11],
            state.native_startup_cpu.as_ref().and_then(|cpu| cpu.startup.retained_context())
        );
    }
    panic!("native animation did not finish; compared={compared}");
}
