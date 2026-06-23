use super::*;

#[test]
fn world_location_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
    ram[PLAYER_IS_INDOORS] = 1;

    let mut world = WorldLocationState::load_from_ram(&ram);
    assert_eq!(world.dungeon_room, 0x0124);
    assert_eq!(world.dungeon_room_index(), 0x24);
    assert_eq!(world.overworld_screen, 0x0040);
    assert_eq!(world.overworld_screen_index(), 0x40);
    assert!(world.is_indoors());
    assert!(!world.is_outdoors());

    world.dungeon_room = 0x0181;
    world.overworld_screen = 0x005b;
    world.indoor_flag = 0;
    world.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
    assert_eq!(ram[PLAYER_IS_INDOORS], 0);
}

#[test]
fn native_world_location_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
    ram[PLAYER_IS_INDOORS] = 1;

    let mut world = WorldLocationState::load_from_ram(&ram);
    {
        let mut bridge = NativeWorldLocationBridgeMut::new(&mut world, &mut ram);
        bridge.increment_dungeon_room_index_by(2);
        bridge.set_overworld_screen(0x5b);
        bridge.set_indoor_flag(0);
    }

    assert_eq!(world.dungeon_room, 0x0126);
    assert_eq!(world.overworld_screen, 0x005b);
    assert_eq!(world.indoor_flag, 0);
    assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0126);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
    assert_eq!(ram[PLAYER_IS_INDOORS], 0);
}

#[test]
fn native_world_location_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut world = WorldLocationState {
        dungeon_room: 0x0124,
        overworld_screen: 0x0040,
        indoor_flag: 1,
    };
    world.write_to_ram(&mut ram);

    write_le_u16(&mut ram, DUNGEON_ROOM, 0x00aa);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x00bb);
    ram[PLAYER_IS_INDOORS] = 0xcc;

    {
        let mut bridge = NativeWorldLocationBridgeMut::new(&mut world, &mut ram);
        bridge.set_overworld_screen(0x5b);
    }

    assert_eq!(world.dungeon_room, 0x0124);
    assert_eq!(world.overworld_screen, 0x005b);
    assert_eq!(world.indoor_flag, 1);
    assert_eq!(WorldLocationState::load_from_ram(&ram), world);
    assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0124);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
    assert_eq!(ram[PLAYER_IS_INDOORS], 1);
}

#[test]
fn world_camera_boundaries_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW, 0x0101);
    write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_HI, 0x0202);
    write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW, 0x0303);
    write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_HI, 0x0404);
    write_le_u16(&mut ram, UP_DOWN_SCROLL_TARGET, 0x0505);
    write_le_u16(&mut ram, LEFT_RIGHT_SCROLL_TARGET, 0x0606);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_UP_COUNTER, 0x0707);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_LEFT_COUNTER, 0x0808);
    write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT, 0x0909);
    write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT, 0x0a0a);
    write_le_u16(&mut ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_START, 0x0b0b);
    write_le_u16(&mut ram, SPECIAL_EXIT_ROOM_BOUNDS_X_END, 0x0c0c);

    let boundaries = WorldCameraBoundariesState::load_from_ram(&ram);
    assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x0101);
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x0202);
    assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0303);
    assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0404);
    assert_eq!(boundaries.up_down_scroll_target(0), 0x0505);
    assert_eq!(boundaries.up_down_scroll_target(2), 0x0606);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(0), 0x0707);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(2), 0x0808);
    assert_eq!(boundaries.spexit_camera_y_scroll_low(), 0x0909);
    assert_eq!(boundaries.spexit_camera_x_scroll_low(), 0x0a0a);
    assert_eq!(boundaries.spexit_room_bound_y_start(), 0x0b0b);
    assert_eq!(boundaries.spexit_room_bound_x_end(), 0x0c0c);

    let mut projected = vec![0; WRAM_SIZE];
    boundaries.write_to_ram(&mut projected);
    assert_eq!(
        WorldCameraBoundariesState::load_from_ram(&projected),
        boundaries
    );
}

#[test]
fn native_world_camera_boundaries_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut boundaries = WorldCameraBoundariesState::load_from_ram(&ram);
    {
        let mut bridge = NativeWorldCameraBoundariesBridgeMut::new(&mut boundaries, &mut ram);
        bridge.set_camera_y_coord_scroll_low(0x0101);
        bridge.set_camera_y_coord_scroll_hi(0x0202);
        bridge.set_camera_x_coord_scroll_low(0x0303);
        bridge.set_camera_x_coord_scroll_hi(0x0404);
        bridge.set_up_down_scroll_target(0x0505);
        bridge.set_left_right_scroll_target(0x0606);
        bridge.set_overworld_scroll_up_counter(0x0707);
        bridge.set_overworld_scroll_left_counter(0x0808);
        bridge.set_special_exit_room_bounds(0x0909, 0x0a0a, 0x0b0b, 0x0c0c);
        bridge.save_spexit_camera_coords();
    }

    assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x0101);
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x0202);
    assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0303);
    assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0404);
    assert_eq!(boundaries.up_down_scroll_target(0), 0x0505);
    assert_eq!(boundaries.up_down_scroll_target(2), 0x0606);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(0), 0x0707);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(2), 0x0808);
    assert_eq!(boundaries.spexit_camera_y_scroll_low(), 0x0101);
    assert_eq!(boundaries.spexit_camera_x_scroll_low(), 0x0303);
    assert_eq!(boundaries.spexit_room_bound_y_start(), 0x0909);
    assert_eq!(boundaries.spexit_room_bound_x_end(), 0x0c0c);
    assert_eq!(WorldCameraBoundariesState::load_from_ram(&ram), boundaries);
}

#[test]
fn world_camera_boundaries_state_owns_camera_target_and_cache_behavior() {
    let mut boundaries = WorldCameraBoundariesState::default();

    boundaries.set_camera_y_coord_scroll_low(0x0101);
    boundaries.set_camera_y_coord_scroll_hi(0x0202);
    boundaries.set_camera_x_coord_scroll_low(0x0303);
    boundaries.set_camera_x_coord_scroll_hi(0x0404);
    assert_eq!(boundaries.add_camera_scroll_for_axis(true, 0x10), 0x0414);
    assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0416);
    boundaries.set_camera_scroll_from_link_for_axis(false, 0x1200);
    assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x1202);
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x1200);

    boundaries.set_up_down_scroll_target(0x0505);
    boundaries.set_up_down_scroll_target_end(0x1515);
    boundaries.set_left_right_scroll_target(0x0606);
    boundaries.set_left_right_scroll_target_end(0x1616);
    boundaries.cache_scroll_targets();
    boundaries.set_up_down_scroll_target(0xaaaa);
    boundaries.restore_scroll_targets_from_cached();
    assert_eq!(boundaries.up_down_scroll_target(0), 0x0505);
    assert_eq!(boundaries.up_down_scroll_target(1), 0x1515);
    assert_eq!(boundaries.up_down_scroll_target(2), 0x0606);
    assert_eq!(boundaries.up_down_scroll_target(3), 0x1616);

    boundaries.set_overworld_scroll_up_counter(0x0707);
    boundaries.set_overworld_scroll_left_counter(0x0808);
    boundaries.set_opposed_scroll_counter_pair(2, 0x0010);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(2), 0x0010);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(3), 0xfff0);
    boundaries.clear_opposed_scroll_counters(2);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(2), 0);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(3), 0);

    boundaries.set_special_exit_room_bounds(0x0909, 0x0a0a, 0x0b0b, 0x0c0c);
    boundaries.save_spexit_camera_coords();
    boundaries.save_exit_camera_coords();
    boundaries.copy_spexit_scroll_targets();
    boundaries.copy_spexit_scroll_counters();
    boundaries.copy_exit_scroll_targets();
    boundaries.copy_exit_scroll_counters();
    assert_eq!(boundaries.spexit_camera_y_scroll_low(), 0x1202);
    assert_eq!(boundaries.spexit_camera_x_scroll_low(), 0x0416);
    assert_eq!(boundaries.spexit_room_bound_y_start(), 0x0909);
    assert_eq!(boundaries.spexit_room_bound_x_end(), 0x0c0c);

    boundaries.set_camera_scroll_from_link_for_axis(true, 0x2222);
    boundaries.restore_special_exit_camera_scroll();
    assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x1202);
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x1200);
    assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0416);
    assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0414);

    boundaries.cache_camera_scroll();
    boundaries.set_camera_scroll_from_link_for_axis(false, 0x3333);
    boundaries.set_camera_scroll_from_link_for_axis(true, 0x4444);
    boundaries.restore_camera_y_from_cached_indoor();
    boundaries.restore_camera_x_from_cached_indoor();
    assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x1202);
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x1204);
    assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0416);
    assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0418);
    boundaries.update_camera_hi_outdoor();
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x1200);
    assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0414);
}

#[test]
fn native_world_camera_boundaries_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut boundaries = WorldCameraBoundariesState::default();
    {
        let mut bridge = NativeWorldCameraBoundariesBridgeMut::new(&mut boundaries, &mut ram);
        bridge.set_camera_y_coord_scroll_low(0x0101);
        bridge.set_camera_y_coord_scroll_hi(0x0202);
        bridge.set_camera_x_coord_scroll_low(0x0303);
        bridge.set_up_down_scroll_target(0x0404);
        bridge.set_overworld_scroll_up_counter(0x0505);
    }

    write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW, 0xaaaa);
    write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_HI, 0xbbbb);
    write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW, 0xcccc);
    write_le_u16(&mut ram, UP_DOWN_SCROLL_TARGET, 0xdddd);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_UP_COUNTER, 0xeeee);

    {
        let mut bridge = NativeWorldCameraBoundariesBridgeMut::new(&mut boundaries, &mut ram);
        bridge.set_camera_x_coord_scroll_hi(0x0606);
    }

    assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x0101);
    assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x0202);
    assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0303);
    assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0606);
    assert_eq!(boundaries.up_down_scroll_target(0), 0x0404);
    assert_eq!(boundaries.overworld_scroll_counter_for_axis(0), 0x0505);
    assert_eq!(WorldCameraBoundariesState::load_from_ram(&ram), boundaries);
}

#[test]
fn world_palette_theme_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LAST_LIGHT_VS_DARK_WORLD] = 0x01;
    ram[AUX_BG_SUBSET_0] = 0x02;
    ram[AUX_BG_SUBSET_0 + 1] = 0x03;
    ram[AUX_BG_SUBSET_0 + 2] = 0x04;
    ram[AUX_BG_SUBSET_0 + 3] = 0x05;
    ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = 0x06;
    ram[OVERWORLD_PALETTE_MODE] = 0x07;
    ram[PALETTE_MAIN_INDOORS] = 0x08;
    ram[PALETTE_MAIN_INDOORS_COPY] = 0x09;
    ram[PALETTE_SWAP_FLAG] = 0x0a;
    ram[PALETTE_SP0L] = 0x0b;
    ram[PALETTE_SP5L] = 0x0c;
    ram[PALETTE_SP6L] = 0x0d;
    ram[PALETTE_SP6R_INDOORS] = 0x0e;
    ram[HUD_PALETTE] = 0x0f;
    ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 0x10;
    ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 0x11;
    ram[MISC_SPRITES_GRAPHICS_INDEX] = 0x12;
    ram[OVERWORLD_TILE_THEME_INDEX] = 0x13;
    ram[MAIN_TILE_THEME_INDEX] = 0x14;
    ram[AUX_TILE_THEME_INDEX] = 0x15;
    ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX] = 0x16;
    ram[MAIN_TILE_THEME_INDEX_SPEXIT] = 0x17;
    ram[AUX_TILE_THEME_INDEX_SPEXIT] = 0x18;
    ram[OVERWORLD_TILE_THEME_INDEX_EXIT] = 0x19;
    ram[MAIN_TILE_THEME_INDEX_EXIT] = 0x1a;
    ram[AUX_TILE_THEME_INDEX_EXIT] = 0x1b;

    let theme = WorldPaletteThemeState::load_from_ram(&ram);
    assert_eq!(theme.last_light_vs_dark_world(), 0x01);
    assert_eq!(theme.aux_bg_subset(0), 0x02);
    assert_eq!(theme.aux_bg_subset(3), 0x05);
    assert_eq!(theme.overworld_palette_aux1_hi(), 0x06);
    assert_eq!(theme.overworld_palette_mode(), 0x07);
    assert_eq!(theme.palette_main_indoors(), 0x08);
    assert_eq!(theme.palette_main_indoors_copy(), 0x09);
    assert_eq!(theme.palette_swap_flag(), 0x0a);
    assert_eq!(theme.palette_sp0l(), 0x0b);
    assert_eq!(theme.palette_sp5l(), 0x0c);
    assert_eq!(theme.palette_sp6l(), 0x0d);
    assert_eq!(theme.palette_sp6r_indoors(), 0x0e);
    assert_eq!(theme.hud_palette(), 0x0f);
    assert_eq!(theme.overworld_palette_aux2_hi(), 0x10);
    assert_eq!(theme.overworld_palette_aux3_lo(), 0x11);
    assert_eq!(theme.misc_sprites_graphics_index(), 0x12);
    assert_eq!(theme.main_tile_theme_index(), 0x14);
    assert_eq!(theme.aux_tile_theme_index(), 0x15);

    let mut projected = vec![0; WRAM_SIZE];
    theme.write_to_ram(&mut projected);
    // The exit_* tile-theme indices (0xc164-0xc166) are owned and projected by
    // DungeonEntranceBackupState (the Dungeon_LoadEntrance save). This struct
    // loads them only to feed restore_exit_tile_themes, so they intentionally
    // do NOT round-trip through WorldPaletteThemeState::write_to_ram.
    let mut expected = theme;
    expected.exit_overworld_tile_theme_index = 0;
    expected.exit_main_tile_theme_index = 0;
    expected.exit_aux_tile_theme_index = 0;
    // PALETTE_SWAP_FLAG (0xabd) is a load-only mirror owned/projected by the
    // follower; it intentionally does NOT round-trip through write_to_ram.
    expected.palette_swap_flag = 0;
    assert_eq!(WorldPaletteThemeState::load_from_ram(&projected), expected);
}

#[test]
fn native_world_palette_theme_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut theme = WorldPaletteThemeState::load_from_ram(&ram);
    {
        let mut bridge = NativeWorldPaletteThemeBridgeMut::new(&mut theme, &mut ram);
        bridge.set_last_light_vs_dark_world(0x40);
        bridge.set_aux_bg_subset(2, 0x22);
        bridge.set_overworld_palette_aux1_hi(0x33);
        bridge.set_hud_palette(0x44);
        bridge.set_overworld_tile_theme_index(0x55);
        bridge.set_main_tile_theme_index(0x66);
        bridge.set_aux_tile_theme_index(0x77);
        bridge.set_misc_sprites_graphics_index(0x88);
        bridge.set_palette_sp6r_indoors(0x99);
        bridge.save_special_exit_tile_themes();
    }

    assert_eq!(theme.last_light_vs_dark_world(), 0x40);
    assert_eq!(theme.aux_bg_subset(2), 0x22);
    assert_eq!(theme.overworld_palette_aux1_hi(), 0x33);
    assert_eq!(theme.hud_palette(), 0x44);
    assert_eq!(theme.main_tile_theme_index(), 0x66);
    assert_eq!(theme.aux_tile_theme_index(), 0x77);
    assert_eq!(theme.misc_sprites_graphics_index(), 0x88);
    assert_eq!(theme.palette_sp6r_indoors(), 0x99);
    assert_eq!(WorldPaletteThemeState::load_from_ram(&ram), theme);
    assert_eq!(ram[LAST_LIGHT_VS_DARK_WORLD], 0x40);
    assert_eq!(ram[AUX_BG_SUBSET_0 + 2], 0x22);
    assert_eq!(ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI], 0x33);
    assert_eq!(ram[HUD_PALETTE], 0x44);
    assert_eq!(ram[OVERWORLD_TILE_THEME_INDEX], 0x55);
    assert_eq!(ram[MAIN_TILE_THEME_INDEX], 0x66);
    assert_eq!(ram[AUX_TILE_THEME_INDEX], 0x77);
    assert_eq!(ram[MISC_SPRITES_GRAPHICS_INDEX], 0x88);
    assert_eq!(ram[PALETTE_SP6R_INDOORS], 0x99);
    assert_eq!(ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX], 0x55);
    assert_eq!(ram[MAIN_TILE_THEME_INDEX_SPEXIT], 0x66);
    assert_eq!(ram[AUX_TILE_THEME_INDEX_SPEXIT], 0x77);
}

#[test]
fn world_palette_theme_state_owns_theme_save_restore_behavior() {
    let mut theme = WorldPaletteThemeState::default();
    theme.set_aux_bg_subset(2, 0x22);
    theme.set_overworld_tile_theme_index(0x55);
    theme.set_main_tile_theme_index(0x66);
    theme.set_aux_tile_theme_index(0x77);
    theme.set_misc_sprites_graphics_index(0x88);
    theme.set_hud_palette(0x99);
    theme.set_palette_sp6r_indoors(0xaa);
    theme.save_special_exit_tile_themes();
    theme.set_overworld_tile_theme_index(0x11);
    theme.set_main_tile_theme_index(0x12);
    theme.set_aux_tile_theme_index(0x13);
    theme.restore_special_exit_tile_themes();

    assert_eq!(theme.aux_bg_subset(2), 0x22);
    assert_eq!(theme.overworld_tile_theme_index, 0x55);
    assert_eq!(theme.main_tile_theme_index(), 0x66);
    assert_eq!(theme.aux_tile_theme_index(), 0x77);
    assert_eq!(theme.misc_sprites_graphics_index(), 0x88);

    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_PALETTE_MODE] = 0x20;
    ram[PALETTE_MAIN_INDOORS] = 0x21;
    ram[PALETTE_SP0L] = 0x22;
    ram[PALETTE_SP5L] = 0x23;
    ram[PALETTE_SP6L] = 0x24;
    ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 0x25;
    ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 0x26;
    ram[HUD_PALETTE] = 0x27;
    ram[PALETTE_SP6R_INDOORS] = 0x28;

    theme.sync_shared_palette_aliases_from_ram(&ram, false, false);
    assert_eq!(theme.overworld_palette_mode(), 0x20);
    assert_eq!(theme.palette_main_indoors(), 0x21);
    assert_eq!(theme.palette_sp0l(), 0x22);
    assert_eq!(theme.palette_sp5l(), 0x23);
    assert_eq!(theme.palette_sp6l(), 0x24);
    assert_eq!(theme.overworld_palette_aux2_hi(), 0x25);
    assert_eq!(theme.overworld_palette_aux3_lo(), 0x26);
    assert_eq!(theme.hud_palette(), 0x99);
    assert_eq!(theme.palette_sp6r_indoors(), 0xaa);

    theme.sync_shared_palette_aliases_from_ram(&ram, true, true);
    assert_eq!(theme.hud_palette(), 0x27);
    assert_eq!(theme.palette_sp6r_indoors(), 0x28);
}

#[test]
fn native_world_palette_theme_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut theme = WorldPaletteThemeState::default();
    {
        let mut bridge = NativeWorldPaletteThemeBridgeMut::new(&mut theme, &mut ram);
        bridge.set_last_light_vs_dark_world(0x40);
        bridge.set_aux_bg_subset(1, 0x12);
        bridge.set_overworld_palette_aux1_hi(0x34);
        bridge.set_hud_palette(0x56);
        bridge.set_overworld_tile_theme_index(0x78);
    }

    ram[LAST_LIGHT_VS_DARK_WORLD] = 0xaa;
    ram[AUX_BG_SUBSET_0 + 1] = 0xbb;
    ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = 0xcc;
    ram[HUD_PALETTE] = 0xdd;
    ram[OVERWORLD_TILE_THEME_INDEX] = 0xee;
    ram[PALETTE_MAIN_INDOORS] = 0x06;
    ram[OVERWORLD_PALETTE_MODE] = 0x05;

    {
        let mut bridge = NativeWorldPaletteThemeBridgeMut::new(&mut theme, &mut ram);
        bridge.set_main_tile_theme_index(0x9a);
    }

    assert_eq!(theme.last_light_vs_dark_world(), 0x40);
    assert_eq!(theme.aux_bg_subset(1), 0x12);
    assert_eq!(theme.overworld_palette_aux1_hi(), 0x34);
    assert_eq!(theme.hud_palette(), 0xdd);
    assert_eq!(theme.main_tile_theme_index(), 0x9a);
    assert_eq!(theme.palette_main_indoors(), 0x06);
    assert_eq!(theme.overworld_palette_mode(), 0x05);
    assert_eq!(WorldPaletteThemeState::load_from_ram(&ram), theme);
}

#[test]
fn sprite_system_projection_preserves_world_palette_theme_fields() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[SPRITE_GRAPHICS_INDEX] = 0x12;
    ram[MAIN_TILE_THEME_INDEX] = 0x34;
    ram[AUX_TILE_THEME_INDEX] = 0x56;
    ram[MISC_SPRITES_GRAPHICS_INDEX] = 0x78;

    let system = SpriteSystemState::load_from_ram(&ram);
    let mut projected = ram.clone();
    projected[MAIN_TILE_THEME_INDEX] = 0x9a;
    projected[AUX_TILE_THEME_INDEX] = 0xbc;
    projected[MISC_SPRITES_GRAPHICS_INDEX] = 0xde;

    system.write_to_ram(&mut projected);

    assert_eq!(projected[SPRITE_GRAPHICS_INDEX], 0x12);
    assert_eq!(projected[MAIN_TILE_THEME_INDEX], 0x9a);
    assert_eq!(projected[AUX_TILE_THEME_INDEX], 0xbc);
    assert_eq!(projected[MISC_SPRITES_GRAPHICS_INDEX], 0xde);
}

#[test]
fn world_region_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, CURRENT_AREA_OF_PLAYER, 0x0102);
    write_le_u16(&mut ram, OVERWORLD_AREA_INDEX, 0x0304);
    write_le_u16(&mut ram, OVERWORLD_AREA_INDEX_SPEXIT, 0x0506);
    write_le_u16(&mut ram, OVERWORLD_AREA_INDEX_EXIT, 0x0708);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_PREV, 0x090a);
    write_le_u16(&mut ram, OVERLAY_INDEX, 0x0b0c);
    ram[RNG_SEED] = 0x0d;
    ram[IS_IN_DARK_WORLD_FLAG] = 0x0e;
    ram[FLAG_OVERWORLD_AREA_CHANGED] = 0x0f;
    write_le_u16(&mut ram, WHICH_ENTRANCE, 0x1011);
    write_le_u16(&mut ram, OW_ENTRANCE_VALUE, 0x1213);

    let region = WorldRegionState::load_from_ram(&ram);
    assert_eq!(region.current_area_of_player_word(), 0x0102);
    assert_eq!(region.overworld_area_index_word(), 0x0304);
    assert_eq!(region.spexit_area_index(), 0x0506);
    assert_eq!(region.prev_screen_index_word(), 0x090a);
    assert_eq!(region.overlay_index(), 0x0c);
    assert_eq!(region.rng_seed(), 0x0d);
    assert_eq!(region.dark_world_region_index(), 0x0e);
    assert!(region.is_in_dark_world());
    assert!(region.flag_overworld_area_changed());
    assert_eq!(region.which_entrance(), 0x1011);
    assert_eq!(region.ow_entrance_value(), 0x1213);

    let mut projected = vec![0; WRAM_SIZE];
    region.write_to_ram(&mut projected);
    assert_eq!(WorldRegionState::load_from_ram(&projected), region);
}

#[test]
fn native_world_region_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut region = WorldRegionState::load_from_ram(&ram);
    {
        let mut bridge = NativeWorldRegionBridgeMut::new(&mut region, &mut ram);
        bridge.set_current_area_of_player_word(0x0102);
        bridge.set_overworld_area_index_word(0x0304);
        bridge.save_spexit_area_index();
        bridge.save_exit_area_index();
        bridge.set_prev_screen_index_word(0x0506);
        bridge.set_overlay_index_word(0x0708);
        bridge.set_rng_seed(0x09);
        bridge.set_dark_world_region_index(0x0a);
        bridge.set_flag_overworld_area_changed(0x0b);
        bridge.set_which_entrance(0x0c0d);
        bridge.set_ow_entrance_value(0x0e0f);
    }

    assert_eq!(region.current_area_of_player_word(), 0x0102);
    assert_eq!(region.overworld_area_index_word(), 0x0304);
    assert_eq!(region.spexit_area_index(), 0x0304);
    assert_eq!(region.prev_screen_index_word(), 0x0506);
    assert_eq!(region.overlay_index(), 0x08);
    assert_eq!(region.rng_seed(), 0x09);
    assert_eq!(region.dark_world_region_index(), 0x0a);
    assert!(region.flag_overworld_area_changed());
    assert_eq!(region.which_entrance(), 0x0c0d);
    assert_eq!(region.ow_entrance_value(), 0x0e0f);
    assert_eq!(WorldRegionState::load_from_ram(&ram), region);
}

#[test]
fn world_region_state_owns_area_and_entrance_behavior() {
    let mut region = WorldRegionState::default();

    region.set_current_area_of_player_word(0x0102);
    region.set_overworld_area_index_word(0x0304);
    region.save_spexit_area_index();
    region.save_exit_area_index();
    region.set_overworld_area_index(0x05);
    assert_eq!(region.overworld_area_index_word(), 0x0305);
    region.restore_spexit_area_index();
    assert_eq!(region.overworld_area_index_word(), 0x0304);
    region.set_overworld_area_index_word(0x0607);
    region.restore_exit_area_index();
    assert_eq!(region.overworld_area_index_word(), 0x0304);

    region.set_prev_screen_index_word(0x0506);
    region.set_overlay_index_word(0x0708);
    region.set_overlay_high(0x09);
    region.set_rng_seed(0x0a);
    region.set_dark_world_region_index(0x0b);
    region.set_flag_overworld_area_changed(1);
    region.set_which_entrance(0x0c0d);
    region.set_which_entrance_byte(0x0e);
    region.set_ow_entrance_value(0x0f10);

    assert_eq!(region.current_area_of_player_word(), 0x0102);
    assert_eq!(region.spexit_area_index(), 0x0304);
    assert_eq!(region.prev_screen_index_word(), 0x0506);
    assert_eq!(region.overlay_index(), 0x08);
    assert_eq!(region.overlay_index, 0x0908);
    assert_eq!(region.rng_seed(), 0x0a);
    assert_eq!(region.dark_world_region_index(), 0x0b);
    assert!(region.flag_overworld_area_changed());
    assert_eq!(region.which_entrance(), 0x0c0e);
    assert_eq!(region.ow_entrance_value(), 0x0f10);

    region.clear_flag_overworld_area_changed();
    region.clear_overlay_index_word();
    assert!(!region.flag_overworld_area_changed());
    assert_eq!(region.overlay_index, 0);
}

#[test]
fn native_world_region_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut region = WorldRegionState::default();
    {
        let mut bridge = NativeWorldRegionBridgeMut::new(&mut region, &mut ram);
        bridge.set_current_area_of_player_word(0x0102);
        bridge.set_overworld_area_index_word(0x0304);
        bridge.set_rng_seed(0x05);
        bridge.set_dark_world_region_index(0x06);
        bridge.set_which_entrance(0x0708);
    }

    write_le_u16(&mut ram, CURRENT_AREA_OF_PLAYER, 0xaaaa);
    write_le_u16(&mut ram, OVERWORLD_AREA_INDEX, 0xbbbb);
    ram[RNG_SEED] = 0xcc;
    ram[IS_IN_DARK_WORLD_FLAG] = 0xdd;
    write_le_u16(&mut ram, WHICH_ENTRANCE, 0xeeee);

    {
        let mut bridge = NativeWorldRegionBridgeMut::new(&mut region, &mut ram);
        bridge.set_ow_entrance_value(0x090a);
    }

    assert_eq!(region.current_area_of_player_word(), 0x0102);
    assert_eq!(region.overworld_area_index_word(), 0x0304);
    assert_eq!(region.rng_seed(), 0x05);
    assert_eq!(region.dark_world_region_index(), 0x06);
    assert_eq!(region.which_entrance(), 0x0708);
    assert_eq!(region.ow_entrance_value(), 0x090a);
    assert_eq!(WorldRegionState::load_from_ram(&ram), region);
}

#[test]
fn world_transient_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0x01;
    ram[ALLOW_SCROLL_Z] = 0x02;
    ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] = 0x03;
    write_le_u16(&mut ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED, 0x0405);
    write_le_u16(&mut ram, SAVEGAME_HAS_MASTER_SWORD_FLAGS, 0x0607);
    ram[IS_STANDING_IN_DOORWAY_CACHED] = 0x09;
    write_le_u16(&mut ram, CACHED_ROOM_BOUNDS_Y_START, 0x0a0b);
    write_le_u16(&mut ram, CACHED_ROOM_BOUNDS_X_END, 0x0c0d);
    write_le_u16(&mut ram, OVERWORLD_PEG_PUZZLE_PROGRESS, 0x0e0f);
    ram[OVERWORLD_HOLE_TILEMAP_POS] = 0x10;
    ram[HUD_CUR_ITEM_X] = 0x11;
    write_le_u16(&mut ram, DOOR_ANIMATION_STEP_INDICATOR, 0x1213);
    ram[ROOM_TRANSITIONING_FLAGS] = 0x14;
    ram[QUADRANT_FULLSIZE_X] = 0x15;
    ram[QUADRANT_FULLSIZE_Y] = 0x16;
    ram[MAPBAK_TM] = 0x17;
    ram[MAPBAK_TS] = 0x18;
    ram[OVERWORLD_HOLE_SCAN_STEP] = 0x19;
    write_le_u16(&mut ram, DUNG_REPLACEMENT_TILE_STATE + 4, 0x1a1b);

    let transient = WorldTransientState::load_from_ram(&ram);
    assert_eq!(transient.flag_custom_spell_anim_active(), 0x01);
    assert_eq!(transient.allow_scroll_z(), 0x02);
    assert_eq!(transient.milestone_item_gfx_swap_countdown(), 0x03);
    assert_eq!(transient.big_key_door_message_triggered(), 0x0405);
    assert_eq!(transient.savegame_has_master_sword_flags(), 0x0607);
    // super_bomb_indicator_timer (0x4b4) is no longer owned by world_transient — it
    // belongs to display.hud_tilemap runtime (see overworld super-bomb indicator fix).
    assert_eq!(transient.is_standing_in_doorway_cached(), 0x09);
    assert_eq!(transient.overworld_peg_puzzle_progress(), 0x0e0f);
    assert_eq!(transient.overworld_hole_tilemap_pos(), 0x10);
    assert_eq!(transient.hud_cur_item_x(), 0x11);
    assert_eq!(transient.door_animation_step(), 0x1213);
    assert_eq!(transient.room_transitioning_flags(), 0x14);
    assert_eq!(transient.quadrant_fullsize_x(), 0x15);
    assert_eq!(transient.quadrant_fullsize_y(), 0x16);
    assert_eq!(transient.dung_replacement_tile_state(2), 0x1a1b);

    let mut projected = vec![0; WRAM_SIZE];
    transient.write_to_ram(&mut projected);
    assert_eq!(WorldTransientState::load_from_ram(&projected), transient);
}

#[test]
fn world_transient_state_owns_transient_behavior() {
    let mut transient = WorldTransientState::default();

    transient.set_custom_spell_animation_active();
    transient.set_allow_scroll_z(0x02);
    transient.set_room_transitioning_flags(0x03);
    transient.set_cached_room_bounds(0x0405, 0x0607, 0x0809, 0x0a0b);
    transient.set_standing_in_doorway_cached(0x0c);
    transient.set_door_animation_step_word(0x0d0e);
    transient.set_quadrant_fullsize_x(0x0f);
    transient.set_quadrant_fullsize_y(0x10);
    transient.cache_quadrant_fullsize_state();
    transient.set_quadrant_fullsize_x(0x20);
    transient.set_quadrant_fullsize_y(0x21);
    transient.restore_quadrant_fullsize_from_cached();
    transient.set_mapbak_tm(0x11);
    transient.set_mapbak_ts(0x12);
    transient.set_overworld_peg_puzzle_progress(0x1314);
    transient.set_dung_replacement_tile_state(2, 0x1516);

    assert_eq!(transient.flag_custom_spell_anim_active(), 1);
    assert_eq!(transient.allow_scroll_z(), 0x02);
    assert_eq!(transient.room_transitioning_flags(), 0x03);
    assert_eq!(transient.is_standing_in_doorway_cached(), 0x0c);
    assert_eq!(transient.door_animation_step(), 0x0d0e);
    assert_eq!(
        transient.dung_replacement_tile_state(DOOR_ANIMATION_REPLACEMENT_TILE_INDEX),
        0x0d0e
    );
    assert_eq!(transient.quadrant_fullsize_x(), 0x0f);
    assert_eq!(transient.quadrant_fullsize_y(), 0x10);
    assert_eq!(transient.overworld_peg_puzzle_progress(), 0x1314);
    assert_eq!(transient.dung_replacement_tile_state(2), 0x1516);

    transient.clear_custom_spell_animation();
    transient.clear_tile_interaction_shared_flag();
    transient.clear_hud_floor_changed_timer();
    assert_eq!(transient.flag_custom_spell_anim_active(), 0);

    transient.set_fullsize_overworld_quadrants();
    assert_eq!(transient.quadrant_fullsize_x(), 2);
    assert_eq!(transient.quadrant_fullsize_y(), 2);

    transient.apply_dungeon_layout_quadrant_fullsize(0xff, 0x01, 0x02, false, true);
    assert_eq!(transient.quadrant_fullsize_x(), 0);
    assert_eq!(transient.quadrant_fullsize_y(), 2);

    transient.apply_reset_xy_quadrant_overrides(0x4433);
    assert_eq!(transient.quadrant_fullsize_x(), 0x33);
    assert_eq!(transient.quadrant_fullsize_y(), 0x44);

    transient.set_tilemap_layer_copy(0x1234);
    transient.save_spexit_tm_copy();
    transient.set_tilemap_layer_copy(0);
    transient.restore_spexit_layer_masks();
    assert_eq!(transient.tilemap_layer_copy, 0x1234);

    transient.save_exit_tm_copy();
    transient.set_tilemap_layer_copy(0);
    transient.restore_exit_layer_masks();
    assert_eq!(transient.tilemap_layer_copy, 0x1234);

    assert_eq!(transient.increment_move_overlay_ctr(), 1);
    transient.decrement_milestone_item_gfx_swap_countdown();
    assert_eq!(transient.milestone_item_gfx_swap_countdown(), 0xff);
}

#[test]
fn native_world_transient_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut transient = WorldTransientState::default();
    {
        let mut bridge = NativeWorldTransientBridgeMut::new(&mut transient, &mut ram);
        bridge.set_custom_spell_animation_active();
        bridge.set_allow_scroll_z(0x02);
        bridge.set_room_transitioning_flags(0x03);
        bridge.set_cached_room_bounds(0x0405, 0x0607, 0x0809, 0x0a0b);
        bridge.set_standing_in_doorway_cached(0x0c);
        bridge.set_door_animation_step_word(0x0d0e);
        bridge.set_quadrant_fullsize_x(0x0f);
        bridge.set_quadrant_fullsize_y(0x10);
        bridge.cache_quadrant_fullsize_state();
        bridge.set_mapbak_tm(0x11);
        bridge.set_mapbak_ts(0x12);
        bridge.set_overworld_peg_puzzle_progress(0x1314);
        bridge.set_dung_replacement_tile_state(2, 0x1516);
    }

    assert_eq!(transient.flag_custom_spell_anim_active(), 1);
    assert_eq!(transient.allow_scroll_z(), 0x02);
    assert_eq!(transient.room_transitioning_flags(), 0x03);
    assert_eq!(transient.is_standing_in_doorway_cached(), 0x0c);
    assert_eq!(transient.door_animation_step(), 0x0d0e);
    assert_eq!(transient.quadrant_fullsize_x(), 0x0f);
    assert_eq!(transient.quadrant_fullsize_y(), 0x10);
    assert_eq!(transient.overworld_peg_puzzle_progress(), 0x1314);
    assert_eq!(transient.dung_replacement_tile_state(2), 0x1516);
    assert_eq!(ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE], 1);
    assert_eq!(ram[ALLOW_SCROLL_Z], 0x02);
    assert_eq!(ram[ROOM_TRANSITIONING_FLAGS], 0x03);
    assert_eq!(ram[IS_STANDING_IN_DOORWAY_CACHED], 0x0c);
    assert_eq!(read_le_u16(&ram, DOOR_ANIMATION_STEP_INDICATOR), 0x0d0e);
    assert_eq!(ram[QUADRANT_FULLSIZE_X], 0x0f);
    assert_eq!(ram[QUADRANT_FULLSIZE_Y], 0x10);
    assert_eq!(read_le_u16(&ram, OVERWORLD_PEG_PUZZLE_PROGRESS), 0x1314);
    assert_eq!(read_le_u16(&ram, DUNG_REPLACEMENT_TILE_STATE + 4), 0x1516);
}

#[test]
fn native_world_transient_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut transient = WorldTransientState::default();
    {
        let mut bridge = NativeWorldTransientBridgeMut::new(&mut transient, &mut ram);
        bridge.set_custom_spell_animation_active();
        bridge.set_allow_scroll_z(0x02);
        bridge.set_room_transitioning_flags(0x03);
        bridge.set_door_animation_step_word(0x0405);
    }

    ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0xaa;
    ram[ALLOW_SCROLL_Z] = 0xbb;
    ram[ROOM_TRANSITIONING_FLAGS] = 0xcc;
    write_le_u16(&mut ram, DOOR_ANIMATION_STEP_INDICATOR, 0xdddd);

    {
        let mut bridge = NativeWorldTransientBridgeMut::new(&mut transient, &mut ram);
        bridge.clear_custom_spell_animation();
    }

    assert_eq!(transient.flag_custom_spell_anim_active(), 0);
    assert_eq!(transient.allow_scroll_z(), 0x02);
    assert_eq!(transient.room_transitioning_flags(), 0x03);
    assert_eq!(transient.door_animation_step(), 0xdddd);
    assert_eq!(ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE], 0);
    assert_eq!(ram[ALLOW_SCROLL_Z], 0x02);
    assert_eq!(ram[ROOM_TRANSITIONING_FLAGS], 0x03);
    assert_eq!(read_le_u16(&ram, DOOR_ANIMATION_STEP_INDICATOR), 0xdddd);
}

#[test]
fn world_scroll_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    // BG scroll copy2 (0xe0-0xe9) moved to PpuScrollCopyState; tested there.
    write_le_u16(&mut ram, BG1_X_OFFSET, 0x0505);
    write_le_u16(&mut ram, BG1_Y_OFFSET, 0x0606);
    write_le_u16(&mut ram, OVERWORLD_OFFSET_BASE_X, 0x0909);
    write_le_u16(&mut ram, OVERWORLD_OFFSET_BASE_Y, 0x0a0a);
    write_le_u16(&mut ram, OVERWORLD_OFFSET_MASK_X, 0x0b0b);
    write_le_u16(&mut ram, OVERWORLD_OFFSET_MASK_Y, 0x0c0c);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_X_START, 0x0d0d);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_X_END, 0x0e0e);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_Y_END, 0x0f0f);

    let mut scroll = WorldScrollState::load_from_ram(&ram);
    assert_eq!(scroll.bg1_x_offset(), 0x0505);
    assert_eq!(scroll.bg1_y_offset(), 0x0606);
    assert_eq!(scroll.overworld_offset_base_x(), 0x0909);
    assert_eq!(scroll.overworld_offset_base_y(), 0x0a0a);
    assert_eq!(scroll.overworld_offset_mask_x(), 0x0b0b);
    assert_eq!(scroll.overworld_offset_mask_y(), 0x0c0c);
    assert_eq!(scroll.scroll_x_start(), 0x0d0d);
    assert_eq!(scroll.scroll_x_end(), 0x0e0e);
    assert_eq!(scroll.scroll_y_end(), 0x0f0f);

    scroll.set_bg1_x_offset(0x5555);
    scroll.set_bg1_y_offset(0x6666);
    scroll.set_overworld_offset_base_x(0x9999);
    scroll.set_overworld_offset_base_y(0xaaaa);
    scroll.set_overworld_offset_mask_x(0xbbbb);
    scroll.set_overworld_offset_mask_y(0xcccc);
    scroll.set_scroll_x_start(0xdddd);
    scroll.set_scroll_x_end(0xeeee);
    scroll.set_scroll_y_end(0xffff);
    scroll.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, BG1_X_OFFSET), 0x5555);
    assert_eq!(read_le_u16(&ram, BG1_Y_OFFSET), 0x6666);
    assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_BASE_X), 0x9999);
    assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_BASE_Y), 0xaaaa);
    assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_MASK_X), 0xbbbb);
    assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_MASK_Y), 0xcccc);
    // scroll_x_start/x_end/y_end (0x604/0x606/0x602) are C's ow_scroll_vars0, owned by
    // RoomBoundsState. WorldScrollState only mirrors them on load; write_to_ram must NOT
    // project them (doing so clobbered RoomBoundsState's camera boundary). They retain the
    // seeded value.
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCROLL_X_START), 0x0d0d);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCROLL_X_END), 0x0e0e);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCROLL_Y_END), 0x0f0f);
}

#[test]
fn world_scroll_state_owns_scroll_and_offset_behavior() {
    let mut scroll = WorldScrollState::default();

    scroll.set_bg1_offsets(0x1111, 0x2222);
    assert_eq!(scroll.bg1_offset_mask(), 0x3333);
    scroll.clear_bg1_offsets();
    scroll.set_overworld_offset_base_x(0x9999);
    scroll.set_overworld_offset_base_y(0xaaaa);
    scroll.set_overworld_offset_mask_x(0xbbbb);
    scroll.set_overworld_offset_mask_y(0xcccc);
    scroll.set_scroll_x_start(0xdddd);
    scroll.set_scroll_x_end(0xeeee);
    scroll.set_scroll_y_end(0xffff);

    assert_eq!(scroll.bg1_x_offset(), 0);
    assert_eq!(scroll.bg1_y_offset(), 0);
    assert_eq!(scroll.overworld_offset_base_x(), 0x9999);
    assert_eq!(scroll.overworld_offset_base_y(), 0xaaaa);
    assert_eq!(scroll.overworld_offset_mask_x(), 0xbbbb);
    assert_eq!(scroll.overworld_offset_mask_y(), 0xcccc);
    assert_eq!(scroll.scroll_x_start(), 0xdddd);
    assert_eq!(scroll.scroll_x_end(), 0xeeee);
    assert_eq!(scroll.scroll_y_end(), 0xffff);
}

#[test]
fn native_world_scroll_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut scroll = WorldScrollState {
        bg1_x_offset: 0x0505,
        bg1_y_offset: 0x0606,
        overworld_offset_base_x: 0x0909,
        overworld_offset_base_y: 0x0a0a,
        overworld_offset_mask_x: 0x0b0b,
        overworld_offset_mask_y: 0x0c0c,
        scroll_x_start: 0x0d0d,
        scroll_x_end: 0x0e0e,
        scroll_y_end: 0x0f0f,
    };
    scroll.write_to_ram(&mut ram);
    // scroll_x_start/x_end/y_end are RoomBoundsState-owned (ow_scroll_vars0); WorldScrollState
    // only mirrors them on load and no longer projects them, so seed them in RAM for the
    // load round-trip below to stay consistent.
    write_le_u16(&mut ram, OVERWORLD_SCROLL_X_START, 0x0d0d);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_X_END, 0x0e0e);
    write_le_u16(&mut ram, OVERWORLD_SCROLL_Y_END, 0x0f0f);

    write_le_u16(&mut ram, OVERWORLD_OFFSET_BASE_X, 0xcccc);
    write_le_u16(&mut ram, BG1_X_OFFSET, 0xaaaa);

    {
        let mut bridge = NativeWorldScrollBridgeMut::new(&mut scroll, &mut ram);
        bridge.set_bg1_x_offset(0x1234);
    }

    assert_eq!(scroll.bg1_x_offset(), 0x1234);
    assert_eq!(scroll.overworld_offset_base_x(), 0x0909);
    assert_eq!(WorldScrollState::load_from_ram(&ram), scroll);
    assert_eq!(read_le_u16(&ram, BG1_X_OFFSET), 0x1234);
    assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_BASE_X), 0x0909);
}

#[test]
fn game_state_loads_grouped_world_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
    ram[PLAYER_IS_INDOORS] = 1;
    write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0206);
    ram[OVERWORLD_MAP_FLAGS] = 0x03;
    write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0004);
    ram[MODE7_ZOOM_STEP_COUNTER] = 2;
    ram[TIMER_FOR_MODE7_ZOOM] = 12;
    write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
    ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x20;
    write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x03e4);
    ram[OVERWORLD_SCROLL_DELTA] = 0x11;
    ram[OVERWORLD_SCROLL_DELTA + 1] = 0x22;
    ram[OVERWORLD_SCROLL_DELTA + 2] = 0x33;
    ram[BIRD_TRAVEL_X_LO + 3] = 0x34;
    ram[BIRD_TRAVEL_X_HI + 3] = 0x12;
    ram[BIRD_TRAVEL_Y_LO + 3] = 0x78;
    ram[BIRD_TRAVEL_Y_HI + 3] = 0x56;
    write_le_u16(&mut ram, WEATHERVANE_COUNTDOWN, 0x0280);
    ram[WEATHERVANE_MUSIC_LATCH] = 1;
    ram[WEATHERVANE_SOURCE_SLOT] = 7;
    ram[WEATHERVANE_OAM_OFFSET] = 0x10;
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
    ram[TRIGGER_SPECIAL_ENTRANCE] = 1;
    ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 3;
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0022);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0033);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0004);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0008);
    ram[OVERWORLD_TRANSITION_DIR] = 2;
    ram[OVERWORLD_EVENT_INFO + 0x5b] = 0x20;
    write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0x0123);
    write_le_u16(&mut ram, MULTISELECT_CHOICE, 0x0204);
    ram[MULTISELECT_CHOICE_BACKUP] = 0x07;
    ram[DIALOGUE_NUMBER_LO] = 0x12;
    ram[DIALOGUE_NUMBER_HI] = 0x34;

    let mut state = GameState::load_from_ram(&ram);
    assert_eq!(state.world.location.dungeon_room, 0x0124);
    assert_eq!(state.world.location.overworld_screen, 0x0040);
    assert_eq!(state.world.location.indoor_flag, 1);
    assert_eq!(state.world.overworld.map_ui.map_state_word(), 0x0206);
    assert_eq!(state.world.overworld.map_zoom.timer, 12);
    assert_eq!(state.world.overworld.screen_size.is_big_area_word(), 0x0120);
    assert!(state.world.overworld.screen_size.is_big_area());
    assert_eq!(
        state.world.overworld.screen_size.right_bottom_bound_word(),
        0x03e4
    );
    assert_eq!(
        state.world.overworld.scroll_delta.vertical_delta_word(),
        0x2211
    );
    assert_eq!(
        state.world.overworld.scroll_delta.horizontal_delta_word(),
        0x3322
    );
    assert_eq!(
        state
            .world
            .overworld
            .bird_travel_destinations
            .destination(3),
        BirdTravelDestinationState {
            x: 0x1234,
            y: 0x5678,
        }
    );
    assert_eq!(
        state.world.overworld.weather_vane,
        WeatherVaneState {
            countdown: 0x0280,
            music_latch: 1,
            source_slot: 7,
            oam_offset: 0x10,
        }
    );
    assert_eq!(state.world.overworld.map16.active_load.src_off, 0x1234);
    assert_eq!(state.world.overworld.entrance.sequence_counter, 3);
    assert_eq!(state.world.overworld.exit.special_exit_screen, 0x0033);
    assert_eq!(
        state.world.overworld.transition.direction_bits_word(),
        0x0008
    );
    assert_eq!(state.world.overworld.event_info.event_info(0x5b), 0x20);
    assert_eq!(state.messaging.dialogue_message_index.value(), 0x0123);
    assert_eq!(state.messaging.multiselect_choice.value(), 0x04);
    assert_eq!(
        MultiselectChoiceRead::new(
            &state.messaging.multiselect_choice,
            &state.messaging.runtime
        )
        .value_word(),
        0x0204
    );
    assert_eq!(state.messaging.multiselect_choice.backup(), 0x07);
    assert_eq!(state.messaging.dialogue_number.packed_digits(0), 0x12);
    assert_eq!(state.messaging.dialogue_number.packed_digits(1), 0x34);

    state.world.location.dungeon_room = 0x0181;
    state.world.location.overworld_screen = 0x005b;
    state.world.location.indoor_flag = 0;
    state.world.overworld.event_info.set_event_bits(0x5b, 0x40);
    state.world.overworld.map_ui.map_flags = 0x81;
    state.world.overworld.map_zoom.timer = 4;
    state.world.overworld.screen_size.big_area = 0x0020;
    state.world.overworld.screen_size.big_area_backup = 0x20;
    state.world.overworld.screen_size.right_bottom_scroll_bound = 0x01e4;
    state
        .world
        .overworld
        .scroll_delta
        .set_vertical_delta_word(0x4433);
    state
        .world
        .overworld
        .scroll_delta
        .set_horizontal_delta_word(0x5544);
    state
        .world
        .overworld
        .bird_travel_destinations
        .set_destination(3, 0x2345, 0x6789);
    state.world.overworld.weather_vane.countdown = 0x0001;
    state.world.overworld.weather_vane.music_latch = 2;
    state.world.overworld.weather_vane.source_slot = 9;
    state.world.overworld.weather_vane.oam_offset = 0x20;
    state.world.overworld.map16.active_load.src_off = 0x4567;
    state.world.overworld.entrance.sequence_counter = 9;
    state.world.overworld.exit.exit_screen = 0x0044;
    state.world.overworld.transition.direction_enum = 3;
    state.messaging.dialogue_message_index.set_value(0x0140);
    state.messaging.multiselect_choice.set_value(0x05);
    state.messaging.multiselect_choice.save_backup();
    state
        .messaging
        .dialogue_number
        .set_packed_digits(0x56, 0x78);
    state.write_to_ram(&mut ram);
    // entrance sequence_counter (0xc8) is reused scratch — written through, not by the
    // master projection — so flush it explicitly (as the entrance bridge does).
    state
        .world
        .overworld
        .entrance
        .write_sequence_counter_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
    assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x5b], 0x60);
    assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x81);
    assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 4);
    assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
    assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
    assert_eq!(
        read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
        0x01e4
    );
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA], 0x33);
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 1], 0x44);
    // 0x6a0 (horizontal-delta high byte) is mode-reused MIRROR_VARS scratch: the master
    // projection does NOT own it (it would clobber the mirror-warp target index), so it
    // keeps its prior value. The horizontal-word setter writes it through instead.
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 2], 0x33);
    assert_eq!(ram[BIRD_TRAVEL_X_LO + 3], 0x45);
    assert_eq!(ram[BIRD_TRAVEL_X_HI + 3], 0x23);
    assert_eq!(ram[BIRD_TRAVEL_Y_LO + 3], 0x89);
    assert_eq!(ram[BIRD_TRAVEL_Y_HI + 3], 0x67);
    assert_eq!(read_le_u16(&ram, WEATHERVANE_COUNTDOWN), 0x0001);
    assert_eq!(ram[WEATHERVANE_MUSIC_LATCH], 2);
    assert_eq!(ram[WEATHERVANE_SOURCE_SLOT], 9);
    assert_eq!(ram[WEATHERVANE_OAM_OFFSET], 0x20);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x4567);
    assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 9);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0044);
    assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 3);
    assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
    assert_eq!(read_le_u16(&ram, MULTISELECT_CHOICE), 0x0205);
    assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 0x05);
    assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x56);
    assert_eq!(ram[DIALOGUE_NUMBER_HI], 0x78);
}

#[test]
fn overworld_event_info_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_EVENT_INFO + 0x02] = 0x40;
    ram[OVERWORLD_EVENT_INFO + 0x5b] = 0x20;
    ram[OVERWORLD_EVENT_INFO + 0x9f] = 0x02;

    let mut event_info = OverworldEventInfoState::load_from_ram(&ram);
    assert_eq!(event_info.event_info(0x02), 0x40);
    assert_eq!(event_info.event_info(0x5b), 0x20);
    assert_eq!(event_info.event_info(0x9f), 0x02);
    assert_eq!(event_info.event_info(0xa0), 0);
    assert!(event_info.has_event_bits(0x5b, 0x20));

    event_info.set_event_info(0x02, 0x10);
    event_info.set_event_bits(0x5b, 0x40);
    event_info.clear_event_bits(0x9f, 0x02);
    event_info.write_to_ram(&mut ram);

    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x02], 0x10);
    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x5b], 0x60);
    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x9f], 0);
}

#[test]
fn native_overworld_event_info_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_EVENT_INFO + 0x02] = 0x40;
    ram[OVERWORLD_EVENT_INFO + 0x5b] = 0x20;
    ram[OVERWORLD_EVENT_INFO + 0x9f] = 0x02;

    let mut event_info = OverworldEventInfoState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldEventInfoBridgeMut::new(&mut event_info, &mut ram);
        bridge.set_event_info(0x02, 0x10);
        bridge.set_event_bits(0x5b, 0x40);
        bridge.clear_event_bits(0x9f, 0x02);
    }

    assert_eq!(event_info.event_info(0x02), 0x10);
    assert_eq!(event_info.event_info(0x5b), 0x60);
    assert_eq!(event_info.event_info(0x9f), 0);
    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x02], 0x10);
    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x5b], 0x60);
    assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x9f], 0);
}

#[test]
fn overworld_config_table_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_MUSIC_TABLE + 0x02] = 0x31;
    ram[OVERWORLD_MUSIC_TABLE + 0x80] = 0x42;
    ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02] = 0x05;
    ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02] = 0x18;
    ram[OVERWORLD_SPRITE_PALETTE_TABLE] = 0x09;

    let mut config_table = OverworldConfigTableState::load_from_ram(&ram);
    assert_eq!(config_table.music(0x02), 0x31);
    assert_eq!(config_table.music(0x80), 0x42);
    assert_eq!(config_table.sprite_palette(0x02), 0x05);
    assert_eq!(config_table.sprite_graphics(0x02), 0x18);
    assert_eq!(config_table.sprite_graphics(0x80), 0x09);
    assert_eq!(config_table.music(0xa0), 0);

    config_table.set_music(0x02, 0x6a);
    config_table.write_to_ram(&mut ram);

    assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x02], 0x6a);
    assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x80], 0x42);
    assert_eq!(ram[OVERWORLD_SPRITE_PALETTE_TABLE], 0x09);
    assert_eq!(ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02], 0x05);
    assert_eq!(ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02], 0x18);
    assert_eq!(ram[OVERWORLD_SPRITE_GFX_TABLE + 0x80], 0x09);
}

#[test]
fn native_overworld_config_table_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_MUSIC_TABLE + 0x02] = 0x31;
    ram[OVERWORLD_MUSIC_TABLE + 0x80] = 0x42;
    ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02] = 0x05;
    ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02] = 0x18;

    let primary = [0x24; 64];
    let secondary = [0x46; 96];
    let mut config_table = OverworldConfigTableState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldConfigTableBridgeMut::new(&mut config_table, &mut ram);
        bridge.copy_music_primary(&primary);
        bridge.copy_music_secondary(&secondary);
        bridge.set_music(0x02, 0x6a);
        bridge.set_music(0x80, 0x7b);
    }

    assert_eq!(config_table.music(0), 0x24);
    assert_eq!(config_table.music(0x02), 0x6a);
    assert_eq!(config_table.music(0x40), 0x46);
    assert_eq!(config_table.music(0x80), 0x7b);
    assert_eq!(config_table.sprite_palette(0x02), 0x05);
    assert_eq!(config_table.sprite_graphics(0x02), 0x18);
    assert_eq!(ram[OVERWORLD_MUSIC_TABLE], 0x24);
    assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x02], 0x6a);
    assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x40], 0x46);
    assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x80], 0x7b);
    assert_eq!(ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02], 0x05);
    assert_eq!(ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02], 0x18);
}

#[test]
fn overworld_map_ui_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0205);
    ram[OVERWORLD_MAP_FLAGS] = 0x81;
    write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0307);
    ram[BIRD_TRAVEL_STATUS + 15] = 0xaa;

    let mut map_ui = OverworldMapUiState::load_from_ram(&ram);
    assert_eq!(map_ui.map_state(), 5);
    assert_eq!(map_ui.map_state_word(), 0x0205);
    assert_eq!(map_ui.map_flags, 0x81);
    assert_eq!(map_ui.birdtravel_status(), 7);
    assert_eq!(map_ui.birdtravel_status_word(), 0x0307);
    assert_eq!(map_ui.bird_travel_statuses.status(15), 0xaa);

    map_ui.map_state = 0x0104;
    map_ui.map_flags = 0x40;
    map_ui.bird_travel_statuses.set_status_word(0x0008);
    map_ui.bird_travel_statuses.set_status(15, 0x55);
    map_ui.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, OVERWORLD_MAP_STATE), 0x0104);
    assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x40);
    assert_eq!(read_le_u16(&ram, BIRDTRAVEL_STATUS), 0x0008);
    assert_eq!(ram[BIRD_TRAVEL_STATUS + 15], 0x55);
}

#[test]
fn native_overworld_map_ui_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0205);
    ram[OVERWORLD_MAP_FLAGS] = 0x81;
    write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0307);
    ram[BIRD_TRAVEL_STATUS + 15] = 0xfe;

    let mut map_ui = OverworldMapUiState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldMapUiBridgeMut::new(&mut map_ui, &mut ram);
        bridge.increment_map_state();
        bridge.and_map_flags(!0x80);
        bridge.or_map_flags(0x02);
        bridge.increment_birdtravel_status();
        bridge.and_birdtravel_status(7);
        bridge.set_birdtravel_status_word(0x0004);
        bridge.increment_bird_travel_stop_status(15);
        bridge.clear_bird_travel_stop_status(1);
    }

    assert_eq!(map_ui.map_state_word(), 0x0206);
    assert_eq!(map_ui.map_flags, 0x03);
    assert_eq!(map_ui.birdtravel_status_word(), 0x0004);
    assert_eq!(map_ui.bird_travel_statuses.status(15), 0xff);
    assert_eq!(map_ui.bird_travel_statuses.status(1), 0);
    assert_eq!(read_le_u16(&ram, OVERWORLD_MAP_STATE), 0x0206);
    assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x03);
    assert_eq!(read_le_u16(&ram, BIRDTRAVEL_STATUS), 0x0004);
    assert_eq!(ram[BIRD_TRAVEL_STATUS + 15], 0xff);
    assert_eq!(ram[BIRD_TRAVEL_STATUS + 1], 0);
}

#[test]
fn compact_world_substates_own_simple_behavior() {
    let mut location = WorldLocationState {
        dungeon_room: 0x1200,
        overworld_screen: 0x3400,
        indoor_flag: 0,
    };
    location.set_dungeon_room_index(0x56);
    assert_eq!(location.increment_dungeon_room_index_by(0x10), 0x66);
    assert_eq!(location.decrement_dungeon_room_index_by(0x20), 0x46);
    location.set_overworld_screen(0x78);
    location.set_indoor_flag(1);
    assert_eq!(location.dungeon_room, 0x1246);
    assert_eq!(location.overworld_screen, 0x3478);
    assert!(location.is_indoors());

    let mut map_ui = OverworldMapUiState::default();
    map_ui.set_map_state_word(0x0205);
    map_ui.increment_map_state();
    map_ui.set_map_flags(0x81);
    map_ui.and_map_flags(!0x80);
    map_ui.or_map_flags(0x02);
    map_ui.set_birdtravel_status_word(0x0307);
    map_ui.increment_birdtravel_status();
    map_ui.and_birdtravel_status(7);
    map_ui.set_birdtravel_status_word(0x0004);
    map_ui.increment_bird_travel_stop_status(15);
    map_ui.clear_bird_travel_stop_status(1);
    assert_eq!(map_ui.map_state_word(), 0x0206);
    assert_eq!(map_ui.map_flags, 0x03);
    assert_eq!(map_ui.birdtravel_status_word(), 0x0004);
    assert_eq!(map_ui.bird_travel_statuses.status(15), 1);
    assert_eq!(map_ui.bird_travel_statuses.status(1), 0);

    let mut weather_vane = WeatherVaneState::default();
    assert_eq!(weather_vane.tick_countdown(), 0xffff);
    weather_vane.set_countdown(0x0280);
    weather_vane.set_music_latch(1);
    weather_vane.set_source_slot(5);
    weather_vane.reset_oam_offset();
    weather_vane.advance_oam_offset(4);
    assert_eq!(weather_vane.countdown, 0x0280);
    assert_eq!(weather_vane.music_latch, 1);
    assert_eq!(weather_vane.source_slot, 5);
    assert_eq!(weather_vane.oam_offset, 4);

    let mut zoom = OverworldMapZoomState::default();
    zoom.set_step_counter(4);
    zoom.decrement_timer();
    zoom.set_timer(12);
    assert_eq!(zoom.step_counter, 4);
    assert_eq!(zoom.timer, 12);

    let mut screen_size = OverworldScreenSizeState {
        big_area: 0x0120,
        big_area_backup: 0x11,
        right_bottom_scroll_bound: 0x02c0,
    };
    screen_size.backup_big_area_low();
    screen_size.clear_big_area_high();
    screen_size.set_big_area_low(0x20);
    screen_size.set_right_bottom_bound_low(0xe4);
    screen_size.set_right_bottom_bound_high(0x01);
    assert_eq!(screen_size.big_area, 0x0020);
    assert_eq!(screen_size.big_area_backup, 0x20);
    assert_eq!(screen_size.right_bottom_scroll_bound, 0x01e4);

    let mut entrance = OverworldEntranceState {
        special_entrance_trigger: 5,
        sequence_counter: 0xff,
    };
    entrance.set_special_entrance_trigger(3);
    assert_eq!(entrance.increment_sequence_counter(), 0);
    assert_eq!(entrance.decrement_sequence_counter(), 0xff);
    entrance.clear_special_entrance_trigger();
    entrance.clear_sequence_counter();
    assert_eq!(entrance.special_entrance_trigger, 0);
    assert_eq!(entrance.sequence_counter, 0);

    let mut exit = OverworldExitState::default();
    exit.set_exit_screen(0x0033);
    exit.set_special_exit_screen(0x0044);
    assert_eq!(exit.exit_screen, 0x0033);
    assert_eq!(exit.special_exit_screen, 0x0044);
}

#[test]
fn weather_vane_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, WEATHERVANE_COUNTDOWN, 0x0280);
    ram[WEATHERVANE_MUSIC_LATCH] = 3;
    ram[WEATHERVANE_SOURCE_SLOT] = 4;
    ram[WEATHERVANE_OAM_OFFSET] = 0x10;

    let mut weather_vane = WeatherVaneState::load_from_ram(&ram);
    assert_eq!(
        weather_vane,
        WeatherVaneState {
            countdown: 0x0280,
            music_latch: 3,
            source_slot: 4,
            oam_offset: 0x10,
        }
    );

    assert_eq!(weather_vane.tick_countdown(), 0x027f);
    weather_vane.reset_oam_offset();
    weather_vane.advance_oam_offset(0xfc);
    weather_vane.music_latch = 1;
    weather_vane.source_slot = 7;
    weather_vane.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, WEATHERVANE_COUNTDOWN), 0x027f);
    assert_eq!(ram[WEATHERVANE_MUSIC_LATCH], 1);
    assert_eq!(ram[WEATHERVANE_SOURCE_SLOT], 7);
    assert_eq!(ram[WEATHERVANE_OAM_OFFSET], 0xfc);
}

#[test]
fn native_weather_vane_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut weather_vane = WeatherVaneState::default();
    {
        let mut bridge = NativeWeatherVaneBridgeMut::new(&mut weather_vane, &mut ram);
        assert_eq!(bridge.tick_countdown(), 0xffff);
        bridge.set_countdown(0x0280);
        bridge.set_music_latch(1);
        bridge.set_source_slot(5);
        bridge.reset_oam_offset();
        bridge.advance_oam_offset(4);
    }

    assert_eq!(
        weather_vane,
        WeatherVaneState {
            countdown: 0x0280,
            music_latch: 1,
            source_slot: 5,
            oam_offset: 4,
        }
    );
    assert_eq!(read_le_u16(&ram, WEATHERVANE_COUNTDOWN), 0x0280);
    assert_eq!(ram[WEATHERVANE_MUSIC_LATCH], 1);
    assert_eq!(ram[WEATHERVANE_SOURCE_SLOT], 5);
    assert_eq!(ram[WEATHERVANE_OAM_OFFSET], 4);
}

#[test]
fn native_weather_vane_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut weather_vane = WeatherVaneState {
        countdown: 0x0102,
        music_latch: 3,
        source_slot: 4,
        oam_offset: 5,
    };
    weather_vane.write_to_ram(&mut ram);

    write_le_u16(&mut ram, WEATHERVANE_COUNTDOWN, 0xaaaa);
    ram[WEATHERVANE_MUSIC_LATCH] = 0xbb;
    ram[WEATHERVANE_SOURCE_SLOT] = 0xcc;
    ram[WEATHERVANE_OAM_OFFSET] = 0xdd;

    {
        let mut bridge = NativeWeatherVaneBridgeMut::new(&mut weather_vane, &mut ram);
        bridge.set_music_latch(7);
    }

    assert_eq!(
        weather_vane,
        WeatherVaneState {
            countdown: 0x0102,
            music_latch: 7,
            source_slot: 4,
            oam_offset: 5,
        }
    );
    assert_eq!(WeatherVaneState::load_from_ram(&ram), weather_vane);
}

#[test]
fn bird_travel_destinations_load_from_and_project_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BIRD_TRAVEL_X_LO + 2] = 0x34;
    ram[BIRD_TRAVEL_X_HI + 2] = 0x12;
    ram[BIRD_TRAVEL_Y_LO + 2] = 0x78;
    ram[BIRD_TRAVEL_Y_HI + 2] = 0x56;

    let mut destinations = BirdTravelDestinationsState::load_from_ram(&ram);
    assert_eq!(
        destinations.destination(2),
        BirdTravelDestinationState {
            x: 0x1234,
            y: 0x5678,
        }
    );
    assert!(!destinations.destination(2).is_empty());
    assert!(destinations.destination(3).is_empty());

    destinations.set_destination(2, 0x2345, 0x6789);
    destinations.clear_destination(3);
    destinations.write_to_ram(&mut ram);

    assert_eq!(ram[BIRD_TRAVEL_X_LO + 2], 0x45);
    assert_eq!(ram[BIRD_TRAVEL_X_HI + 2], 0x23);
    assert_eq!(ram[BIRD_TRAVEL_Y_LO + 2], 0x89);
    assert_eq!(ram[BIRD_TRAVEL_Y_HI + 2], 0x67);
    assert_eq!(ram[BIRD_TRAVEL_X_LO + 3], 0);
    assert_eq!(ram[BIRD_TRAVEL_X_HI + 3], 0);
    assert_eq!(ram[BIRD_TRAVEL_Y_LO + 3], 0);
    assert_eq!(ram[BIRD_TRAVEL_Y_HI + 3], 0);
}

#[test]
fn native_bird_travel_destination_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut destinations = BirdTravelDestinationsState::default();
    {
        let mut bridge = NativeBirdTravelDestinationBridgeMut::new(&mut destinations, &mut ram);
        bridge.set_destination(15, 0x2345, 0x6789);
        bridge.clear_destination(2);
    }

    assert_eq!(
        destinations.destination(15),
        BirdTravelDestinationState {
            x: 0x2345,
            y: 0x6789,
        }
    );
    assert!(destinations.destination(2).is_empty());
    assert_eq!(ram[BIRD_TRAVEL_X_LO + 15], 0x45);
    assert_eq!(ram[BIRD_TRAVEL_X_HI + 15], 0x23);
    assert_eq!(ram[BIRD_TRAVEL_Y_LO + 15], 0x89);
    assert_eq!(ram[BIRD_TRAVEL_Y_HI + 15], 0x67);
    assert_eq!(ram[BIRD_TRAVEL_X_LO + 2], 0);
    assert_eq!(ram[BIRD_TRAVEL_X_HI + 2], 0);
    assert_eq!(ram[BIRD_TRAVEL_Y_LO + 2], 0);
    assert_eq!(ram[BIRD_TRAVEL_Y_HI + 2], 0);
}

#[test]
fn native_bird_travel_destination_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut destinations = BirdTravelDestinationsState::default();
    destinations.set_destination(15, 0x1234, 0x5678);
    destinations.write_to_ram(&mut ram);

    ram[BIRD_TRAVEL_X_LO + 15] = 0xaa;
    ram[BIRD_TRAVEL_X_HI + 15] = 0xbb;
    ram[BIRD_TRAVEL_Y_LO + 15] = 0xcc;
    ram[BIRD_TRAVEL_Y_HI + 15] = 0xdd;

    {
        let mut bridge = NativeBirdTravelDestinationBridgeMut::new(&mut destinations, &mut ram);
        bridge.clear_destination(2);
    }

    assert_eq!(
        destinations.destination(15),
        BirdTravelDestinationState {
            x: 0x1234,
            y: 0x5678,
        }
    );
    assert_eq!(
        BirdTravelDestinationsState::load_from_ram(&ram),
        destinations
    );
}

#[test]
fn overworld_map_zoom_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MODE7_ZOOM_STEP_COUNTER] = 4;
    ram[TIMER_FOR_MODE7_ZOOM] = 12;

    let mut zoom = OverworldMapZoomState::load_from_ram(&ram);
    assert_eq!(zoom.step_counter, 4);
    assert_eq!(zoom.timer, 12);

    zoom.step_counter = 7;
    zoom.timer = 33;
    zoom.write_to_ram(&mut ram);

    assert_eq!(ram[MODE7_ZOOM_STEP_COUNTER], 7);
    assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 33);
}

#[test]
fn native_overworld_map_zoom_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut zoom = OverworldMapZoomState::default();
    {
        let mut bridge = NativeOverworldMapZoomBridgeMut::new(&mut zoom, &mut ram);
        bridge.set_step_counter(4);
        bridge.decrement_timer();
        bridge.set_timer(12);
    }

    assert_eq!(zoom.step_counter, 4);
    assert_eq!(zoom.timer, 12);
    assert_eq!(ram[MODE7_ZOOM_STEP_COUNTER], 4);
    assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 12);
}

#[test]
fn native_overworld_map_zoom_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut zoom = OverworldMapZoomState {
        step_counter: 2,
        timer: 8,
    };
    zoom.write_to_ram(&mut ram);

    ram[MODE7_ZOOM_STEP_COUNTER] = 0xaa;
    ram[TIMER_FOR_MODE7_ZOOM] = 0xbb;

    {
        let mut bridge = NativeOverworldMapZoomBridgeMut::new(&mut zoom, &mut ram);
        bridge.decrement_timer();
    }

    assert_eq!(
        zoom,
        OverworldMapZoomState {
            step_counter: 2,
            timer: 7,
        }
    );
    assert_eq!(OverworldMapZoomState::load_from_ram(&ram), zoom);
}

#[test]
fn overworld_screen_size_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
    ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x20;
    write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x03e4);

    let mut screen_size = OverworldScreenSizeState::load_from_ram(&ram);
    assert_eq!(screen_size.is_big_area_word(), 0x0120);
    assert!(screen_size.is_big_area());
    assert_eq!(screen_size.big_area_backup, 0x20);
    assert_eq!(screen_size.right_bottom_bound_word(), 0x03e4);

    screen_size.big_area = 0x0020;
    screen_size.big_area_backup = 0x20;
    screen_size.right_bottom_scroll_bound = 0x01e4;
    screen_size.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
    assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
    assert_eq!(
        read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
        0x01e4
    );
}

#[test]
fn native_overworld_screen_size_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
    ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x11;
    write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x02c0);

    let mut screen_size = OverworldScreenSizeState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldScreenSizeBridgeMut::new(&mut screen_size, &mut ram);
        bridge.backup_big_area_low();
        bridge.clear_big_area_high();
        bridge.set_big_area_low(0x20);
        bridge.set_right_bottom_bound_low(0xe4);
        bridge.set_right_bottom_bound_high(0x01);
    }

    assert_eq!(screen_size.big_area, 0x0020);
    assert_eq!(screen_size.big_area_backup, 0x20);
    assert_eq!(screen_size.right_bottom_scroll_bound, 0x01e4);
    assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
    assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
    assert_eq!(
        read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
        0x01e4
    );
}

#[test]
fn overworld_scroll_delta_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_SCROLL_DELTA] = 0x11;
    ram[OVERWORLD_SCROLL_DELTA + 1] = 0x22;
    ram[OVERWORLD_SCROLL_DELTA + 2] = 0x33;

    let mut scroll_delta = OverworldScrollDeltaState::load_from_ram(&ram);
    assert_eq!(scroll_delta.vertical_delta_low_byte(), 0x11);
    assert_eq!(scroll_delta.horizontal_delta_low_byte(), 0x22);
    assert_eq!(scroll_delta.vertical_delta_word(), 0x2211);
    assert_eq!(scroll_delta.horizontal_delta_word(), 0x3322);

    scroll_delta.set_vertical_delta_word(0x4433);
    scroll_delta.set_horizontal_delta_word(0x5544);
    scroll_delta.write_to_ram(&mut ram);

    assert_eq!(ram[OVERWORLD_SCROLL_DELTA], 0x33);
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 1], 0x44);
    // write_to_ram owns only 0x69e/0x69f; 0x6a0 (foreign MIRROR_VARS scratch) is left
    // untouched here and written through by the horizontal-word setters instead.
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 2], 0x33);
}

#[test]
fn native_overworld_scroll_delta_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_SCROLL_DELTA] = 0x11;
    ram[OVERWORLD_SCROLL_DELTA + 1] = 0x22;
    ram[OVERWORLD_SCROLL_DELTA + 2] = 0x33;

    let mut scroll_delta = OverworldScrollDeltaState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldScrollDeltaBridgeMut::new(&mut scroll_delta, &mut ram);
        bridge.set_vertical_delta_low_byte(0x44);
        bridge.set_horizontal_delta_low_byte(0x55);
        bridge.set_vertical_delta_word(0x6677);
        bridge.set_horizontal_delta_word(0x8899);
        bridge.clear_vertical_delta_low_byte();
    }

    assert_eq!(scroll_delta.vertical_delta_low_byte(), 0);
    assert_eq!(scroll_delta.horizontal_delta_low_byte(), 0x99);
    assert_eq!(scroll_delta.vertical_delta_word(), 0x9900);
    assert_eq!(scroll_delta.horizontal_delta_word(), 0x8899);
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA], 0);
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 1], 0x99);
    assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 2], 0x88);
}

#[test]
fn overworld_map16_load_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);
    write_le_u16(&mut ram, ORANGE_BLUE_BARRIER_STATE, 0x5678);
    write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, 0x0079);
    write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, 0x000a);

    let mut map16 = OverworldMap16State::load_from_ram(&ram);
    assert_eq!(map16.active_load.src_off, 0x1234);
    assert_eq!(map16.active_load.dst_off, 0x0056);
    assert_eq!(map16.active_load.y_unit, 0x0007);
    assert_eq!(map16.previous_load.src_off, 0x2345);
    assert_eq!(map16.previous_load.dst_off, 0x0067);
    assert_eq!(map16.previous_load.y_unit, 0x0008);
    assert_eq!(map16.special_exit_src_off, 0x3456);
    assert_eq!(map16.exit_src_off, 0x4567);
    assert_eq!(map16.small_scroll_backup.src_off, 0x5678);
    assert_eq!(map16.small_scroll_backup.dst_off, 0x0079);
    assert_eq!(map16.small_scroll_backup.y_unit, 0x000a);

    map16.active_load.src_off = 0x2222;
    map16.active_load.dst_off = 0x0034;
    map16.active_load.y_unit = 0x0009;
    map16.previous_load.src_off = 0x3333;
    map16.previous_load.dst_off = 0x0045;
    map16.previous_load.y_unit = 0x000b;
    map16.special_exit_src_off = 0x4444;
    map16.exit_src_off = 0x5555;
    map16.small_scroll_backup = SmallOverworldMap16ScrollBackupState {
        src_off: 0x6666,
        dst_off: 0x0056,
        y_unit: 0x000c,
    };
    map16.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x2222);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF), 0x0034);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT), 0x0009);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_PREV), 0x3333);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF_PREV), 0x0045);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT_PREV), 0x000b);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_SPEXIT), 0x4444);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_EXIT), 0x5555);
    assert_eq!(read_le_u16(&ram, ORANGE_BLUE_BARRIER_STATE), 0x6666);
    assert_eq!(
        read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
        0x0056
    );
    assert_eq!(
        read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
        0x000c
    );
}

#[test]
fn native_overworld_map16_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);
    write_le_u16(&mut ram, ORANGE_BLUE_BARRIER_STATE, 0x5678);
    write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, 0x0079);
    write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, 0x000a);

    let mut map16 = OverworldMap16State::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldMap16BridgeMut::new(&mut map16, &mut ram);
        bridge.set_active_load(OverworldMap16LoadState {
            src_off: 0x3456,
            dst_off: 0x0078,
            y_unit: 0x000a,
        });
        bridge.set_previous_load(OverworldMap16LoadState {
            src_off: 0x4567,
            dst_off: 0x0089,
            y_unit: 0x000b,
        });
        bridge.set_special_exit_src_off(0x5678);
        bridge.set_exit_src_off(0x6789);
        bridge.set_small_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: 0x789a,
            dst_off: 0x009b,
            y_unit: 0x000c,
        });
    }

    assert_eq!(map16.active_load.src_off, 0x3456);
    assert_eq!(map16.active_load.dst_off, 0x0078);
    assert_eq!(map16.active_load.y_unit, 0x000a);
    assert_eq!(map16.previous_load.src_off, 0x4567);
    assert_eq!(map16.previous_load.dst_off, 0x0089);
    assert_eq!(map16.previous_load.y_unit, 0x000b);
    assert_eq!(map16.special_exit_src_off, 0x5678);
    assert_eq!(map16.exit_src_off, 0x6789);
    assert_eq!(map16.small_scroll_backup.src_off, 0x789a);
    assert_eq!(map16.small_scroll_backup.dst_off, 0x009b);
    assert_eq!(map16.small_scroll_backup.y_unit, 0x000c);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x3456);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF), 0x0078);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT), 0x000a);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_PREV), 0x4567);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF_PREV), 0x0089);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT_PREV), 0x000b);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_SPEXIT), 0x5678);
    assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_EXIT), 0x6789);
    assert_eq!(read_le_u16(&ram, ORANGE_BLUE_BARRIER_STATE), 0x789a);
    assert_eq!(
        read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
        0x009b
    );
    assert_eq!(
        read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
        0x000c
    );
}

#[test]
fn native_overworld_map16_bridge_syncs_from_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut map16 = OverworldMap16State::default();
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
    write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
    write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
    write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);

    {
        let mut bridge = NativeOverworldMap16BridgeMut::new(&mut map16, &mut ram);
        bridge.sync_from_ram();
    }

    assert_eq!(map16.active_load.src_off, 0x1234);
    assert_eq!(map16.active_load.dst_off, 0x0056);
    assert_eq!(map16.active_load.y_unit, 0x0007);
    assert_eq!(map16.previous_load.src_off, 0x2345);
    assert_eq!(map16.previous_load.dst_off, 0x0067);
    assert_eq!(map16.previous_load.y_unit, 0x0008);
    assert_eq!(map16.special_exit_src_off, 0x3456);
    assert_eq!(map16.exit_src_off, 0x4567);
}

#[test]
fn overworld_entrance_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TRIGGER_SPECIAL_ENTRANCE] = 5;
    ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 9;

    let mut entrance = OverworldEntranceState::load_from_ram(&ram);
    assert_eq!(entrance.special_entrance_trigger, 5);
    assert_eq!(entrance.sequence_counter, 9);

    entrance.special_entrance_trigger = 2;
    entrance.sequence_counter = 7;
    entrance.write_to_ram(&mut ram);
    // sequence_counter (0xc8) is reused scratch — not bulk-projected by write_to_ram; it
    // is written through separately (as the bridge sync does).
    entrance.write_sequence_counter_to_ram(&mut ram);

    assert_eq!(ram[TRIGGER_SPECIAL_ENTRANCE], 2);
    assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 7);
}

#[test]
fn native_overworld_entrance_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TRIGGER_SPECIAL_ENTRANCE] = 5;
    ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 0xff;

    let mut entrance = OverworldEntranceState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldEntranceBridgeMut::new(&mut entrance, &mut ram);
        bridge.set_special_entrance_trigger(3);
        assert_eq!(bridge.increment_sequence_counter(), 0);
        assert_eq!(bridge.decrement_sequence_counter(), 0xff);
        bridge.clear_special_entrance_trigger();
        bridge.clear_sequence_counter();
    }

    assert_eq!(entrance.special_entrance_trigger, 0);
    assert_eq!(entrance.sequence_counter, 0);
    assert_eq!(ram[TRIGGER_SPECIAL_ENTRANCE], 0);
    assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 0);
}

#[test]
fn overworld_exit_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0123);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0045);

    let mut exit = OverworldExitState::load_from_ram(&ram);
    assert_eq!(exit.exit_screen, 0x0123);
    assert_eq!(exit.special_exit_screen, 0x0045);

    exit.exit_screen = 0x0067;
    exit.special_exit_screen = 0x0089;
    exit.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0067);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_SPEXIT), 0x0089);
}

#[test]
fn native_overworld_exit_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0111);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0222);

    let mut exit = OverworldExitState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldExitBridgeMut::new(&mut exit, &mut ram);
        bridge.set_exit_screen(0x0033);
        bridge.set_special_exit_screen(0x0044);
    }

    assert_eq!(exit.exit_screen, 0x0033);
    assert_eq!(exit.special_exit_screen, 0x0044);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0033);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_SPEXIT), 0x0044);
}

#[test]
fn overworld_transition_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0302);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0108);
    ram[OVERWORLD_TRANSITION_DIR] = 6;
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANSITION, 0x0203);
    ram[TRANSITION_COUNTER] = 9;
    ram[OW_COUNTDOWN_TRANSITION] = 12;
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV, 0x0004);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV, 0x0002);
    ram[OVERWORLD_SCREEN_TRANSITION_PREV] = 7;

    let mut transition = OverworldTransitionState::load_from_ram(&ram);
    assert_eq!(transition.edge_direction_bits(), 2);
    assert_eq!(transition.edge_direction_bits, 0x0302);
    assert_eq!(transition.direction_bits(), 8);
    assert_eq!(transition.direction_bits_word(), 0x0108);
    assert_eq!(transition.direction_enum(), 6);
    assert!(transition.has_direction_bits());
    assert_eq!(transition.screen_transition(), 3);
    assert_eq!(transition.screen_transition_word(), 0x0203);
    assert_eq!(transition.transition_counter, 9);
    assert_eq!(transition.countdown(), 12);
    assert_eq!(transition.previous_direction_bits, 4);
    assert_eq!(transition.previous_direction_bits2, 2);
    assert_eq!(transition.previous_screen_transition, 7);

    transition.edge_direction_bits = 0x0003;
    transition.direction_bits = 0x0001;
    transition.direction_enum = 4;
    transition.screen_transition = 0x0002;
    transition.transition_counter = 5;
    transition.countdown = 11;
    transition.previous_direction_bits = 0x0008;
    transition.previous_direction_bits2 = 0x0004;
    transition.previous_screen_transition = 6;
    transition.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS), 3);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2), 1);
    assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 4);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANSITION), 2);
    assert_eq!(ram[TRANSITION_COUNTER], 5);
    assert_eq!(ram[OW_COUNTDOWN_TRANSITION], 11);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV), 8);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV), 4);
    assert_eq!(ram[OVERWORLD_SCREEN_TRANSITION_PREV], 6);
}

#[test]
fn overworld_transition_state_owns_direction_and_countdown_behavior() {
    let mut transition = OverworldTransitionState {
        edge_direction_bits: 0x0102,
        direction_bits: 0x0108,
        direction_enum: 6,
        screen_transition: 0x0203,
        transition_counter: 9,
        countdown: 1,
        ..OverworldTransitionState::default()
    };

    transition.and_direction_bits(0x0b);
    transition.or_direction_bits(0x04);
    assert_eq!(transition.or_direction_bits_word(0x0100), 0x010c);
    transition.set_direction_enum(4);
    transition.set_screen_transition(5);
    assert_eq!(transition.increment_transition_counter(), 10);
    assert_eq!(transition.decrement_countdown(), 0);
    transition.set_countdown(12);
    transition.save_previous_direction_bits();
    transition.set_edge_direction_bits(0x04);
    transition.clear_direction_bits_word();
    transition.restore_previous_direction_bits();
    transition.set_previous_screen_transition(6);

    assert_eq!(transition.edge_direction_bits(), 2);
    assert_eq!(transition.edge_direction_bits, 2);
    assert_eq!(transition.direction_bits_word(), 0x010c);
    assert_eq!(transition.direction_enum(), 4);
    assert_eq!(transition.screen_transition_word(), 0x0205);
    assert_eq!(transition.transition_counter, 10);
    assert_eq!(transition.countdown(), 12);
    assert_eq!(transition.previous_direction_bits, 2);
    assert_eq!(transition.previous_direction_bits2, 0x010c);
    assert_eq!(transition.previous_screen_transition, 6);
}

#[test]
fn native_overworld_transition_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0102);
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0108);
    ram[OVERWORLD_TRANSITION_DIR] = 6;
    write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANSITION, 0x0203);
    ram[TRANSITION_COUNTER] = 9;
    ram[OW_COUNTDOWN_TRANSITION] = 1;

    let mut transition = OverworldTransitionState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldTransitionBridgeMut::new(&mut transition, &mut ram);
        bridge.and_direction_bits(0x0b);
        bridge.or_direction_bits(0x04);
        assert_eq!(bridge.or_direction_bits_word(0x0100), 0x010c);
        bridge.set_direction_enum(4);
        bridge.set_screen_transition(5);
        bridge.increment_transition_counter();
        assert_eq!(bridge.decrement_countdown(), 0);
        bridge.set_countdown(12);
        bridge.save_previous_direction_bits();
        bridge.set_edge_direction_bits(0x04);
        bridge.clear_direction_bits_word();
        bridge.restore_previous_direction_bits();
        bridge.set_previous_screen_transition(6);
    }

    assert_eq!(transition.edge_direction_bits(), 2);
    assert_eq!(transition.edge_direction_bits, 2);
    assert_eq!(transition.direction_bits_word(), 0x010c);
    assert_eq!(transition.direction_enum(), 4);
    assert_eq!(transition.screen_transition_word(), 0x0205);
    assert_eq!(transition.transition_counter, 10);
    assert_eq!(transition.countdown(), 12);
    assert_eq!(transition.previous_direction_bits, 2);
    assert_eq!(transition.previous_direction_bits2, 0x010c);
    assert_eq!(transition.previous_screen_transition, 6);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS), 2);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2), 0x010c);
    assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 4);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANSITION), 0x0205);
    assert_eq!(ram[TRANSITION_COUNTER], 10);
    assert_eq!(ram[OW_COUNTDOWN_TRANSITION], 12);
    assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV), 2);
    assert_eq!(
        read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV),
        0x010c
    );
    assert_eq!(ram[OVERWORLD_SCREEN_TRANSITION_PREV], 6);
}

#[test]
fn overworld_palette_backup_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = 0x12;
    ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = 0x34;
    ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = 0x56;

    let mut backup = OverworldPaletteBackupState::load_from_ram(&ram);
    assert_eq!(backup.main_indoors(), 0x12);
    assert_eq!(backup.aux3_bg_palette_7(), 0x34);
    assert_eq!(backup.main_indoors_copy(), 0x56);

    backup.set_main_indoors(0x9a);
    backup.set_aux3_bg_palette_7(0xbc);
    backup.set_main_indoors_copy(0xde);
    backup.write_to_ram(&mut ram);

    assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP], 0x9a);
    assert_eq!(ram[OVERWORLD_PAL_AUX3_BP7_BACKUP], 0xbc);
    assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP], 0xde);
}

#[test]
fn native_overworld_palette_backup_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = 0x12;
    ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = 0x34;
    ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = 0x56;

    let mut backup = OverworldPaletteBackupState::load_from_ram(&ram);
    {
        let mut bridge = NativeOverworldPaletteBackupBridgeMut::new(&mut backup, &mut ram);
        bridge.set_main_indoors_backup(0x9a);
        bridge.set_aux3_bg_palette_7_backup(0xbc);
        bridge.set_main_indoors_copy_backup(0xde);
    }

    assert_eq!(backup.main_indoors(), 0x9a);
    assert_eq!(backup.aux3_bg_palette_7(), 0xbc);
    assert_eq!(backup.main_indoors_copy(), 0xde);
    assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP], 0x9a);
    assert_eq!(ram[OVERWORLD_PAL_AUX3_BP7_BACKUP], 0xbc);
    assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP], 0xde);
}

#[test]
fn native_overworld_palette_backup_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = 0x12;
    ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = 0x34;
    ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = 0x56;
    let mut backup = OverworldPaletteBackupState::default();
    backup.set_main_indoors(0x80);
    backup.set_aux3_bg_palette_7(0x81);
    backup.set_main_indoors_copy(0x82);

    {
        let mut bridge = NativeOverworldPaletteBackupBridgeMut::new(&mut backup, &mut ram);
        bridge.set_main_indoors_backup(0x90);
    }

    assert_eq!(backup.main_indoors(), 0x90);
    assert_eq!(backup.aux3_bg_palette_7(), 0x81);
    assert_eq!(backup.main_indoors_copy(), 0x82);
    assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP], 0x90);
    assert_eq!(ram[OVERWORLD_PAL_AUX3_BP7_BACKUP], 0x81);
    assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP], 0x82);
}

#[test]
fn room_bounds_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ROOM_BOUNDS, 0x0010);
    write_le_u16(&mut ram, ROOM_BOUNDS + 2, 0x0020);
    write_le_u16(&mut ram, ROOM_BOUNDS + 8, 0x0030);
    write_le_u16(&mut ram, ROOM_BOUNDS + 10, 0x0040);

    let mut bounds = RoomBoundsState::load_from_ram(&ram);
    assert_eq!(bounds.y_bound(0), 0x0010);
    assert_eq!(bounds.y_bound(1), 0x0020);
    assert_eq!(bounds.x_bound(0), 0x0030);
    assert_eq!(bounds.x_bound(1), 0x0040);

    bounds.set_y_bound(2, 0x3000);
    bounds.set_x_bound(3, 0x4000);
    bounds.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 4), 0x3000);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 14), 0x4000);
}

#[test]
fn native_room_bounds_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];

    let mut bounds = RoomBoundsState::default();
    bounds.set_y_bound(0, 0x0010);
    bounds.set_y_bound(1, 0x0020);
    bounds.set_y_bound(2, 0x0030);
    bounds.set_y_bound(3, 0x0040);
    bounds.set_x_bound(0, 0x0030);
    bounds.set_x_bound(1, 0x0040);
    bounds.set_x_bound(2, 0x0050);
    bounds.set_x_bound(3, 0x0060);
    bounds.write_to_ram(&mut ram);
    {
        let mut bridge = NativeRoomBoundsBridgeMut::new(&mut bounds, &mut ram);
        bridge.add_y_bounds_a(0x0005);
        bridge.add_x_bounds_b(0x0007);
        bridge.set_y_bound(1, 0x0aaa);
        bridge.set_x_bound(0, 0x0bbb);
        bridge.set_packed_bounds(0x1000, 0x2000, 0x3000, 0x4000);
    }

    assert_eq!(bounds.packed_top(), 0x1000);
    assert_eq!(bounds.packed_bottom(), 0x2000);
    assert_eq!(bounds.packed_left(), 0x3000);
    assert_eq!(bounds.packed_right(), 0x4000);
    assert_eq!(bounds.x_bound(0), 0x0bbb);
    assert_eq!(bounds.x_bound(1), 0x0047);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS), 0x1000);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 2), 0x2000);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 4), 0x3000);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 6), 0x4000);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 8), 0x0bbb);
    assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 10), 0x0047);
}

#[test]
fn native_room_bounds_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut bounds = RoomBoundsState::default();
    bounds.set_y_bound(0, 0x0010);
    bounds.set_y_bound(1, 0x0020);
    bounds.set_y_bound(2, 0x0030);
    bounds.set_y_bound(3, 0x0040);
    bounds.set_x_bound(0, 0x0050);
    bounds.set_x_bound(1, 0x0060);
    bounds.set_x_bound(2, 0x0070);
    bounds.set_x_bound(3, 0x0080);
    bounds.write_to_ram(&mut ram);

    write_le_u16(&mut ram, ROOM_BOUNDS, 0xaaaa);
    write_le_u16(&mut ram, ROOM_BOUNDS + 8, 0xbbbb);

    {
        let mut bridge = NativeRoomBoundsBridgeMut::new(&mut bounds, &mut ram);
        bridge.set_y_bound(1, 0x1234);
    }

    assert_eq!(bounds.y_bound(0), 0x0010);
    assert_eq!(bounds.y_bound(1), 0x1234);
    assert_eq!(bounds.x_bound(0), 0x0050);
    assert_eq!(RoomBoundsState::load_from_ram(&ram), bounds);
}
