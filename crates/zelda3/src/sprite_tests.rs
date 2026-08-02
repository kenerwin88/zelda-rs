use super::*;

fn fresh_state() -> Box<ZeldaState> {
    Box::new(ZeldaState::new())
}

fn empty_hit_box() -> SpriteHitBox {
    SpriteHitBox {
        r0_xlo: 0,
        r8_xhi: 0,
        r1_ylo: 0,
        r9_yhi: 0,
        r2: 0,
        r3: 0,
        r4_spr_xlo: 0,
        r10_spr_xhi: 0,
        r5_spr_ylo: 0,
        r11_spr_yhi: 0,
        r6_spr_xsize: 0,
        r7_spr_ysize: 0,
    }
}

#[test]
fn sprite_func3_sets_death_delay_and_flags() {
    let mut s = fresh_state();
    let k = 5;
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_slot_view_mut(k).set_delay_main(0xaa);
    s.sprite_slot_view_mut(k).set_flags2(0xbb);

    s.sprite_func3(k);

    assert_eq!(s.sprite_slot_view(k).state(), 6);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 31);
    assert_eq!(s.sprite_slot_view(k).flags2(), 3);
}

#[test]
fn sprite_func8_resets_sound_then_queues_panned_sfx2() {
    let mut s = fresh_state();
    let k = 4;
    s.set_sound_effect_1(0xff);
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_slot_view_mut(k).set_delay_main(0);
    s.sprite_set_x(k, 0x0170);
    s.set_bg2_x(0x0100);
    let expected_sfx = s.sprite_calculate_sfx_pan(k) | 0x20;

    s.sprite_func8(k);

    assert_eq!(s.sprite_slot_view(k).state(), 1);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 0x1f);
    assert_eq!(s.game_state.system_signals.sound_effect_1(), expected_sfx);
}

#[test]
fn sprite_func22_sets_transition_state_and_advances_rng() {
    let mut s = fresh_state();
    let k = 6;
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_slot_view_mut(k).set_delay_main(0xaa);
    s.sprite_slot_view_mut(k).set_ai_state(0xbb);
    s.sprite_slot_view_mut(k).set_flags2(0xcc);
    s.sprite_set_x(k, 0x0040);
    s.set_bg2_x(0x0000);
    let expected_sfx = s.sprite_calculate_sfx_pan(k) | 0x28;

    s.sprite_func22(k);

    assert_eq!(s.game_state.system_signals.sound_effect_1(), expected_sfx);
    assert_eq!(s.sprite_slot_view(k).state(), 3);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 15);
    assert_eq!(s.sprite_slot_view(k).ai_state(), 0);
    assert_eq!(s.sprite_slot_view(k).flags2(), 3);
}

#[test]
fn outdoor_secret_gate_uses_the_shared_rng_sequence() {
    let mut s = fresh_state();
    s.set_indoor_flag(0);
    s.set_frame_counter(0x3e);
    s.set_rng_seed(0x88);
    s.dungeon_secret_scratch_mut().set_pending_kind(1);

    let mut expected = s.clone();
    let expected_seed = expected.get_random_number();
    assert_ne!(expected_seed & 8, 0);

    s.sprite_spawn_secret(14);

    assert_eq!(s.game_state.world.region.rng_seed(), expected_seed);
    assert!((0..16).all(|slot| s.sprite_slot_view(slot).state() == 0));
}

#[test]
fn outdoor_powder_secret_consumes_two_shared_rng_values() {
    let mut s = fresh_state();
    s.set_indoor_flag(0);
    s.set_frame_counter(0x99);
    s.set_rng_seed(0xa8);
    s.dungeon_secret_scratch_mut().set_pending_kind(4);

    let mut expected = s.clone();
    let gate_roll = expected.get_random_number();
    assert_eq!(gate_roll & 8, 0);
    let expected_seed = expected.get_random_number();

    s.sprite_spawn_secret(14);

    assert_eq!(s.game_state.world.region.rng_seed(), expected_seed);
}

#[test]
fn throwable_scenery_transmute_if_valid_only_transmutes_throwable_scenery() {
    let k = 5;

    let mut other = fresh_state();
    other.sprite_slot_view_mut(k).set_sprite_type(0x12);
    other.ram[REPULSESPARK_TIMER_SPRITE] = 7;
    other.sprite_slot_view_mut(k).set_delay_main(0xaa);
    other.sprite_slot_view_mut(k).set_state(9);
    other.sprite_slot_view_mut(k).set_flags2(0x20);
    other.throwable_scenery_transmute_if_valid(k);
    assert_eq!(other.ram[REPULSESPARK_TIMER_SPRITE], 7);
    assert_eq!(other.sprite_slot_view(k).delay_main(), 0xaa);
    assert_eq!(other.sprite_slot_view(k).state(), 9);
    assert_eq!(other.sprite_slot_view(k).flags2(), 0x20);

    let mut scenery = fresh_state();
    scenery.sprite_slot_view_mut(k).set_sprite_type(0xec);
    scenery.ram[REPULSESPARK_TIMER_SPRITE] = 7;
    scenery.sprite_slot_view_mut(k).set_delay_main(0xaa);
    scenery.sprite_slot_view_mut(k).set_state(9);
    scenery.sprite_slot_view_mut(k).set_flags2(0x20);
    scenery.throwable_scenery_transmute_if_valid(k);
    assert_eq!(scenery.ram[REPULSESPARK_TIMER_SPRITE], 0);
    assert_eq!(
        scenery.game_state.system_signals.sound_effect_1() & 0x3f,
        0x1f
    );
    assert_eq!(scenery.sprite_slot_view(k).delay_main(), 31);
    assert_eq!(scenery.sprite_slot_view(k).state(), 6);
    assert_eq!(scenery.sprite_slot_view(k).flags2(), 0x24);
}

#[test]
fn sprite_apply_ricochet_inverts_halves_and_transmutes_if_valid() {
    let k = 5;
    let mut s = fresh_state();
    s.sprite_slot_view_mut(k).set_sprite_type(0xec);
    s.sprite_slot_view_mut(k).set_x_velocity(0x10);
    s.sprite_slot_view_mut(k).set_y_velocity(0xf0);
    s.ram[REPULSESPARK_TIMER_SPRITE] = 9;
    s.sprite_slot_view_mut(k).set_flags2(0x03);

    s.sprite_apply_ricochet(k);

    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0xf8);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0x08);
    assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 0);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 31);
    assert_eq!(s.sprite_slot_view(k).state(), 6);
    assert_eq!(s.sprite_slot_view(k).flags2(), 0x07);
}

#[test]
fn sprite_func18_changes_type_resets_damage_and_spawns_poof_garnish() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_slot_view_mut(k).set_sprite_type(0x12);
    s.sprite_slot_view_mut(k).set_subtype(0xaa);
    s.sprite_slot_view_mut(k).set_die_action(0xbb);
    s.set_sound_effect_2(0xff);
    s.sprite_slot_view_mut(k).set_hit_timer(0xcc);
    s.sprite_slot_view_mut(k).set_incoming_damage(0xdd);
    s.sprite_set_x(k, 0x0123);
    s.sprite_set_y(k, 0x0340);
    s.sprite_slot_view_mut(k).set_floor(2);

    s.sprite_func18(k, 0xe3);

    assert_eq!(s.sprite_slot_view(k).sprite_type(), 0xe3);
    assert_eq!(s.sprite_slot_view(k).subtype(), 0xaa);
    assert_eq!(s.sprite_slot_view(k).die_action(), 0xbb);
    assert_eq!(s.game_state.system_signals.sound_effect_2() & 0x3f, 0x32);
    assert_eq!(s.sprite_slot_view(k).hit_timer(), 0);
    assert_eq!(s.sprite_slot_view(k).incoming_damage(), 0);

    assert_eq!(s.ram[GARNISH_ACTIVE_SPRITE], 10);
    assert_eq!(s.garnish_slot_view(29).garnish_type(), 10);
    assert_eq!(s.ram[GARNISH_X_LO_SPRITE + 29], 0x23);
    assert_eq!(s.ram[GARNISH_X_HI_SPRITE + 29], 0x01);
    assert_eq!(s.ram[GARNISH_Y_LO_SPRITE + 29], 0x50);
    assert_eq!(s.ram[GARNISH_Y_HI_SPRITE + 29], 0x03);
    assert_eq!(s.ram[GARNISH_SPRITE_SPRITE + 29], 2);
    assert_eq!(s.ram[GARNISH_COUNTDOWN_SPRITE + 29], 15);
}

#[test]
fn sprite_apply_conveyor_skips_even_frames() {
    let mut s = fresh_state();
    let k = 3;
    s.set_frame_counter(0);
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);

    s.sprite_apply_conveyor(k, 0x68);

    assert_eq!(s.sprite_get_x(k), 0x0100);
    assert_eq!(s.sprite_get_y(k), 0x0200);
}

#[test]
fn sprite_apply_conveyor_moves_by_direction_table_on_odd_frames() {
    for (j, expected_x, expected_y) in [
        (0x68, 0x0100, 0x01ff),
        (0x69, 0x0100, 0x0201),
        (0x6a, 0x00ff, 0x0200),
        (0x6b, 0x0101, 0x0200),
    ] {
        let mut s = fresh_state();
        let k = 3;
        s.set_frame_counter(1);
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);

        s.sprite_apply_conveyor(k, j);

        assert_eq!(s.sprite_get_x(k), expected_x);
        assert_eq!(s.sprite_get_y(k), expected_y);
    }
}

#[test]
fn sprite_add_xy_applies_signed_offsets_to_16_bit_coords() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);

    s.sprite_add_xy(k, -3, 5);

    assert_eq!(s.sprite_get_x(k), 0x00fd);
    assert_eq!(s.sprite_get_y(k), 0x0205);
}

#[test]
fn sprite_fall_adjust_position_adds_signed_floor_velocity() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.dungeon_moving_floor_mut().set_floor_x_velocity(0xfffe);
    s.dungeon_moving_floor_mut().set_floor_y_velocity(0x0003);

    s.sprite_fall_adjust_position(k);

    assert_eq!(s.sprite_get_x(k), 0x00fe);
    assert_eq!(s.sprite_get_y(k), 0x0203);
}

#[test]
fn sprite_move_xyz_updates_z_then_x_then_y_subpixels() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.sprite_slot_view_mut(k).set_x_subpixel(0xf0);
    s.sprite_slot_view_mut(k).set_y_subpixel(0x10);
    s.sprite_slot_view_mut(k).set_z(0x03);
    s.sprite_slot_view_mut(k).set_z_subpixel(0xf0);
    s.sprite_slot_view_mut(k).set_x_velocity(0x02);
    s.sprite_slot_view_mut(k).set_y_velocity(0xfe);
    s.sprite_slot_view_mut(k).set_z_velocity(0x02);

    s.sprite_move_xyz(k);

    assert_eq!(s.sprite_get_x(k), 0x0101);
    assert_eq!(s.sprite_slot_view(k).x_subpixel(), 0x10);
    assert_eq!(s.sprite_get_y(k), 0x01ff);
    assert_eq!(s.sprite_slot_view(k).y_subpixel(), 0xf0);
    assert_eq!(s.sprite_slot_view(k).z(), 0x04);
    assert_eq!(s.sprite_slot_view(k).z_subpixel(), 0x10);
}

#[test]
fn alloc_overlord_returns_highest_free_slot_or_negative_one() {
    let mut s = fresh_state();
    assert_eq!(s.alloc_overlord(), 7);

    s.overlord_slot_view_mut(7).set_overlord_type(1);
    s.overlord_slot_view_mut(6).set_overlord_type(1);
    assert_eq!(s.alloc_overlord(), 5);

    for i in 0..8 {
        s.overlord_slot_view_mut(i).set_overlord_type(1);
    }
    assert_eq!(s.alloc_overlord(), -1);
}

#[test]
fn overworld_alloc_sprite_matches_start_slots_and_reuse_rule() {
    let mut s = fresh_state();
    s.sprite_system_mut().fill_live_states(1);
    s.sprite_slot_view_mut(13).set_state(0);
    assert_eq!(s.overworld_alloc_sprite(0x01), 13);

    s.sprite_slot_view_mut(13).set_state(1);
    s.sprite_slot_view_mut(12).set_sprite_type(0x41);
    s.sprite_slot_view_mut(12).set_c(2);
    assert_eq!(s.overworld_alloc_sprite(0x01), 12);

    let mut special = fresh_state();
    special.sprite_system_mut().fill_live_states(1);
    special.sprite_slot_view_mut(4).set_state(0);
    assert_eq!(special.overworld_alloc_sprite(0x58), 4);

    let mut full = fresh_state();
    full.sprite_system_mut().fill_live_states(1);
    assert_eq!(full.overworld_alloc_sprite(0xd0), -1);
}

#[test]
fn dungeon_load_single_overlord_allocates_and_initializes_coords() {
    let mut s = fresh_state();
    s.overlord_slot_view_mut(7).set_overlord_type(1);
    s.sprite_workspace_mut().set_room_origin_y_high(0x20);
    s.sprite_workspace_mut().set_room_origin_x_high(0x10);
    s.set_overworld_area_index_word(0x1234);

    s.dungeon_load_single_overlord(&[0x83, 0xe4, 10]);

    assert_eq!(s.overlord_slot_view(6).overlord_type(), 10);
    assert_eq!(s.ram[OVERLORD_FLOOR_SPRITE + 6], 1);
    assert_eq!(s.overlord_slot_view(6).y_low(), 0x30);
    assert_eq!(s.overlord_slot_view(6).y_high(), 0x20);
    assert_eq!(s.overlord_slot_view(6).x_low(), 0x40);
    assert_eq!(s.overlord_slot_view(6).x_high(), 0x10);
    assert_eq!(s.ram[OVERLORD_SPAWNED_IN_AREA_SPRITE + 6], 0x34);
    assert_eq!(s.ram[OVERLORD_GEN1 + 6], 0);
    assert_eq!(s.ram[OVERLORD_GEN2 + 6], 160);
    assert_eq!(s.ram[OVERLORD_GEN3_SPRITE + 6], 0);

    let mut trap = fresh_state();
    trap.sprite_workspace_mut().set_room_origin_x_high(0x10);
    trap.dungeon_load_single_overlord(&[0x00, 0xe0, 3]);
    assert_eq!(
        trap.game_state
            .sprites
            .overlord_slots
            .slot(7)
            .overlord_type(),
        3
    );
    assert_eq!(trap.ram[OVERLORD_GEN2 + 7], 255);
    assert_eq!(trap.overlord_slot_view(7).x_low(), 0xf8);
}

#[test]
fn sprite_initialize_slots_clears_stale_sprite_and_overlord_slots() {
    let mut s = fresh_state();
    s.set_overworld_area_index(0x34);
    s.follower_link_state_mut().set_picking_throw_state(7);
    s.follower_link_state_mut().set_state_bits(0x80);

    s.sprite_slot_view_mut(1).set_state(10);
    s.sprite_slot_view_mut(1).set_sprite_type(0x20);
    s.sprite_slot_view_mut(2).set_state(10);
    s.sprite_slot_view_mut(2).set_sprite_type(0xec);
    s.sprite_slot_view_mut(3).set_state(9);
    s.sprite_slot_view_mut(3).set_sprite_type(0x20);
    s.sprite_slot_view_mut(3).set_room(0x12);
    s.sprite_slot_view_mut(4).set_state(9);
    s.sprite_slot_view_mut(4).set_sprite_type(0x20);
    s.sprite_slot_view_mut(4).set_room(0x34);
    s.sprite_slot_view_mut(5).set_state(9);
    s.sprite_slot_view_mut(5).set_sprite_type(0x6c);
    s.sprite_slot_view_mut(5).set_room(0x12);
    s.overlord_slot_view_mut(1).set_overlord_type(0x14);
    s.overlord_slot_view_mut(1).set_spawned_area(0x12);
    s.overlord_slot_view_mut(2).set_overlord_type(0x14);
    s.overlord_slot_view_mut(2).set_spawned_area(0x34);

    s.sprite_initialize_slots();

    assert_eq!(s.sprite_slot_view(1).state(), 0);
    assert!(!s.game_state.player.follower_link.has_picking_throw_state());
    assert!(!s.game_state.player.follower_link.has_action_state());
    assert_eq!(s.sprite_slot_view(2).state(), 10);
    assert_eq!(s.sprite_slot_view(3).state(), 0);
    assert_eq!(s.sprite_slot_view(4).state(), 9);
    assert_eq!(s.sprite_slot_view(5).state(), 9);
    assert_eq!(s.overlord_slot_view(1).overlord_type(), 0);
    assert_eq!(s.overlord_slot_view(2).overlord_type(), 0x14);
}

#[test]
fn sprite_initialize_mirror_portal_replaces_existing_portal_and_sets_travel_coords() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(4).set_state(9);
    s.sprite_slot_view_mut(4).set_sprite_type(0x6c);
    s.sprite_slot_view_mut(15).set_state(1);
    s.set_bird_travel_destination(15, 0x1234, 0x01f8);

    s.sprite_initialize_mirror_portal();

    assert_eq!(s.sprite_slot_view(4).state(), 0);
    assert_eq!(s.sprite_slot_view(14).sprite_type(), 0x6c);
    assert_eq!(s.sprite_slot_view(14).state(), 9);
    assert_eq!(s.sprite_get_x(14), 0x1234);
    assert_eq!(s.sprite_get_y(14), 0x0200);
    assert_eq!(s.sprite_slot_view(14).floor(), 0);
    assert_eq!(s.sprite_slot_view(14).ignore_projectile(), 1);

    let mut full = fresh_state();
    full.sprite_system_mut().fill_live_states(9);
    full.sprite_slot_view_mut(0).set_state(7);
    full.set_bird_travel_destination(15, 0xabcd, 0x0201);
    full.sprite_initialize_mirror_portal();
    assert_eq!(full.sprite_get_x(0), 0xabcd);
    assert_eq!(full.sprite_get_y(0), 0x0209);
    assert_eq!(full.sprite_slot_view(0).floor(), 0);
    assert_eq!(full.sprite_slot_view(0).ignore_projectile(), 1);
}

#[test]
fn dungeon_load_single_sprite_preserves_c_tmp_counter_side_effect() {
    let mut s = fresh_state();
    s.dungeon_room_tracking_mut().set_room_index2_word(0x004a);
    s.sprite_workspace_mut().set_room_origin_y_high(0x08);
    s.sprite_workspace_mut().set_room_origin_x_high(0x04);

    let next = s.dungeon_load_single_sprite(3, 0xa0, 0x60, 0x2f);

    assert_eq!(next, 3);
    assert_eq!(s.sprite_slot_view(3).state(), 8);
    assert_eq!(s.sprite_slot_view(3).floor(), 1);
    assert_eq!(s.game_state.sprites.workspace.shared_scratch_a(), 0x60);
    assert_eq!(s.game_state.scratch_counter.value(), 0x08);
    assert_eq!(s.sprite_slot_view(3).subtype(), 0x0b);
}

#[test]
fn garnish_get_x_and_y_read_16_bit_coords() {
    let mut s = fresh_state();
    let k = 7;
    s.garnish_slot_view_mut(k).set_x(0x1234);
    s.garnish_slot_view_mut(k).set_y(0xabcd);

    assert_eq!(s.garnish_get_x(k), 0x1234);
    assert_eq!(s.garnish_get_y(k), 0xabcd);
}

#[test]
fn sprite_inactive_sprite_invalidates_room_or_overworld_slot_marker() {
    let mut outdoor = fresh_state();
    let k = 5;
    outdoor.set_indoor_flag(0);
    let n_word = outdoor.sprite_slot_view(k).n_word();
    outdoor
        .sprite_slot_view_mut(k)
        .set_n_word((n_word & 0xff00) | 0x0034);
    let n_word = outdoor.sprite_slot_view(k).n_word();
    outdoor
        .sprite_slot_view_mut(k)
        .set_n_word((n_word & 0x00ff) | 0x1200);
    outdoor.sprite_inactive_sprite(k);
    assert_eq!(outdoor.sprite_slot_view(k).n_word(), 0xffff);

    let mut indoor = fresh_state();
    indoor.set_indoor_flag(1);
    indoor.sprite_slot_view_mut(k).set_n(0x34);
    indoor.sprite_inactive_sprite(k);
    assert_eq!(indoor.sprite_slot_view(k).n(), 0xff);
}

#[test]
fn sprite_get_tile_attribute_reads_indoor_floor_table_and_caches_type() {
    let mut s = fresh_state();
    let k = 5;
    s.set_indoor_flag(1);
    s.sprite_slot_view_mut(k).set_floor(1);
    let mut x = 0x0128;
    let y = 0x0030;
    let offset = 0x1000 + (((x & 0x01f8) >> 3) as usize) + (((y & 0x01f8) << 3) as usize);
    s.dungeon_bg2_attributes_mut().set_bg2_attr(offset, 0x72);

    assert_eq!(s.sprite_get_tile_attribute(k, &mut x, y), 0x72);

    assert_eq!(x, 0x0128);
    assert_eq!(s.game_state.sprites.workspace.tile_type(), 0x72);

    let mut floor0_x = 0x0008;
    s.dungeon_bg2_attributes_mut().set_bg2_attr(1, 0x34);
    assert_eq!(s.GetTileAttribute(0, &mut floor0_x, 0), 0x34);
    assert_eq!(floor0_x, 0x0008);
    assert_eq!(s.game_state.sprites.workspace.tile_type(), 0x34);

    s.set_indoor_flag(0);
    let mut outdoor_x = 0x0128;
    let outdoor_y = 0x0040;
    let expected = s.overworld_get_tile_attribute_at_location(outdoor_x >> 3, outdoor_y);
    assert_eq!(s.GetTileAttribute(0, &mut outdoor_x, outdoor_y), expected);
    assert_eq!(outdoor_x, 0x0025);
    assert_eq!(s.game_state.sprites.workspace.tile_type(), expected);
}

#[test]
fn link_setup_hit_box_matches_c_offsets_and_disabled_sentinel() {
    let mut s = fresh_state();
    s.follower_link_state_mut().set_x(0x12fc);
    s.follower_link_state_mut().set_y(0x34f9);
    let mut hb = empty_hit_box();

    s.link_setup_hit_box(&mut hb);

    assert_eq!(hb.r2, 8);
    assert_eq!(hb.r3, 8);
    assert_eq!(hb.r0_xlo, 0x00);
    assert_eq!(hb.r8_xhi, 0x13);
    assert_eq!(hb.r1_ylo, 0x01);
    assert_eq!(hb.r9_yhi, 0x35);

    s.follower_link_state_mut()
        .set_sprite_damage_disable_timer(1);
    hb.r0_xlo = 0xaa;
    hb.r8_xhi = 0xbb;
    hb.r1_ylo = 0xcc;
    hb.r9_yhi = 0xdd;
    s.link_setup_hit_box_conditional(&mut hb);
    assert_eq!(hb.r0_xlo, 0xaa);
    assert_eq!(hb.r8_xhi, 0xbb);
    assert_eq!(hb.r1_ylo, 0xcc);
    assert_eq!(hb.r9_yhi, 0x80);

    s.follower_link_state_mut()
        .clear_sprite_damage_disable_timer();
    s.link_setup_hit_box_conditional(&mut hb);
    assert_eq!(hb.r0_xlo, 0x00);
    assert_eq!(hb.r8_xhi, 0x13);
    assert_eq!(hb.r1_ylo, 0x01);
    assert_eq!(hb.r9_yhi, 0x35);
}

#[test]
fn sprite_setup_hit_box00_uses_current_sprite_link_bounds_and_z() {
    let mut s = fresh_state();
    let k = 5;
    s.sprite_workspace_mut().set_current_sprite_x(0x0100);
    s.sprite_workspace_mut().set_current_sprite_y(0x0200);
    s.follower_link_state_mut().set_x(0x0100);
    s.follower_link_state_mut().set_y(0x0200);

    assert!(s.sprite_setup_hit_box00(k));

    s.follower_link_state_mut().set_x(0x010c);
    assert!(!s.sprite_setup_hit_box00(k));

    s.follower_link_state_mut().set_x(0x0100);
    s.follower_link_state_mut().set_y(0x0208);
    assert!(!s.sprite_setup_hit_box00(k));

    s.follower_link_state_mut().set_y(0x01f9);
    s.sprite_slot_view_mut(k).set_z(7);
    assert!(s.sprite_setup_hit_box00(k));
}

#[test]
fn sprite_place_rupulse_spark_2_sets_visible_sprite_position() {
    let mut s = fresh_state();
    let k = 5;
    s.set_bg2_x(0x0100);
    s.set_bg2_y(0x0200);
    s.sprite_set_x(k, 0x0184);
    s.sprite_set_y(k, 0x027f);
    s.sprite_slot_view_mut(k).set_floor(2);

    s.sprite_place_rupulse_spark_2(k);

    assert_eq!(s.ram[REPULSESPARK_X_LO_SPRITE], 0x84);
    assert_eq!(s.ram[REPULSESPARK_Y_LO_SPRITE], 0x7f);
    assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 5);
    assert_eq!(s.ram[REPULSESPARK_FLOOR_STATUS_SPRITE], 2);

    let mut offscreen = fresh_state();
    offscreen.sprite_set_x(k, 0x0200);
    offscreen.sprite_set_y(k, 0x0000);
    offscreen.sprite_place_rupulse_spark_2(k);
    assert_eq!(offscreen.ram[REPULSESPARK_TIMER_SPRITE], 0);
}

#[test]
fn sprite_place_weapon_tink_respects_active_repulsespark_timer() {
    let mut active = fresh_state();
    let k = 5;
    active.garnish_state_mut().set_repulsespark_timer(3);
    active.set_sound_effect_1(0);
    active.sprite_place_weapon_tink(k);
    assert_eq!(active.ram[REPULSESPARK_TIMER_SPRITE], 3);
    assert_eq!(active.game_state.system_signals.sound_effect_1(), 0);

    let mut s = fresh_state();
    s.sprite_set_x(k, 0x0050);
    s.sprite_set_y(k, 0x0060);
    s.sprite_slot_view_mut(k).set_floor(1);
    s.sprite_place_weapon_tink(k);
    assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 5);
    assert_eq!(s.ram[REPULSESPARK_X_LO_SPRITE], 0x50);
    assert_eq!(s.ram[REPULSESPARK_Y_LO_SPRITE], 0x60);
    assert_eq!(s.ram[REPULSESPARK_FLOOR_STATUS_SPRITE], 1);
    assert_eq!(s.game_state.system_signals.sound_effect_1(), 5);
}

#[test]
fn link_place_weapon_tink_uses_link_oam_offsets_and_x_carry() {
    let mut active = fresh_state();
    active.garnish_state_mut().set_repulsespark_timer(3);
    active.follower_link_state_mut().set_x(0x00f0);
    active.follower_link_state_mut().set_oam_x_offset(0x20);
    active.link_place_weapon_tink();
    assert_eq!(active.ram[REPULSESPARK_TIMER_SPRITE], 3);
    assert_eq!(active.ram[REPULSESPARK_X_LO_SPRITE], 0);
    assert_eq!(active.game_state.system_signals.sound_effect_1(), 0);

    let mut s = fresh_state();
    s.follower_link_state_mut().set_x(0x01f0);
    s.follower_link_state_mut().set_y(0x0020);
    s.follower_link_state_mut().set_oam_x_offset(0x20);
    s.follower_link_state_mut().set_oam_y_offset(0x30);
    s.follower_link_state_mut().set_lower_level_state(2);

    s.link_place_weapon_tink();

    assert_eq!(s.ram[REPULSESPARK_TIMER_SPRITE], 5);
    assert_eq!(s.ram[REPULSESPARK_X_LO_SPRITE], 0x10);
    assert_eq!(s.ram[REPULSESPARK_Y_LO_SPRITE], 0x51);
    assert_eq!(s.ram[REPULSESPARK_FLOOR_STATUS_SPRITE], 2);
    assert_eq!(
        s.game_state.system_signals.sound_effect_1(),
        s.link_calculate_sfx_pan() | 5
    );
}

#[test]
fn sprite_apply_recoil_to_link_projects_speed_and_resets_z_coord() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.follower_link_state_mut().set_x(0x0140);
    s.follower_link_state_mut().set_y(0x01d0);
    s.sprite_slot_view_mut(k).set_z(4);
    s.follower_link_state_mut().set_z(0x1234);

    let expected = s.sprite_project_speed_towards_link(k, 0x30);
    s.sprite_apply_recoil_to_link(k, 0x30);

    assert_eq!(
        s.game_state.player.follower_link.actual_x_velocity(),
        expected.x
    );
    assert_eq!(
        s.game_state.player.follower_link.actual_y_velocity(),
        expected.y
    );
    assert_eq!(s.game_state.player.follower_link.actual_z_velocity(), 0x18);
    assert_eq!(
        s.game_state
            .player
            .follower_link
            .recoil_z_velocity_for_dungeon_reset(),
        0x18
    );
    assert_eq!(s.game_state.player.follower_link.z(), 0);
}

#[test]
fn sprite_direction_to_face_link_matches_c_axis_and_coords_output() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.follower_link_state_mut().set_x(0x0120);
    s.follower_link_state_mut().set_y(0x0204);
    let mut coords = PointU8 { x: 0, y: 0 };

    assert_eq!(s.sprite_direction_to_face_link(k, Some(&mut coords)), 0);
    assert_eq!(coords, PointU8 { x: 0x20, y: 0x0c });
    assert_eq!(s.game_state.scratch_counter.value(), 0x0c);

    s.follower_link_state_mut().set_x(0x00f8);
    s.follower_link_state_mut().set_y(0x0240);
    s.sprite_slot_view_mut(k).set_z(0);
    assert_eq!(s.sprite_direction_to_face_link(k, None), 2);
    assert_eq!(s.game_state.scratch_counter.value(), 0x48);
}

#[test]
fn sprite_do_hit_boxes_fast_uses_dungmap_offsets_and_large_type_size() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_set_x(k, 0x0120);
    s.sprite_set_y(k, 0x0202);
    s.hitbox_scratch_offset_mut().set_offsets(0xfc, 0x08);
    let mut hb = empty_hit_box();

    s.sprite_do_hit_boxes_fast(k, &mut hb);

    assert_eq!(hb.r4_spr_xlo, 0x28);
    assert_eq!(hb.r10_spr_xhi, 0x01);
    assert_eq!(hb.r5_spr_ylo, 0xfe);
    assert_eq!(hb.r11_spr_yhi, 0x01);
    assert_eq!(hb.r6_spr_xsize, 3);
    assert_eq!(hb.r7_spr_ysize, 3);

    s.sprite_slot_view_mut(k).set_sprite_type(0x6a);
    s.hitbox_scratch_offset_mut().set_offsets(0x02, 0xfe);
    s.sprite_do_hit_boxes_fast(k, &mut hb);
    assert_eq!(hb.r4_spr_xlo, 0x1e);
    assert_eq!(hb.r10_spr_xhi, 0x01);
    assert_eq!(hb.r5_spr_ylo, 0x04);
    assert_eq!(hb.r11_spr_yhi, 0x02);
    assert_eq!(hb.r6_spr_xsize, 16);
    assert_eq!(hb.r7_spr_ysize, 16);

    hb.r10_spr_xhi = 0x12;
    s.hitbox_scratch_offset_mut().set_x_high_offset(0x80);
    s.sprite_do_hit_boxes_fast(k, &mut hb);
    assert_eq!(hb.r10_spr_xhi, 0x80);
}

#[test]
fn sprite_correct_oam_entries_recomputes_ext_bits_and_hides_offscreen_y() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_set_x(k, 0x0120);
    s.sprite_set_y(k, 0x0200);
    s.set_bg2_x(0x0100);
    s.set_bg2_y(0x0200);
    s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    s.oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    s.oam_state_mut().write_entry(OAM_BUF, 0x30, 0x04, 0, 0);
    s.oam_state_mut().write_entry(OAM_BUF + 4, 0xf0, 0xef, 0, 0);
    s.oam_state_mut()
        .set_extended_byte_at(BYTEWISE_EXTENDED_OAM, 2);
    s.oam_state_mut()
        .set_extended_byte_at(BYTEWISE_EXTENDED_OAM + 1, 0);

    s.sprite_correct_oam_entries(k, 1, 0xff);

    assert_eq!(s.ram[BYTEWISE_EXTENDED_OAM], 2);
    assert_eq!(s.ram[BYTEWISE_EXTENDED_OAM + 1], 1);
    assert_eq!(s.ram[OAM_BUF + 1], 0x04);
    assert_eq!(s.ram[OAM_BUF + 5], 0xf0);

    s.oam_state_mut().write_entry(OAM_BUF, 0x30, 0x04, 0, 0);
    s.oam_state_mut()
        .set_extended_byte_at(BYTEWISE_EXTENDED_OAM, 2);
    s.sprite_correct_oam_entries(k, 0, 0);
    assert_eq!(s.ram[BYTEWISE_EXTENDED_OAM], 0);
}

#[test]
fn sprite_kill_self_matches_indoor_guard_and_loaded_bit_clear() {
    let k = 5;

    let mut guarded = fresh_state();
    guarded.set_indoor_flag(1);
    guarded.sprite_slot_view_mut(k).set_state(9);
    guarded.sprite_slot_view_mut(k).set_n(0x12);
    guarded.sprite_kill_self(k);
    assert_eq!(guarded.sprite_slot_view(k).state(), 9);
    assert_eq!(guarded.sprite_slot_view(k).n(), 0x12);

    let mut indoor_allowed = fresh_state();
    indoor_allowed.set_indoor_flag(1);
    indoor_allowed
        .sprite_slot_view_mut(k)
        .set_deflection_bits(0x40);
    indoor_allowed.sprite_slot_view_mut(k).set_state(9);
    indoor_allowed.sprite_slot_view_mut(k).set_n(0x12);
    indoor_allowed.sprite_kill_self(k);
    assert_eq!(indoor_allowed.sprite_slot_view(k).state(), 0);
    assert_eq!(indoor_allowed.sprite_slot_view(k).n(), 0xff);

    let mut outdoor = fresh_state();
    outdoor.sprite_slot_view_mut(k).set_state(9);
    outdoor.sprite_slot_view_mut(k).set_n_word(0x0012);
    outdoor.ram[OVERWORLD_SPRITE_WAS_LOADED + 2] = 0xff;
    outdoor.sprite_kill_self(k);
    assert_eq!(outdoor.sprite_slot_view(k).state(), 0);
    assert_eq!(outdoor.ram[0], 0x12);
    assert_eq!(read_le_u16(&outdoor.ram, 1), 0xef82);
    assert_eq!(outdoor.ram[OVERWORLD_SPRITE_WAS_LOADED + 2], 0xdf);
    assert_eq!(outdoor.sprite_slot_view(k).n_word(), 0xffff);

    // Legacy parity check: the SNES address `0xEF80 + (blk>>3)` is computed in 16-bit
    // (wraps mod 0x10000) BEFORE the 0x10000 bank is added, so blk=0xff00
    // (blk>>3 = 0x1fe0) yields addr16 = (0xEF80+0x1FE0)&0xFFFF = 0x0F60 and a
    // final byte at 0x0F60 + 0x10000 = 0x10F60 (inside the BG char buffer).
    let mut wrapped = fresh_state();
    wrapped.sprite_slot_view_mut(k).set_state(9);
    wrapped.sprite_slot_view_mut(k).set_n_word(0xff00);
    let wrapped_addr = 0x10000 + usize::from(0xEF80u16.wrapping_add(0xff00 >> 3));
    assert_eq!(wrapped_addr, 0x10f60);
    wrapped.ram[wrapped_addr] = 0xff;
    wrapped.sprite_kill_self(k);
    assert_eq!(wrapped.ram[wrapped_addr], 0x7f);
}

#[test]
fn stunned_sprite_sparkle_gate_uses_reference_masks() {
    let k = 12;
    let mut s = fresh_state();
    s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    s.oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    s.set_frame_counter(0x94);
    s.ram[0x0fa1] = 0x48;
    s.sprite_slot_view_mut(k).set_sprite_type(0x22);
    s.sprite_slot_view_mut(k).set_state(11);
    s.sprite_set_x(k, 0x0d0b);
    s.sprite_set_y(k, 0x056a);
    s.sprite_slot_view_mut(k).set_draw_work_byte_5(1);
    s.sprite_slot_view_mut(k).set_delay_main(0x18);
    s.sprite_slot_view_mut(k).set_ai_state(1);
    s.sprite_slot_view_mut(k).set_z(3);
    s.sprite_slot_view_mut(k).set_z_velocity(0x0b);
    s.sprite_stunned_main_func1(k);

    assert_eq!(s.ram[0x0fa1], 0x48);
    assert_eq!(s.ram[GARNISH_ACTIVE_SPRITE], 0);
    assert_eq!(s.garnish_slot_view(28).garnish_type(), 0);
}

#[test]
fn sprite_prep_oam_coord_fills_ret_and_out_of_bounds_side_effects() {
    let k = 4;
    let mut visible = fresh_state();
    visible.sprite_workspace_mut().set_current_sprite_x(0x0120);
    visible.sprite_workspace_mut().set_current_sprite_y(0x0230);
    visible.set_bg2_x(0x0100);
    visible.set_bg2_y(0x0200);
    visible.sprite_slot_view_mut(k).set_z(3);
    visible.sprite_slot_view_mut(k).set_oam_flags(0x0a);
    visible.sprite_slot_view_mut(k).set_object_priority(0x03);
    let mut ret = PrepOamCoordsRet {
        x: 0,
        y: 0,
        r4: 0xff,
        flags: 0,
    };

    visible.sprite_prep_oam_coord(k, &mut ret);

    assert_eq!(ret.x, 0x20);
    assert_eq!(ret.y, 0x2d);
    assert_eq!(ret.r4, 0);
    assert_eq!(ret.flags, 0x09);
    assert_eq!(
        visible
            .game_state
            .sprites
            .draw_hitbox_work
            .low_position_word(),
        0x2d20
    );
    assert_eq!(visible.sprite_slot_view(k).pause(), 0);

    let mut out = fresh_state();
    out.sprite_slot_view_mut(k).set_state(9);
    out.sprite_slot_view_mut(k).set_n_word(0x0012);
    out.ram[OVERWORLD_SPRITE_WAS_LOADED + 2] = 0xff;
    out.sprite_workspace_mut().set_current_sprite_x(0x0130);
    let mut out_ret = PrepOamCoordsRet {
        x: 0,
        y: 0,
        r4: 0xff,
        flags: 0,
    };

    out.sprite_prep_oam_coord(k, &mut out_ret);

    assert_eq!(out_ret.x, 0x8212);
    assert_eq!(out_ret.y, 0x00ef);
    assert_eq!(out_ret.r4, 0);
    assert_eq!(out.sprite_slot_view(k).pause(), 1);
    assert_eq!(out.sprite_slot_view(k).state(), 0);
    assert_eq!(out.ram[OVERWORLD_SPRITE_WAS_LOADED + 2], 0xdf);
}

#[test]
fn sprite_spawn_simple_sparkle_garnish_ex_initializes_allocated_slot() {
    let mut s = fresh_state();
    let k = 4;
    s.garnish_slot_view_mut(29).set_garnish_type(1);
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.sprite_slot_view_mut(k).set_z(3);
    s.sprite_slot_view_mut(k).set_floor(2);

    assert_eq!(s.sprite_garnish_spawn_sparkle(k, 0x12, 0x20), 28);

    assert_eq!(s.garnish_slot_view(28).garnish_type(), 5);
    assert_eq!(s.ram[GARNISH_ACTIVE_SPRITE], 5);
    assert_eq!(s.garnish_get_x(28), 0x0112);
    assert_eq!(s.garnish_get_y(28), 0x022d);
    assert_eq!(s.ram[GARNISH_COUNTDOWN_SPRITE + 28], 31);
    assert_eq!(s.ram[GARNISH_SPRITE_SPRITE + 28], k as u8);
    assert_eq!(s.ram[GARNISH_FLOOR_SPRITE + 28], 2);
    assert_eq!(s.ram[15], 28);

    let mut full = fresh_state();
    for slot in 0..15 {
        full.garnish_slot_view_mut(slot).set_garnish_type(1);
    }
    full.sprite_garnish_spawn_sparkle_limited(k, 0, 0);
    assert_eq!(full.ram[15], 0xff);
    assert_eq!(full.ram[GARNISH_ACTIVE_SPRITE], 0);
}

#[test]
fn release_fairy_spawns_fairy_at_link_position_or_returns_negative_one() {
    let mut s = fresh_state();
    s.follower_link_state_mut().set_x(0x0100);
    s.follower_link_state_mut().set_y(0x0200);
    s.follower_link_state_mut().mark_lower_level();
    s.sprite_slot_view_mut(0).set_direction(3);

    assert_eq!(s.release_fairy(), 15);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xe3);
    assert_eq!(s.sprite_slot_view(15).state(), 9);
    assert_eq!(s.sprite_slot_view(15).floor(), 1);
    assert_eq!(s.sprite_get_x(15), 0x0108);
    assert_eq!(s.sprite_get_y(15), 0x0210);
    assert_eq!(s.sprite_slot_view(15).direction(), 0);
    assert_eq!(s.sprite_slot_view(15).delay_aux4(), 96);

    let mut full = fresh_state();
    full.sprite_system_mut().fill_live_states(9);
    assert_eq!(full.release_fairy(), -1);
}

#[test]
fn sprite_convert_velocity_to_angle_matches_c_tables() {
    for (x, y, expected) in [
        (16, 0, 0),
        (16, 8, 1),
        (0, 16, 4),
        (8, 16, 3),
        (0xf0, 0, 8),
        (0, 0xf0, 12),
    ] {
        assert_eq!(ZeldaState::sprite_convert_velocity_to_angle(x, y), expected);
    }
}

#[test]
fn sprite_zero_and_invert_velocity_helpers_match_c() {
    let mut s = fresh_state();
    let k = 5;
    s.sprite_slot_view_mut(k).set_x_velocity(0x12);
    s.sprite_slot_view_mut(k).set_y_velocity(0xf0);
    s.sprite_zero_velocity_xy(k);
    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0);

    s.sprite_slot_view_mut(k).set_x_velocity(0x12);
    s.sprite_slot_view_mut(k).set_y_velocity(0xf0);
    s.sprite_invert_xy_speeds(k);
    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0xee);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0x10);

    s.sprite_invert_speed_xy(k);
    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0x12);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0xf0);
}

#[test]
fn sprite_bounce_off_wall_inverts_only_colliding_axes() {
    let k = 5;

    let mut x_only = fresh_state();
    x_only.sprite_slot_view_mut(k).set_x_velocity(0x08);
    x_only.sprite_slot_view_mut(k).set_y_velocity(0x09);
    x_only.sprite_slot_view_mut(k).set_wall_collision(0x01);
    x_only.sprite_bounce_off_wall(k);
    assert_eq!(x_only.sprite_slot_view(k).x_velocity(), 0xf8);
    assert_eq!(x_only.sprite_slot_view(k).y_velocity(), 0x09);

    let mut y_only = fresh_state();
    y_only.sprite_slot_view_mut(k).set_x_velocity(0x08);
    y_only.sprite_slot_view_mut(k).set_y_velocity(0x09);
    y_only.sprite_slot_view_mut(k).set_wall_collision(0x04);
    y_only.sprite_bounce_off_wall(k);
    assert_eq!(y_only.sprite_slot_view(k).x_velocity(), 0x08);
    assert_eq!(y_only.sprite_slot_view(k).y_velocity(), 0xf7);

    let mut both = fresh_state();
    both.sprite_slot_view_mut(k).set_x_velocity(0x08);
    both.sprite_slot_view_mut(k).set_y_velocity(0x09);
    both.sprite_slot_view_mut(k).set_wall_collision(0x0f);
    both.sprite_bounce_off_wall(k);
    assert_eq!(both.sprite_slot_view(k).x_velocity(), 0xf8);
    assert_eq!(both.sprite_slot_view(k).y_velocity(), 0xf7);
}

#[test]
fn sprite_return_if_paused_matches_c_boolean_gates() {
    let k = 2;

    let mut active = fresh_state();
    active.sprite_slot_view_mut(k).set_pause(0);
    assert!(!active.sprite_return_if_paused(k));

    let mut global_pause = fresh_state();
    global_pause.set_modal_pause_flag(1);
    assert!(global_pause.sprite_return_if_paused(k));

    let mut submodule = fresh_state();
    submodule.set_submodule(2);
    assert!(submodule.sprite_return_if_paused(k));

    let mut sprite_pause = fresh_state();
    sprite_pause.sprite_slot_view_mut(k).set_pause(1);
    sprite_pause.sprite_slot_view_mut(k).set_deflection_bits(0);
    assert!(sprite_pause.sprite_return_if_paused(k));

    sprite_pause
        .sprite_slot_view_mut(k)
        .set_deflection_bits(0x80);
    assert!(!sprite_pause.sprite_return_if_paused(k));
}

#[test]
fn sprite_return_if_phasing_out_matches_stun_countdown_and_draw_gate() {
    let k = 2;

    let mut idle = fresh_state();
    assert!(!idle.sprite_return_if_phasing_out(k));

    let mut blocked = fresh_state();
    blocked.sprite_slot_view_mut(k).set_stunned(4);
    blocked.set_submodule(1);
    assert!(!blocked.sprite_return_if_phasing_out(k));
    assert_eq!(blocked.sprite_slot_view(k).stunned(), 4);

    let mut high_timer = fresh_state();
    high_timer.set_frame_counter(1);
    high_timer.sprite_slot_view_mut(k).set_stunned(0x28);
    assert!(!high_timer.sprite_return_if_phasing_out(k));
    assert_eq!(high_timer.sprite_slot_view(k).stunned(), 0x28);

    let mut odd_after_tick = fresh_state();
    odd_after_tick.sprite_slot_view_mut(k).set_stunned(2);
    assert!(!odd_after_tick.sprite_return_if_phasing_out(k));
    assert_eq!(odd_after_tick.sprite_slot_view(k).stunned(), 1);

    let mut expired = fresh_state();
    expired.sprite_slot_view_mut(k).set_state(9);
    expired.sprite_slot_view_mut(k).set_stunned(1);
    expired.sprite_slot_view_mut(k).set_pause(7);
    assert!(expired.sprite_return_if_phasing_out(k));
    assert_eq!(expired.sprite_slot_view(k).stunned(), 0);
    assert_eq!(expired.sprite_slot_view(k).state(), 0);
    assert_eq!(expired.sprite_slot_view(k).pause(), 0);

    let mut even_visible = fresh_state();
    even_visible.set_frame_counter(1);
    even_visible.sprite_slot_view_mut(k).set_stunned(2);
    even_visible.sprite_slot_view_mut(k).set_pause(7);
    assert!(even_visible.sprite_return_if_phasing_out(k));
    assert_eq!(even_visible.sprite_slot_view(k).stunned(), 2);
    assert_eq!(even_visible.sprite_slot_view(k).pause(), 0);
}

#[test]
fn sprite_check_if_lifted_permissive_delegates_to_lifted_helper_side_effects() {
    let mut s = fresh_state();
    let k = 3;
    s.sprite_system_mut().set_cur_object_index(k as u8);
    s.follower_link_state_mut()
        .set_sprite_pickup_flag_cached((k as u8).wrapping_add(1));
    s.sprite_slot_view_mut(k).set_state(9);
    s.follower_link_state_mut().set_filtered_joypad_l(0xff);
    s.sprite_slot_view_mut(k).set_e(5);
    s.sprite_slot_view_mut(k).set_draw_work_byte_3(6);
    s.sprite_slot_view_mut(k).set_draw_i(7);

    s.sprite_check_if_lifted_permissive(k);

    assert_eq!(s.game_state.player.follower_link.filtered_joypad_l(), 0);
    assert_eq!(s.sprite_slot_view(k).e(), 0);
    assert_eq!(s.sprite_slot_view(k).draw_work_byte_4(), 9);
    assert_eq!(s.sprite_slot_view(k).state(), 10);
    assert_eq!(s.sprite_slot_view(k).delay_main(), 16);
    assert_eq!(s.sprite_slot_view(k).draw_work_byte_3(), 0);
    assert_eq!(s.sprite_slot_view(k).draw_i(), 0);

    let mut running = fresh_state();
    running.follower_link_state_mut().start_running();
    running.sprite_slot_view_mut(k).set_state(9);
    running.sprite_check_if_lifted_permissive(k);
    assert_eq!(running.sprite_slot_view(k).state(), 9);
}

#[test]
fn sprite_hit_timer31_shows_message_only_for_light_world_good_bee_death() {
    let mut s = fresh_state();
    let k = 3;
    s.sprite_slot_view_mut(k).set_sprite_type(0x7a);
    s.sprite_slot_view_mut(k).set_health(4);
    s.sprite_slot_view_mut(k).set_incoming_damage(4);
    s.set_main_module(7);

    s.sprite_hit_timer31(k);

    assert_eq!(
        s.game_state.messaging.dialogue_message_index.value(),
        0x0140
    );
    assert_eq!(s.game_state.frame.submodule, 2);
    assert_eq!(s.game_state.frame.saved_module_for_menu, 7);
    assert_eq!(s.game_state.frame.main_module, 14);

    let mut dark = fresh_state();
    dark.sprite_slot_view_mut(k).set_sprite_type(0x7a);
    dark.sprite_slot_view_mut(k).set_health(1);
    dark.sprite_slot_view_mut(k).set_incoming_damage(1);
    dark.set_dark_world_region_index(1);
    dark.sprite_hit_timer31(k);
    assert_eq!(dark.game_state.messaging.dialogue_message_index.value(), 0);
}

#[test]
fn sprite_track_body_to_head_matches_frame_gated_turning() {
    let mut equal = fresh_state();
    let k = 6;
    equal.sprite_slot_view_mut(k).set_head_direction(2);
    equal.sprite_slot_view_mut(k).set_direction(2);
    assert!(equal.sprite_track_body_to_head(k));
    assert_eq!(equal.sprite_slot_view(k).direction(), 2);

    let mut waiting = fresh_state();
    waiting.set_frame_counter(1);
    waiting.sprite_slot_view_mut(k).set_head_direction(0);
    waiting.sprite_slot_view_mut(k).set_direction(1);
    assert!(!waiting.sprite_track_body_to_head(k));
    assert_eq!(waiting.sprite_slot_view(k).direction(), 1);

    let mut same_axis = fresh_state();
    same_axis.set_frame_counter(0x20);
    same_axis.sprite_slot_view_mut(k).set_head_direction(0);
    same_axis.sprite_slot_view_mut(k).set_direction(1);
    assert!(!same_axis.sprite_track_body_to_head(k));
    assert_eq!(same_axis.sprite_slot_view(k).direction(), 3);

    let mut opposite_axis = fresh_state();
    opposite_axis.set_frame_counter(0x20);
    opposite_axis.sprite_slot_view_mut(k).set_head_direction(2);
    opposite_axis.sprite_slot_view_mut(k).set_direction(0);
    assert!(opposite_axis.sprite_track_body_to_head(k));
    assert_eq!(opposite_axis.sprite_slot_view(k).direction(), 2);
}

#[test]
fn sprite_direction_to_face_location_uses_larger_axis_and_caches_y_distance() {
    let mut s = fresh_state();
    let k = 6;
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0100);

    assert_eq!(s.sprite_direction_to_face_location(k, 0x0120, 0x0108), 0);
    assert_eq!(s.game_state.scratch_counter.value(), 0x08);

    assert_eq!(s.sprite_direction_to_face_location(k, 0x0104, 0x00e0), 3);
    assert_eq!(s.game_state.scratch_counter.value(), 0x20);
}

#[test]
fn sprite_approach_target_speed_steps_one_toward_targets() {
    let mut s = fresh_state();
    let k = 4;
    s.sprite_slot_view_mut(k).set_x_velocity(0x10);
    s.sprite_slot_view_mut(k).set_y_velocity(0x20);

    s.sprite_approach_target_speed(k, 0x20, 0x10);

    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0x11);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0x1f);

    s.sprite_approach_target_speed(k, 0x11, 0x1f);

    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0x11);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0x1f);
}

#[test]
fn sprite_halt_all_movement_nullifies_hookshot_drag_and_speed() {
    let mut s = fresh_state();
    s.ancilla_slot_view_mut(4).set_ancilla_type(0);
    s.follower_link_state_mut().set_hookshot_interlock(1);
    s.follower_link_state_mut().set_position(0x1234, 0x5678);
    s.follower_link_state_mut()
        .set_previous_position(0x9abc, 0xdef0);
    s.follower_link_state_mut().set_speed_setting(7);

    s.sprite_halt_all_movement();

    assert_eq!(s.game_state.player.follower_link.hookshot_interlock(), 0);
    assert_eq!(s.game_state.player.follower_link.safe_return_x_high(), 0x12);
    assert_eq!(s.game_state.player.follower_link.safe_return_y_high(), 0x56);
    assert_eq!(s.game_state.player.follower_link.x(), 0x9abc);
    assert_eq!(s.game_state.player.follower_link.y(), 0xdef0);
    assert_eq!(s.game_state.player.follower_link.speed_setting(), 0);
}

#[test]
fn sprite_check_if_link_is_busy_matches_link_and_hookshot_gates() {
    assert!(!fresh_state().sprite_check_if_link_is_busy());

    let mut aux = fresh_state();
    aux.follower_link_state_mut().set_auxiliary_state(1);
    assert!(aux.sprite_check_if_link_is_busy());

    let mut item_pose = fresh_state();
    item_pose.follower_link_state_mut().set_item_hold_pose(2);
    assert!(item_pose.sprite_check_if_link_is_busy());

    let mut lifted = fresh_state();
    lifted.follower_link_state_mut().set_state_bits(0x80);
    assert!(lifted.sprite_check_if_link_is_busy());

    let mut hookshot = fresh_state();
    hookshot.ancilla_slot_view_mut(4).set_ancilla_type(0x27);
    assert!(hookshot.sprite_check_if_link_is_busy());
}

#[test]
fn sprite_schedule_for_breakage_sets_state_delay_and_flags() {
    let mut s = fresh_state();
    let k = 5;
    s.sprite_slot_view_mut(k).set_flags2(0xfe);

    s.sprite_schedule_for_breakage(k);

    assert_eq!(s.sprite_slot_view(k).delay_main(), 31);
    assert_eq!(s.sprite_slot_view(k).state(), 6);
    assert_eq!(s.sprite_slot_view(k).flags2(), 2);
}

#[test]
fn sprite_check_if_overlords_clear_rejects_active_overlord_types() {
    let mut s = fresh_state();

    assert!(s.sprite_check_if_overlords_clear());

    s.overlord_slot_view_mut(3).set_overlord_type(0x14);
    assert!(!s.sprite_check_if_overlords_clear());

    s.overlord_slot_view_mut(3).set_overlord_type(0x18);
    assert!(!s.sprite_check_if_overlords_clear());

    s.overlord_slot_view_mut(3).set_overlord_type(0x13);
    assert!(s.sprite_check_if_overlords_clear());
}

#[test]
fn sprite_check_if_room_is_clear_ignores_inactive_and_ignored_sprites() {
    let mut s = fresh_state();
    let k = 5;

    assert!(s.sprite_check_if_room_is_clear());

    s.sprite_slot_view_mut(k).set_state(9);
    assert!(!s.sprite_check_if_room_is_clear());

    s.sprite_slot_view_mut(k).set_flags4(0x40);
    assert!(s.sprite_check_if_room_is_clear());

    s.sprite_slot_view_mut(k).set_state(0);
    s.sprite_slot_view_mut(k).set_flags4(0);
    s.overlord_slot_view_mut(2).set_overlord_type(0x18);
    assert!(!s.sprite_check_if_room_is_clear());
}

#[test]
fn sprite_check_if_screen_is_clear_uses_camera_bounds_and_overlords() {
    let mut s = fresh_state();
    let k = 5;

    s.set_bg2_x(0);
    s.set_bg2_y(0);
    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_set_x(k, 0x00f0);
    s.sprite_set_y(k, 0x00f0);
    assert!(!s.sprite_check_if_screen_is_clear());

    s.sprite_set_x(k, 0x0100);
    assert!(s.sprite_check_if_screen_is_clear());

    s.sprite_set_x(k, 0x00f0);
    s.sprite_slot_view_mut(k).set_flags4(0x40);
    assert!(s.sprite_check_if_screen_is_clear());

    s.sprite_slot_view_mut(k).set_state(0);
    s.sprite_slot_view_mut(k).set_flags4(0);
    s.overlord_slot_view_mut(1).set_overlord_type(0x14);
    assert!(!s.sprite_check_if_screen_is_clear());
}

#[test]
fn sprite_manually_set_death_flag_uw_sets_room_bit_only_when_allowed() {
    let mut s = fresh_state();
    let k = 8;
    s.set_indoor_flag(1);
    s.sprite_slot_view_mut(k).set_n(8);
    s.dungeon_room_tracking_mut().set_room_index2_word(0x0123);

    s.sprite_manually_set_death_flag_uw(k);

    assert_eq!(s.game_state.sprites.workspace.where_in_room(0x0123), 0x0100);

    let mut outdoors = fresh_state();
    outdoors.sprite_slot_view_mut(k).set_n(8);
    outdoors
        .dungeon_room_tracking_mut()
        .set_room_index2_word(0x0123);
    outdoors.sprite_manually_set_death_flag_uw(k);
    assert_eq!(
        outdoors.game_state.sprites.workspace.where_in_room(0x0123),
        0
    );

    let mut ignored = fresh_state();
    ignored.set_indoor_flag(1);
    ignored.sprite_slot_view_mut(k).set_deflection_bits(1);
    ignored.sprite_slot_view_mut(k).set_n(8);
    ignored
        .dungeon_room_tracking_mut()
        .set_room_index2_word(0x0123);
    ignored.sprite_manually_set_death_flag_uw(k);
    assert_eq!(
        ignored.game_state.sprites.workspace.where_in_room(0x0123),
        0
    );

    let mut signed = fresh_state();
    signed.set_indoor_flag(1);
    signed.sprite_slot_view_mut(k).set_n(0x80);
    signed
        .dungeon_room_tracking_mut()
        .set_room_index2_word(0x0123);
    signed.sprite_manually_set_death_flag_uw(k);
    assert_eq!(signed.game_state.sprites.workspace.where_in_room(0x0123), 0);
}

#[test]
fn sprite_bounce_from_tile_collision_returns_zero_without_collision() {
    let mut s = fresh_state();
    let k = 2;
    s.sprite_slot_view_mut(k).set_x_velocity(0x12);
    s.sprite_slot_view_mut(k).set_y_velocity(0xf0);
    s.sprite_slot_view_mut(k).set_g(7);
    s.sprite_slot_view_mut(k).set_flags2(0x60);

    assert_eq!(s.sprite_bounce_from_tile_collision(k), 0);
    assert_eq!(s.sprite_slot_view(k).x_velocity(), 0x12);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), 0xf0);
    assert_eq!(s.sprite_slot_view(k).g(), 7);
}
