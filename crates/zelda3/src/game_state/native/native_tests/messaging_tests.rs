use super::*;

#[test]
fn dialogue_message_index_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0x0123);

    let mut message_index = DialogueMessageIndexState::load_from_ram(&ram);
    assert_eq!(message_index.value(), 0x0123);

    message_index.set_value(0x0140);
    message_index.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
}

#[test]
fn native_dialogue_message_index_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0x0123);

    let mut message_index = DialogueMessageIndexState::load_from_ram(&ram);
    {
        let mut bridge = NativeDialogueMessageIndexBridgeMut::new(&mut message_index, &mut ram);
        bridge.set_value(0x0140);
    }

    assert_eq!(message_index.value(), 0x0140);
    assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
}

#[test]
fn native_dialogue_message_index_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0xffff);
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, DIALOGUE_MESSAGE_INDEX, 0x0123);
    let mut message_index = DialogueMessageIndexState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDialogueMessageIndexBridgeMut::new(&mut message_index, &mut ram);
        bridge.set_value(0x0140);
    }

    assert_eq!(message_index.value(), 0x0140);
    assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
}

#[test]
fn select_file_menu_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SELECTFILE_SAVE_SLOT_FLAGS, 1);
    write_le_u16(&mut ram, SELECTFILE_SAVE_SLOT_FLAGS + 2, 2);
    write_le_u16(&mut ram, SELECTFILE_SAVE_SLOT_FLAGS + 4, 3);
    ram[SELECT_FILE_CURSOR_WORK] = 4;
    ram[SELECT_FILE_TRANSITION_WORK] = 7;
    write_le_u16(&mut ram, SELECT_FILE_TARGET_WORK, 0x1234);
    write_le_u16(&mut ram, SELECT_FILE_COPY_SOURCE_SLOT_X2, 0x1204);
    ram[SELECT_FILE_REMEMBERED_CURSOR] = 2;
    write_le_u16(&mut ram, SELECT_FILE_NAME_SCROLL_X, 0x01f0);
    ram[SELECT_FILE_NAME_COLUMN] = 5;
    ram[SELECT_FILE_NAME_CURSOR_Y] = 0x83;
    ram[SELECT_FILE_NAME_SLOT] = 3;
    ram[SELECT_FILE_NAME_SCROLL_X_STEP] = 8;
    ram[SELECT_FILE_NAME_SCROLL_Y_STEP] = 9;
    ram[SELECT_FILE_NAME_ROW] = 6;
    ram[SELECT_FILE_NAME_SCROLL_X_DIRECTION] = 1;

    let mut menu = SelectFileMenuState::load_from_ram(&ram);
    assert_eq!(menu.save_slot_flags(), [1, 2, 3]);
    assert!(menu.any_save_slot_flag());
    assert_eq!(menu.cursor(), 4);
    assert_eq!(menu.choice(0), 0x34);
    assert_eq!(menu.target_word(), 0x1234);
    assert_eq!(menu.copy_source_slot_x2(), 0x1204);
    assert_eq!(menu.copy_source_slot(), 0x0902);
    assert_eq!(menu.name_scroll_x(), 0x01f0);
    assert!(menu.is_name_scrolling());
    menu.clear_name_entry_state();
    menu.write_to_ram(&mut ram);

    assert_eq!(ram[SELECT_FILE_NAME_COLUMN], 0);
    assert_eq!(ram[SELECT_FILE_NAME_SLOT], 0);
    assert_eq!(ram[SELECT_FILE_NAME_ROW], 0);
    assert_eq!(ram[SELECT_FILE_CHOICE_WORK], 0);
    assert_eq!(ram[SELECT_FILE_COPY_SOURCE_SLOT_X2], 0);
    assert_eq!(ram[SELECT_FILE_COPY_SOURCE_SLOT_X2 + 1], 0x12);
    assert_eq!(ram[SELECT_FILE_NAME_CURSOR_Y], 0x83);
    assert_eq!(read_le_u16(&ram, SELECT_FILE_NAME_SCROLL_X), 0x01f0);
}

#[test]
fn native_select_file_menu_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[SELECT_FILE_CURSOR_WORK] = 0xff;
    ram[SELECT_FILE_NAME_SLOT] = 0;
    ram[SELECT_FILE_NAME_CURSOR_Y] = 0x80;
    ram[SELECT_FILE_NAME_SCROLL_X_STEP] = 0xfe;

    let mut menu = SelectFileMenuState::load_from_ram(&ram);
    {
        let mut bridge = NativeSelectFileMenuBridgeMut::new(&mut menu, &mut ram);
        assert_eq!(bridge.increment_cursor(), 0);
        bridge.remember_current_cursor();
        bridge.set_cursor(3);
        bridge.restore_remembered_cursor();
        bridge.set_target_word(0x1234);
        bridge.set_copy_source_slot(2);
        assert_eq!(bridge.move_name_slot_left_wrapped(), 5);
        assert_eq!(bridge.move_name_slot_right_wrapped(), 0);
        assert!(bridge.step_name_cursor_y_toward(0x84));
        assert_eq!(bridge.advance_name_scroll_x_step_by(4), 2);
        bridge.mark_save_slot_present(1);
        bridge.clear_transition_scratch();
    }

    assert_eq!(menu.cursor(), 0);
    assert_eq!(menu.remembered_cursor(), 0);
    assert_eq!(menu.target_word(), 0x1234);
    assert_eq!(menu.copy_source_slot_x2(), 4);
    assert_eq!(menu.name_slot(), 0);
    assert_eq!(menu.name_cursor_y(), 0x82);
    assert_eq!(menu.name_scroll_x_step(), 2);
    assert_eq!(menu.save_slot_flag(1), 1);
    assert_eq!(ram[SELECT_FILE_CURSOR_WORK], 0);
    assert_eq!(ram[SELECT_FILE_REMEMBERED_CURSOR], 0);
    assert_eq!(read_le_u16(&ram, SELECT_FILE_TARGET_WORK), 0x1234);
    assert_eq!(read_le_u16(&ram, SELECT_FILE_COPY_SOURCE_SLOT_X2), 4);
    assert_eq!(ram[SELECT_FILE_NAME_SLOT], 0);
    assert_eq!(ram[SELECT_FILE_NAME_CURSOR_Y], 0x82);
    assert_eq!(ram[SELECT_FILE_NAME_SCROLL_X_STEP], 2);
    assert_eq!(read_le_u16(&ram, SELECTFILE_SAVE_SLOT_FLAGS + 2), 1);
    assert_eq!(ram[SELECT_FILE_TRANSITION_WORK], 0);
}

#[test]
fn native_select_file_menu_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[SELECT_FILE_CURSOR_WORK] = 0xff;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[SELECT_FILE_CURSOR_WORK] = 2;
    native_ram[SELECT_FILE_NAME_CURSOR_Y] = 0x80;
    let mut menu = SelectFileMenuState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeSelectFileMenuBridgeMut::new(&mut menu, &mut ram);
        assert_eq!(bridge.increment_cursor(), 3);
        assert!(bridge.step_name_cursor_y_toward(0x84));
    }

    assert_eq!(menu.cursor(), 3);
    assert_eq!(menu.name_cursor_y(), 0x82);
    assert_eq!(ram[SELECT_FILE_CURSOR_WORK], 3);
    assert_eq!(ram[SELECT_FILE_NAME_CURSOR_Y], 0x82);
}

#[test]
fn multiselect_choice_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MULTISELECT_CHOICE, 0x0204);
    ram[MULTISELECT_CHOICE_BACKUP] = 0x07;

    let mut choice = MultiselectChoiceState::load_from_ram(&ram);
    assert_eq!(choice.value(), 0x04);
    assert_eq!(
        MultiselectChoiceRead::new(&choice, &MessagingRuntimeState::load_from_ram(&ram))
            .value_word(),
        0x0204
    );
    assert_eq!(choice.backup(), 0x07);

    choice.increment_value();
    choice.save_backup();
    choice.decrement_value();
    choice.restore_backup();
    choice.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, MULTISELECT_CHOICE), 0x0205);
    assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 0x05);
}

#[test]
fn native_multiselect_choice_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MULTISELECT_CHOICE, 0x0204);
    ram[MULTISELECT_CHOICE_BACKUP] = 0x07;

    let mut choice = MultiselectChoiceState::load_from_ram(&ram);
    {
        let mut bridge = NativeMultiselectChoiceBridgeMut::new(&mut choice, &mut ram);
        bridge.increment_value();
        bridge.save_backup();
        bridge.set_value(0x01);
        bridge.restore_backup();
        bridge.decrement_value();
    }

    assert_eq!(choice.value(), 0x04);
    assert_eq!(
        MultiselectChoiceRead::new(&choice, &MessagingRuntimeState::load_from_ram(&ram))
            .value_word(),
        0x0204
    );
    assert_eq!(choice.backup(), 0x05);
    assert_eq!(read_le_u16(&ram, MULTISELECT_CHOICE), 0x0204);
    assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 0x05);
}

#[test]
fn native_multiselect_choice_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MULTISELECT_CHOICE] = 0xff;
    ram[MULTISELECT_CHOICE_BACKUP] = 0xee;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[MULTISELECT_CHOICE] = 4;
    native_ram[MULTISELECT_CHOICE_BACKUP] = 7;
    let mut choice = MultiselectChoiceState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeMultiselectChoiceBridgeMut::new(&mut choice, &mut ram);
        bridge.increment_value();
        bridge.save_backup();
        bridge.set_value(1);
        bridge.restore_backup();
    }

    assert_eq!(choice.value(), 5);
    assert_eq!(choice.backup(), 5);
    assert_eq!(ram[MULTISELECT_CHOICE], 5);
    assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 5);
}

#[test]
fn dialogue_number_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIALOGUE_NUMBER_LO] = 0x12;
    ram[DIALOGUE_NUMBER_HI] = 0x34;

    let mut number = DialogueNumberState::load_from_ram(&ram);
    assert_eq!(number.packed_digits(0), 0x12);
    assert_eq!(number.packed_digits(1), 0x34);
    assert_eq!(number.packed_digits(2), 0);

    number.set_low_pair(0x56);
    number.set_high_pair(0x78);
    number.write_to_ram(&mut ram);

    assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x56);
    assert_eq!(ram[DIALOGUE_NUMBER_HI], 0x78);
}

#[test]
fn native_dialogue_number_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIALOGUE_NUMBER_LO] = 0x12;
    ram[DIALOGUE_NUMBER_HI] = 0x34;

    let mut number = DialogueNumberState::load_from_ram(&ram);
    {
        let mut bridge = NativeDialogueNumberBridgeMut::new(&mut number, &mut ram);
        bridge.set_low_pair(0x56);
        bridge.set_high_pair(0x78);
        bridge.set_packed_digits(0x9a, 0xbc);
    }

    assert_eq!(number.packed_digits(0), 0x9a);
    assert_eq!(number.packed_digits(1), 0xbc);
    assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x9a);
    assert_eq!(ram[DIALOGUE_NUMBER_HI], 0xbc);
}

#[test]
fn native_dialogue_number_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIALOGUE_NUMBER_LO] = 0xff;
    ram[DIALOGUE_NUMBER_HI] = 0xee;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[DIALOGUE_NUMBER_LO] = 0x12;
    native_ram[DIALOGUE_NUMBER_HI] = 0x34;
    let mut number = DialogueNumberState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDialogueNumberBridgeMut::new(&mut number, &mut ram);
        bridge.set_low_pair(0x56);
    }

    assert_eq!(number.packed_digits(0), 0x56);
    assert_eq!(number.packed_digits(1), 0x34);
    assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x56);
    assert_eq!(ram[DIALOGUE_NUMBER_HI], 0x34);
}

#[test]
fn dialogue_source_offset_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIALOGUE_MSG_SRC_OFFS] = 0xaa;
    ram[DIALOGUE_MSG_SRC_OFFS + 1] = 0xbb;
    ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0x0e;

    let mut source_offset = DialogueSourceOffsetState::load_from_ram(&ram);
    assert_eq!(source_offset.bank_offset_low_nibble(), 0x0e);
    assert_eq!(source_offset.increment_bank_offset_low_nibble(), 0x0f);
    assert_eq!(source_offset.increment_bank_offset_low_nibble(), 0x10);
    source_offset.write_to_ram(&mut ram);

    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS], 0xaa);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 1], 0xbb);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 0x10);
}

#[test]
fn native_dialogue_source_offset_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIALOGUE_MSG_SRC_OFFS] = 0xaa;
    ram[DIALOGUE_MSG_SRC_OFFS + 1] = 0xbb;
    ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0xff;

    let mut source_offset = DialogueSourceOffsetState::load_from_ram(&ram);
    {
        let mut bridge = NativeDialogueSourceOffsetBridgeMut::new(&mut source_offset, &mut ram);
        assert_eq!(bridge.increment_bank_offset_low_nibble(), 0);
        assert_eq!(bridge.increment_bank_offset_low_nibble(), 1);
    }

    assert_eq!(source_offset.bank_offset_low_nibble(), 1);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS], 0xaa);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 1], 0xbb);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 1);
}

#[test]
fn native_dialogue_source_offset_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0xff;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0x0e;
    let mut source_offset = DialogueSourceOffsetState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDialogueSourceOffsetBridgeMut::new(&mut source_offset, &mut ram);
        assert_eq!(bridge.increment_bank_offset_low_nibble(), 0x0f);
    }

    assert_eq!(source_offset.bank_offset_low_nibble(), 0x0f);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 0x0f);
}

#[test]
fn decoded_message_text_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MESSAGING_TEXT_BUFFER] = 0x12;
    ram[MESSAGING_TEXT_BUFFER + 1] = 0x34;
    ram[MESSAGING_TEXT_BUFFER + 2] = 0x56;

    let mut text = DecodedMessageTextState::load_from_ram(&ram);
    assert_eq!(text.byte(0), 0x12);
    assert_eq!(text.next_byte(0), Some(0x34));
    assert_eq!(text.byte(usize::MAX), 0);

    assert_eq!(text.load_decoded_dialogue(&[1, 2, 3, 4]), 4);
    assert_eq!(
        text.write_decoded_text_at(MESSAGING_TEXT_BUFFER + 2, &[0xaa, 0xbb]),
        2
    );
    text.write_to_ram(&mut ram);

    assert_eq!(
        &ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + 4],
        &[1, 2, 0xaa, 0xbb]
    );
}

#[test]
fn dialogue_pointer_table_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TEXT_DIALOGUE_POINTERS + 3] = 0x12;
    ram[TEXT_DIALOGUE_POINTERS + 4] = 0x34;
    ram[TEXT_DIALOGUE_POINTERS + 5] = 0x56;

    let mut pointers = DialoguePointerTableState::load_from_ram(&ram);
    assert_eq!(pointers.pointer(1), 0x563412);
    assert_eq!(pointers.pointer(398), 0);

    pointers.set_pointer(1, 0xffaabbcc);
    pointers.write_to_ram(&mut ram);

    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 3], 0xcc);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 4], 0xbb);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 5], 0xaa);
}

#[test]
fn native_decoded_message_text_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MESSAGING_TEXT_BUFFER] = 0x12;
    ram[TEXT_DIALOGUE_POINTERS] = 0x11;
    ram[TEXT_DIALOGUE_POINTERS + 1] = 0x22;
    ram[TEXT_DIALOGUE_POINTERS + 2] = 0x33;

    let mut messaging = MessagingState::load_from_ram(&ram);
    {
        let mut bridge = NativeDecodedMessageTextBridgeMut::new(&mut messaging, &mut ram);
        assert_eq!(bridge.load_decoded_dialogue(&[1, 2, 3]), 3);
        assert_eq!(
            bridge.write_decoded_text_at(MESSAGING_TEXT_BUFFER + 1, &[0xaa, 0xbb]),
            2
        );
        bridge.set_dialogue_pointer(0, 0xffc0de);
    }

    assert_eq!(messaging.decoded_text.byte(0), 1);
    assert_eq!(messaging.decoded_text.byte(1), 0xaa);
    assert_eq!(messaging.decoded_text.byte(2), 0xbb);
    assert_eq!(messaging.dialogue_pointers.pointer(0), 0xffc0de);
    assert_eq!(
        &ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + 3],
        &[1, 0xaa, 0xbb]
    );
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS], 0xde);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 1], 0xc0);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 2], 0xff);
}

#[test]
fn native_decoded_message_text_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MESSAGING_TEXT_BUFFER] = 0xff;
    ram[TEXT_DIALOGUE_POINTERS] = 0xee;
    ram[TEXT_DIALOGUE_POINTERS + 1] = 0xdd;
    ram[TEXT_DIALOGUE_POINTERS + 2] = 0xcc;

    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[MESSAGING_TEXT_BUFFER] = 0x12;
    native_ram[MESSAGING_TEXT_BUFFER + 1] = 0x34;
    native_ram[TEXT_DIALOGUE_POINTERS] = 0x11;
    native_ram[TEXT_DIALOGUE_POINTERS + 1] = 0x22;
    native_ram[TEXT_DIALOGUE_POINTERS + 2] = 0x33;
    let mut messaging = MessagingState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeDecodedMessageTextBridgeMut::new(&mut messaging, &mut ram);
        assert_eq!(
            bridge.write_decoded_text_at(MESSAGING_TEXT_BUFFER + 1, &[0xaa]),
            1
        );
        bridge.set_dialogue_pointer(0, 0x445566);
    }

    assert_eq!(messaging.decoded_text.byte(0), 0x12);
    assert_eq!(messaging.decoded_text.byte(1), 0xaa);
    assert_eq!(messaging.dialogue_pointers.pointer(0), 0x445566);
    assert_eq!(ram[MESSAGING_TEXT_BUFFER], 0x12);
    assert_eq!(ram[MESSAGING_TEXT_BUFFER + 1], 0xaa);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS], 0x66);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 1], 0x55);
    assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 2], 0x44);
}

#[test]
fn native_messaging_runtime_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MESSAGING_MODULE] = 1;
    ram[TEXT_WAIT_COUNTDOWN] = 0x34;
    ram[TEXT_WAIT_COUNTDOWN + 1] = 0x12;
    ram[MULTISELECT_CHOICE] = 3;
    ram[DIALOGUE_MSG_SRC_OFFS + 2] = 7;

    let mut messaging = MessagingState::load_from_ram(&ram);
    {
        let mut bridge = NativeMessagingRuntimeBridgeMut::new(&mut messaging, &mut ram);
        bridge.set_module(2);
        bridge.clear_text_wait_countdown();
        bridge.increment_text_incremental_state();
        bridge.init_msgbox_state_from(&[
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0x39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0x1c, 4,
            0, 0, 0, 0, 0,
        ]);
    }

    assert_eq!(messaging.runtime.module(), 1);
    assert_eq!(messaging.runtime.dialogue_text_color(), 0x39);
    assert_eq!(messaging.runtime.text_wait_countdown2(), 0x1c);
    assert_eq!(messaging.runtime.dialogue_scroll_speed(), 4);
    assert_eq!(messaging.multiselect_choice.value(), 5);
    assert_eq!(messaging.dialogue_source_offset.bank_offset_low_nibble(), 0);
    assert_eq!(ram[MESSAGING_MODULE], 1);
    assert_eq!(
        ram[crate::game_state::constants::messaging::DIALOGUE_TEXT_COLOR],
        0x39
    );
    assert_eq!(ram[TEXT_WAIT_COUNTDOWN2], 0x1c);
    assert_eq!(ram[DIALOGUE_SCROLL_SPEED], 4);
    assert_eq!(ram[MULTISELECT_CHOICE], 5);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 0);
}

#[test]
fn native_messaging_runtime_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MESSAGING_MODULE] = 0xff;
    ram[TEXT_INCREMENTAL_STATE] = 0xee;
    ram[TEXT_WAIT_COUNTDOWN] = 0xdd;
    ram[TEXT_WAIT_COUNTDOWN + 1] = 0xcc;
    ram[MULTISELECT_CHOICE] = 0xbb;
    ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0xaa;

    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[MESSAGING_MODULE] = 3;
    native_ram[TEXT_INCREMENTAL_STATE] = 4;
    native_ram[TEXT_WAIT_COUNTDOWN] = 0x34;
    native_ram[TEXT_WAIT_COUNTDOWN + 1] = 0x12;
    native_ram[MULTISELECT_CHOICE] = 6;
    native_ram[DIALOGUE_MSG_SRC_OFFS + 2] = 9;
    let mut messaging = MessagingState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeMessagingRuntimeBridgeMut::new(&mut messaging, &mut ram);
        bridge.increment_text_incremental_state();
        bridge.clear_text_wait_countdown();
    }

    assert_eq!(messaging.runtime.module(), 3);
    assert_eq!(messaging.runtime.text_incremental_state(), 5);
    assert_eq!(messaging.runtime.text_wait_countdown(), 0x1200);
    assert_eq!(messaging.multiselect_choice.value(), 6);
    assert_eq!(messaging.dialogue_source_offset.bank_offset_low_nibble(), 9);
    assert_eq!(ram[MESSAGING_MODULE], 3);
    assert_eq!(ram[TEXT_INCREMENTAL_STATE], 5);
    assert_eq!(ram[TEXT_WAIT_COUNTDOWN], 0);
    assert_eq!(ram[TEXT_WAIT_COUNTDOWN + 1], 0x12);
    assert_eq!(ram[MULTISELECT_CHOICE], 6);
    assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 9);
}

#[test]
fn messaging_render_buffer_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MESSAGING_RENDER_BUFFER + 4, 0x1234);
    ram[MESSAGING_RENDER_BUFFER + 8] = 0xaa;

    let mut render_buffer = MessagingRenderBufferState::load_from_ram(&ram);
    assert_eq!(render_buffer.word(2), 0x1234);
    assert_eq!(render_buffer.word_at_byte_offset(8), 0x00aa);
    assert_eq!(render_buffer.word_at_byte_offset(0x7df), 0);

    render_buffer.set_word(2, 0x5678);
    render_buffer.xor_mask(8, 0x0f);
    render_buffer.clear_mask(8, 0xf0);
    render_buffer.fill_word_range(4, 2, 0x1111);
    render_buffer.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 4), 0x5678);
    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 8), 0x1111);
    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 10), 0x1111);
}

#[test]
fn native_messaging_render_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MESSAGING_RENDER_BUFFER + 2, 0x1111);
    ram[MESSAGING_RENDER_BUFFER + 6] = 0xf0;

    let mut render_buffer = MessagingRenderBufferState::load_from_ram(&ram);
    {
        let mut bridge = NativeMessagingRenderBufferBridgeMut::new(&mut render_buffer, &mut ram);
        bridge.set_word(1, 0x2222);
        bridge.set_word_at_byte_offset(4, 0x3333);
        bridge.xor_mask(6, 0x0f);
        bridge.clear_mask(6, 0xf0);
        bridge.fill_word_range(4, 2, 0x4444);
        bridge.clear_range(2);
    }

    assert_eq!(render_buffer.word(0), 0);
    assert_eq!(render_buffer.word(1), 0x2222);
    assert_eq!(render_buffer.word_at_byte_offset(4), 0x3333);
    assert_eq!(render_buffer.word_at_byte_offset(8), 0x4444);
    assert_eq!(render_buffer.word_at_byte_offset(10), 0x4444);
    assert_eq!(ram[MESSAGING_RENDER_BUFFER], 0);
    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 2), 0x2222);
    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 4), 0x3333);
    assert_eq!(ram[MESSAGING_RENDER_BUFFER + 6], 0x0f);
    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 8), 0x4444);
}

#[test]
fn native_messaging_render_buffer_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MESSAGING_RENDER_BUFFER + 2, 0xffff);
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, MESSAGING_RENDER_BUFFER + 2, 0x1111);
    native_ram[MESSAGING_RENDER_BUFFER + 6] = 0xf0;
    let mut render_buffer = MessagingRenderBufferState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeMessagingRenderBufferBridgeMut::new(&mut render_buffer, &mut ram);
        bridge.xor_mask(6, 0x0f);
    }

    assert_eq!(render_buffer.word(1), 0x1111);
    assert_eq!(render_buffer.word_at_byte_offset(6), 0x00ff);
    assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 2), 0x1111);
    assert_eq!(ram[MESSAGING_RENDER_BUFFER + 6], 0xff);
}

#[test]
fn vwf_render_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[VWF_ARR + 3] = 0x22;
    write_le_u16(&mut ram, VWF_GLYPH_CURSOR, 0x0040);
    write_le_u16(&mut ram, VWF_FLAG_NEXT_LINE, 1);
    write_le_u16(&mut ram, VWF_CURLINE, 4);
    write_le_u16(&mut ram, VWF_LINE_PTR, 0x0540);
    write_le_u16(&mut ram, VWF_TILE_BUFFER + 0x0c2, 0x3456);

    let mut vwf = VwfRenderState::load_from_ram(&ram);
    assert_eq!(vwf.glyph_advance_prefix_sum(3), 0x22);
    assert_eq!(vwf.glyph_cursor(), 0x0040);
    assert_eq!(vwf.glyph_cursor_usize(), 0x40);
    assert_eq!(vwf.next_line_requested(), 1);
    assert_eq!(vwf.current_line(), 4);
    assert_eq!(vwf.line_render_offset(), 0x0540);
    assert_eq!(vwf.tile_word_at_byte_offset(0x0c2), 0x3456);
    assert_eq!(vwf.tile_word_at_byte_offset(0x0fb), 0);

    vwf.set_next_glyph_advance_prefix_sum(3, 0x33);
    vwf.set_glyph_cursor(0x0080);
    vwf.request_next_line(1);
    vwf.set_current_line(2);
    vwf.set_line_render_offset(0x02a0);
    vwf.set_tile_word_at_byte_offset(0x0c4, 0x789a);
    vwf.write_to_ram(&mut ram);

    assert_eq!(ram[VWF_ARR + 4], 0x33);
    assert_eq!(read_le_u16(&ram, VWF_GLYPH_CURSOR), 0x0080);
    assert_eq!(read_le_u16(&ram, VWF_FLAG_NEXT_LINE), 1);
    assert_eq!(read_le_u16(&ram, VWF_CURLINE), 2);
    assert_eq!(read_le_u16(&ram, VWF_LINE_PTR), 0x02a0);
    assert_eq!(read_le_u16(&ram, VWF_TILE_BUFFER + 0x0c4), 0x789a);
}

#[test]
fn native_vwf_render_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[VWF_ARR + 2] = 0x10;
    write_le_u16(&mut ram, VWF_GLYPH_CURSOR, 0x0002);
    write_le_u16(&mut ram, VWF_FLAG_NEXT_LINE, 1);
    write_le_u16(&mut ram, VWF_CURLINE, 2);
    write_le_u16(&mut ram, VWF_LINE_PTR, 0x02a0);

    let mut vwf = VwfRenderState::load_from_ram(&ram);
    {
        let mut bridge = NativeVwfRenderBridgeMut::new(&mut vwf, &mut ram);
        bridge.set_next_glyph_advance_prefix_sum(2, 0x18);
        assert_eq!(bridge.increment_glyph_cursor(), 3);
        bridge.set_glyph_cursor(0x0040);
        bridge.clear_glyph_cursor();
        bridge.request_next_line(1);
        bridge.clear_next_line_request();
        bridge.set_current_line(4);
        bridge.set_line_render_offset(0x0540);
        bridge.set_tile_word_at_byte_offset(0x0ec, 0xabcd);
    }

    assert_eq!(vwf.glyph_advance_prefix_sum(3), 0x18);
    assert_eq!(vwf.glyph_cursor(), 0);
    assert_eq!(vwf.next_line_requested(), 0);
    assert_eq!(vwf.current_line(), 4);
    assert_eq!(vwf.line_render_offset(), 0x0540);
    assert_eq!(vwf.tile_word_at_byte_offset(0x0ec), 0xabcd);
    assert_eq!(ram[VWF_ARR + 3], 0x18);
    assert_eq!(read_le_u16(&ram, VWF_GLYPH_CURSOR), 0);
    assert_eq!(read_le_u16(&ram, VWF_FLAG_NEXT_LINE), 0);
    assert_eq!(read_le_u16(&ram, VWF_CURLINE), 4);
    assert_eq!(read_le_u16(&ram, VWF_LINE_PTR), 0x0540);
    assert_eq!(read_le_u16(&ram, VWF_TILE_BUFFER + 0x0ec), 0xabcd);
}

#[test]
fn native_vwf_render_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, VWF_GLYPH_CURSOR, 0xffff);
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[VWF_ARR + 2] = 0x10;
    write_le_u16(&mut native_ram, VWF_GLYPH_CURSOR, 0x0002);
    write_le_u16(&mut native_ram, VWF_CURLINE, 4);
    let mut vwf = VwfRenderState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeVwfRenderBridgeMut::new(&mut vwf, &mut ram);
        assert_eq!(bridge.increment_glyph_cursor(), 3);
    }

    assert_eq!(vwf.glyph_cursor(), 3);
    assert_eq!(vwf.current_line(), 4);
    assert_eq!(read_le_u16(&ram, VWF_GLYPH_CURSOR), 3);
    assert_eq!(read_le_u16(&ram, VWF_CURLINE), 4);
    assert_eq!(ram[VWF_ARR + 2], 0x10);
}

#[test]
fn shared_message_timer_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SHARED_MESSAGE_TIMER, 0x0200);

    let mut timer = SharedMessageTimerState::load_from_ram(&ram);
    assert_eq!(timer.timer, 0x0200);
    assert_eq!(timer.tick(), 0x01ff);
    timer.clear();
    timer.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, SHARED_MESSAGE_TIMER), 0);
}

#[test]
fn native_shared_message_timer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SHARED_MESSAGE_TIMER, 0x0000);

    let mut timer = SharedMessageTimerState::load_from_ram(&ram);
    {
        let mut bridge = NativeSharedMessageTimerBridgeMut::new(&mut timer, &mut ram);
        assert_eq!(bridge.tick(), 0xffff);
        bridge.start(0x0df3);
        bridge.clear();
        bridge.start(0x0040);
    }

    assert_eq!(timer.timer, 0x0040);
    assert_eq!(read_le_u16(&ram, SHARED_MESSAGE_TIMER), 0x0040);
}

#[test]
fn native_shared_message_timer_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SHARED_MESSAGE_TIMER, 0xffff);
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, SHARED_MESSAGE_TIMER, 0x0002);
    let mut timer = SharedMessageTimerState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeSharedMessageTimerBridgeMut::new(&mut timer, &mut ram);
        assert_eq!(bridge.tick(), 1);
    }

    assert_eq!(timer.timer, 1);
    assert_eq!(read_le_u16(&ram, SHARED_MESSAGE_TIMER), 1);
}
