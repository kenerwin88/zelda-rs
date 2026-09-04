use super::*;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

#[test]
fn head_direction_preserves_sprite_is_right_of_link_carry() {
    let sprite_x = 0x0141;

    // The ROM's ADC includes the no-borrow carry from the helper. Therefore
    // +30 is the last centered coordinate on the right, while +31 selects
    // right. The negative side has no carry and remains centered through -32.
    assert_eq!(ganon_head_direction(0x015f, sprite_x), 1);
    assert_eq!(ganon_head_direction(0x0160, sprite_x), 2);
    assert_eq!(ganon_head_direction(0x0121, sprite_x), 1);
    assert_eq!(ganon_head_direction(0x0120, sprite_x), 0);
}

#[test]
fn attempt_trident_catch_matches_8x8_window() {
    // cur_sprite_(x,y) within +/- 4 of the target should report a catch.
    let mut s = fresh_state();
    s.sprite_workspace_mut().set_current_sprite_x(0x100);
    s.sprite_workspace_mut().set_current_sprite_y(0x80);
    // Same coords -> 4 + (0) wraps into window — bool true.
    assert!(s.ganon_attempt_trident_catch(0x100, 0x80));
    // 7-unit X delta should still land within (uint16)(dx + 4) < 8 -> 4+(-7)=-3 wraps high -> false.
    assert!(!s.ganon_attempt_trident_catch(0x107, 0x80));
    // 3-unit positive delta -> (uint16)(0x100-0x103+4) = 1 < 8 -> true.
    assert!(s.ganon_attempt_trident_catch(0x103, 0x80));
}

#[test]
fn enable_invincibility_only_triggers_on_hit_timer_26() {
    let mut s = fresh_state();
    let k = 2;
    // Hit timer must be exactly 26 in its low 7 bits.
    s.sprite_slot_view_mut(k).set_hit_timer(26);
    s.ganon_enable_invincibility(k);
    assert_eq!(s.sprite_slot_view(k).hit_timer(), 0);
    assert_eq!(s.sprite_slot_view(k).ai_state(), 19);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 127);
    assert_eq!(s.sprite_slot_view(k).sprite_type(), 215);

    let mut s2 = fresh_state();
    s2.sprite_slot_view_mut(k).set_hit_timer(27);
    s2.ganon_enable_invincibility(k);
    // Nothing should change.
    assert_eq!(s2.sprite_slot_view(k).hit_timer(), 27);
    assert_eq!(s2.sprite_slot_view(k).ai_state(), 0);
    assert_eq!(s2.sprite_slot_view(k).sprite_type(), 0);

    let mut s3 = fresh_state();
    // Top bit set + low 7 bits == 26 -> still triggers.
    s3.sprite_slot_view_mut(k).set_hit_timer(26 | 0x80);
    s3.ganon_enable_invincibility(k);
    assert_eq!(s3.sprite_slot_view(k).hit_timer(), 0);
}

#[test]
fn phase1_animate_trident_spin_indexes_into_func2_tables() {
    let mut s = fresh_state();
    let k = 0;
    // delay_main = 0 -> base = 0; sprite_D = 0 -> bonus = 0 -> j = 0.
    s.sprite_slot_view_mut(k).set_delay_main(0);
    s.sprite_slot_view_mut(k).set_direction(0);
    s.ganon_phase1_animate_trident_spin(k);
    assert_eq!(s.sprite_slot_view(k).g(), GANON_SPIN_G_STATES[0]); // 8
    assert_eq!(
        s.sprite_slot_view(k).graphics(),
        GANON_TRIDENT_SPIN_GRAPHICS[0]
    ); // 0

    // delay_main = 28 (>> 2 == 7, & 7 == 7); D = 1 -> bonus = 8 -> j = 15.
    s.sprite_slot_view_mut(k).set_delay_main(28);
    s.sprite_slot_view_mut(k).set_direction(1);
    s.ganon_phase1_animate_trident_spin(k);
    assert_eq!(s.sprite_slot_view(k).g(), GANON_SPIN_G_STATES[15]); // 1
    assert_eq!(
        s.sprite_slot_view(k).graphics(),
        GANON_TRIDENT_SPIN_GRAPHICS[15]
    );
    // 9
}

#[test]
fn handle_animation_idle_writes_g_and_gfx_per_direction() {
    let mut s = fresh_state();
    let k = 3;
    s.sprite_slot_view_mut(k).set_direction(0);
    s.ganon_handle_animation_idle(k);
    assert_eq!(s.sprite_slot_view(k).g(), 9);
    assert_eq!(s.sprite_slot_view(k).graphics(), 2);

    s.sprite_slot_view_mut(k).set_direction(1);
    s.ganon_handle_animation_idle(k);
    assert_eq!(s.sprite_slot_view(k).g(), 10);
    assert_eq!(s.sprite_slot_view(k).graphics(), 10);
}

#[test]
fn shake_head_indexes_table_by_delay_main_shift3() {
    let mut s = fresh_state();
    let k = 1;
    // delay_main 24 -> idx 3 -> GANON_SHAKE_HEAD_DIRECTIONS[3] = 1.
    s.sprite_slot_view_mut(k).set_delay_main(24);
    s.ganon_shake_head(k);
    assert_eq!(s.sprite_slot_view(k).head_direction(), 1);
    // delay_main 0 -> idx 0 -> 0.
    s.sprite_slot_view_mut(k).set_delay_main(0);
    s.ganon_shake_head(k);
    assert_eq!(s.sprite_slot_view(k).head_direction(), 0);
    // delay_main 32 -> idx 4 -> 2.
    s.sprite_slot_view_mut(k).set_delay_main(32);
    s.ganon_shake_head(k);
    assert_eq!(s.sprite_slot_view(k).head_direction(), 2);
}

#[test]
fn select_warp_location_zeroes_velocities_and_sets_targets() {
    let mut s = fresh_state();
    let k = 0;
    // Seed sprite_subtype so the (rnd & 3 | subtype<<2) index is
    // deterministic enough for an assertion: with subtype = 0, the
    // resulting index is rnd & 3 only — that picks one of the first
    // four entries of GANON_WARP_SUBTYPES (which are 4,5,6,7).
    s.sprite_slot_view_mut(k).set_subtype(0);
    // Pre-clobber vels to ensure they get zeroed.
    s.sprite_slot_view_mut(k).set_x_velocity(5);
    s.sprite_slot_view_mut(k).set_y_velocity(7);
    s.ganon_select_warp_location(k, 12);
    let j = s.sprite_slot_view(k).subtype();
    assert!((4..=7).contains(&j));
    assert_eq!(
        s.ram[SWAMOLA_TARGET_X_LO_GANON],
        GANON_WARP_TARGET_X_LOW[j as usize]
    );
    assert_eq!(
        s.ram[SWAMOLA_TARGET_Y_LO_GANON],
        GANON_WARP_TARGET_Y_LOW[j as usize]
    );
    assert_eq!(s.sprite_slot_view(k).ai_state(), 12);
    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 48);
}

#[test]
fn spawn_falling_tiles_increments_anim_clock_until_four() {
    let mut s = fresh_state();
    let k = 0;
    // Pre-clear overlord slot 7 so the search succeeds.
    s.overlord_slot_view_mut(7).clear();
    s.sprite_slot_view_mut(k).set_anim_clock(0);
    // Seed link coords so we can verify the high-byte copy.
    s.follower_link_state_mut().set_x(0x0234);
    s.follower_link_state_mut().set_y(0x0588);
    s.ganon_spawn_falling_tiles_overlord(k);
    assert_eq!(s.sprite_slot_view(k).anim_clock(), 1);
    assert_eq!(
        s.overlord_slot_view(7).overlord_type(),
        GANON_FALLING_TILE_OVERLORD_TYPES[0]
    );
    assert_eq!(
        s.overlord_slot_view(7).x_low(),
        GANON_FALLING_TILE_OVERLORD_X_LOW[0]
    );
    assert_eq!(s.overlord_slot_view(7).x_high(), 0x02);
    assert_eq!(s.overlord_slot_view(7).y_high(), 0x05);

    // Advance the anim clock past 3 and ensure no further write happens.
    s.sprite_slot_view_mut(k).set_anim_clock(4);
    let bak = s.overlord_slot_view(7).overlord_type();
    s.ganon_spawn_falling_tiles_overlord(k);
    assert_eq!(s.overlord_slot_view(7).overlord_type(), bak);
    assert_eq!(s.sprite_slot_view(k).anim_clock(), 4);
}

#[test]
fn handle_fire_bat_circle_writes_eight_overlords_and_seeds_counter() {
    let mut s = fresh_state();
    // Seed overlord_x_lo word to a known value so we can predict t for each i.
    s.overlord_slot_view_mut(0).set_adjacent_x_low_word(0x10);
    s.overlord_slot_view_mut(2).set_x_low(0); // scale = 0 -> GanonSin returns 0.
                                              // Sprite 0 at (0x80, 0x80) for predictable add.
    s.sprite_set_x(0, 0x80);
    s.sprite_set_y(0, 0x80);
    // sprite_ai_state for indices 1..=8: leave them at 0 so the velocity
    // assignments fire for every i.
    for i in 1..=8 {
        s.sprite_slot_view_mut(i).set_ai_state(0);
    }

    s.ganon_handle_fire_bat_circle(0);

    // overlord_x_lo word should have decremented by 4.
    assert_eq!(
        s.game_state
            .sprites
            .overlord_slots
            .slot(0)
            .adjacent_x_low_word(),
        0x10u16.wrapping_sub(4)
    );
    // tmp_counter is set to 8.
    assert_eq!(s.game_state.scratch_counter.value(), 8);
    // With scale = 0, GanonSin -> 0, so every overlord_x_hi[i+1] == sprite_x_lo(0) == 0x80.
    for i in 0..8 {
        assert_eq!(s.overlord_slot_view(i + 1).x_high(), 0x80);
        assert_eq!(s.overlord_slot_view(i + 1).gen2(), 0x80);
    }
}

#[test]
fn spawn_spiral_bat_initializes_dynamic_slot_fields() {
    let mut s = fresh_state();
    let k = 0;
    // Canonical Sprite_SpawnDynamicallyEx walks j_in (8) down to 0; the
    // highest free slot in [0..=8] wins. Ensure slot 8 is free so it
    // gets picked (matching the C entry-point behavior).
    s.sprite_slot_view_mut(8).set_state(0);
    s.sprite_workspace_mut().set_current_sprite_x(0x40);
    s.sprite_workspace_mut().set_current_sprite_y(0x60);
    s.ganon_spawn_spiral_bat(k);
    let j = 8;
    assert_eq!(s.sprite_slot_view(j).state(), 9);
    assert_eq!(s.sprite_slot_view(j).sprite_type(), 0xc9);
    assert_eq!(s.sprite_slot_view(j).anim_clock(), 4);
    assert_eq!(s.sprite_slot_view(j).oam_flags(), 3);
    assert_eq!(s.sprite_slot_view(j).flags3(), 0x40);
    assert_eq!(s.sprite_slot_view(j).flags2(), 1);
    assert_eq!(s.sprite_slot_view(j).deflection_bits(), 0x80);
    assert_eq!(s.sprite_slot_view(j).y_high(), 128);
    assert_eq!(s.sprite_slot_view(j).delay_main(), 48);
    assert_eq!(s.sprite_slot_view(j).bump_damage(), 7);
    assert_eq!(s.sprite_slot_view(j).ignore_projectile(), 7);
}
