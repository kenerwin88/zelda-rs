use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const MESSAGING_RENDER_BUFFER_LEN: usize = 0x7e0;
const DECODED_MESSAGE_TEXT_CAPACITY: usize = 0x400;
const DIALOGUE_POINTER_COUNT: usize = 398;
const VWF_GLYPH_ADVANCE_BUFFER_LEN: usize = 0x100;
const VWF_TILE_BUFFER_LEN: usize = 6 * 21 * 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueMessageIndexState {
    value: u16,
}

impl DialogueMessageIndexState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            value: read_le_u16(ram, DIALOGUE_MESSAGE_INDEX),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DIALOGUE_MESSAGE_INDEX, self.value);
    }

    pub(crate) fn value(&self) -> u16 {
        self.value
    }

    pub(crate) fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MultiselectChoiceState {
    value: u8,
    backup: u8,
}

impl MultiselectChoiceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            value: ram.get(MULTISELECT_CHOICE).copied().unwrap_or(0),
            backup: ram.get(MULTISELECT_CHOICE_BACKUP).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MULTISELECT_CHOICE] = self.value;
        ram[MULTISELECT_CHOICE_BACKUP] = self.backup;
    }

    pub(crate) fn value(&self) -> u8 {
        self.value
    }

    pub(crate) fn backup(&self) -> u8 {
        self.backup
    }

    pub(crate) fn set_value(&mut self, value: u8) {
        self.value = value;
    }

    pub(crate) fn increment_value(&mut self) {
        self.set_value(self.value().wrapping_add(1));
    }

    pub(crate) fn decrement_value(&mut self) {
        self.set_value(self.value().wrapping_sub(1));
    }

    pub(crate) fn restore_backup(&mut self) {
        self.set_value(self.backup);
    }

    pub(crate) fn save_backup(&mut self) {
        self.backup = self.value();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueNumberState {
    low_pair: u8,
    high_pair: u8,
}

impl DialogueNumberState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            low_pair: ram.get(DIALOGUE_NUMBER_LO).copied().unwrap_or(0),
            high_pair: ram.get(DIALOGUE_NUMBER_HI).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DIALOGUE_NUMBER_LO] = self.low_pair;
        ram[DIALOGUE_NUMBER_HI] = self.high_pair;
    }

    pub(crate) fn packed_digits(&self, pair_index: usize) -> u8 {
        match pair_index {
            0 => self.low_pair,
            1 => self.high_pair,
            _ => 0,
        }
    }

    pub(crate) fn set_packed_digits(&mut self, low_pair: u8, high_pair: u8) {
        self.low_pair = low_pair;
        self.high_pair = high_pair;
    }

    pub(crate) fn set_low_pair(&mut self, value: u8) {
        self.low_pair = value;
    }

    pub(crate) fn set_high_pair(&mut self, value: u8) {
        self.high_pair = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueSourceOffsetState {
    bank_offset_low_nibble: u8,
}

impl DialogueSourceOffsetState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            bank_offset_low_nibble: ram.get(DIALOGUE_MSG_SRC_OFFS + 2).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DIALOGUE_MSG_SRC_OFFS + 2] = self.bank_offset_low_nibble;
    }

    pub(crate) fn bank_offset_low_nibble(&self) -> u8 {
        self.bank_offset_low_nibble
    }

    pub(crate) fn increment_bank_offset_low_nibble(&mut self) -> u8 {
        self.bank_offset_low_nibble = self.bank_offset_low_nibble.wrapping_add(1);
        self.bank_offset_low_nibble
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DecodedMessageTextState {
    bytes: Vec<u8>,
}

impl Default for DecodedMessageTextState {
    fn default() -> Self {
        Self {
            bytes: vec![0; DECODED_MESSAGE_TEXT_CAPACITY],
        }
    }
}

impl DecodedMessageTextState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut bytes = vec![0; DECODED_MESSAGE_TEXT_CAPACITY];
        if let Some(src) = ram.get(MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + bytes.len()) {
            bytes.copy_from_slice(src);
        }
        Self { bytes }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + self.bytes.len()]
            .copy_from_slice(&self.bytes);
    }

    pub(crate) fn byte(&self, offset: usize) -> u8 {
        self.bytes.get(offset).copied().unwrap_or(0)
    }

    pub(crate) fn next_byte(&self, offset: usize) -> Option<u8> {
        self.bytes.get(offset + 1).copied()
    }

    pub(crate) fn load_decoded_dialogue(&mut self, decoded: &[u8]) -> usize {
        let len = decoded.len().min(self.bytes.len());
        self.bytes[..len].copy_from_slice(&decoded[..len]);
        len
    }

    pub(crate) fn write_decoded_text_at(&mut self, dst: usize, decoded: &[u8]) -> usize {
        let Some(start) = dst.checked_sub(MESSAGING_TEXT_BUFFER) else {
            return 0;
        };
        if start > self.bytes.len() {
            return 0;
        }
        let len = decoded.len().min(self.bytes.len().saturating_sub(start));
        self.bytes[start..start + len].copy_from_slice(&decoded[..len]);
        len
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialoguePointerTableState {
    pointers: Vec<u32>,
}

impl Default for DialoguePointerTableState {
    fn default() -> Self {
        Self {
            pointers: vec![0; DIALOGUE_POINTER_COUNT],
        }
    }
}

impl DialoguePointerTableState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut pointers = vec![0; DIALOGUE_POINTER_COUNT];
        for (index, pointer) in pointers.iter_mut().enumerate() {
            let src = TEXT_DIALOGUE_POINTERS + index * 3;
            if src + 2 < ram.len() {
                *pointer = u32::from(ram[src])
                    | (u32::from(ram[src + 1]) << 8)
                    | (u32::from(ram[src + 2]) << 16);
            }
        }
        Self { pointers }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (index, pointer) in self.pointers.iter().copied().enumerate() {
            let dst = TEXT_DIALOGUE_POINTERS + index * 3;
            ram[dst] = pointer as u8;
            ram[dst + 1] = (pointer >> 8) as u8;
            ram[dst + 2] = (pointer >> 16) as u8;
        }
    }

    pub(crate) fn pointer(&self, index: usize) -> u32 {
        self.pointers.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn set_pointer(&mut self, index: usize, pointer: u32) {
        self.pointers[index] = pointer & 0x00ff_ffff;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MessagingRuntimeState {
    pub(crate) module: u8,
    pub(crate) text_render_state: u8,
    pub(crate) text_wait_countdown2: u8,
    pub(crate) menu_animation_timer: u8,
    pub(crate) game_over_letter_cursor: u8,
    pub(crate) flag_which_music_type_messaging: u8,
    pub(crate) dialogue_scroll_speed: u8,
    pub(crate) text_incremental_state: u8,
    pub(crate) vwf_line_speed_cur: u8,
    pub(crate) vwf_line_speed: u8,
    pub(crate) text_wait_countdown: u16,
    pub(crate) text_msgbox_topleft: u16,
    pub(crate) text_msgbox_topleft_copy: u16,
    pub(crate) text_tilemap_cur: u16,
    pub(crate) dialogue_msg_read_pos: u16,
    pub(crate) message_or_sprite_state_cache: u8,
}

impl MessagingRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            module: ram.get(MESSAGING_MODULE).copied().unwrap_or(0),
            text_render_state: ram.get(TEXT_RENDER_STATE).copied().unwrap_or(0),
            text_wait_countdown2: ram.get(TEXT_WAIT_COUNTDOWN2).copied().unwrap_or(0),
            menu_animation_timer: ram.get(MENU_ANIMATION_TIMER).copied().unwrap_or(0),
            game_over_letter_cursor: ram.get(GAME_OVER_LETTER_CURSOR).copied().unwrap_or(0),
            flag_which_music_type_messaging: ram
                .get(FLAG_WHICH_MUSIC_TYPE_MESSAGING)
                .copied()
                .unwrap_or(0),
            dialogue_scroll_speed: ram.get(DIALOGUE_SCROLL_SPEED).copied().unwrap_or(0),
            text_incremental_state: ram.get(TEXT_INCREMENTAL_STATE).copied().unwrap_or(0),
            vwf_line_speed_cur: ram.get(VWF_LINE_SPEED_CUR).copied().unwrap_or(0),
            vwf_line_speed: ram.get(VWF_LINE_SPEED).copied().unwrap_or(0),
            text_wait_countdown: read_le_u16(ram, TEXT_WAIT_COUNTDOWN),
            text_msgbox_topleft: read_le_u16(ram, TEXT_MSGBOX_TOPLEFT),
            text_msgbox_topleft_copy: read_le_u16(ram, TEXT_MSGBOX_TOPLEFT_COPY),
            text_tilemap_cur: read_le_u16(ram, TEXT_TILEMAP_CUR),
            dialogue_msg_read_pos: read_le_u16(ram, DIALOGUE_MSG_READ_POS),
            message_or_sprite_state_cache: ram
                .get(MESSAGE_OR_SPRITE_STATE_CACHE)
                .copied()
                .unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MESSAGING_MODULE] = self.module;
        ram[TEXT_RENDER_STATE] = self.text_render_state;
        ram[TEXT_WAIT_COUNTDOWN2] = self.text_wait_countdown2;
        ram[MENU_ANIMATION_TIMER] = self.menu_animation_timer;
        ram[GAME_OVER_LETTER_CURSOR] = self.game_over_letter_cursor;
        ram[FLAG_WHICH_MUSIC_TYPE_MESSAGING] = self.flag_which_music_type_messaging;
        ram[DIALOGUE_SCROLL_SPEED] = self.dialogue_scroll_speed;
        ram[TEXT_INCREMENTAL_STATE] = self.text_incremental_state;
        ram[VWF_LINE_SPEED_CUR] = self.vwf_line_speed_cur;
        ram[VWF_LINE_SPEED] = self.vwf_line_speed;
        write_le_u16(ram, TEXT_WAIT_COUNTDOWN, self.text_wait_countdown);
        write_le_u16(ram, TEXT_MSGBOX_TOPLEFT, self.text_msgbox_topleft);
        write_le_u16(ram, TEXT_MSGBOX_TOPLEFT_COPY, self.text_msgbox_topleft_copy);
        write_le_u16(ram, TEXT_TILEMAP_CUR, self.text_tilemap_cur);
        write_le_u16(ram, DIALOGUE_MSG_READ_POS, self.dialogue_msg_read_pos);
        ram[MESSAGE_OR_SPRITE_STATE_CACHE] = self.message_or_sprite_state_cache;
    }

    pub(crate) fn module(&self) -> u8 {
        self.module
    }

    pub(crate) fn text_render_state(&self) -> u8 {
        self.text_render_state
    }

    pub(crate) fn text_wait_countdown2(&self) -> u8 {
        self.text_wait_countdown2
    }

    pub(crate) fn menu_animation_timer(&self) -> u8 {
        self.menu_animation_timer
    }

    pub(crate) fn game_over_letter_cursor(&self) -> u8 {
        self.game_over_letter_cursor
    }

    pub(crate) fn effect_index(&self) -> u8 {
        self.game_over_letter_cursor
    }

    pub(crate) fn flag_which_music_type_messaging(&self) -> u8 {
        self.flag_which_music_type_messaging
    }

    pub(crate) fn dialogue_scroll_speed(&self) -> u8 {
        self.dialogue_scroll_speed
    }

    pub(crate) fn text_incremental_state(&self) -> u8 {
        self.text_incremental_state
    }

    pub(crate) fn vwf_line_speed_cur(&self) -> u8 {
        self.vwf_line_speed_cur
    }

    pub(crate) fn vwf_line_speed(&self) -> u8 {
        self.vwf_line_speed
    }

    pub(crate) fn text_wait_countdown(&self) -> u16 {
        self.text_wait_countdown
    }

    pub(crate) fn text_msgbox_topleft(&self) -> u16 {
        self.text_msgbox_topleft
    }

    pub(crate) fn text_msgbox_topleft_copy(&self) -> u16 {
        self.text_msgbox_topleft_copy
    }

    pub(crate) fn text_tilemap_cur(&self) -> u16 {
        self.text_tilemap_cur
    }

    pub(crate) fn dialogue_msg_read_pos(&self) -> u16 {
        self.dialogue_msg_read_pos
    }

    pub(crate) fn message_or_sprite_state_cache(&self) -> u8 {
        self.message_or_sprite_state_cache
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SharedMessageTimerState {
    pub(crate) timer: u16,
}

impl SharedMessageTimerState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            timer: if SHARED_MESSAGE_TIMER + 1 < ram.len() {
                read_le_u16(ram, SHARED_MESSAGE_TIMER)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, SHARED_MESSAGE_TIMER, self.timer);
    }

    pub(crate) fn tick(&mut self) -> u16 {
        self.timer = self.timer.wrapping_sub(1);
        self.timer
    }

    pub(crate) fn clear(&mut self) {
        self.timer = 0;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MessagingRenderBufferState {
    bytes: Vec<u8>,
}

impl Default for MessagingRenderBufferState {
    fn default() -> Self {
        Self {
            bytes: vec![0; MESSAGING_RENDER_BUFFER_LEN],
        }
    }
}

impl MessagingRenderBufferState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut bytes = vec![0; MESSAGING_RENDER_BUFFER_LEN];
        if let Some(src) = ram.get(MESSAGING_RENDER_BUFFER..MESSAGING_RENDER_BUFFER + bytes.len()) {
            bytes.copy_from_slice(src);
        }
        Self { bytes }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MESSAGING_RENDER_BUFFER..MESSAGING_RENDER_BUFFER + self.bytes.len()]
            .copy_from_slice(&self.bytes);
    }

    pub(crate) fn word(&self, index: usize) -> u16 {
        self.word_at_byte_offset(index * 2)
    }

    pub(crate) fn word_at_byte_offset(&self, byte_offset: usize) -> u16 {
        if byte_offset + 1 < self.bytes.len() {
            read_le_u16(&self.bytes, byte_offset)
        } else {
            0
        }
    }

    pub(crate) fn xor_mask(&mut self, offset: usize, mask: u8) {
        self.bytes[offset] ^= mask;
    }

    pub(crate) fn clear_mask(&mut self, offset: usize, mask: u8) {
        self.bytes[offset] &= !mask;
    }

    pub(crate) fn set_word(&mut self, index: usize, value: u16) {
        self.set_word_at_byte_offset(index * 2, value);
    }

    pub(crate) fn set_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        write_le_u16(&mut self.bytes, byte_offset, value);
    }

    pub(crate) fn clear_range(&mut self, byte_count: usize) {
        self.bytes[..byte_count].fill(0);
    }

    pub(crate) fn fill_word_range(&mut self, start_index: usize, count: usize, value: u16) {
        for i in 0..count {
            self.set_word(start_index + i, value);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct VwfRenderState {
    glyph_advance_prefix_sums: Vec<u8>,
    glyph_cursor: u16,
    next_line_requested: u16,
    current_line: u16,
    line_render_offset: u16,
    tile_words: Vec<u8>,
}

impl Default for VwfRenderState {
    fn default() -> Self {
        Self {
            glyph_advance_prefix_sums: vec![0; VWF_GLYPH_ADVANCE_BUFFER_LEN],
            glyph_cursor: 0,
            next_line_requested: 0,
            current_line: 0,
            line_render_offset: 0,
            tile_words: vec![0; VWF_TILE_BUFFER_LEN],
        }
    }
}

impl VwfRenderState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut glyph_advance_prefix_sums = vec![0; VWF_GLYPH_ADVANCE_BUFFER_LEN];
        if let Some(src) = ram.get(VWF_ARR..VWF_ARR + glyph_advance_prefix_sums.len()) {
            glyph_advance_prefix_sums.copy_from_slice(src);
        }

        let mut tile_words = vec![0; VWF_TILE_BUFFER_LEN];
        if let Some(src) = ram.get(VWF_TILE_BUFFER..VWF_TILE_BUFFER + tile_words.len()) {
            tile_words.copy_from_slice(src);
        }

        Self {
            glyph_advance_prefix_sums,
            glyph_cursor: read_le_u16(ram, VWF_GLYPH_CURSOR),
            next_line_requested: read_le_u16(ram, VWF_FLAG_NEXT_LINE),
            current_line: read_le_u16(ram, VWF_CURLINE),
            line_render_offset: read_le_u16(ram, VWF_LINE_PTR),
            tile_words,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[VWF_ARR..VWF_ARR + self.glyph_advance_prefix_sums.len()]
            .copy_from_slice(&self.glyph_advance_prefix_sums);
        write_le_u16(ram, VWF_GLYPH_CURSOR, self.glyph_cursor);
        write_le_u16(ram, VWF_FLAG_NEXT_LINE, self.next_line_requested);
        write_le_u16(ram, VWF_CURLINE, self.current_line);
        write_le_u16(ram, VWF_LINE_PTR, self.line_render_offset);
        ram[VWF_TILE_BUFFER..VWF_TILE_BUFFER + self.tile_words.len()]
            .copy_from_slice(&self.tile_words);
    }

    pub(crate) fn glyph_advance_prefix_sum(&self, index: usize) -> u8 {
        self.glyph_advance_prefix_sums
            .get(index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn glyph_cursor(&self) -> u16 {
        self.glyph_cursor
    }

    pub(crate) fn glyph_cursor_usize(&self) -> usize {
        usize::from(self.glyph_cursor())
    }

    pub(crate) fn next_line_requested(&self) -> u16 {
        self.next_line_requested
    }

    pub(crate) fn current_line(&self) -> u16 {
        self.current_line
    }

    pub(crate) fn line_render_offset(&self) -> u16 {
        self.line_render_offset
    }

    pub(crate) fn tile_word_at_byte_offset(&self, byte_offset: usize) -> u16 {
        if byte_offset + 1 < self.tile_words.len() {
            read_le_u16(&self.tile_words, byte_offset)
        } else {
            0
        }
    }

    pub(crate) fn set_next_glyph_advance_prefix_sum(&mut self, index: usize, value: u8) {
        self.glyph_advance_prefix_sums[index + 1] = value;
    }

    pub(crate) fn set_glyph_cursor(&mut self, value: u16) {
        self.glyph_cursor = value;
    }

    pub(crate) fn clear_glyph_cursor(&mut self) {
        self.set_glyph_cursor(0);
    }

    pub(crate) fn increment_glyph_cursor(&mut self) -> u16 {
        self.glyph_cursor = self.glyph_cursor.wrapping_add(1);
        self.glyph_cursor
    }

    pub(crate) fn request_next_line(&mut self, value: u16) {
        self.next_line_requested = value;
    }

    pub(crate) fn clear_next_line_request(&mut self) {
        self.next_line_requested = 0;
    }

    pub(crate) fn set_current_line(&mut self, value: u16) {
        self.current_line = value;
    }

    pub(crate) fn set_line_render_offset(&mut self, value: u16) {
        self.line_render_offset = value;
    }

    pub(crate) fn set_tile_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        write_le_u16(&mut self.tile_words, byte_offset, value);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MessagingState {
    pub(crate) dialogue_message_index: DialogueMessageIndexState,
    pub(crate) multiselect_choice: MultiselectChoiceState,
    pub(crate) dialogue_number: DialogueNumberState,
    pub(crate) dialogue_source_offset: DialogueSourceOffsetState,
    pub(crate) decoded_text: DecodedMessageTextState,
    pub(crate) dialogue_pointers: DialoguePointerTableState,
    pub(crate) runtime: MessagingRuntimeState,
    pub(crate) shared_message_timer: SharedMessageTimerState,
    pub(crate) render_buffer: MessagingRenderBufferState,
    pub(crate) vwf_render: VwfRenderState,
}

impl MessagingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dialogue_message_index: DialogueMessageIndexState::load_from_ram(ram),
            multiselect_choice: MultiselectChoiceState::load_from_ram(ram),
            dialogue_number: DialogueNumberState::load_from_ram(ram),
            dialogue_source_offset: DialogueSourceOffsetState::load_from_ram(ram),
            decoded_text: DecodedMessageTextState::load_from_ram(ram),
            dialogue_pointers: DialoguePointerTableState::load_from_ram(ram),
            runtime: MessagingRuntimeState::load_from_ram(ram),
            shared_message_timer: SharedMessageTimerState::load_from_ram(ram),
            render_buffer: MessagingRenderBufferState::load_from_ram(ram),
            vwf_render: VwfRenderState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.dialogue_message_index.write_to_ram(ram);
        self.multiselect_choice.write_to_ram(ram);
        self.dialogue_number.write_to_ram(ram);
        self.dialogue_source_offset.write_to_ram(ram);
        self.decoded_text.write_to_ram(ram);
        self.dialogue_pointers.write_to_ram(ram);
        self.shared_message_timer.write_to_ram(ram);
        self.render_buffer.write_to_ram(ram);
        self.vwf_render.write_to_ram(ram);
    }
}

pub(crate) struct NativeMultiselectChoiceView<'a> {
    choice: &'a MultiselectChoiceState,
    runtime: &'a MessagingRuntimeState,
}

impl<'a> NativeMultiselectChoiceView<'a> {
    pub(crate) fn new(
        choice: &'a MultiselectChoiceState,
        runtime: &'a MessagingRuntimeState,
    ) -> Self {
        Self { choice, runtime }
    }

    pub(crate) fn value(&self) -> u8 {
        self.choice.value()
    }

    pub(crate) fn value_word(&self) -> u16 {
        u16::from(self.choice.value()) | (u16::from(self.runtime.text_wait_countdown2()) << 8)
    }

    pub(crate) fn backup(&self) -> u8 {
        self.choice.backup()
    }
}

pub(crate) struct NativeDialogueMessageIndexBridgeMut<'a> {
    message_index: &'a mut DialogueMessageIndexState,
    ram: &'a mut [u8],
}

impl<'a> NativeDialogueMessageIndexBridgeMut<'a> {
    pub(crate) fn new(message_index: &'a mut DialogueMessageIndexState, ram: &'a mut [u8]) -> Self {
        *message_index = DialogueMessageIndexState::load_from_ram(ram);
        Self { message_index, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.message_index,
            DialogueMessageIndexState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_value(&mut self, value: u16) {
        self.message_index.set_value(value);
        write_le_u16(self.ram, DIALOGUE_MESSAGE_INDEX, value);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeMultiselectChoiceBridgeMut<'a> {
    choice: &'a mut MultiselectChoiceState,
    ram: &'a mut [u8],
}

impl<'a> NativeMultiselectChoiceBridgeMut<'a> {
    pub(crate) fn new(choice: &'a mut MultiselectChoiceState, ram: &'a mut [u8]) -> Self {
        *choice = MultiselectChoiceState::load_from_ram(ram);
        Self { choice, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.choice,
            MultiselectChoiceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_value(&mut self, value: u8) {
        self.choice.set_value(value);
        self.ram[MULTISELECT_CHOICE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_value(&mut self) {
        self.choice.increment_value();
        self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE].wrapping_add(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_value(&mut self) {
        self.choice.decrement_value();
        self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE].wrapping_sub(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_backup(&mut self) {
        self.choice.restore_backup();
        self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE_BACKUP];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn save_backup(&mut self) {
        self.choice.save_backup();
        self.ram[MULTISELECT_CHOICE_BACKUP] = self.ram[MULTISELECT_CHOICE];
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeDialogueNumberBridgeMut<'a> {
    number: &'a mut DialogueNumberState,
    ram: &'a mut [u8],
}

impl<'a> NativeDialogueNumberBridgeMut<'a> {
    pub(crate) fn new(number: &'a mut DialogueNumberState, ram: &'a mut [u8]) -> Self {
        *number = DialogueNumberState::load_from_ram(ram);
        Self { number, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.number, DialogueNumberState::load_from_ram(self.ram));
    }

    pub(crate) fn set_packed_digits(&mut self, low_pair: u8, high_pair: u8) {
        self.number.set_packed_digits(low_pair, high_pair);
        self.ram[DIALOGUE_NUMBER_LO] = low_pair;
        self.ram[DIALOGUE_NUMBER_HI] = high_pair;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_low_pair(&mut self, value: u8) {
        self.number.set_low_pair(value);
        self.ram[DIALOGUE_NUMBER_LO] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_high_pair(&mut self, value: u8) {
        self.number.set_high_pair(value);
        self.ram[DIALOGUE_NUMBER_HI] = value;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeDialogueSourceOffsetBridgeMut<'a> {
    source_offset: &'a mut DialogueSourceOffsetState,
    ram: &'a mut [u8],
}

impl<'a> NativeDialogueSourceOffsetBridgeMut<'a> {
    pub(crate) fn new(source_offset: &'a mut DialogueSourceOffsetState, ram: &'a mut [u8]) -> Self {
        *source_offset = DialogueSourceOffsetState::load_from_ram(ram);
        Self { source_offset, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.source_offset,
            DialogueSourceOffsetState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn increment_bank_offset_low_nibble(&mut self) -> u8 {
        let next = self.source_offset.increment_bank_offset_low_nibble();
        self.ram[DIALOGUE_MSG_SRC_OFFS + 2] = next;
        self.debug_assert_matches_ram();
        next
    }
}

pub(crate) struct NativeDecodedMessageTextBridgeMut<'a> {
    messaging: &'a mut MessagingState,
    ram: &'a mut [u8],
}

impl<'a> NativeDecodedMessageTextBridgeMut<'a> {
    pub(crate) fn new(messaging: &'a mut MessagingState, ram: &'a mut [u8]) -> Self {
        messaging.decoded_text = DecodedMessageTextState::load_from_ram(ram);
        messaging.dialogue_pointers = DialoguePointerTableState::load_from_ram(ram);
        Self { messaging, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.messaging.decoded_text,
            DecodedMessageTextState::load_from_ram(self.ram)
        );
        debug_assert_eq!(
            self.messaging.dialogue_pointers,
            DialoguePointerTableState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn load_decoded_dialogue(&mut self, decoded: &[u8]) -> usize {
        let len = self.messaging.decoded_text.load_decoded_dialogue(decoded);
        self.ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + len]
            .copy_from_slice(&decoded[..len]);
        self.debug_assert_matches_ram();
        len
    }

    pub(crate) fn write_decoded_text_at(&mut self, dst: usize, decoded: &[u8]) -> usize {
        let len = self
            .messaging
            .decoded_text
            .write_decoded_text_at(dst, decoded);
        if len != 0 {
            self.ram[dst..dst + len].copy_from_slice(&decoded[..len]);
        }
        self.debug_assert_matches_ram();
        len
    }

    pub(crate) fn set_dialogue_pointer(&mut self, index: usize, pointer: u32) {
        self.messaging.dialogue_pointers.set_pointer(index, pointer);
        let dst = TEXT_DIALOGUE_POINTERS + index * 3;
        self.ram[dst] = pointer as u8;
        self.ram[dst + 1] = (pointer >> 8) as u8;
        self.ram[dst + 2] = (pointer >> 16) as u8;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeMessagingRuntimeBridgeMut<'a> {
    messaging: &'a mut MessagingState,
    ram: &'a mut [u8],
}

impl<'a> NativeMessagingRuntimeBridgeMut<'a> {
    pub(crate) fn new(messaging: &'a mut MessagingState, ram: &'a mut [u8]) -> Self {
        *messaging = MessagingState::load_from_ram(ram);
        Self { messaging, ram }
    }

    fn debug_assert_runtime_matches_ram(&self) {
        debug_assert_eq!(
            self.messaging.runtime,
            MessagingRuntimeState::load_from_ram(self.ram)
        );
        debug_assert_eq!(
            self.messaging.multiselect_choice,
            MultiselectChoiceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_module(&mut self, value: u8) {
        self.messaging.runtime.module = value;
        self.ram[MESSAGING_MODULE] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_module(&mut self) {
        self.set_module(0);
    }

    pub(crate) fn clear_message_or_sprite_state_cache(&mut self) {
        self.messaging.runtime.message_or_sprite_state_cache = 0;
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_render_state(&mut self, value: u8) {
        self.messaging.runtime.text_render_state = value;
        self.ram[TEXT_RENDER_STATE] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn increment_text_render_state(&mut self) -> u8 {
        let next = self.messaging.runtime.text_render_state.wrapping_add(1);
        self.set_text_render_state(next);
        next
    }

    pub(crate) fn set_text_wait_countdown2(&mut self, value: u8) {
        self.messaging.runtime.text_wait_countdown2 = value;
        self.ram[TEXT_WAIT_COUNTDOWN2] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_text_wait_countdown2(&mut self) {
        self.set_text_wait_countdown2(0);
    }

    pub(crate) fn decrement_text_wait_countdown2(&mut self) -> u8 {
        let next = self.messaging.runtime.text_wait_countdown2.wrapping_sub(1);
        self.set_text_wait_countdown2(next);
        next
    }

    pub(crate) fn set_menu_animation_timer(&mut self, value: u8) {
        self.messaging.runtime.menu_animation_timer = value;
        self.ram[MENU_ANIMATION_TIMER] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn decrement_menu_animation_timer(&mut self) -> u8 {
        let next = self.messaging.runtime.menu_animation_timer.wrapping_sub(1);
        self.set_menu_animation_timer(next);
        next
    }

    pub(crate) fn set_game_over_letter_cursor(&mut self, value: u8) {
        self.messaging.runtime.game_over_letter_cursor = value;
        self.ram[GAME_OVER_LETTER_CURSOR] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_effect_index(&mut self, value: u8) {
        self.set_game_over_letter_cursor(value);
    }

    pub(crate) fn or_effect_index(&mut self, value: u8) {
        let next = self.messaging.runtime.game_over_letter_cursor | value;
        self.set_game_over_letter_cursor(next);
    }

    pub(crate) fn clear_game_over_letter_cursor(&mut self) {
        self.set_game_over_letter_cursor(0);
    }

    pub(crate) fn clear_effect_index(&mut self) {
        self.clear_game_over_letter_cursor();
    }

    pub(crate) fn increment_game_over_letter_cursor(&mut self) -> u8 {
        let next = self
            .messaging
            .runtime
            .game_over_letter_cursor
            .wrapping_add(1);
        self.set_game_over_letter_cursor(next);
        next
    }

    pub(crate) fn decrement_game_over_letter_cursor(&mut self) -> u8 {
        let next = self
            .messaging
            .runtime
            .game_over_letter_cursor
            .wrapping_sub(1);
        self.set_game_over_letter_cursor(next);
        next
    }

    pub(crate) fn clear_flag_which_music_type_messaging(&mut self) {
        self.messaging.runtime.flag_which_music_type_messaging = 0;
        self.ram[FLAG_WHICH_MUSIC_TYPE_MESSAGING] = 0;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn xor_message_or_sprite_state_cache(&mut self, value: u8) {
        self.messaging.runtime.message_or_sprite_state_cache ^= value;
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] ^= value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn increment_text_incremental_state(&mut self) {
        self.messaging.runtime.text_incremental_state = self
            .messaging
            .runtime
            .text_incremental_state
            .wrapping_add(1);
        self.ram[TEXT_INCREMENTAL_STATE] = self.ram[TEXT_INCREMENTAL_STATE].wrapping_add(1);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn decrement_vwf_line_speed_cur(&mut self) {
        self.messaging.runtime.vwf_line_speed_cur =
            self.messaging.runtime.vwf_line_speed_cur.wrapping_sub(1);
        self.ram[VWF_LINE_SPEED_CUR] = self.ram[VWF_LINE_SPEED_CUR].wrapping_sub(1);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_vwf_line_speed(&mut self, value: u8) {
        self.messaging.runtime.vwf_line_speed = value;
        self.ram[VWF_LINE_SPEED] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_vwf_line_speed_cur(&mut self, value: u8) {
        self.messaging.runtime.vwf_line_speed_cur = value;
        self.ram[VWF_LINE_SPEED_CUR] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_dialogue_scroll_speed(&mut self, value: u8) {
        self.messaging.runtime.dialogue_scroll_speed = value;
        self.ram[DIALOGUE_SCROLL_SPEED] = value;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_wait_countdown(&mut self, value: u16) {
        self.messaging.runtime.text_wait_countdown = value;
        write_le_u16(self.ram, TEXT_WAIT_COUNTDOWN, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_text_wait_countdown(&mut self) {
        self.messaging.runtime.text_wait_countdown &= 0xff00;
        self.ram[TEXT_WAIT_COUNTDOWN] = 0;
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_msgbox_topleft(&mut self, value: u16) {
        self.messaging.runtime.text_msgbox_topleft = value;
        write_le_u16(self.ram, TEXT_MSGBOX_TOPLEFT, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_msgbox_topleft_copy(&mut self, value: u16) {
        self.messaging.runtime.text_msgbox_topleft_copy = value;
        write_le_u16(self.ram, TEXT_MSGBOX_TOPLEFT_COPY, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn set_text_tilemap_cur(&mut self, value: u16) {
        self.messaging.runtime.text_tilemap_cur = value;
        write_le_u16(self.ram, TEXT_TILEMAP_CUR, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn clear_dialogue_msg_read_pos(&mut self) {
        self.set_dialogue_msg_read_pos(0);
    }

    pub(crate) fn set_dialogue_msg_read_pos(&mut self, value: u16) {
        self.messaging.runtime.dialogue_msg_read_pos = value;
        write_le_u16(self.ram, DIALOGUE_MSG_READ_POS, value);
        self.debug_assert_runtime_matches_ram();
    }

    pub(crate) fn init_msgbox_state_from(&mut self, data: &[u8]) {
        self.ram[TEXT_MSGBOX_TOPLEFT_COPY..TEXT_MSGBOX_TOPLEFT_COPY + data.len()]
            .copy_from_slice(data);
        self.messaging.runtime = MessagingRuntimeState::load_from_ram(self.ram);
        self.messaging.multiselect_choice = MultiselectChoiceState::load_from_ram(self.ram);
        self.debug_assert_runtime_matches_ram();
    }
}

pub(crate) struct NativeMessagingRenderBufferBridgeMut<'a> {
    render_buffer: &'a mut MessagingRenderBufferState,
    ram: &'a mut [u8],
}

impl<'a> NativeMessagingRenderBufferBridgeMut<'a> {
    pub(crate) fn new(
        render_buffer: &'a mut MessagingRenderBufferState,
        ram: &'a mut [u8],
    ) -> Self {
        *render_buffer = MessagingRenderBufferState::load_from_ram(ram);
        Self { render_buffer, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.render_buffer,
            MessagingRenderBufferState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn xor_mask(&mut self, offset: usize, mask: u8) {
        self.render_buffer.xor_mask(offset, mask);
        self.ram[MESSAGING_RENDER_BUFFER + offset] ^= mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_mask(&mut self, offset: usize, mask: u8) {
        self.render_buffer.clear_mask(offset, mask);
        self.ram[MESSAGING_RENDER_BUFFER + offset] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_word(&mut self, index: usize, value: u16) {
        self.render_buffer.set_word(index, value);
        write_le_u16(self.ram, MESSAGING_RENDER_BUFFER + index * 2, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        self.render_buffer
            .set_word_at_byte_offset(byte_offset, value);
        write_le_u16(self.ram, MESSAGING_RENDER_BUFFER + byte_offset, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_range(&mut self, byte_count: usize) {
        self.render_buffer.clear_range(byte_count);
        self.ram[MESSAGING_RENDER_BUFFER..MESSAGING_RENDER_BUFFER + byte_count].fill(0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn fill_word_range(&mut self, start_index: usize, count: usize, value: u16) {
        self.render_buffer
            .fill_word_range(start_index, count, value);
        for i in 0..count {
            write_le_u16(
                self.ram,
                MESSAGING_RENDER_BUFFER + (start_index + i) * 2,
                value,
            );
        }
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeVwfRenderBridgeMut<'a> {
    vwf_render: &'a mut VwfRenderState,
    ram: &'a mut [u8],
}

impl<'a> NativeVwfRenderBridgeMut<'a> {
    pub(crate) fn new(vwf_render: &'a mut VwfRenderState, ram: &'a mut [u8]) -> Self {
        *vwf_render = VwfRenderState::load_from_ram(ram);
        Self { vwf_render, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.vwf_render, VwfRenderState::load_from_ram(self.ram));
    }

    pub(crate) fn set_next_glyph_advance_prefix_sum(&mut self, index: usize, value: u8) {
        self.vwf_render
            .set_next_glyph_advance_prefix_sum(index, value);
        self.ram[VWF_ARR + index + 1] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_glyph_cursor(&mut self, value: u16) {
        self.vwf_render.set_glyph_cursor(value);
        write_le_u16(self.ram, VWF_GLYPH_CURSOR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_glyph_cursor(&mut self) {
        self.vwf_render.clear_glyph_cursor();
        write_le_u16(self.ram, VWF_GLYPH_CURSOR, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_glyph_cursor(&mut self) -> u16 {
        let value = self.vwf_render.increment_glyph_cursor();
        write_le_u16(self.ram, VWF_GLYPH_CURSOR, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn request_next_line(&mut self, value: u16) {
        self.vwf_render.request_next_line(value);
        write_le_u16(self.ram, VWF_FLAG_NEXT_LINE, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_next_line_request(&mut self) {
        self.vwf_render.clear_next_line_request();
        write_le_u16(self.ram, VWF_FLAG_NEXT_LINE, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_current_line(&mut self, value: u16) {
        self.vwf_render.set_current_line(value);
        write_le_u16(self.ram, VWF_CURLINE, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_line_render_offset(&mut self, value: u16) {
        self.vwf_render.set_line_render_offset(value);
        write_le_u16(self.ram, VWF_LINE_PTR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        self.vwf_render
            .set_tile_word_at_byte_offset(byte_offset, value);
        write_le_u16(self.ram, VWF_TILE_BUFFER + byte_offset, value);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeSharedMessageTimerBridgeMut<'a> {
    timer: &'a mut SharedMessageTimerState,
    ram: &'a mut [u8],
}

impl<'a> NativeSharedMessageTimerBridgeMut<'a> {
    pub(crate) fn new(timer: &'a mut SharedMessageTimerState, ram: &'a mut [u8]) -> Self {
        *timer = SharedMessageTimerState::load_from_ram(ram);
        Self { timer, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.timer,
            SharedMessageTimerState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn start(&mut self, value: u16) {
        self.timer.timer = value;
        write_le_u16(self.ram, SHARED_MESSAGE_TIMER, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear(&mut self) {
        self.timer.clear();
        write_le_u16(self.ram, SHARED_MESSAGE_TIMER, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn tick(&mut self) -> u16 {
        let value = self.timer.tick();
        write_le_u16(self.ram, SHARED_MESSAGE_TIMER, value);
        self.debug_assert_matches_ram();
        value
    }
}
