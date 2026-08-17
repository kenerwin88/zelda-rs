use super::*;

#[test]
fn effect_angle_scratch_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    for slot in 0..9 {
        ram[EFFECT_ANGLE_WORK + slot] = slot as u8;
    }

    let mut angles = EffectAngleScratchState::load_from_ram(&ram);
    assert_eq!(angles.angle(2), 2);
    assert_eq!(angles.trailing_angle(), 4);
    assert_eq!(angles.radial_radius(), 8);

    angles.set_angles4(&[10, 20, 30, 40], 0);
    assert_eq!(angles.add_angle_mod64(1, 50), 6);
    assert_eq!(angles.add_trailing_angle_mod64(63), 3);
    angles.set_radial_radius(14);
    angles.write_to_ram(&mut ram);

    assert_eq!(ram[EFFECT_ANGLE_WORK], 10);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 1], 6);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 4], 3);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 8], 14);
}

#[test]
fn native_effect_angle_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[EFFECT_ANGLE_WORK + 1] = 60;
    ram[EFFECT_ANGLE_WORK + 4] = 2;
    ram[EFFECT_ANGLE_WORK + 8] = 9;

    let mut angles = EffectAngleScratchState::load_from_ram(&ram);
    {
        let mut bridge = NativeEffectAngleScratchBridgeMut::new(&mut angles, &mut ram);
        bridge.set_angle(0, 12);
        bridge.set_angles4(&[1, 2, 3, 4, 5], 1);
        assert_eq!(bridge.add_angle_mod64(1, 63), 2);
        assert_eq!(bridge.add_trailing_angle_mod64(10), 12);
        bridge.set_radial_radius(20);
    }

    assert_eq!(angles.angle(0), 2);
    assert_eq!(angles.angle(1), 2);
    assert_eq!(angles.trailing_angle(), 12);
    assert_eq!(angles.radial_radius(), 20);
    assert_eq!(ram[EFFECT_ANGLE_WORK], 2);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 1], 2);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 4], 12);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 8], 20);
}

#[test]
fn native_effect_angle_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[EFFECT_ANGLE_WORK + 1] = 60;
    let mut ram = vec![0; WRAM_SIZE];
    ram[EFFECT_ANGLE_WORK + 1] = 3;
    ram[EFFECT_ANGLE_WORK + 8] = 9;
    let mut angles = EffectAngleScratchState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeEffectAngleScratchBridgeMut::new(&mut angles, &mut ram);
        assert_eq!(bridge.add_angle_mod64(1, 2), 5);
    }

    assert_eq!(angles.angle(1), 5);
    assert_eq!(angles.radial_radius(), 9);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 1], 5);
    assert_eq!(ram[EFFECT_ANGLE_WORK + 8], 9);
}

#[test]
fn quake_spell_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[QUAKE_ACTIVE_BOLT_LIMIT] = 4;
    ram[QUAKE_PENDING_STEP] = 1;
    write_le_u16(&mut ram, QUAKE_ORIGIN_X, 0x1234);
    write_le_u16(&mut ram, QUAKE_ORIGIN_Y, 0x5678);
    write_le_u16(&mut ram, QUAKE_SCREEN_SHAKE_Y, 3);

    let mut quake = QuakeSpellState::load_from_ram(&ram);
    assert_eq!(quake.active_bolt_limit(), 4);
    assert_eq!(quake.pending_step(), 1);
    assert_eq!(quake.origin_x(), 0x1234);
    assert_eq!(quake.origin_y(), 0x5678);
    assert_eq!(quake.screen_shake_y(), 3);
    assert_eq!(quake.invert_screen_shake_y(), 3);
    quake.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, QUAKE_SCREEN_SHAKE_Y), 0xfffd);
}

#[test]
fn native_quake_spell_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[QUAKE_ACTIVE_BOLT_LIMIT] = 4;
    ram[QUAKE_PENDING_STEP] = 1;
    write_le_u16(&mut ram, QUAKE_SCREEN_SHAKE_Y, 5);

    let mut quake = QuakeSpellState::load_from_ram(&ram);
    {
        let mut bridge = NativeQuakeSpellBridgeMut::new(&mut quake, &mut ram);
        bridge.set_active_bolt_limit(2);
        bridge.set_pending_step(3);
        bridge.set_origin(0x4567, 0x89ab);
        assert_eq!(bridge.invert_screen_shake_y(), 5);
        bridge.set_screen_shake_y(9);
    }

    assert_eq!(quake.active_bolt_limit(), 2);
    assert_eq!(quake.pending_step(), 3);
    assert_eq!(quake.origin_x(), 0x4567);
    assert_eq!(quake.origin_y(), 0x89ab);
    assert_eq!(quake.screen_shake_y(), 9);
    assert_eq!(ram[QUAKE_ACTIVE_BOLT_LIMIT], 2);
    assert_eq!(ram[QUAKE_PENDING_STEP], 3);
    assert_eq!(read_le_u16(&ram, QUAKE_ORIGIN_X), 0x4567);
    assert_eq!(read_le_u16(&ram, QUAKE_ORIGIN_Y), 0x89ab);
    assert_eq!(read_le_u16(&ram, QUAKE_SCREEN_SHAKE_Y), 9);
}

#[test]
fn native_quake_spell_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[QUAKE_ACTIVE_BOLT_LIMIT] = 9;
    write_le_u16(&mut stale_ram, QUAKE_ORIGIN_X, 0xffff);
    let mut ram = vec![0; WRAM_SIZE];
    stale_ram[QUAKE_PENDING_STEP] = 8;
    ram[QUAKE_ACTIVE_BOLT_LIMIT] = 2;
    ram[QUAKE_PENDING_STEP] = 3;
    write_le_u16(&mut ram, QUAKE_ORIGIN_X, 0x1234);
    write_le_u16(&mut ram, QUAKE_ORIGIN_Y, 0x5678);
    write_le_u16(&mut ram, QUAKE_SCREEN_SHAKE_Y, 0x0004);
    let mut quake = QuakeSpellState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeQuakeSpellBridgeMut::new(&mut quake, &mut ram);
        assert_eq!(bridge.invert_screen_shake_y(), 4);
    }

    assert_eq!(quake.active_bolt_limit(), 2);
    assert_eq!(quake.pending_step(), 3);
    assert_eq!(quake.origin_x(), 0x1234);
    assert_eq!(ram[QUAKE_ACTIVE_BOLT_LIMIT], 2);
    assert_eq!(ram[QUAKE_PENDING_STEP], 3);
    assert_eq!(read_le_u16(&ram, QUAKE_ORIGIN_X), 0x1234);
    assert_eq!(read_le_u16(&ram, QUAKE_SCREEN_SHAKE_Y), 0xfffc);
}

#[test]
fn native_quake_bolt_bridge_syncs_seeded_ram_and_dual_writes_slot_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[QUAKE_BOLT_TIMER + 2] = 7;
    ram[QUAKE_BOLT_PHASE + 2] = 0xfe;

    let mut bolts = QuakeBoltState::load_from_ram(&ram);
    {
        let mut bridge = NativeQuakeBoltBridgeMut::new(&mut bolts, &mut ram, 2);
        assert_eq!(bridge.tick_timer(), 6);
        assert_eq!(bridge.advance_phase(), 0xff);
        bridge.set_timer(1);
        bridge.set_phase(0x10);
    }

    assert_eq!(bolts.slot(2).timer(), 1);
    assert_eq!(bolts.slot(2).phase(), 0x10);
    assert_eq!(ram[QUAKE_BOLT_TIMER + 2], 1);
    assert_eq!(ram[QUAKE_BOLT_PHASE + 2], 0x10);
}

#[test]
fn native_quake_bolt_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[QUAKE_BOLT_TIMER + 2] = 0xff;
    stale_ram[QUAKE_BOLT_PHASE + 2] = 0xff;
    let mut ram = vec![0; WRAM_SIZE];
    ram[QUAKE_BOLT_TIMER + 2] = 7;
    ram[QUAKE_BOLT_PHASE + 2] = 1;
    let mut bolts = QuakeBoltState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeQuakeBoltBridgeMut::new(&mut bolts, &mut ram, 2);
        assert_eq!(bridge.tick_timer(), 6);
        assert_eq!(bridge.advance_phase(), 2);
    }

    assert_eq!(bolts.slot(2).timer(), 6);
    assert_eq!(bolts.slot(2).phase(), 2);
    assert_eq!(ram[QUAKE_BOLT_TIMER + 2], 6);
    assert_eq!(ram[QUAKE_BOLT_PHASE + 2], 2);
}

#[test]
fn bombos_spell_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BOMBOS_MODE] = 2;
    ram[BOMBOS_FIRE_COLUMN_RADIUS] = 16;
    ram[BOMBOS_BLAST_RELEASE_LOCKED] = 1;
    ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = 0x80;
    write_le_u16(&mut ram, BOMBOS_FIRE_COLUMN_SEED_X, 0x1234);
    write_le_u16(&mut ram, BOMBOS_FIRE_COLUMN_SEED_Y, 0x5678);
    write_le_u16(&mut ram, BOMBOS_BLAST_X + 4, 0x9abc);
    write_le_u16(&mut ram, BOMBOS_BLAST_Y + 4, 0xdef0);

    let mut bombos = BombosSpellState::load_from_ram(&ram);
    assert_eq!(bombos.mode(), 2);
    assert_eq!(bombos.fire_column_radius(), 16);
    assert!(bombos.blast_release_locked());
    assert_eq!(bombos.fire_column_seed_x(0), 0x1234);
    assert_eq!(bombos.fire_column_seed_y(0), 0x5678);
    assert_eq!(bombos.blast_x(2), 0x9abc);
    assert_eq!(bombos.blast_y(2), 0xdef0);

    bombos.set_mode(1);
    assert_eq!(bombos.grow_fire_column_radius(200, 207), 207);
    bombos.set_blast_release_locked(false);
    assert_eq!(bombos.tick_blast_release_countdown(), 0x7f);
    bombos.set_fire_column_seed_position(1, 0x1111, 0x2222);
    bombos.set_blast_position(3, 0x3333, 0x4444);
    bombos.write_to_ram(&mut ram);

    assert_eq!(ram[BOMBOS_MODE], 1);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIUS], 207);
    assert_eq!(ram[BOMBOS_BLAST_RELEASE_LOCKED], 0);
    assert_eq!(ram[BOMBOS_BLAST_RELEASE_COUNTDOWN], 0x7f);
    assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_X + 2), 0x1111);
    assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_Y + 2), 0x2222);
    assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_X + 6), 0x3333);
    assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_Y + 6), 0x4444);
}

#[test]
fn native_bombos_spell_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BOMBOS_FIRE_COLUMN_RADIUS] = 10;
    ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = 1;

    let mut bombos = BombosSpellState::load_from_ram(&ram);
    {
        let mut bridge = NativeBombosSpellBridgeMut::new(&mut bombos, &mut ram);
        bridge.set_mode(2);
        assert_eq!(bridge.grow_fire_column_radius(5, 207), 15);
        bridge.set_blast_release_locked(true);
        assert_eq!(bridge.tick_blast_release_countdown(), 0);
        bridge.set_blast_release_countdown(4);
        bridge.set_fire_column_seed_position(0, 0x1234, 0x5678);
        bridge.set_blast_position(15, 0x9abc, 0xdef0);
    }

    assert_eq!(bombos.mode(), 2);
    assert_eq!(bombos.fire_column_radius(), 15);
    assert!(bombos.blast_release_locked());
    assert_eq!(bombos.fire_column_seed_x(0), 0x1234);
    assert_eq!(bombos.fire_column_seed_y(0), 0x5678);
    assert_eq!(bombos.blast_x(15), 0x9abc);
    assert_eq!(bombos.blast_y(15), 0xdef0);
    assert_eq!(ram[BOMBOS_MODE], 2);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIUS], 15);
    assert_eq!(ram[BOMBOS_BLAST_RELEASE_LOCKED], 1);
    assert_eq!(ram[BOMBOS_BLAST_RELEASE_COUNTDOWN], 4);
    assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_X), 0x1234);
    assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_Y), 0x5678);
    assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_X + 30), 0x9abc);
    assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_Y + 30), 0xdef0);
}

#[test]
fn native_bombos_spell_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let stale_ram = vec![0xff; WRAM_SIZE];
    let mut ram = vec![0; WRAM_SIZE];
    ram[BOMBOS_MODE] = 1;
    ram[BOMBOS_FIRE_COLUMN_RADIUS] = 10;
    ram[BOMBOS_BLAST_RELEASE_LOCKED] = 1;
    ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = 2;
    write_le_u16(&mut ram, BOMBOS_FIRE_COLUMN_SEED_X + 2, 0x1234);
    write_le_u16(&mut ram, BOMBOS_FIRE_COLUMN_SEED_Y + 2, 0x5678);
    write_le_u16(&mut ram, BOMBOS_BLAST_X + 8, 0x9abc);
    write_le_u16(&mut ram, BOMBOS_BLAST_Y + 8, 0xdef0);
    let mut bombos = BombosSpellState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeBombosSpellBridgeMut::new(&mut bombos, &mut ram);
        assert_eq!(bridge.grow_fire_column_radius(5, 207), 15);
        assert_eq!(bridge.tick_blast_release_countdown(), 1);
        bridge.set_fire_column_seed_position(2, 0x1111, 0x2222);
        bridge.set_blast_position(4, 0x3333, 0x4444);
    }

    assert_eq!(bombos.mode(), 1);
    assert_eq!(bombos.fire_column_radius(), 15);
    assert!(bombos.blast_release_locked());
    assert_eq!(bombos.fire_column_seed_x(2), 0x1111);
    assert_eq!(bombos.fire_column_seed_y(2), 0x2222);
    assert_eq!(bombos.blast_x(4), 0x3333);
    assert_eq!(bombos.blast_y(4), 0x4444);
    assert_eq!(ram[BOMBOS_MODE], 1);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIUS], 15);
    assert_eq!(ram[BOMBOS_BLAST_RELEASE_LOCKED], 1);
    assert_eq!(ram[BOMBOS_BLAST_RELEASE_COUNTDOWN], 1);
    assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_X + 4), 0x1111);
    assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_Y + 4), 0x2222);
    assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_X + 8), 0x3333);
    assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_Y + 8), 0x4444);
}

#[test]
fn native_bombos_slot_bridges_preserve_overlapping_fire_column_layout() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BOMBOS_FIRE_COLUMN_TIMER + 3] = 1;
    ram[BOMBOS_FIRE_COLUMN_PHASE + 3] = 0xfe;
    ram[BOMBOS_BLAST_TIMER + 7] = 1;
    ram[BOMBOS_BLAST_PHASE + 7] = 0xfe;

    let mut bombos = BombosSpellState::load_from_ram(&ram);
    {
        let mut column = NativeBombosFireColumnBridgeMut::new(&mut bombos, &mut ram, 3);
        assert_eq!(column.tick_timer(), 0);
        assert_eq!(column.advance_phase(), 0xff);
        column.set_position(0x1234, 0x56cc);
    }
    assert_eq!(bombos.fire_column(3).timer(), 0);
    assert_eq!(bombos.fire_column(3).phase(), 0xff);
    assert_eq!(bombos.fire_column(3).x(), 0x1234);
    assert_eq!(bombos.fire_column(3).y(), 0x56cc);

    {
        let mut column = NativeBombosFireColumnBridgeMut::new(&mut bombos, &mut ram, 7);
        column.set_radial_angle(0x77);
    }
    assert_eq!(bombos.fire_column(3).y(), 0x5677);
    assert_eq!(bombos.fire_column(7).radial_angle(), 0x77);

    {
        let mut blast = NativeBombosBlastBridgeMut::new(&mut bombos, &mut ram, 7);
        assert_eq!(blast.tick_timer(), 0);
        assert_eq!(blast.advance_phase(), 0xff);
    }
    assert_eq!(bombos.blast(7).phase(), 0xff);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_TIMER + 3], 0);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_PHASE + 3], 0xff);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_X_LO + 3], 0x34);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_X_HI + 3], 0x12);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_Y_LO + 3], 0x77);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_Y_HI + 3], 0x56);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + 7], 0x77);
    assert_eq!(ram[BOMBOS_BLAST_TIMER + 7], 0);
    assert_eq!(ram[BOMBOS_BLAST_PHASE + 7], 0xff);
}

#[test]
fn native_bombos_slot_bridges_compose_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let stale_ram = vec![0xff; WRAM_SIZE];
    let mut ram = vec![0; WRAM_SIZE];
    ram[BOMBOS_FIRE_COLUMN_TIMER + 3] = 2;
    ram[BOMBOS_FIRE_COLUMN_PHASE + 3] = 0x40;
    ram[BOMBOS_FIRE_COLUMN_X_LO + 3] = 0x34;
    ram[BOMBOS_FIRE_COLUMN_X_HI + 3] = 0x12;
    ram[BOMBOS_FIRE_COLUMN_Y_LO + 3] = 0x78;
    ram[BOMBOS_FIRE_COLUMN_Y_HI + 3] = 0x56;
    ram[BOMBOS_BLAST_TIMER + 7] = 2;
    ram[BOMBOS_BLAST_PHASE + 7] = 0x80;
    let mut bombos = BombosSpellState::load_from_ram(&stale_ram);

    {
        let mut column = NativeBombosFireColumnBridgeMut::new(&mut bombos, &mut ram, 3);
        assert_eq!(column.tick_timer(), 1);
        assert_eq!(column.advance_phase(), 0x41);
    }
    {
        let mut column = NativeBombosFireColumnBridgeMut::new(&mut bombos, &mut ram, 7);
        column.set_radial_angle(0x9a);
    }
    {
        let mut blast = NativeBombosBlastBridgeMut::new(&mut bombos, &mut ram, 7);
        assert_eq!(blast.tick_timer(), 1);
        assert_eq!(blast.advance_phase(), 0x81);
    }

    assert_eq!(bombos.fire_column(3).timer(), 1);
    assert_eq!(bombos.fire_column(3).phase(), 0x41);
    assert_eq!(bombos.fire_column(3).x(), 0x1234);
    assert_eq!(bombos.fire_column(3).y(), 0x569a);
    assert_eq!(bombos.fire_column(7).radial_angle(), 0x9a);
    assert_eq!(bombos.blast(7).timer(), 1);
    assert_eq!(bombos.blast(7).phase(), 0x81);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_TIMER + 3], 1);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_PHASE + 3], 0x41);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_X_LO + 3], 0x34);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_X_HI + 3], 0x12);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_Y_LO + 3], 0x9a);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_Y_HI + 3], 0x56);
    assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + 7], 0x9a);
    assert_eq!(ram[BOMBOS_BLAST_TIMER + 7], 1);
    assert_eq!(ram[BOMBOS_BLAST_PHASE + 7], 0x81);
}

#[test]
fn tower_seal_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TOWER_SEAL_RING_RADIUS] = 32;
    write_le_u16(&mut ram, TOWER_SEAL_CENTER_X, 0x1234);
    write_le_u16(&mut ram, TOWER_SEAL_CENTER_Y, 0x5678);
    ram[TOWER_SEAL_WAIT_COUNTDOWN] = 2;

    let mut tower = TowerSealState::load_from_ram(&ram);
    assert_eq!(tower.ring_radius(), 32);
    assert_eq!(tower.center_x(), 0x1234);
    assert_eq!(tower.center_y(), 0x5678);
    tower.set_ring_radius(48);
    tower.set_center(0x9abc, 0xdef0);
    assert_eq!(tower.tick_wait_countdown(), 1);
    tower.write_to_ram(&mut ram);

    assert_eq!(ram[TOWER_SEAL_RING_RADIUS], 48);
    assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_X), 0x9abc);
    assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_Y), 0xdef0);
    assert_eq!(ram[TOWER_SEAL_WAIT_COUNTDOWN], 1);
}

#[test]
fn native_tower_seal_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TOWER_SEAL_WAIT_COUNTDOWN] = 1;

    let mut tower = TowerSealState::load_from_ram(&ram);
    {
        let mut bridge = NativeTowerSealBridgeMut::new(&mut tower, &mut ram);
        bridge.set_ring_radius(48);
        bridge.set_center(0x1234, 0x5678);
        assert_eq!(bridge.tick_wait_countdown(), 0);
        bridge.set_wait_countdown(240);
    }

    assert_eq!(tower.ring_radius(), 48);
    assert_eq!(tower.center_x(), 0x1234);
    assert_eq!(tower.center_y(), 0x5678);
    assert_eq!(ram[TOWER_SEAL_RING_RADIUS], 48);
    assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_X), 0x1234);
    assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_Y), 0x5678);
    assert_eq!(ram[TOWER_SEAL_WAIT_COUNTDOWN], 240);
}

#[test]
fn native_tower_seal_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[TOWER_SEAL_RING_RADIUS] = 0xff;
    stale_ram[TOWER_SEAL_WAIT_COUNTDOWN] = 0xee;

    let mut ram = vec![0; WRAM_SIZE];
    ram[TOWER_SEAL_RING_RADIUS] = 12;
    ram[TOWER_SEAL_WAIT_COUNTDOWN] = 3;
    write_le_u16(&mut ram, TOWER_SEAL_CENTER_X, 0x1234);
    write_le_u16(&mut ram, TOWER_SEAL_CENTER_Y, 0x5678);
    let mut tower = TowerSealState::load_from_ram(&stale_ram);

    {
        let mut bridge = NativeTowerSealBridgeMut::new(&mut tower, &mut ram);
        assert_eq!(bridge.tick_wait_countdown(), 2);
    }

    assert_eq!(tower.ring_radius(), 12);
    assert_eq!(tower.center_x(), 0x1234);
    assert_eq!(tower.center_y(), 0x5678);
    assert_eq!(ram[TOWER_SEAL_RING_RADIUS], 12);
    assert_eq!(ram[TOWER_SEAL_WAIT_COUNTDOWN], 2);
    assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_X), 0x1234);
    assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_Y), 0x5678);
}

#[test]
fn native_tower_seal_slot_bridges_sync_transient_orbits_and_sparkles() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TOWER_SEAL_ORBIT_ANGLE + 2] = 0x3f;
    ram[TOWER_SEAL_SPARKLE_TIMER + 5] = 1;

    let mut tower = TowerSealState::load_from_ram(&ram);
    {
        let mut orbit = NativeTowerSealOrbitBridgeMut::new(&mut tower, &mut ram, 2);
        assert_eq!(orbit.advance_angle_mod64(), 0);
        orbit.set_base_sparkle_position(0x1234, 0x5678);
    }
    {
        let mut sparkle = NativeTowerSealSparkleBridgeMut::new(&mut tower, &mut ram, 5);
        sparkle.set_phase(1);
        assert_eq!(sparkle.tick_timer(), 0);
        assert_eq!(sparkle.advance_phase(), 2);
        sparkle.set_position(0x9abc, 0xdef0);
        assert_eq!(sparkle.base_sparkle_position(2), (0x1234, 0x5678));
    }

    assert_eq!(tower.orbit(2).angle(), 0);
    assert_eq!(tower.sparkle(5).phase(), 2);
    assert_eq!(tower.sparkle(5).x(), 0x9abc);
    assert_eq!(tower.sparkle(5).y(), 0xdef0);
    assert_eq!(ram[TOWER_SEAL_ORBIT_ANGLE + 2], 0);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_X_LO + 2], 0x34);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_X_HI + 2], 0x12);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + 2], 0x78);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + 2], 0x56);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_PHASE + 5], 2);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_TIMER + 5], 0);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_X_LO + 5], 0xbc);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_X_HI + 5], 0x9a);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_Y_LO + 5], 0xf0);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_Y_HI + 5], 0xde);
}

#[test]
fn native_tower_seal_slot_bridges_compose_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[TOWER_SEAL_ORBIT_ANGLE + 2] = 0xff;
    stale_ram[TOWER_SEAL_SPARKLE_PHASE + 5] = 0xee;

    let mut ram = vec![0; WRAM_SIZE];
    ram[TOWER_SEAL_ORBIT_ANGLE + 2] = 0x3f;
    ram[TOWER_SEAL_SPARKLE_PHASE + 5] = 7;
    ram[TOWER_SEAL_SPARKLE_TIMER + 5] = 2;
    ram[TOWER_SEAL_BASE_SPARKLE_X_LO + 2] = 0x34;
    ram[TOWER_SEAL_BASE_SPARKLE_X_HI + 2] = 0x12;
    ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + 2] = 0x78;
    ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + 2] = 0x56;
    let mut tower = TowerSealState::load_from_ram(&stale_ram);

    {
        let mut orbit = NativeTowerSealOrbitBridgeMut::new(&mut tower, &mut ram, 2);
        assert_eq!(orbit.advance_angle_mod64(), 0);
    }
    {
        let mut sparkle = NativeTowerSealSparkleBridgeMut::new(&mut tower, &mut ram, 5);
        assert_eq!(sparkle.tick_timer(), 1);
        assert_eq!(sparkle.base_sparkle_position(2), (0x1234, 0x5678));
    }

    assert_eq!(tower.orbit(2).angle(), 0);
    assert_eq!(tower.sparkle(5).phase(), 7);
    assert_eq!(ram[TOWER_SEAL_ORBIT_ANGLE + 2], 0);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_PHASE + 5], 7);
    assert_eq!(ram[TOWER_SEAL_SPARKLE_TIMER + 5], 1);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_X_LO + 2], 0x34);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_X_HI + 2], 0x12);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + 2], 0x78);
    assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + 2], 0x56);
}

#[test]
fn skull_woods_fire_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[SKULL_WOODS_FIRE_STARTED] = 1;
    write_le_u16(&mut ram, SKULL_WOODS_FIRE_INNER_X, 0x1234);
    write_le_u16(&mut ram, SKULL_WOODS_FIRE_INNER_Y, 0x5678);
    write_le_u16(&mut ram, SKULL_WOODS_FIRE_OUTER_X, 0x9abc);
    write_le_u16(&mut ram, SKULL_WOODS_FIRE_OUTER_Y, 0xdef0);

    let mut fire = SkullWoodsFireState::load_from_ram(&ram);
    assert!(fire.has_started_entrance_opening());
    assert_eq!(fire.inner_x(), 0x1234);
    assert_eq!(fire.inner_y(), 0x5678);
    fire.clear_entrance_opening_started();
    assert_eq!(fire.retreat_inner_y(8), 0x5670);
    fire.set_inner_position(0x1111, 0x2222);
    fire.set_outer_position(0x3333, 0x4444);
    fire.write_to_ram(&mut ram);

    assert_eq!(ram[SKULL_WOODS_FIRE_STARTED], 0);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_X), 0x1111);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_Y), 0x2222);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_X), 0x3333);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_Y), 0x4444);
}

#[test]
fn native_skull_woods_fire_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SKULL_WOODS_FIRE_INNER_Y, 0x0100);

    let mut effects = EntranceEffectState::load_from_ram(&ram);
    {
        let mut bridge = NativeSkullWoodsFireBridgeMut::new(&mut effects, &mut ram);
        bridge.set_entrance_opening_started();
        bridge.set_inner_position(0x0098, 0x0100);
        bridge.set_outer_position(0x0098, 0x0100);
        assert_eq!(bridge.retreat_inner_y(8), 0x00f8);
    }

    let fire = effects.skull_woods_fire();
    assert!(fire.has_started_entrance_opening());
    assert_eq!(fire.inner_x(), 0x0098);
    assert_eq!(fire.inner_y(), 0x00f8);
    assert_eq!(ram[SKULL_WOODS_FIRE_STARTED], 1);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_X), 0x0098);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_Y), 0x00f8);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_X), 0x0098);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_Y), 0x0100);
}

#[test]
fn blast_wall_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BLAST_WALL_ENTRY_STATE] = 3;
    ram[BLAST_WALL_SECONDARY_STATE] = 4;
    ram[BLAST_WALL_DIRECTION] = 2;
    write_le_u16(&mut ram, BLAST_WALL_CENTER_X, 0x1234);
    write_le_u16(&mut ram, BLAST_WALL_CENTER_Y, 0x5678);

    let mut wall = BlastWallState::load_from_ram(&ram);
    assert_eq!(wall.direction(), 2);
    assert_eq!(wall.center_x(), 0x1234);
    assert_eq!(wall.center_y(), 0x5678);
    wall.clear_entry_state();
    wall.clear_secondary_state();
    assert_eq!(wall.offset_center(-4, 8), (0x1230, 0x5680));
    wall.write_to_ram(&mut ram);

    assert_eq!(ram[BLAST_WALL_ENTRY_STATE], 0);
    assert_eq!(ram[BLAST_WALL_SECONDARY_STATE], 0);
    assert_eq!(ram[BLAST_WALL_DIRECTION], 2);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_X), 0x1230);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_Y), 0x5680);
}

#[test]
fn native_blast_wall_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BLAST_WALL_ENTRY_STATE] = 1;
    ram[BLAST_WALL_SECONDARY_STATE] = 1;
    write_le_u16(&mut ram, BLAST_WALL_CENTER_X, 0x0100);
    write_le_u16(&mut ram, BLAST_WALL_CENTER_Y, 0x0200);

    let mut effects = EntranceEffectState::load_from_ram(&ram);
    {
        let mut bridge = NativeBlastWallBridgeMut::new(&mut effects, &mut ram);
        bridge.clear_entry_state();
        bridge.clear_secondary_state();
        assert_eq!(bridge.offset_center(2, -3), (0x0102, 0x01fd));
    }

    let wall = effects.blast_wall();
    assert_eq!(wall.center_x(), 0x0102);
    assert_eq!(wall.center_y(), 0x01fd);
    assert_eq!(ram[BLAST_WALL_ENTRY_STATE], 0);
    assert_eq!(ram[BLAST_WALL_SECONDARY_STATE], 0);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_X), 0x0102);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_Y), 0x01fd);
}

#[test]
fn native_blast_wall_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BLAST_WALL_ENTRY_STATE] = 0xff;
    write_le_u16(&mut ram, BLAST_WALL_CENTER_X, 0xffff);
    write_le_u16(&mut ram, BLAST_WALL_CENTER_Y, 0xeeee);

    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[BLAST_WALL_ENTRY_STATE] = 1;
    native_ram[BLAST_WALL_SECONDARY_STATE] = 1;
    native_ram[BLAST_WALL_DIRECTION] = 2;
    write_le_u16(&mut native_ram, BLAST_WALL_CENTER_X, 0x0100);
    write_le_u16(&mut native_ram, BLAST_WALL_CENTER_Y, 0x0200);
    let mut effects = EntranceEffectState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeBlastWallBridgeMut::new(&mut effects, &mut ram);
        bridge.clear_entry_state();
    }

    let wall = effects.blast_wall();
    assert_eq!(wall.direction(), 2);
    assert_eq!(wall.center_x(), 0x0100);
    assert_eq!(wall.center_y(), 0x0200);
    assert_eq!(ram[BLAST_WALL_ENTRY_STATE], 0);
    assert_eq!(ram[BLAST_WALL_SECONDARY_STATE], 1);
    assert_eq!(ram[BLAST_WALL_DIRECTION], 2);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_X), 0x0100);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_Y), 0x0200);
}

#[test]
fn entrance_effect_bank_syncs_shared_blast_wall_and_skull_woods_slots() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BLAST_WALL_EXPLOSION_PHASE] = 2;
    ram[BLAST_WALL_EXPLOSION_TIMER] = 3;
    write_le_u16(&mut ram, BLAST_WALL_FRAGMENT_X + 4, 0x0100);
    write_le_u16(&mut ram, BLAST_WALL_FRAGMENT_Y + 4, 0x0200);
    ram[BLAST_WALL_FIREBALL_TIMER + 7] = 9;

    let effects = EntranceEffectState::load_from_ram(&ram);
    assert_eq!(effects.blast_wall_explosion_slot(0).phase(), 2);
    assert_eq!(effects.blast_wall_explosion_slot(0).timer(), 3);
    assert_eq!(effects.blast_wall_fragment_slot(2).x(), 0x0100);
    assert_eq!(effects.skull_woods_fire_slot(2).y(), 0x0200);
    assert_eq!(effects.blast_wall_fireball_slot(7).timer(), 9);

    let mut effects = EntranceEffectState::load_from_ram(&ram);
    NativeSkullWoodsFireSlotBridgeMut::new(&mut effects, &mut ram, 2).set_phase(0xff);
    NativeSkullWoodsFireSlotBridgeMut::new(&mut effects, &mut ram, 2).set_timer(5);
    NativeSkullWoodsFireSlotBridgeMut::new(&mut effects, &mut ram, 2).set_position(0x0300, 0x0400);
    NativeBlastWallFireballBridgeMut::new(&mut effects, &mut ram, 7).set_timer(8);

    assert!(effects.skull_woods_fire_slot(2).is_finished());
    assert_eq!(ram[SKULL_WOODS_FIRE_PHASE + 2], 0xff);
    assert_eq!(ram[SKULL_WOODS_FIRE_TIMER + 2], 5);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_X + 4), 0x0300);
    assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_Y + 4), 0x0400);
    assert_eq!(ram[BLAST_WALL_FIREBALL_TIMER + 7], 8);
}

#[test]
fn native_entrance_effect_slot_bridges_project_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BLAST_WALL_EXPLOSION_PHASE + 2] = 0xff;
    ram[BLAST_WALL_EXPLOSION_TIMER + 2] = 0xee;
    ram[BLAST_WALL_FIREBALL_TIMER + 7] = 0xdd;

    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[BLAST_WALL_EXPLOSION_PHASE + 2] = 3;
    native_ram[BLAST_WALL_EXPLOSION_TIMER + 2] = 4;
    write_le_u16(&mut native_ram, BLAST_WALL_FRAGMENT_X + 4, 0x0100);
    write_le_u16(&mut native_ram, BLAST_WALL_FRAGMENT_Y + 4, 0x0200);
    native_ram[BLAST_WALL_FIREBALL_TIMER + 7] = 9;
    let mut effects = EntranceEffectState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeBlastWallExplosionBridgeMut::new(&mut effects, &mut ram, 2);
        assert_eq!(bridge.advance_phase(), 4);
    }
    {
        let mut bridge = NativeBlastWallFragmentBridgeMut::new(&mut effects, &mut ram, 2);
        assert_eq!(bridge.offset(0x10, -0x20), (0x0110, 0x01e0));
    }
    {
        let mut bridge = NativeBlastWallFireballBridgeMut::new(&mut effects, &mut ram, 7);
        assert_eq!(bridge.tick_timer(), 8);
    }

    assert_eq!(effects.blast_wall_explosion_slot(2).phase(), 4);
    assert_eq!(effects.blast_wall_explosion_slot(2).timer(), 4);
    assert_eq!(effects.blast_wall_fragment_slot(2).x(), 0x0110);
    assert_eq!(effects.blast_wall_fragment_slot(2).y(), 0x01e0);
    assert_eq!(effects.blast_wall_fireball_slot(7).timer(), 8);
    assert_eq!(ram[BLAST_WALL_EXPLOSION_PHASE + 2], 4);
    assert_eq!(ram[BLAST_WALL_EXPLOSION_TIMER + 2], 4);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_FRAGMENT_X + 4), 0x0110);
    assert_eq!(read_le_u16(&ram, BLAST_WALL_FRAGMENT_Y + 4), 0x01e0);
    assert_eq!(ram[BLAST_WALL_FIREBALL_TIMER + 7], 8);
}

#[test]
fn digging_game_prize_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIGGING_GAME_PRIZE_ATTEMPTS] = 24;
    ram[DIGGING_GAME_PRIZE_SPAWNED] = 0;

    let mut prize = DiggingGamePrizeState::load_from_ram(&ram);
    assert_eq!(prize.attempts(), 24);
    assert_eq!(prize.spawned_marker(), 0);
    prize.increment_attempts();
    prize.mark_spawned();
    prize.write_to_ram(&mut ram);

    assert_eq!(ram[DIGGING_GAME_PRIZE_ATTEMPTS], 25);
    assert_eq!(ram[DIGGING_GAME_PRIZE_SPAWNED], 0xeb);
}

#[test]
fn native_digging_game_prize_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIGGING_GAME_PRIZE_ATTEMPTS] = 0xff;
    ram[DIGGING_GAME_PRIZE_SPAWNED] = 0xeb;

    let mut prize = DiggingGamePrizeState::load_from_ram(&ram);
    {
        let mut bridge = NativeDiggingGamePrizeBridgeMut::new(&mut prize, &mut ram);
        bridge.increment_attempts();
        bridge.clear_prize_spawned();
    }

    assert_eq!(prize.attempts(), 0);
    assert_eq!(prize.spawned_marker(), 0);
    assert_eq!(ram[DIGGING_GAME_PRIZE_ATTEMPTS], 0);
    assert_eq!(ram[DIGGING_GAME_PRIZE_SPAWNED], 0);
}

#[test]
fn native_digging_game_prize_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIGGING_GAME_PRIZE_ATTEMPTS] = 0xff;
    ram[DIGGING_GAME_PRIZE_SPAWNED] = 0xeb;

    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[DIGGING_GAME_PRIZE_ATTEMPTS] = 9;
    native_ram[DIGGING_GAME_PRIZE_SPAWNED] = 0;
    let mut prize = DiggingGamePrizeState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDiggingGamePrizeBridgeMut::new(&mut prize, &mut ram);
        bridge.increment_attempts();
    }

    assert_eq!(prize.attempts(), 10);
    assert_eq!(prize.spawned_marker(), 0);
    assert_eq!(ram[DIGGING_GAME_PRIZE_ATTEMPTS], 10);
    assert_eq!(ram[DIGGING_GAME_PRIZE_SPAWNED], 0);
}

#[test]
fn door_debris_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DOOR_DEBRIS_X + 4, 0x1234);
    write_le_u16(&mut ram, DOOR_DEBRIS_Y + 4, 0x5678);
    ram[DOOR_DEBRIS_X + 7] = 0x9a;
    ram[DOOR_DEBRIS_Y + 7] = 0xbc;
    ram[DOOR_DEBRIS_DIRECTION + 7] = 3;

    let debris = DoorDebrisState::load_from_ram(&ram);
    assert_eq!(debris.x_word(2), 0x1234);
    assert_eq!(debris.y_word(2), 0x5678);
    assert_eq!(debris.x(7), 0x9a);
    assert_eq!(debris.y(7), 0xbc);
    assert_eq!(debris.direction(7), 3);
    assert_eq!(debris.x_word(5), 0);

    let mut projected = vec![0; WRAM_SIZE];
    debris.write_to_ram(&mut projected);
    assert_eq!(DoorDebrisState::load_from_ram(&projected), debris);
}

#[test]
fn native_door_debris_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DOOR_DEBRIS_X + 3] = 0xff;
    ram[DOOR_DEBRIS_Y + 3] = 0xff;
    ram[DOOR_DEBRIS_DIRECTION + 3] = 0xff;

    let mut debris = DoorDebrisState::load_from_ram(&ram);
    {
        let mut bridge = NativeDoorDebrisBridgeMut::new(&mut debris, &mut ram);
        bridge.set_y_low_and_x_low_from_word(3, 0x1234);
        bridge.set_x_word(2, 0x4567);
        bridge.set_y_word(2, 0x89ab);
        bridge.set_direction(3, 2);
        bridge.set_direction(12, 1);
    }

    assert_eq!(debris.x(3), 0x12);
    assert_eq!(debris.y(3), 0x34);
    assert_eq!(debris.x_word(2), 0x4567);
    assert_eq!(debris.y_word(2), 0x89ab);
    assert_eq!(debris.direction(3), 2);
    assert_eq!(ram[DOOR_DEBRIS_X + 3], 0x12);
    assert_eq!(ram[DOOR_DEBRIS_Y + 3], 0x34);
    assert_eq!(read_le_u16(&ram, DOOR_DEBRIS_X + 4), 0x4567);
    assert_eq!(read_le_u16(&ram, DOOR_DEBRIS_Y + 4), 0x89ab);
    assert_eq!(ram[DOOR_DEBRIS_DIRECTION + 3], 2);
}

#[test]
fn native_door_debris_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DOOR_DEBRIS_X + 3] = 0xff;
    ram[DOOR_DEBRIS_Y + 3] = 0xee;
    ram[DOOR_DEBRIS_DIRECTION + 3] = 0xdd;

    let mut native_ram = vec![0; WRAM_SIZE];
    ram[DOOR_DEBRIS_X + 1] = 0x99;
    native_ram[DOOR_DEBRIS_X + 3] = 0x12;
    native_ram[DOOR_DEBRIS_Y + 3] = 0x34;
    native_ram[DOOR_DEBRIS_DIRECTION + 3] = 1;
    let mut debris = DoorDebrisState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDoorDebrisBridgeMut::new(&mut debris, &mut ram);
        bridge.set_direction(3, 2);
    }

    assert_eq!(debris.x(3), 0x12);
    assert_eq!(debris.y(3), 0x34);
    assert_eq!(debris.direction(3), 2);
    assert_eq!(ram[DOOR_DEBRIS_X + 1], 0);
    assert_eq!(ram[DOOR_DEBRIS_X + 3], 0x12);
    assert_eq!(ram[DOOR_DEBRIS_Y + 3], 0x34);
    assert_eq!(ram[DOOR_DEBRIS_DIRECTION + 3], 2);
}
