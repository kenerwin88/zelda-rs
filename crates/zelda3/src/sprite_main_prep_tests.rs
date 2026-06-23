use super::*;
use crate::types::{read_le_u16, write_le_u16};

fn fresh_state() -> Box<ZeldaState> {
    Box::new(ZeldaState::new())
}

#[test]
fn simple_sprite_prep_offsets_and_flags_match_c() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_slot_view_mut(k).set_x_low(0xf9);
    s.sprite_slot_view_mut(k).set_y_low(0xfb);
    s.sprite_prep_mantle(k);
    assert_eq!(s.sprite_slot_view(k).x_low(), 1);
    assert_eq!(s.sprite_slot_view(k).y_low(), 0xfe);

    s.sprite_prep_move_down_8px_right8px(k);
    assert_eq!(s.sprite_slot_view(k).x_low(), 9);
    assert_eq!(s.sprite_slot_view(k).y_low(), 6);

    s.sprite_slot_view_mut(k).set_ignore_projectile(0xff);
    s.sprite_prep_ignore_projectiles(k);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 0);
}

#[test]
fn dark_world_enemy_prep_uses_second_property_row() {
    let mut s = fresh_state();
    let k = 3;
    s.set_dark_world_region_index(1);
    s.sprite_prep_keese(k);
    assert_eq!(s.sprite_slot_view(k).bump_damage(), 0x85);
    assert_eq!(s.sprite_slot_view(k).health(), 4);
    assert_eq!(s.sprite_slot_view(k).flags5(), 7);

    s.sprite_prep_rope(k);
    assert_eq!(s.sprite_slot_view(k).bump_damage(), 5);
    assert_eq!(s.sprite_slot_view(k).health(), 8);
    assert_eq!(s.sprite_slot_view(k).flags5(), 7);
}

#[test]
fn position_snapshot_prep_copies_low_high_coords() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_slot_view_mut(k).set_x_low(0x12);
    s.sprite_slot_view_mut(k).set_x_high(0x01);
    s.sprite_slot_view_mut(k).set_y_low(0x34);
    s.sprite_slot_view_mut(k).set_y_high(0x02);
    s.sprite_prep_kyameron(k);
    assert_eq!(s.sprite_slot_view(k).a(), 0x12);
    assert_eq!(s.sprite_slot_view(k).b(), 0x01);
    assert_eq!(s.sprite_slot_view(k).c(), 0x34);
    assert_eq!(s.sprite_slot_view(k).head_direction(), 0x02);
}

#[test]
fn key_prep_consumes_item_drop_counter() {
    let mut s = fresh_state();
    let k = 5;
    s.sprite_battle_mut().set_item_drop_counter(0x7e);
    s.sprite_prep_small_key(k);
    assert_eq!(s.sprite_slot_view(k).subtype(), 0xff);
    assert_eq!(s.sprite_slot_view(k).die_action(), 0x7e);
    assert_eq!(s.ram[ITEM_DROP_COUNTER], 0x7f);

    s.sprite_prep_key_set_item_drop(k);
    assert_eq!(s.sprite_slot_view(k).die_action(), 0x7f);
    assert_eq!(s.ram[ITEM_DROP_COUNTER], 0x80);
}

#[test]
fn flute_kid_prep_handles_light_and_dark_world_branches() {
    let mut light = fresh_state();
    let k = 6;
    light.inventory_items_mut().set_flute(2);
    light.sprite_slot_view_mut(k).set_state(9);
    light.sprite_prep_flute_kid(k);
    assert_eq!(light.sprite_slot_view(k).state(), 0);

    let mut dark = fresh_state();
    dark.save_progress_mut().set_dark_world_state(0x40);
    dark.save_progress_mut().set_progress_indicator_3(8);
    dark.sprite_slot_view_mut(k).set_x_low(10);
    dark.sprite_slot_view_mut(k).set_y_low(20);
    dark.sprite_prep_flute_kid(k);
    assert_eq!(dark.sprite_slot_view(k).subtype2(), 1);
    assert_eq!(dark.sprite_slot_view(k).graphics(), 3);
    assert_eq!(dark.sprite_slot_view(k).ai_state(), 5);
    assert_eq!(dark.sprite_slot_view(k).x_low(), 18);
    assert_eq!(dark.sprite_slot_view(k).y_low(), 12);
}

#[test]
fn return_if_boss_finished_clears_non_boss_sprites_or_self_when_finished() {
    let mut s = fresh_state();
    for k in 0..16 {
        s.sprite_slot_view_mut(k).set_state(9);
        s.sprite_slot_view_mut(k).set_sprite_type(0);
    }
    s.sprite_slot_view_mut(3).set_sprite_type(9); // bump damage 0x13 keeps state.
    assert!(!s.sprite_return_if_boss_finished(2));
    assert_eq!(s.sprite_slot_view(0).state(), 0);
    assert_eq!(s.sprite_slot_view(3).state(), 9);

    let mut finished = fresh_state();
    finished.sprite_slot_view_mut(2).set_state(9);
    finished
        .dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x8000);
    assert!(finished.sprite_return_if_boss_finished(2));
    assert_eq!(finished.sprite_slot_view(2).state(), 0);
}

#[test]
fn room_lookup_prep_sets_subtype_and_ignore_projectile() {
    let mut s = fresh_state();
    let k = 7;
    s.set_dungeon_room_index(0x12);
    s.sprite_prep_storyteller(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 2);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

    s.set_dungeon_room_index(0x03);
    s.sprite_slot_view_mut(k).set_ignore_projectile(0);
    s.sprite_prep_adults(k);
    assert_eq!(s.sprite_slot_view(k).subtype2(), 0);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
}

#[test]
fn rupee_pull_and_grate_shift_x_16_bit_left() {
    let mut s = fresh_state();
    let k = 8;
    s.sprite_set_x(k, 0x0104);
    s.sprite_prep_rupee_pull(k);
    assert_eq!(s.sprite_get_x(k), 0x00fc);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

    s.sprite_set_x(k, 0x0004);
    s.set_overworld_event_info(0x58, 0x20);
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_prep_thieves_town_grate(k);
    assert_eq!(s.sprite_get_x(k), 0xfffc);
    assert_eq!(s.sprite_slot_view(k).state(), 0);
}

#[test]
fn boss_gated_prep_sets_expected_state_when_unfinished() {
    let mut s = fresh_state();
    let k = 10;
    s.sprite_slot_view_mut(k).set_x_low(0x20);
    s.sprite_slot_view_mut(k).set_y_low(0x30);
    s.set_dark_world_region_index(1);
    s.sprite_prep_agahnim(k);
    assert_eq!(s.sprite_slot_view(k).graphics(), 0);
    assert_eq!(s.sprite_slot_view(k).direction(), 3);
    assert_eq!(s.sprite_slot_view(k).oam_flags(), 7);
    assert_eq!(s.sprite_slot_view(k).x_low(), 0x28);
    assert_eq!(s.sprite_slot_view(k).y_low(), 0x38);

    s.sprite_prep_kholdstare(k);
    assert_eq!(s.sprite_slot_view(k).ai_state(), 3);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
}

#[test]
fn armos_desert_and_big_spike_prep_update_state() {
    let mut s = fresh_state();
    let k = 11;
    s.sprite_slot_view_mut(k).set_x_low(0x2f);
    s.sprite_slot_view_mut(k).set_y_low(0x40);
    s.sprite_system_mut().set_limit_instance(5);
    s.sprite_prep_desert_statue(k);
    assert_eq!(s.sprite_slot_view(k).a(), 5);
    assert_eq!(s.game_state.sprites.system.limit_instance(), 6);
    assert_eq!(s.sprite_slot_view(k).direction(), 3); // after +8, x is now 0x37.

    s.sprite_prep_armos_knight(k);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 255);
    assert_eq!(s.game_state.sprites.workspace.prep_shared_counter(), 1);

    s.sprite_slot_view_mut(k).set_x_low(0x10);
    s.sprite_slot_view_mut(k).set_x_high(1);
    s.sprite_slot_view_mut(k).set_y_low(0x20);
    s.sprite_slot_view_mut(k).set_y_high(2);
    s.sprite_prep_big_spike(k);
    assert_eq!(s.sprite_slot_view(k).a(), 0x18);
    assert_eq!(s.sprite_slot_view(k).b(), 1);
    assert_eq!(s.sprite_slot_view(k).c(), 0x28);
    assert_eq!(s.sprite_slot_view(k).head_direction(), 2);
}

#[test]
fn barrier_catfish_and_mini_vitreous_prep_match_simple_branches() {
    let mut s = fresh_state();
    let k = 12;
    s.set_overworld_screen(5);
    s.set_overworld_event_info(5, 0x40);
    s.sprite_slot_view_mut(k).set_x_low(0x10);
    s.sprite_slot_view_mut(k).set_y_low(0x20);
    s.sprite_prep_agahnims_barrier(k);
    assert_eq!(s.sprite_slot_view(k).graphics(), 4);
    assert_eq!(s.sprite_slot_view(k).x_low(), 0x18);
    assert_eq!(s.sprite_slot_view(k).y_low(), 0x1c);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

    s.sprite_slot_view_mut(k).set_x_low(0x30);
    s.sprite_slot_view_mut(k).set_y_low(0x40);
    s.sprite_slot_view_mut(k).set_ignore_projectile(0);
    s.sprite_prep_catfish(k);
    assert_eq!(s.sprite_slot_view(k).x_low(), 0x38);
    assert_eq!(s.sprite_slot_view(k).y_low(), 0x3c);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

    s.sprite_slot_view_mut(k).set_state(9);
    s.dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x8000);
    s.sprite_prep_mini_vitreous(k);
    assert_eq!(s.sprite_slot_view(k).state(), 0);

    let mut cutscene = fresh_state();
    cutscene.sprite_slot_view_mut(k).set_state(9);
    cutscene.sprite_set_x(k, 0x0100);
    cutscene.sprite_set_y(k, 0x0200);
    cutscene.sprite_prep_cutscene_agahnim(k);
    assert_eq!(cutscene.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(cutscene.sprite_get_x(k), 0x0108);
    assert_eq!(cutscene.sprite_get_y(k), 0x0206);
    assert_eq!(cutscene.sprite_slot_view(15).sprite_type(), 0xc1);
    assert_eq!(cutscene.sprite_slot_view(15).a(), 1);
    assert_eq!(cutscene.sprite_slot_view(15).ignore_projectile(), 1);
    assert_eq!(cutscene.sprite_get_x(15), 0x0108);
    assert_eq!(cutscene.sprite_slot_view(15).y_high(), 0x02);
    assert_eq!(cutscene.sprite_slot_view(15).y_low(), 0x2e);
    assert_eq!(cutscene.sprite_slot_view(15).flags2(), 0);
    assert_eq!(cutscene.sprite_slot_view(15).oam_flags(), 12);

    let mut cutscene_done = fresh_state();
    cutscene_done.sprite_slot_view_mut(k).set_state(9);
    cutscene_done
        .dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x4000);
    cutscene_done.sprite_prep_cutscene_agahnim(k);
    assert_eq!(cutscene_done.sprite_slot_view(k).state(), 0);
    assert_eq!(cutscene_done.sprite_slot_view(15).state(), 0);
}

#[test]
fn ganon_helmasaur_and_trinexx_prep_call_existing_initializers() {
    let mut ganon = fresh_state();
    let k = 13;
    ganon.sprite_slot_view_mut(k).set_direction(1);
    ganon.sprite_prep_ganon(k);
    assert_eq!(ganon.sprite_slot_view(k).delay_main(), 128);
    assert_eq!(ganon.sprite_slot_view(k).room(), 2);
    assert_eq!(ganon.game_state.system_signals.music_control(), 0x1e);

    let mut helmasaur = fresh_state();
    for i in 0..16 {
        helmasaur.ram[ALT_SPRITE_STATE_PREP + i] = 0xff;
    }
    helmasaur.sprite_prep_helmasaur_king(k);
    assert_eq!(
        &helmasaur.ram[ALT_SPRITE_STATE_PREP..ALT_SPRITE_STATE_PREP + 16],
        &[0; 16]
    );

    let mut trinexx_body = fresh_state();
    let k = 5;
    trinexx_body.sprite_slot_view_mut(k).set_sprite_type(0xcb);
    trinexx_body.sprite_slot_view_mut(k).set_x_low(0x20);
    trinexx_body.sprite_slot_view_mut(k).set_x_high(1);
    trinexx_body.sprite_slot_view_mut(k).set_y_low(0x30);
    trinexx_body.sprite_slot_view_mut(k).set_y_high(2);
    trinexx_body.overlord_slot_view_mut(0).set_y_high(0x0c);
    trinexx_body.overlord_slot_view_mut(0).set_gen2(0x97);
    trinexx_body.overlord_slot_view_mut(0).set_floor(0x01);
    trinexx_body.ram[ALT_SPRITE_STATE_PREP + 3] = 0xaa;
    trinexx_body.sprite_prep_trinexx(k);
    assert_eq!(trinexx_body.sprite_slot_view(k).a(), 0x28);
    assert_eq!(trinexx_body.sprite_slot_view(k).b(), 1);
    assert_eq!(trinexx_body.sprite_slot_view(k).c(), 0x40);
    assert_eq!(trinexx_body.sprite_slot_view(k).g(), 2);
    assert_eq!(trinexx_body.sprite_get_x(k), 0x0128);
    assert_eq!(trinexx_body.sprite_get_y(k), 0x024c);
    assert_eq!(
        trinexx_body
            .game_state
            .sprites
            .overlord_slots
            .slot(2)
            .x_low(),
        0
    );
    assert_eq!(
        trinexx_body
            .game_state
            .sprites
            .overlord_slots
            .slot(6)
            .x_low(),
        255
    );
    assert_eq!(trinexx_body.ram[OVERLORD_X_HI_PREP], 0);
    assert_eq!(trinexx_body.ram[OVERLORD_Y_HI_PREP], 0x0c);
    assert_eq!(trinexx_body.ram[OVERLORD_GEN2_PREP], 0x97);
    assert_eq!(trinexx_body.ram[OVERLORD_FLOOR_PREP], 0x01);
    assert_eq!(trinexx_body.ram[ALT_SPRITE_STATE_PREP + 3], 0);

    let mut trinexx_head = fresh_state();
    trinexx_head.sprite_slot_view_mut(k).set_sprite_type(0xcc);
    trinexx_head.sprite_slot_view_mut(k).set_x_low(0x44);
    trinexx_head.sprite_slot_view_mut(k).set_x_high(3);
    trinexx_head.sprite_slot_view_mut(k).set_y_low(0x55);
    trinexx_head.sprite_slot_view_mut(k).set_y_high(4);
    trinexx_head.ram[ALT_SPRITE_TYPE_PREP + 0x1a] = 0;
    trinexx_head.ram[ALT_SPRITE_X_HI_PREP + 0x1a] = 0xff;
    trinexx_head.ram[ALT_SPRITE_Y_HI_PREP + 0x1a] = 0xff;
    trinexx_head.sprite_prep_trinexx(k);
    assert_eq!(trinexx_head.sprite_slot_view(k).graphics(), 3);
    assert_eq!(trinexx_head.sprite_slot_view(k).delay_main(), 128);
    assert_eq!(trinexx_head.sprite_slot_view(k).subtype2(), 1);
    assert_eq!(trinexx_head.ram[ALT_SPRITE_TYPE_PREP + 0x1a], 0x40);
    assert_eq!(trinexx_head.ram[ALT_SPRITE_X_HI_PREP + 0x1a], 0);
    assert_eq!(trinexx_head.ram[ALT_SPRITE_Y_HI_PREP + 0x1a], 0);
    assert_eq!(trinexx_head.sprite_slot_view(k).a(), 0x44);
    assert_eq!(trinexx_head.sprite_slot_view(k).g(), 4);
}

#[test]
fn moldorm_and_chainchomp_history_buffers_are_seeded_from_sprite_position() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_slot_view_mut(k).set_x_low(0x44);
    s.sprite_slot_view_mut(k).set_x_high(0x01);
    s.sprite_slot_view_mut(k).set_y_low(0x55);
    s.sprite_slot_view_mut(k).set_y_high(0x02);
    s.sprite_prep_mini_moldorm_bounce(k);
    let base = 32 * k;
    assert_eq!(s.ram[MOLDORM_X_LO_PREP + base], 0x44);
    assert_eq!(s.ram[MOLDORM_X_HI_PREP + base + 31], 0x01);
    assert_eq!(s.ram[MOLDORM_Y_LO_PREP + base + 15], 0x55);
    assert_eq!(s.ram[MOLDORM_Y_HI_PREP + base + 31], 0x02);

    s.sprite_workspace_mut().set_current_sprite_x(0x1234);
    s.sprite_workspace_mut().set_current_sprite_y(0x5678);
    s.sprite_prep_chainchomp_bounce(k);
    let hist = k * 8;
    assert_eq!(
        read_le_u16(&s.ram, CHAINCHOMP_X_HIST_PREP + hist * 2),
        0x1234
    );
    assert_eq!(
        read_le_u16(&s.ram, CHAINCHOMP_Y_HIST_PREP + (hist + 5) * 2),
        0x5678
    );
    assert_eq!(s.sprite_slot_view(k).a(), 0x44);
    assert_eq!(s.sprite_slot_view(k).g(), 0x02);

    let mut leash = fresh_state();
    leash.sprite_workspace_mut().set_current_sprite_x(0x0100);
    leash.sprite_workspace_mut().set_current_sprite_y(0x0200);
    leash.chain_chomp_history_mut().set_x(hist + 1, 0x0120);
    leash.chain_chomp_history_mut().set_y(hist + 1, 0x01e0);
    leash.chain_chomp_handle_leash(k);
    assert_eq!(
        read_le_u16(&leash.ram, CHAINCHOMP_X_HIST_PREP + hist * 2),
        0x0100
    );
    assert_eq!(
        read_le_u16(&leash.ram, CHAINCHOMP_Y_HIST_PREP + hist * 2),
        0x0200
    );
    assert_eq!(
        read_le_u16(&leash.ram, CHAINCHOMP_X_HIST_PREP + (hist + 1) * 2),
        0x0108
    );
    assert_eq!(
        read_le_u16(&leash.ram, CHAINCHOMP_Y_HIST_PREP + (hist + 1) * 2),
        0x01f8
    );

    let mut moving_chain = fresh_state();
    moving_chain.sprite_slot_view_mut(k).set_a(0x00);
    moving_chain.sprite_slot_view_mut(k).set_b(0x01);
    moving_chain.sprite_slot_view_mut(k).set_c(0x00);
    moving_chain.sprite_slot_view_mut(k).set_g(0x02);
    moving_chain.chain_chomp_history_mut().set_x(hist, 0x0110);
    moving_chain.chain_chomp_history_mut().set_y(hist, 0x0220);
    moving_chain
        .chain_chomp_history_mut()
        .set_x(hist + 1, 0x0100);
    moving_chain
        .chain_chomp_history_mut()
        .set_y(hist + 1, 0x0230);
    moving_chain.chain_chomp_move_chain(k);
    assert_eq!(
        read_le_u16(&moving_chain.ram, CHAINCHOMP_X_HIST_PREP + (hist + 1) * 2),
        0x0101
    );
    assert_eq!(
        read_le_u16(&moving_chain.ram, CHAINCHOMP_Y_HIST_PREP + (hist + 1) * 2),
        0x022f
    );
}

#[test]
fn bonk_big_key_and_purple_chest_prep_match_state_gates() {
    let mut outdoor = fresh_state();
    let k = 3;
    outdoor.sprite_prep_bonk_item(k);
    assert_eq!(outdoor.sprite_slot_view(k).graphics(), 2);

    let mut indoor = fresh_state();
    indoor.set_indoor_flag(1);
    indoor.sprite_battle_mut().set_item_drop_counter(1);
    indoor.sprite_slot_view_mut(k).set_graphics(4);
    indoor
        .dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x2000);
    indoor.sprite_slot_view_mut(k).set_state(9);
    indoor.sprite_prep_bonk_item(k);
    assert_eq!(indoor.sprite_slot_view(k).floor(), 2);
    assert_eq!(indoor.sprite_slot_view(k).die_action(), 1);
    assert_eq!(indoor.sprite_slot_view(k).state(), 0);
    assert_eq!(indoor.sprite_slot_view(k).graphics(), 5);
    assert_eq!(indoor.sprite_slot_view(k).oam_flags(), 8);
    assert_eq!(indoor.sprite_slot_view(k).flags3() & 0x20, 0x20);

    let mut key = fresh_state();
    key.sprite_slot_view_mut(k).set_x_low(0x20);
    key.sprite_battle_mut().set_item_drop_counter(7);
    key.sprite_prep_big_key(k);
    assert_eq!(key.sprite_slot_view(k).x_low(), 0x28);
    assert_eq!(key.sprite_slot_view(k).subtype(), 0xff);
    assert_eq!(key.sprite_slot_view(k).die_action(), 7);
    assert_eq!(key.ram[ITEM_DROP_COUNTER], 8);

    let mut chest = fresh_state();
    chest.save_progress_mut().set_progress_indicator_3(32);
    chest.sprite_prep_purple_chest(k);
    assert_eq!(chest.sprite_slot_view(k).ignore_projectile(), 1);
    chest.follower_state_mut().set_indicator(12);
    chest.sprite_slot_view_mut(k).set_state(9);
    chest.sprite_prep_purple_chest(k);
    assert_eq!(chest.sprite_slot_view(k).state(), 0);
}

#[test]
fn smithy_prep_matches_world_and_progress_gates() {
    let k = 6;

    let mut dark_waiting = fresh_state();
    dark_waiting.save_progress_mut().set_dark_world_state(0x40);
    dark_waiting.sprite_slot_view_mut(k).set_state(9);
    dark_waiting.sprite_prep_smithy(k);
    assert_eq!(dark_waiting.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(dark_waiting.sprite_slot_view(k).subtype2(), 2);
    assert_eq!(dark_waiting.sprite_slot_view(k).state(), 9);

    let mut dark_done = fresh_state();
    dark_done.save_progress_mut().set_dark_world_state(0x40);
    dark_done.save_progress_mut().set_progress_indicator_3(32);
    dark_done.sprite_slot_view_mut(k).set_state(9);
    dark_done.sprite_prep_smithy(k);
    assert_eq!(dark_done.sprite_slot_view(k).state(), 0);

    let mut light_alone = fresh_state();
    light_alone.sprite_slot_view_mut(k).set_state(9);
    light_alone.sprite_set_x(k, 0x0100);
    light_alone.sprite_set_y(k, 0x0200);
    light_alone.sprite_prep_smithy(k);
    assert_eq!(light_alone.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(light_alone.sprite_get_x(k), 0x0102);
    assert_eq!(light_alone.sprite_get_y(k), 0x02fd);
    assert_eq!(light_alone.sprite_slot_view(15).sprite_type(), 0x31);
    assert_eq!(light_alone.sprite_get_x(15), 0x0100);
    assert_eq!(light_alone.sprite_get_y(15), 0x0200);
    assert_eq!(light_alone.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(light_alone.sprite_slot_view(15).ignore_projectile(), 1);

    let mut light_reunited = fresh_state();
    light_reunited.sprite_slot_view_mut(k).set_state(9);
    light_reunited
        .save_progress_mut()
        .set_progress_indicator_3(0xa0);
    light_reunited.sprite_set_x(k, 0x0100);
    light_reunited.sprite_set_y(k, 0x0200);
    light_reunited.sprite_prep_smithy(k);
    assert_eq!(light_reunited.sprite_slot_view(15).sprite_type(), 0x31);
    assert_eq!(light_reunited.sprite_slot_view(14).sprite_type(), 0x1a);
    assert_eq!(light_reunited.sprite_get_x(14), 0x012e);
    assert_eq!(light_reunited.sprite_get_y(14), 0x02fd);
    assert_eq!(light_reunited.sprite_slot_view(14).direction(), 1);
    assert_eq!(light_reunited.sprite_slot_view(14).a(), 4);
    assert_eq!(light_reunited.sprite_slot_view(14).ignore_projectile(), 4);
    assert_eq!(light_reunited.sprite_slot_view(13).sprite_type(), 0x31);
    assert_eq!(light_reunited.sprite_get_x(13), 0x012e);
    assert_eq!(light_reunited.sprite_get_y(13), 0x02fd);
    assert_eq!(light_reunited.sprite_slot_view(14).e(), k as u8);
    assert_eq!(light_reunited.sprite_slot_view(k).e(), 14);
    assert_eq!(light_reunited.sprite_slot_view(k).ai_state(), 5);
    assert_eq!(light_reunited.sprite_slot_view(14).ai_state(), 5);
}

#[test]
fn lanmolas_moldorm_and_tektite_prep_initialize_state() {
    let mut s = fresh_state();
    let k = 1;
    s.sprite_slot_view_mut(k).set_x_low(0x66);
    s.sprite_slot_view_mut(k).set_x_high(0x03);
    s.sprite_slot_view_mut(k).set_y_low(0x77);
    s.sprite_slot_view_mut(k).set_y_high(0x04);
    s.sprite_prep_moldorm(k);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(s.ram[MOLDORM_X_LO_PREP], 0x66);
    assert_eq!(s.ram[MOLDORM_Y_HI_PREP + 127], 0x04);

    let mut lanmolas = fresh_state();
    let k = 2;
    lanmolas.sprite_prep_lanmolas(k);
    assert_eq!(lanmolas.sprite_slot_view(k).delay_main(), 255);
    assert_eq!(lanmolas.sprite_slot_view(k).z(), 0xff);
    assert_eq!(lanmolas.ram[BEAMOS_X_HI + k * 0x40], 0xff);
    assert_eq!(lanmolas.ram[BEAMOS_X_HI + k * 0x40 + 63], 0xff);
    assert_eq!(lanmolas.garnish_slot_view(k).y_low(), 7);

    let mut shrapnel = fresh_state();
    shrapnel.sprite_slot_view_mut(k).set_state(9);
    shrapnel.sprite_set_x(k, 0x0120);
    shrapnel.sprite_set_y(k, 0x0340);
    shrapnel.lanmola_spawn_shrapnel(k);
    assert_eq!(shrapnel.game_state.scratch_counter.value(), 0xff);
    assert_eq!(shrapnel.sprite_slot_view(15).sprite_type(), 0xc2);
    assert_eq!(shrapnel.sprite_get_x(15), 0x0124);
    assert_eq!(shrapnel.sprite_get_y(15), 0x0344);
    assert_eq!(shrapnel.sprite_slot_view(15).ignore_projectile(), 1);
    assert_eq!(shrapnel.sprite_slot_view(15).bump_damage(), 1);
    assert_eq!(shrapnel.sprite_slot_view(15).flags4(), 1);
    assert_eq!(shrapnel.sprite_slot_view(15).z(), 0);
    assert_eq!(shrapnel.sprite_slot_view(15).flags2(), 0x20);
    assert_eq!(shrapnel.sprite_slot_view(15).x_velocity(), 0);
    assert_eq!(shrapnel.sprite_slot_view(15).y_velocity(), (-36i8) as u8);
    assert_eq!(shrapnel.sprite_slot_view(15).graphics(), 0);
    assert_eq!(shrapnel.sprite_slot_view(8).sprite_type(), 0xc2);
    assert_eq!(shrapnel.sprite_slot_view(8).x_velocity(), (-28i8) as u8);
    assert_eq!(shrapnel.sprite_slot_view(8).y_velocity(), 28);

    let mut short_shrapnel = fresh_state();
    short_shrapnel.sprite_slot_view_mut(0).set_state(9);
    short_shrapnel.sprite_slot_view_mut(1).set_state(9);
    short_shrapnel.sprite_slot_view_mut(2).set_state(9);
    short_shrapnel.sprite_slot_view_mut(k).set_state(9);
    short_shrapnel.sprite_set_x(k, 0x0050);
    short_shrapnel.sprite_set_y(k, 0x0060);
    short_shrapnel.lanmola_spawn_shrapnel(k);
    assert_eq!(short_shrapnel.sprite_slot_view(15).sprite_type(), 0xc2);
    assert_eq!(short_shrapnel.sprite_slot_view(15).x_velocity(), 28);
    assert_eq!(
        short_shrapnel.sprite_slot_view(15).y_velocity(),
        (-28i8) as u8
    );
    assert_eq!(short_shrapnel.sprite_slot_view(12).sprite_type(), 0xc2);
    assert_eq!(short_shrapnel.sprite_slot_view(11).sprite_type(), 0);

    let mut tektite = fresh_state();
    let k = 4;
    tektite.sprite_slot_view_mut(k).set_x_low(0x10);
    tektite.sprite_prep_tektite(k);
    assert_eq!(tektite.sprite_slot_view(k).a(), 1);
    assert_eq!(tektite.sprite_slot_view(k).oam_flags(), 7);
    assert_eq!(tektite.sprite_slot_view(k).health(), 12);
    assert_eq!(tektite.sprite_slot_view(k).bump_damage(), 5);
    assert_eq!(tektite.sprite_slot_view(k).z_velocity(), 32);
    assert_eq!(tektite.sprite_slot_view(k).ai_state(), 1);
}

#[test]
fn snitch_running_man_and_mushroom_prep_match_simple_gates() {
    let k = 6;

    let mut snitch = fresh_state();
    snitch.sprite_slot_view_mut(k).set_x_low(0x34);
    snitch.sprite_slot_view_mut(k).set_x_high(0x12);
    snitch.sprite_prep_snitches(k);
    assert_eq!(snitch.sprite_slot_view(k).direction(), 2);
    assert_eq!(snitch.sprite_slot_view(k).head_direction(), 2);
    assert_eq!(snitch.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(snitch.sprite_slot_view(k).a(), 0x34);
    assert_eq!(snitch.sprite_slot_view(k).b(), 0x12);
    assert_eq!(snitch.sprite_slot_view(k).x_velocity(), (-9i8) as u8);

    let mut bounce = fresh_state();
    bounce.sprite_slot_view_mut(k).set_x_low(0x55);
    bounce.sprite_prep_snitch_bounce_2(k);
    assert_eq!(bounce.sprite_slot_view(k).a(), 0x55);
    bounce.sprite_prep_snitch_bounce_3(k);
    assert_eq!(bounce.sprite_slot_view(k).ignore_projectile(), 2);

    let mut runner = fresh_state();
    runner.sprite_prep_running_man(k);
    assert_eq!(runner.sprite_slot_view(k).direction(), 2);
    assert_eq!(runner.sprite_slot_view(k).head_direction(), 2);
    assert_eq!(runner.sprite_slot_view(k).ignore_projectile(), 1);

    let mut mushroom = fresh_state();
    mushroom.inventory_items_mut().set_mushroom(1);
    mushroom.sprite_slot_view_mut(k).set_graphics(7);
    mushroom.sprite_prep_mushroom(k);
    assert_eq!(mushroom.sprite_slot_view(k).graphics(), 0);
    assert_eq!(mushroom.sprite_slot_view(k).oam_flags() & 8, 8);
    assert_eq!(mushroom.sprite_slot_view(k).ignore_projectile(), 1);

    mushroom.inventory_items_mut().set_mushroom(2);
    mushroom.sprite_slot_view_mut(k).set_state(9);
    mushroom.sprite_prep_mushroom(k);
    assert_eq!(mushroom.sprite_slot_view(k).state(), 0);
}

#[test]
fn potion_shop_prep_spawns_powder_and_cauldrons_with_barrier_flags() {
    let k = 4;
    let mut s = fresh_state();
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.set_flag_overworld_area_changed(1);
    s.inventory_items_mut().set_mushroom(1);
    s.save_progress_mut().set_dungeon_info_word(0x109, 0x80);

    s.sprite_prep_potion_shop(k);

    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
    for (slot, subtype, x, y) in [
        (15, 1, 0x0100u16.wrapping_sub(16), 0x0200),
        (
            14,
            2,
            0x0100u16.wrapping_sub(40),
            0x0200u16.wrapping_sub(72),
        ),
        (13, 3, 0x0100u16.wrapping_add(8), 0x0200u16.wrapping_sub(72)),
        (
            12,
            4,
            0x0100u16.wrapping_sub(88),
            0x0200u16.wrapping_sub(72),
        ),
    ] {
        assert_eq!(s.sprite_slot_view(slot).state(), 9);
        assert_eq!(s.sprite_slot_view(slot).sprite_type(), 0xe9);
        assert_eq!(s.sprite_slot_view(slot).subtype2(), subtype);
        assert_eq!(s.sprite_get_x(slot), x);
        assert_eq!(s.sprite_get_y(slot), y);
        assert_eq!(s.sprite_slot_view(slot).flags4(), 3);
        assert_eq!(s.sprite_slot_view(slot).deflection_bits() & 0x20, 0x20);
    }

    let mut skipped_powder = fresh_state();
    skipped_powder.sprite_slot_view_mut(k).set_state(9);
    skipped_powder.clear_flag_overworld_area_changed();
    skipped_powder.inventory_items_mut().set_mushroom(1);
    skipped_powder
        .save_progress_mut()
        .set_dungeon_info_word(0x109, 0x80);
    skipped_powder.sprite_prep_potion_shop(k);
    assert_eq!(skipped_powder.sprite_slot_view(15).subtype2(), 2);
    assert_eq!(skipped_powder.sprite_slot_view(14).subtype2(), 3);
    assert_eq!(skipped_powder.sprite_slot_view(13).subtype2(), 4);
    assert_eq!(skipped_powder.sprite_slot_view(12).state(), 0);
}

#[test]
fn arrow_game_prep_seeds_archery_sprites_from_link_state() {
    let k = 0;
    let mut s = fresh_state();
    s.sprite_slot_view_mut(k).set_y_low(0x30);
    s.ram[ARCHERY_GAME_HIT_COUNTER] = 0xaa;
    s.follower_link_state_mut().set_position(0x1200, 0x3400);
    s.follower_link_state_mut().mark_lower_level();
    s.player_resources_mut().set_arrows(17);

    s.sprite_prep_arrow_game_bounce(k);

    assert_eq!(s.ram[ARCHERY_GAME_HIT_COUNTER], 0);
    assert_eq!(s.sprite_slot_view(k).y_low(), 0x27);
    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(s.sprite_slot_view(k).subtype(), 17);

    assert_eq!(s.sprite_slot_view(1).sprite_type(), 0x65);
    assert_eq!(s.sprite_slot_view(1).state(), 9);
    assert_eq!(s.sprite_slot_view(1).x_high(), 0x12);
    assert_eq!(s.sprite_slot_view(1).x_low(), 0x40);
    assert_eq!(s.sprite_slot_view(1).y_high(), 0x34);
    assert_eq!(s.sprite_slot_view(1).y_low(), 0x4f);
    assert_eq!(s.sprite_slot_view(1).a(), 1);
    assert_eq!(s.sprite_slot_view(1).graphics(), 0);
    assert_eq!(s.sprite_slot_view(1).x_velocity(), (-8i8) as u8);
    assert_eq!(s.sprite_slot_view(1).flags4(), 0x1c);
    assert_eq!(s.sprite_slot_view(1).oam_flags(), 13);
    assert_eq!(s.sprite_slot_view(1).floor(), 1);

    assert_eq!(s.sprite_slot_view(7).x_low(), 0xc0);
    assert_eq!(s.sprite_slot_view(7).y_low(), 0x5a);
    assert_eq!(s.sprite_slot_view(7).a(), 2);
    assert_eq!(s.sprite_slot_view(7).graphics(), 1);
    assert_eq!(s.sprite_slot_view(7).x_velocity(), 12);
    assert_eq!(s.sprite_slot_view(7).flags4(), 0x15);
}

#[test]
fn heart_upgrade_prep_clears_already_obtained_entries() {
    let k = 4;

    let mut overworld = fresh_state();
    overworld.sprite_slot_view_mut(k).set_state(9);
    overworld.set_overworld_screen(0x22);
    overworld.set_overworld_event_info(0x22, 0x40);
    overworld.sprite_prep_heart_container(k);
    assert_eq!(overworld.sprite_slot_view(k).state(), 0);
    overworld.set_overworld_event_info(0x22, 0x10);
    overworld.heart_upgrade_set_obtained_flag(k);
    assert_eq!(
        overworld
            .game_state
            .world
            .overworld
            .event_info
            .event_info(0x22),
        0x50
    );

    let mut lumberjack = fresh_state();
    lumberjack.sprite_slot_view_mut(k).set_state(9);
    lumberjack.set_overworld_screen(0x3b);
    lumberjack.set_overworld_event_info(0x3b, 0);
    lumberjack.sprite_prep_heart_piece(k);
    assert_eq!(lumberjack.sprite_slot_view(k).state(), 0);

    let mut dungeon = fresh_state();
    dungeon.set_indoor_flag(1);
    dungeon.sprite_slot_view_mut(k).set_state(9);
    dungeon.sprite_slot_view_mut(k).set_x_high(0);
    dungeon
        .dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x4000);
    dungeon.heart_upgrade_check_if_already_obtained(k);
    assert_eq!(dungeon.sprite_slot_view(k).state(), 0);
    dungeon
        .dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x0001);
    dungeon.heart_upgrade_set_obtained_flag(k);
    assert_eq!(
        dungeon
            .game_state
            .dungeon
            .savegame_state
            .savegame_state_bits(),
        0x4001
    );

    dungeon.sprite_slot_view_mut(k).set_x_high(1);
    dungeon
        .dungeon_savegame_state_mut()
        .set_savegame_state_bits(0x0002);
    dungeon.heart_upgrade_set_obtained_flag(k);
    assert_eq!(
        dungeon
            .game_state
            .dungeon
            .savegame_state
            .savegame_state_bits(),
        0x2002
    );

    let mut untouched = fresh_state();
    untouched.sprite_slot_view_mut(k).set_state(9);
    untouched.set_overworld_screen(0x11);
    untouched.heart_upgrade_check_if_already_obtained(k);
    assert_eq!(untouched.sprite_slot_view(k).state(), 9);
}

#[test]
fn swamola_prep_initializes_segment_history_and_position_snapshot() {
    let k = 2;
    let mut buggy = fresh_state();
    buggy.sprite_slot_view_mut(k).set_x_low(0x44);
    buggy.sprite_slot_view_mut(k).set_x_high(0x01);
    buggy.sprite_slot_view_mut(k).set_y_low(0x88);
    buggy.sprite_slot_view_mut(k).set_y_high(0x02);
    buggy.sprite_prep_swamola(k);
    let buggy_start = 0x03;
    assert_eq!(buggy.ram[SWAMOLA_X_LO_PREP + buggy_start], 0x44);
    assert_eq!(buggy.ram[SWAMOLA_X_HI_PREP + buggy_start + 31], 0x01);
    assert_eq!(buggy.ram[SWAMOLA_Y_LO_PREP + buggy_start], 0x88);
    assert_eq!(buggy.ram[SWAMOLA_Y_HI_PREP + buggy_start + 31], 0x02);
    assert_eq!(buggy.sprite_slot_view(k).a(), 0x44);
    assert_eq!(buggy.sprite_slot_view(k).b(), 0x01);
    assert_eq!(buggy.sprite_slot_view(k).c(), 0x88);
    assert_eq!(buggy.sprite_slot_view(k).head_direction(), 0x02);

    let mut fixed = fresh_state();
    fixed
        .enhanced_features_mut()
        .set_bits(FEATURE_MISC_BUG_FIXES_PREP);
    fixed.sprite_slot_view_mut(k).set_x_low(0x77);
    fixed.sprite_slot_view_mut(k).set_x_high(0x03);
    fixed.sprite_slot_view_mut(k).set_y_low(0x99);
    fixed.sprite_slot_view_mut(k).set_y_high(0x04);
    fixed.sprite_prep_swamola_initialize_segments(k);
    let fixed_start = k * 32;
    assert_eq!(fixed.ram[SWAMOLA_X_LO_PREP + fixed_start], 0x77);
    assert_eq!(fixed.ram[SWAMOLA_X_HI_PREP + fixed_start + 31], 0x03);
    assert_eq!(fixed.ram[SWAMOLA_Y_LO_PREP + fixed_start], 0x99);
    assert_eq!(fixed.ram[SWAMOLA_Y_HI_PREP + fixed_start + 31], 0x04);
}

#[test]
fn blind_maiden_and_old_man_prep_follow_follower_gates() {
    let k = 5;

    let mut maiden = fresh_state();
    maiden.sprite_slot_view_mut(k).set_state(9);
    maiden.follower_state_mut().set_indicator(0);
    maiden.follower_state_mut().set_dropped(0x80);
    maiden.follower_state_mut().set_appearance_none_flag(7);
    maiden.sprite_prep_blind_maiden(k);
    assert_eq!(maiden.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(maiden.game_state.sprites.follower_runtime.indicator(), 0);
    assert_eq!(maiden.game_state.sprites.follower_runtime.dropped(), 0);
    assert_eq!(
        maiden
            .game_state
            .sprites
            .follower_runtime
            .appearance_none_flag(),
        0
    );
    assert_eq!(maiden.sprite_slot_view(k).state(), 9);

    let mut maiden_finished = fresh_state();
    maiden_finished.sprite_slot_view_mut(k).set_state(9);
    maiden_finished
        .save_progress_mut()
        .set_dungeon_info_word(0xac, 0x0800);
    maiden_finished.sprite_prep_blind_maiden(k);
    assert_eq!(maiden_finished.sprite_slot_view(k).state(), 0);

    let mut old_man_room = fresh_state();
    old_man_room.set_dungeon_room_index(0xe4);
    old_man_room.sprite_prep_old_man_bounce(k);
    assert_eq!(old_man_room.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(old_man_room.sprite_slot_view(k).subtype2(), 2);

    let mut old_man_mirror = fresh_state();
    old_man_mirror.inventory_items_mut().set_mirror(2);
    old_man_mirror.sprite_slot_view_mut(k).set_state(9);
    old_man_mirror.sprite_prep_old_man_bounce(k);
    assert_eq!(old_man_mirror.sprite_slot_view(k).state(), 0);
    assert_eq!(
        old_man_mirror
            .game_state
            .sprites
            .follower_runtime
            .indicator(),
        0
    );

    let mut old_man_followed = fresh_state();
    old_man_followed.follower_state_mut().set_indicator(1);
    old_man_followed.sprite_slot_view_mut(k).set_state(9);
    old_man_followed.sprite_prep_old_man_bounce(k);
    assert_eq!(old_man_followed.sprite_slot_view(k).state(), 0);
    assert_eq!(
        old_man_followed
            .game_state
            .sprites
            .follower_runtime
            .indicator(),
        1
    );
}

#[test]
fn zelda_bounce_prep_matches_sword_room_and_progress_gates() {
    let k = 6;

    let mut has_sword = fresh_state();
    has_sword.inventory_items_mut().set_sword_type(2);
    has_sword.sprite_slot_view_mut(k).set_state(9);
    has_sword.sprite_prep_zelda_bounce(k);
    assert_eq!(has_sword.sprite_slot_view(k).state(), 0);

    let mut cell = fresh_state();
    cell.sprite_slot_view_mut(k).set_state(9);
    cell.set_dungeon_room_index(0x12);
    cell.save_progress_mut().set_progress_flags(4);
    cell.follower_state_mut().set_indicator(7);
    cell.sprite_set_x(k, 0x0100);
    cell.sprite_set_y(k, 0x0200);
    cell.follower_link_state_mut().set_x(0x0180);
    cell.follower_link_state_mut().set_y(0x0200);
    cell.sprite_prep_zelda_bounce(k);
    assert_eq!(cell.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(cell.sprite_slot_view(k).direction(), 3);
    assert_eq!(cell.sprite_slot_view(k).head_direction(), 3);
    assert_eq!(cell.game_state.sprites.follower_runtime.indicator(), 7);
    assert_eq!(cell.sprite_slot_view(k).subtype2(), 2);
    assert_eq!(cell.sprite_get_x(k), 0x0106);
    assert_eq!(cell.sprite_get_y(k), 0x020f);
    assert_eq!(cell.sprite_slot_view(k).flags4(), 3);
    assert_eq!(cell.sprite_slot_view(k).state(), 9);

    let mut not_rescued = fresh_state();
    not_rescued.sprite_slot_view_mut(k).set_state(9);
    not_rescued.set_dungeon_room_index(0x12);
    not_rescued.save_progress_mut().set_progress_flags(0);
    not_rescued.sprite_prep_zelda_bounce(k);
    assert_eq!(not_rescued.sprite_slot_view(k).state(), 0);

    let mut follower_present = fresh_state();
    follower_present.sprite_slot_view_mut(k).set_state(9);
    follower_present.set_dungeon_room_index(0x20);
    follower_present.follower_state_mut().set_indicator(1);
    follower_present.sprite_prep_zelda_bounce(k);
    assert_eq!(follower_present.sprite_slot_view(k).subtype2(), 0);
    assert_eq!(follower_present.sprite_slot_view(k).state(), 0);
}

#[test]
fn bomb_shoppe_prep_spawns_visible_bombs_and_big_bomb_when_unlocked() {
    let k = 2;
    let mut s = fresh_state();
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_set_x(k, 0x0120);
    s.sprite_set_y(k, 0x0230);
    s.player_resources_mut().set_crystal_flags(5);
    s.save_progress_mut().set_progress_indicator_3(32);

    s.sprite_prep_bomb_shoppe(k);

    assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(s.sprite_slot_view(15).state(), 9);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xb5);
    assert_eq!(s.sprite_get_x(15), 0x0120u16.wrapping_sub(24));
    assert_eq!(s.sprite_get_y(15), 0x0230u16.wrapping_sub(24));
    assert_eq!(s.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 1);
    assert_eq!(s.sprite_slot_view(14).state(), 9);
    assert_eq!(s.sprite_slot_view(14).sprite_type(), 0xb5);
    assert_eq!(s.sprite_get_x(14), 0x0120u16.wrapping_sub(56));
    assert_eq!(s.sprite_get_y(14), 0x0230u16.wrapping_sub(24));
    assert_eq!(s.sprite_slot_view(14).subtype2(), 2);
    assert_eq!(s.sprite_slot_view(14).ignore_projectile(), 2);

    let mut locked = fresh_state();
    locked.sprite_slot_view_mut(k).set_state(9);
    locked.sprite_set_x(k, 0x0040);
    locked.sprite_set_y(k, 0x0050);
    locked.player_resources_mut().set_crystal_flags(4);
    locked.save_progress_mut().set_progress_indicator_3(32);
    locked.sprite_prep_bomb_shoppe(k);
    assert_eq!(locked.sprite_slot_view(15).state(), 9);
    assert_eq!(locked.sprite_slot_view(14).state(), 0);
}

#[test]
fn bomb_shop_clerk_exhalation_spawns_huff_with_exact_state() {
    let k = 2;
    let mut s = fresh_state();
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_set_x(k, 0x0120);
    s.sprite_set_y(k, 0x0230);
    s.sprite_slot_view_mut(k).set_z(9);
    s.sprite_slot_view_mut(15).set_flags3(0xff);

    s.bomb_shop_clerk_exhalation(k);

    assert_eq!(s.sprite_slot_view(15).state(), 9);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xb5);
    assert_eq!(s.sprite_get_x(15), 0x0124);
    assert_eq!(s.sprite_get_y(15), 0x0240);
    assert_eq!(s.sprite_slot_view(15).subtype2(), 3);
    assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 3);
    assert_eq!(s.sprite_slot_view(15).z(), 4);
    assert_eq!(s.sprite_slot_view(15).z_velocity(), (-12i8) as u8);
    assert_eq!(s.sprite_slot_view(15).delay_main(), 23);
    assert_eq!(s.sprite_slot_view(15).flags3() & 0x11, 0);
}

#[test]
fn bomb_shop_clerk_exhalation_noops_when_no_spawn_slot_exists() {
    let k = 2;
    let mut s = fresh_state();
    for slot in 0..16 {
        s.sprite_slot_view_mut(slot).set_state(9);
        s.sprite_slot_view_mut(slot)
            .set_sprite_type(0xa0 + slot as u8);
    }
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    let before = s.sprite_slot_view(15).sprite_type();

    s.bomb_shop_clerk_exhalation(k);

    assert_eq!(s.sprite_slot_view(15).sprite_type(), before);
    assert_eq!(s.sprite_get_x(15), 0);
    assert_eq!(s.sprite_get_y(15), 0);
}

#[test]
fn archery_game_guy_show_msg_sets_message_module_and_clears_delay() {
    let k = 4;
    let mut s = fresh_state();
    s.ram[TILE_INTERACTION_SHARED_FLAG] = 7;
    s.ram[MESSAGING_MODULE] = 9;
    s.set_submodule(1);
    s.set_main_module(3);
    s.clear_saved_module_for_menu();
    s.sprite_slot_view_mut(k).set_delay_main(88);

    s.archery_game_guy_show_msg(k, 0x86);

    assert_eq!(s.game_state.messaging.dialogue_message_index.value(), 0x86);
    assert_eq!(s.ram[TILE_INTERACTION_SHARED_FLAG], 0);
    assert_eq!(s.ram[MESSAGING_MODULE], 0);
    assert_eq!(s.game_state.frame.submodule, 2);
    assert_eq!(s.game_state.frame.saved_module_for_menu, 3);
    assert_eq!(s.game_state.frame.main_module, 14);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 0);
}

#[test]
fn debirando_prep_spawns_pit_pair_and_fire_variant_reloads_properties() {
    let k = 3;
    let mut pit = fresh_state();
    pit.sprite_slot_view_mut(k).set_state(9);
    pit.sprite_slot_view_mut(k).set_g(0);
    pit.sprite_slot_view_mut(k).set_delay_main(7);
    pit.sprite_slot_view_mut(k).set_graphics(2);
    pit.sprite_slot_view_mut(k).set_x_low(0x70);
    pit.sprite_slot_view_mut(k).set_y_low(0x80);

    pit.sprite_prep_debirando_pit(k);

    assert_eq!(pit.sprite_slot_view(k).g(), 1);
    assert_eq!(pit.sprite_slot_view(k).delay_main(), 0);
    assert_eq!(pit.sprite_slot_view(k).graphics(), 6);
    assert_eq!(pit.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(pit.sprite_slot_view(k).head_direction(), 15);
    assert_eq!(pit.sprite_slot_view(15).state(), 9);
    assert_eq!(pit.sprite_slot_view(15).sprite_type(), 0x64);
    assert_eq!(pit.sprite_slot_view(15).delay_main(), 96);
    assert_eq!(pit.sprite_slot_view(15).g(), 1);
    assert_eq!(pit.sprite_slot_view(15).oam_flags(), 8);
    assert_eq!(pit.sprite_get_x(15), pit.sprite_get_x(k));
    assert_eq!(pit.sprite_get_y(15), pit.sprite_get_y(k));

    let mut fire = fresh_state();
    fire.sprite_slot_view_mut(k).set_state(9);
    fire.sprite_slot_view_mut(k).set_sprite_type(0x64);
    fire.sprite_slot_view_mut(k).set_g(7);
    fire.sprite_slot_view_mut(k).set_delay_main(9);
    fire.sprite_slot_view_mut(k).set_x_low(0x44);
    fire.sprite_slot_view_mut(k).set_y_low(0x55);
    fire.sprite_prep_fire_debirando(k);
    assert_eq!(fire.sprite_slot_view(k).sprite_type(), 0x63);
    assert_eq!(fire.sprite_slot_view(k).g(), 0);
    assert_eq!(fire.sprite_slot_view(k).graphics(), 6);
    assert_eq!(fire.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(fire.sprite_slot_view(15).sprite_type(), 0x64);
    assert_eq!(fire.sprite_slot_view(15).g(), 0);
    assert_eq!(fire.sprite_slot_view(15).oam_flags(), 6);
}

#[test]
fn bully_hobo_and_talking_tree_prep_spawn_helper_sprites() {
    let k = 4;

    let mut bully = fresh_state();
    bully.sprite_slot_view_mut(k).set_state(9);
    bully.sprite_set_x(k, 0x0110);
    bully.sprite_set_y(k, 0x0220);
    bully.sprite_prep_bully_and_victim(k);
    assert_eq!(bully.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(bully.sprite_slot_view(15).state(), 9);
    assert_eq!(bully.sprite_slot_view(15).sprite_type(), 0xb9);
    assert_eq!(bully.sprite_get_x(15), 0x0110);
    assert_eq!(bully.sprite_get_y(15), 0x0220);
    assert_eq!(bully.sprite_slot_view(15).subtype2(), 2);
    assert_eq!(bully.sprite_slot_view(15).head_direction(), k as u8);
    assert_eq!(bully.sprite_slot_view(15).ignore_projectile(), 1);
    bully.ball_guy_play_bounce_noise(k);
    assert_eq!(
        bully.game_state.system_signals.sound_effect_2() & 0x3f,
        0x32
    );

    let mut garnish = fresh_state();
    garnish.garnish_slot_view_mut(29).set_garnish_type(1);
    garnish.garnish_slot_view_mut(14).set_garnish_type(1);
    assert_eq!(garnish.garnish_alloc_force(), 28);
    assert_eq!(garnish.garnish_alloc(), 28);
    assert_eq!(garnish.garnish_alloc_low(), 13);
    assert_eq!(garnish.garnish_alloc_limit(12), 12);

    for slot in 0..30 {
        garnish.garnish_slot_view_mut(slot).set_garnish_type(1);
    }
    assert_eq!(garnish.garnish_alloc_force(), 0);
    assert_eq!(garnish.garnish_alloc(), -1);
    assert_eq!(garnish.garnish_alloc_low(), -1);
    assert_eq!(garnish.garnish_alloc_limit(12), -1);
    assert_eq!(garnish.garnish_alloc_overwrite_old_low(), 14);
    assert_eq!(garnish.garnish_alloc_overwrite_old(), 13);

    let mut coords = fresh_state();
    coords.garnish_set_x(3, 0x1234);
    coords.garnish_set_y(3, 0xabcd);
    assert_eq!(coords.garnish_slot_view(3).x_low(), 0x34);
    assert_eq!(coords.garnish_slot_view(3).x_high(), 0x12);
    assert_eq!(coords.garnish_slot_view(3).y_low(), 0xcd);
    assert_eq!(coords.garnish_slot_view(3).y_high(), 0xab);

    let mut debris = fresh_state();
    debris.garnish_slot_view_mut(29).set_garnish_type(1);
    debris.garnish_spawn_pyramid_debris(-4, 5, -7, 9);
    assert_eq!(debris.game_state.system_signals.sound_effect_2(), 3);
    assert_eq!(debris.game_state.system_signals.sound_effect_1(), 31);
    assert_eq!(debris.game_state.system_signals.ambient_sound_effect(), 5);
    assert_eq!(debris.garnish_slot_view(28).garnish_type(), 19);
    assert_eq!(debris.game_state.sprites.garnish_runtime.active_type(), 19);
    assert_eq!(debris.garnish_slot_view(28).x_low(), 228);
    assert_eq!(debris.garnish_slot_view(28).y_low(), 101);
    assert_eq!(debris.garnish_slot_view(28).x_velocity(), (-7i8) as u8);
    assert_eq!(debris.garnish_slot_view(28).y_velocity(), 9);
    assert_eq!(debris.garnish_slot_view(28).countdown(), 72);

    let mut puff = fresh_state();
    let puff_owner = 6;
    puff.set_frame_counter(2);
    puff.garnish_slot_view_mut(14).set_garnish_type(1);
    puff.sprite_workspace_mut().set_current_sprite_x(0x0200);
    puff.sprite_workspace_mut().set_current_sprite_y(0x0300);
    puff.kholdstare_spawn_puff_cloud_garnish(puff_owner);
    assert_eq!(puff.garnish_slot_view(13).garnish_type(), 7);
    assert_eq!(puff.game_state.sprites.garnish_runtime.active_type(), 7);
    assert_eq!(puff.garnish_slot_view(13).countdown(), 31);
    assert_eq!(puff.garnish_slot_view(13).x_low(), 0xfa);
    assert_eq!(puff.garnish_slot_view(13).x_high(), 0x01);
    assert_eq!(puff.garnish_slot_view(13).y_low(), 0x12);
    assert_eq!(puff.garnish_slot_view(13).y_high(), 0x03);
    assert_eq!(puff.garnish_slot_view(13).floor(), 0);

    let mut flame = fresh_state();
    flame.garnish_slot_view_mut(29).set_garnish_type(1);
    flame.sprite_set_x(k, 0x0456);
    flame.sprite_set_y(k, 0x0789);
    assert_eq!(flame.garnish_flame_trail(k, false), 28);
    assert_eq!(flame.garnish_slot_view(28).garnish_type(), 0x10);
    assert_eq!(flame.game_state.sprites.garnish_runtime.active_type(), 0x10);
    assert_eq!(flame.garnish_slot_view(28).sprite(), k as u8);
    assert_eq!(flame.garnish_slot_view(28).x_low(), 0x56);
    assert_eq!(flame.garnish_slot_view(28).x_high(), 0x04);
    assert_eq!(flame.garnish_slot_view(28).y_low(), 0x99);
    assert_eq!(flame.garnish_slot_view(28).y_high(), 0x07);
    assert_eq!(flame.garnish_slot_view(28).countdown(), 127);

    let mut low_flame = fresh_state();
    low_flame.garnish_slot_view_mut(14).set_garnish_type(1);
    low_flame.sprite_set_x(k, 0x0012);
    low_flame.sprite_set_y(k, 0x00f8);
    assert_eq!(low_flame.garnish_flame_trail(k, true), 13);
    assert_eq!(low_flame.garnish_slot_view(13).garnish_type(), 0x10);
    assert_eq!(low_flame.garnish_slot_view(13).y_low(), 0x08);
    assert_eq!(low_flame.garnish_slot_view(13).y_high(), 0x01);

    let mut fire_bat = fresh_state();
    fire_bat.sprite_slot_view_mut(k).set_subtype2(3);
    fire_bat.fire_bat_animate(k);
    assert_eq!(fire_bat.sprite_slot_view(k).subtype2(), 4);
    assert_eq!(fire_bat.sprite_slot_view(k).graphics(), 5);

    let mut moving_fire_bat = fresh_state();
    moving_fire_bat
        .garnish_slot_view_mut(14)
        .set_garnish_type(1);
    moving_fire_bat.sprite_slot_view_mut(k).set_subtype2(7);
    moving_fire_bat.sprite_slot_view_mut(k).set_anim_clock(5);
    moving_fire_bat.sprite_set_x(k, 0x0124);
    moving_fire_bat.sprite_set_y(k, 0x0340);
    moving_fire_bat.fire_bat_move(k);
    assert_eq!(moving_fire_bat.sprite_slot_view(k).subtype2(), 8);
    assert_eq!(moving_fire_bat.sprite_slot_view(k).graphics(), 6);
    assert_eq!(moving_fire_bat.garnish_slot_view(13).garnish_type(), 0x10);
    assert_eq!(
        moving_fire_bat
            .game_state
            .sprites
            .garnish_runtime
            .active_type(),
        0x10
    );
    assert_eq!(moving_fire_bat.garnish_slot_view(13).sprite(), k as u8);
    assert_eq!(moving_fire_bat.garnish_slot_view(13).x_low(), 0x24);
    assert_eq!(moving_fire_bat.garnish_slot_view(13).x_high(), 0x01);
    assert_eq!(moving_fire_bat.garnish_slot_view(13).y_low(), 0x50);
    assert_eq!(moving_fire_bat.garnish_slot_view(13).y_high(), 0x03);
    assert_eq!(moving_fire_bat.garnish_slot_view(13).countdown(), 0x2f);

    let mut skipped_fire_bat = fresh_state();
    skipped_fire_bat.sprite_slot_view_mut(k).set_subtype2(0);
    skipped_fire_bat.fire_bat_move(k);
    assert_eq!(skipped_fire_bat.sprite_slot_view(k).subtype2(), 1);
    assert_eq!(
        skipped_fire_bat
            .game_state
            .sprites
            .garnish_runtime
            .active_type(),
        0
    );

    let mut fireball = fresh_state();
    fireball.set_frame_counter(0);
    fireball.garnish_slot_view_mut(29).set_garnish_type(1);
    fireball.sprite_workspace_mut().set_current_sprite_x(0x0123);
    fireball.sprite_workspace_mut().set_current_sprite_y(0x02f5);
    fireball.fireball_spawn_trail_garnish(k);
    assert_eq!(fireball.garnish_slot_view(28).garnish_type(), 8);
    assert_eq!(fireball.game_state.sprites.garnish_runtime.active_type(), 8);
    assert_eq!(fireball.garnish_slot_view(28).countdown(), 11);
    assert_eq!(fireball.garnish_slot_view(28).x_low(), 0x23);
    assert_eq!(fireball.garnish_slot_view(28).x_high(), 0x01);
    assert_eq!(fireball.garnish_slot_view(28).y_low(), 0x05);
    assert_eq!(fireball.garnish_slot_view(28).y_high(), 0x03);
    assert_eq!(fireball.garnish_slot_view(28).sprite(), k as u8);

    let mut skipped_fireball = fresh_state();
    skipped_fireball.set_frame_counter(1);
    skipped_fireball.fireball_spawn_trail_garnish(k);
    assert_eq!(
        skipped_fireball
            .game_state
            .sprites
            .garnish_runtime
            .active_type(),
        0
    );

    let mut firesnake = fresh_state();
    firesnake.set_frame_counter(k as u8);
    firesnake.garnish_slot_view_mut(29).set_garnish_type(1);
    firesnake.sprite_set_x(k, 0x0167);
    firesnake.sprite_set_y(k, 0x02f0);
    firesnake.sprite_slot_view_mut(k).set_floor(2);
    firesnake.firesnake_spawn_fireball(k);
    assert_eq!(firesnake.garnish_slot_view(28).garnish_type(), 1);
    assert_eq!(
        firesnake.game_state.sprites.garnish_runtime.active_type(),
        1
    );
    assert_eq!(firesnake.garnish_slot_view(28).x_low(), 0x67);
    assert_eq!(firesnake.garnish_slot_view(28).x_high(), 0x01);
    assert_eq!(firesnake.garnish_slot_view(28).y_low(), 0x00);
    assert_eq!(firesnake.garnish_slot_view(28).y_high(), 0x03);
    assert_eq!(firesnake.garnish_slot_view(28).countdown(), 32);
    assert_eq!(firesnake.garnish_slot_view(28).sprite(), k as u8);
    assert_eq!(firesnake.garnish_slot_view(28).floor(), 2);

    let mut skipped_firesnake = fresh_state();
    skipped_firesnake.set_frame_counter((k as u8) ^ 1);
    skipped_firesnake.firesnake_spawn_fireball(k);
    assert_eq!(
        skipped_firesnake
            .game_state
            .sprites
            .garnish_runtime
            .active_type(),
        0
    );

    let mut plop = fresh_state();
    plop.sprite_slot_view_mut(k).set_state(9);
    plop.sprite_set_x(k, 0x0100);
    plop.sprite_set_y(k, 0x0200);
    plop.catfish_spawn_plop(k);
    assert_eq!(plop.sprite_slot_view(15).sprite_type(), 0xec);
    assert_eq!(plop.sprite_get_x(15), 0x0100);
    assert_eq!(plop.sprite_get_y(15), 0x0200);
    assert_eq!(plop.sprite_slot_view(15).state(), 3);
    assert_eq!(plop.sprite_slot_view(15).delay_main(), 15);
    assert_eq!(plop.sprite_slot_view(15).ai_state(), 0);
    assert_eq!(plop.sprite_slot_view(15).flags2(), 3);
    assert_eq!(plop.game_state.system_signals.sound_effect_1() & 0x3f, 0x28);

    let mut medallion = fresh_state();
    medallion.sprite_slot_view_mut(k).set_state(9);
    medallion.sprite_set_x(k, 0x0100);
    medallion.sprite_set_y(k, 0x0200);
    medallion.catfish_regurgitate_medallion(k);
    assert_eq!(medallion.sprite_slot_view(15).sprite_type(), 0xc0);
    assert_eq!(medallion.sprite_get_x(15), 0x0100);
    assert_eq!(medallion.sprite_get_y(15), 0x0200);
    assert_eq!(medallion.sprite_slot_view(15).x_velocity(), 24);
    assert_eq!(medallion.sprite_slot_view(15).z_velocity(), 48);
    assert_eq!(medallion.sprite_slot_view(15).a(), 17);
    assert_eq!(
        medallion.game_state.system_signals.sound_effect_1() & 0x3f,
        0x20
    );
    assert_eq!(medallion.sprite_slot_view(15).flags2(), 0x83);
    assert_eq!(medallion.sprite_slot_view(15).flags3(), 0x58);
    assert_eq!(medallion.sprite_slot_view(15).oam_flags(), 8);

    let mut splash = fresh_state();
    splash.sprite_slot_view_mut(k).set_state(9);
    splash.sprite_set_x(k, 0x0030);
    splash.sprite_set_y(k, 0x0040);
    assert_eq!(splash.sprite_spawn_water_splash(k), 15);
    assert_eq!(splash.sprite_slot_view(15).sprite_type(), 0xc0);
    assert_eq!(splash.sprite_get_x(15), 0x0030);
    assert_eq!(splash.sprite_get_y(15), 0x0040);
    assert_eq!(splash.sprite_slot_view(15).a(), 0x80);
    assert_eq!(splash.sprite_slot_view(15).flags2(), 2);
    assert_eq!(splash.sprite_slot_view(15).ignore_projectile(), 2);
    assert_eq!(splash.sprite_slot_view(15).oam_flags(), 4);
    assert_eq!(splash.sprite_slot_view(15).delay_main(), 31);

    let mut small_splash = fresh_state();
    small_splash.sprite_slot_view_mut(k).set_state(9);
    small_splash.sprite_set_x(k, 0x0060);
    small_splash.sprite_set_y(k, 0x0070);
    small_splash.set_sound_effect_1(0xff);
    assert_eq!(small_splash.sprite_spawn_small_splash(k), 14);
    assert_eq!(small_splash.sprite_slot_view(14).sprite_type(), 0xec);
    assert_eq!(small_splash.sprite_get_x(14), 0x0060);
    assert_eq!(small_splash.sprite_get_y(14), 0x0070);
    assert_eq!(
        small_splash.game_state.system_signals.sound_effect_1() & 0x3f,
        0x28
    );
    assert_eq!(small_splash.sprite_slot_view(14).state(), 3);
    assert_eq!(small_splash.sprite_slot_view(14).delay_main(), 15);
    assert_eq!(small_splash.sprite_slot_view(14).ai_state(), 0);
    assert_eq!(small_splash.sprite_slot_view(14).flags2(), 3);

    let mut dust = fresh_state();
    dust.sprite_slot_view_mut(k).set_state(9);
    dust.sprite_set_x(k, 0x0100);
    dust.sprite_set_y(k, 0x0200);
    assert_eq!(dust.sprite_spawn_dust_cloud(k), 15);
    assert_eq!(dust.sprite_slot_view(15).sprite_type(), 0xf2);
    assert_eq!(dust.sprite_get_x(15), 0x00fc);
    assert_eq!(dust.sprite_get_y(15), 0x0208);
    assert_eq!(dust.sprite_slot_view(15).subtype2(), 1);

    let mut blast = fresh_state();
    blast.sprite_slot_view_mut(k).set_state(9);
    blast.sprite_set_x(k, 0x0018);
    blast.sprite_set_y(k, 0x0028);
    assert_eq!(blast.sprite_spawn_superficial_bomb_blast(k), 15);
    assert_eq!(blast.sprite_slot_view(15).sprite_type(), 0x4a);
    assert_eq!(blast.sprite_get_x(15), 0x0018);
    assert_eq!(blast.sprite_get_y(15), 0x0028);
    assert_eq!(blast.sprite_slot_view(15).state(), 6);
    assert_eq!(blast.sprite_slot_view(15).delay_aux1(), 31);
    assert_eq!(blast.sprite_slot_view(15).c(), 3);
    assert_eq!(blast.sprite_slot_view(15).flags2(), 3);
    assert_eq!(blast.sprite_slot_view(15).oam_flags(), 4);
    assert_eq!(
        blast.game_state.system_signals.sound_effect_1() & 0x3f,
        0x15
    );

    let mut bomb = fresh_state();
    bomb.sprite_slot_view_mut(k).set_state(9);
    bomb.sprite_set_x(k, 0x0044);
    bomb.sprite_set_y(k, 0x0055);
    assert_eq!(bomb.sprite_spawn_bomb(k), 15);
    assert_eq!(bomb.sprite_slot_view(15).sprite_type(), 0x4a);
    assert_eq!(bomb.sprite_get_x(15), 0x0044);
    assert_eq!(bomb.sprite_get_y(15), 0x0055);
    assert_eq!(bomb.sprite_slot_view(15).c(), 1);
    assert_eq!(bomb.sprite_slot_view(15).delay_aux1(), 80);
    assert_eq!(bomb.sprite_slot_view(15).flags3(), 0x18);
    assert_eq!(bomb.sprite_slot_view(15).oam_flags(), 8);
    assert_eq!(bomb.sprite_slot_view(15).health(), 0);
    assert_eq!(bomb.sprite_slot_view(15).x_velocity(), 24);
    assert_eq!(bomb.sprite_slot_view(15).z_velocity(), 48);

    let mut poof = fresh_state();
    poof.sprite_slot_view_mut(k).set_state(9);
    poof.sprite_set_x(k, 0x0100);
    poof.sprite_set_y(k, 0x0200);
    assert_eq!(poof.spawn_boss_poof(k), 15);
    assert_eq!(poof.sprite_slot_view(15).sprite_type(), 0xce);
    assert_eq!(poof.sprite_get_x(15), 0x0110);
    assert_eq!(poof.sprite_get_y(15), 0x0228);
    assert_eq!(poof.sprite_slot_view(15).graphics(), 0x0f);
    assert_eq!(poof.sprite_slot_view(15).a(), 1);
    assert_eq!(poof.sprite_slot_view(15).delay_main(), 47);
    assert_eq!(poof.sprite_slot_view(15).flags2(), 9);
    assert_eq!(poof.sprite_slot_view(15).ignore_projectile(), 9);
    assert_eq!(poof.game_state.system_signals.sound_effect_1(), 12);

    let mut fireball = fresh_state();
    fireball.sprite_slot_view_mut(k).set_state(9);
    fireball.sprite_set_x(k, 0x0100);
    fireball.sprite_set_y(k, 0x0200);
    fireball.sprite_slot_view_mut(k).set_z(16);
    fireball.follower_link_state_mut().set_x(0x0124);
    fireball.follower_link_state_mut().set_y(0x01ec);
    assert_eq!(fireball.sprite_spawn_fireball(k), 13);
    assert_eq!(fireball.sprite_slot_view(13).sprite_type(), 0x55);
    assert_eq!(fireball.sprite_get_x(13), 0x0104);
    assert_eq!(fireball.sprite_get_y(13), 0x01f4);
    assert_eq!(fireball.sprite_slot_view(13).flags3(), 0x42);
    assert_eq!(fireball.sprite_slot_view(13).oam_flags(), 6);
    assert_eq!(fireball.sprite_slot_view(13).flags4(), 0x54);
    assert_eq!(fireball.sprite_slot_view(13).e(), 0x54);
    assert_eq!(fireball.sprite_slot_view(13).flags2(), 0x20);
    assert_eq!(fireball.sprite_slot_view(13).x_velocity(), 0x20);
    assert_eq!(fireball.sprite_slot_view(13).y_velocity(), 0);
    assert_eq!(fireball.sprite_slot_view(13).delay_main(), 20);
    assert_eq!(fireball.sprite_slot_view(13).delay_aux1(), 16);
    assert_eq!(fireball.sprite_slot_view(13).flags5(), 0);
    assert_eq!(fireball.sprite_slot_view(13).deflection_bits(), 0x48);
    assert_eq!(
        fireball.game_state.system_signals.sound_effect_2() & 0x3f,
        0x19
    );

    let mut phlegm = fresh_state();
    phlegm.sprite_slot_view_mut(k).set_state(9);
    phlegm.sprite_set_x(k, 0x0040);
    phlegm.sprite_set_y(k, 0x0060);
    phlegm.sprite_slot_view_mut(k).set_z(7);
    phlegm.sprite_slot_view_mut(k).set_direction(1);
    phlegm.inventory_items_mut().set_shield_type(3);
    assert_eq!(phlegm.sprite_spawn_fire_phlegm(k), 15);
    assert_eq!(phlegm.sprite_slot_view(15).sprite_type(), 0xa5);
    assert_eq!(phlegm.sprite_get_x(15), 0x0038);
    assert_eq!(phlegm.sprite_get_y(15), 0x005e);
    assert_eq!(phlegm.sprite_slot_view(15).x_velocity(), (-48i8) as u8);
    assert_eq!(phlegm.sprite_slot_view(15).y_velocity(), 0);
    assert_eq!(phlegm.sprite_slot_view(15).flags3() & 0x40, 0x40);
    assert_eq!(phlegm.sprite_slot_view(15).deflection_bits(), 0x40);
    assert_eq!(phlegm.sprite_slot_view(15).flags2(), 0x21);
    assert_eq!(phlegm.sprite_slot_view(15).b(), 0x21);
    assert_eq!(phlegm.sprite_slot_view(15).oam_flags(), 2);
    assert_eq!(phlegm.sprite_slot_view(15).flags4(), 0x14);
    assert_eq!(phlegm.sprite_slot_view(15).ignore_projectile(), 20);
    assert_eq!(phlegm.sprite_slot_view(15).bump_damage(), 37);
    assert_eq!(phlegm.sprite_slot_view(15).flags5(), 0x20);
    assert_eq!(phlegm.game_state.system_signals.sound_effect_2() & 0x3f, 5);

    let mut leaves = fresh_state();
    leaves.sprite_slot_view_mut(k).set_state(9);
    leaves.sprite_set_x(k, 0x0120);
    leaves.sprite_set_y(k, 0x0340);
    leaves.sprite_slot_view_mut(k).set_z_velocity(0x24);
    assert_eq!(leaves.lumberjack_tree_spawn_leaves(k), 15);
    assert_eq!(leaves.sprite_slot_view(15).sprite_type(), 0x3b);
    assert_eq!(leaves.sprite_get_x(15), 0x0120);
    assert_eq!(leaves.sprite_get_y(15), 0x0340);
    assert_eq!(leaves.sprite_slot_view(15).graphics(), 2);
    assert_eq!(leaves.sprite_slot_view(15).z_velocity(), 0x24);
    assert_eq!(leaves.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(leaves.sprite_slot_view(15).ai_state(), 2);
    assert_eq!(leaves.sprite_slot_view(15).delay_main(), 8);

    let mut garnish_poof = fresh_state();
    garnish_poof.sprite_set_x(k, 0x0234);
    garnish_poof.sprite_set_y(k, 0x0456);
    garnish_poof.sprite_slot_view_mut(k).set_floor(2);
    garnish_poof.sprite_spawn_poof_garnish(k);
    assert_eq!(garnish_poof.garnish_slot_view(29).garnish_type(), 10);
    assert_eq!(
        garnish_poof
            .game_state
            .sprites
            .garnish_runtime
            .active_type(),
        10
    );
    assert_eq!(garnish_poof.garnish_slot_view(29).x_low(), 0x34);
    assert_eq!(garnish_poof.garnish_slot_view(29).x_high(), 0x02);
    assert_eq!(garnish_poof.garnish_slot_view(29).y_low(), 0x66);
    assert_eq!(garnish_poof.garnish_slot_view(29).y_high(), 0x04);
    assert_eq!(garnish_poof.garnish_slot_view(29).sprite(), 2);
    assert_eq!(garnish_poof.garnish_slot_view(29).countdown(), 15);

    let mut octorok = fresh_state();
    octorok.sprite_slot_view_mut(k).set_state(9);
    octorok.sprite_set_x(k, 0x0100);
    octorok.sprite_set_y(k, 0x0200);
    octorok.sprite_slot_view_mut(k).set_direction(0);
    octorok.octorok_fire_loogie(k);
    assert_eq!(octorok.sprite_slot_view(15).sprite_type(), 0x0c);
    assert_eq!(octorok.sprite_get_x(15), 0x010c);
    assert_eq!(octorok.sprite_get_y(15), 0x0204);
    assert_eq!(octorok.sprite_slot_view(15).x_velocity(), 44);
    assert_eq!(octorok.sprite_slot_view(15).y_velocity(), 0);
    assert_eq!(octorok.game_state.system_signals.sound_effect_1() & 0x3f, 7);

    let mut moblin = fresh_state();
    moblin.sprite_slot_view_mut(k).set_state(9);
    moblin.sprite_set_x(k, 0x0200);
    moblin.sprite_set_y(k, 0x0100);
    moblin.sprite_slot_view_mut(k).set_direction(3);
    moblin.moblin_materialize_spear(k);
    assert_eq!(moblin.sprite_slot_view(15).sprite_type(), 0x1b);
    assert_eq!(moblin.sprite_slot_view(15).a(), 3);
    assert_eq!(moblin.sprite_slot_view(15).direction(), 3);
    assert_eq!(moblin.sprite_get_x(15), 0x020b);
    assert_eq!(moblin.sprite_get_y(15), 0x00f5);
    assert_eq!(moblin.sprite_slot_view(15).x_velocity(), 0);
    assert_eq!(moblin.sprite_slot_view(15).y_velocity(), (-32i8) as u8);

    let mut snitch = fresh_state();
    snitch.sprite_slot_view_mut(k).set_state(9);
    snitch.sprite_slot_view_mut(k).set_sprite_type(0x35);
    snitch.garnish_state_mut().set_sprcoll_x_base(0x1200);
    snitch.garnish_state_mut().set_sprcoll_y_base(0x3400);
    snitch.snitch_spawn_guard(k);
    assert_eq!(snitch.sprite_slot_view(0).sprite_type(), 0x45);
    assert_eq!(snitch.sprite_slot_view(0).state(), 9);
    assert_eq!(snitch.sprite_get_x(0), 0x1540);
    assert_eq!(snitch.sprite_get_y(0), 0x37b0);
    assert_eq!(snitch.sprite_slot_view(0).floor(), 0);
    assert_eq!(snitch.sprite_slot_view(0).health(), 4);
    assert_eq!(snitch.sprite_slot_view(0).deflection_bits(), 0x80);
    assert_eq!(snitch.sprite_slot_view(0).flags5(), 0x90);
    assert_eq!(snitch.sprite_slot_view(0).oam_flags(), 0x0b);

    let mut sparkle = fresh_state();
    for (idx, ty) in [0x2a, 0x21, 0x30, 0x19, 0x0c].into_iter().enumerate() {
        sparkle.ancilla_slot_view_mut(idx).set_ancilla_type(ty);
    }
    sparkle.ancilla_terminate_sparkle_objects();
    assert_eq!(sparkle.ancilla_slot_view(0).ancilla_type(), 0);
    assert_eq!(sparkle.ancilla_slot_view(1).ancilla_type(), 0x21);
    assert_eq!(sparkle.ancilla_slot_view(2).ancilla_type(), 0);
    assert_eq!(sparkle.ancilla_slot_view(3).ancilla_type(), 0);
    assert_eq!(sparkle.ancilla_slot_view(4).ancilla_type(), 0);

    let mut kodongo = fresh_state();
    kodongo.sprite_slot_view_mut(k).set_direction(2);
    kodongo.kodongo_set_direction(k);
    assert_eq!(kodongo.sprite_slot_view(k).x_velocity(), 0);
    assert_eq!(kodongo.sprite_slot_view(k).y_velocity(), 16);

    let mut kodongo_fire = fresh_state();
    kodongo_fire.sprite_slot_view_mut(k).set_state(9);
    kodongo_fire.sprite_set_x(k, 0x0300);
    kodongo_fire.sprite_set_y(k, 0x0040);
    kodongo_fire.sprite_slot_view_mut(k).set_direction(1);
    kodongo_fire.kodongo_spawn_fire(k);
    assert_eq!(kodongo_fire.sprite_slot_view(13).sprite_type(), 0x87);
    assert_eq!(kodongo_fire.sprite_get_x(13), 0x02f8);
    assert_eq!(kodongo_fire.sprite_get_y(13), 0x0040);
    assert_eq!(
        kodongo_fire.sprite_slot_view(13).x_velocity(),
        (-24i8) as u8
    );
    assert_eq!(kodongo_fire.sprite_slot_view(13).y_velocity(), 0);
    assert_eq!(kodongo_fire.sprite_slot_view(13).ignore_projectile(), 1);

    let mut blue_balls = fresh_state();
    blue_balls.sprite_slot_view_mut(k).set_state(9);
    blue_balls.sprite_set_x(k, 0x0120);
    blue_balls.sprite_set_y(k, 0x0340);
    blue_balls.create_six_blue_balls(k);
    assert_eq!(
        blue_balls.game_state.system_signals.sound_effect_2() & 0x3f,
        0x36
    );
    assert_eq!(blue_balls.game_state.scratch_counter.value(), 0);
    assert_eq!(blue_balls.sprite_slot_view(15).sprite_type(), 0x55);
    assert_eq!(blue_balls.sprite_get_x(15), 0x0124);
    assert_eq!(blue_balls.sprite_get_y(15), 0x0344);
    assert_eq!(blue_balls.sprite_slot_view(15).flags3(), 0x42);
    assert_eq!(blue_balls.sprite_slot_view(15).oam_flags(), 4);
    assert_eq!(blue_balls.sprite_slot_view(15).delay_aux1(), 4);
    assert_eq!(blue_balls.sprite_slot_view(15).flags4(), 20);
    assert_eq!(blue_balls.sprite_slot_view(15).c(), 20);
    assert_eq!(blue_balls.sprite_slot_view(15).e(), 20);
    assert_eq!(blue_balls.sprite_slot_view(15).x_velocity(), (-24i8) as u8);
    assert_eq!(blue_balls.sprite_slot_view(15).y_velocity(), (-16i8) as u8);
    assert_eq!(blue_balls.sprite_slot_view(10).sprite_type(), 0x55);
    assert_eq!(blue_balls.sprite_slot_view(10).x_velocity(), 0);
    assert_eq!(blue_balls.sprite_slot_view(10).y_velocity(), (-32i8) as u8);

    let mut octoballoon = fresh_state();
    octoballoon.sprite_slot_view_mut(k).set_state(9);
    octoballoon.sprite_set_x(k, 0x0110);
    octoballoon.sprite_set_y(k, 0x0220);
    octoballoon.octoballoon_form_babby(k);
    assert_eq!(
        octoballoon.game_state.system_signals.sound_effect_1() & 0x3f,
        0x0c
    );
    assert_eq!(octoballoon.sprite_slot_view(15).sprite_type(), 0x10);
    assert_eq!(octoballoon.sprite_get_x(15), 0x0110);
    assert_eq!(octoballoon.sprite_get_y(15), 0x0220);
    assert_eq!(octoballoon.sprite_slot_view(15).x_velocity(), 11);
    assert_eq!(octoballoon.sprite_slot_view(15).y_velocity(), (-11i8) as u8);
    assert_eq!(octoballoon.sprite_slot_view(15).z_velocity(), 48);
    assert_eq!(octoballoon.sprite_slot_view(15).subtype2(), 255);
    assert_eq!(octoballoon.sprite_slot_view(10).sprite_type(), 0x10);
    assert_eq!(octoballoon.sprite_slot_view(10).x_velocity(), 16);
    assert_eq!(octoballoon.sprite_slot_view(10).y_velocity(), 0);

    let mut bully = fresh_state();
    bully.sprite_slot_view_mut(k).set_state(9);
    bully.sprite_set_x(k, 0x0440);
    bully.sprite_set_y(k, 0x0550);
    bully.ball_guy_play_bounce_noise(k);
    assert_eq!(
        bully.game_state.system_signals.sound_effect_2() & 0x3f,
        0x32
    );
    bully.spawn_bully(k);
    assert_eq!(bully.sprite_slot_view(15).sprite_type(), 0xb9);
    assert_eq!(bully.sprite_get_x(15), 0x0440);
    assert_eq!(bully.sprite_get_y(15), 0x0550);
    assert_eq!(bully.sprite_slot_view(15).subtype2(), 2);
    assert_eq!(bully.sprite_slot_view(15).head_direction(), k as u8);
    assert_eq!(bully.sprite_slot_view(15).ignore_projectile(), 1);

    let mut rupees = fresh_state();
    rupees.sprite_slot_view_mut(k).set_state(9);
    rupees.sprite_set_x(k, 0x0180);
    rupees.sprite_set_y(k, 0x0280);
    rupees.sprite_battle_mut().set_sprites_killed(4);
    rupees.sprite_battle_mut().set_times_hurt_by_sprites(0);
    rupees.rupee_pull_spawn_prize(k);
    assert_eq!(rupees.game_state.sprites.workspace.shared_scratch_a(), 2);
    assert_eq!(rupees.game_state.scratch_counter.value(), 0xff);
    assert_eq!(rupees.ram[NUM_SPRITES_KILLED_PREP], 0);
    assert_eq!(rupees.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES], 0);
    assert_eq!(rupees.sprite_slot_view(15).sprite_type(), 0xdb);
    assert_eq!(rupees.sprite_get_x(15), 0x0180);
    assert_eq!(rupees.sprite_get_y(15), 0x0280);
    assert_eq!(rupees.sprite_slot_view(15).x_velocity(), 18);
    assert_eq!(rupees.sprite_slot_view(15).y_velocity(), 16);
    assert_eq!(rupees.sprite_slot_view(15).stunned(), 255);
    assert_eq!(rupees.sprite_slot_view(15).delay_aux4(), 32);
    assert_eq!(rupees.sprite_slot_view(15).delay_aux3(), 32);
    assert_eq!(rupees.sprite_slot_view(15).z_velocity(), 32);
    assert_eq!(rupees.sprite_slot_view(12).sprite_type(), 0xdb);
    assert_eq!(rupees.sprite_slot_view(12).x_velocity(), (-18i8) as u8);
    assert_eq!(rupees.sprite_slot_view(12).y_velocity(), 16);

    let mut pink = fresh_state();
    pink.sprite_slot_view_mut(k).set_x_velocity(10);
    pink.sprite_slot_view_mut(k).set_y_velocity((-10i8) as u8);
    pink.pink_ball_handle_deceleration(k);
    assert_eq!(pink.sprite_slot_view(k).x_velocity(), 8);
    assert_eq!(pink.sprite_slot_view(k).y_velocity(), (-8i8) as u8);
    write_le_u16(&mut pink.ram, OAM_CUR_PTR, 0x0800);
    pink.sprite_set_x(k, 0x0100);
    pink.sprite_set_y(k, 0x0120);
    pink.set_frame_counter(0x18);
    pink.pink_ball_distress(k);
    assert_eq!(pink.sprite_slot_view(k).pause(), 0);

    let mut pink_msg = fresh_state();
    pink_msg.sprite_slot_view_mut(k).set_direction(3);
    pink_msg.sprite_slot_view_mut(k).set_x_velocity(0x12);
    pink_msg.sprite_slot_view_mut(k).set_y_velocity(0x34);
    pink_msg.pink_ball_handle_message(k);
    assert_eq!(
        pink_msg.game_state.messaging.dialogue_message_index.value(),
        0x15b
    );
    assert_eq!(pink_msg.sprite_slot_view(k).x_velocity(), 0xed);
    assert_eq!(pink_msg.sprite_slot_view(k).y_velocity(), 0xcb);
    assert_eq!(pink_msg.sprite_slot_view(k).delay_aux4(), 64);
    pink_msg.sprite_slot_view_mut(k).set_delay_aux4(0);
    pink_msg.inventory_items_mut().set_moon_pearl(1);
    pink_msg.pink_ball_handle_message(k);
    assert_eq!(
        pink_msg.game_state.messaging.dialogue_message_index.value(),
        0x15c
    );

    let mut bully_msg = fresh_state();
    bully_msg.sprite_slot_view_mut(k).set_direction(2);
    bully_msg.sprite_slot_view_mut(k).set_x_velocity(0x12);
    bully_msg.sprite_slot_view_mut(k).set_y_velocity(0x34);
    bully_msg.bully_handle_message(k);
    assert_eq!(
        bully_msg
            .game_state
            .messaging
            .dialogue_message_index
            .value(),
        0x15d
    );
    assert_eq!(bully_msg.sprite_slot_view(k).x_velocity(), 0xed);
    assert_eq!(bully_msg.sprite_slot_view(k).y_velocity(), 0xcb);
    assert_eq!(bully_msg.sprite_slot_view(k).delay_aux4(), 64);
    bully_msg.sprite_slot_view_mut(k).set_delay_aux4(0);
    bully_msg.inventory_items_mut().set_moon_pearl(1);
    bully_msg.bully_handle_message(k);
    assert_eq!(
        bully_msg
            .game_state
            .messaging
            .dialogue_message_index
            .value(),
        0x15e
    );

    let mut sasha = fresh_state();
    sasha.sprite_slot_view_mut(k).set_state(9);
    sasha.set_frame_counter(0x20);
    sasha.sasha_idle(k);
    assert_eq!(
        sasha.game_state.messaging.dialogue_message_index.value(),
        0x32
    );
    assert_eq!(sasha.sprite_slot_view(k).graphics(), 1);
    sasha.player_resources_mut().set_pendant_flags(4);
    sasha.save_progress_mut().set_map_icons_indicator(3);
    sasha.sasha_idle(k);
    assert_eq!(
        sasha.game_state.messaging.dialogue_message_index.value(),
        0x38
    );
    sasha.inventory_items_mut().set_boots(1);
    sasha.sasha_idle(k);
    assert_eq!(
        sasha.game_state.messaging.dialogue_message_index.value(),
        0x37
    );
    sasha.inventory_items_mut().set_ice_rod(1);
    sasha.sasha_idle(k);
    assert_eq!(
        sasha.game_state.messaging.dialogue_message_index.value(),
        0x34
    );
    sasha.player_resources_mut().set_pendant_flags(7);
    sasha.sasha_idle(k);
    assert_eq!(
        sasha.game_state.messaging.dialogue_message_index.value(),
        0x30
    );
    sasha.inventory_items_mut().set_sword_type(2);
    sasha.sasha_idle(k);
    assert_eq!(
        sasha.game_state.messaging.dialogue_message_index.value(),
        0x31
    );

    let mut old_man = fresh_state();
    let t = 2;
    old_man.tagalong_slot_mut(t).set_layer_bits(2);
    old_man.tagalong_slot_mut(t).set_position(0x0420, 0x0340);
    old_man.follower_link_state_mut().mark_lower_level();
    old_man.follower_state_mut().set_indicator(6);
    old_man.follower_link_state_mut().set_speed_setting(9);
    old_man.old_man_revert_to_sprite(t);
    assert_eq!(old_man.sprite_slot_view(15).sprite_type(), 0xad);
    assert_eq!(old_man.sprite_slot_view(15).direction(), 2);
    assert_eq!(old_man.sprite_slot_view(15).head_direction(), 2);
    assert_eq!(old_man.sprite_get_y(15), 0x0342);
    assert_eq!(old_man.sprite_get_x(15), 0x0422);
    assert_eq!(old_man.sprite_slot_view(15).floor(), 1);
    assert_eq!(old_man.sprite_slot_view(15).ignore_projectile(), 1);
    assert_eq!(old_man.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(
        old_man.game_state.player.follower_link.immobilized_flag(),
        1
    );
    assert_eq!(
        old_man
            .game_state
            .player
            .follower_link
            .sprite_damage_disable_timer(),
        1
    );
    assert_eq!(old_man.game_state.sprites.follower_runtime.indicator(), 0);
    assert_eq!(old_man.game_state.player.follower_link.speed_setting(), 0);

    let mut apple = fresh_state();
    apple.sprite_slot_view_mut(k).set_state(9);
    apple.sprite_set_x(k, 0x0200);
    apple.sprite_set_y(k, 0x0300);
    apple.set_frame_counter(0);
    apple.ram[0x0fa1] = 0;
    apple.spawn_apple(k);
    assert_eq!(apple.sprite_slot_view(15).sprite_type(), 0xac);
    assert_eq!(apple.sprite_get_x(15), 0x0200);
    assert_eq!(apple.sprite_get_y(15), 0x0300);
    assert_eq!(apple.sprite_slot_view(15).ai_state(), 1);
    assert_eq!(apple.sprite_slot_view(15).a(), 255);
    assert_eq!(apple.sprite_slot_view(15).z(), 8);
    assert_eq!(apple.sprite_slot_view(15).z_velocity(), 22);
    assert_eq!(apple.sprite_slot_view(15).x_velocity(), 10);
    assert_eq!(apple.sprite_slot_view(15).y_velocity(), 3);

    let mut transmute = fresh_state();
    transmute.sprite_slot_view_mut(k).set_sprite_type(0xd8);
    transmute.sprite_slot_view_mut(k).set_health(7);
    transmute.sprite_transmute_to_bomb(k);
    assert_eq!(transmute.sprite_slot_view(k).sprite_type(), 0x4a);
    assert_eq!(transmute.sprite_slot_view(k).c(), 1);
    assert_eq!(transmute.sprite_slot_view(k).delay_aux1(), 255);
    assert_eq!(transmute.sprite_slot_view(k).flags3(), 0x18);
    assert_eq!(transmute.sprite_slot_view(k).oam_flags(), 8);
    assert_eq!(transmute.sprite_slot_view(k).health(), 0);

    let mut sluggula = fresh_state();
    sluggula.sprite_slot_view_mut(k).set_state(9);
    sluggula.sprite_set_x(k, 0x0120);
    sluggula.sprite_set_y(k, 0x0340);
    sluggula.sluggula_drop_bomb(k);
    assert_eq!(sluggula.sprite_slot_view(11).sprite_type(), 0x4a);
    assert_eq!(sluggula.sprite_get_x(11), 0x0120);
    assert_eq!(sluggula.sprite_get_y(11), 0x0340);
    assert_eq!(sluggula.sprite_slot_view(11).c(), 1);
    assert_eq!(sluggula.sprite_slot_view(11).delay_aux1(), 255);
    assert_eq!(sluggula.sprite_slot_view(11).flags3(), 0x18);
    assert_eq!(sluggula.sprite_slot_view(11).oam_flags(), 8);
    assert_eq!(sluggula.sprite_slot_view(11).health(), 0);

    let mut tree_bomb = fresh_state();
    tree_bomb.sprite_slot_view_mut(k).set_state(9);
    tree_bomb.sprite_set_x(k, 0x0048);
    tree_bomb.sprite_set_y(k, 0x0058);
    tree_bomb.talking_tree_spawn_bomb(k);
    assert_eq!(tree_bomb.sprite_slot_view(15).sprite_type(), 0x4a);
    assert_eq!(tree_bomb.sprite_get_x(15), 0x0048);
    assert_eq!(tree_bomb.sprite_get_y(15), 0x0058);
    assert_eq!(tree_bomb.sprite_slot_view(15).c(), 1);
    assert_eq!(tree_bomb.sprite_slot_view(15).delay_aux1(), 64);
    assert_eq!(tree_bomb.sprite_slot_view(15).flags3(), 0x18);
    assert_eq!(tree_bomb.sprite_slot_view(15).oam_flags(), 8);
    assert_eq!(tree_bomb.sprite_slot_view(15).health(), 0);
    assert_eq!(tree_bomb.sprite_slot_view(15).y_velocity(), 24);
    assert_eq!(tree_bomb.sprite_slot_view(15).z_velocity(), 18);

    let mut tree_eye = fresh_state();
    tree_eye.sprite_slot_view_mut(k).set_state(9);
    tree_eye.sprite_set_x(k, 0x0200);
    tree_eye.sprite_set_y(k, 0x0300);
    tree_eye.sprite_prep_talking_tree_spawn_eyeball(k, 1);
    assert_eq!(tree_eye.sprite_slot_view(15).sprite_type(), 0x25);
    assert_eq!(tree_eye.sprite_slot_view(15).head_direction(), 1);
    assert_eq!(tree_eye.sprite_get_x(15), 0x020e);
    assert_eq!(tree_eye.sprite_get_y(15), 0x02f5);
    assert_eq!(tree_eye.sprite_slot_view(15).a(), 0x0e);
    assert_eq!(tree_eye.sprite_slot_view(15).b(), 0x02);
    assert_eq!(tree_eye.sprite_slot_view(15).c(), 0xf5);
    assert_eq!(tree_eye.sprite_slot_view(15).e(), 0x02);
    assert_eq!(tree_eye.sprite_slot_view(15).subtype2(), 1);

    let mut pirogusu = fresh_state();
    pirogusu.set_frame_counter(k as u8);
    pirogusu.garnish_slot_view_mut(14).set_garnish_type(1);
    pirogusu.sprite_set_x(k, 0x0110);
    pirogusu.sprite_set_y(k, 0x0220);
    pirogusu.pirogusu_spawn_splash(k);
    assert_eq!(pirogusu.garnish_slot_view(13).garnish_type(), 11);
    assert_eq!(
        pirogusu.game_state.sprites.garnish_runtime.active_type(),
        11
    );
    assert_eq!(pirogusu.garnish_slot_view(13).x_low(), 0x15);
    assert_eq!(pirogusu.garnish_slot_view(13).x_high(), 0x01);
    assert_eq!(pirogusu.garnish_slot_view(13).y_low(), 0x34);
    assert_eq!(pirogusu.garnish_slot_view(13).y_high(), 0x02);
    assert_eq!(pirogusu.garnish_slot_view(13).countdown(), 15);

    let mut lightning = fresh_state();
    lightning.garnish_slot_view_mut(29).set_garnish_type(1);
    lightning.sprite_set_x(k, 0x0123);
    lightning.sprite_set_y(k, 0x02f4);
    lightning.sprite_slot_view_mut(k).set_a(7);
    lightning.lightning_spawn_garnish(k);
    assert_eq!(lightning.garnish_slot_view(28).garnish_type(), 9);
    assert_eq!(
        lightning.game_state.sprites.garnish_runtime.active_type(),
        9
    );
    assert_eq!(lightning.garnish_slot_view(28).sprite(), 7);
    assert_eq!(lightning.garnish_slot_view(28).x_low(), 0x23);
    assert_eq!(lightning.garnish_slot_view(28).x_high(), 0x01);
    assert_eq!(lightning.garnish_slot_view(28).y_low(), 0x04);
    assert_eq!(lightning.garnish_slot_view(28).y_high(), 0x03);
    assert_eq!(lightning.garnish_slot_view(28).countdown(), 32);

    let mut laser = fresh_state();
    laser.garnish_slot_view_mut(29).set_garnish_type(1);
    laser.sprite_set_x(k, 0x0034);
    laser.sprite_set_y(k, 0x00f0);
    laser.sprite_slot_view_mut(k).set_graphics(5);
    laser.sprite_slot_view_mut(k).set_floor(2);
    laser.laser_beam_build_up_garnish(k);
    assert_eq!(laser.garnish_slot_view(28).garnish_type(), 4);
    assert_eq!(laser.game_state.sprites.garnish_runtime.active_type(), 4);
    assert_eq!(laser.garnish_slot_view(28).x_low(), 0x34);
    assert_eq!(laser.garnish_slot_view(28).x_high(), 0x00);
    assert_eq!(laser.garnish_slot_view(28).y_low(), 0x00);
    assert_eq!(laser.garnish_slot_view(28).y_high(), 0x01);
    assert_eq!(laser.garnish_slot_view(28).countdown(), 16);
    assert_eq!(laser.garnish_slot_view(28).oam_flags(), 5);
    assert_eq!(laser.garnish_slot_view(28).sprite(), k as u8);
    assert_eq!(laser.garnish_slot_view(28).floor(), 2);

    let mut logic = fresh_state();
    assert!(!logic.octoballoon_find());
    logic.sprite_slot_view_mut(10).set_state(9);
    logic.sprite_slot_view_mut(10).set_sprite_type(0x10);
    assert!(logic.octoballoon_find());

    assert!(!logic.potion_cauldron_check_bottles());
    logic.inventory_items_mut().set_bottle(2, 2);
    assert!(logic.potion_cauldron_check_bottles());
    logic.potion_cauldron_go_beep(k);
    assert_eq!(
        logic.game_state.system_signals.sound_effect_1() & 0x3f,
        0x3c
    );

    logic.player_resources_mut().set_rupees_goal(19);
    assert!(!logic.dark_world_hint_npc_handle_payment());
    assert_eq!(
        logic.game_state.inventory.player_resources.rupees_goal(),
        19
    );
    logic.player_resources_mut().set_rupees_goal(20);
    assert!(logic.dark_world_hint_npc_handle_payment());
    assert_eq!(logic.game_state.inventory.player_resources.rupees_goal(), 0);
    logic.sprite_slot_view_mut(k).set_ai_state(0);
    logic.dark_world_hint_npc_idle(k);
    assert_eq!(
        logic.game_state.messaging.dialogue_message_index.value(),
        0xfe
    );
    assert_eq!(logic.sprite_slot_view(k).ai_state(), 0);

    logic.set_submodule(2);
    logic.dialogue_message_index_mut().set_value(0xc9);
    logic.fairy_check_if_touchable(k);
    assert_eq!(logic.sprite_slot_view(k).delay_aux4(), 40);
    logic.sprite_slot_view_mut(k).set_delay_aux4(0);
    logic.dialogue_message_index_mut().set_value(0xcb);
    logic.fairy_check_if_touchable(k);
    assert_eq!(logic.sprite_slot_view(k).delay_aux4(), 0);

    let mut buzzblob = fresh_state();
    buzzblob.buzzblob_select_new_direction(k);
    assert_eq!(buzzblob.sprite_slot_view(k).x_velocity(), 3);
    assert_eq!(buzzblob.sprite_slot_view(k).y_velocity(), 0);
    assert_eq!(buzzblob.sprite_slot_view(k).delay_main(), 48);

    let mut lumberjack = fresh_state();
    lumberjack
        .sprite_workspace_mut()
        .set_current_sprite_x(0x0100);
    lumberjack
        .sprite_workspace_mut()
        .set_current_sprite_y(0x0200);
    lumberjack.follower_link_state_mut().set_x(0x0100);
    lumberjack.follower_link_state_mut().set_y(0x0200);
    assert!(lumberjack.lumberjack_check_proximity(k, 0));
    lumberjack.follower_link_state_mut().set_x(0x0200);
    assert!(!lumberjack.lumberjack_check_proximity(k, 0));

    let mut blind_laser = fresh_state();
    blind_laser.garnish_slot_view_mut(29).set_garnish_type(1);
    blind_laser.sprite_set_x(k, 0x0456);
    blind_laser.sprite_set_y(k, 0x0789);
    blind_laser.sprite_slot_view_mut(k).set_graphics(6);
    blind_laser.blind_laser_spawn_trail_garnish(k);
    assert_eq!(blind_laser.garnish_slot_view(28).garnish_type(), 15);
    assert_eq!(
        blind_laser.game_state.sprites.garnish_runtime.active_type(),
        15
    );
    assert_eq!(blind_laser.garnish_slot_view(28).oam_flags(), 6);
    assert_eq!(blind_laser.garnish_slot_view(28).sprite(), k as u8);
    assert_eq!(blind_laser.garnish_slot_view(28).x_low(), 0x56);
    assert_eq!(blind_laser.garnish_slot_view(28).x_high(), 0x04);
    assert_eq!(blind_laser.garnish_slot_view(28).y_low(), 0x99);
    assert_eq!(blind_laser.garnish_slot_view(28).y_high(), 0x07);
    assert_eq!(blind_laser.garnish_slot_view(28).countdown(), 10);

    let mut runner_dust = fresh_state();
    runner_dust.sprite_slot_view_mut(k).set_die_action(14);
    runner_dust.running_boy_spawn_dust_garnish(k);
    assert_eq!(
        runner_dust.game_state.sprites.garnish_runtime.active_type(),
        0
    );
    runner_dust.sprite_slot_view_mut(k).set_die_action(15);
    runner_dust.sprite_set_x(k, 0x0100);
    runner_dust.sprite_set_y(k, 0x0200);
    runner_dust.garnish_slot_view_mut(29).set_garnish_type(1);
    runner_dust.running_boy_spawn_dust_garnish(k);
    assert_eq!(runner_dust.garnish_slot_view(28).garnish_type(), 20);
    assert_eq!(
        runner_dust.game_state.sprites.garnish_runtime.active_type(),
        20
    );
    assert_eq!(runner_dust.garnish_slot_view(28).x_low(), 0x04);
    assert_eq!(runner_dust.garnish_slot_view(28).x_high(), 0x01);
    assert_eq!(runner_dust.garnish_slot_view(28).y_low(), 0x1c);
    assert_eq!(runner_dust.garnish_slot_view(28).y_high(), 0x02);
    assert_eq!(runner_dust.garnish_slot_view(28).countdown(), 10);

    let mut cd = fresh_state();
    cd.sprite_slot_view_mut(k).set_subtype2(6);
    cd.sprite_cd_spawn_garnish(k);
    assert_eq!(cd.game_state.sprites.garnish_runtime.active_type(), 0);
    cd.sprite_slot_view_mut(k).set_subtype2(7);
    cd.garnish_slot_view_mut(29).set_garnish_type(1);
    cd.sprite_set_x(k, 0x0033);
    cd.sprite_set_y(k, 0x0044);
    cd.sprite_cd_spawn_garnish(k);
    assert_eq!(cd.sprite_slot_view(k).subtype2(), 8);
    assert_eq!(cd.game_state.system_signals.sound_effect_2() & 0x3f, 0x14);
    assert_eq!(cd.garnish_slot_view(28).garnish_type(), 0x0c);
    assert_eq!(cd.game_state.sprites.garnish_runtime.active_type(), 0x0c);
    assert_eq!(cd.garnish_slot_view(28).sprite(), k as u8);
    assert_eq!(cd.garnish_slot_view(28).x_low(), 0x33);
    assert_eq!(cd.garnish_slot_view(28).y_low(), 0x54);
    assert_eq!(cd.garnish_slot_view(28).countdown(), 127);

    let mut hint = fresh_state();
    hint.sprite_slot_view_mut(k).set_ai_state(2);
    hint.dark_world_hint_npc_restore_health(k);
    assert_eq!(
        hint.game_state.inventory.player_resources.heart_filler(),
        0xa0
    );
    assert_eq!(hint.sprite_slot_view(k).ai_state(), 0);

    let mut pipe = fresh_state();
    pipe.follower_link_state_mut().set_position_mode(7);
    pipe.player_state_mut().set_direction_lock(9);
    pipe.ancilla_slot_view_mut(3).set_ancilla_type(0x31);
    assert!(!pipe.pipe_validate_entry());
    assert_eq!(pipe.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(pipe.game_state.player.follower_link.direction_lock(), 0);
    assert_eq!(pipe.ancilla_slot_view(3).ancilla_type(), 0);
    pipe.follower_link_state_mut().set_state_bits(0x80);
    assert!(pipe.pipe_validate_entry());
    pipe.follower_link_state_mut().clear_state_bits();
    pipe.follower_link_state_mut().set_auxiliary_state(2);
    assert!(pipe.pipe_validate_entry());

    let mut hobo_smoke = fresh_state();
    hobo_smoke.sprite_slot_view_mut(k).set_state(9);
    hobo_smoke.sprite_set_x(k, 0x0030);
    hobo_smoke.sprite_set_y(k, 0x0040);
    hobo_smoke.sprite_prep_hobo_spawn_smoke(k);
    assert_eq!(hobo_smoke.sprite_slot_view(15).sprite_type(), 0x2b);
    assert_eq!(hobo_smoke.sprite_get_x(15), 0x0030);
    assert_eq!(hobo_smoke.sprite_get_y(15), 0x0040);
    assert_eq!(hobo_smoke.sprite_slot_view(15).subtype2(), 0);
    assert_eq!(hobo_smoke.sprite_slot_view(15).ignore_projectile(), 0);

    let mut hobo_fire = fresh_state();
    hobo_fire.sprite_slot_view_mut(k).set_state(9);
    hobo_fire.sprite_slot_view_mut(15).set_oam_flags(0xff);
    hobo_fire.sprite_prep_hobo_spawn_fire(k);
    assert_eq!(hobo_fire.sprite_slot_view(15).sprite_type(), 0x2b);
    assert_eq!(hobo_fire.sprite_get_x(15), 0x0194);
    assert_eq!(hobo_fire.sprite_get_y(15), 0x003f);
    assert_eq!(hobo_fire.sprite_slot_view(15).subtype2(), 2);
    assert_eq!(hobo_fire.sprite_slot_view(15).ignore_projectile(), 2);
    assert_eq!(hobo_fire.sprite_slot_view(15).flags2(), 0);
    assert_eq!(hobo_fire.sprite_slot_view(15).oam_flags() & 0x0f, 0x03);

    let mut hobo_bubble = fresh_state();
    hobo_bubble.sprite_slot_view_mut(k).set_state(9);
    hobo_bubble.sprite_set_x(k, 0x0050);
    hobo_bubble.sprite_set_y(k, 0x0060);
    assert_eq!(hobo_bubble.hobo_spawn_bubble(k), 15);
    assert_eq!(hobo_bubble.sprite_slot_view(15).sprite_type(), 0x2b);
    assert_eq!(hobo_bubble.sprite_get_x(15), 0x0050);
    assert_eq!(hobo_bubble.sprite_get_y(15), 0x0060);
    assert_eq!(hobo_bubble.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(hobo_bubble.sprite_slot_view(15).z_velocity(), 2);
    assert_eq!(hobo_bubble.sprite_slot_view(15).delay_main(), 96);
    assert_eq!(hobo_bubble.sprite_slot_view(15).delay_aux1(), 48);
    assert_eq!(hobo_bubble.sprite_slot_view(15).ignore_projectile(), 48);
    assert_eq!(hobo_bubble.sprite_slot_view(15).flags2(), 0);

    let mut hobo_smoke_active = fresh_state();
    hobo_smoke_active.sprite_slot_view_mut(k).set_state(9);
    hobo_smoke_active.sprite_set_x(k, 0x0070);
    hobo_smoke_active.sprite_set_y(k, 0x0080);
    hobo_smoke_active.hobo_spawn_smoke(k);
    assert_eq!(hobo_smoke_active.sprite_slot_view(15).sprite_type(), 0x2b);
    assert_eq!(hobo_smoke_active.sprite_get_x(15), 0x0070);
    assert_eq!(hobo_smoke_active.sprite_get_y(15), 0x007c);
    assert_eq!(hobo_smoke_active.sprite_slot_view(15).subtype2(), 3);
    assert_eq!(hobo_smoke_active.sprite_slot_view(15).z_velocity(), 7);
    assert_eq!(hobo_smoke_active.sprite_slot_view(15).delay_main(), 96);
    assert_eq!(
        hobo_smoke_active.sprite_slot_view(15).ignore_projectile(),
        96
    );
    assert_eq!(hobo_smoke_active.sprite_slot_view(15).flags2(), 0);

    let mut hobo = fresh_state();
    hobo.sprite_slot_view_mut(k).set_state(9);
    hobo.sprite_set_x(k, 0x0080);
    hobo.sprite_set_y(k, 0x0090);
    hobo.save_progress_mut().set_progress_indicator_3(1);
    hobo.sprite_prep_hobo(k);
    assert_eq!(hobo.sprite_slot_view(0).ai_state(), 3);
    assert_eq!(hobo.sprite_slot_view(0).ignore_projectile(), 1);
    assert_eq!(hobo.sprite_slot_view(15).state(), 9);
    assert_eq!(hobo.sprite_slot_view(1).state(), 0);
    assert_eq!(hobo.sprite_slot_view(15).sprite_type(), 0x2b);
    assert_eq!(hobo.sprite_slot_view(15).subtype2(), 2);
    assert_eq!(hobo.sprite_get_x(15), 0x0194);
    assert_eq!(hobo.sprite_get_y(15), 0x003f);

    let mut tree = fresh_state();
    tree.sprite_slot_view_mut(k).set_state(9);
    tree.sprite_set_x(k, 0x0120);
    tree.sprite_set_y(k, 0x0240);
    tree.sprite_prep_talking_tree(k);
    assert_eq!(tree.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(tree.sprite_get_x(k), 0x0118);
    assert_eq!(tree.sprite_slot_view(15).sprite_type(), 0x25);
    assert_eq!(tree.sprite_slot_view(15).head_direction(), 0);
    assert_eq!(tree.sprite_get_x(15), 0x0114);
    assert_eq!(tree.sprite_get_y(15), 0x0235);
    assert_eq!(tree.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(tree.sprite_slot_view(14).sprite_type(), 0x25);
    assert_eq!(tree.sprite_slot_view(14).head_direction(), 1);
    assert_eq!(tree.sprite_get_x(14), 0x0126);
    assert_eq!(tree.sprite_get_y(14), 0x0235);
    assert_eq!(tree.sprite_slot_view(14).a(), 0x26);
    assert_eq!(tree.sprite_slot_view(14).b(), 0x01);
    assert_eq!(tree.sprite_slot_view(14).c(), 0x35);
    assert_eq!(tree.sprite_slot_view(14).e(), 0x02);
}

#[test]
fn shopkeeper_and_antifairy_circle_prep_spawn_expected_helpers() {
    let k = 4;

    let mut shop = fresh_state();
    shop.sprite_slot_view_mut(k).set_state(9);
    shop.set_dungeon_room_index(0x0f);
    shop.sprite_set_x(k, 0x0200);
    shop.sprite_set_y(k, 0x0100);
    shop.sprite_prep_shopkeeper(k);
    assert_eq!(shop.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(shop.sprite_slot_view(k).flags2() & 2, 2);
    assert_eq!(shop.sprite_slot_view(k).oam_flags() & 12, 12);
    assert_eq!(shop.sprite_slot_view(k).flags3() & 16, 16);
    for (slot, what, x) in [
        (12, 7, 0x0200u16.wrapping_sub(44)),
        (11, 8, 0x0200u16.wrapping_add(8)),
        (10, 12, 0x0200u16.wrapping_add(60)),
    ] {
        assert_eq!(shop.sprite_slot_view(slot).state(), 9);
        assert_eq!(shop.sprite_slot_view(slot).sprite_type(), 0xbb);
        assert_eq!(shop.sprite_slot_view(slot).ignore_projectile(), what);
        assert_eq!(shop.sprite_slot_view(slot).subtype2(), what);
        assert_eq!(shop.sprite_get_x(slot), x);
        assert_eq!(shop.sprite_get_y(slot), 0x0127);
        assert_eq!(shop.sprite_slot_view(slot).flags2() & 4, 4);
    }

    let mut minigame = fresh_state();
    minigame.sprite_slot_view_mut(k).set_state(9);
    minigame.set_dungeon_room_index(0x06);
    minigame.sprite_prep_shopkeeper(k);
    assert_eq!(minigame.sprite_slot_view(k).subtype2(), 1);
    assert_eq!(minigame.sprite_slot_view(k).graphics(), 1);
    assert_eq!(minigame.ram[MINIGAME_CREDITS_PREP], 0xff);

    let mut terminate = fresh_state();
    terminate.ancilla_slot_view_mut(0).set_ancilla_type(0x22);
    terminate.ancilla_slot_view_mut(1).set_ancilla_type(0x21);
    terminate.ancilla_slot_view_mut(4).set_ancilla_type(0x22);
    terminate.ram[ANCILLA_AUX_TIMER] = 9;
    terminate.ram[ANCILLA_AUX_TIMER + 1] = 9;
    terminate.ram[ANCILLA_AUX_TIMER + 4] = 9;
    terminate.shop_keeper_rapid_terminate_receive_item();
    assert_eq!(terminate.ram[ANCILLA_AUX_TIMER], 1);
    assert_eq!(terminate.ram[ANCILLA_AUX_TIMER + 1], 9);
    assert_eq!(terminate.ram[ANCILLA_AUX_TIMER + 4], 1);

    let mut bat = fresh_state();
    bat.sprite_spawn_bat_crash_cutscene();
    assert_eq!(bat.sprite_slot_view(15).sprite_type(), 0x37);
    assert_eq!(bat.sprite_slot_view(15).y_velocity(), 0);
    assert_eq!(bat.sprite_slot_view(15).b(), 0);
    assert_eq!(bat.sprite_slot_view(15).direction(), 0);
    assert_eq!(bat.sprite_slot_view(15).floor(), 0);
    assert_eq!(bat.sprite_slot_view(15).subtype2(), 1);
    assert_eq!(bat.sprite_slot_view(15).flags2(), 1);
    assert_eq!(bat.sprite_slot_view(15).flags3(), 1);
    assert_eq!(bat.sprite_slot_view(15).oam_flags(), 1);
    assert_eq!(bat.sprite_get_x(15), 0x07cc);
    assert_eq!(bat.sprite_get_y(15), 0x0632);
    assert_eq!(bat.sprite_slot_view(15).deflection_bits(), 128);

    let mut circle = fresh_state();
    circle.sprite_slot_view_mut(k).set_state(9);
    circle.sprite_set_x(k, 0x0100);
    circle.sprite_set_y(k, 0x0200);
    circle.sprite_slot_view_mut(k).set_a(9);
    circle.sprite_slot_view_mut(k).set_b(9);
    circle.sprite_prep_antifairy_circle(k);
    assert_eq!(circle.sprite_get_x(k), 0x00f6);
    assert_eq!(circle.sprite_slot_view(k).y_velocity(), (-18i8) as u8);
    assert_eq!(circle.sprite_slot_view(k).x_velocity(), 0);
    assert_eq!(circle.sprite_slot_view(k).a(), 0);
    assert_eq!(circle.sprite_slot_view(k).b(), 0);
    assert_eq!(circle.game_state.scratch_counter.value(), 0xff);

    for (slot, x, y, xv, yv, a, b) in [
        (
            15,
            0x00f6u16.wrapping_add(10),
            0x0200u16.wrapping_add(10),
            (-18i8) as u8,
            0,
            0,
            1,
        ),
        (14, 0x00f6u16.wrapping_add(20), 0x0200, 0, 18, 1, 1),
        (
            13,
            0x00f6u16.wrapping_add(10),
            0x0200u16.wrapping_sub(10),
            18,
            0,
            1,
            0,
        ),
    ] {
        assert_eq!(circle.sprite_slot_view(slot).state(), 9);
        assert_eq!(circle.sprite_slot_view(slot).sprite_type(), 0x82);
        assert_eq!(circle.sprite_get_x(slot), x);
        assert_eq!(circle.sprite_get_y(slot), y);
        assert_eq!(circle.sprite_slot_view(slot).x_velocity(), xv);
        assert_eq!(circle.sprite_slot_view(slot).y_velocity(), yv);
        assert_eq!(circle.sprite_slot_view(slot).a(), a);
        assert_eq!(circle.sprite_slot_view(slot).b(), b);
    }
}

#[test]
fn medallion_table_and_eyegore_prep_match_room_and_item_gates() {
    let k = 7;

    let mut bombos = fresh_state();
    bombos.set_overworld_screen(2);
    bombos.inventory_items_mut().set_bombos(1);
    bombos.sprite_slot_view_mut(k).set_x_low(0xf9);
    bombos.sprite_prep_medallion_table(k);
    assert_eq!(bombos.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(bombos.sprite_slot_view(k).x_low(), 1);
    assert_eq!(bombos.sprite_slot_view(k).graphics(), 4);
    assert_eq!(bombos.sprite_slot_view(k).ai_state(), 3);

    let mut ether_only_on_bombos_screen = fresh_state();
    ether_only_on_bombos_screen.set_overworld_screen(2);
    ether_only_on_bombos_screen
        .inventory_items_mut()
        .set_ether(1);
    ether_only_on_bombos_screen.sprite_prep_medallion_table(k);
    assert_eq!(
        ether_only_on_bombos_screen
            .sprite_slot_view(k)
            .ignore_projectile(),
        1
    );
    assert_eq!(
        ether_only_on_bombos_screen.sprite_slot_view(k).graphics(),
        0
    );
    assert_eq!(
        ether_only_on_bombos_screen.sprite_slot_view(k).ai_state(),
        0
    );

    let mut ether = fresh_state();
    ether.set_overworld_screen(3);
    ether.inventory_items_mut().set_ether(1);
    ether.sprite_slot_view_mut(k).set_x_low(0x20);
    ether.sprite_prep_medallion_table(k);
    assert_eq!(ether.sprite_slot_view(k).ignore_projectile(), 1);
    assert_eq!(ether.sprite_slot_view(k).x_low(), 0x20);
    assert_eq!(ether.sprite_slot_view(k).graphics(), 4);
    assert_eq!(ether.sprite_slot_view(k).ai_state(), 3);

    let mut eyegore = fresh_state();
    eyegore.dungeon_room_tracking_mut().set_room_index2(75);
    eyegore.sprite_slot_view_mut(k).set_sprite_type(0x83);
    eyegore.sprite_slot_view_mut(k).set_b(0xff);
    eyegore.sprite_slot_view_mut(k).set_deflection_bits(0xaa);
    eyegore.sprite_prep_eyegore(k);
    assert_eq!(eyegore.sprite_slot_view(k).b(), 0);
    assert_eq!(eyegore.sprite_slot_view(k).deflection_bits(), 0);

    let mut untouched = fresh_state();
    untouched.dungeon_room_tracking_mut().set_room_index2(74);
    untouched.sprite_slot_view_mut(k).set_b(4);
    untouched.sprite_slot_view_mut(k).set_deflection_bits(0xaa);
    untouched.sprite_prep_eyegore(k);
    assert_eq!(untouched.sprite_slot_view(k).b(), 4);
    assert_eq!(untouched.sprite_slot_view(k).deflection_bits(), 0xaa);
}
