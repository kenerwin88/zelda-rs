use super::*;

#[test]
fn oam_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OAM_PRIORITY_VALUE, 0x2100);
    write_le_u16(&mut ram, OAM_CUR_PTR, OAM_BUF as u16 + 8);
    write_le_u16(&mut ram, OAM_EXT_CUR_PTR, BYTEWISE_EXTENDED_OAM as u16 + 2);
    ram[SORT_SPRITES_SETTING] = 3;
    write_le_u16(&mut ram, OAM_PRIORITY_VALUE_2, 0x1200);
    write_le_u16(&mut ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER, 0x0040);
    ram[VALUE_COMPUTED_FOR_PLAYER_OAM] = 0x77;
    ram[TURTLE_ROCK_OAM_PRIORITY_FLAG] = 1;
    write_le_u16(&mut ram, OAM_REGION_BASE + 2, 0x01d0);
    write_le_u16(&mut ram, OAM_REGION_ALLOC + 2, 0x0008);
    ram[OAM_BUF + 8] = 0x12;
    ram[OAM_BUF + 9] = 0x34;
    ram[OAM_BUF + 10] = 0x56;
    ram[OAM_BUF + 11] = 0x78;
    ram[EXTENDED_OAM + 1] = 0xab;
    ram[BYTEWISE_EXTENDED_OAM + 2] = 0xcd;

    let oam = OamState::load_from_ram(&ram);
    assert_eq!(oam.priority_word(), 0x2100);
    assert_eq!(oam.current_pointer(), OAM_BUF as u16 + 8);
    assert_eq!(
        oam.current_extended_pointer(),
        BYTEWISE_EXTENDED_OAM as u16 + 2
    );
    assert_eq!(oam.sprite_sorting_setting(), 3);
    assert_eq!(oam.priority_value_2(), 0x1200);
    assert_eq!(oam.sort_sprites_offset(), 0x0040);
    assert_eq!(oam.player_oam_computed_value(), 0x77);
    assert_eq!(oam.turtle_rock_priority_flag(), 1);
    assert_eq!(oam.region_base_word(1), 0x01d0);
    assert_eq!(oam.region_alloc_counter(1), 0x0008);
    assert_eq!(oam.entry_x(OAM_BUF + 8), 0x12);
    assert_eq!(oam.entry_y(OAM_BUF + 8), 0x34);
    assert_eq!(oam.entry_char(OAM_BUF + 8), 0x56);
    assert_eq!(oam.entry_flags(OAM_BUF + 8), 0x78);
    assert_eq!(oam.extended_byte(2), 0xcd);

    let mut projected = vec![0; WRAM_SIZE];
    oam.write_to_ram(&mut projected);
    // OAM_PRIORITY_VALUE / OAM_REGION_BASE / OAM_REGION_ALLOC are write-through (excluded from
    // write_to_ram, mode-reused with the attract-scene scratch), so their setters own the RAM
    // bytes. Project them explicitly the way the setters do before round-tripping.
    crate::types::write_le_u16(
        &mut projected,
        crate::game_state::constants::OAM_PRIORITY_VALUE,
        oam.priority_word(),
    );
    for region in 0..6 {
        crate::types::write_le_u16(
            &mut projected,
            crate::game_state::constants::OAM_REGION_BASE + region * 2,
            oam.region_base_word(region),
        );
        crate::types::write_le_u16(
            &mut projected,
            crate::game_state::constants::OAM_REGION_ALLOC + region * 2,
            oam.region_alloc_counter(region),
        );
    }
    assert_eq!(OamState::load_from_ram(&projected), oam);
}

#[test]
fn oam_state_owns_scalar_pointer_behavior() {
    let mut oam = OamState::default();

    oam.set_priority_word(0x2201);
    oam.subtract_priority_word(0x0001);
    oam.set_priority_high(0x30);
    oam.set_current_pointer(OAM_BUF as u16 + 4);
    oam.add_current_pointer(4);
    oam.subtract_current_pointer(2);
    oam.set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16 + 1);
    oam.add_current_extended_pointer(2);
    oam.subtract_current_extended_pointer(1);
    oam.set_sprite_sorting_setting(2);
    oam.set_priority_value_2(0x1100);
    oam.set_sort_sprites_offset(0x0030);
    oam.set_player_oam_computed_value(0x44);

    assert_eq!(oam.priority_word(), 0x3000);
    assert_eq!(oam.current_pointer(), OAM_BUF as u16 + 6);
    assert_eq!(
        oam.current_extended_pointer(),
        BYTEWISE_EXTENDED_OAM as u16 + 2
    );
    assert_eq!(oam.sprite_sorting_setting(), 2);
    assert!(oam.has_sprite_sorting());
    assert_eq!(oam.priority_value_2(), 0x1100);
    assert_eq!(oam.sort_sprites_offset(), 0x0030);
    assert_eq!(oam.player_oam_computed_value(), 0x44);

    oam.clear_sprite_sorting_setting();
    oam.clear_sort_sprites_offset();
    assert_eq!(oam.sprite_sorting_setting(), 0);
    assert!(!oam.has_sprite_sorting());
    assert_eq!(oam.sort_sprites_offset(), 0);
}

#[test]
fn native_oam_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, OAM_PRIORITY_VALUE, 0x0100);
    write_le_u16(&mut ram, OAM_CUR_PTR, OAM_BUF as u16);
    write_le_u16(&mut ram, OAM_EXT_CUR_PTR, BYTEWISE_EXTENDED_OAM as u16);

    let mut oam = OamState::load_from_ram(&ram);
    {
        let mut bridge = NativeOamStateBridgeMut::new(&mut oam, &mut ram);
        bridge.set_priority_word(0x2200);
        bridge.subtract_priority_word(0x0100);
        bridge.set_priority_high(0x30);
        bridge.set_current_pointer(OAM_BUF as u16 + 4);
        bridge.add_current_pointer(4);
        bridge.set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16 + 1);
        bridge.add_current_extended_pointer(1);
        bridge.set_sprite_sorting_setting(2);
        bridge.set_priority_value_2(0x1100);
        bridge.set_sort_sprites_offset(0x0030);
        bridge.set_player_oam_computed_value(0x44);
        bridge.write_entry(OAM_BUF + 8, 0x12, 0x34, 0x56, 0x78);
        bridge.set_entry_x(OAM_BUF + 8, 0x13);
        bridge.set_entry_y(OAM_BUF + 8, 0x35);
        bridge.set_entry_char_flags(OAM_BUF + 8, 0x9abc);
        bridge.set_extended_byte(3, 0x55);
        bridge.set_extended_byte_at(BYTEWISE_EXTENDED_OAM + 4, 0x66);
        bridge.set_packed_extended_oam_byte(1, 0x77);
        bridge.set_region_base_word(1, 0x01d0);
        bridge.set_region_alloc_counter(1, 0x0008);
    }

    assert_eq!(oam.priority_word(), 0x3000);
    assert_eq!(oam.current_pointer(), OAM_BUF as u16 + 8);
    assert_eq!(
        oam.current_extended_pointer(),
        BYTEWISE_EXTENDED_OAM as u16 + 2
    );
    assert_eq!(oam.sprite_sorting_setting(), 2);
    assert_eq!(oam.priority_value_2(), 0x1100);
    assert_eq!(oam.sort_sprites_offset(), 0x0030);
    assert_eq!(oam.player_oam_computed_value(), 0x44);
    assert_eq!(oam.entry_x(OAM_BUF + 8), 0x13);
    assert_eq!(oam.entry_y(OAM_BUF + 8), 0x35);
    assert_eq!(oam.entry_char(OAM_BUF + 8), 0xbc);
    assert_eq!(oam.entry_flags(OAM_BUF + 8), 0x9a);
    assert_eq!(oam.extended_byte(3), 0x55);
    assert_eq!(oam.extended_byte(4), 0x66);
    assert_eq!(oam.region_base_word(1), 0x01d0);
    assert_eq!(oam.region_alloc_counter(1), 0x0008);
    assert_eq!(OamState::load_from_ram(&ram), oam);
}

#[test]
fn native_oam_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut oam = OamState::default();
    {
        let mut bridge = NativeOamStateBridgeMut::new(&mut oam, &mut ram);
        bridge.set_priority_word(0x2100);
        bridge.write_entry(OAM_BUF + 4, 1, 2, 3, 4);
        bridge.set_extended_byte(1, 5);
    }

    write_le_u16(&mut ram, OAM_PRIORITY_VALUE, 0xaaaa);
    ram[OAM_BUF + 4] = 0xbb;
    ram[BYTEWISE_EXTENDED_OAM + 1] = 0xcc;

    {
        let mut bridge = NativeOamStateBridgeMut::new(&mut oam, &mut ram);
        bridge.set_priority_high(0x22);
    }

    assert_eq!(oam.priority_word(), 0x2200);
    assert_eq!(oam.entry_x(OAM_BUF + 4), 1);
    assert_eq!(oam.extended_byte(1), 5);
    assert_eq!(OamState::load_from_ram(&ram), oam);
}
