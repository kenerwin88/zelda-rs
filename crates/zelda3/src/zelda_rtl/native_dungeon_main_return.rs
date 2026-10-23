//! Publications of the dungeon's real return, drawing and sprite preparation.
use super::*;
use crate::game_state::constants::{BG_TILE_ANIMATION_COUNTDOWN, LINK_RUPEES_ACTUAL};

impl ZeldaState {
    /// ItemBus has already retired the WRAM bytes. Publish only the domains
    /// authored by this caller; scratch and stack bytes remain on that bus.
    pub(super) fn native_dungeon_main_return_store(&mut self, address: u32, value: u16, width: u8) {
        use crate::cycle_models::sprite_dma::{POINTER_WORD_ADDRESSES, SOURCE_WORD_ADDRESSES};
        assert!(matches!(width, 1 | 2));
        if self.native_load_hud_store(address, value, width)
            || self.native_player_drawing_store(address, value, width)
        {
            return;
        }
        let offset = if (0x7e0000..0x800000).contains(&address) {
            (address - 0x7e0000) as usize
        } else {
            address as usize
        };
        if (0xe0..=0xe3).contains(&offset) || (0xe6..=0xe9).contains(&offset) {
            self.native_dungeon_camera_store(offset, value, width);
            return;
        }
        if let Some(index) = SOURCE_WORD_ADDRESSES
            .iter()
            .position(|&a| u32::from(a) == address)
        {
            assert_eq!(width, 2);
            self.display_core_mut()
                .set_sprite_preparation_source_word(index, value);
            return;
        }
        if let Some(index) = POINTER_WORD_ADDRESSES
            .iter()
            .position(|&a| u32::from(a) == address)
        {
            assert_eq!(width, 2);
            self.display_core_mut()
                .set_sprite_preparation_pointer_word(index, value);
            return;
        }
        match address {
            0x12 => {
                assert_eq!((width, value), (1, 0));
                self.clear_nmi_update_latch();
            }
            0x100 => {
                assert_eq!(width, 2);
                self.follower_link_state_mut()
                    .set_link_dma_graphics_index_word(value);
            }
            0x109 => {
                assert!(matches!(width, 1 | 2));
                self.follower_link_state_mut()
                    .set_link_dma_staging_index(value as u8);
                if width == 2 {
                    // Some drawing tails store a word across the staging
                    // byte and the neighboring game-over signal.
                    self.game_state.system_signals.set_game_over_check_flag((value >> 8) as u8);
                }
            }
            0x200 => {
                assert_eq!(width, 1);
                self.set_overworld_map_state(value as u8);
            }
            0x208 => {
                assert_eq!(width, 1);
                self.set_heart_refill_countdown(value as u8);
            }
            0x209 => {
                assert_eq!(width, 1);
                self.set_heart_refill_anim_subpos(value as u8);
            }
            0x20a => {
                assert_eq!(width, 1);
                self.set_is_doing_heart_animation(value as u8);
            }
            0x31f => {
                assert_eq!(width, 1);
                self.follower_link_state_mut()
                    .set_blink_countdown(value as u8);
            }
            0x355 => {
                assert_eq!(width, 1);
                self.follower_link_state_mut()
                    .set_secondary_water_grass_timer(value as u8);
            }
            0x356 => {
                assert_eq!(width, 1);
                self.follower_link_state_mut()
                    .set_primary_water_grass_timer(value as u8);
            }
            0x3fa => {
                assert_eq!(width, 2);
                self.follower_link_state_mut()
                    .set_bit9_of_xcoord_word(value);
            }
            0x4a0 | 0x4a1 => {
                let word = read_le_u16(&self.ram, HUD_FLOOR_CHANGED_TIMER);
                self.hud_mut().set_floor_changed_timer(word);
            }
            0xadc => {
                assert_eq!(width, 2);
                self.set_animated_tile_data_source_address(value);
            }
            0xae0 | 0xae2 => {
                assert_eq!(width, 2);
                self.display_core_mut()
                    .set_sprite_preparation_animation_word(((address - 0xae0) / 2) as usize, value);
            }
            0x7ec00d | 0x7ec00e => {
                self.reset_bg_tile_animation_countdown(read_le_u16(&self.ram, BG_TILE_ANIMATION_COUNTDOWN))
            }
            0x7ec00f => {
                assert_eq!(width, 2);
                self.player_state_mut().set_link_dma_source_offset(value);
            }
            0x7ec013 | 0x7ec014 => {
                let word = read_le_u16(&self.ram, LINK_DMA_COUNTDOWN);
                self.player_state_mut().set_link_dma_countdown(word);
            }
            0x7ec015 => {
                assert_eq!(width, 2);
                self.player_state_mut().set_link_dma_tile_offset(value);
            }
            0x7ef343 => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_bombs(value as u8);
            }
            0x7ef360 | 0x7ef361 => {
                let word = self.read_wram_image_word(LINK_RUPEES_GOAL, 2);
                self.player_resources_mut().set_rupees_goal(word);
            }
            0x7ef362 | 0x7ef363 => {
                let word = self.read_wram_image_word(LINK_RUPEES_ACTUAL, 2);
                self.player_resources_mut().set_rupees_actual(word);
            }
            0x7ef36d => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_current_health(value as u8);
            }
            0x7ef36e => {
                assert_eq!(width, 1);
                self.player_magic_mut().set_magic_power(value as u8);
            }
            0x7ef372 => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_heart_filler(value as u8);
            }
            0x7ef373 => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_magic_filler(value as u8);
            }
            0x7ef375 => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_bomb_filler(value as u8);
            }
            0x7ef376 => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_arrow_filler(value as u8);
            }
            0x7ef377 => {
                assert_eq!(width, 1);
                self.player_resources_mut().set_arrows(value as u8);
            }
            0x12d => {
                assert_eq!(width, 1);
                self.set_ambient_sound_effect(value as u8);
            }
            0x12e | 0x12f => {} // ItemBus owns the live SFX latches.
            _ if offset < 0x10 || (0x1c0..0x200).contains(&offset) => {}
            _ => {
                for (byte, value) in value.to_le_bytes()[..usize::from(width)]
                    .iter()
                    .copied()
                    .enumerate()
                {
                    assert!(
                        self.game_state.oam.write_drawing_byte(offset + byte, value),
                        "unowned native dungeon return byte {:06x}",
                        address + byte as u32
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::native_item_graphics::DungeonReturnBus;
    use super::*;
    use crate::cycle_models::{
        dungeon_main_return::DungeonMainReturn, main_wait::MainLoopCallerPhase,
        nmi_stack::NativeNmiContext,
    };

    #[test]
    fn standard_hud_rebuild_matches_source_template_and_bow_owner() {
        use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for bow in [0, 1, 2, 3, 4] {
            for arrows in [0, 5] {
                for keys in [0, 3, 255] {
                    let mut actual = ZeldaState::new();
                    actual.ram[0xc700..0xc880].fill(0x5a);
                    actual.write_wram_byte(0xf340, bow);
                    actual.write_wram_byte(0xf377, arrows);
                    actual.write_wram_byte(0xf36f, keys);
                    actual.write_wram_byte(0xf36c, 0x50);
                    actual.write_wram_byte(0xf36d, 0x38);
                    actual.sync_native_game_state_from_ram();
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: 0x0dfa70,
                        stop_pc: 0x0d8034,
                        a: 0,
                        x: 0,
                        y: 0,
                        sp: 0x1fd,
                        dp: 0,
                        db: 0x0d,
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
                        stack_address: 0x1fe,
                        stack_bytes: &[0x33, 0x80],
                    };
                    let mut source = RomCpuTimingRun::new(
                        &rom,
                        &actual.wram_image(),
                        &actual.sram,
                        &actual.ppu,
                        &actual.dma,
                        [0; 4],
                        checkpoint,
                    )
                    .unwrap();
                    source.enable_cpu_write_trace();
                    let mut expected = actual.wram_image().into_owned();
                    for _ in 0..20000 {
                        if source.is_complete() {
                            break;
                        }
                        source.step();
                        for (at, value) in source.take_cpu_wram_writes() {
                            expected[at] = value;
                        }
                    }
                    assert!(source.is_complete());
                    actual.hud_rebuild();
                    let differences = actual
                        .wram_image()
                        .iter()
                        .zip(&expected)
                        .enumerate()
                        .filter_map(|(i, (a, b))| {
                            (i >= 0x10 && !(0x1c0..0x200).contains(&i) && a != b)
                                .then_some((i, *a, *b))
                        })
                        .take(16)
                        .collect::<Vec<_>>();
                    assert!(
                        differences.is_empty(),
                        "bow={bow} arrows={arrows} keys={keys}: {differences:x?}"
                    );
                }
            }
        }
    }

    #[test]
    fn live_dungeon_main_return_matches_complete_c_state_without_runtime_rom() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for case in 0..64usize {
            let mut expected = ZeldaState::new();
            expected.ram[0x10] = 7;
            expected.ram[0x1b] = 1;
            expected.ram[0x5fc] = 1;
            expected.ram[0x5fd] = 2;
            expected.ram[0x500] = [0, 1, 2, 4][case % 4];
            expected.ram[0x540..0x542].copy_from_slice(&0x280u16.to_le_bytes());
            expected.ram[0x474] = (case % 4 * 2) as u8;
            expected.ram[0x2f] = (case % 4 * 2) as u8;
            expected.ram[0x11] = if case & 1 != 0 { 15 } else { 0 };
            expected.ram[0x6c] = u8::from(case & 2 != 0);
            expected.ram[0x454] = u8::from(case & 4 != 0);
            expected.ram[0x428] = u8::from(case & 8 != 0);
            expected.ram[0x478] = 2;
            expected.ram[0x458] = (case & 1) as u8;
            expected.ram[0x4a0] = [0, 1, 0xbf, 0x40][case / 4 % 4];
            expected.ram[0x20..0x24].copy_from_slice(&[0x40, 2, 0x70, 1]);
            if case & 16 != 0 {
                { let mut image = expected.wram_image().into_owned(); image[0xf360..0xf364].copy_from_slice(&[11, 0, 7, 0]); expected.import_wram_image(&image); }
                expected.write_wram_byte(0xf36c, 0x50);
                expected.write_wram_byte(0xf36d, 0x38);
                expected.write_wram_byte(0xf372, 3);
                expected.write_wram_byte(0xf343, 9);
                expected.write_wram_byte(0xf375, 2);
                expected.write_wram_byte(0xf376, 2);
                expected.write_wram_byte(0xf377, 29);
            }
            if case & 16 == 0 && case & 8 != 0 {
                expected.write_wram_byte(0xf340, 1);
            }
            if case & 32 != 0 {
                expected.ram[0x20a] = 1;
                expected.ram[0x208] = if case & 1 == 0 { 1 } else { 2 };
                expected.ram[0x209] = (case / 2 % 3) as u8;
                expected.write_wram_byte(0xf36c, 0xa0);
                expected.write_wram_byte(0xf36d, [0x18, 0x38, 0x78][case / 4 % 3]);
            }
            let saved = DungeonSpriteMainReturn {
                bg1_y: (case as u16).wrapping_mul(997),
                bg1_x: (case as u16).wrapping_mul(71),
                bg2_y: (case as u16).wrapping_mul(229),
                bg2_x: (case as u16).wrapping_mul(113),
                link_oam: None,
            };
            for (at, word) in [
                (0x1f5, saved.bg1_y),
                (0x1f7, saved.bg1_x),
                (0x1f9, saved.bg2_y),
                (0x1fb, saved.bg2_x),
            ] {
                expected.ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
            }
            expected.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
            for (i, byte) in expected.ram[0xa20..0xaa0].iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(37) ^ case as u8;
            }
            expected.sync_native_game_state_from_ram();
            expected.main_loop_caller_phase = MainLoopCallerPhase::ModuleRouting;
            let mut actual = expected.clone();
            actual.rom.clear();
            let mut cpu = NativeNmiContext {
                a: 0x1200,
                x: 3,
                y: 7,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x8842,
                sp: 0x1f4,
                p: 0x30,
            };
            let mut clock = snes::CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(76, 468),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&actual.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                clock.clone(),
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut hdma = snes::HdmaTimingState::new(&actual.dma);
            let mut deadline = None;
            let mut work = DungeonMainReturn::default();
            let mut owned = 0;
            for _ in 0..20000 {
                if work.complete() {
                    break;
                }
                let pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                source.step().unwrap();
                let (cycles, _) = work
                    .step(
                        &mut cpu,
                        &mut clock,
                        DungeonReturnBus::new(&mut actual, &mut hdma, &mut deadline),
                    )
                    .unwrap();
                let diffs = actual
                    .wram_image()
                    .iter()
                    .zip(&source.snes().ram)
                    .enumerate()
                    .filter_map(|(i, (a, b))| (a != b).then_some((i, *a, *b)))
                    .take(16)
                    .collect::<Vec<_>>();
                assert!(
                    diffs.is_empty(),
                    "case={case} pc={pc:06x} native bus vs Source {diffs:x?}"
                );
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
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
                    "case={case} pc={pc:06x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:06x}");
                owned += cycles;
                instructions += 1;
                work = work.clone();
            }
            assert!(work.complete(), "case={case}");
            let before = crate::cycle_ledger::master();
            expected.complete_module07_link_oam_after_sprite_main(saved);
            expected.complete_module07_hud_after_link_oam();
            expected.nmi_prepare_sprites_for_main_loop();
            expected.finish_main_loop_caller_return();
            let full = crate::cycle_ledger::master() - before;
            // Link's ROM table pointer shares $74 with collision scratch.
            // The C draw uses arrays rather than this pointer. Exact Source
            // RAM above still covers these bytes after every instruction.
            let excluded =
                |i: usize| i < 0x10 || (0x74..0x76).contains(&i) || (0x1c0..0x200).contains(&i);
            let differences = |a: &[u8], b: &[u8]| {
                a.iter()
                    .zip(b)
                    .enumerate()
                    .filter_map(|(i, (a, b))| (!excluded(i) && a != b).then_some((i, *a, *b)))
                    .take(16)
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                differences(&actual.wram_image(), &expected.wram_image()),
                vec![],
                "case={case} RAM"
            );
            let mut a = vec![0xa5; snes::WRAM_SIZE];
            let mut b = a.clone();
            actual.game_state.write_to_ram(&mut a);
            expected.game_state.write_to_ram(&mut b);
            assert_eq!(differences(&a, &b), vec![], "case={case} typed projection");
            assert_eq!(owned, full, "case={case} complete caller work");
            // Exercise the actual production handoff on the returned cursor.
            // The reference retires ordinary wait instructions to the same
            // atomic NMI eligibility boundary, without entering the handler.
            let wait_at = clock.master_cycles_at_raster(
                clock.field_index(),
                snes::CpuRasterPosition::new(225, 12),
            );
            assert!(clock.clock_master_cycles() < wait_at);
            while source.timeline().clock_master_cycles() < wait_at {
                source.step().unwrap();
            }
            actual.retain_native_dungeon_main_wait(&mut cpu, &clock, &mut hdma, &mut deadline);
            let phase = actual.native_main_wait_cpu_phase.as_ref().unwrap();
            assert_eq!(phase.host, actual.frame_ctr_dbg + 1);
            assert_eq!(phase.budget.retained_native_timeline(), *source.timeline());
            assert_eq!(
                *actual.wram_image(),
                source.snes().ram,
                "case={case} retained wait RAM"
            );
            let c = &source.snes().cpu;
            assert_eq!(
                cpu,
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
                }
            );
            assert_eq!(
                (
                    phase.checkpoint.entry_pc,
                    phase.checkpoint.a,
                    phase.checkpoint.x,
                    phase.checkpoint.y,
                    phase.checkpoint.sp
                ),
                (u32::from(cpu.pc), cpu.a, cpu.x, cpu.y, cpu.sp)
            );
        }
        eprintln!("64 native dungeon returns match {instructions} Source instructions and C owned state/work with no runtime ROM");
    }
}
