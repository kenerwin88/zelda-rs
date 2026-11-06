use super::*;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

#[test]
fn prefix10_guard_collision_caller_matches_rom() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
    for slot in [0usize, 15] {
        for ai in [0, 3, 4] {
            for recoil in [0, 1, 4, 128] {
                for subtype in [0, 5] {
                    for alert in [0, 1] {
                        let mut n = fresh_state();
                        for (a, v) in [(0xdd0, 9), (0xe20, 0x42), (0xd80, ai),
                            (0xea0, recoil), (0xe30, subtype), (0xd10, 0x40), (0xd00, 0x40)] {
                            n.write_wram_byte(a + slot, v);
                        }
                        for (a, v) in [(0x10, 9), (0x1b, 0), (0xfdc, alert),
                            (0xfa0, slot as u8), (0xf359, 1)] { n.write_wram_byte(a, v); }
                        n.write_wram_word(0x20, 0x1000);
                        n.write_wram_word(0x22, 0x1000);
                        n.sync_native_game_state_from_ram();
                        n.set_lag_free(true);
                        let mut s = snes::Snes::new();
                        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                        s.ram.copy_from_slice(&n.wram_image());
                        s.cpu.e = false; s.cpu.unpack_flags(0x30);
                        s.cpu.k = 5; s.cpu.db = 5; s.cpu.pc = 0xc263;
                        s.cpu.x = slot as u16; s.cpu.sp = 0x1fb;
                        s.ram[0x1fc..0x200].copy_from_slice(&[0xff, 0x7f, 0, 0]);
                        crate::cycle_ledger::assert_rom_marker_work(0x05c263, s,
                            |s| s.cpu.k == 5 && (0xc263..0xc2c6).contains(&s.cpu.pc), || {
                                let _scope = crate::cycle_ledger::routine(0x05c263);
                                if n.guard_main_after_parry_through_collision(slot) {
                                    n.guard_main_after_collision_before_ai(slot);
                                }
                            });
                    }
                }
            }
        }
    }
}

#[test]
fn guard_probe_oam_boundary_commits_movement_without_replaying_it_on_resume() {
    let mut split = fresh_state();
    let probe_slot = 8;
    let parent_slot = 10;
    split.follower_link_state_mut().set_position(0x0200, 0x0200);
    split
        .sprite_slot_view_mut(parent_slot)
        .set_sprite_type(0xce);
    {
        let mut probe = split.sprite_slot_view_mut(probe_slot);
        probe.set_state(9);
        probe.set_sprite_type(0x41);
        probe.set_c((parent_slot + 1) as u8);
        probe.set_x(0x0080);
        probe.set_y(0x0080);
        probe.set_x_velocity((-16i8) as u8);
        probe.set_y_velocity(2);
    }
    let mut atomic = split.clone();

    let oam_position = split
        .probe_until_after_oam_coordinates(probe_slot)
        .expect("visible guard probe did not reach its post-OAM boundary");
    assert_eq!(split.sprite_slot_view(probe_slot).x(), 0x0070);
    assert_eq!(split.sprite_slot_view(probe_slot).y(), 0x0082);
    split.complete_probe_after_oam_coordinates(probe_slot, oam_position);

    atomic.probe(probe_slot);
    assert_eq!(
        split.sprite_slot_view(probe_slot).state(),
        atomic.sprite_slot_view(probe_slot).state()
    );
    assert_eq!(
        split.sprite_slot_view(probe_slot).x(),
        atomic.sprite_slot_view(probe_slot).x()
    );
    assert_eq!(
        split.sprite_slot_view(probe_slot).y(),
        atomic.sprite_slot_view(probe_slot).y()
    );
}

#[test]
fn guard_random_patrol_delay_preserves_rng_carry_through_masked_adc() {
    assert_eq!(
        soldier_random_patrol_delay(crate::rom_random::RomRandomResult::new(0x7b, false)),
        99,
    );
    assert_eq!(
        soldier_random_patrol_delay(crate::rom_random::RomRandomResult::new(0x7b, true)),
        100,
    );
}

#[test]
fn guard_tick_and_update_body_advances_subtype_and_writes_gfx() {
    // sprite_subtype2 ++; t = sprite_D * 4 + (sprite_subtype2 >> 3 & 3);
    // sprite_graphics = kSoldier_Gfx2[t].
    let mut state = fresh_state();
    let k = 4;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_direction(2); // base index 8
        sprite.set_subtype2(0x10); // pre-incr -> 0x11 -> shift>>3 = 2 -> &3 = 2
    }
    state.guard_tick_and_update_body(k);
    assert_eq!(state.sprite_slot_view(k).subtype2(), 0x11);
    // t = 2*4 + 2 = 10 -> kSoldier_Gfx2[10] = 2
    assert_eq!(state.sprite_slot_view(k).graphics(), 2);
}

#[test]
fn guard_set_timer_writes_main_subtype_and_flags() {
    // sprite_delay_main = a; sprite_subtype = 0;
    // sprite_flags = (sprite_flags & 0xf) | 0x60.
    let mut state = fresh_state();
    let k = 7;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_flags(0xab);
        sprite.set_subtype(0x55);
    }
    state.guard_set_timer_and_assert_tile_hit_box(k, 0x7f);
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.delay_main(), 0x7f);
    assert_eq!(sprite.subtype(), 0);
    assert_eq!(sprite.flags(), (0xab & 0xf) | 0x60);
}

#[test]
fn green_knife_guard_moving_resets_when_wallcoll() {
    // wallcoll != 0 -> takes main branch with t = 0x10, zero velocity,
    // set head_dir from table, reset ai_state, and bump subtype2 by
    // (delay_aux1?2:1).
    let mut state = fresh_state();
    let k = 0;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_wall_collision(0x01);
        sprite.set_direction(2); // table index base = 4
        sprite.set_delay_aux1(0); // increment by 1
        sprite.set_subtype2(0x10);
        sprite.set_x_velocity(12);
        sprite.set_y_velocity(33);
        sprite.set_ai_state(9);
    }

    state.green_knife_guard_moving(k);
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.delay_main(), 0x10);
    assert_eq!(sprite.x_velocity(), 0);
    assert_eq!(sprite.y_velocity(), 0);
    assert_eq!(sprite.ai_state(), 0);
    // get_random_number called once; rnd&1 in 0..=1, idx in 4..=5,
    // table at idx 4=0 or idx 5=1. We accept either; both ∈ {0,1}.
    let hd = sprite.head_direction();
    assert!(hd == 0 || hd == 1);
    assert_eq!(sprite.subtype2(), 0x11);
}

#[test]
fn green_knife_guard_moving_skips_main_when_delay_active() {
    // wallcoll == 0 and delay_main != 0 -> jump to "out".
    // Expect: delay_main unchanged, ai_state unchanged, head_dir unchanged,
    // subtype2 incremented.
    let mut state = fresh_state();
    let k = 2;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_wall_collision(0);
        sprite.set_delay_main(0x20);
        sprite.set_ai_state(1);
        sprite.set_head_direction(9);
        sprite.set_subtype2(5);
        sprite.set_delay_aux1(1); // increment by 2
    }

    state.green_knife_guard_moving(k);
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.delay_main(), 0x20);
    assert_eq!(sprite.ai_state(), 1);
    assert_eq!(sprite.head_direction(), 9);
    assert_eq!(sprite.subtype2(), 7);
}

#[test]
fn bolt_guard_trigger_chase_theme_pings_sfx_and_music() {
    // When sprite_G == 15, postincrement fires SFX + sets music=12
    // (when sram_progress=2 and overworld_area_lo=24).
    let mut state = fresh_state();
    let k = 1;
    state.sprite_slot_view_mut(k).set_g(15);
    state.save_progress_mut().set_progress_indicator(2);
    state.set_overworld_area_index(24);
    state.bolt_guard_trigger_chase_theme(k);
    assert_eq!(state.sprite_slot_view(k).g(), 16);
    assert_eq!(state.game_state.system_signals.music_control(), 12);
}

#[test]
fn bolt_guard_trigger_chase_theme_does_nothing_at_cap() {
    // sprite_G == 16 already: short-circuit, no change.
    let mut state = fresh_state();
    let k = 3;
    state.sprite_slot_view_mut(k).set_g(16);
    state.set_music_control(7);
    state.bolt_guard_trigger_chase_theme(k);
    assert_eq!(state.sprite_slot_view(k).g(), 16);
    assert_eq!(state.game_state.system_signals.music_control(), 7);
}

#[test]
fn guard_shoot_probe_writes_vel_dir_and_gfx() {
    // i = sprite_B (e.g. 0) -> Xvel=1, Yvel=-1; then with wallcoll = 0
    // and delay_aux2=0, falls through to set delay_aux2 = 88. Then
    // mask2[0] = 1, so if wallcoll & 1 = 0, B stays.
    // Final Xvel2[0]=8, Yvel2[0]=0; Dir[0]=0; calls tick+update.
    let mut state = fresh_state();
    let k = 6;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_b(0);
        sprite.set_wall_collision(0);
        sprite.set_delay_aux2(0);
        sprite.set_direction(0);
        sprite.set_subtype2(0);
        sprite.set_flags2(0x60);
    }
    state.guard_shoot_probe_and_stuff(k);
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.delay_aux2(), 88);
    assert_eq!(sprite.x_velocity(), 8u8);
    assert_eq!(sprite.y_velocity(), 0);
    assert_eq!(sprite.direction(), 0);
    assert_eq!(sprite.head_direction(), 0);
    // tick_and_update bumped subtype2.
    assert_eq!(sprite.subtype2(), 1);
}


#[test]
fn prefix11_guard_chase_and_speed_callers_match_rom() {
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
    for slot in [0usize, 15] {
        for delay in [0, 1, 128] {
            for frame in [0, 15, 31, 32] {
                for wall in [0, 1, 4, 15] {
                    for g in [0, 1] {
                        let mut n = fresh_state();
                        n.set_lag_free(true); n.write_wram_byte(0x10, 7); n.write_wram_byte(0x1b, 1);
                        for (a, v) in [(0xdd0, 9), (0xe20, 0x42), (0xd80, 4),
                            (0xdf0, delay), (0xde0, 2), (0xed0, g), (0xe70, wall),
                            (0xd10, 0x40), (0xd00, 0x40)] { n.write_wram_byte(a + slot, v); }
                        n.write_wram_byte(0xfa0, slot as u8); n.write_wram_byte(0x1a, frame);
                        n.write_wram_word(0x20, 0x1000); n.write_wram_word(0x22, 0x1000);
                        n.sync_native_game_state_from_ram();
                        let mut s = snes::Snes::new(); s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                        s.ram.copy_from_slice(&n.wram_image());
                        s.cpu.e = false; s.cpu.unpack_flags(0x30); s.cpu.k = 5; s.cpu.db = 5;
                        s.cpu.pc = 0xc4e8; s.cpu.x = slot as u16; s.cpu.sp = 0x1fa;
                        s.ram[0x1fb..0x200].copy_from_slice(&[0xff, 0x7f, 0xff, 0x7f, 0]);
                        crate::cycle_ledger::assert_rom_marker_work(0x05c4e8, s,
                            |s| s.cpu.k == 5 && ((0xc4e8..0xc4f9).contains(&s.cpu.pc)
                                || (0xc500..0xc53c).contains(&s.cpu.pc)
                                || (0xc417..0xc425).contains(&s.cpu.pc)), || {
                                let _owner = crate::cycle_ledger::routine(0x05c4e8);
                                n.guard_main_ai(slot);
                            });
                    }
                }
            }
        }
    }
}


#[test]
fn prefix11_parry_guards_and_callers_all_paths_match_rom() {
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    for slot in [0usize, 15] {
        for variant in 0..128u8 {
            for frames in [0, 9, 0x80] {
                for near in [false, true] {
                    let mut n = fresh_state(); n.set_lag_free(true);
                    for (a, v) in [(0xdd0, 9), (0xe20, if variant & 64 == 0 { 0x42 } else { 0x6a }), (0xf20, variant & 1),
                        (0xef0, if variant & 2 != 0 { 0x80 } else { 0 }),
                        (0xd10, 0x40), (0xd00, 0x40)] { n.write_wram_byte(a + slot, v); }
                    for (a, v) in [(0x10, 7), (0x1b, 1), (0x46, (variant >> 2) & 1),
                        (0x4d, (variant >> 3) & 1), (0x37a, if variant & 16 != 0 { 0x10 } else { 0 }),
                        (0x44, if variant & 32 != 0 { 0x80 } else { 0 }), (0x3c, frames),
                        (0xfa0, slot as u8), (0x1a, 0)] { n.write_wram_byte(a, v); }
                    n.write_wram_word(0x20, if near { 0x40 } else { 0x1000 });
                    n.write_wram_word(0x22, if near { 0x40 } else { 0x1000 });
                    n.write_wram_byte(0xf359, 1);
                    n.sync_native_game_state_from_ram();
                    let mut s = snes::Snes::new(); s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    s.ram.copy_from_slice(&n.wram_image()); s.cpu.e = false; s.cpu.unpack_flags(0x30);
                    s.cpu.k = 6; s.cpu.db = 6; s.cpu.pc = 0xeb76; s.cpu.x = slot as u16;
                    s.cpu.sp = 0x1fa; s.ram[0x1fb..0x200].copy_from_slice(&[0xff, 0x7f, 0xff, 0x7f, 0]);
                    crate::cycle_ledger::assert_rom_marker_work(0x06eb76, s,
                        |s| s.cpu.k == 6 && s.cpu.sp == 0x1fa
                            && ((0xeb76..0xec02).contains(&s.cpu.pc)
                                || (0xf3ca..0xf3db).contains(&s.cpu.pc) || s.cpu.pc == 0xf41e), || {
                            let _owner = crate::cycle_ledger::routine(0x06eb76);
                            n.guard_parry_sword_attacks(slot);
                        });
                }
            }
        }
    }
}


#[test]
fn prefix11_guard_facing_state_all_paths_match_rom() {
    let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap();
    for slot in [0usize, 15] {
        for delay in [0, 1, 128] {
            for near in [false, true] {
                for owner in [0x05c4c1, 0x05f93f, 0x06eaa0] {
                    let mut n = fresh_state(); n.set_lag_free(true);
                    for (a, v) in [(0xdd0, 9), (0xe20, 0x42), (0xd80, 3), (0xdf0, delay),
                        (0xd10, 0x40), (0xd00, 0x40)] { n.write_wram_byte(a + slot, v); }
                    n.write_wram_byte(0xfa0, slot as u8);
                    n.write_wram_word(0x20, if near { 0x40 } else { 0x1000 });
                    n.write_wram_word(0x22, if near { 0x40 } else { 0x1000 });
                    n.sync_native_game_state_from_ram();
                    let mut s = snes::Snes::new(); s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    s.ram.copy_from_slice(&n.wram_image()); s.cpu.e = false; s.cpu.unpack_flags(0x30);
                    s.cpu.k = 5; s.cpu.db = 5; s.cpu.pc = 0xc4c1; s.cpu.x = slot as u16;
                    s.cpu.sp = 0x1fa; s.ram[0x1fb..0x200].copy_from_slice(&[0xff, 0x7f, 0xff, 0x7f, 0]);
                    crate::cycle_ledger::assert_rom_marker_work(owner, s,
                        |s| match owner {
                            0x05c4c1 => s.cpu.k == 5 && (0xc4c1..0xc4e8).contains(&s.cpu.pc),
                            0x05f93f => s.cpu.k == 5 && (0xf93f..0xf944).contains(&s.cpu.pc),
                            _ => s.cpu.k == 6 && (0xeaa0..0xeaa4).contains(&s.cpu.pc),
                        }, || {
                            let _owner = crate::cycle_ledger::routine(0x05c4c1);
                            n.guard_main_ai(slot);
                        });
                }
            }
        }
    }
}
