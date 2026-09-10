use super::*;
use crate::game_state::constants::{ANCILLA_ALLOC_ROTATE, ANCILLA_H, ANCILLA_T_PLAYER};
use crate::game_state::native::sprites::{AncillaSlotsState, SpriteWorkspaceState};
use crate::tile_definition::NativeTile;

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
fn dual_layer_tile_cache_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUAL_LAYER_TILE_CACHE] = 0x1c;
    ram[DUAL_LAYER_TILE_CACHE + 15] = 0x2a;

    let tile = |attribute| NativeTile::from_cartridge(attribute);
    let mut cache = DualLayerTileCacheState::load_from_ram(&ram);
    assert_eq!(cache.tile(0), tile(0x1c));
    assert_eq!(cache.tile(15), tile(0x2a));
    assert_eq!(cache.tile(16), tile(0));
    assert!(cache.set_tile(15, tile(0x3b)));
    assert!(!cache.set_tile(16, tile(0x4c)));
    assert_eq!(cache.tile(15), tile(0x3b));
    assert_eq!(cache.tile(16), tile(0));

    let mut projected = vec![0; WRAM_SIZE];
    cache.write_to_ram(&mut projected);
    assert_eq!(DualLayerTileCacheState::load_from_ram(&projected), cache);
    assert_eq!(projected[DUAL_LAYER_TILE_CACHE], 0x1c);
    assert_eq!(projected[DUAL_LAYER_TILE_CACHE + 15], 0x3b);
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
fn boss_home_reads_observe_shared_wram_without_reloading_native_state() {
    let mut game = crate::zelda_rtl::ZeldaState::new();
    // Frozen old Arrghus projection offsets, including its seven-slot bias.
    // Puff slots 1..26 share Armos slots 0..25's physical coordinate bytes.
    // The legacy Armos reader masks Y-high at slots 24+ beyond its work bank;
    // Arrghus reads those same bytes directly from WRAM.
    let bases = [0x0b0f, 0x0b1f, 0x0b2f, 0x0b3f];
    for puff_slot in 0..27 {
        let original = [0x34, 0x12, 0x78, 0x56];
        for (base, byte) in bases.into_iter().zip(original) {
            game.ram[base + puff_slot] = byte;
        }
        let captured = game.arrghus_puff_home_position(puff_slot);
        assert_eq!((captured.x(), captured.y()), (0x1234, 0x5678));

        for (base, byte) in bases.into_iter().zip([0xbc, 0x9a, 0xf0, 0xde]) {
            game.ram[base + puff_slot] = byte;
        }
        let current = game.arrghus_puff_home_position(puff_slot);
        assert_eq!((current.x(), current.y()), (0x9abc, 0xdef0));
        assert_eq!((captured.x(), captured.y()), (0x1234, 0x5678));
        if puff_slot != 0 {
            let aliased = game.armos_knight_home_position(puff_slot - 1);
            if puff_slot <= 24 {
                assert_eq!((aliased.x(), aliased.y()), (current.x(), current.y()));
            } else {
                assert_eq!(
                    (aliased.x(), aliased.y()),
                    (current.x(), current.y() & 0xff)
                );
            }
        }
    }
}

#[test]
fn armos_home_writes_only_four_shared_bytes_and_preserves_invalid_slot_noop() {
    let mut game = crate::zelda_rtl::ZeldaState::new();
    // Frozen old Armos projection addresses: X low/high, then Y low/high.
    let bases = [0x0b10, 0x0b20, 0x0b30, 0x0b40];
    let original: Vec<u8> = (0..WRAM_SIZE)
        .map(|address| (address.wrapping_mul(37) + address / 251) as u8)
        .collect();
    for slot in 0..27 {
        for (x, y) in [(0u16, 0xffffu16), (0x00ff, 0xff00), (0x1234, 0x5678)] {
            game.ram.clone_from(&original);
            let mut expected = original.clone();
            for (base, byte) in
                bases
                    .into_iter()
                    .zip([x as u8, (x >> 8) as u8, y as u8, (y >> 8) as u8])
            {
                expected[base + slot] = byte;
            }

            game.armos_knight_home_position_mut(slot).set_position(x, y);

            assert!(
                game.ram == expected,
                "Armos slot {slot} changed unrelated WRAM"
            );
            // All 27 writers retain their raw Y-high store, including the
            // bytes the bounded Armos reader does not expose at slots 24+.
            assert_eq!(game.ram[0x0b40 + slot], (y >> 8) as u8);
            let home = game.armos_knight_home_position(slot);
            let read_y = if slot < 24 { y } else { y & 0xff };
            assert_eq!((home.x(), home.y()), (x, read_y));
            if slot < 24 {
                let puff = game.arrghus_puff_home_position(slot + 1);
                assert_eq!((puff.x(), puff.y()), (home.x(), home.y()));
            }
        }
    }
    for invalid_slot in [27, 28, usize::MAX] {
        game.ram.clone_from(&original);
        game.armos_knight_home_position_mut(invalid_slot)
            .set_position(0, 0);
        assert!(game.ram == original, "invalid Armos slot must be a no-op");
    }
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

#[test]
fn ancilla_work_arrays_alias_alloc_rotate_and_ancilla_h_like_the_rom() {
    // Hardware layout: ancilla_arr26 at $03C0, ancilla_arr25 at $03C2,
    // ancilla_alloc_rotate at $03C4, ancilla_H at $03C5, ancilla_arr22 at
    // $03D2, ancilla_T at $03D5. The C port relocated arr26/arr25/arr22 to
    // break the overlaps; the ROM's fairy revival stores arr25[2] = 9 straight
    // into the allocation rotation, so the bank keeps every alias one cell.
    let mut ram = vec![0; WRAM_SIZE];
    let mut ancilla = AncillaSlotsState::load_from_ram(&ram);
    ancilla.slot_mut(&mut ram, 2).set_work_byte_25(9);
    assert_eq!(ram[ANCILLA_ALLOC_ROTATE], 9, "arr25[2] is $03C4");
    assert_eq!(
        ancilla.slot(4).work_byte_26(),
        9,
        "arr26[4] is the same byte"
    );
    ancilla.slot_mut(&mut ram, 1).set_work_byte_26(7);
    assert_eq!(ram[0x3c1], 7, "arr26[1] is $03C1");
    ancilla.slot_mut(&mut ram, 3).set_work_byte_25(0x55);
    assert_eq!(ram[ANCILLA_H], 0x55, "arr25[3] is ancilla_H[0]");
    assert_eq!(ancilla.slot(0).h(), 0x55);
    ancilla.slot_mut(&mut ram, 4).set_work_byte_22(0x66);
    assert_eq!(ram[ANCILLA_T_PLAYER + 1], 0x66, "arr22[4] is ancilla_T[1]");
    ancilla.set_shared_byte(ANCILLA_ALLOC_ROTATE, 4);
    assert_eq!(ancilla.slot(2).work_byte_25(), 4);
    // Round trip through RAM keeps a single value for the shared cell.
    ancilla.write_to_ram(&mut ram);
    assert_eq!(ram[ANCILLA_ALLOC_ROTATE], 4);
    assert_eq!(AncillaSlotsState::load_from_ram(&ram), ancilla);
}
