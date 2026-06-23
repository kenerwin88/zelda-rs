use super::*;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

fn make_active(state: &mut ZeldaState, k: usize) {
    // Configure the slot so `sprite_return_if_inactive_for_mothula`
    // returns false: state=9, no flags, no pause via defl bit 0x80.
    state.sprite_slot_view_mut(k).set_state(9);
    state.clear_modal_pause_flag();
    state.set_submodule(0);
    state.sprite_slot_view_mut(k).set_deflection_bits(0x80);
    state.sprite_slot_view_mut(k).set_pause(0);
    state.sprite_slot_view_mut(k).set_hit_timer(0);
}

#[test]
fn flap_wings_advances_subtype2_and_picks_gfx() {
    // Mothula_FlapWings: pre-increments sprite_subtype2[k], picks
    // gfx from kMothula_FlapWingsGfx[(subtype2 >> 2) & 3].
    let mut s = fresh_state();
    let k = 3;

    // subtype2 starts at 0 -> after ++ = 1, j = (1 >> 2) & 3 = 0
    // -> sfx queued, gfx = kMothula_FlapWingsGfx[0] = 0.
    s.sprite_slot_view_mut(k).set_subtype2(0);
    s.mothula_flap_wings(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 1);
    assert_eq!(s.sprite_slot_view(k).graphics(), 0);

    // subtype2 = 3 -> ++ = 4 -> j = 1 -> gfx = 1.
    s.sprite_slot_view_mut(k).set_subtype2(3);
    s.mothula_flap_wings(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 4);
    assert_eq!(s.sprite_slot_view(k).graphics(), 1);

    // subtype2 = 7 -> ++ = 8 -> j = 2 -> gfx = 2.
    s.sprite_slot_view_mut(k).set_subtype2(7);
    s.mothula_flap_wings(k);
    assert_eq!(s.sprite_slot_view(k).graphics(), 2);

    // subtype2 = 11 -> ++ = 12 -> j = 3 -> gfx = 1.
    s.sprite_slot_view_mut(k).set_subtype2(11);
    s.mothula_flap_wings(k);
    assert_eq!(s.sprite_slot_view(k).graphics(), 1);
}

#[test]
fn spawn_beams_sets_tmp_counter_and_beam_state() {
    // Mothula_SpawnBeams writes 0xff into tmp_counter and, for each
    // spawned slot, populates the per-beam velocity/x_lo/z/delay
    // fields. With an empty pool of sprite slots, three slots are
    // taken from the back of the array.
    let mut s = fresh_state();
    let k = 0;
    // Canonical Sprite_SpawnDynamically reads info.r0_x from
    // sprite_x_lo[k] | sprite_x_hi[k] << 8 (via Sprite_GetX). Seed the
    // sprite's per-slot position so r0_x / r2_y land at 0x80 / 0x50.
    s.sprite_slot_view_mut(k).set_x_low(0x80);
    s.sprite_slot_view_mut(k).set_x_high(0x00);
    s.sprite_slot_view_mut(k).set_y_low(0x50);
    s.sprite_slot_view_mut(k).set_y_high(0x00);
    s.sprite_slot_view_mut(k).set_z(0);

    s.mothula_spawn_beams(k);
    assert_eq!(s.game_state.scratch_counter.value(), 0xff);

    // The reverse-allocator hands out 15, 14, 13 across the loop.
    // First iter (i = 2): x_vel = 16, y_vel = 24, x_lo = 0x80 + 16.
    assert_eq!(s.sprite_slot_view(15).x_velocity() as i8, 16);
    assert_eq!(s.sprite_slot_view(15).y_velocity(), 24);
    assert_eq!(s.sprite_slot_view(15).x_low(), 0x80u8.wrapping_add(16));
    assert_eq!(s.sprite_slot_view(15).delay_main(), 16);
    assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 16);
    assert_eq!(s.sprite_slot_view(15).z(), 0);
    // Second iter (i = 1): x_vel = 0, y_vel = 32.
    assert_eq!(s.sprite_slot_view(14).x_velocity(), 0);
    assert_eq!(s.sprite_slot_view(14).y_velocity(), 32);
    // Third iter (i = 0): x_vel = -16 (=> 0xf0), y_vel = 24.
    assert_eq!(s.sprite_slot_view(13).x_velocity() as i8, -16);
    assert_eq!(s.sprite_slot_view(13).y_velocity(), 24);
}

#[test]
fn main_transitions_delay_to_ascend() {
    // ai_state = 0, sprite_delay_main = 0 -> ai_state becomes 1.
    let mut s = fresh_state();
    let k = 4;
    make_active(&mut s, k);
    s.sprite_slot_view_mut(k).set_ai_state(0);
    s.sprite_slot_view_mut(k).set_delay_main(0);
    s.sprite_slot_view_mut(k).set_f(0);
    s.mothula_main(k);
    assert_eq!(s.sprite_slot_view(k).ai_state(), 1);
    assert_eq!(s.sprite_slot_view(k).flags3(), 0);
}

#[test]
fn main_flag_f6_arms_phase2() {
    // sprite_F & 127 == 6 forces F=0, delay_aux3=32, ai_state=2,
    // delay_main=0, G=64. After that the case-2 branch fires
    // (because we are mid-call): G is decremented to 63, the
    // flap-wings + z-vel maths run, and since delay_main==0 the
    // "++C == 7" else-branch overwrites x_vel/y_vel/delay_main.
    let mut s = fresh_state();
    let k = 2;
    make_active(&mut s, k);
    s.sprite_slot_view_mut(k).set_f(6);
    s.sprite_slot_view_mut(k).set_ai_state(3);
    s.sprite_slot_view_mut(k).set_delay_main(5);
    s.sprite_slot_view_mut(k).set_g(0);
    s.mothula_main(k);
    assert_eq!(s.sprite_slot_view(k).f(), 0);
    assert_eq!(s.sprite_slot_view(k).delay_aux3(), 32);
    assert_eq!(s.sprite_slot_view(k).ai_state(), 2);
    // case 2 ran: G decremented from 64 to 63.
    assert_eq!(s.sprite_slot_view(k).g(), 63);
    // sprite_flags3 is evaluated BEFORE the F=6 branch sets
    // delay_aux3, so it stays 0 here.
    assert_eq!(s.sprite_slot_view(k).flags3(), 0);
}

#[test]
fn main_state11_resets_ai_state() {
    // sprite_state[k] == 11 forces ai_state -> 0, then the active
    // check trips on state != 9 and exits early.
    let mut s = fresh_state();
    let k = 5;
    s.sprite_slot_view_mut(k).set_state(11);
    s.sprite_slot_view_mut(k).set_ai_state(3);
    s.mothula_main(k);
    assert_eq!(s.sprite_slot_view(k).ai_state(), 0);
    // Inactive exit means flags3 wasn't reset.
    assert_eq!(s.sprite_slot_view(k).flags3(), 0);
}

#[test]
fn handle_spikes_decrements_and_returns_early() {
    // First call: sprite_head_dir is decremented; non-zero -> early
    // return (no allocation occurs).
    let mut s = fresh_state();
    let k = 1;
    s.sprite_slot_view_mut(k).set_head_direction(3);
    // Pre-fill a slot so we can prove no allocation happened.
    for j in 0..16 {
        s.sprite_slot_view_mut(j).set_state(9);
    }
    s.mothula_handle_spikes(k);
    assert_eq!(s.sprite_slot_view(k).head_direction(), 2);
    // No slot freed -> SPAWN cannot succeed even if it tried.
    for j in 0..16 {
        assert_eq!(s.sprite_slot_view(j).state(), 9);
    }
}

#[test]
fn handle_spikes_arms_when_decrement_hits_zero() {
    // sprite_head_dir = 1 -> after decrement = 0 -> reload to 0x40
    // and try to spawn. Spawn succeeds (one slot free); spike
    // tables populate the target slot.
    let mut s = fresh_state();
    let k = 0;
    s.sprite_slot_view_mut(k).set_head_direction(1);
    // Mark all slots active except 15, so allocator picks 15.
    for j in 0..15 {
        s.sprite_slot_view_mut(j).set_state(9);
    }
    s.sprite_slot_view_mut(15).set_state(0);
    // Force the random number deterministically by seeding RNG via
    // calling get_random_number isn't an option here, but the
    // table lookup with whatever index is produced just needs to be
    // valid; we assert side-effects independent of index.
    s.sprite_workspace_mut().set_room_origin_x_high(0x10);
    s.sprite_workspace_mut().set_room_origin_y_high(0x20);
    s.garnish_state_mut().set_sprcoll_x_size(0xffff);
    s.garnish_state_mut().set_sprcoll_y_size(0xffff);

    s.mothula_handle_spikes(k);
    // head_dir was set to 0x40 before the spawn path. If the spawn
    // succeeded and tile-collision found no wall, head_dir is reset
    // to 1; this fixture leaves the spawned spike away from walls, so
    // wallcoll stays 0 and the final branch fires:
    assert_eq!(s.sprite_slot_view(k).head_direction(), 1);
    // Slot 15 was claimed and then re-zeroed by the wallcoll==0
    // branch. So sprite_state[15] == 0 again.
    assert_eq!(s.sprite_slot_view(15).state(), 0);
    // x_hi / y_hi reflect the current sprite-room origin plus 1.
    assert_eq!(s.sprite_slot_view(15).x_high(), 0x11);
    assert_eq!(s.sprite_slot_view(15).y_high(), 0x21);
    // x_vel was zeroed after the collision check.
    assert_eq!(s.sprite_slot_view(15).x_velocity(), 0);
}
