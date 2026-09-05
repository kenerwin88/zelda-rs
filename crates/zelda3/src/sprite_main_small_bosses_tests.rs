use super::*;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

#[test]
fn trinexx_cache_position_writes_all_four_components() {
    // Trinexx_CachePosition copies the current XY (lo/hi) into the
    // sprite scratch fields A/B/C/G. Verify the byte order matches
    // the C source 1:1.
    let mut s = fresh_state();
    let k = 3;
    s.sprite_slot_view_mut(k).set_x_low(0x40);
    s.sprite_slot_view_mut(k).set_x_high(0x01);
    s.sprite_slot_view_mut(k).set_y_low(0x80);
    s.sprite_slot_view_mut(k).set_y_high(0x02);
    s.trinexx_cache_position(k);
    assert_eq!(s.sprite_slot_view(k).a(), 0x40);
    assert_eq!(s.sprite_slot_view(k).b(), 0x01);
    assert_eq!(s.sprite_slot_view(k).c(), 0x80);
    assert_eq!(s.sprite_slot_view(k).g(), 0x02);
}

#[test]
fn trinexx_restore_xy_recomputes_y_with_plus_12() {
    // Trinexx_RestoreXY restores X from sprite_A and Y from
    // (sprite_G << 8) + sprite_C + 12.
    let mut s = fresh_state();
    let k = 5;
    s.sprite_slot_view_mut(k).set_a(0x77);
    s.sprite_slot_view_mut(k).set_g(0x01);
    s.sprite_slot_view_mut(k).set_c(0xf0);
    s.trinexx_restore_xy(k);
    assert_eq!(s.sprite_slot_view(k).x_low(), 0x77);
    // (1 << 8) + 0xf0 + 12 = 0x100 + 0xf0 + 0x0c = 0x1fc.
    assert_eq!(s.sprite_slot_view(k).y_low(), 0xfc);
    assert_eq!(s.sprite_slot_view(k).y_high(), 0x01);
}

#[test]
fn trinexx_wag_tail_advances_through_cooldown() {
    // overlord_x_lo[5] is the cooldown timer. Non-zero ticks down by
    // one and leaves the rest of the tail state untouched.
    let mut s = fresh_state();
    s.overlord_slot_view_mut(5).set_x_low(4);
    s.overlord_slot_view_mut(4).set_x_low(0);
    s.trinexx_wag_tail(0);
    assert_eq!(s.overlord_slot_view(5).x_low(), 3);
    assert_eq!(s.overlord_slot_view(4).x_low(), 0);

    // With the cooldown cleared and the step counter at 3, the next
    // call bumps to 4 (the 0&3 branch fires), advances the swing
    // amount, and arms the cooldown when it hits the boundary (6).
    s.overlord_slot_view_mut(5).set_x_low(0);
    s.overlord_slot_view_mut(4).set_x_low(3);
    s.overlord_slot_view_mut(3).set_x_low(0); // direction bit: forward.
    s.overlord_slot_view_mut(2).set_x_low(5);
    s.trinexx_wag_tail(0);
    assert_eq!(s.overlord_slot_view(4).x_low(), 4);
    assert_eq!(s.overlord_slot_view(2).x_low(), 6);
    assert_eq!(s.overlord_slot_view(3).x_low(), 1);
    assert_eq!(s.overlord_slot_view(5).x_low(), 8);
}

#[test]
fn vitreous_set_minions_forth_activates_dormant_minion() {
    // sprite_subtype2 increments every call; on the multiple-of-64
    // tick it tries to wake one of the kVitreous_WhichToActivate
    // slots. Use a deterministic random seed so we know which slot.
    let mut s = fresh_state();
    let k = 0;
    // Pre-arm subtype2 so the next increment hits 64.
    s.sprite_slot_view_mut(k).set_subtype2(63);
    // Use the get_random_number seed default; whichever minion slot
    // it picks should transition from 0 -> 1.
    let rand_peek = {
        // Snapshot the RNG by mirroring its current state via a clone.
        let mut clone = s.clone();
        clone.get_random_number()
    };
    let pick = VITREOUS_MINION_ACTIVATION_SLOTS[(rand_peek & 15) as usize] as usize;
    // Mark the picked slot dormant so we exercise the activation arm.
    s.sprite_slot_view_mut(pick).set_ai_state(0);
    s.vitreous_set_minions_forth(k);
    assert_eq!(s.sprite_slot_view(pick).ai_state(), 1);
    assert_eq!(s.game_state.system_signals.sound_effect_1(), 0x15);
    // subtype2 was 63 → bumped to 64 → kept (not rolled back).
    assert_eq!(s.sprite_slot_view(k).subtype2(), 64);
}

#[test]
fn vitreous_set_minions_forth_rolls_back_when_minion_busy() {
    // Same setup as above but with the picked slot already active —
    // the C code decrements subtype2 back to 63 so the rate-limiter
    // keeps trying.
    let mut s = fresh_state();
    let k = 0;
    s.sprite_slot_view_mut(k).set_subtype2(63);
    let rand_peek = {
        let mut clone = s.clone();
        clone.get_random_number()
    };
    let pick = VITREOUS_MINION_ACTIVATION_SLOTS[(rand_peek & 15) as usize] as usize;
    s.sprite_slot_view_mut(pick).set_ai_state(2); // anything non-zero.
    s.vitreous_set_minions_forth(k);
    assert_eq!(s.sprite_slot_view(pick).ai_state(), 2);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 63);
}

#[test]
fn vitreous_damage_checkpoint_does_not_repeat_minion_selection() {
    for cadence in [63, 71] {
        for minion_state in [0, 2] {
            let mut state = fresh_state();
            state.set_main_module(7);
            state.set_submodule(0);
            state.set_indoor_flag(1);
            {
                let mut sprite = state.sprite_slot_view_mut(0);
                sprite.set_state(9);
                sprite.set_sprite_type(0xbd);
                sprite.set_subtype2(cadence);
                sprite.set_x(0x80);
                sprite.set_y(0x80);
                sprite.set_a(5);
                sprite.set_g(9);
            }
            for slot in 5..=13 {
                state.sprite_slot_view_mut(slot).set_ai_state(minion_state);
            }
            state.sprite_get16_bit_coords(0);
            let mut atomic = state.clone();
            let mut ai_checkpoint = state.clone();
            let mut player_checkpoint = state.clone();
            atomic.sprite_bd_vitreous(0);
            player_checkpoint.sprite_main_cpu_boundary =
                Some(SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot: 0 });
            player_checkpoint.sprite_bd_vitreous(0);
            player_checkpoint.sprite_main_cpu_boundary = None;
            player_checkpoint.vitreous_after_player_damage_checkpoint(0);
            assert_eq!(player_checkpoint.game_state, atomic.game_state);
            assert_eq!(player_checkpoint.ram, atomic.ram);
            ai_checkpoint.sprite_main_cpu_boundary =
                Some(SpriteMainCpuBoundary::VitreousAiPending { slot: 0 });
            ai_checkpoint.sprite_bd_vitreous(0);
            assert_eq!(ai_checkpoint.sprite_slot_view(0).a(), 5);
            ai_checkpoint.sprite_main_cpu_boundary = None;
            ai_checkpoint.vitreous_ai_after_damage(0);
            assert_eq!(ai_checkpoint.game_state, atomic.game_state);
            assert_eq!(ai_checkpoint.ram, atomic.ram);
            state.sprite_main_cpu_boundary =
                Some(SpriteMainCpuBoundary::VitreousDamagePending { slot: 0 });
            state.sprite_bd_vitreous(0);
            assert_eq!(state.sprite_slot_view(0).ai_state(), 0);
            assert_eq!(state.sprite_slot_view(0).a(), 5);
            let expected = if cadence == 63 && minion_state != 0 {
                63
            } else {
                cadence + 1
            };
            assert_eq!(state.sprite_slot_view(0).subtype2(), expected);
            state.sprite_main_cpu_boundary = None;
            state.vitreous_after_damage_checkpoint(0);
            assert_eq!(state.game_state, atomic.game_state);
            assert_eq!(state.ram, atomic.ram);
            assert_eq!(state.get_random_number(), atomic.get_random_number());
        }
    }
}

#[test]
fn generate_iceball_spawns_at_link_when_counter_wraps() {
    let mut s = fresh_state();
    let k = 1;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.follower_link_state_mut().set_x(0x0340);
    s.follower_link_state_mut().set_y(0x0450);
    s.sprite_slot_view_mut(k).set_subtype2(126);
    s.generate_iceball(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 127);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0);

    s.sprite_slot_view_mut(k).set_subtype2(127);
    s.generate_iceball(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 128);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xa4);
    assert_eq!(s.sprite_get_x(15), 0x0340);
    assert_eq!(s.sprite_get_y(15), 0x0450);
    assert_eq!(s.sprite_slot_view(15).z(), (-32i8) as u8);
    assert_eq!(s.sprite_slot_view(15).c(), (-32i8) as u8);
    assert_eq!(s.game_state.system_signals.sound_effect_1() & 0x3f, 0x20);
}

#[test]
fn ice_ball_split_spawns_four_shards_from_source_position() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_set_x(k, 0x0120);
    s.sprite_set_y(k, 0x0240);
    s.sprite_slot_view_mut(k).set_z(6);
    s.ice_ball_split(k);
    assert_eq!(s.game_state.system_signals.sound_effect_1() & 0x3f, 0x1f);
    assert_eq!(s.game_state.scratch_counter.value(), 0xff);

    let first_x = s.sprite_slot_view(15).x_velocity();
    let b = if first_x == (-32i8) as u8 {
        0usize
    } else {
        4usize
    };
    let xvel = [0i8, 32, 0, -32, 24, 24, -24, -24];
    let yvel = [-32i8, 0, 32, 0, -24, 24, -24, 24];
    for (slot, i) in [(15usize, 3usize), (14, 2), (13, 1), (12, 0)] {
        assert_eq!(s.sprite_slot_view(slot).sprite_type(), 0xa4);
        assert_eq!(s.sprite_get_x(slot), 0x0120);
        assert_eq!(s.sprite_get_y(slot), 0x0240);
        assert_eq!(s.sprite_slot_view(slot).z(), 6);
        assert_eq!(s.sprite_slot_view(slot).ai_state(), 1);
        assert_eq!(s.sprite_slot_view(slot).graphics(), 1);
        assert_eq!(s.sprite_slot_view(slot).c(), 1);
        assert_eq!(s.sprite_slot_view(slot).z_velocity(), 32);
        assert_eq!(s.sprite_slot_view(slot).x_velocity(), xvel[i + b] as u8);
        assert_eq!(s.sprite_slot_view(slot).y_velocity(), yvel[i + b] as u8);
        assert_eq!(s.sprite_slot_view(slot).flags4(), 0x1c);
    }
}

#[test]
fn red_bari_split_spawns_two_children_with_recoil_state() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_set_x(k, 0x0180);
    s.sprite_set_y(k, 0x0240);
    s.sprite_slot_view_mut(k).set_z(7);
    s.red_bari_split(k);

    assert_eq!(s.game_state.scratch_counter.value(), 0xff);
    for (slot, x, x_vel) in [(15usize, 0x0188u16, 32i8), (14, 0x0180, -32i8)] {
        assert_eq!(s.sprite_slot_view(slot).sprite_type(), 0x23);
        assert_eq!(s.sprite_slot_view(slot).state(), 9);
        assert_eq!(s.sprite_get_x(slot), x);
        assert_eq!(s.sprite_get_y(slot), 0x0240);
        assert_eq!(s.sprite_slot_view(slot).z(), 7);
        assert_eq!(s.sprite_slot_view(slot).flags3(), 0x33);
        assert_eq!(s.sprite_slot_view(slot).oam_flags(), 3);
        assert_eq!(s.sprite_slot_view(slot).flags4(), 1);
        assert_eq!(s.sprite_slot_view(slot).c(), 1);
        assert_eq!(s.sprite_slot_view(slot).x_velocity(), x_vel as u8);
        assert_eq!(s.sprite_slot_view(slot).delay_aux2(), 8);
        assert_eq!(s.sprite_slot_view(slot).delay_aux1(), 64);
    }
}

#[test]
fn sidenexx_exhale_danger_spawns_two_blue_fire_heads() {
    let mut s = fresh_state();
    let k = 1;
    s.sprite_slot_view_mut(k).set_sprite_type(0xcd);
    s.sprite_slot_view_mut(k).set_floor(3);
    s.sprite_set_x(k, 0x0108);
    s.sprite_set_y(k, 0x0210);
    s.sidenexx_exhale_danger(k);

    assert_eq!(s.ram[SMALL_BOSS_SHARED_WORK_A], 1);
    assert_eq!(s.game_state.system_signals.sound_effect_2() & 0x3f, 0x19);
    for (slot, c) in [(15usize, (-2i8) as u8), (14, 1u8)] {
        assert_eq!(s.sprite_slot_view(slot).sprite_type(), 0xcd);
        assert_eq!(s.sprite_slot_view(slot).state(), 9);
        assert_eq!(s.sprite_get_x(slot), 0x0108);
        assert_eq!(s.sprite_get_y(slot), 0x0210);
        assert_eq!(s.sprite_slot_view(slot).floor(), 3);
        assert_eq!(s.sprite_slot_view(slot).c(), c);
        assert_eq!(s.sprite_slot_view(slot).ignore_projectile(), 1);
        assert_eq!(s.sprite_slot_view(slot).e(), 1);
        assert_eq!(s.sprite_slot_view(slot).y_velocity(), 24);
        assert_eq!(s.sprite_slot_view(slot).flags2(), 0);
        assert_eq!(s.sprite_slot_view(slot).flags3(), 0x40);
    }
}

#[test]
fn sidenexx_exhale_danger_spawns_single_matching_red_head() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_slot_view_mut(k).set_sprite_type(0xcc);
    s.sprite_set_x(k, 0x0130);
    s.sprite_set_y(k, 0x0228);
    s.sprite_slot_view_mut(15).set_flags2(0xff);
    s.sidenexx_exhale_danger(k);

    assert_eq!(s.ram[SMALL_BOSS_SHARED_WORK_A], 0);
    assert_eq!(s.game_state.system_signals.sound_effect_1() & 0x3f, 0x2a);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xcc);
    assert_eq!(s.sprite_slot_view(15).state(), 9);
    assert_eq!(s.sprite_get_x(15), 0x0130);
    assert_eq!(s.sprite_get_y(15), 0x0228);
    assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 1);
    assert_eq!(s.sprite_slot_view(15).e(), 1);
    assert_eq!(s.sprite_slot_view(15).y_velocity(), 24);
    assert_eq!(s.sprite_slot_view(15).flags2(), 0);
    assert_eq!(s.sprite_slot_view(15).flags3(), 0x40);
    assert_eq!(s.sprite_slot_view(14).sprite_type(), 0);
}

#[test]
fn spike_block_check_statue_collision_filters_by_frame_parity_and_overlap() {
    let mut s = fresh_state();
    let spike = 2;
    let statue_even = 4;
    let statue_odd = 5;
    s.set_frame_counter(0);
    s.sprite_set_x(spike, 0x0100);
    s.sprite_set_y(spike, 0x0200);

    s.sprite_slot_view_mut(statue_even).set_state(9);
    s.sprite_slot_view_mut(statue_even).set_sprite_type(0x1c);
    s.sprite_set_x(statue_even, 0x010f);
    s.sprite_set_y(statue_even, 0x020f);
    assert!(!s.spike_block_check_statue_collision(spike));

    s.sprite_set_x(statue_even, 0x0120);
    assert!(s.spike_block_check_statue_collision(spike));

    s.sprite_slot_view_mut(statue_odd).set_state(9);
    s.sprite_slot_view_mut(statue_odd).set_sprite_type(0x1c);
    s.sprite_set_x(statue_odd, 0x0100);
    s.sprite_set_y(statue_odd, 0x0200);
    assert!(s.spike_block_check_statue_collision(spike));

    s.set_frame_counter(1);
    assert!(!s.spike_block_check_statue_collision(spike));
}

#[test]
fn yellow_stalfos_animate_maps_d_to_gfx2() {
    // YellowStalfos_Animate: graphics = kYellowStalfos_Gfx2[D];
    // flags3 has bit 0x40 cleared.
    let mut s = fresh_state();
    let k = 2;
    s.sprite_slot_view_mut(k).set_flags3(0xff);
    for d in 0u8..4 {
        s.sprite_slot_view_mut(k).set_direction(d);
        s.yellow_stalfos_animate(k);
        assert_eq!(
            s.sprite_slot_view(k).graphics(),
            YELLOW_STALFOS_IDLE_GRAPHICS_BY_DIRECTION[d as usize]
        );
        assert_eq!(s.sprite_slot_view(k).flags3() & 0x40, 0);
        // The non-0x40 bits should remain set.
        assert_eq!(s.sprite_slot_view(k).flags3(), 0xff & !0x40);
        // Restore for the next iteration.
        s.sprite_slot_view_mut(k).set_flags3(0xff);
    }
}
