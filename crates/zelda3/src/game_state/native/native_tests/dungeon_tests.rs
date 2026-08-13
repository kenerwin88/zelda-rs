use super::*;
use crate::game_state::native::dungeon::{
    DungeonEnvironmentState, NativeDungeonEnvironmentBridgeMut,
};

#[test]
fn native_dungeon_environment_bridge_ignores_write_through_water_counter_in_coherence_check() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TURN_ON_OFF_WATER_CTR] = 1;

    let mut environment = DungeonEnvironmentState::default();

    {
        let mut bridge = NativeDungeonEnvironmentBridgeMut::new(&mut environment, &mut ram);
        bridge.set_water_hdma_y_radius(0x30);
    }

    assert_eq!(environment.water_transition_counter(), 0);
    assert_eq!(environment.water_hdma_y_radius(), 0x30);
    assert_eq!(ram[TURN_ON_OFF_WATER_CTR], 1);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x30);
}

#[test]
fn native_dungeon_door_bridge_loads_room_tilemap_addresses_from_door_info() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut doors = DungeonDoorState::default();
    let door_info = [0x34, 0x12, 0x78, 0x56, 0xff, 0xff, 0xaa, 0xaa];

    {
        let mut bridge = NativeDungeonDoorBridgeMut::new(&mut doors, &mut ram);
        bridge.load_room_door_tilemap_addresses_from_info(&door_info);
    }

    assert_eq!(doors.door_tilemap_address(0), 0x1234);
    assert_eq!(doors.door_tilemap_address(1), 0x5678);
    assert_eq!(doors.door_tilemap_address(2), 0);
    assert_eq!(read_le_u16(&ram, DUNG_DOOR_TILEMAP_ADDRESS), 0x1234);
    assert_eq!(read_le_u16(&ram, DUNG_DOOR_TILEMAP_ADDRESS + 2), 0x5678);
    assert_eq!(read_le_u16(&ram, DUNG_DOOR_TILEMAP_ADDRESS + 4), 0);
}

#[test]
fn native_dungeon_room_door_setup_bridge_loads_adjacent_doors_from_door_info() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut setup = DungeonRoomDoorSetupState::default();
    let door_info = [0x10, 0x00, 0x01, 0x40, 0x00, 0x02, 0xff, 0xff, 0xaa, 0xaa];

    {
        let mut bridge = NativeDungeonRoomDoorSetupBridgeMut::new(&mut setup, &mut ram);
        bridge.load_adjacent_doors_from_room_info(&door_info);
    }

    assert_eq!(setup.adjacent_door(0), 0x0010);
    assert_eq!(setup.adjacent_door(1), 0x4001);
    assert_eq!(setup.adjacent_door(2), 0x0200);
    assert_eq!(setup.adjacent_door(3), 0xffff);
    assert_eq!(setup.adjacent_door_flags(), 0xc000);
    assert_eq!(read_le_u16(&ram, ADJACENT_DOORS), 0x0010);
    assert_eq!(read_le_u16(&ram, ADJACENT_DOORS + 2), 0x4001);
    assert_eq!(read_le_u16(&ram, ADJACENT_DOORS + 4), 0x0200);
    assert_eq!(read_le_u16(&ram, ADJACENT_DOORS + 6), 0xffff);
    assert_eq!(read_le_u16(&ram, ADJACENT_DOORS_FLAGS), 0xc000);
}

#[test]
fn native_dungeon_room_door_setup_projects_adjacent_door_sentinel_slot() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut setup = DungeonRoomDoorSetupState::default();
    let door_info = [
        0x82, 0x1c, 0x62, 0x38, 0x61, 0x00, 0x63, 0x00, 0x20, 0x00, 0x81, 0x00, 0x60, 0x69, 0xce,
        0x0c, 0xff, 0xff,
    ];

    {
        let mut bridge = NativeDungeonRoomDoorSetupBridgeMut::new(&mut setup, &mut ram);
        bridge.load_adjacent_doors_from_room_info(&door_info);
    }

    assert_eq!(setup.adjacent_door(7), 0x0cce);
    assert_eq!(read_le_u16(&ram, ADJACENT_DOORS + 16), 0xffff);
}

#[test]
fn native_dungeon_torch_bridge_clears_room_load_indices() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNG_INDEX_OF_TORCHES_START, 6);
    write_le_u16(&mut ram, DUNG_INDEX_OF_TORCHES, 6);
    let mut torch = DungeonTorchState::load_from_ram(&ram);

    write_le_u16(&mut ram, DUNG_INDEX_OF_TORCHES_START, 0);
    write_le_u16(&mut ram, DUNG_INDEX_OF_TORCHES, 0);

    {
        let mut bridge = NativeDungeonTorchBridgeMut::new(&mut torch, &mut ram);
        bridge.clear_torch_indices();
        bridge.clear_timers();
    }

    assert_eq!(torch.torches_start_index(), 0);
    assert_eq!(torch.torch_index(), 0);
    assert_eq!(read_le_u16(&ram, DUNG_INDEX_OF_TORCHES_START), 0);
    assert_eq!(read_le_u16(&ram, DUNG_INDEX_OF_TORCHES), 0);
}

#[test]
fn dungeon_secret_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUNGEON_SECRET_PENDING_KIND] = 0x82;
    ram[DUNGEON_SECRET_PENDING_KIND + 1] = 0x7f;
    ram[OVERWORLD_SECRET_SUBST_CTR] = 3;

    let mut secret = DungeonSecretState::load_from_ram(&ram);
    assert_eq!(secret.pending_kind(), 0x82);
    assert_eq!(secret.graphics_kind(), Some(2));
    assert!(secret.has_pending_kind());
    assert!(secret.is_available());
    assert_eq!(secret.overworld_subst_counter(), 3);

    secret.set_powder_pending_kind();
    secret.increment_overworld_subst_counter();
    secret.mark_graphics_kind();
    secret.write_to_ram(&mut ram);

    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND], 0x84);
    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND + 1], 0);
    assert_eq!(ram[OVERWORLD_SECRET_SUBST_CTR], 4);
}

#[test]
fn native_dungeon_secret_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUNGEON_SECRET_PENDING_KIND] = 0xff;
    ram[DUNGEON_SECRET_PENDING_KIND + 1] = 0x44;
    ram[OVERWORLD_SECRET_SUBST_CTR] = 0xff;

    let mut secret = DungeonSecretState::load_from_ram(&ram);
    {
        let mut bridge = NativeDungeonSecretBridgeMut::new(&mut secret, &mut ram);
        bridge.clear_pending_kind();
        bridge.set_pending_kind(2);
        bridge.or_pending_kind(4);
        bridge.mark_graphics_kind();
        bridge.increment_overworld_subst_counter();
        bridge.set_powder_pending_kind();
    }

    assert_eq!(secret.pending_kind(), 4);
    assert_eq!(secret.graphics_kind(), None);
    assert_eq!(secret.overworld_subst_counter(), 0);
    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND], 4);
    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND + 1], 0);
    assert_eq!(ram[OVERWORLD_SECRET_SUBST_CTR], 0);
}

#[test]
fn native_dungeon_secret_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUNGEON_SECRET_PENDING_KIND] = 0xff;
    ram[OVERWORLD_SECRET_SUBST_CTR] = 0xff;
    let mut secret = DungeonSecretState::default();
    secret.set_pending_kind(2);

    {
        let mut bridge = NativeDungeonSecretBridgeMut::new(&mut secret, &mut ram);
        bridge.or_pending_kind(4);
    }

    assert_eq!(secret.pending_kind(), 6);
    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND], 6);
    assert_eq!(ram[OVERWORLD_SECRET_SUBST_CTR], 0);
}

#[test]
fn native_game_state_bulk_projection_preserves_dungeon_secret_scratch() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUNGEON_SECRET_PENDING_KIND] = 0x0b;
    ram[DUNGEON_SECRET_PENDING_KIND + 1] = 0xaa;
    ram[OVERWORLD_SECRET_SUBST_CTR] = 0x05;

    let mut state = GameState::load_from_ram(&ram);
    state.dungeon_secret.clear_pending_kind();
    state.dungeon_secret.increment_overworld_subst_counter();
    state.write_to_ram(&mut ram);

    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND], 0x0b);
    assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND + 1], 0xaa);
    assert_eq!(ram[OVERWORLD_SECRET_SUBST_CTR], 0x05);
}

#[test]
fn save_load_transfer_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SAVE_LOAD_SOURCE_OFFSET, 0x1234);

    let mut transfer = SaveLoadTransferState::load_from_ram(&ram);
    assert_eq!(transfer.source_offset(), 0x1234);
    assert_eq!(transfer.source_offset_usize(), 0x1234);

    transfer.set_source_offset(0x4567);
    transfer.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, SAVE_LOAD_SOURCE_OFFSET), 0x4567);
}

#[test]
fn native_save_load_transfer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SAVE_LOAD_SOURCE_OFFSET, 0x1234);

    let mut transfer = SaveLoadTransferState::load_from_ram(&ram);
    {
        let mut bridge = NativeSaveLoadTransferBridgeMut::new(&mut transfer, &mut ram);
        bridge.set_source_offset(0x4567);
    }

    assert_eq!(transfer.source_offset(), 0x4567);
    assert_eq!(read_le_u16(&ram, SAVE_LOAD_SOURCE_OFFSET), 0x4567);
}

#[test]
fn native_save_load_transfer_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SAVE_LOAD_SOURCE_OFFSET, 0x1234);
    let mut transfer = SaveLoadTransferState::default();
    transfer.set_source_offset(0x2000);

    {
        let mut bridge = NativeSaveLoadTransferBridgeMut::new(&mut transfer, &mut ram);
        bridge.set_source_offset(0x2100);
    }

    assert_eq!(transfer.source_offset(), 0x2100);
    assert_eq!(read_le_u16(&ram, SAVE_LOAD_SOURCE_OFFSET), 0x2100);
}

#[test]
fn dungeon_map_display_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, 0x0010);
    write_le_u16(&mut ram, DUNGEON_MAP_SCROLL_INPUT, 0x0008);
    write_le_u16(&mut ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0044);
    write_le_u16(&mut ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x0055);
    write_le_u16(&mut ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y, 0xabcd);
    ram[DUNGMAP_INIT_STATE] = 2;
    write_le_u16(&mut ram, DUNGMAP_CUR_FLOOR, 0x1234);
    ram[DUNGMAP_FLOOR_SCROLL_STEP] = 3;
    write_le_u16(&mut ram, DUNGMAP_IDX, 0x0006);
    write_le_u16(&mut ram, DUNGMAP_SCROLL_TARGET_Y, 0x0080);
    write_le_u16(&mut ram, DUNGMAP_PLAYER_MARKER_X, 0x0090);
    write_le_u16(&mut ram, DUNGMAP_PLAYER_MARKER_Y, 0x00a0);

    let mut display = DungeonMapDisplayState::load_from_ram(&ram);
    assert_eq!(display.scroll_draw_offset(), 0x0010);
    assert_eq!(display.scroll_input_direction_index(), 1);
    assert_eq!(display.marker_x_offset(), 0x0044);
    assert_eq!(display.marker_y_offset(), 0x0055);
    assert_eq!(display.location_marker_base_y(), 0xcd);
    assert_eq!(display.dungmap_init_state(), 2);
    assert_eq!(display.current_floor(), 0x1234);
    assert_eq!(display.dungmap_cur_floor(), 0x1234);
    assert_eq!(display.dungmap_floor_scroll_step(), 3);
    assert_eq!(display.dungmap_idx(), 0x0006);
    assert_eq!(display.dungmap_scroll_target_y(), 0x0080);
    assert_eq!(display.dungmap_player_marker_x(), 0x0090);
    assert_eq!(display.dungmap_player_marker_y(), 0x00a0);

    display.clear_current_floor_high();
    display.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, DUNGMAP_CUR_FLOOR), 0x0034);
    assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_X_OFFSET), 0x0044);
}

#[test]
fn native_dungeon_map_display_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x01f0);
    write_le_u16(&mut ram, DUNGMAP_CUR_FLOOR, 0x1234);

    let mut display = DungeonMapDisplayState::load_from_ram(&ram);
    {
        let mut bridge = NativeDungeonMapDisplayBridgeMut::new(&mut display, &mut ram);
        bridge.reset_marker_offsets();
        bridge.shift_marker_x_left();
        bridge.set_location_marker_base_y(0x77);
        bridge.set_scroll_input(0x0008);
        bridge.increment_dungmap_init_state();
        bridge.set_dungmap_floor_scroll_step(4);
        bridge.set_dungmap_idx(0x0012);
        bridge.set_dungmap_scroll_target_y(0x0070);
        bridge.set_dungmap_player_marker_x(0x0088);
        bridge.set_dungmap_player_marker_y(0x0099);
        bridge.clear_current_floor_high();
    }

    assert_eq!(display.marker_x_offset(), 0x0030);
    assert_eq!(display.marker_y_offset(), 0x0040);
    assert_eq!(display.location_marker_base_y(), 0x77);
    assert_eq!(display.scroll_input_direction_index(), 1);
    assert_eq!(display.dungmap_init_state(), 1);
    assert_eq!(display.dungmap_floor_scroll_step(), 4);
    assert_eq!(display.dungmap_idx(), 0x0012);
    assert_eq!(display.dungmap_scroll_target_y(), 0x0070);
    assert_eq!(display.dungmap_player_marker_x_byte(), 0x88);
    assert_eq!(display.dungmap_player_marker_y(), 0x0099);
    assert_eq!(display.current_floor(), 0x0034);
    assert_eq!(read_le_u16(&ram, DUNGMAP_CUR_FLOOR), 0x0034);
    assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_X_OFFSET), 0x0030);
    assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_Y_OFFSET), 0x0040);
    assert_eq!(
        read_le_u16(&ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y),
        0x0077
    );
}

#[test]
fn native_dungeon_map_display_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0100);
    write_le_u16(&mut ram, DUNGMAP_CUR_FLOOR, 0x1234);
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0020);
    write_le_u16(&mut native_ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x0030);
    write_le_u16(&mut native_ram, DUNGMAP_CUR_FLOOR, 0x5678);
    let mut display = DungeonMapDisplayState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDungeonMapDisplayBridgeMut::new(&mut display, &mut ram);
        bridge.shift_marker_x_left();
        bridge.clear_current_floor_high();
    }

    assert_eq!(display.marker_x_offset(), 0x0010);
    assert_eq!(display.marker_y_offset(), 0x0030);
    assert_eq!(display.current_floor(), 0x0078);
    assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_X_OFFSET), 0x0010);
    assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_Y_OFFSET), 0x0030);
    assert_eq!(read_le_u16(&ram, DUNGMAP_CUR_FLOOR), 0x0078);
}

#[test]
fn dungeon_header_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS] = 0x12;
    ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS + 4] = 0x34;
    ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE] = 1;
    ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1] = 2;
    ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2] = 3;
    ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3] = 0;
    ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4] = 1;

    let mut header = DungeonHeaderState::load_from_ram(&ram);
    assert_eq!(header.travel_destination(0), 0x12);
    assert_eq!(header.travel_destination(4), 0x34);
    assert_eq!(header.hole_teleporter_plane(0), 1);
    assert_eq!(header.hole_teleporter_plane(4), 1);
    assert_eq!(header.staircase_plane(0), 2);
    assert_eq!(header.staircase_plane(3), 1);

    header.set_hole_teleporter_planes(0b11_10_01_00, 0b101);
    header.write_to_ram(&mut ram);

    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE], 0);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1], 1);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2], 2);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3], 3);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4], 1);
    assert_eq!(ram[DUNGEON_HEADER_STAIRCASE_PLANE], 1);
    assert_eq!(ram[DUNGEON_HEADER_STAIRCASE_PLANE + 3], 1);
    assert_eq!(ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS], 0x12);
    assert_eq!(ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS + 4], 0x34);
}

#[test]
fn native_dungeon_header_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE] = 3;

    let mut header = DungeonHeaderState::default();
    {
        let mut bridge = NativeDungeonHeaderBridgeMut::new(&mut header, &mut ram);
        bridge.set_hole_teleporter_planes(0b00_11_10_01, 2);
    }

    assert_eq!(header.hole_teleporter_plane(0), 1);
    assert_eq!(header.hole_teleporter_plane(1), 2);
    assert_eq!(header.hole_teleporter_plane(2), 3);
    assert_eq!(header.hole_teleporter_plane(3), 0);
    assert_eq!(header.hole_teleporter_plane(4), 2);
    assert_eq!(header.staircase_plane(0), 2);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE], 1);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1], 2);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2], 3);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3], 0);
    assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4], 2);
}

#[test]
fn dungeon_scratch_word_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_WORK_R16, 0x1201);
    write_le_u16(&mut ram, DUNGEON_WORK_R18, 0x3456);

    let mut scratch = DungeonScratchWordState::load_from_ram(&ram);
    assert_eq!(scratch.high(), 0x12);
    assert_eq!(scratch.word(), 0x1201);
    assert_eq!(scratch.minigame_previous_chest_choice(), 1);
    assert_eq!(scratch.primary_word(), 0x1201);
    assert_eq!(scratch.secondary_word(), 0x3456);
    assert_eq!(scratch.primary_low(), 1);
    assert_eq!(scratch.secondary_low(), 0x56);

    assert_eq!(scratch.decrement_high(), 0x11);
    assert_eq!(scratch.decrement_ganon_door_bounce_low(), 0);
    scratch.set_liftable_tile_probe_position(0x789a, 0xbcde);
    scratch.set_minigame_previous_chest_choice(0xef);
    scratch.set_primary_low(0x34);
    assert_eq!(scratch.decrement_primary_low(), 0x33);
    assert_eq!(scratch.increment_secondary_low(), 0xdf);
    scratch.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, ENDING_WORK_PRIMARY), 0x7833);
    assert_eq!(read_le_u16(&ram, ENDING_WORK_SECONDARY), 0xbcdf);
}

#[test]
fn native_dungeon_scratch_word_bridge_is_write_through_over_live_ram() {
    // R16/R18 (0xc8-0xcb) are shared SNES bytes that other code writes directly
    // (the 3bpp->4bpp gfx converter's DUNG_LINE_PTRS_ROW0 scratch, select-file R17).
    // The scratch-word native is NOT bulk-projected each frame; the bridge re-reads
    // RAM before mutating (every production access is a fresh ending_scratch_mut()),
    // so byte/half-word setters preserve the live RAM half instead of re-stamping a
    // stale frame-start word.
    let mut scratch = DungeonScratchWordState::default();
    let mut ram = vec![0; WRAM_SIZE];
    // A direct RAM write to the shared bytes (e.g. the gfx converter), with the
    // native left stale (default 0) to model a frame-start/persisted value.
    write_le_u16(&mut ram, DUNGEON_WORK_R16, 0x0201);
    write_le_u16(&mut ram, DUNGEON_WORK_R18, 0x0403);

    // Byte setter preserves the live RAM high byte (like C's `ram[R16] = v`) and
    // leaves R18 untouched — it does NOT re-stamp the stale native (which is 0).
    NativeDungeonScratchWordBridgeMut::new(&mut scratch, &mut ram).set_primary_low(0x80);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R16), 0x0280);
    assert_eq!(read_le_u16(&ram, ENDING_WORK_PRIMARY), 0x0280);
    assert_eq!(scratch.primary_word(), 0x0280);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R18), 0x0403);

    // decrement_primary_low preserves the high byte.
    let next =
        NativeDungeonScratchWordBridgeMut::new(&mut scratch, &mut ram).decrement_primary_low();
    assert_eq!(next, 0x7f);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R16), 0x027f);

    // increment_secondary_low preserves R18's high byte and leaves R16 alone.
    let next =
        NativeDungeonScratchWordBridgeMut::new(&mut scratch, &mut ram).increment_secondary_low();
    assert_eq!(next, 0x04);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R18), 0x0404);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R16), 0x027f);

    // Full-word setters overwrite the whole word.
    NativeDungeonScratchWordBridgeMut::new(&mut scratch, &mut ram).set_primary_word(0x1234);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R16), 0x1234);
    assert_eq!(scratch.primary_word(), 0x1234);
    NativeDungeonScratchWordBridgeMut::new(&mut scratch, &mut ram).set_secondary_word(0x5678);
    assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R18), 0x5678);
    assert_eq!(read_le_u16(&ram, ENDING_WORK_SECONDARY), 0x5678);
}

#[test]
fn native_dungeon_entrance_backup_bridge_does_not_project_world_owned_words() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_TILE_THEME_INDEX] = 0x11;
    ram[MAIN_TILE_THEME_INDEX] = 0x22;
    ram[AUX_TILE_THEME_INDEX] = 0x33;
    ram[SPRITE_GRAPHICS_INDEX] = 0x44;
    ram[OVERWORLD_SCREEN_INDEX + 1] = 0xaa;
    ram[OVERLAY_INDEX + 1] = 0xbb;

    let mut backup = DungeonEntranceBackupState::default();
    {
        let mut bridge = NativeDungeonEntranceBackupBridgeMut::new(&mut backup, &mut ram);
        bridge.cache_exit_tile_themes();
    }

    assert_eq!(backup.exit_tile_theme(0), 0x11);
    assert_eq!(backup.exit_tile_theme(1), 0x22);
    assert_eq!(backup.exit_tile_theme(2), 0x33);
    assert_eq!(backup.exit_tile_theme(3), 0x44);
    assert_eq!(
        &ram[OVERWORLD_EXIT_TILE_THEME_INDEX..OVERWORLD_EXIT_TILE_THEME_INDEX + 4],
        &[0x11, 0x22, 0x33, 0x44]
    );
    assert_eq!(ram[OVERWORLD_SCREEN_INDEX + 1], 0xaa);
    assert_eq!(ram[OVERLAY_INDEX + 1], 0xbb);
}
