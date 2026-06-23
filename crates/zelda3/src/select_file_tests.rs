use super::*;

#[test]
fn name_entry_vertical_scroll_down_from_top_row_matches_c() {
    let mut state = ZeldaState::new();
    state.set_select_file_name_row(0);
    state.follower_link_state_mut().set_joypad1h_last(0x04);

    state.name_file_check_for_scroll_input_y();

    assert_eq!(state.game_state.messaging.select_file_menu.name_row(), 1);
    assert_eq!(
        state
            .game_state
            .messaging
            .select_file_menu
            .name_scroll_y_step(),
        1
    );
    assert_eq!(state.game_state.messaging.select_file_menu.choice(1), 1);
}

#[test]
fn name_entry_vertical_scroll_down_released_settles_on_next_row() {
    let mut state = ZeldaState::new();
    state.set_select_file_name_row(0);
    state.set_select_file_name_cursor_y(0x83);
    state.follower_link_state_mut().set_joypad1h_last(0x04);

    state.name_file_check_for_scroll_input_y();
    state.follower_link_state_mut().set_joypad1h_last(0);

    for _ in 0..9 {
        state.name_file_do_the_naming();
    }

    assert_eq!(state.game_state.messaging.select_file_menu.name_row(), 1);
    assert_eq!(
        state.game_state.messaging.select_file_menu.name_cursor_y(),
        0x93
    );
    assert_eq!(
        state
            .game_state
            .messaging
            .select_file_menu
            .name_scroll_y_step(),
        0
    );
}
