use super::*;
use crate::game_state::native::sprites::SpriteWorkspaceState;

#[test]
fn native_sprite_slot_bridge_projects_position_and_packed_n_word() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut sprite_slots = SpriteSlotsState::load_from_ram(&ram);

    {
        let mut bridge = sprite_slots.slot_mut(&mut ram, 3);
        bridge.set_x(0x4567);
        bridge.set_y(0x89ab);
        bridge.set_n_word(0x1234);
        bridge.set_x_velocity(0x10);
        bridge.set_y_velocity(0xf0);
        bridge.set_z(0x20);
        bridge.set_z_velocity(0x10);
        bridge.move_x();
        bridge.move_y();
        bridge.move_z();
    }

    let slot = sprite_slots.slot(3);
    assert_eq!(slot.x(), 0x4568);
    assert_eq!(slot.y(), 0x89aa);
    assert_eq!(slot.z(), 0x21);
    assert_eq!(slot.n_word(), 0x1234);
    assert_eq!(ram[SPRITE_X_SUBPIXEL + 3], 0);
    assert_eq!(ram[SPRITE_X_LO + 3], 0x68);
    assert_eq!(ram[SPRITE_X_HI + 3], 0x45);
    assert_eq!(ram[SPRITE_Y_SUBPIXEL + 3], 0);
    assert_eq!(ram[SPRITE_Y_LO + 3], 0xaa);
    assert_eq!(ram[SPRITE_Y_HI + 3], 0x89);
    assert_eq!(ram[SPRITE_Z_SUBPIXEL + 3], 0);
    assert_eq!(ram[SPRITE_Z + 3], 0x21);
    assert_eq!(read_le_u16(&ram, SPRITE_N + 3 * 2), 0x1234);

    let mut projected = vec![0; WRAM_SIZE];
    sprite_slots.write_to_ram(&mut projected);
    assert_eq!(projected[SPRITE_X_SUBPIXEL + 3], 0);
    assert_eq!(projected[SPRITE_X_LO + 3], 0x68);
    assert_eq!(projected[SPRITE_X_HI + 3], 0x45);
    assert_eq!(projected[SPRITE_Y_SUBPIXEL + 3], 0);
    assert_eq!(projected[SPRITE_Y_LO + 3], 0xaa);
    assert_eq!(projected[SPRITE_Y_HI + 3], 0x89);
    assert_eq!(projected[SPRITE_Z_SUBPIXEL + 3], 0);
    assert_eq!(projected[SPRITE_Z + 3], 0x21);
    assert_eq!(read_le_u16(&projected, SPRITE_N + 3 * 2), 0x1234);
}

#[test]
fn maze_game_timer_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MAZE_GAME_TIMER_LO, 0x0012);
    write_le_u16(&mut ram, MAZE_GAME_TIMER_HI, 0x0034);
    write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_LO, 0x0056);
    write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_HI, 0x0078);

    let mut timer = MazeGameTimerState::load_from_ram(&ram);
    assert_eq!(timer.elapsed_low(), 0x0012);
    assert_eq!(timer.elapsed_high(), 0x0034);
    assert_eq!(timer.snapshot_low(), 0x0056);
    assert_eq!(timer.snapshot_high(), 0x0078);
    assert_eq!(timer.increment_elapsed_low(), 0x0013);
    assert_eq!(timer.increment_elapsed_high(), 0x0035);
    timer.capture_snapshot();
    assert_eq!(timer.snapshot_low(), 0x0013);
    assert_eq!(timer.snapshot_high(), 0x0035);
    timer.clear_elapsed();
    assert_eq!(timer.elapsed_low(), 0);
    assert_eq!(timer.elapsed_high(), 0);

    let mut projected = vec![0; WRAM_SIZE];
    timer.write_to_ram(&mut projected);
    assert_eq!(MazeGameTimerState::load_from_ram(&projected), timer);
}

#[test]
fn native_maze_game_timer_bridge_composes_edits_onto_live_ram() {
    // The 0x1fe00 window is shared with mutually-exclusive systems and is no longer
    // bulk-projected, so the bridge must compose its edits onto live RAM rather than
    // re-stamp a stale native snapshot over whichever system wrote it last.
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MAZE_GAME_TIMER_LO, 0x0007);
    write_le_u16(&mut ram, MAZE_GAME_TIMER_HI, 0x0009);
    write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_LO, 0x0011);
    write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_HI, 0x0013);
    let stale_ram = vec![0xff; WRAM_SIZE];
    let mut timer = MazeGameTimerState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeMazeGameTimerBridgeMut::new(&mut timer, &mut ram);
        assert_eq!(bridge.increment_elapsed_low(), 8);
        assert_eq!(bridge.increment_elapsed_high(), 10);
        bridge.capture_snapshot();
    }

    assert_eq!(timer.elapsed_low(), 8);
    assert_eq!(timer.elapsed_high(), 10);
    assert_eq!(timer.snapshot_low(), 8);
    assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_LO), 8);
    assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_HI), 10);
    assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_SNAPSHOT_LO), 8);
    assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_SNAPSHOT_HI), 10);
}

#[test]
fn prize_drop_cycle_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[PRIZE_DROP_CYCLE] = 2;
    ram[PRIZE_DROP_CYCLE + 15] = 7;

    let mut cycle = PrizeDropCycleState::load_from_ram(&ram);
    assert_eq!(cycle.next_index_for_slot(0), 2);
    assert_eq!(cycle.next_index_for_slot(15), 7);
    assert_eq!(cycle.next_index_for_slot(16), 0);
    assert_eq!(cycle.take_next_index(15), 7);
    assert_eq!(cycle.take_next_index(15), 0);
    assert_eq!(cycle.next_index_for_slot(15), 1);
    assert_eq!(cycle.take_next_index(16), 0);

    let mut projected = vec![0; WRAM_SIZE];
    cycle.write_to_ram(&mut projected);
    assert_eq!(PrizeDropCycleState::load_from_ram(&projected), cycle);
    assert_eq!(projected[PRIZE_DROP_CYCLE], 2);
    assert_eq!(projected[PRIZE_DROP_CYCLE + 15], 1);
}

#[test]
fn native_prize_drop_cycle_bridge_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[PRIZE_DROP_CYCLE + 3] = 7;
    let mut cycle = PrizeDropCycleState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativePrizeDropCycleBridgeMut::new(&mut cycle, &mut ram);
        assert_eq!(bridge.take_next_index(3), 7);
        assert_eq!(bridge.take_next_index(3), 0);
        assert_eq!(bridge.take_next_index(18), 0);
    }

    assert_eq!(cycle.next_index_for_slot(3), 1);
    assert_eq!(ram[PRIZE_DROP_CYCLE + 3], 1);
}

#[test]
fn dual_layer_tile_cache_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUAL_LAYER_TILE_CACHE] = 0x1c;
    ram[DUAL_LAYER_TILE_CACHE + 15] = 0x2a;

    let mut cache = DualLayerTileCacheState::load_from_ram(&ram);
    assert_eq!(cache.tile_type(0), 0x1c);
    assert_eq!(cache.tile_type(15), 0x2a);
    assert_eq!(cache.tile_type(16), 0);
    assert!(cache.set_tile_type(15, 0x3b));
    assert!(!cache.set_tile_type(16, 0x4c));
    assert_eq!(cache.tile_type(15), 0x3b);
    assert_eq!(cache.tile_type(16), 0);

    let mut projected = vec![0; WRAM_SIZE];
    cache.write_to_ram(&mut projected);
    assert_eq!(DualLayerTileCacheState::load_from_ram(&projected), cache);
    assert_eq!(projected[DUAL_LAYER_TILE_CACHE], 0x1c);
    assert_eq!(projected[DUAL_LAYER_TILE_CACHE + 15], 0x3b);
}

#[test]
fn native_dual_layer_tile_cache_bridge_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[DUAL_LAYER_TILE_CACHE + 4] = 0x1c;
    let mut cache = DualLayerTileCacheState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativeDualLayerTileCacheBridgeMut::new(&mut cache, &mut ram);
        bridge.set_tile_type(4, 0x2a);
        bridge.set_tile_type(18, 0x7f);
    }

    assert_eq!(cache.tile_type(4), 0x2a);
    assert_eq!(cache.tile_type(18), 0);
    assert_eq!(ram[DUAL_LAYER_TILE_CACHE + 4], 0x2a);
}

#[test]
fn tagalong_trail_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TAGALONG_X_LO + 3] = 0x34;
    ram[TAGALONG_X_HI + 3] = 0x12;
    ram[TAGALONG_Y_LO + 3] = 0x78;
    ram[TAGALONG_Y_HI + 3] = 0x56;
    ram[TAGALONG_Z + 3] = 0xf0;
    ram[TAGALONG_LAYERBITS + 3] = 0x23;

    let trail = TagalongTrailState::load_from_ram(&ram);
    assert_eq!(trail.x(3), 0x1234);
    assert_eq!(trail.y(3), 0x5678);
    assert_eq!(trail.z(3), 0xf0);
    assert_eq!(trail.layer_bits(3), 0x23);
    assert_eq!(trail.x(20), 0);

    let mut projected = vec![0; WRAM_SIZE];
    trail.write_to_ram(&mut projected);
    assert_eq!(TagalongTrailState::load_from_ram(&projected), trail);
}

#[test]
fn native_tagalong_slot_bridge_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[TAGALONG_X_LO + 2] = 1;
    native_ram[TAGALONG_X_HI + 2] = 2;
    let mut trail = TagalongTrailState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut slot = NativeTagalongSlotBridgeMut::new(&mut trail, &mut ram, 2);
        slot.set_position(0x1234, 0x5678);
        slot.set_y_high(0x9a);
        slot.set_z(0xf8);
        slot.set_layer_bits(0x23);
    }

    assert_eq!(trail.x(2), 0x1234);
    assert_eq!(trail.y(2), 0x9a78);
    assert_eq!(trail.z(2), 0xf8);
    assert_eq!(trail.layer_bits(2), 0x23);
    assert_eq!(ram[TAGALONG_X_LO + 2], 0x34);
    assert_eq!(ram[TAGALONG_X_HI + 2], 0x12);
    assert_eq!(ram[TAGALONG_Y_LO + 2], 0x78);
    assert_eq!(ram[TAGALONG_Y_HI + 2], 0x9a);
    assert_eq!(ram[TAGALONG_Z + 2], 0xf8);
    assert_eq!(ram[TAGALONG_LAYERBITS + 2], 0x23);

    {
        let mut out_of_range = NativeTagalongSlotBridgeMut::new(&mut trail, &mut ram, 20);
        out_of_range.set_position(0xffff, 0xffff);
        out_of_range.set_z(0xff);
    }

    assert_eq!(trail.x(20), 0);
    assert_eq!(trail.z(20), 0);
}

#[test]
fn chain_chomp_history_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_X + 4, 0x1234);
    write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_Y + 4, 0x5678);
    write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_X + 0xfe, 0x9abc);
    write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_Y + 0xfe, 0xdef0);

    let mut history = ChainChompHistoryState::load_from_ram(&ram);
    assert_eq!(history.x(2), 0x1234);
    assert_eq!(history.y(2), 0x5678);
    assert_eq!(history.x(0x7f), 0x9abc);
    assert_eq!(history.y(0x7f), 0xdef0);
    assert_eq!(history.x(0x80), 0);
    history.set_x(2, 0x1111);
    history.set_y(2, 0x2222);
    history.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_X + 4), 0x1111);
    assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_Y + 4), 0x2222);
    assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_X + 0xfe), 0x9abc);
    assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_Y + 0xfe), 0xdef0);
}

#[test]
fn native_chain_chomp_history_bridge_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, CHAIN_CHOMP_HISTORY_X, 0x1234);
    write_le_u16(&mut native_ram, CHAIN_CHOMP_HISTORY_Y, 0x5678);
    let mut history = ChainChompHistoryState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativeChainChompHistoryBridgeMut::new(&mut history, &mut ram);
        bridge.set_x(0, 0x1111);
        bridge.set_y(0, 0x2222);
        bridge.set_x(0x80, 0xffff);
        bridge.set_y(0x80, 0xffff);
    }

    assert_eq!(history.x(0), 0x1111);
    assert_eq!(history.y(0), 0x2222);
    assert_eq!(history.x(0x80), 0);
    assert_eq!(history.y(0x80), 0);
    assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_X), 0x1111);
    assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_Y), 0x2222);
}

#[test]
fn ether_orbit_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[ETHER_ANGLE + 2] = 0x3f;
    ram[ETHER_RADIUS] = 0x20;
    write_le_u16(&mut ram, ETHER_BEAM_Y, 0x1234);
    write_le_u16(&mut ram, ETHER_BEAM_TOP_BUCKET, 0xabcd);
    write_le_u16(&mut ram, ETHER_ORBIT_X, 0x4567);
    write_le_u16(&mut ram, ETHER_ORBIT_Y, 0x89ab);
    ram[ETHER_SPIN_COUNTDOWN] = 1;
    write_le_u16(&mut ram, ETHER_ORB_X, 0xdef0);
    write_le_u16(&mut ram, ETHER_ORB_Y, 0x1357);

    let mut orbit = EtherOrbitState::load_from_ram(&ram);
    assert_eq!(orbit.angle(2), 0x3f);
    assert_eq!(orbit.radius(), 0x20);
    assert_eq!(orbit.beam_y(), 0x1234);
    assert_eq!(orbit.beam_top_bucket(), 0xcd);
    assert_eq!(orbit.orbit_x(), 0x4567);
    assert_eq!(orbit.swordbeam_temp_y(), 0x89ab);
    assert_eq!(orbit.orb_x(), 0xdef0);
    assert_eq!(orbit.orb_y(), 0x1357);
    orbit.advance_angle(2);
    orbit.set_beam_top_bucket(0x55);
    orbit.set_swordbeam_temp(0x1111, 0x2222);
    orbit.write_to_ram(&mut ram);

    assert_eq!(ram[ETHER_ANGLE + 2], 0);
    assert_eq!(read_le_u16(&ram, ETHER_BEAM_TOP_BUCKET), 0xab55);
    assert_eq!(read_le_u16(&ram, ETHER_ORBIT_X), 0x1111);
    assert_eq!(read_le_u16(&ram, ETHER_ORBIT_Y), 0x2222);
}

#[test]
fn native_ether_orbit_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ETHER_BEAM_TOP_BUCKET, 0x1200);
    ram[ETHER_SPIN_COUNTDOWN] = 0;
    let stale_ram = vec![0xff; WRAM_SIZE];
    let mut orbit = EtherOrbitState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeEtherOrbitBridgeMut::new(&mut orbit, &mut ram);
        bridge.set_angle(0, 0x3f);
        assert_eq!(bridge.advance_angle(0), 0);
        bridge.set_radius(0x40);
        assert_eq!(bridge.tick_spin_countdown(), 0xff);
        bridge.set_spin_countdown(3);
        bridge.set_beam_top_bucket(0x34);
        bridge.initialize_beam_adjusted_y(0x5678);
        bridge.set_beam_y(0x9abc);
        bridge.set_orbit_position(0x1111, 0x2222);
        bridge.set_orb_position(0x3333, 0x4444);
    }

    assert_eq!(orbit.angle(0), 0);
    assert_eq!(orbit.radius(), 0x40);
    assert_eq!(orbit.beam_top_bucket(), 0x78);
    assert_eq!(orbit.beam_y(), 0x9abc);
    assert_eq!(orbit.orbit_x(), 0x1111);
    assert_eq!(orbit.orb_y(), 0x4444);
    assert_eq!(ram[ETHER_ANGLE], 0);
    assert_eq!(ram[ETHER_RADIUS], 0x40);
    assert_eq!(ram[ETHER_SPIN_COUNTDOWN], 3);
    assert_eq!(read_le_u16(&ram, ETHER_BEAM_TOP_BUCKET), 0x5678);
    assert_eq!(read_le_u16(&ram, ETHER_BEAM_Y), 0x9abc);
    assert_eq!(read_le_u16(&ram, ETHER_ORBIT_X), 0x1111);
    assert_eq!(read_le_u16(&ram, ETHER_ORBIT_Y), 0x2222);
    assert_eq!(read_le_u16(&ram, ETHER_ORB_X), 0x3333);
    assert_eq!(read_le_u16(&ram, ETHER_ORB_Y), 0x4444);
}

#[test]
fn enemy_damage_subclass_table_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[ENEMY_DAMAGE_DATA] = 3;
    ram[ENEMY_DAMAGE_DATA + 0x918] = 2;
    ram[ENEMY_DAMAGE_DATA + 0x0fff] = 7;

    let table = EnemyDamageSubclassTableState::load_from_ram(&ram);
    assert_eq!(table.entry(0), 3);
    assert_eq!(table.entry(0x918), 2);
    assert_eq!(table.entry(0x0fff), 7);
    assert_eq!(table.entry(0x1000), 0);

    let mut projected = vec![0; WRAM_SIZE];
    table.write_to_ram(&mut projected);
    assert_eq!(
        EnemyDamageSubclassTableState::load_from_ram(&projected),
        table
    );
}

#[test]
fn native_enemy_damage_subclass_table_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[ENEMY_DAMAGE_DATA + 0x918] = 9;
    let mut table = EnemyDamageSubclassTableState::load_from_ram(&native_ram);

    let packed = vec![0xab, 0xcd, 0xef];
    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativeEnemyDamageSubclassTableBridgeMut::new(&mut table, &mut ram);
        bridge.load_from_packed_nibbles(&packed);
        bridge.set_entry(0x918, 2);
        bridge.set_entry(0x1000, 7);
    }

    assert_eq!(table.entry(0), 0x0a);
    assert_eq!(table.entry(1), 0x0b);
    assert_eq!(table.entry(2), 0x0c);
    assert_eq!(table.entry(3), 0x0d);
    assert_eq!(table.entry(4), 0x0e);
    assert_eq!(table.entry(5), 0x0f);
    assert_eq!(table.entry(6), 0);
    assert_eq!(table.entry(0x918), 2);
    assert_eq!(table.entry(0x1000), 0);
    assert_eq!(ram[ENEMY_DAMAGE_DATA], 0x0a);
    assert_eq!(ram[ENEMY_DAMAGE_DATA + 1], 0x0b);
    assert_eq!(ram[ENEMY_DAMAGE_DATA + 0x918], 2);
}

#[test]
fn sprite_draw_hitbox_work_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DRAW_WORK_POSITION_X] = 0x34;
    ram[DRAW_WORK_POSITION_Y] = 0x12;
    ram[HITBOX_WORK_Y_OFFSET] = 0xfc;
    ram[DRAW_WORK_FLAGS_HI] = 0x80;

    let mut work = SpriteDrawHitboxWorkState::load_from_ram(&ram);
    assert_eq!(work.x_low(), 0x34);
    assert_eq!(work.y_low(), 0x12);
    assert_eq!(work.low_position_word(), 0x1234);
    assert_eq!(work.hitbox_y_low_offset(), 0xfc);
    assert_eq!(work.hitbox_x_high_offset(), 0x80);
    work.set_low_position_word(0x9abc);
    assert_eq!(work.offset_low_position(1, 2), (0xbd, 0x9c));
    work.set_flags_high(0x7f);
    work.set_offsets(0xfc, 0x08);
    assert_eq!(work.low_position_word(), 0x9cbd);
    assert_eq!(work.hitbox_y_low_offset(), 0xfc);
    assert_eq!(work.hitbox_x_high_offset(), 0x08);

    let mut projected = vec![0; WRAM_SIZE];
    work.write_to_ram(&mut projected);
    assert_eq!(SpriteDrawHitboxWorkState::load_from_ram(&projected), work);
    assert_eq!(projected[DRAW_WORK_POSITION_X], 0xbd);
    assert_eq!(projected[DRAW_WORK_POSITION_Y], 0x9c);
    assert_eq!(projected[DRAW_WORK_FLAGS_HI], 0x08);
    assert_eq!(projected[HITBOX_WORK_X_OFFSET], 0x08);
}

#[test]
fn native_sprite_draw_hitbox_work_bridges_project_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[DRAW_WORK_POSITION_X] = 0x10;
    native_ram[DRAW_WORK_POSITION_Y] = 0x20;
    native_ram[HITBOX_WORK_Y_OFFSET] = 0x30;
    native_ram[HITBOX_WORK_X_OFFSET] = 0x40;
    let mut work = SpriteDrawHitboxWorkState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut draw = NativeSpriteDrawWorkPositionBridgeMut::new(&mut work, &mut ram);
        draw.set_low_position_word(0x9abc);
        draw.offset_low_position(1, 2);
        draw.set_flags_high(0x7f);
    }

    assert_eq!(work.low_position_word(), 0x9cbd);
    assert_eq!(work.hitbox_x_high_offset(), 0x7f);
    assert_eq!(ram[DRAW_WORK_POSITION_X], 0xbd);
    assert_eq!(ram[DRAW_WORK_POSITION_Y], 0x9c);
    assert_eq!(ram[DRAW_WORK_FLAGS_HI], 0x7f);

    {
        let mut hitbox = NativeSpriteHitboxWorkOffsetBridgeMut::new(&mut work, &mut ram);
        hitbox.set_offsets(0xfc, 0x08);
    }

    assert_eq!(work.hitbox_y_low_offset(), 0xfc);
    assert_eq!(work.hitbox_x_high_offset(), 0x08);
    assert_eq!(ram[HITBOX_WORK_Y_OFFSET], 0xfc);
    assert_eq!(ram[HITBOX_WORK_X_OFFSET], 0x08);
    assert_eq!(ram[DRAW_WORK_FLAGS_HI], 0x08);
}

#[test]
fn native_weather_vane_debris_bridge_updates_transient_slots() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut effects = EffectState::load_from_ram(&ram);

    {
        let mut bridge =
            NativeWeatherVaneDebrisBridgeMut::new(&mut effects.weather_vane_debris, &mut ram, 3);
        bridge.initialize(0x1234, 0x5678, 0x9a, 0xbc, 0xde, 0x21, 1);
    }
    let debris = effects.weather_vane_debris.debris(3).snapshot();
    assert_eq!(
        debris,
        effects::WeatherVaneDebrisSnapshot {
            y: 0x5678,
            x: 0x1234,
            z: 0x21,
            y_velocity: 0xbc,
            x_velocity: 0x9a,
            z_velocity: 0xde,
            draw_state: 1,
        }
    );
    assert_eq!(ram[WEATHERVANE_ANIM_TIMER + 3], 1);

    {
        let mut bridge =
            NativeWeatherVaneDebrisBridgeMut::new(&mut effects.weather_vane_debris, &mut ram, 3);
        assert_eq!(bridge.tick_animation(), 1);
        assert_eq!(bridge.tick_z_velocity(), 0xdd);
        bridge.mark_finished_if_landed(0xef);
    }
    assert!(!effects.weather_vane_debris.debris(3).is_finished());
    {
        let mut bridge =
            NativeWeatherVaneDebrisBridgeMut::new(&mut effects.weather_vane_debris, &mut ram, 3);
        bridge.mark_finished_if_landed(0xf0);
        bridge.save_position(0xabcd, 0xef01, 0x45);
    }
    let debris = effects.weather_vane_debris.debris(3);
    assert!(debris.is_finished());
    assert_eq!(debris.snapshot().x, 0xabcd);
    assert_eq!(debris.snapshot().y, 0xef01);
    assert_eq!(debris.snapshot().z, 0x45);
    assert_eq!(ram[WEATHERVANE_X_LO + 3], 0xcd);
    assert_eq!(ram[WEATHERVANE_X_HI + 3], 0xab);
    assert_eq!(ram[WEATHERVANE_Y_LO + 3], 0x01);
    assert_eq!(ram[WEATHERVANE_Y_HI + 3], 0xef);
    assert_eq!(ram[WEATHERVANE_Z + 3], 0x45);
    assert_eq!(ram[WEATHERVANE_Z_VELOCITY + 3], 0xdd);
    assert_eq!(ram[WEATHERVANE_DRAW_STATE + 3], 0xff);
}

#[test]
fn native_weather_vane_debris_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[WEATHERVANE_X_LO + 3] = 0xff;
    stale_ram[WEATHERVANE_X_HI + 3] = 0xee;
    stale_ram[WEATHERVANE_DRAW_STATE + 3] = 0xdd;

    let mut ram = vec![0; WRAM_SIZE];
    ram[WEATHERVANE_X_LO + 3] = 0x34;
    ram[WEATHERVANE_X_HI + 3] = 0x12;
    ram[WEATHERVANE_Y_LO + 3] = 0x78;
    ram[WEATHERVANE_Y_HI + 3] = 0x56;
    ram[WEATHERVANE_Z + 3] = 0x21;
    ram[WEATHERVANE_ANIM_TIMER + 3] = 1;
    ram[WEATHERVANE_DRAW_STATE + 3] = 1;
    let mut effects = EffectState::load_from_ram(&stale_ram);

    {
        let mut bridge =
            NativeWeatherVaneDebrisBridgeMut::new(&mut effects.weather_vane_debris, &mut ram, 3);
        assert_eq!(bridge.tick_animation(), 1);
    }

    let debris = effects.weather_vane_debris.debris(3).snapshot();
    assert_eq!(debris.x, 0x1234);
    assert_eq!(debris.y, 0x5678);
    assert_eq!(debris.z, 0x21);
    assert_eq!(debris.draw_state, 1);
    assert_eq!(ram[WEATHERVANE_X_LO + 3], 0x34);
    assert_eq!(ram[WEATHERVANE_X_HI + 3], 0x12);
    assert_eq!(ram[WEATHERVANE_Y_LO + 3], 0x78);
    assert_eq!(ram[WEATHERVANE_Y_HI + 3], 0x56);
    assert_eq!(ram[WEATHERVANE_Z + 3], 0x21);
    assert_eq!(ram[WEATHERVANE_DRAW_STATE + 3], 1);
}

#[test]
fn native_sprite_history_bridges_update_position_and_motion_banks() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut effects = EffectState::load_from_ram(&ram);

    {
        let mut bridge =
            NativeMoldormHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 7);
        bridge.set_position(0x1234, 0x5678);
    }
    assert_eq!(effects.sprite_histories.moldorm_history(7).x(), 0x1234);
    assert_eq!(effects.sprite_histories.moldorm_history(7).y(), 0x5678);
    {
        let mut bridge =
            NativeMoldormHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 7);
        bridge.set_low_position(0xab, 0xcd);
    }
    assert_eq!(effects.sprite_histories.moldorm_history(7).x(), 0x12ab);
    assert_eq!(effects.sprite_histories.moldorm_history(7).y(), 0x56cd);
    assert_eq!(ram[MOLDORM_HISTORY_X_LO + 7], 0xab);
    assert_eq!(ram[MOLDORM_HISTORY_Y_LO + 7], 0xcd);

    {
        let mut bridge =
            NativeSwamolaTargetBridgeMut::new(&mut effects.sprite_histories, &mut ram, 2);
        bridge.set_position(0x2345, 0x6789);
        bridge.set_x_low(0xef);
        bridge.set_y_low(0x01);
    }
    assert_eq!(effects.sprite_histories.swamola_target(2).x(), 0x23ef);
    assert_eq!(effects.sprite_histories.swamola_target(2).y(), 0x6701);

    {
        let mut bridge =
            NativeSwamolaHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 0x40);
        bridge.set_position(0x3456, 0x789a);
    }
    assert_eq!(effects.sprite_histories.swamola_history(0x40).x(), 0x3456);
    assert_eq!(effects.sprite_histories.swamola_history(0x40).y(), 0x789a);

    {
        let mut bridge =
            NativeBeamosLaserHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 9);
        bridge.set_position(0x4567, 0x89ab);
    }
    assert_eq!(effects.sprite_histories.beamos_laser_history(9).x(), 0x4567);
    assert_eq!(effects.sprite_histories.beamos_laser_history(9).y(), 0x89ab);

    {
        let mut bridge =
            NativeLanmolaSegmentMotionBridgeMut::new(&mut effects.sprite_histories, &mut ram, 9);
        bridge.set_z_offset(0x55);
        bridge.set_direction(0xaa);
    }
    let segment = effects.sprite_histories.lanmola_segment_motion(9);
    assert_eq!(segment.z_offset(), 0x55);
    assert_eq!(segment.direction(), 0xaa);
    assert_eq!(ram[BEAMOS_LASER_HISTORY_X_HI + 9], 0x55);
    assert_eq!(ram[BEAMOS_LASER_HISTORY_Y_HI + 9], 0xaa);
    assert_eq!(effects.sprite_histories.beamos_laser_history(9).x(), 0x5567);
    assert_eq!(effects.sprite_histories.beamos_laser_history(9).y(), 0xaaab);
}

#[test]
fn lanmola_flat_trail_entry_reads_raw_192_slot_alias_region() {
    let mut ram = vec![0; WRAM_SIZE];
    let slot = 0x82;
    ram[MOLDORM_HISTORY_X_LO + slot] = 0x34;
    ram[MOLDORM_HISTORY_Y_LO + slot] = 0x56;
    ram[BEAMOS_LASER_HISTORY_X_HI + slot] = 0x78;
    ram[BEAMOS_LASER_HISTORY_Y_HI + slot] = 0x09;

    let entry = lanmola_flat_trail_entry_from_ram(&ram, slot);

    assert_eq!(entry.x_low(), 0x34);
    assert_eq!(entry.y_low(), 0x56);
    assert_eq!(entry.z_offset(), 0x78);
    assert_eq!(entry.direction(), 0x09);
}

#[test]
fn lanmola_flat_trail_entry_prefers_raw_ram_over_native_128_slot_history() {
    let mut ram = vec![0; WRAM_SIZE];
    let slot = 0x82;
    ram[MOLDORM_HISTORY_X_LO + slot] = 0xaa;
    ram[MOLDORM_HISTORY_Y_LO + slot] = 0xbb;
    ram[BEAMOS_LASER_HISTORY_X_HI + slot] = 0xcc;
    ram[BEAMOS_LASER_HISTORY_Y_HI + slot] = 0xdd;

    let native = EffectState::load_from_ram(&ram);
    let native_moldorm = native.sprite_histories.moldorm_history(slot);
    let native_motion = native.sprite_histories.lanmola_segment_motion(slot);
    assert_eq!(native_moldorm.x(), 0);
    assert_eq!(native_moldorm.y(), 0);
    assert_eq!(native_motion.z_offset(), 0xcc);
    assert_eq!(native_motion.direction(), 0xdd);

    let entry = lanmola_flat_trail_entry_from_ram(&ram, slot);
    assert_eq!(entry.x_low(), 0xaa);
    assert_eq!(entry.y_low(), 0xbb);
    assert_eq!(entry.z_offset(), 0xcc);
    assert_eq!(entry.direction(), 0xdd);
}

#[test]
fn native_sprite_history_bridges_project_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MOLDORM_HISTORY_X_LO + 7] = 0xff;
    ram[MOLDORM_HISTORY_Y_LO + 7] = 0xee;
    ram[SWAMOLA_TARGET_X_LO + 2] = 0xdd;
    ram[BEAMOS_LASER_HISTORY_X_HI + 9] = 0xcc;

    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[MOLDORM_HISTORY_X_LO + 7] = 0x34;
    native_ram[MOLDORM_HISTORY_X_HI + 7] = 0x12;
    native_ram[MOLDORM_HISTORY_Y_LO + 7] = 0x78;
    native_ram[MOLDORM_HISTORY_Y_HI + 7] = 0x56;
    native_ram[SWAMOLA_TARGET_X_LO + 2] = 0x45;
    native_ram[SWAMOLA_TARGET_X_HI + 2] = 0x23;
    native_ram[SWAMOLA_TARGET_Y_LO + 2] = 0x89;
    native_ram[SWAMOLA_TARGET_Y_HI + 2] = 0x67;
    native_ram[BEAMOS_LASER_HISTORY_X_LO + 9] = 0x67;
    native_ram[BEAMOS_LASER_HISTORY_X_HI + 9] = 0x45;
    native_ram[BEAMOS_LASER_HISTORY_Y_LO + 9] = 0xab;
    native_ram[BEAMOS_LASER_HISTORY_Y_HI + 9] = 0x89;
    let mut effects = EffectState::load_from_ram(&native_ram);

    {
        let mut bridge =
            NativeMoldormHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 7);
        bridge.set_low_position(0xaa, 0xbb);
    }
    {
        let mut bridge =
            NativeSwamolaTargetBridgeMut::new(&mut effects.sprite_histories, &mut ram, 2);
        bridge.set_x_low(0xef);
    }
    {
        let mut bridge =
            NativeLanmolaSegmentMotionBridgeMut::new(&mut effects.sprite_histories, &mut ram, 9);
        bridge.set_z_offset(0x55);
    }

    assert_eq!(effects.sprite_histories.moldorm_history(7).x(), 0x12aa);
    assert_eq!(effects.sprite_histories.moldorm_history(7).y(), 0x56bb);
    assert_eq!(effects.sprite_histories.swamola_target(2).x(), 0x23ef);
    assert_eq!(effects.sprite_histories.beamos_laser_history(9).x(), 0x5567);
    assert_eq!(ram[MOLDORM_HISTORY_X_LO + 7], 0xaa);
    // set_low_position writes ONLY the low byte to RAM (like C's single-byte trail write);
    // the X_HI/Y_HI bytes alias a different flat trail slot for the lanmola, so syncing them
    // would clobber a neighbor (f220638). They stay at their prior RAM value (0 here).
    assert_eq!(ram[MOLDORM_HISTORY_X_HI + 7], 0);
    assert_eq!(ram[MOLDORM_HISTORY_Y_LO + 7], 0xbb);
    assert_eq!(ram[MOLDORM_HISTORY_Y_HI + 7], 0);
    assert_eq!(ram[SWAMOLA_TARGET_X_LO + 2], 0xef);
    assert_eq!(ram[SWAMOLA_TARGET_X_HI + 2], 0x23);
    assert_eq!(ram[BEAMOS_LASER_HISTORY_X_HI + 9], 0x55);

    let mut projected = vec![0; WRAM_SIZE];
    effects
        .sprite_histories
        .write_moldorm_history_to_ram(&mut projected);
    effects
        .sprite_histories
        .write_swamola_target_to_ram(&mut projected);
    effects
        .sprite_histories
        .write_lanmola_segment_motion_to_ram(&mut projected);
    assert_eq!(projected[MOLDORM_HISTORY_X_LO + 7], 0xaa);
    assert_eq!(projected[MOLDORM_HISTORY_X_HI + 7], 0x12);
    assert_eq!(projected[MOLDORM_HISTORY_Y_LO + 7], 0xbb);
    assert_eq!(projected[MOLDORM_HISTORY_Y_HI + 7], 0x56);
    assert_eq!(projected[SWAMOLA_TARGET_X_LO + 2], 0xef);
    assert_eq!(projected[SWAMOLA_TARGET_X_HI + 2], 0x23);
    assert_eq!(projected[BEAMOS_LASER_HISTORY_X_LO + 9], 0x67);
    assert_eq!(projected[BEAMOS_LASER_HISTORY_X_HI + 9], 0x55);
}

#[test]
fn native_cached_sprite_bridge_updates_alt_and_live_banks() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut state = SpriteState::load_from_ram(&ram);

    for (address, value) in [
        (SPRITE_TYPE, 0xaa),
        (SPRITE_X_LO, 0x11),
        (SPRITE_GRAPHICS, 0x55),
        (SPRITE_X_HI, 0x22),
        (SPRITE_Y_LO, 0x33),
        (SPRITE_Y_HI, 0x44),
    ] {
        ram[address + 3] = value;
    }

    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            3,
        );
        for field in CachedSpriteCacheField::C_SOURCE_ORDER[..7].iter().copied() {
            bridge.cache_field_from_live(field);
        }
    }
    let slot = state.cached_sprites.slot(3);
    assert!(!slot.is_active());
    assert_eq!(slot.type_byte(), 0xaa);
    assert_eq!(slot.y_high(), 0x44);
    assert_eq!(ram[ALT_SPRITE_TYPE + 3], 0xaa);
    assert_eq!(ram[ALT_SPRITE_X_LO + 3], 0x11);
    assert_eq!(ram[ALT_SPRITE_X_HI + 3], 0x22);
    assert_eq!(ram[ALT_SPRITE_Y_LO + 3], 0x33);
    assert_eq!(ram[ALT_SPRITE_Y_HI + 3], 0x44);
    assert_eq!(ram[ALT_SPRITE_GRAPHICS + 3], 0x55);

    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            3,
        );
        bridge.initialize_trinexx_component();
        bridge.set_type_byte(0x66);
        bridge.set_y_high(0x77);
    }
    assert_eq!(state.cached_sprites.slot(3).type_byte(), 0x66);
    assert_eq!(state.cached_sprites.slot(3).y_high(), 0x77);
    assert_eq!(ram[ALT_SPRITE_X_HI + 3], 0);

    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        ram[live + 3] = index as u8;
    }
    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            3,
        );
        for field in CachedSpriteCacheField::C_SOURCE_ORDER
            .iter()
            .copied()
            .skip(1)
        {
            bridge.cache_field_from_live(field);
        }
    }
    for (index, alt) in CACHED_SPRITE_ALT_FIELDS.iter().copied().enumerate() {
        assert_eq!(ram[alt + 3], index as u8);
    }

    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        ram[live + 3] = 0x80 | index as u8;
    }
    let mut backup = [0; 24];
    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            3,
        );
        bridge.load_cached_into_live(&mut backup);
        bridge.clear_state();
    }
    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        assert_eq!(backup[index], 0x80 | index as u8);
        assert_eq!(ram[live + 3], index as u8);
    }
    assert!(!state.cached_sprites.slot(3).is_active());
    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            3,
        );
        bridge.restore_live_from_backup(&backup);
    }
    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        assert_eq!(ram[live + 3], 0x80 | index as u8);
    }

    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            0x1a,
        );
        bridge.initialize_trinexx_component();
    }
    assert_eq!(state.cached_sprites.slot(0x1a).type_byte(), 0x40);
    assert_eq!(ram[ALT_SPRITE_TYPE + 0x1a], 0x40);
}

#[test]
fn native_cached_sprite_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0xff; WRAM_SIZE];
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[ALT_SPRITE_STATE + 3] = 1;
    native_ram[ALT_SPRITE_TYPE + 3] = 0x12;
    native_ram[ALT_SPRITE_X_LO + 3] = 0x34;
    native_ram[ALT_SPRITE_X_HI + 3] = 0x56;
    native_ram[ALT_SPRITE_Y_LO + 3] = 0x78;
    native_ram[ALT_SPRITE_Y_HI + 3] = 0x9a;
    native_ram[ALT_SPRITE_GRAPHICS + 3] = 0xbc;
    let mut state = SpriteState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            3,
        );
        bridge.set_type_byte(0x66);
        bridge.set_y_high(0x77);
    }

    let slot = state.cached_sprites.slot(3);
    assert!(slot.is_active());
    assert_eq!(slot.type_byte(), 0x66);
    assert_eq!(slot.y_high(), 0x77);
    assert_eq!(ram[ALT_SPRITE_STATE + 3], 0xff);
    assert_eq!(ram[ALT_SPRITE_TYPE + 3], 0x66);
    assert_eq!(ram[ALT_SPRITE_X_LO + 3], 0xff);
    assert_eq!(ram[ALT_SPRITE_X_HI + 3], 0xff);
    assert_eq!(ram[ALT_SPRITE_Y_LO + 3], 0xff);
    assert_eq!(ram[ALT_SPRITE_Y_HI + 3], 0x77);
    assert_eq!(ram[ALT_SPRITE_GRAPHICS + 3], 0xff);
}

#[test]
fn native_boss_home_positions_load_and_update_overlord_scratch() {
    let mut ram = vec![0; WRAM_SIZE];
    let puff_slot = 5;
    let puff_overlord_slot = puff_slot + 7;
    ram[OVERLORD_X_LO + puff_overlord_slot] = 0x34;
    ram[OVERLORD_Y_LO + puff_overlord_slot] = 0x12;
    ram[OVERLORD_GEN1 + puff_overlord_slot] = 0x78;
    ram[OVERLORD_GEN3 + puff_overlord_slot] = 0x56;

    let mut state = SpriteState::load_from_ram(&ram);
    let puff_home = state
        .boss_home_positions
        .arrghus_puff_home_position(puff_slot);
    assert_eq!(puff_home.x(), 0x1234);
    assert_eq!(puff_home.y(), 0x5678);

    {
        let mut home = NativeArrghusPuffHomePositionBridgeMut::new(
            &mut state.boss_home_positions,
            &mut ram,
            puff_slot,
        );
        home.set_position(0x1357, 0x2468);
    }
    let puff_home = state
        .boss_home_positions
        .arrghus_puff_home_position(puff_slot);
    assert_eq!(puff_home.x(), 0x1357);
    assert_eq!(puff_home.y(), 0x2468);
    assert_eq!(ram[OVERLORD_X_LO + puff_overlord_slot], 0x57);
    assert_eq!(ram[OVERLORD_Y_LO + puff_overlord_slot], 0x13);
    assert_eq!(ram[OVERLORD_GEN1 + puff_overlord_slot], 0x68);
    assert_eq!(ram[OVERLORD_GEN3 + puff_overlord_slot], 0x24);

    {
        let mut home = NativeArmosKnightHomePositionBridgeMut::new(
            &mut state.boss_home_positions,
            &mut ram,
            3,
        );
        home.set_position(0x9abc, 0xdef0);
    }
    let armos_home = state.boss_home_positions.armos_knight_home_position(3);
    assert_eq!(armos_home.x(), 0x9abc);
    assert_eq!(armos_home.y(), 0xdef0);
    assert_eq!(ram[OVERLORD_X_HI + 3], 0xbc);
    assert_eq!(ram[OVERLORD_Y_HI + 3], 0x9a);
    assert_eq!(ram[OVERLORD_GEN2 + 3], 0xf0);
    assert_eq!(ram[OVERLORD_FLOOR + 3], 0xde);
}

#[test]
fn native_boss_home_positions_project_native_state_over_stale_ram() {
    let mut ram = vec![0xff; WRAM_SIZE];
    let mut native_ram = vec![0; WRAM_SIZE];
    let puff_slot = 5;
    let puff_overlord_slot = puff_slot + 7;
    native_ram[OVERLORD_X_LO + puff_overlord_slot] = 0x34;
    native_ram[OVERLORD_Y_LO + puff_overlord_slot] = 0x12;
    native_ram[OVERLORD_GEN1 + puff_overlord_slot] = 0x78;
    native_ram[OVERLORD_GEN3 + puff_overlord_slot] = 0x56;
    native_ram[OVERLORD_X_HI + 3] = 0xaa;
    native_ram[OVERLORD_Y_HI + 3] = 0xbb;
    native_ram[OVERLORD_GEN2 + 3] = 0xcc;
    native_ram[OVERLORD_FLOOR + 3] = 0xdd;
    let mut state = SpriteState::load_from_ram(&native_ram);

    {
        let mut home = NativeArrghusPuffHomePositionBridgeMut::new(
            &mut state.boss_home_positions,
            &mut ram,
            puff_slot,
        );
        home.set_position(0x1357, 0x2468);
    }

    {
        let mut home = NativeArmosKnightHomePositionBridgeMut::new(
            &mut state.boss_home_positions,
            &mut ram,
            3,
        );
        home.set_position(0x9abc, 0xdef0);
    }

    let puff_home = state
        .boss_home_positions
        .arrghus_puff_home_position(puff_slot);
    let armos_home = state.boss_home_positions.armos_knight_home_position(3);
    assert_eq!(puff_home.x(), 0x1357);
    assert_eq!(puff_home.y(), 0x2468);
    assert_eq!(armos_home.x(), 0x9abc);
    assert_eq!(armos_home.y(), 0xdef0);
    assert_eq!(ram[OVERLORD_X_LO + puff_overlord_slot], 0x57);
    assert_eq!(ram[OVERLORD_Y_LO + puff_overlord_slot], 0x13);
    assert_eq!(ram[OVERLORD_GEN1 + puff_overlord_slot], 0x68);
    assert_eq!(ram[OVERLORD_GEN3 + puff_overlord_slot], 0x24);
    assert_eq!(ram[OVERLORD_X_HI + 3], 0xbc);
    assert_eq!(ram[OVERLORD_Y_HI + 3], 0x9a);
    assert_eq!(ram[OVERLORD_GEN2 + 3], 0xf0);
    assert_eq!(ram[OVERLORD_FLOOR + 3], 0xde);
}

#[test]
fn native_overworld_sprite_flag_bridges_project_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[OVERWORLD_SPRITE_PRESENCE + 3] = 0x12;
    native_ram[OVERWORLD_SPRITE_WAS_LOADED + 4] = 0b1010_0000;

    let mut ram = vec![0xff; WRAM_SIZE];
    // sprite_where_in_overworld presence projects only OUTDOORS (indoors the same
    // WRAM is the dungeon where_in_room bitmask), so exercise the outdoors path.
    ram[PLAYER_IS_INDOORS] = 0;
    let mut presence = OverworldSpritePresenceState::load_from_ram(&native_ram);
    {
        let mut bridge = NativeOverworldSpritePresenceBridgeMut::new(&mut presence, &mut ram);
        bridge.set_marker(3, 0x34);
    }
    assert_eq!(presence.marker(3), 0x34);
    assert_eq!(ram[OVERWORLD_SPRITE_PRESENCE + 3], 0x34);

    let mut loaded = OverworldSpriteLoadedState::load_from_ram(&native_ram);
    {
        let mut bridge = NativeOverworldSpriteLoadedBridgeMut::new(&mut loaded, &mut ram);
        bridge.clear_loaded_mask(32, 0b0010_0000);
        bridge.set_loaded_mask(32, 0b0000_0010);
    }
    assert!(loaded.is_loaded(32, 0b0000_0010));
    assert!(!loaded.is_loaded(32, 0b0010_0000));
    assert_eq!(ram[OVERWORLD_SPRITE_WAS_LOADED + 4], 0b1000_0010);
}

#[test]
fn native_sprite_workspace_bridge_allows_outdoor_presence_owner() {
    let native_ram = vec![0; WRAM_SIZE];
    let mut workspace = SpriteWorkspaceState::load_from_ram(&native_ram);
    let mut presence = OverworldSpritePresenceState::load_from_ram(&native_ram);
    let mut ram = vec![0; WRAM_SIZE];
    ram[PLAYER_IS_INDOORS] = 0;
    ram[SPRITE_WHERE_IN_ROOM + 0x123] = 0x41;

    {
        let mut bridge =
            NativeSpriteWorkspaceBridgeMut::new(&mut workspace, &mut presence, &mut ram);
        bridge.set_pickup_slot_cache(0x5a);
    }

    assert_eq!(workspace.pickup_slot_cache(), 0x5a);
    assert_eq!(ram[SPRITE_PICKUP_SLOT_CACHE], 0x5a);
    assert_eq!(ram[SPRITE_WHERE_IN_ROOM + 0x123], 0x41);
}

#[test]
fn native_failed_spin_sparkle_spawn_bridge_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[ANCILLA_STEP - 1] = 0x05;
    native_ram[ANCILLA_TIMER - 1] = 0x06;
    native_ram[ANCILLA_AUX_TIMER - 1] = 0x08;
    native_ram[ANCILLA_X_LO - 1] = 0xcd;
    native_ram[ANCILLA_X_HI - 1] = 0xab;
    native_ram[ANCILLA_Y_LO - 1] = 0x34;
    native_ram[ANCILLA_Y_HI - 1] = 0x12;
    let mut spawn = FailedSpinSparkleSpawnState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let _bridge = NativeFailedSpinSparkleSpawnBridgeMut::new(&mut spawn, &mut ram);
    }

    assert_eq!(spawn.step(), 0x05);
    assert_eq!(spawn.timer(), 0x06);
    assert_eq!(spawn.aux_timer(), 0x08);
    assert_eq!(spawn.x(), 0xabcd);
    assert_eq!(spawn.y(), 0x1234);

    {
        let mut bridge = NativeFailedSpinSparkleSpawnBridgeMut::new(&mut spawn, &mut ram);
        bridge.write_failed_spin_sparkle(0x07, 0x1234, 0x5678);
    }

    assert_eq!(spawn.step(), 0x07);
    assert_eq!(spawn.timer(), 4);
    assert_eq!(spawn.aux_timer(), 3);
    assert_eq!(spawn.x(), 0x1234);
    assert_eq!(spawn.y(), 0x5678);
    assert_eq!(ram[ANCILLA_ITEM_TO_LINK - 1], 0);
    assert_eq!(ram[ANCILLA_STEP - 1], 0x07);
    assert_eq!(ram[ANCILLA_TIMER - 1], 4);
    assert_eq!(ram[ANCILLA_AUX_TIMER - 1], 3);
    assert_eq!(ram[ANCILLA_X_LO - 1], 0x34);
    assert_eq!(ram[ANCILLA_X_HI - 1], 0x12);
    assert_eq!(ram[ANCILLA_Y_LO - 1], 0x78);
    assert_eq!(ram[ANCILLA_Y_HI - 1], 0x56);
}

#[test]
fn native_garnish_runtime_bridge_projects_native_state_over_stale_ram() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[GARNISH_ACTIVE] = 0x03;
    native_ram[OVERWORLD_BOULDER_TRAP_COUNT] = 0xff;
    native_ram[OVERWORLD_BOULDER_TRAP_TIMER] = 0x7f;
    native_ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = 0x22;
    native_ram[REPULSESPARK_ANIM_DELAY] = 0x00;
    write_le_u16(&mut native_ram, SPRCOLL_X_BASE, 0x1234);
    write_le_u16(&mut native_ram, SPRCOLL_Y_BASE, 0x5678);
    let mut garnish = GarnishRuntimeState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativeGarnishRuntimeBridgeMut::new(&mut garnish, &mut ram);
        bridge.set_active_type(0x0a);
        bridge.increment_boulder_trap_count();
        assert_eq!(bridge.increment_boulder_trap_timer(), 0x80);
        bridge.set_sprcoll_x_size(0x0102);
        bridge.set_sprcoll_y_size(0x0304);
        bridge.set_sprcoll_x_base(0x1112);
        bridge.set_sprcoll_y_base(0x1314);
        assert_eq!(bridge.decrement_repulsespark_anim_delay(), 0xff);
        bridge.clear_haunted_grove_flute_event_latch();
    }

    assert_eq!(garnish.active_type(), 0x0a);
    assert_eq!(garnish.boulder_trap_count(), 0x00);
    assert_eq!(garnish.boulder_trap_timer(), 0x80);
    assert_eq!(garnish.sprcoll_x_size(), 0x0102);
    assert_eq!(garnish.sprcoll_y_size(), 0x0304);
    assert_eq!(garnish.sprcoll_x_word(), 0x1112);
    assert_eq!(garnish.sprcoll_y_word(), 0x1314);
    assert_eq!(garnish.repulsespark_anim_delay(), 0xff);
    assert_eq!(garnish.haunted_grove_flute_event_latch(), 0);
    assert_eq!(ram[GARNISH_ACTIVE], 0x0a);
    assert_eq!(ram[OVERWORLD_BOULDER_TRAP_COUNT], 0);
    assert_eq!(ram[OVERWORLD_BOULDER_TRAP_TIMER], 0x80);
    assert_eq!(read_le_u16(&ram, SPRCOLL_X_SIZE), 0x0102);
    assert_eq!(read_le_u16(&ram, SPRCOLL_Y_SIZE), 0x0304);
    assert_eq!(read_le_u16(&ram, SPRCOLL_X_BASE), 0x1112);
    assert_eq!(read_le_u16(&ram, SPRCOLL_Y_BASE), 0x1314);
    assert_eq!(ram[REPULSESPARK_ANIM_DELAY], 0xff);
    assert_eq!(ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH], 0);
}

#[test]
fn native_follower_runtime_bridge_preserves_overlapping_timer_tail_byte() {
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[FOLLOWER_INDICATOR] = 0x04;
    native_ram[TAGALONG_DATA_INDEX] = 0x13;
    native_ram[TAGALONG_HOOKSHOT_INTERLOCK] = 0x02;
    native_ram[TIMER_TAGALONG_REACQUIRE] = 0x34;
    native_ram[FOLLOWER_TAIL_WRITE_INDEX] = 0x12;
    native_ram[TAGALONG_ANIM_FRAME_COUNTER] = 0x02;
    write_le_u16(&mut native_ram, FOLLOWER_SAVED_Y, 0x5678);
    write_le_u16(&mut native_ram, FOLLOWER_SAVED_X, 0x9abc);
    let mut follower = FollowerRuntimeState::load_from_ram(&native_ram);

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativeFollowerRuntimeBridgeMut::new(&mut follower, &mut ram);
        bridge.set_reacquire_timer(0xabcd);
        bridge.increment_tail_write_index();
        bridge.set_hookshot_release_tail_index_from_tail_write_index();
        bridge.advance_data_index_wrapping_at_20();
        bridge.increment_and_cycle_draw_anim_frame();
        bridge.set_saved_y(0x1112);
        bridge.set_saved_x(0x1314);
        bridge.set_palette_swap_flag(0x80);
    }

    assert_eq!(follower.reacquire_timer_low(), 0xcd);
    assert_eq!(follower.tail_write_index(), 0xac);
    assert_eq!(follower.reacquire_timer(), 0xaccd);
    assert_eq!(follower.hookshot_release_tail_index(), 0xac);
    assert_eq!(follower.data_index(), 0);
    assert_eq!(follower.draw_anim_frame(), 0);
    assert_eq!(follower.saved_y(), 0x1112);
    assert_eq!(follower.saved_x(), 0x1314);
    assert_eq!(follower.palette_swap_flag(), 0x80);
    assert_eq!(ram[TIMER_TAGALONG_REACQUIRE], 0xcd);
    assert_eq!(ram[FOLLOWER_TAIL_WRITE_INDEX], 0xac);
    assert_eq!(read_le_u16(&ram, TIMER_TAGALONG_REACQUIRE), 0xaccd);
    assert_eq!(ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX], 0xac);
    assert_eq!(ram[TAGALONG_DATA_INDEX], 0);
    assert_eq!(ram[TAGALONG_ANIM_FRAME_COUNTER], 0);
    assert_eq!(read_le_u16(&ram, FOLLOWER_SAVED_Y), 0x1112);
    assert_eq!(read_le_u16(&ram, FOLLOWER_SAVED_X), 0x1314);
    assert_eq!(ram[FOLLOWER_PALETTE_SWAP_FLAG], 0x80);
    assert_eq!(
        ram[ZELDA_RESCUE_CUTSCENE_STATE], 0xff,
        "bulk follower projection must preserve the independently write-through rescue byte"
    );

    {
        let mut bridge = NativeFollowerRuntimeBridgeMut::new(&mut follower, &mut ram);
        bridge.set_zelda_rescue_cutscene_state(2);
    }
    assert_eq!(follower.zelda_rescue_cutscene_state(), 2);
    assert_eq!(ram[ZELDA_RESCUE_CUTSCENE_STATE], 2);
}

#[test]
fn cached_sprite_nmi_split_restore_matches_the_rom_field_order() {
    // UncacheAndExecuteSprite executes the cached sprite before restoring the
    // displaced live slot in reverse order. At a 12-field cut the low half is
    // still the cached sprite's post-execution generation while the high half
    // has already returned to the displaced live generation.
    const LIVE_FIELDS: usize = 12;
    let slot = 2;
    let mut ram = vec![0; WRAM_SIZE];
    let mut state = SpriteState::load_from_ram(&ram);
    for (index, (live, alt)) in CACHED_SPRITE_LIVE_FIELDS
        .iter()
        .copied()
        .zip(CACHED_SPRITE_ALT_FIELDS.iter().copied())
        .enumerate()
    {
        ram[live + slot] = 0x80 | index as u8;
        ram[alt + slot] = index as u8;
    }

    let mut backup = [0; 24];
    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            slot,
        );
        bridge.load_cached_into_live(&mut backup);
    }
    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        ram[live + slot] = 0x40 | index as u8;
    }
    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            slot,
        );
        bridge.restore_live_suffix_from_backup_before_nmi(&backup, LIVE_FIELDS);
    }
    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        let expected = if index < LIVE_FIELDS {
            0x40 | index as u8
        } else {
            0x80 | index as u8
        };
        assert_eq!(ram[live + slot], expected, "field {index} at the boundary");
    }
    assert_eq!(ram[SPRITE_D + slot], 0x40 | 11);
    assert_eq!(ram[SPRITE_FLAGS2 + slot], 0x80 | 12);

    {
        let mut bridge = NativeCachedSpriteBridgeMut::new(
            &mut state.cached_sprites,
            &mut state.sprite_slots,
            &mut state.system,
            &mut ram,
            slot,
        );
        bridge.restore_live_prefix_from_backup_after_nmi(&backup, LIVE_FIELDS);
    }
    for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
        assert_eq!(
            ram[live + slot],
            0x80 | index as u8,
            "field {index} after the NMI"
        );
        assert_eq!(backup[index], 0x80 | index as u8, "backup {index}");
    }
}
