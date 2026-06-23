use super::*;

#[test]
fn draw_completely_open_door_uses_inter_room_upnorth_count_before_wall_spirals() {
    let mut state = ZeldaState::new();
    let pos = 0x011eusize;

    state.dungeon_bg2_attributes_mut().fill_bg2_attr_range(
        pos + xy(1, 0),
        xy(1, 3) - xy(1, 0) + 2,
        0xf0,
    );
    state
        .dungeon_stair_lists_mut()
        .set_inter_staircase_pos(0, pos as u16);

    state
        .dungeon_stair_lists_mut()
        .set_stair_list_count(DungeonStairList::InterRoomUpNorth, 0);
    state.dungeon_stair_lists_mut().sync_stair_list_counts(
        &[
            DungeonStairList::WallUpNorthSpiral,
            DungeonStairList::WallUpNorthSpiralBg1,
            DungeonStairList::InterRoomUpNorthStraight,
            DungeonStairList::InterRoomUpSouthStraight,
            DungeonStairList::InterRoomSouthDown,
            DungeonStairList::WallDownNorthSpiral,
            DungeonStairList::WallDownNorthSpiralBg1,
            DungeonStairList::InterPseudoUpNorth,
        ],
        2,
    );

    state.DrawCompletelyOpenDoor();

    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 0), 2),
        &[0x5e, 0x5e]
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 1), 2),
        &[0x30, 0x30]
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 2), 2),
        &[0x00, 0x00]
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_slice(pos + xy(1, 3), 2),
        &[0x00, 0x00]
    );
}

#[test]
fn room_draw_all_objects_clears_width_height_before_terminator() {
    let mut state = ZeldaState::new();
    state
        .dungeon_room_load_mut()
        .set_draw_dimensions_words(0x4000, 0x2000);
    write_le_u16(&mut state.ram, DUNG_LOAD_PTR_OFFS, 0);

    state.RoomData_DrawObjects_from(&[0xff, 0xff]);

    assert_eq!(
        state
            .game_state
            .dungeon
            .room_load
            .draw_width_indicator_word(),
        0
    );
    assert_eq!(
        state
            .game_state
            .dungeon
            .room_load
            .draw_height_indicator_word(),
        0
    );
}

#[test]
fn dungeon_load_room_resets_floor_velocity_words() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, DUNG_FLOOR_Y_VEL_DUNGEON, 0x0002);
    write_le_u16(&mut state.ram, DUNG_FLOOR_X_VEL, 0x0001);

    state.dungeon_load_room_reset_floor_velocity();

    assert_eq!(read_le_u16(&state.ram, DUNG_FLOOR_Y_VEL_DUNGEON), 0);
    assert_eq!(read_le_u16(&state.ram, DUNG_FLOOR_X_VEL), 0);
}
