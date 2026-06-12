use super::*;

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
