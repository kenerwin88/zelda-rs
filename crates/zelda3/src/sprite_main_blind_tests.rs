use super::*;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

#[test]
fn prep_blind_sets_battle_fields_when_unlocked() {
    let mut s = fresh_state();
    s.follower_state_mut().set_indicator(0);
    s.dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x2000);
    // Mark our slot active and clear other slots.
    s.sprite_prep_blind_prepare_battle(3);
    let sprite = s.sprite_slot_view(3);
    assert_eq!(sprite.delay_aux2(), 96);
    assert_eq!(sprite.c(), 1);
    assert_eq!(sprite.direction(), 2);
    assert_eq!(sprite.head_direction(), 4);
    assert_eq!(sprite.graphics(), 7);
    assert_eq!(s.ram[BLIND_HEAD_ANIM_COUNTER], 0);
}

#[test]
fn prep_blind_kills_sprite_when_locked() {
    let mut s = fresh_state();
    s.follower_state_mut().set_indicator(6); // wrong indicator -> branch to else
    s.sprite_slot_view_mut(5).set_state(9);
    s.sprite_prep_blind_prepare_battle(5);
    assert_eq!(s.sprite_slot_view(5).state(), 0);
}

#[test]
fn blind_spit_fireball_returns_minus_one_when_subtype2_masks() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(2).set_subtype2(0xff);
    let r = s.blind_spit_fireball(2, 0x1f);
    assert_eq!(r, -1);
}

#[test]
fn blind_spit_fireball_writes_velocity_table() {
    let mut s = fresh_state();
    // Zero all sprite states so allocation can succeed (slot 13 picked).
    {
        let mut sprite = s.sprite_slot_view_mut(0);
        sprite.set_subtype2(0);
        sprite.set_head_direction(8); // xvel=32, yvel=0
    }
    let r = s.blind_spit_fireball(0, 0xf);
    assert!(r >= 0, "expected fireball spawn slot, got {r}");
    let j = r as usize;
    let fireball = s.sprite_slot_view(j);
    assert_eq!(fireball.x_velocity(), 32);
    assert_eq!(fireball.y_velocity(), 0);
    assert_eq!(fireball.deflection_bits() & 8, 8);
    assert_eq!(fireball.bump_damage(), 4);
}

#[test]
fn blind_decelerate_x_brings_velocity_toward_zero() {
    let mut s = fresh_state();
    // Negative velocity -> add +2.
    {
        let mut sprite = s.sprite_slot_view_mut(4);
        sprite.set_x_velocity((-5i8) as u8);
        sprite.set_wall_collision(0); // suppress flurry branch
    }
    s.blind_decelerate_x(4);
    assert_eq!(s.sprite_slot_view(4).x_velocity() as i8, -3);

    // Positive velocity -> subtract 2.
    s.sprite_slot_view_mut(4).set_x_velocity(7);
    s.blind_decelerate_x(4);
    assert_eq!(s.sprite_slot_view(4).x_velocity(), 5);

    // Zero velocity stays zero.
    s.sprite_slot_view_mut(4).set_x_velocity(0);
    s.blind_decelerate_x(4);
    assert_eq!(s.sprite_slot_view(4).x_velocity(), 0);
}

#[test]
fn blind_animate_picks_head_dir_from_table() {
    let mut s = fresh_state();
    {
        let mut sprite = s.sprite_slot_view_mut(1);
        sprite.set_wall_collision(0);
        sprite.set_direction(2); // t0 = 0, no negation
    }
    s.follower_link_state_mut().set_x(0); // tab idx 0 -> t1 = 0
    s.sprite_system_mut().set_blind_head_anim_counter(0); // idx 0 -> table[0] = 0
    s.blind_animate(1);
    assert_eq!(s.sprite_slot_view(1).head_direction(), 0);

    // BLIND_HEAD_ANIM_COUNTER=8 -> (8>>3 & 7)=1, (8>>2 & 1)=0, idx=1 -> table[1] = 1
    s.sprite_system_mut().set_blind_head_anim_counter(8);
    s.blind_animate(1);
    assert_eq!(s.sprite_slot_view(1).head_direction(), 1);
}
