use super::*;
use crate::rom_random::RomRandomResult;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

#[test]
fn rat_random_run_delay_preserves_rng_carry_through_and_adc() {
    assert_eq!(rat_random_run_delay(RomRandomResult::new(1, false)), 0x41);
    assert_eq!(rat_random_run_delay(RomRandomResult::new(1, true)), 0x42);
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
    assert_eq!(read_le_u16(&s.ram, SAVE_DUNG_INFO + 0x109 * 2), 0x0082);
    assert_eq!(s.ram[SAVE_DUNG_INFO + 0x109], 0);
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
