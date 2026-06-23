use super::*;

#[test]
fn tile_detection_state_owns_tile_probe_behavior() {
    let mut detection = TileDetectionState::default();

    detection.set_y(0x1200);
    detection.set_y_high(0x34);
    detection.set_x(0x5678);
    detection.set_location_calc_mask(0x00ff);
    detection.set_interacting_tile(0xab00);
    detection.set_interacting_tile_low(0xcd);
    detection.set_interaction_scratch_y_bytes(0x11, 0x22);
    detection.set_tile_probe_anchor(0x3344);
    detection.set_diagonal_tile(0x0001);
    detection.or_diagonal_tile(0x0002);
    detection.set_stair_tile(0x04);
    detection.or_stair_tile(0x08);
    detection.or_block_flags(0x0100);
    detection.or_deepwater(0x0002);
    detection.or_normal_tiles(0x0004);
    detection.or_misc_tiles(0x0008);
    detection.or_thick_grass(0x0010);
    detection.or_vertical_ledge(0x20);
    detection.or_horizontal_ledge(0x40);
    detection.or_chest(0x0080);
    detection.set_key_lock_gravestones(0x55);
    detection.or_spike_cactus_tiles(0xaa);
    detection.set_tile_type(0x1234);
    detection.or_spike_floor_and_triggers(0x01);
    detection.or_dashable_tiles(0x02);
    detection.set_staircase_cache(0x03);
    detection.or_slope_collision_bits(0x0004);
    detection.or_collision_bits(0x0008);
    detection.or_inroom_staircase(0x0010);
    detection.set_liftable_tile_index(0x12);
    detection.set_tile_collision_bits_primary(0x34);
    detection.set_liftable_action_index_primary(0x56);
    detection.set_liftable_action_index_secondary(0x78);

    assert_eq!(detection.y(), 0x3400);
    assert_eq!(detection.x(), 0x5678);
    assert_eq!(detection.location_calc_mask(), 0x00ff);
    assert_eq!(detection.interacting_tile(), 0xabcd);
    assert_eq!(detection.interaction_scratch_y(), 0x2211);
    assert_eq!(detection.interaction_scratch_x(), 0x3344);
    assert_eq!(detection.diagonal_tile(), 0x0003);
    assert_eq!(detection.stair_tile(), 0x0c);
    assert_eq!(detection.block_flags(), 0x0100);
    assert_eq!(detection.deepwater(), 0x0002);
    assert_eq!(detection.normal_tiles(), 0x0004);
    assert_eq!(detection.misc_tiles(), 0x0008);
    assert_eq!(detection.thick_grass(), 0x0010);
    assert_eq!(detection.ledge_mask(), 0x60);
    assert_eq!(detection.chest(), 0x0080);
    assert_eq!(detection.key_lock_gravestones_low(), 0x55);
    assert_eq!(detection.spike_cactus_tiles(), 0xaa);
    assert_eq!(detection.tile_type(), 0x1234);
    assert_eq!(detection.spike_floor_and_triggers(), 0x01);
    assert_eq!(detection.dashable_tiles(), 0x02);
    assert_eq!(detection.staircase_cache(), 0x03);
    assert_eq!(detection.slope_collision_bits(), 0x0004);
    assert_eq!(detection.collision_bits(), 0x0008);
    assert_eq!(detection.inroom_staircase(), 0x0010);
    assert_eq!(detection.liftable_tile_index(), 0x12);
    assert_eq!(detection.tile_collision_bits_primary(), 0x34);
    assert_eq!(detection.liftable_action_index_primary(), 0x56);
}

#[test]
fn follower_link_state_owns_tagalong_link_semantics() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, LINK_X_COORD, 0x1234);
    write_le_u16(&mut ram, LINK_Y_COORD, 0x5678);
    ram[LINK_Z_COORD] = 0xf1;
    ram[LINK_X_VELOCITY] = 0x04;
    ram[LINK_Y_VELOCITY] = 0;
    ram[LINK_IS_ON_LOWER_LEVEL] = 2;
    ram[LINK_FACING] = 8;
    ram[LINK_SPEED_SETTING] = 4;
    ram[LINK_HANDLER_STATE] = 17;
    ram[LINK_AUXILIARY_STATE] = 0;
    ram[LINK_IS_RUNNING] = 0;

    let link = FollowerLinkState::load_from_ram(&ram);

    assert_eq!(link.x(), 0x1234);
    assert_eq!(link.y(), 0x5678);
    assert_eq!(link.z_for_follow(), 0);
    assert_eq!(link.z_for_oam(), 0xf1);
    assert!(link.is_moving());
    assert_eq!(link.floor(), 2);
    assert_eq!(link.floor_layer_bits(), 0x0c);
    assert_eq!(link.oam_priority_for_floor(), 0x30);
    assert_eq!(link.facing_layer_bits(), 4);
    assert_eq!(link.speed_setting(), 4);
    assert!(link.is_ground_swim_or_dash_start());
    assert!(link.can_open_follower_message());
    assert!(link.can_drop_follower());

    link.write_to_ram(&mut ram);
    assert_eq!(read_le_u16(&ram, LINK_X_COORD), 0x1234);
    assert_eq!(read_le_u16(&ram, LINK_Y_COORD), 0x5678);
    assert_eq!(ram[LINK_SPEED_SETTING], 4);

    ram[LINK_Z_COORD] = 0xf1;
    ram[LINK_Z_COORD + 1] = 0xff;
    assert_eq!(FollowerLinkState::load_from_ram(&ram).z_for_oam(), 0);
}

#[test]
fn native_follower_link_bridge_refreshes_projection_before_dual_writes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, LINK_X_COORD, 0x1234);
    write_le_u16(&mut ram, LINK_Y_COORD, 0x5678);
    ram[LINK_SPEED_SETTING] = 4;
    ram[LINK_HANDLER_STATE] = 17;
    ram[LINK_IS_RUNNING] = 1;
    let mut link = FollowerLinkState::default();

    {
        let mut bridge = NativeFollowerLinkBridgeMut::new(&mut link, &mut ram);
        bridge.set_speed_setting(12);
        bridge.set_ground_state();
        bridge.clear_running();
        bridge.immobilize();
        bridge.enable_cutscene_immunity();
    }

    assert_eq!(link.x(), 0x1234);
    assert_eq!(link.y(), 0x5678);
    assert_eq!(link.speed_setting(), 12);
    assert_eq!(read_le_u16(&ram, LINK_X_COORD), 0x1234);
    assert_eq!(read_le_u16(&ram, LINK_Y_COORD), 0x5678);
    assert_eq!(ram[LINK_SPEED_SETTING], 12);
    assert_eq!(ram[LINK_HANDLER_STATE], 0);
    assert_eq!(ram[LINK_IS_RUNNING], 0);
    assert_eq!(ram[FLAG_IS_LINK_IMMOBILIZED], 1);
    assert_eq!(ram[LINK_DISABLE_SPRITE_DAMAGE], 1);
}

#[test]
fn native_follower_link_bridge_disables_both_oam_offsets() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[PLAYER_OAM_X_OFFSET] = 0x11;
    ram[PLAYER_OAM_Y_OFFSET] = 0x22;
    let mut link = FollowerLinkState::default();

    {
        let mut bridge = NativeFollowerLinkBridgeMut::new(&mut link, &mut ram);
        bridge.disable_oam_offsets();
    }

    assert_eq!(link.oam_x_offset(), 0x80);
    assert_eq!(link.oam_y_offset(), 0x80);
    assert_eq!(ram[PLAYER_OAM_X_OFFSET], 0x80);
    assert_eq!(ram[PLAYER_OAM_Y_OFFSET], 0x80);
}

#[test]
fn special_exit_position_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, LINK_X_COORD_SPEXIT, 0x0900);
    write_le_u16(&mut ram, LINK_Y_COORD_SPEXIT, 0x0500);

    let mut position = SpecialExitPositionState::load_from_ram(&ram);
    assert_eq!(position.x(), 0x0900);
    assert_eq!(position.y(), 0x0500);
    assert_eq!(position.map_zoom_x_offset(), 0x0010);
    assert_eq!(position.map_zoom_y(), 0x0008);

    write_le_u16(&mut ram, LINK_X_COORD, 0x0300);
    write_le_u16(&mut ram, LINK_Y_COORD, 0x0400);
    position.set_x(0x0500);
    position.set_y(0x0600);
    position.offset_position(0x0010, 0x0020);
    assert_eq!(position.x(), 0x0510);
    assert_eq!(position.y(), 0x0620);
    position.store_from_player_ram(&ram);
    assert_eq!(position.x(), 0x0300);
    assert_eq!(position.y(), 0x0400);
    position.set_position(0x0700, 0x0800);
    position.restore_player_position_to_ram(&mut ram);
    assert_eq!(read_le_u16(&ram, LINK_X_COORD), 0x0700);
    assert_eq!(read_le_u16(&ram, LINK_Y_COORD), 0x0800);

    position = SpecialExitPositionState::load_from_ram(&[0]);
    let mut projected = vec![0; WRAM_SIZE];
    position.write_to_ram(&mut projected);
    assert_eq!(read_le_u16(&projected, LINK_X_COORD_SPEXIT), 0);
    assert_eq!(read_le_u16(&projected, LINK_Y_COORD_SPEXIT), 0);
}

#[test]
fn native_special_exit_position_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, LINK_X_COORD_SPEXIT, 0x0100);
    write_le_u16(&mut ram, LINK_Y_COORD_SPEXIT, 0x0200);
    write_le_u16(&mut ram, LINK_X_COORD, 0x0300);
    write_le_u16(&mut ram, LINK_Y_COORD, 0x0400);

    let mut position = SpecialExitPositionState::load_from_ram(&ram);
    {
        let mut bridge = NativeSpecialExitPositionBridgeMut::new(&mut position, &mut ram);
        bridge.set_x(0x0500);
        bridge.set_y(0x0600);
        bridge.offset_position(0x0010, 0x0020);
        bridge.store_from_player();
        bridge.set_position(0x0700, 0x0800);
        bridge.restore_player_position();
    }

    assert_eq!(position.x(), 0x0700);
    assert_eq!(position.y(), 0x0800);
    assert_eq!(read_le_u16(&ram, LINK_X_COORD_SPEXIT), 0x0700);
    assert_eq!(read_le_u16(&ram, LINK_Y_COORD_SPEXIT), 0x0800);
    assert_eq!(read_le_u16(&ram, LINK_X_COORD), 0x0700);
    assert_eq!(read_le_u16(&ram, LINK_Y_COORD), 0x0800);
}

#[test]
fn swim_acceleration_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SWIM_ACCELERATION_MODE, 1);
    write_le_u16(&mut ram, SWIM_ACCELERATION_MODE + 2, 2);
    write_le_u16(&mut ram, SWIM_SPEED_ACTIVE_FLAG, 3);
    write_le_u16(&mut ram, SWIM_SPEED_ACTIVE_FLAG + 2, 4);
    write_le_u16(&mut ram, SWIM_MAX_SPEED, 0x0180);
    write_le_u16(&mut ram, SWIM_MAX_SPEED + 2, 0x0240);
    write_le_u16(&mut ram, SWIM_ACCELERATION_DIRECTION, 5);
    write_le_u16(&mut ram, SWIM_ACCELERATION_DIRECTION + 2, 6);
    write_le_u16(&mut ram, SWIM_ACCELERATION, 7);
    write_le_u16(&mut ram, SWIM_ACCELERATION + 2, 8);

    let mut swim = SwimAccelerationState::load_from_ram(&ram);
    assert_eq!(swim.mode(0), 1);
    assert_eq!(swim.mode(2), 2);
    assert_eq!(swim.mode(1), 0);
    assert_eq!(swim.mode_low(1), 2);
    assert_eq!(swim.speed_active_flag(0), 3);
    assert_eq!(swim.speed_active_flag(2), 4);
    assert_eq!(swim.max_speed(0), 0x0180);
    assert_eq!(swim.max_speed(2), 0x0240);
    assert_eq!(swim.acceleration_direction(0), 5);
    assert_eq!(swim.acceleration_direction(2), 6);
    assert_eq!(swim.acceleration(0), 7);
    assert_eq!(swim.acceleration(2), 8);
    assert!(swim.has_any_acceleration());
    assert!(swim.set_mode(0, 0x10));
    assert!(swim.set_mode(2, 0x20));
    assert!(!swim.set_mode(1, 0x30));
    swim.clear_mode_low_axis();
    assert!(swim.set_speed_active_flag(0, 0x40));
    swim.set_max_speed_both_axes(0x0180);
    assert!(swim.set_max_speed(2, 0x0240));
    assert!(swim.set_acceleration_direction(2, 0x50));
    assert!(swim.set_acceleration(0, 0x60));
    assert!(swim.set_acceleration(2, 0x70));
    assert!(swim.clear_axis_motion(0));
    assert!(!swim.clear_axis_motion(1));
    assert_eq!(swim.mode(0), 0);
    assert_eq!(swim.mode(2), 0x20);
    assert_eq!(swim.speed_active_flag(0), 0);
    assert_eq!(swim.max_speed(0), 0);
    assert_eq!(swim.max_speed(2), 0x0240);
    assert_eq!(swim.acceleration_direction(2), 0x50);
    assert_eq!(swim.acceleration(0), 0);
    assert_eq!(swim.acceleration(2), 0x70);

    let mut projected = vec![0; WRAM_SIZE];
    swim.write_to_ram(&mut projected);
    assert_eq!(SwimAccelerationState::load_from_ram(&projected), swim);
}

#[test]
fn native_swim_acceleration_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SWIM_ACCELERATION_MODE, 0xffff);
    write_le_u16(&mut ram, SWIM_ACCELERATION_MODE + 2, 0xffff);
    write_le_u16(&mut ram, SWIM_SPEED_ACTIVE_FLAG, 0xffff);
    write_le_u16(&mut ram, SWIM_MAX_SPEED, 0xffff);
    write_le_u16(&mut ram, SWIM_ACCELERATION_DIRECTION + 2, 0xffff);
    write_le_u16(&mut ram, SWIM_ACCELERATION, 0xffff);

    let mut swim = SwimAccelerationState::load_from_ram(&ram);
    {
        let mut bridge = NativeSwimAccelerationBridgeMut::new(&mut swim, &mut ram);
        bridge.set_mode(0, 1);
        bridge.set_mode(2, 2);
        bridge.clear_mode_low_axis();
        bridge.set_speed_active_flag(0, 3);
        bridge.set_max_speed_both_axes(0x0180);
        bridge.set_max_speed(2, 0x0240);
        bridge.set_acceleration_direction(2, 4);
        bridge.set_acceleration(0, 5);
        bridge.set_acceleration(2, 6);
        bridge.clear_axis_motion(0);
        bridge.set_mode(1, 9);
    }

    assert_eq!(swim.mode(0), 0);
    assert_eq!(swim.mode(2), 2);
    assert_eq!(swim.speed_active_flag(0), 0);
    assert_eq!(swim.max_speed(0), 0);
    assert_eq!(swim.max_speed(2), 0x0240);
    assert_eq!(swim.acceleration_direction(2), 4);
    assert_eq!(swim.acceleration(0), 0);
    assert_eq!(swim.acceleration(2), 6);
    assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION_MODE), 0);
    assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION_MODE + 2), 2);
    assert_eq!(read_le_u16(&ram, SWIM_SPEED_ACTIVE_FLAG), 0);
    assert_eq!(read_le_u16(&ram, SWIM_MAX_SPEED), 0);
    assert_eq!(read_le_u16(&ram, SWIM_MAX_SPEED + 2), 0x0240);
    assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION_DIRECTION + 2), 4);
    assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION), 0);
    assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION + 2), 6);
}

#[test]
fn pushed_block_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[PUSHEDBLOCKS_X_LO] = 0x34;
    ram[PUSHEDBLOCKS_X_HI] = 0x12;
    ram[PUSHEDBLOCKS_Y_LO + 2] = 0x78;
    ram[PUSHEDBLOCKS_Y_HI + 2] = 0x56;
    ram[PUSHEDBLOCKS_SUBPIXEL + 2] = 0x9a;
    ram[PUSHEDBLOCKS_TARGET + 2] = 0x0b;
    ram[PUSHEDBLOCK_FACING_PLAYER + 2] = 4;
    ram[PUSHED_BLOCK_MODE] = 3;
    ram[PUSHED_BLOCK_ANIMATION_TIMER] = 7;
    ram[PUSH_BLOCK_DIRECTION] = 6;

    let mut pushed = PushedBlockState::load_from_ram(&ram);
    assert_eq!(pushed.x(0), 0x1234);
    assert_eq!(pushed.y(1), 0x5678);
    assert_eq!(pushed.y_fixed24(1), 0x56789a);
    assert_eq!(pushed.target_low(1), 0x0b);
    assert_eq!(pushed.facing_player(1), 4);
    assert_eq!(pushed.animation_mode(), 3);
    assert_eq!(pushed.animation_timer(), 7);
    assert_eq!(pushed.push_direction(), 6);
    assert_eq!(pushed.push_direction_index(), 3);
    assert_eq!(pushed.x(2), 0);
    pushed.init_slot(0, 0x2345, 0x6789);
    assert!(pushed.set_facing_player(1, 5));
    assert!(pushed.set_target_low(1, 0x0c));
    pushed.set_push_direction(4);
    pushed.set_animation_mode(2);
    pushed.reset_animation_timer();
    assert_eq!(pushed.decrement_animation_timer(), 8);
    assert_eq!(pushed.advance_animation_mode(), 3);
    assert!(pushed.set_x_fixed24(1, 0x00abcd));
    assert!(pushed.set_y_fixed24(1, 0x001234));
    assert!(!pushed.set_target_low(4, 0xff));
    assert_eq!(pushed.x(0), 0x2345);
    assert_eq!(pushed.y(0), 0x6789);
    assert_eq!(pushed.x_fixed24(1), 0x00ab34);
    assert_eq!(pushed.y_fixed24(1), 0x001234);
    assert_eq!(pushed.target_low(1), 0x0c);
    assert_eq!(pushed.facing_player(1), 5);
    assert_eq!(pushed.push_direction(), 4);
    assert_eq!(pushed.animation_mode(), 3);
    assert_eq!(pushed.animation_timer(), 9);

    let mut projected = vec![0; WRAM_SIZE];
    pushed.write_to_ram(&mut projected);
    assert_eq!(PushedBlockState::load_from_ram(&projected), pushed);
}

#[test]
fn native_pushed_block_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[PUSHEDBLOCKS_X_LO + 1] = 0xff;
    ram[PUSHEDBLOCKS_X_HI + 1] = 0xff;
    ram[PUSHEDBLOCKS_Y_LO + 1] = 0xff;
    ram[PUSHEDBLOCKS_Y_HI + 1] = 0xff;
    ram[PUSHEDBLOCKS_TARGET + 1] = 0xff;
    ram[PUSHEDBLOCKS_SUBPIXEL + 1] = 0xff;

    let mut pushed = PushedBlockState::load_from_ram(&ram);
    {
        let mut bridge = NativePushedBlockBridgeMut::new(&mut pushed, &mut ram);
        bridge.init_slot(0, 0x1234, 0x5678);
        bridge.set_facing_player(1, 4);
        bridge.set_target_low(1, 0x0b);
        bridge.set_push_direction(6);
        bridge.set_animation_mode(2);
        bridge.reset_animation_timer();
        assert_eq!(bridge.decrement_animation_timer(), 8);
        assert_eq!(bridge.advance_animation_mode(), 3);
        bridge.set_x_fixed24(1, 0x00abcdu32);
        bridge.set_y_fixed24(1, 0x001234u32);
        bridge.set_target_low(4, 0xff);
    }

    assert_eq!(pushed.x(0), 0x1234);
    assert_eq!(pushed.y(0), 0x5678);
    assert_eq!(pushed.x_fixed24(1), 0x00ab34);
    assert_eq!(pushed.y_fixed24(1), 0x001234);
    assert_eq!(pushed.target_low(1), 0x0b);
    assert_eq!(pushed.facing_player(1), 4);
    assert_eq!(pushed.push_direction_index(), 3);
    assert_eq!(pushed.animation_mode(), 3);
    assert_eq!(pushed.animation_timer(), 9);
    assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_X_LO), 0x0034);
    assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_X_HI), 0x0012);
    assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_Y_LO), 0x0078);
    assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_Y_HI), 0x0056);
    assert_eq!(ram[PUSHEDBLOCKS_X_LO + 2], 0xab);
    assert_eq!(ram[PUSHEDBLOCKS_X_HI + 2], 0);
    assert_eq!(ram[PUSHEDBLOCKS_SUBPIXEL + 2], 0x34);
    assert_eq!(ram[PUSH_BLOCK_DIRECTION], 6);
    assert_eq!(ram[PUSHED_BLOCK_MODE], 3);
    assert_eq!(ram[PUSHED_BLOCK_ANIMATION_TIMER], 9);
}

#[test]
fn player_resources_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_MAGIC_POWER] = 0x40;
    ram[LINK_MAGIC_CONSUMPTION] = 2;
    ram[LINK_ITEM_BOMBS] = 7;
    ram[LINK_ITEM_BOTTLE_INDEX] = 3;
    write_le_u16(&mut ram, LINK_RUPEES_GOAL, 0x0123);
    write_le_u16(&mut ram, LINK_RUPEES_ACTUAL, 0x0045);
    write_le_u16(&mut ram, LINK_COMPASS, 0x0008);
    write_le_u16(&mut ram, LINK_BIGKEY, 0x0010);
    write_le_u16(&mut ram, LINK_DUNGEON_MAP, 0x0020);
    ram[LINK_RUPEES_IN_POND] = 30;
    ram[LINK_HEART_PIECES] = 2;
    ram[LINK_HEALTH_CAPACITY] = 0x38;
    ram[LINK_CURRENT_HEALTH] = 0x28;
    ram[LINK_NUM_KEYS] = 4;
    ram[LINK_BOMB_UPGRADES] = 1;
    ram[LINK_ARROW_UPGRADES] = 2;
    ram[LINK_HEARTS_FILLER] = 5;
    ram[LINK_MAGIC_FILLER] = 6;
    ram[LINK_WHICH_PENDANTS] = 7;
    ram[LINK_BOMB_FILLER] = 8;
    ram[LINK_ARROW_REFILL_COUNTER] = 9;
    ram[LINK_NUM_ARROWS] = 10;
    ram[LINK_ABILITY_FLAGS] = 0x11;
    ram[LINK_HAS_CRYSTALS] = 0x22;
    ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] = 0x33;

    let resources = PlayerResourcesState::load_from_ram(&ram);
    // magic_power is now owned by FollowerLinkState, not PlayerResourcesState.
    assert_eq!(resources.magic_consumption_level(), 2);
    assert_eq!(resources.bombs(), 7);
    assert_eq!(resources.equipped_bottle_index(), 3);
    assert_eq!(resources.rupees_goal(), 0x0123);
    assert_eq!(resources.rupees_actual(), 0x0045);
    assert!(resources.has_compass_mask(0x0008));
    assert!(resources.has_big_key_mask(0x0010));
    assert!(resources.has_dungeon_map_mask(0x0020));
    assert_eq!(resources.rupees_in_pond(), 30);
    assert_eq!(resources.heart_pieces(), 2);
    assert_eq!(resources.health_capacity(), 0x38);
    assert_eq!(resources.current_health(), 0x28);
    assert_eq!(resources.keys(), 4);
    assert_eq!(resources.bomb_upgrade_level(), 1);
    assert_eq!(resources.arrow_upgrade_level(), 2);
    assert_eq!(resources.heart_filler(), 5);
    assert_eq!(resources.magic_filler(), 6);
    assert_eq!(resources.pendant_flags(), 7);
    assert_eq!(resources.bomb_filler(), 8);
    assert_eq!(resources.arrow_filler(), 9);
    assert_eq!(resources.arrows(), 10);
    assert_eq!(resources.ability_flags(), 0x11);
    assert_eq!(resources.crystal_flags(), 0x22);
    assert_eq!(resources.low_health_beep_timer(), 0x33);

    let mut projected = vec![0; WRAM_SIZE];
    resources.write_to_ram(&mut projected);
    assert_eq!(PlayerResourcesState::load_from_ram(&projected), resources);
}

#[test]
fn native_player_resources_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_ITEM_BOMBS] = 1;
    ram[LINK_NUM_ARROWS] = 2;
    ram[LINK_HEARTS_FILLER] = 0xff;
    ram[LINK_MAGIC_FILLER] = 0;
    write_le_u16(&mut ram, LINK_RUPEES_GOAL, 10);
    ram[LINK_NUM_KEYS] = 0xff;

    let mut resources = PlayerResourcesState::load_from_ram(&ram);
    {
        let mut bridge = NativePlayerResourcesBridgeMut::new(&mut resources, &mut ram);
        bridge.set_bombs(4);
        bridge.decrement_bombs();
        bridge.increment_arrows_by(5);
        bridge.increment_heart_filler_word_by(2);
        bridge.add_rupees_goal(90);
        bridge.subtract_rupees_goal(25);
        bridge.increment_keys();
        bridge.add_ability_flags(0x04);
        bridge.add_crystal_flags(0x20);
        bridge.set_pendant_flags(0x07);
    }

    assert_eq!(resources.bombs(), 3);
    assert_eq!(resources.arrows(), 7);
    assert_eq!(resources.heart_filler(), 1);
    assert_eq!(resources.magic_filler(), 1);
    assert_eq!(resources.rupees_goal(), 75);
    assert_eq!(resources.keys(), 0);
    assert_eq!(resources.ability_flags(), 0x04);
    assert_eq!(resources.crystal_flags(), 0x20);
    assert_eq!(resources.pendant_flags(), 0x07);
    assert_eq!(ram[LINK_ITEM_BOMBS], 3);
    assert_eq!(ram[LINK_NUM_ARROWS], 7);
    assert_eq!(read_le_u16(&ram, LINK_HEARTS_FILLER), 0x0101);
    assert_eq!(read_le_u16(&ram, LINK_RUPEES_GOAL), 75);
    assert_eq!(ram[LINK_NUM_KEYS], 0);
    assert_eq!(ram[LINK_ABILITY_FLAGS], 0x04);
    assert_eq!(ram[LINK_HAS_CRYSTALS], 0x20);
    assert_eq!(ram[LINK_WHICH_PENDANTS], 0x07);
}

#[test]
fn native_player_resources_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0xff; WRAM_SIZE];
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[LINK_ITEM_BOMBS] = 1;
    native_ram[LINK_NUM_ARROWS] = 2;
    native_ram[LINK_HEARTS_FILLER] = 0xff;
    native_ram[LINK_MAGIC_FILLER] = 0;
    write_le_u16(&mut native_ram, LINK_RUPEES_GOAL, 10);
    native_ram[LINK_NUM_KEYS] = 0xff;
    let mut resources = PlayerResourcesState::load_from_ram(&native_ram);

    {
        let mut bridge = NativePlayerResourcesBridgeMut::new(&mut resources, &mut ram);
        bridge.decrement_bombs();
        bridge.increment_arrows_by(5);
        bridge.increment_heart_filler_word_by(2);
        bridge.add_rupees_goal(90);
        bridge.increment_keys();
        bridge.add_ability_flags(0x04);
    }

    assert_eq!(resources.bombs(), 0);
    assert_eq!(resources.arrows(), 7);
    assert_eq!(resources.heart_filler(), 1);
    assert_eq!(resources.magic_filler(), 1);
    assert_eq!(resources.rupees_goal(), 100);
    assert_eq!(resources.keys(), 0);
    assert_eq!(resources.ability_flags(), 0x04);
    assert_eq!(ram[LINK_ITEM_BOMBS], 0);
    assert_eq!(ram[LINK_NUM_ARROWS], 7);
    assert_eq!(read_le_u16(&ram, LINK_HEARTS_FILLER), 0x0101);
    assert_eq!(read_le_u16(&ram, LINK_RUPEES_GOAL), 100);
    assert_eq!(ram[LINK_NUM_KEYS], 0);
    assert_eq!(ram[LINK_ABILITY_FLAGS], 0x04);
}

#[test]
fn native_bg1_movement_accumulator_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, BG1_MOVE_CALC_BUFFER, 0x1203);

    let mut accumulator = Bg1MovementAccumulatorState::load_from_ram(&ram);
    {
        let mut bridge = NativeBg1MovementAccumulatorBridgeMut::new(&mut accumulator, &mut ram);
        bridge.set_y_subpixel(0x44);
        assert_eq!(bridge.advance_x_subpixel(0xf1), 0x0103);
    }

    assert_eq!(accumulator.x_subpixel(), 0x03);
    assert_eq!(accumulator.y_subpixel(), 0x44);
    assert_eq!(read_le_u16(&ram, BG1_MOVE_CALC_BUFFER), 0x0344);
}
