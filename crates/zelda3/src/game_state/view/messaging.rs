use super::*;

pub(crate) struct MessagingStateView<'a> {
    ram: &'a [u8],
}

impl<'a> MessagingStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn module(&self) -> u8 {
        byte(self.ram, MESSAGING_MODULE)
    }

    pub(crate) fn text_render_state(&self) -> u8 {
        byte(self.ram, TEXT_RENDER_STATE)
    }

    pub(crate) fn text_wait_countdown2(&self) -> u8 {
        byte(self.ram, TEXT_WAIT_COUNTDOWN2)
    }

    pub(crate) fn menu_animation_timer(&self) -> u8 {
        byte(self.ram, MENU_ANIMATION_TIMER)
    }

    pub(crate) fn game_over_letter_cursor(&self) -> u8 {
        byte(self.ram, GAME_OVER_LETTER_CURSOR)
    }

    pub(crate) fn effect_index(&self) -> u8 {
        byte(self.ram, GAME_OVER_LETTER_CURSOR)
    }

    pub(crate) fn flag_which_music_type_messaging(&self) -> u8 {
        byte(self.ram, FLAG_WHICH_MUSIC_TYPE_MESSAGING)
    }

    pub(crate) fn dialogue_scroll_speed(&self) -> u8 {
        byte(self.ram, DIALOGUE_SCROLL_SPEED)
    }

    pub(crate) fn text_incremental_state(&self) -> u8 {
        byte(self.ram, TEXT_INCREMENTAL_STATE)
    }

    pub(crate) fn vwf_line_speed_cur(&self) -> u8 {
        byte(self.ram, VWF_LINE_SPEED_CUR)
    }

    pub(crate) fn vwf_line_speed(&self) -> u8 {
        byte(self.ram, VWF_LINE_SPEED)
    }

    pub(crate) fn text_wait_countdown(&self) -> u16 {
        word(self.ram, TEXT_WAIT_COUNTDOWN)
    }

    pub(crate) fn text_msgbox_topleft(&self) -> u16 {
        word(self.ram, TEXT_MSGBOX_TOPLEFT)
    }

    pub(crate) fn text_msgbox_topleft_copy(&self) -> u16 {
        word(self.ram, TEXT_MSGBOX_TOPLEFT_COPY)
    }

    pub(crate) fn text_tilemap_cur(&self) -> u16 {
        word(self.ram, TEXT_TILEMAP_CUR)
    }

    pub(crate) fn dialogue_msg_read_pos(&self) -> u16 {
        word(self.ram, DIALOGUE_MSG_READ_POS)
    }

    pub(crate) fn message_or_sprite_state_cache(&self) -> u8 {
        byte(self.ram, MESSAGE_OR_SPRITE_STATE_CACHE)
    }
}

pub(crate) struct MessagingStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> MessagingStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_module(&mut self, value: u8) {
        self.ram[MESSAGING_MODULE] = value;
    }

    pub(crate) fn clear_module(&mut self) {
        self.ram[MESSAGING_MODULE] = 0;
    }

    pub(crate) fn clear_message_or_sprite_state_cache(&mut self) {
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;
    }

    pub(crate) fn set_text_render_state(&mut self, value: u8) {
        self.ram[TEXT_RENDER_STATE] = value;
    }

    pub(crate) fn increment_text_render_state(&mut self) -> u8 {
        self.ram[TEXT_RENDER_STATE] = self.ram[TEXT_RENDER_STATE].wrapping_add(1);
        self.ram[TEXT_RENDER_STATE]
    }

    pub(crate) fn set_text_wait_countdown2(&mut self, value: u8) {
        self.ram[TEXT_WAIT_COUNTDOWN2] = value;
    }

    pub(crate) fn clear_text_wait_countdown2(&mut self) {
        self.ram[TEXT_WAIT_COUNTDOWN2] = 0;
    }

    pub(crate) fn decrement_text_wait_countdown2(&mut self) -> u8 {
        self.ram[TEXT_WAIT_COUNTDOWN2] = self.ram[TEXT_WAIT_COUNTDOWN2].wrapping_sub(1);
        self.ram[TEXT_WAIT_COUNTDOWN2]
    }

    pub(crate) fn set_menu_animation_timer(&mut self, value: u8) {
        self.ram[MENU_ANIMATION_TIMER] = value;
    }

    pub(crate) fn decrement_menu_animation_timer(&mut self) -> u8 {
        self.ram[MENU_ANIMATION_TIMER] = self.ram[MENU_ANIMATION_TIMER].wrapping_sub(1);
        self.ram[MENU_ANIMATION_TIMER]
    }

    pub(crate) fn set_game_over_letter_cursor(&mut self, value: u8) {
        self.ram[GAME_OVER_LETTER_CURSOR] = value;
    }

    pub(crate) fn set_effect_index(&mut self, value: u8) {
        self.ram[GAME_OVER_LETTER_CURSOR] = value;
    }

    pub(crate) fn or_effect_index(&mut self, value: u8) {
        self.ram[GAME_OVER_LETTER_CURSOR] |= value;
    }

    pub(crate) fn clear_game_over_letter_cursor(&mut self) {
        self.ram[GAME_OVER_LETTER_CURSOR] = 0;
    }

    pub(crate) fn clear_effect_index(&mut self) {
        self.ram[GAME_OVER_LETTER_CURSOR] = 0;
    }

    pub(crate) fn increment_game_over_letter_cursor(&mut self) -> u8 {
        self.ram[GAME_OVER_LETTER_CURSOR] = self.ram[GAME_OVER_LETTER_CURSOR].wrapping_add(1);
        self.ram[GAME_OVER_LETTER_CURSOR]
    }

    pub(crate) fn decrement_game_over_letter_cursor(&mut self) -> u8 {
        self.ram[GAME_OVER_LETTER_CURSOR] = self.ram[GAME_OVER_LETTER_CURSOR].wrapping_sub(1);
        self.ram[GAME_OVER_LETTER_CURSOR]
    }

    pub(crate) fn clear_flag_which_music_type_messaging(&mut self) {
        self.ram[FLAG_WHICH_MUSIC_TYPE_MESSAGING] = 0;
    }

    pub(crate) fn xor_message_or_sprite_state_cache(&mut self, value: u8) {
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] ^= value;
    }

    pub(crate) fn increment_text_incremental_state(&mut self) {
        self.ram[TEXT_INCREMENTAL_STATE] = self.ram[TEXT_INCREMENTAL_STATE].wrapping_add(1);
    }

    pub(crate) fn decrement_vwf_line_speed_cur(&mut self) {
        self.ram[VWF_LINE_SPEED_CUR] = self.ram[VWF_LINE_SPEED_CUR].wrapping_sub(1);
    }

    pub(crate) fn set_vwf_line_speed(&mut self, value: u8) {
        self.ram[VWF_LINE_SPEED] = value;
    }

    pub(crate) fn set_vwf_line_speed_cur(&mut self, value: u8) {
        self.ram[VWF_LINE_SPEED_CUR] = value;
    }

    pub(crate) fn set_dialogue_scroll_speed(&mut self, value: u8) {
        self.ram[DIALOGUE_SCROLL_SPEED] = value;
    }

    pub(crate) fn set_text_wait_countdown(&mut self, value: u16) {
        write_le_u16(self.ram, TEXT_WAIT_COUNTDOWN, value);
    }

    pub(crate) fn clear_text_wait_countdown(&mut self) {
        self.ram[TEXT_WAIT_COUNTDOWN] = 0;
    }

    pub(crate) fn set_text_msgbox_topleft(&mut self, value: u16) {
        write_le_u16(self.ram, TEXT_MSGBOX_TOPLEFT, value);
    }

    pub(crate) fn set_text_msgbox_topleft_copy(&mut self, value: u16) {
        write_le_u16(self.ram, TEXT_MSGBOX_TOPLEFT_COPY, value);
    }

    pub(crate) fn set_text_tilemap_cur(&mut self, value: u16) {
        write_le_u16(self.ram, TEXT_TILEMAP_CUR, value);
    }

    pub(crate) fn clear_dialogue_msg_read_pos(&mut self) {
        write_le_u16(self.ram, DIALOGUE_MSG_READ_POS, 0);
    }

    pub(crate) fn set_dialogue_msg_read_pos(&mut self, value: u16) {
        write_le_u16(self.ram, DIALOGUE_MSG_READ_POS, value);
    }

    pub(crate) fn init_msgbox_state_from(&mut self, data: &[u8]) {
        self.ram[TEXT_MSGBOX_TOPLEFT_COPY..TEXT_MSGBOX_TOPLEFT_COPY + data.len()]
            .copy_from_slice(data);
    }
}

pub(crate) struct MessagingTextView<'a> {
    ram: &'a [u8],
}

impl<'a> MessagingTextView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn byte(&self, offset: usize) -> u8 {
        byte(self.ram, MESSAGING_TEXT_BUFFER + offset)
    }

    pub(crate) fn next_byte(&self, offset: usize) -> Option<u8> {
        self.ram.get(MESSAGING_TEXT_BUFFER + offset + 1).copied()
    }
}

pub(crate) struct MessagingTextViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> MessagingTextViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn load_decoded_dialogue(&mut self, decoded: &[u8]) -> usize {
        let len = decoded
            .len()
            .min(self.ram.len().saturating_sub(MESSAGING_TEXT_BUFFER));
        self.ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + len]
            .copy_from_slice(&decoded[..len]);
        len
    }

    pub(crate) fn write_decoded_text_at(&mut self, dst: usize, decoded: &[u8]) -> usize {
        let len = decoded.len().min(self.ram.len().saturating_sub(dst));
        self.ram[dst..dst + len].copy_from_slice(&decoded[..len]);
        len
    }

    pub(crate) fn set_dialogue_pointer(&mut self, index: usize, pointer: u32) {
        let dst = TEXT_DIALOGUE_POINTERS + index * 3;
        self.ram[dst] = pointer as u8;
        self.ram[dst + 1] = (pointer >> 8) as u8;
        self.ram[dst + 2] = (pointer >> 16) as u8;
    }
}

pub(crate) struct MessagingRenderBufferView<'a> {
    ram: &'a [u8],
}

impl<'a> MessagingRenderBufferView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn word(&self, index: usize) -> u16 {
        word(self.ram, MESSAGING_RENDER_BUFFER + index * 2)
    }

    pub(crate) fn word_at_byte_offset(&self, byte_offset: usize) -> u16 {
        word(self.ram, MESSAGING_RENDER_BUFFER + byte_offset)
    }
}

pub(crate) struct MessagingRenderBufferViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> MessagingRenderBufferViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn xor_mask(&mut self, offset: usize, mask: u8) {
        self.ram[MESSAGING_RENDER_BUFFER + offset] ^= mask;
    }

    pub(crate) fn clear_mask(&mut self, offset: usize, mask: u8) {
        self.ram[MESSAGING_RENDER_BUFFER + offset] &= !mask;
    }

    pub(crate) fn set_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, MESSAGING_RENDER_BUFFER + index * 2, value);
    }

    pub(crate) fn set_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        write_le_u16(self.ram, MESSAGING_RENDER_BUFFER + byte_offset, value);
    }

    pub(crate) fn clear_range(&mut self, byte_count: usize) {
        self.ram[MESSAGING_RENDER_BUFFER..MESSAGING_RENDER_BUFFER + byte_count].fill(0);
    }

    pub(crate) fn fill_word_range(&mut self, start_index: usize, count: usize, value: u16) {
        for i in 0..count {
            write_le_u16(
                self.ram,
                MESSAGING_RENDER_BUFFER + (start_index + i) * 2,
                value,
            );
        }
    }
}

pub(crate) struct VwfGlyphSpacingView<'a> {
    ram: &'a [u8],
}

impl<'a> VwfGlyphSpacingView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn offset(&self, index: usize) -> u8 {
        byte(self.ram, VWF_ARR + index)
    }

    pub(crate) fn cursor(&self) -> u16 {
        word(self.ram, VWF_GLYPH_CURSOR)
    }

    pub(crate) fn cursor_usize(&self) -> usize {
        usize::from(self.cursor())
    }

    pub(crate) fn vwf_flag_next_line(&self) -> u16 {
        word(self.ram, VWF_FLAG_NEXT_LINE)
    }

    pub(crate) fn vwf_curline(&self) -> u16 {
        word(self.ram, VWF_CURLINE)
    }

    pub(crate) fn vwf_line_ptr(&self) -> u16 {
        word(self.ram, VWF_LINE_PTR)
    }

    pub(crate) fn vwf_tile_buffer_word_at(&self, byte_offset: usize) -> u16 {
        word(self.ram, VWF_TILE_BUFFER + byte_offset)
    }
}

pub(crate) struct VwfGlyphSpacingViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> VwfGlyphSpacingViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_next_offset(&mut self, index: usize, value: u8) {
        self.ram[VWF_ARR + index + 1] = value;
    }

    pub(crate) fn set_cursor(&mut self, value: u16) {
        write_le_u16(self.ram, VWF_GLYPH_CURSOR, value);
    }

    pub(crate) fn clear_cursor(&mut self) {
        self.set_cursor(0);
    }

    pub(crate) fn increment_cursor(&mut self) -> u16 {
        let value = word(self.ram, VWF_GLYPH_CURSOR).wrapping_add(1);
        self.set_cursor(value);
        value
    }

    pub(crate) fn set_vwf_flag_next_line(&mut self, value: u16) {
        write_le_u16(self.ram, VWF_FLAG_NEXT_LINE, value);
    }

    pub(crate) fn clear_vwf_flag_next_line(&mut self) {
        write_le_u16(self.ram, VWF_FLAG_NEXT_LINE, 0);
    }

    pub(crate) fn set_vwf_curline(&mut self, value: u16) {
        write_le_u16(self.ram, VWF_CURLINE, value);
    }

    pub(crate) fn set_vwf_line_ptr(&mut self, value: u16) {
        write_le_u16(self.ram, VWF_LINE_PTR, value);
    }

    pub(crate) fn set_vwf_tile_buffer_word(&mut self, byte_offset: usize, value: u16) {
        write_le_u16(self.ram, VWF_TILE_BUFFER + byte_offset, value);
    }
}

pub(crate) struct DialogueSourceOffsetViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DialogueSourceOffsetViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn increment_bank_offset_low_nibble(&mut self) -> u8 {
        let next = self.ram[DIALOGUE_MSG_SRC_OFFS + 2].wrapping_add(1);
        self.ram[DIALOGUE_MSG_SRC_OFFS + 2] = next;
        next
    }
}

pub(crate) struct SelectFileScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> SelectFileScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn choice(&self, index: usize) -> u8 {
        byte(self.ram, SELECT_FILE_CHOICE_WORK + index)
    }

    pub(crate) fn cursor(&self) -> u8 {
        byte(self.ram, SELECT_FILE_CURSOR_WORK)
    }

    pub(crate) fn cursor_usize(&self) -> usize {
        usize::from(self.cursor())
    }

    pub(crate) fn remembered_cursor(&self) -> u8 {
        byte(self.ram, SELECT_FILE_REMEMBERED_CURSOR)
    }

    pub(crate) fn target_word(&self) -> u16 {
        word(self.ram, SELECT_FILE_TARGET_WORK)
    }

    pub(crate) fn copy_source_slot_x2(&self) -> u16 {
        word(self.ram, SELECT_FILE_COPY_SOURCE_SLOT_X2)
    }

    pub(crate) fn copy_source_slot(&self) -> usize {
        usize::from(self.copy_source_slot_x2() >> 1)
    }

    pub(crate) fn name_scroll_x(&self) -> u16 {
        word(self.ram, SELECT_FILE_NAME_SCROLL_X)
    }

    pub(crate) fn name_column(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_COLUMN)
    }

    pub(crate) fn name_column_usize(&self) -> usize {
        usize::from(self.name_column())
    }

    pub(crate) fn name_cursor_y(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_CURSOR_Y)
    }

    pub(crate) fn name_slot(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_SLOT)
    }

    pub(crate) fn name_slot_usize(&self) -> usize {
        usize::from(self.name_slot())
    }

    pub(crate) fn name_scroll_x_step(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_SCROLL_X_STEP)
    }

    pub(crate) fn name_scroll_y_step(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_SCROLL_Y_STEP)
    }

    pub(crate) fn is_name_scrolling(&self) -> bool {
        (self.name_scroll_x_step() | self.name_scroll_y_step()) != 0
    }

    pub(crate) fn name_row(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_ROW)
    }

    pub(crate) fn name_row_usize(&self) -> usize {
        usize::from(self.name_row())
    }

    pub(crate) fn name_scroll_x_direction(&self) -> u8 {
        byte(self.ram, SELECT_FILE_NAME_SCROLL_X_DIRECTION)
    }

    pub(crate) fn save_slot_flag(&self, slot: usize) -> u16 {
        word(self.ram, SELECTFILE_SAVE_SLOT_FLAGS + slot * 2)
    }

    pub(crate) fn save_slot_flags(&self) -> [u16; 3] {
        [
            self.save_slot_flag(0),
            self.save_slot_flag(1),
            self.save_slot_flag(2),
        ]
    }

    pub(crate) fn any_save_slot_flag(&self) -> bool {
        self.save_slot_flags().into_iter().any(|flag| flag != 0)
    }
}

pub(crate) struct SelectFileScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SelectFileScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_choice(&mut self, index: usize, value: u8) {
        self.ram[SELECT_FILE_CHOICE_WORK + index] = value;
    }

    pub(crate) fn set_cursor(&mut self, value: u8) {
        self.ram[SELECT_FILE_CURSOR_WORK] = value;
    }

    pub(crate) fn clear_cursor(&mut self) {
        self.set_cursor(0);
    }

    pub(crate) fn clear_transition_scratch(&mut self) {
        self.ram[SELECT_FILE_TRANSITION_WORK] = 0;
    }

    pub(crate) fn increment_cursor(&mut self) -> u8 {
        self.ram[SELECT_FILE_CURSOR_WORK] = self.ram[SELECT_FILE_CURSOR_WORK].wrapping_add(1);
        self.ram[SELECT_FILE_CURSOR_WORK]
    }

    pub(crate) fn decrement_cursor(&mut self) -> u8 {
        self.ram[SELECT_FILE_CURSOR_WORK] = self.ram[SELECT_FILE_CURSOR_WORK].wrapping_sub(1);
        self.ram[SELECT_FILE_CURSOR_WORK]
    }

    pub(crate) fn set_remembered_cursor(&mut self, value: u8) {
        self.ram[SELECT_FILE_REMEMBERED_CURSOR] = value;
    }

    pub(crate) fn clear_remembered_cursor(&mut self) {
        self.set_remembered_cursor(0);
    }

    pub(crate) fn remember_current_cursor(&mut self) {
        self.ram[SELECT_FILE_REMEMBERED_CURSOR] = self.ram[SELECT_FILE_CURSOR_WORK];
    }

    pub(crate) fn restore_remembered_cursor(&mut self) {
        self.ram[SELECT_FILE_CURSOR_WORK] = self.ram[SELECT_FILE_REMEMBERED_CURSOR];
    }

    pub(crate) fn set_target_word(&mut self, value: u16) {
        write_le_u16(self.ram, SELECT_FILE_TARGET_WORK, value);
    }

    pub(crate) fn set_copy_source_slot_x2(&mut self, value: u16) {
        write_le_u16(self.ram, SELECT_FILE_COPY_SOURCE_SLOT_X2, value);
    }

    pub(crate) fn set_copy_source_slot(&mut self, slot: u8) {
        self.set_copy_source_slot_x2(u16::from(slot) * 2);
    }

    pub(crate) fn set_name_scroll_x(&mut self, value: u16) {
        write_le_u16(self.ram, SELECT_FILE_NAME_SCROLL_X, value);
    }

    pub(crate) fn clear_name_entry_state(&mut self) {
        self.ram[SELECT_FILE_NAME_COLUMN] = 0;
        self.ram[SELECT_FILE_NAME_SLOT] = 0;
        self.ram[SELECT_FILE_NAME_ROW] = 0;
        self.ram[SELECT_FILE_CHOICE_WORK] = 0;
        self.ram[SELECT_FILE_COPY_SOURCE_SLOT_X2] = 0;
        self.ram[SELECT_FILE_NAME_CURSOR_Y] = 0x83;
        write_le_u16(self.ram, SELECT_FILE_NAME_SCROLL_X, 0x01f0);
    }

    pub(crate) fn set_name_column(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_COLUMN] = value;
    }

    pub(crate) fn set_name_cursor_y(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_CURSOR_Y] = value;
    }

    pub(crate) fn step_name_cursor_y_toward(&mut self, target_y: u8) -> bool {
        let diff = self.ram[SELECT_FILE_NAME_CURSOR_Y].wrapping_sub(target_y);
        if diff == 0 {
            return false;
        }
        self.ram[SELECT_FILE_NAME_CURSOR_Y] = if diff & 0x80 != 0 {
            self.ram[SELECT_FILE_NAME_CURSOR_Y].wrapping_add(2)
        } else {
            self.ram[SELECT_FILE_NAME_CURSOR_Y].wrapping_sub(2)
        };
        true
    }

    pub(crate) fn set_name_slot(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_SLOT] = value;
    }

    pub(crate) fn move_name_slot_left_wrapped(&mut self) -> u8 {
        let next = if self.ram[SELECT_FILE_NAME_SLOT] == 0 {
            5
        } else {
            self.ram[SELECT_FILE_NAME_SLOT].wrapping_sub(1)
        };
        self.ram[SELECT_FILE_NAME_SLOT] = next;
        next
    }

    pub(crate) fn move_name_slot_right_wrapped(&mut self) -> u8 {
        self.ram[SELECT_FILE_NAME_SLOT] = self.ram[SELECT_FILE_NAME_SLOT].wrapping_add(1);
        if self.ram[SELECT_FILE_NAME_SLOT] == 6 {
            self.ram[SELECT_FILE_NAME_SLOT] = 0;
        }
        self.ram[SELECT_FILE_NAME_SLOT]
    }

    pub(crate) fn set_name_scroll_x_step(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_SCROLL_X_STEP] = value;
    }

    pub(crate) fn advance_name_scroll_x_step_by(&mut self, value: u8) -> u8 {
        self.ram[SELECT_FILE_NAME_SCROLL_X_STEP] =
            self.ram[SELECT_FILE_NAME_SCROLL_X_STEP].wrapping_add(value);
        self.ram[SELECT_FILE_NAME_SCROLL_X_STEP]
    }

    pub(crate) fn set_name_scroll_y_step(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP] = value;
    }

    pub(crate) fn clear_name_scroll_y_step(&mut self) {
        self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP] = 0;
    }

    pub(crate) fn increment_name_scroll_y_step(&mut self) -> u8 {
        self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP] =
            self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP].wrapping_add(1);
        self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP]
    }

    pub(crate) fn set_name_row(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_ROW] = value;
    }

    pub(crate) fn set_name_scroll_x_direction(&mut self, value: u8) {
        self.ram[SELECT_FILE_NAME_SCROLL_X_DIRECTION] = value;
    }

    pub(crate) fn set_save_slot_flag(&mut self, slot: usize, value: u16) {
        write_le_u16(self.ram, SELECTFILE_SAVE_SLOT_FLAGS + slot * 2, value);
    }

    pub(crate) fn mark_save_slot_present(&mut self, slot: usize) {
        self.set_save_slot_flag(slot, 1);
    }

    pub(crate) fn clear_save_slot_flag(&mut self, slot: usize) {
        self.set_save_slot_flag(slot, 0);
    }

    pub(crate) fn clear_save_slot_flags(&mut self) {
        self.ram[SELECTFILE_SAVE_SLOT_FLAGS..SELECTFILE_SAVE_SLOT_FLAGS + 6].fill(0);
    }
}
