use super::*;

#[test]
fn enhanced_features_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[ENHANCED_FEATURE_FLAGS] = 0x78;
    ram[ENHANCED_FEATURE_FLAGS + 1] = 0x56;
    ram[ENHANCED_FEATURE_FLAGS + 2] = 0x34;
    ram[ENHANCED_FEATURE_FLAGS + 3] = 0x12;

    let features = EnhancedFeaturesState::load_from_ram(&ram);
    assert_eq!(features.bits(), 0x1234_5678);
    assert!(features.has(0x1000_0000));
    assert!(!features.is_empty());

    let mut projected = vec![0; WRAM_SIZE];
    features.write_to_ram(&mut projected);
    assert_eq!(
        &projected[ENHANCED_FEATURE_FLAGS..ENHANCED_FEATURE_FLAGS + 4],
        &[0x78, 0x56, 0x34, 0x12]
    );
}

#[test]
fn native_enhanced_features_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ENHANCED_FEATURE_FLAGS, 0x1000);

    let mut features = EnhancedFeaturesState::load_from_ram(&ram);
    {
        let mut bridge = NativeEnhancedFeaturesBridgeMut::new(&mut features, &mut ram);
        bridge.set_bits(0x1234_5678);
    }

    assert_eq!(features.bits(), 0x1234_5678);
    assert_eq!(read_le_u16(&ram, ENHANCED_FEATURE_FLAGS), 0x5678);
    assert_eq!(read_le_u16(&ram, ENHANCED_FEATURE_FLAGS + 2), 0x1234);
}

#[test]
fn native_enhanced_features_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ENHANCED_FEATURE_FLAGS, 0x1000);
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[ENHANCED_FEATURE_FLAGS] = 0x04;
    let mut features = EnhancedFeaturesState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeEnhancedFeaturesBridgeMut::new(&mut features, &mut ram);
        bridge.set_bits(0x0000_0008);
    }

    assert_eq!(features.bits(), 0x0000_0008);
    assert_eq!(read_le_u16(&ram, ENHANCED_FEATURE_FLAGS), 0x0008);
}

#[test]
fn scratch_counter_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TEMP_COUNTER] = 0x80;

    let mut counter = ScratchCounterState::load_from_ram(&ram);
    assert_eq!(counter.value(), 0x80);
    assert_eq!(counter.as_usize(), 0x80);
    assert!(counter.is_negative());
    counter.set(2);
    assert_eq!(counter.decrement(), 1);
    counter.write_to_ram(&mut ram);

    assert_eq!(ram[TEMP_COUNTER], 1);
}

#[test]
fn native_scratch_counter_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TEMP_COUNTER] = 0;

    let mut counter = ScratchCounterState::load_from_ram(&ram);
    {
        let mut bridge = NativeScratchCounterBridgeMut::new(&mut counter, &mut ram);
        assert_eq!(bridge.decrement(), 0xff);
        bridge.set(7);
    }

    assert_eq!(counter.value(), 7);
    assert_eq!(ram[TEMP_COUNTER], 7);
}

#[test]
fn native_scratch_counter_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TEMP_COUNTER] = 0x80;
    let mut counter = ScratchCounterState::default();
    counter.set(3);

    {
        let mut bridge = NativeScratchCounterBridgeMut::new(&mut counter, &mut ram);
        assert_eq!(bridge.decrement(), 2);
    }

    assert_eq!(counter.value(), 2);
    assert_eq!(ram[TEMP_COUNTER], 2);
}

#[test]
fn memorized_tile_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 4);
    write_le_u16(&mut ram, MEMORIZED_TILE_ADDR, 0x1111);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE, 0x2222);
    write_le_u16(&mut ram, MEMORIZED_TILE_ADDR + 2, 0x3333);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE + 2, 0x4444);

    let mut memorized_tiles = MemorizedTileState::load_from_ram(&ram);
    assert_eq!(memorized_tiles.count(), 4);
    assert_eq!(memorized_tiles.entry_addr(0), 0x1111);
    assert_eq!(memorized_tiles.entry_value(0), 0x2222);
    assert_eq!(memorized_tiles.entry_addr(1), 0x3333);
    assert_eq!(memorized_tiles.entry_value(1), 0x4444);
    assert_eq!(memorized_tiles.entry_addr(0x80), 0);

    memorized_tiles.append_entry(0x5555, 0x6666);
    memorized_tiles.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 6);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 4), 0x5555);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 4), 0x6666);
}

#[test]
fn native_memorized_tile_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 2);
    write_le_u16(&mut ram, MEMORIZED_TILE_ADDR, 0x1111);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE, 0x2222);
    // last slot of the C-sized 0x20-entry table (offset 0x3e = slot 31)
    write_le_u16(&mut ram, MEMORIZED_TILE_ADDR + 0x3e, 0xffff);

    let mut memorized_tiles = MemorizedTileState::load_from_ram(&ram);
    {
        let mut bridge = NativeMemorizedTileBridgeMut::new(&mut memorized_tiles, &mut ram);
        bridge.append_entry(0x3333, 0x4444);
        bridge.set_entry_addr(4, 0x5555);
        bridge.set_entry_value(4, 0x6666);
        bridge.set_count(6);
        bridge.clear_entry_addresses();
    }

    assert_eq!(memorized_tiles.count(), 6);
    assert_eq!(memorized_tiles.entry_addr(0), 0);
    assert_eq!(memorized_tiles.entry_addr(1), 0);
    assert_eq!(memorized_tiles.entry_addr(2), 0);
    assert_eq!(memorized_tiles.entry_value(1), 0x4444);
    assert_eq!(memorized_tiles.entry_value(2), 0x6666);
    assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 6);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR), 0);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 0x3e), 0);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 2), 0x4444);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 4), 0x6666);
}

#[test]
fn memorized_tile_state_projects_only_source_owned_value_extent() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 0x40);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE + 0x40, 0xaaaa);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE + 0x42, 0xbbbb);

    let mut memorized_tiles = MemorizedTileState::load_from_ram(&ram);
    memorized_tiles.write_to_ram(&mut ram);

    // count=0x40 owns slots 0..31. The contextual aliases beginning at
    // $7efa40 are untouched until the source actually appends slot 32.
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 0x40), 0xaaaa);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 0x42), 0xbbbb);

    {
        let mut bridge = NativeMemorizedTileBridgeMut::new(&mut memorized_tiles, &mut ram);
        bridge.append_entry(0x1234, 0x5678);
    }

    assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 0x42);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 0x40), 0x1234);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 0x40), 0x5678);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 0x42), 0xbbbb);
}

#[test]
fn memorized_tile_address_clear_matches_the_source_0x100_byte_memset() {
    let mut ram = vec![0xff; WRAM_SIZE];
    write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 0);
    let mut memorized_tiles = MemorizedTileState::load_from_ram(&ram);

    {
        let mut bridge = NativeMemorizedTileBridgeMut::new(&mut memorized_tiles, &mut ram);
        bridge.clear_entry_addresses();
    }

    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR), 0);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 0xfe), 0);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 0x100), 0xffff);
}

#[test]
fn native_memorized_tile_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 2);
    write_le_u16(&mut ram, MEMORIZED_TILE_ADDR, 0x1111);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE, 0x2222);
    let mut memorized_tiles = MemorizedTileState::default();
    memorized_tiles.set_count(2);
    memorized_tiles.set_entry_addr(0, 0x3333);
    memorized_tiles.set_entry_value(0, 0x4444);

    {
        let mut bridge = NativeMemorizedTileBridgeMut::new(&mut memorized_tiles, &mut ram);
        bridge.append_entry(0x5555, 0x6666);
    }

    assert_eq!(memorized_tiles.count(), 4);
    assert_eq!(memorized_tiles.entry_addr(0), 0x3333);
    assert_eq!(memorized_tiles.entry_value(0), 0x4444);
    assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 4);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR), 0x3333);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE), 0x4444);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 2), 0x5555);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 2), 0x6666);
}

#[test]
fn native_memorized_tile_bridge_ignores_indoor_value_table_reuse_in_coherence_check() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[PLAYER_IS_INDOORS] = 1;
    write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 0);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE, 0x44a8);
    write_le_u16(&mut ram, MEMORIZED_TILE_VALUE + 2, 0x4b1e);

    let mut memorized_tiles = MemorizedTileState::default();
    {
        let mut bridge = NativeMemorizedTileBridgeMut::new(&mut memorized_tiles, &mut ram);
        bridge.clear_entry_addresses();
    }

    assert_eq!(memorized_tiles.count(), 0);
    assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 0);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE), 0x44a8);
    assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 2), 0x4b1e);
}

#[test]
fn minigame_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[IS_ARCHER_OR_SHOVEL_GAME] = 2;
    ram[MINIGAME_CREDITS] = 3;
    ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 1;
    write_le_u16(&mut ram, BOOMERANG_TEMP_X, 0x1234);
    write_le_u16(&mut ram, BOOMERANG_TEMP_Y, 0xabcd);

    let mut minigame = MinigameState::load_from_ram(&ram);
    assert_eq!(minigame.is_archer_or_shovel_game(), 2);
    assert_eq!(minigame.credits(), 3);
    assert_eq!(minigame.flag_boomerang_in_place(), 1);
    assert_eq!(minigame.boomerang_temp_x(), 0x1234);
    assert_eq!(minigame.boomerang_temp_y(), 0xabcd);

    minigame.clear_is_archer_or_shovel_game();
    minigame.decrement_credits();
    minigame.clear_flag_boomerang_in_place();
    minigame.set_boomerang_temp_x(0x4567);
    minigame.set_boomerang_temp_y(0xcdef);
    minigame.write_to_ram(&mut ram);

    assert_eq!(ram[IS_ARCHER_OR_SHOVEL_GAME], 0);
    assert_eq!(ram[MINIGAME_CREDITS], 2);
    assert_eq!(ram[FLAG_FOR_BOOMERANG_IN_PLACE], 0);
    assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_X), 0x4567);
    assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_Y), 0xcdef);
}

#[test]
fn native_minigame_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[IS_ARCHER_OR_SHOVEL_GAME] = 2;
    ram[MINIGAME_CREDITS] = 3;
    ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 1;
    write_le_u16(&mut ram, BOOMERANG_TEMP_X, 0x1234);
    write_le_u16(&mut ram, BOOMERANG_TEMP_Y, 0xabcd);

    let mut minigame = MinigameState::load_from_ram(&ram);
    {
        let mut bridge = NativeMinigameBridgeMut::new(&mut minigame, &mut ram);
        bridge.clear_is_archer_or_shovel_game();
        bridge.set_credits(5);
        bridge.decrement_credits();
        bridge.clear_flag_boomerang_in_place();
        bridge.set_boomerang_temp_x(0x4567);
        bridge.set_boomerang_temp_y(0xcdef);
    }

    assert_eq!(minigame.is_archer_or_shovel_game(), 0);
    assert_eq!(minigame.credits(), 4);
    assert_eq!(minigame.flag_boomerang_in_place(), 0);
    assert_eq!(minigame.boomerang_temp_x(), 0x4567);
    assert_eq!(minigame.boomerang_temp_y(), 0xcdef);
    assert_eq!(ram[IS_ARCHER_OR_SHOVEL_GAME], 0);
    assert_eq!(ram[MINIGAME_CREDITS], 4);
    assert_eq!(ram[FLAG_FOR_BOOMERANG_IN_PLACE], 0);
    assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_X), 0x4567);
    assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_Y), 0xcdef);
}

#[test]
fn native_minigame_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MINIGAME_CREDITS] = 0xff;
    write_le_u16(&mut ram, BOOMERANG_TEMP_X, 0x1111);
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[IS_ARCHER_OR_SHOVEL_GAME] = 2;
    native_ram[MINIGAME_CREDITS] = 3;
    write_le_u16(&mut native_ram, BOOMERANG_TEMP_X, 0x2222);
    write_le_u16(&mut native_ram, BOOMERANG_TEMP_Y, 0x3333);
    let mut minigame = MinigameState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeMinigameBridgeMut::new(&mut minigame, &mut ram);
        assert_eq!(bridge.decrement_credits(), 2);
    }

    assert_eq!(minigame.is_archer_or_shovel_game(), 2);
    assert_eq!(minigame.credits(), 2);
    assert_eq!(minigame.boomerang_temp_x(), 0x2222);
    assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_X), 0x2222);
    assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_Y), 0x3333);
    assert_eq!(ram[MINIGAME_CREDITS], 2);
}

#[test]
fn intro_sword_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, INTRO_SWORD_YPOS, 0x1234);
    ram[INTRO_SWORD_SPARKLE_TIMER] = 5;
    ram[INTRO_SWORD_SPARKLE_STEP] = 1;
    ram[INTRO_SWORD_ANIM_STEP] = 4;
    ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = 7;
    write_le_u16(&mut ram, INTRO_SWORD_FLASH_RGB_CHANNEL, 0xab02);

    let mut intro_sword = IntroSwordState::load_from_ram(&ram);
    assert_eq!(intro_sword.ypos(), 0x1234);
    assert_eq!(intro_sword.sparkle_timer(), 5);
    assert_eq!(intro_sword.sparkle_step(), 1);
    assert_eq!(intro_sword.anim_phase(), 2);
    assert_eq!(intro_sword.anim_step_raw(), 4);
    assert_eq!(intro_sword.sparkle_y_offset(), 7);
    assert_eq!(intro_sword.flash_rgb_channel(), 2);

    intro_sword.advance_ypos();
    intro_sword.decrement_sparkle_timer();
    intro_sword.advance_anim_step();
    intro_sword.advance_sparkle_y_offset();
    intro_sword.cycle_flash_rgb_channel();
    intro_sword.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, INTRO_SWORD_YPOS), 0x1244);
    assert_eq!(ram[INTRO_SWORD_SPARKLE_TIMER], 4);
    assert_eq!(ram[INTRO_SWORD_ANIM_STEP], 6);
    assert_eq!(ram[INTRO_SWORD_SPARKLE_Y_OFFSET], 11);
    assert_eq!(read_le_u16(&ram, INTRO_SWORD_FLASH_RGB_CHANNEL), 0xab00);
}

#[test]
fn native_intro_sword_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, INTRO_SWORD_YPOS, 0x1234);
    ram[INTRO_SWORD_SPARKLE_TIMER] = 5;
    ram[INTRO_SWORD_SPARKLE_STEP] = 0;
    ram[INTRO_SWORD_ANIM_STEP] = 4;
    ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = 7;
    write_le_u16(&mut ram, INTRO_SWORD_FLASH_RGB_CHANNEL, 0xab01);

    let mut intro_sword = IntroSwordState::load_from_ram(&ram);
    {
        let mut bridge = NativeIntroSwordBridgeMut::new(&mut intro_sword, &mut ram);
        bridge.advance_ypos();
        bridge.decrement_sparkle_timer();
        assert!(bridge.decrement_sparkle_step_check_negative());
        bridge.advance_anim_step();
        bridge.advance_sparkle_y_offset();
        bridge.cycle_flash_rgb_channel();
        bridge.set_flash_rgb_channel_word(0x0201);
    }

    assert_eq!(intro_sword.ypos(), 0x1244);
    assert_eq!(intro_sword.sparkle_timer(), 4);
    assert_eq!(intro_sword.sparkle_step(), 0xff);
    assert_eq!(intro_sword.anim_step_raw(), 6);
    assert_eq!(intro_sword.sparkle_y_offset(), 11);
    assert_eq!(intro_sword.flash_rgb_channel(), 1);
    assert_eq!(read_le_u16(&ram, INTRO_SWORD_YPOS), 0x1244);
    assert_eq!(ram[INTRO_SWORD_SPARKLE_TIMER], 4);
    assert_eq!(ram[INTRO_SWORD_SPARKLE_STEP], 0xff);
    assert_eq!(ram[INTRO_SWORD_ANIM_STEP], 6);
    assert_eq!(ram[INTRO_SWORD_SPARKLE_Y_OFFSET], 11);
    assert_eq!(read_le_u16(&ram, INTRO_SWORD_FLASH_RGB_CHANNEL), 0x0201);
}

#[test]
fn native_intro_sword_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, INTRO_SWORD_YPOS, 0x9999);
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, INTRO_SWORD_YPOS, 0x1234);
    native_ram[INTRO_SWORD_SPARKLE_TIMER] = 5;
    write_le_u16(&mut native_ram, INTRO_SWORD_FLASH_RGB_CHANNEL, 0xab02);
    let mut intro_sword = IntroSwordState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeIntroSwordBridgeMut::new(&mut intro_sword, &mut ram);
        bridge.advance_ypos();
    }

    assert_eq!(intro_sword.ypos(), 0x1244);
    assert_eq!(intro_sword.sparkle_timer(), 5);
    assert_eq!(read_le_u16(&ram, INTRO_SWORD_YPOS), 0x1244);
    assert_eq!(ram[INTRO_SWORD_SPARKLE_TIMER], 5);
    assert_eq!(read_le_u16(&ram, INTRO_SWORD_FLASH_RGB_CHANNEL), 0xab02);
}

#[test]
fn archery_game_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[ARCHERY_GAME_HIT_COUNTER] = 8;
    ram[ARCHERY_GAME_ARROWS_LEFT] = 5;
    ram[ARCHERY_GAME_OUT_OF_ARROWS] = 1;

    let mut archery = ArcheryGameState::load_from_ram(&ram);
    assert_eq!(archery.hit_counter(), 8);
    assert_eq!(archery.arrows_left(), 5);
    assert_eq!(archery.out_of_arrows(), 1);

    archery.increment_hit_counter();
    archery.decrement_arrows_left();
    archery.clear_out_of_arrows();
    archery.write_to_ram(&mut ram);

    assert_eq!(ram[ARCHERY_GAME_HIT_COUNTER], 9);
    assert_eq!(ram[ARCHERY_GAME_ARROWS_LEFT], 4);
    assert_eq!(ram[ARCHERY_GAME_OUT_OF_ARROWS], 0);
}

#[test]
fn native_archery_game_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[ARCHERY_GAME_HIT_COUNTER] = 0xff;
    ram[ARCHERY_GAME_ARROWS_LEFT] = 0;
    ram[ARCHERY_GAME_OUT_OF_ARROWS] = 0xff;

    let mut archery = ArcheryGameState::load_from_ram(&ram);
    {
        let mut bridge = NativeArcheryGameBridgeMut::new(&mut archery, &mut ram);
        bridge.increment_hit_counter();
        bridge.clear_hit_counter();
        bridge.set_arrows_left(5);
        bridge.decrement_arrows_left();
        bridge.increment_out_of_arrows();
        bridge.clear_out_of_arrows();
    }

    assert_eq!(archery.hit_counter(), 0);
    assert_eq!(archery.arrows_left(), 4);
    assert_eq!(archery.out_of_arrows(), 0);
    assert_eq!(ram[ARCHERY_GAME_HIT_COUNTER], 0);
    assert_eq!(ram[ARCHERY_GAME_ARROWS_LEFT], 4);
    assert_eq!(ram[ARCHERY_GAME_OUT_OF_ARROWS], 0);
}

#[test]
fn native_archery_game_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[ARCHERY_GAME_HIT_COUNTER] = 0xff;
    ram[ARCHERY_GAME_ARROWS_LEFT] = 0;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[ARCHERY_GAME_HIT_COUNTER] = 3;
    native_ram[ARCHERY_GAME_ARROWS_LEFT] = 5;
    native_ram[ARCHERY_GAME_OUT_OF_ARROWS] = 1;
    let mut archery = ArcheryGameState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeArcheryGameBridgeMut::new(&mut archery, &mut ram);
        bridge.decrement_arrows_left();
    }

    assert_eq!(archery.hit_counter(), 3);
    assert_eq!(archery.arrows_left(), 4);
    assert_eq!(archery.out_of_arrows(), 1);
    assert_eq!(ram[ARCHERY_GAME_HIT_COUNTER], 3);
    assert_eq!(ram[ARCHERY_GAME_ARROWS_LEFT], 4);
    assert_eq!(ram[ARCHERY_GAME_OUT_OF_ARROWS], 1);
}

#[test]
fn sprite_battle_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[NUM_SPRITES_KILLED] = 3;
    ram[TIMES_HURT_BY_SPRITES] = 4;
    ram[ITEM_DROP_LUCK] = 5;
    ram[LUCK_KILL_COUNTER] = 6;
    ram[ITEM_DROP_COUNTER] = 7;
    ram[DAMAGE_TYPE_DETERMINER] = 8;
    ram[SET_WHEN_DAMAGING_ENEMIES] = 9;

    let battle = SpriteBattleState::load_from_ram(&ram);
    assert_eq!(battle.sprites_killed(), 3);
    assert_eq!(battle.times_hurt_by_sprites(), 4);
    assert_eq!(battle.item_drop_luck(), 5);
    assert_eq!(battle.luck_kill_counter(), 6);
    assert_eq!(battle.item_drop_counter(), 7);
    assert_eq!(battle.damage_type_determiner(), 8);
    assert_eq!(battle.damaging_enemies_timer(), 9);

    let mut projected = vec![0; WRAM_SIZE];
    battle.write_to_ram(&mut projected);
    assert_eq!(SpriteBattleState::load_from_ram(&projected), battle);
}

#[test]
fn native_sprite_battle_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[NUM_SPRITES_KILLED] = 0xff;
    ram[TIMES_HURT_BY_SPRITES] = 0xff;
    ram[LUCK_KILL_COUNTER] = 0xff;
    ram[ITEM_DROP_COUNTER] = 0xff;
    ram[SET_WHEN_DAMAGING_ENEMIES] = 0x81;

    let mut battle = SpriteBattleState::load_from_ram(&ram);
    {
        let mut bridge = NativeSpriteBattleBridgeMut::new(&mut battle, &mut ram);
        bridge.clear_sprites_killed();
        bridge.increment_sprites_killed();
        bridge.clear_times_hurt_by_sprites();
        bridge.increment_times_hurt_by_sprites();
        bridge.set_item_drop_luck(2);
        bridge.clear_luck_kill_counter();
        bridge.increment_luck_kill_counter();
        bridge.clear_item_drop_counter();
        bridge.increment_item_drop_counter();
        bridge.set_damage_type_determiner(10);
        bridge.set_damaging_enemies_timer(2);
        bridge.tick_damaging_enemies_timer();
        bridge.clear_damaging_enemies_timer();
    }

    assert_eq!(battle.sprites_killed(), 1);
    assert_eq!(battle.times_hurt_by_sprites(), 1);
    assert_eq!(battle.item_drop_luck(), 2);
    assert_eq!(battle.luck_kill_counter(), 1);
    assert_eq!(battle.item_drop_counter(), 1);
    assert_eq!(battle.damage_type_determiner(), 10);
    assert_eq!(battle.damaging_enemies_timer(), 0);
    assert_eq!(ram[NUM_SPRITES_KILLED], 1);
    assert_eq!(ram[TIMES_HURT_BY_SPRITES], 1);
    assert_eq!(ram[ITEM_DROP_LUCK], 2);
    assert_eq!(ram[LUCK_KILL_COUNTER], 1);
    assert_eq!(ram[ITEM_DROP_COUNTER], 1);
    assert_eq!(ram[DAMAGE_TYPE_DETERMINER], 10);
    assert_eq!(ram[SET_WHEN_DAMAGING_ENEMIES], 0);
}

#[test]
fn native_sprite_battle_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[NUM_SPRITES_KILLED] = 0xff;
    ram[TIMES_HURT_BY_SPRITES] = 0xff;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[NUM_SPRITES_KILLED] = 2;
    native_ram[TIMES_HURT_BY_SPRITES] = 3;
    native_ram[ITEM_DROP_LUCK] = 4;
    let mut battle = SpriteBattleState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeSpriteBattleBridgeMut::new(&mut battle, &mut ram);
        bridge.increment_sprites_killed();
    }

    assert_eq!(battle.sprites_killed(), 3);
    assert_eq!(battle.times_hurt_by_sprites(), 3);
    assert_eq!(battle.item_drop_luck(), 4);
    assert_eq!(ram[NUM_SPRITES_KILLED], 3);
    assert_eq!(ram[TIMES_HURT_BY_SPRITES], 3);
    assert_eq!(ram[ITEM_DROP_LUCK], 4);
}

#[test]
fn native_happiness_pond_rupee_bridge_loads_and_stores_snapshots() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut effects = EffectState::load_from_ram(&ram);

    {
        let mut bridge =
            NativeHappinessPondRupeeBridgeMut::new(&mut effects.happiness_pond_rupees, &mut ram, 4);
        bridge.initialize(0x1234, 0x5678, 0x9a, 0xbc, 0xde);
    }

    let rupee = effects.happiness_pond_rupees.rupee(4);
    assert!(rupee.is_active());
    assert_eq!(rupee.step(), 0);
    let snapshot = rupee.snapshot();
    assert_eq!(snapshot.x_low, 0x34);
    assert_eq!(snapshot.x_high, 0x12);
    assert_eq!(snapshot.y_low, 0x78);
    assert_eq!(snapshot.y_high, 0x56);
    assert_eq!(snapshot.x_velocity, 0x9a);
    assert_eq!(snapshot.y_velocity, 0xbc);
    assert_eq!(snapshot.z_velocity, 0xde);
    assert_eq!(snapshot.item_to_link, 53);
    assert_eq!(snapshot.timer, 15);

    let stored = HappinessPondRupeeSnapshot {
        y_low: 1,
        y_high: 2,
        x_low: 3,
        x_high: 4,
        z: 5,
        y_velocity: 6,
        x_velocity: 7,
        z_velocity: 8,
        y_subpixel: 9,
        x_subpixel: 10,
        z_subpixel: 11,
        item_to_link: 12,
        timer: 13,
        step: 14,
    };
    {
        let mut bridge =
            NativeHappinessPondRupeeBridgeMut::new(&mut effects.happiness_pond_rupees, &mut ram, 4);
        bridge.store_snapshot(stored);
    }
    let expected_snapshot = HappinessPondRupeeSnapshot {
        timer: 12,
        ..stored
    };
    assert_eq!(
        effects.happiness_pond_rupees.rupee(4).snapshot(),
        expected_snapshot
    );
    assert_eq!(ram[HAPPINESS_POND_Y_LO + 4], 1);
    assert_eq!(ram[HAPPINESS_POND_Y_HI + 4], 2);
    assert_eq!(ram[HAPPINESS_POND_X_LO + 4], 3);
    assert_eq!(ram[HAPPINESS_POND_X_HI + 4], 4);
    assert_eq!(ram[HAPPINESS_POND_Z + 4], 5);
    assert_eq!(ram[HAPPINESS_POND_Y_VEL + 4], 6);
    assert_eq!(ram[HAPPINESS_POND_X_VEL + 4], 7);
    assert_eq!(ram[HAPPINESS_POND_Z_VEL + 4], 8);
    assert_eq!(ram[HAPPINESS_POND_Y_SUBPIXEL + 4], 9);
    assert_eq!(ram[HAPPINESS_POND_X_SUBPIXEL + 4], 10);
    assert_eq!(ram[HAPPINESS_POND_Z_SUBPIXEL + 4], 11);
    assert_eq!(ram[HAPPINESS_POND_ITEM_TO_LINK + 4], 12);
    assert_eq!(ram[HAPPINESS_POND_TIMER + 4], 13);
    assert_eq!(ram[HAPPINESS_POND_STEP + 4], 14);

    {
        let mut bridge =
            NativeHappinessPondRupeeBridgeMut::new(&mut effects.happiness_pond_rupees, &mut ram, 4);
        bridge.clear();
    }
    assert!(!effects.happiness_pond_rupees.rupee(4).is_active());
    assert_eq!(ram[HAPPINESS_POND_ACTIVE + 4], 0);
}

#[test]
fn native_happiness_pond_rupee_bridge_composes_edits_onto_live_ram() {
    // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is no
    // longer bulk-projected, so the bridge must compose its edits onto whatever is in RAM
    // now rather than re-stamp a stale native snapshot over a live effect's write.
    let mut stale_ram = vec![0; WRAM_SIZE];
    stale_ram[HAPPINESS_POND_ACTIVE + 4] = 0xff;
    stale_ram[HAPPINESS_POND_X_LO + 4] = 0xee;
    stale_ram[HAPPINESS_POND_TIMER + 4] = 0xdd;

    let mut ram = vec![0; WRAM_SIZE];
    ram[HAPPINESS_POND_ACTIVE + 4] = 1;
    ram[HAPPINESS_POND_X_LO + 4] = 0x34;
    ram[HAPPINESS_POND_X_HI + 4] = 0x12;
    ram[HAPPINESS_POND_TIMER + 4] = 8;
    let mut effects = EffectState::load_from_ram(&stale_ram);

    {
        let mut bridge =
            NativeHappinessPondRupeeBridgeMut::new(&mut effects.happiness_pond_rupees, &mut ram, 4);
        bridge.clear();
    }

    let rupee = effects.happiness_pond_rupees.rupee(4);
    assert!(!rupee.is_active());
    assert_eq!(rupee.snapshot().x_low, 0x34);
    assert_eq!(rupee.snapshot().x_high, 0x12);
    assert_eq!(rupee.snapshot().timer, 7);
    assert_eq!(ram[HAPPINESS_POND_ACTIVE + 4], 0);
    assert_eq!(ram[HAPPINESS_POND_X_LO + 4], 0x34);
    assert_eq!(ram[HAPPINESS_POND_X_HI + 4], 0x12);
    assert_eq!(ram[HAPPINESS_POND_TIMER + 4], 8);
}
