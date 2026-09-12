use super::*;

#[test]
fn catfish_medallion_spawn_is_visible_before_its_graphics_decode() {
    let mut atomic = ZeldaState::new();
    atomic.set_main_module(9);
    atomic.set_submodule(0);
    atomic.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    {
        let mut sprite = atomic.sprite_slot_view_mut(11);
        sprite.set_state(9);
        sprite.set_sprite_type(0xc0);
        sprite.set_ai_state(3);
        sprite.set_delay_main(80);
        sprite.set_x(0x40);
        sprite.set_y(0x50);
    }
    let mut staged = atomic.clone();
    atomic.catfish_big_fish(11);
    staged.catfish_before_medallion_graphics(11);
    assert_eq!(staged.sprite_slot_view(15).state(), 9);
    assert_eq!(staged.sprite_slot_view(15).x_velocity(), 24);
    assert_eq!(staged.sprite_slot_view(11).graphics(), 0);
    staged.complete_catfish_medallion_graphics(11);
    assert_eq!(staged.game_state, atomic.game_state);
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn catfish_splash_carry_controls_the_conversation_table_overread() {
    for (indoors, exhausted, expected) in [(false, false, 0xbd), (true, false, 0), (false, true, 0)]
    {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.set_submodule(0);
        state.set_indoor_flag(u8::from(indoors));
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        if exhausted {
            for slot in 0..16 {
                state.sprite_slot_view_mut(slot).set_state(9);
            }
        }
        {
            let mut sprite = state.sprite_slot_view_mut(11);
            sprite.set_state(9);
            sprite.set_sprite_type(0xc0);
            sprite.set_ai_state(3);
            sprite.set_delay_main(160);
            sprite.set_x(0x40);
            sprite.set_y(0x50);
        }
        state.catfish_big_fish(11);
        assert_eq!(state.sprite_slot_view(11).graphics(), expected);
        state.sprite_slot_view_mut(11).set_delay_main(159);
        state.catfish_big_fish(11);
        assert_eq!(state.sprite_slot_view(11).graphics(), 6);
    }
}

#[test]
fn buzzblob_x_subpixel_resume_does_not_repeat_its_direction_prefix() {
    for (velocity, delay) in [(2, 5), (0xfe, 5), (0x7f, 0)] {
        let mut atomic = ZeldaState::new();
        atomic.set_main_module(9);
        atomic.set_submodule(0);
        atomic.set_frame_counter(1);
        atomic.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        atomic.sprite_set_x(11, 0x40);
        atomic.sprite_set_y(11, 0x50);
        {
            let mut sprite = atomic.sprite_slot_view_mut(11);
            sprite.set_state(9);
            sprite.set_sprite_type(0x0d);
            sprite.set_delay_main(delay);
            sprite.set_x_velocity(velocity);
            sprite.set_y_velocity(0xfe);
            sprite.set_x_subpixel(0xf0);
        }
        let mut staged = atomic.clone();
        atomic.sprite_0_d_buzzblob(11);
        staged.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
            slot: 11,
            pending: None,
        });
        staged.sprite_0_d_buzzblob(11);
        assert_eq!(staged.sprite_slot_view(11).subtype2(), 1);
        assert_eq!(staged.sprite_get_x(11), 0x40);
        let Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
            pending: Some((low, high)),
            ..
        }) = staged.sprite_main_cpu_boundary.take()
        else {
            panic!("Buzzblob did not preserve the pending X coordinate");
        };
        staged.complete_sprite_move_x_after_subpixel(11, low, high);
        staged.sprite_move_y(11);
        staged.buzzblob_after_movement(11);
        assert_eq!(staged.game_state, atomic.game_state);
        assert_eq!(staged.ram, atomic.ram);
    }
}

#[test]
fn kholdstare_damage_continuation_preserves_completed_movement() {
    let mut atomic = ZeldaState::new();
    atomic.set_main_module(9);
    atomic.set_frame_counter(1);
    atomic.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    atomic.sprite_set_x(0, 0x40);
    atomic.sprite_set_y(0, 0x50);
    {
        let mut sprite = atomic.sprite_slot_view_mut(0);
        sprite.set_state(9);
        sprite.set_sprite_type(0xa2);
        sprite.set_ai_state(1);
        sprite.set_delay_main(20);
        sprite.set_subtype2(8);
        sprite.set_x_velocity(6);
        sprite.set_y_velocity(0xfa);
    }
    let mut staged = atomic.clone();
    atomic.sprite_a2_kholdstare(0);
    assert!(staged.kholdstare_before_ai(0));
    assert_eq!(staged.sprite_slot_view(0).subtype2(), 7);
    assert_eq!(staged.sprite_slot_view(0).x_subpixel(), 0x60);
    assert_eq!(staged.sprite_slot_view(0).y_subpixel(), 0xa0);
    assert_eq!(staged.sprite_slot_view(0).x_velocity(), 6);
    staged.kholdstare_after_movement(0);
    assert_eq!(staged.sprite_slot_view(0).subtype2(), 7);
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn antifairy_bounce_continuation_preserves_movement_and_animation_prefix() {
    let mut atomic = ZeldaState::new();
    atomic.set_main_module(9);
    atomic.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    atomic.sprite_set_x(0, 0x40);
    atomic.sprite_set_y(0, 0x50);
    {
        let mut sprite = atomic.sprite_slot_view_mut(0);
        sprite.set_state(9);
        sprite.set_sprite_type(0x15);
        sprite.set_x_velocity(0xf0);
        sprite.set_y_velocity(0xf0);
    }
    let mut staged = atomic.clone();
    atomic.sprite_15_antifairy(0);
    assert!(staged.antifairy_before_bounce(0));
    assert_eq!(staged.sprite_slot_view(0).x_low(), 0x3f);
    assert_eq!(staged.sprite_slot_view(0).y_low(), 0x4f);
    assert_eq!(staged.sprite_slot_view(0).subtype2(), 1);
    staged.sprite_bounce_from_tile_collision(0);
    assert_eq!(staged.sprite_slot_view(0).subtype2(), 1);
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn pengator_slide_continuation_does_not_repeat_movement() {
    let mut atomic = ZeldaState::new();
    atomic.set_main_module(9);
    atomic.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    atomic.sprite_set_x(6, 0x40);
    atomic.sprite_set_y(6, 0x50);
    {
        let mut sprite = atomic.sprite_slot_view_mut(6);
        sprite.set_state(9);
        sprite.set_sprite_type(0x99);
        sprite.set_ai_state(3);
        sprite.set_f(0);
        sprite.set_wall_collision(0);
        // Keep the fixture's tile collision from applying its one-pixel
        // position correction, so movement replay is directly observable.
        sprite.set_subtype(5);
        sprite.set_y_velocity(0xf0);
        sprite.set_z(32);
    }
    let mut staged = atomic.clone();
    atomic.sprite_99_pengator(6);
    assert!(staged.pengator_before_ai(6));
    assert_eq!(staged.sprite_slot_view(6).y_low(), 0x4f);
    assert_eq!(staged.sprite_slot_view(6).ai_state(), 3);
    staged.pengator_after_movement(6);
    assert_eq!(staged.sprite_slot_view(6).y_low(), 0x4f);
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn active_guard_weapon_coordinates_hold_pose_until_the_draw_returns() {
    use crate::GuardAnimationCheckpoint as Stage;
    let mut checkpoints = vec![
        Stage::HeadCharacterPending,
        Stage::HeadFlagsPending,
        Stage::DrawReturned,
        Stage::HeadExtendedPending,
    ];
    for entry in 0..4 {
        checkpoints.extend([
            Stage::BodyBeforeEntry { entry },
            Stage::BodyCoordinates { entry },
            Stage::BodyFlagsPending { entry },
        ]);
    }
    for entry in 0..2 {
        checkpoints.extend([
            Stage::WeaponBeforeCoordinates { entry },
            Stage::WeaponCoordinates { entry },
        ]);
    }
    for checkpoint in checkpoints {
        let mut atomic = ZeldaState::new();
        atomic.set_main_module(9);
        atomic.set_submodule(0x2e);
        atomic
            .oam_state_mut()
            .set_current_pointer((OAM_BUF + 0x140) as u16);
        atomic.sprite_set_x(10, 0x40);
        atomic.sprite_set_y(10, 0x50);
        {
            let mut sprite = atomic.sprite_slot_view_mut(10);
            sprite.set_state(9);
            sprite.set_sprite_type(0x41);
            sprite.set_direction(0);
            sprite.set_graphics(11);
            sprite.set_delay_aux1(1);
        }
        let mut staged = atomic.clone();
        atomic.guard_main(10);
        let continuation = staged.guard_animation_until_checkpoint(10, checkpoint);
        assert_eq!(staged.sprite_slot_view(10).direction(), 3);
        assert_eq!(staged.sprite_slot_view(10).graphics(), 8);
        staged.complete_guard_animation_at_checkpoint(10, continuation);
        assert_eq!(staged.sprite_slot_view(10).direction(), 0);
        assert_eq!(staged.sprite_slot_view(10).graphics(), 11);
        assert_eq!(staged.ram, atomic.ram);
    }
}

#[test]
fn guard_body_packs_skipped_oam_and_hides_type_46_blank_tiles() {
    let graphics = (0..SOLDIER_DRAW2_CHAR.len() / 4)
        .find(|&graphics| {
            let emitted: Vec<_> = (0..=3)
                .rev()
                .filter_map(|i| {
                    let j = graphics * 4 + i;
                    if SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20 {
                        None
                    } else {
                        Some(j)
                    }
                })
                .collect();
            emitted.len() < 4 && emitted.iter().any(|&j| SOLDIER_DRAW2_CHAR[j] == 0x20)
        })
        .expect("soldier draw table should contain type-46 skipped blank body entries");
    let expected: Vec<_> = (0..=3)
        .rev()
        .filter_map(|i| {
            let j = graphics * 4 + i;
            if SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20 {
                None
            } else {
                Some(SOLDIER_DRAW2_CHAR[j])
            }
        })
        .collect();

    let mut s = ZeldaState::new();
    s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    for i in 0..4 {
        let base = OAM_BUF + i * 4;
        s.oam_state_mut().set_entry_y(base, 0xee);
        s.oam_state_mut().set_entry_char(base, 0xee);
    }
    let k = 0;
    s.sprite_slot_view_mut(k).set_sprite_type(0x46);
    s.sprite_slot_view_mut(k).set_graphics(graphics as u8);

    s.guard_animate_body(
        k,
        0,
        &PrepOamCoordsRet {
            x: 0x40,
            y: 0x50,
            r4: 0,
            flags: 0,
        },
    );

    for (slot, &charnum) in expected.iter().enumerate() {
        assert_eq!(s.ram[OAM_BUF + slot * 4 + 2], charnum);
        if charnum == 0x20 {
            assert_eq!(s.ram[OAM_BUF + slot * 4 + 1], 0xf0);
        }
    }
    assert_eq!(s.ram[OAM_BUF + expected.len() * 4 + 2], 0xee);
}

#[test]
fn guard_drawing_cycle_charges_match_rom_for_poses_and_clipping() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    use crate::zelda_rtl::sprite::PrepOamCoordsRet;
    let rom_path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(rom_path) else {
        eprintln!("guard cycle reference skipped: no development ROM");
        return;
    };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for (entry, poses) in [(0x05_c6de, SOLDIER_DRAW1_YD.len()),
        (0x05_ca09, SOLDIER_DRAW2_CHAR.len() / 4),
        (0x05_cb64, SOLDIER_DRAW3_CHAR.len() / 2)] {
        for sprite_type in [0x41, 0x42, 0x43, 0x46, 0x47] {
            for graphics in 0..poses {
                for direction in 0..4 {
                    for (x, y) in [(40u16, 50u16), (255, 250), (511, 0xfff0)] {
                        let mut state = ZeldaState::new();
                        state.oam_state_mut().set_current_pointer(0x0800);
                        state.oam_state_mut().set_current_extended_pointer(0x0a20);
                        {
                            let mut slot = state.sprite_slot_view_mut(0);
                            slot.set_sprite_type(sprite_type);
                            slot.set_graphics(graphics as u8);
                            slot.set_direction(direction);
                            slot.set_head_direction(direction);
                        }
                        state.game_state.write_to_ram(&mut state.ram);
                        state.ram[0..2].copy_from_slice(&x.to_le_bytes());
                        state.ram[2..4].copy_from_slice(&y.to_le_bytes());
                        state.ram[5] = 0x20;
                        let checkpoint = RomCpuCheckpoint {
                            entry_pc: entry, stop_pc: 0x05_8000, a: 0, x: 0, y: 0,
                            sp: 0x01fd, dp: 0, db: 5, carry: false, zero: false,
                            overflow: false, negative: false, interrupt_disable: true,
                            decimal: false, accumulator_is_8_bit: true, index_is_8_bit: true,
                            emulation: false, waiting: false, stack_address: 0x01fe,
                            stack_bytes: &[0xff, 0x7f],
                        };
                        let mut cpu = RomCpuTimingRun::new(&rom, &state.ram, &state.sram,
                            &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
                        let mut expected = 0u64;
                        for _ in 0..10_000 {
                            if cpu.is_complete() { break; }
                            expected += u64::from(cpu.step().master_cycles);
                        }
                        assert!(cpu.is_complete());
                        let before = crate::cycle_ledger::master();
                        let poc = PrepOamCoordsRet { x, y, r4: 0, flags: 0x20 };
                        match entry {
                            0x05_c6de => state.guard_animate_head(0, 0, &poc),
                            0x05_ca09 => state.guard_animate_body(0, SOLDIER_DRAW2_OAM_IDX[direction as usize] >> 2, &poc),
                            _ => state.guard_animate_weapon(0, &poc),
                        }
                        assert_eq!(crate::cycle_ledger::master() - before, expected,
                            "entry={entry:06x} type={sprite_type:02x} graphics={graphics} direction={direction} x={x} y={y}");
                    }
                }
            }
        }
    }
}

#[test]
fn mantle_and_oam_correction_cycles_match_rom_for_clipping() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let rom_path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(rom_path) else {
        eprintln!("OAM cycle reference skipped: no development ROM");
        return;
    };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for entry in [0x06_febc, 0x1a_fcb3, 0x1a_fc31] {
        for (x, y) in [(40u16, 50u16), (255, 223), (511, 0xfff0), (0xff80, 0x80)] {
            for size in [0, 2, 0xff] {
                for count in [0, 1, 5, 7] {
                    let mut state = ZeldaState::new();
                    state.clear_oam_buffer();
                    state.sprite_set_x(4, x);
                    state.sprite_set_y(4, y);
                    state.set_bg2_x(0x10);
                    state.set_bg2_y(0x20);
                    state.sprite_slot_view_mut(4).set_state(9);
                    state.sprite_slot_view_mut(4).set_sprite_type(0xee);
                    state.set_submodule(2); // the mantle's dialogue-time inactive return
                    state.oam_state_mut().set_current_pointer(0x0800);
                    state.oam_state_mut().set_current_extended_pointer(0x0a20);
                    for i in 0..8 {
                        state.oam_state_mut().write_entry(0x0800 + 4 * i, (i * 37) as u8, (255 - i * 31) as u8, 0, 0);
                        state.oam_state_mut().set_extended_byte_at(0x0a20 + i, (i & 2) as u8);
                    }
                    state.game_state.write_to_ram(&mut state.ram);
                    let bank = (entry >> 16) as u8;
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: entry, stop_pc: (u32::from(bank) << 16) | 0x8000,
                        a: count as u16, x: 4, y: size as u16,
                        sp: if entry == 0x1a_fc31 { 0x01fc } else { 0x01fd },
                        dp: 0, db: bank, carry: false, zero: false,
                        overflow: false, negative: false, interrupt_disable: true,
                        decimal: false, accumulator_is_8_bit: true, index_is_8_bit: true,
                        emulation: false, waiting: false,
                        stack_address: if entry == 0x1a_fc31 { 0x01fd } else { 0x01fe },
                        stack_bytes: if entry == 0x1a_fc31 { &[0xff, 0x7f, 0x1a] } else { &[0xff, 0x7f] },
                    };
                    let mut cpu = RomCpuTimingRun::new(&rom, &state.ram, &state.sram,
                        &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
                    let mut expected = 0u64;
                    for _ in 0..100_000 {
                        if cpu.is_complete() { break; }
                        expected += u64::from(cpu.step().master_cycles);
                    }
                    assert!(cpu.is_complete());
                    let before = crate::cycle_ledger::master();
                    if entry == 0x06_febc {
                        state.sprite_correct_oam_entries(4, count, size);
                    } else if entry == 0x1a_fcb3 {
                        state.movable_mantle_draw(4);
                    } else {
                        state.sprite_ee_movable_mantle(4);
                    }
                    assert_eq!(crate::cycle_ledger::master() - before, expected,
                        "entry={entry:06x} x={x} y={y} size={size} count={count}");
                    assert_eq!(&state.ram[0x800..0xa40], &cpu.ram()[0x800..0xa40],
                        "OAM entry={entry:06x} x={x} y={y} size={size} count={count}");
                }
            }
        }
    }
}

#[test]
fn zelda_dialogue_drawing_cycles_match_rom() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let rom_path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(rom_path) else { return; };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for entry in [0x0d_ce5f, 0x06_c067] {
        for direction in 0..4 {
            for graphics in 0..2 {
                for (x, y, floor) in [(40u16, 50u16, 1), (255, 223, 1), (511, 0xfff0, 1),
                    (40, 50, 0), (0, 0, 0), (0, 20, 0), (511, 0xfff0, 0)] {
                    let mut state = ZeldaState::new();
                    state.clear_oam_buffer();
                    state.oam_state_mut().set_current_pointer(0x0800);
                    state.oam_state_mut().set_current_extended_pointer(0x0a20);
                    state.sprite_slot_view_mut(4).set_floor(floor);
                    state.oam_reset_region_bases();
                    state.sprite_set_x(4, x);
                    state.sprite_set_y(4, y);
                    state.set_bg2_x(0x10);
                    state.set_bg2_y(0x20);
                    state.sprite_slot_view_mut(4).set_state(9);
                    state.sprite_slot_view_mut(4).set_sprite_type(0x76);
                    state.sprite_slot_view_mut(4).set_direction(direction);
                    state.sprite_slot_view_mut(4).set_graphics(graphics);
                    state.set_submodule(2);
                    state.game_state.write_to_ram(&mut state.ram);
                    let bank = (entry >> 16) as u8;
                    let long = entry == 0x0d_ce5f;
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: entry, stop_pc: (u32::from(bank) << 16) | 0x8000,
                        a: 0, x: 4, y: 0, sp: if long { 0x01fc } else { 0x01fd },
                        dp: 0, db: bank, carry: false, zero: false,
                        overflow: false, negative: false, interrupt_disable: true,
                        decimal: false, accumulator_is_8_bit: true, index_is_8_bit: true,
                        emulation: false, waiting: false,
                        stack_address: if long { 0x01fd } else { 0x01fe },
                        stack_bytes: if long { &[0xff, 0x7f, 0x0d] } else { &[0xff, 0x7f] },
                    };
                    let mut cpu = RomCpuTimingRun::new(&rom, &state.ram, &state.sram,
                        &state.ppu, &state.dma, [0; 4], checkpoint).unwrap();
                    let mut expected = 0u64;
                    for _ in 0..100_000 {
                        if cpu.is_complete() { break; }
                        expected += u64::from(cpu.step().master_cycles);
                    }
                    assert!(cpu.is_complete(), "entry={entry:06x} direction={direction} graphics={graphics} x={x} y={y} pc={:06x}", cpu.pc());
                    let before = crate::cycle_ledger::master();
                    if long { state.crystal_maiden_draw(4); }
                    else { state.sprite_76_zelda(4); }
                    assert_eq!(crate::cycle_ledger::master() - before, expected,
                        "entry={entry:06x} direction={direction} graphics={graphics} x={x} y={y}");
                }
            }
        }
    }
}
