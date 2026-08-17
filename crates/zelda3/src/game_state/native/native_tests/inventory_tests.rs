use super::*;

#[test]
fn inventory_items_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_ITEM_BOW] = 7;
    ram[LINK_ITEM_BOW + 2] = 1;
    ram[LINK_ITEM_MOON_PEARL] = 1;
    ram[LINK_BOTTLE_INFO + 2] = 5;

    let items = InventoryItemsState::load_from_ram(&ram);
    assert_eq!(items.bow(), 7);
    assert!(items.has_silver_arrows());
    assert_eq!(items.hookshot(), 1);
    assert!(items.has_moon_pearl());
    assert_eq!(items.bottle(2), 5);

    let mut projected = vec![0; WRAM_SIZE];
    items.write_to_ram(&mut projected);
    assert_eq!(InventoryItemsState::load_from_ram(&projected), items);
}

#[test]
fn native_inventory_items_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_ITEM_MOON_PEARL] = 1;
    ram[LINK_BOTTLE_INFO] = 2;
    ram[LINK_BOTTLE_INFO + 2] = 2;

    let mut items = InventoryItemsState::load_from_ram(&ram);
    {
        let mut bridge = NativeInventoryItemsBridgeMut::new(&mut items, &mut ram);
        bridge.set_inventory_item(0, 3);
        bridge.set_inventory_item(2, 1);
        bridge.set_bottle(0, 5);
        assert!(bridge.fill_first_empty_bottle_with(6));
        assert!(bridge.replace_first_empty_bottle_with(8));
    }

    assert_eq!(items.bow(), 3);
    assert_eq!(items.hookshot(), 1);
    assert!(items.has_moon_pearl());
    assert_eq!(items.bottle(0), 5);
    assert_eq!(items.bottle(1), 6);
    assert_eq!(items.bottle(2), 8);
    assert_eq!(ram[LINK_ITEM_BOW], 3);
    assert_eq!(ram[LINK_ITEM_BOW + 2], 1);
    assert_eq!(ram[LINK_ITEM_MOON_PEARL], 1);
    assert_eq!(ram[LINK_BOTTLE_INFO], 5);
    assert_eq!(ram[LINK_BOTTLE_INFO + 1], 6);
    assert_eq!(ram[LINK_BOTTLE_INFO + 2], 8);
}

#[test]
fn native_inventory_items_bridge_ignores_resource_owned_item_slots_in_coherence_check() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_ITEM_BOMBS] = 2;
    ram[LINK_ITEM_BOTTLE_INDEX] = 0;

    let mut items = InventoryItemsState::load_from_ram(&ram);
    ram[LINK_ITEM_BOMBS] = 3;
    ram[LINK_ITEM_BOTTLE_INDEX] = 1;

    {
        let mut bridge = NativeInventoryItemsBridgeMut::new(&mut items, &mut ram);
        bridge.set_inventory_item(0, 2);
    }

    assert_eq!(ram[LINK_ITEM_BOW], 2);
    assert_eq!(ram[LINK_ITEM_BOMBS], 3);
    assert_eq!(ram[LINK_ITEM_BOTTLE_INDEX], 1);
}

#[test]
fn dungeon_key_slots_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_KEYS_EARNED_PER_DUNGEON] = 1;
    ram[LINK_KEYS_EARNED_PER_DUNGEON + 5] = 6;
    ram[LINK_KEYS_EARNED_PER_DUNGEON + 15] = 16;

    let slots = DungeonKeySlotsState::load_from_ram(&ram);
    assert_eq!(slots.keys_earned(0), 1);
    assert_eq!(slots.keys_earned(10), 6);
    assert_eq!(slots.keys_earned_slot(15), 16);
    assert_eq!(slots.keys_earned_slot(16), 0);

    let mut projected = vec![0; WRAM_SIZE];
    slots.write_to_ram(&mut projected);
    assert_eq!(projected[LINK_KEYS_EARNED_PER_DUNGEON], 1);
    assert_eq!(projected[LINK_KEYS_EARNED_PER_DUNGEON + 5], 6);
    assert_eq!(projected[LINK_KEYS_EARNED_PER_DUNGEON + 15], 16);
}

#[test]
fn native_dungeon_key_slots_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[LINK_KEYS_EARNED_PER_DUNGEON + 2] = 3;

    let mut slots = DungeonKeySlotsState::load_from_ram(&ram);
    {
        let mut bridge = NativeDungeonKeySlotsBridgeMut::new(&mut slots, &mut ram);
        bridge.set_keys_earned(4, 7);
        bridge.set_keys_earned_slot(5, 9);
        bridge.set_keys_earned_slot(16, 11);
    }

    assert_eq!(slots.keys_earned(4), 7);
    assert_eq!(slots.keys_earned_slot(5), 9);
    assert_eq!(slots.keys_earned_slot(16), 0);
    assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 2], 7);
    assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 5], 9);
    assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 15], 0);
}

#[test]
fn native_dungeon_key_slots_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0xff; WRAM_SIZE];
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[LINK_KEYS_EARNED_PER_DUNGEON + 2] = 3;
    native_ram[LINK_KEYS_EARNED_PER_DUNGEON + 5] = 6;
    let mut slots = DungeonKeySlotsState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDungeonKeySlotsBridgeMut::new(&mut slots, &mut ram);
        bridge.set_keys_earned_slot(5, 9);
    }

    assert_eq!(slots.keys_earned_slot(2), 3);
    assert_eq!(slots.keys_earned_slot(5), 9);
    assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 2], 3);
    assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 5], 9);
    assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 15], 0);
}

#[test]
fn mirror_warp_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MIRROR_WARP_TARGET_INDEX, 2);
    write_le_u16(&mut ram, MIRROR_WARP_TARGET_OFFSETS, 0xfe00);
    write_le_u16(&mut ram, MIRROR_WARP_TARGET_OFFSETS + 2, 0x0200);
    write_le_u16(&mut ram, MIRROR_WARP_VELOCITY_DELTAS, 0xffc0);
    write_le_u16(&mut ram, MIRROR_WARP_VELOCITY_DELTAS + 2, 0x0040);
    write_le_u16(&mut ram, MIRROR_WARP_WAVE_OFFSET, 0x0012);
    write_le_u16(&mut ram, MIRROR_WARP_DISPLACEMENT, 0x0034);
    write_le_u16(&mut ram, MIRROR_WARP_SUBPIXEL, 0x0056);
    ram[MIRROR_WARP_LOAD_STEP_COUNTER] = 7;
    ram[MIRROR_WARP_ANIMATION_COUNTER] = 8;

    let mut mirror = MirrorWarpState::load_from_ram(&ram);
    assert_eq!(mirror.target_index(), 1);
    assert_eq!(mirror.target_offset(), 0x0200);
    assert_eq!(mirror.velocity_delta(), 0x0040);
    assert_eq!(mirror.wave_offset(), 0x0012);
    assert_eq!(mirror.displacement(), 0x0034);
    assert_eq!(mirror.subpixel(), 0x0056);
    assert_eq!(mirror.animation_counter(), 8);

    mirror.reset_wave_and_subpixel();
    mirror.toggle_target_index();
    mirror.set_displacement(0x0078);
    mirror.set_subpixel_low_from(0x019a);
    mirror.set_wave_offset(0x00bc);
    mirror.shrink_target_offsets_for_dewaving();
    assert_eq!(mirror.increment_load_step_counter(), 8);
    assert_eq!(mirror.decrement_animation_counter(), 7);
    mirror.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_INDEX), 0);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS), 0xff00);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS + 2), 0x0100);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_WAVE_OFFSET), 0x00bc);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_DISPLACEMENT), 0x0078);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_SUBPIXEL), 0x009a);
    assert_eq!(ram[MIRROR_WARP_LOAD_STEP_COUNTER], 8);
    assert_eq!(ram[MIRROR_WARP_ANIMATION_COUNTER], 7);
}

#[test]
fn native_mirror_warp_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MIRROR_WARP_TARGET_INDEX, 2);
    ram[MIRROR_WARP_LOAD_STEP_COUNTER] = 0xff;
    ram[MIRROR_WARP_ANIMATION_COUNTER] = 0;

    let mut mirror = MirrorWarpState::load_from_ram(&ram);
    {
        let mut bridge = NativeMirrorWarpBridgeMut::new(&mut mirror, &mut ram);
        bridge.initialize_hdma_wave_state();
        bridge.toggle_target_index();
        bridge.set_displacement(0x0044);
        bridge.set_subpixel_low_from(0x0166);
        bridge.set_wave_offset(0x0088);
        bridge.shrink_target_offsets_for_dewaving();
        assert_eq!(bridge.increment_load_step_counter(), 0);
        bridge.reset_load_step_counter();
        bridge.set_animation_counter(2);
        assert_eq!(bridge.decrement_animation_counter(), 1);
    }

    assert_eq!(mirror.target_index(), 1);
    assert_eq!(mirror.target_offset(), 0x0100);
    assert_eq!(mirror.velocity_delta(), 0x0040);
    assert_eq!(mirror.wave_offset(), 0x0088);
    assert_eq!(mirror.displacement(), 0x0044);
    assert_eq!(mirror.subpixel(), 0x0066);
    assert_eq!(mirror.animation_counter(), 1);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_INDEX), 2);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS), 0xff00);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS + 2), 0x0100);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_WAVE_OFFSET), 0x0088);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_DISPLACEMENT), 0x0044);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_SUBPIXEL), 0x0066);
    assert_eq!(ram[MIRROR_WARP_LOAD_STEP_COUNTER], 0);
    assert_eq!(ram[MIRROR_WARP_ANIMATION_COUNTER], 1);
}

#[test]
fn native_mirror_warp_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0xff; WRAM_SIZE];
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, MIRROR_WARP_TARGET_INDEX, 2);
    write_le_u16(&mut native_ram, MIRROR_WARP_WAVE_OFFSET, 0x0012);
    write_le_u16(&mut native_ram, MIRROR_WARP_DISPLACEMENT, 0x0034);
    write_le_u16(&mut native_ram, MIRROR_WARP_SUBPIXEL, 0x0056);
    native_ram[MIRROR_WARP_LOAD_STEP_COUNTER] = 7;
    native_ram[MIRROR_WARP_ANIMATION_COUNTER] = 8;
    let mut mirror = MirrorWarpState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeMirrorWarpBridgeMut::new(&mut mirror, &mut ram);
        bridge.toggle_target_index();
        bridge.set_wave_offset(0x009a);
        bridge.set_displacement(0x00bc);
        assert_eq!(bridge.increment_load_step_counter(), 8);
        assert_eq!(bridge.decrement_animation_counter(), 7);
    }

    assert_eq!(mirror.target_index(), 0);
    assert_eq!(mirror.wave_offset(), 0x009a);
    assert_eq!(mirror.displacement(), 0x00bc);
    assert_eq!(mirror.subpixel(), 0x0056);
    assert_eq!(mirror.animation_counter(), 7);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_INDEX), 0);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_WAVE_OFFSET), 0x009a);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_DISPLACEMENT), 0x00bc);
    assert_eq!(read_le_u16(&ram, MIRROR_WARP_SUBPIXEL), 0x0056);
    assert_eq!(ram[MIRROR_WARP_LOAD_STEP_COUNTER], 8);
    assert_eq!(ram[MIRROR_WARP_ANIMATION_COUNTER], 7);
}

#[test]
fn save_progress_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[CUR_PALACE_INDEX_X2] = 10;
    ram[SRAM_PROGRESS_INDICATOR] = 2;
    ram[SRAM_PROGRESS_FLAGS] = 0x40;
    ram[SAVEGAME_MAP_ICONS_INDICATOR] = 7;
    ram[WHICH_STARTING_POINT] = 3;
    ram[SRAM_PROGRESS_INDICATOR_3] = 0x20;
    ram[SAVEGAME_IS_DARKWORLD] = 0x40;
    ram[HUD_CUR_ITEM] = 1;
    ram[HUD_CUR_ITEM_X] = 2;
    ram[HUD_CUR_ITEM_L] = 3;
    ram[HUD_CUR_ITEM_R] = 4;
    write_le_u16(&mut ram, SAVE_DUNG_INFO + 0x109 * 2, 0x0080);
    write_le_u16(&mut ram, DEATHS_PER_PALACE + 4 * 2, 0x0012);
    write_le_u16(&mut ram, PENDING_DEATH_SAVE_COUNTER, 0x0034);
    write_le_u16(&mut ram, TOTAL_DEATH_SAVE_COUNTER, 0xffff);
    ram[HUD_POST_MESSAGE_REFRESH_FLAG] = 0x80;

    let mut progress = SaveProgressState::load_from_ram(&ram);
    assert_eq!(progress.palace_index_x2(), 10);
    assert_eq!(progress.palace_index(), 5);
    assert_eq!(progress.progress_indicator_word(), 0x4002);
    assert!(progress.progress_flags_has(0x40));
    assert_eq!(progress.map_icons_indicator(), 7);
    assert_eq!(progress.dark_world_bit6(), 1);
    assert_eq!(progress.hud_current_item(), 1);
    assert_eq!(progress.hud_current_item_slot(3), 4);
    assert_eq!(progress.dungeon_info_word(0x109), 0x0080);
    assert_eq!(progress.death_count_for_palace(4), 0x0012);
    assert_eq!(progress.pending_death_save_counter(), 0x0034);
    assert!(progress.total_death_save_counter_is_uninitialized());
    assert_eq!(progress.which_starting_point(), 3);
    assert_eq!(progress.progress_indicator_3(), 0x20);

    progress.xor_palace_index_x2(2);
    progress.or_progress_flags(1);
    progress.clear_progress_indicator_3_bits(0x20);
    progress.xor_dark_world_state(0x40);
    progress.set_hud_current_item_slot(2, 9);
    progress.or_dungeon_info_word(0x109, 0x0100);
    progress.set_dungeon_info_checksum(0x5a5a);
    progress.increment_pending_death_save_counter();
    progress.set_total_death_save_counter(0x0045);
    progress.write_to_ram(&mut ram);
    // The 0xf000..0xf500 save block is write-through, not bulk-projected, so the
    // block-backed fields flush through the explicit block writer rather than write_to_ram.
    progress.write_dungeon_info_to_ram(&mut ram);

    assert_eq!(ram[CUR_PALACE_INDEX_X2], 8);
    assert_eq!(ram[SRAM_PROGRESS_FLAGS], 0x41);
    assert_eq!(ram[SRAM_PROGRESS_INDICATOR_3], 0);
    assert_eq!(ram[SAVEGAME_IS_DARKWORLD], 0);
    assert_eq!(ram[HUD_CUR_ITEM_L], 9);
    assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 0x109 * 2), 0x0180);
    assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 0x4fe), 0x5a5a);
    assert_eq!(read_le_u16(&ram, PENDING_DEATH_SAVE_COUNTER), 0x0035);
    assert_eq!(read_le_u16(&ram, TOTAL_DEATH_SAVE_COUNTER), 0x0045);
}

#[test]
fn native_save_progress_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[SRAM_PROGRESS_FLAGS] = 0x10;
    ram[SRAM_PROGRESS_INDICATOR_3] = 0xff;
    ram[HUD_CUR_ITEM] = 1;
    write_le_u16(&mut ram, SAVE_DUNG_INFO + 2, 0x0001);

    let mut progress = SaveProgressState::load_from_ram(&ram);
    {
        let mut bridge = NativeSaveProgressBridgeMut::new(&mut progress, &mut ram);
        bridge.set_palace_index_x2(0xff);
        bridge.xor_palace_index_x2(1);
        bridge.set_which_starting_point(5);
        bridge.set_progress_indicator(3);
        bridge.or_progress_flags(0x20);
        bridge.set_progress_flags(0x22);
        bridge.or_progress_indicator_3(0x01);
        bridge.clear_progress_indicator_3_bits(0xf0);
        bridge.set_map_icons_indicator(6);
        bridge.set_dark_world_state(0x40);
        bridge.xor_dark_world_state(0x40);
        bridge.set_hud_current_item(2);
        bridge.set_hud_current_item_slot(3, 7);
        bridge.set_death_count_for_palace(1, 0x0044);
        assert_eq!(bridge.increment_pending_death_save_counter(), 1);
        bridge.clear_pending_death_save_counter();
        bridge.set_total_death_save_counter(0x0055);
        bridge.request_post_message_refresh();
        assert_eq!(bridge.or_dungeon_info_word(1, 0x0100), 0x0101);
        bridge.set_dungeon_info_checksum(0x1234);
        bridge.clear_post_message_refresh_flag();
    }

    assert_eq!(progress.palace_index_x2(), 0xfe);
    assert_eq!(progress.which_starting_point(), 5);
    assert_eq!(progress.progress_indicator(), 3);
    assert_eq!(progress.progress_flags(), 0x22);
    assert_eq!(progress.progress_indicator_3(), 0x0f);
    assert_eq!(progress.map_icons_indicator(), 6);
    assert_eq!(progress.dark_world_state(), 0);
    assert_eq!(progress.hud_current_item(), 2);
    assert_eq!(progress.hud_current_item_slot(3), 7);
    assert_eq!(progress.death_count_for_palace(1), 0x0044);
    assert_eq!(progress.pending_death_save_counter(), 0);
    assert_eq!(progress.total_death_save_counter(), 0x0055);
    assert_eq!(progress.dungeon_info_word(1), 0x0101);
    assert_eq!(ram[CUR_PALACE_INDEX_X2], 0xfe);
    assert_eq!(ram[WHICH_STARTING_POINT], 5);
    assert_eq!(ram[SRAM_PROGRESS_INDICATOR], 3);
    assert_eq!(ram[SRAM_PROGRESS_FLAGS], 0x22);
    assert_eq!(ram[SRAM_PROGRESS_INDICATOR_3], 0x0f);
    assert_eq!(ram[SAVEGAME_MAP_ICONS_INDICATOR], 6);
    assert_eq!(ram[SAVEGAME_IS_DARKWORLD], 0);
    assert_eq!(ram[HUD_CUR_ITEM], 2);
    assert_eq!(ram[HUD_CUR_ITEM_R], 7);
    assert_eq!(read_le_u16(&ram, DEATHS_PER_PALACE + 2), 0x0044);
    assert_eq!(read_le_u16(&ram, PENDING_DEATH_SAVE_COUNTER), 0);
    assert_eq!(read_le_u16(&ram, TOTAL_DEATH_SAVE_COUNTER), 0x0055);
    assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 2), 0x0101);
    assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 0x4fe), 0x1234);
    assert_eq!(ram[HUD_POST_MESSAGE_REFRESH_FLAG], 0);
}

#[test]
fn native_save_progress_bridge_composes_edits_onto_live_ram() {
    // The 0xf000..0xf500 save block is owned and written live by the inventory / player /
    // follower / overworld-event natives, exactly as C writes it straight into the SRAM
    // mirror, so the bridge must compose its edits onto whatever is in RAM now — never
    // re-stamp a stale frame-start snapshot over a live write.
    let mut ram = vec![0; WRAM_SIZE];
    ram[CUR_PALACE_INDEX_X2] = 10;
    ram[SRAM_PROGRESS_FLAGS] = 0x10;
    ram[SRAM_PROGRESS_INDICATOR_3] = 0xff;
    ram[HUD_CUR_ITEM] = 1;
    write_le_u16(&mut ram, SAVE_DUNG_INFO + 2, 0x0001);

    // A deliberately stale native snapshot: every field disagrees with live RAM.
    let mut progress = SaveProgressState::load_from_ram(&vec![0xa5; WRAM_SIZE]);

    {
        let mut bridge = NativeSaveProgressBridgeMut::new(&mut progress, &mut ram);
        bridge.xor_palace_index_x2(2);
        bridge.or_progress_flags(0x20);
        bridge.clear_progress_indicator_3_bits(0xf0);
        bridge.set_hud_current_item(2);
        // 0x0001 is RAM's live value; the stale snapshot held 0xa5a5.
        assert_eq!(bridge.or_dungeon_info_word(1, 0x0100), 0x0101);
    }

    // Each edit landed on the live RAM value, and the stale snapshot was discarded.
    assert_eq!(progress.palace_index_x2(), 8);
    assert_eq!(progress.progress_flags(), 0x30);
    assert_eq!(progress.progress_indicator_3(), 0x0f);
    assert_eq!(progress.hud_current_item(), 2);
    assert_eq!(progress.dungeon_info_word(1), 0x0101);

    // ...and every edit still wrote through to RAM.
    assert_eq!(ram[CUR_PALACE_INDEX_X2], 8);
    assert_eq!(ram[SRAM_PROGRESS_FLAGS], 0x30);
    assert_eq!(ram[SRAM_PROGRESS_INDICATOR_3], 0x0f);
    assert_eq!(ram[HUD_CUR_ITEM], 2);
    assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 2), 0x0101);
}

#[test]
fn save_progress_projection_leaves_the_live_save_block_alone() {
    // Regression: SaveProgressState::write_to_ram used to bulk-copy its 0x500-byte cache
    // over 0xf000..0xf500. Because `inventory` projects after `player` and `save_progress`
    // after `player_resources`, that snapshot became the last writer for every live
    // inventory byte in the block. It must no longer touch them.
    let mut ram = vec![0; WRAM_SIZE];
    let mut progress = SaveProgressState::load_from_ram(&ram);

    // Another native writes live values into the block after the snapshot was taken.
    ram[LINK_MAGIC_POWER] = 0x50;
    ram[LINK_NUM_KEYS] = 3;
    ram[LINK_ABILITY_FLAGS] = 0x04;
    write_le_u16(&mut ram, LINK_RUPEES_GOAL, 0x0123);

    progress.set_palace_index_x2(6);
    progress.write_to_ram(&mut ram);

    assert_eq!(ram[CUR_PALACE_INDEX_X2], 6, "own bytes still project");
    assert_eq!(ram[LINK_MAGIC_POWER], 0x50);
    assert_eq!(ram[LINK_NUM_KEYS], 3);
    assert_eq!(ram[LINK_ABILITY_FLAGS], 0x04);
    assert_eq!(read_le_u16(&ram, LINK_RUPEES_GOAL), 0x0123);
}
