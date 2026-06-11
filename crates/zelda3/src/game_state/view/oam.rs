use super::*;

pub(crate) struct OamStateView<'a> {
    ram: &'a [u8],
}

impl<'a> OamStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn priority_word(&self) -> u16 {
        word(self.ram, OAM_PRIORITY_VALUE)
    }

    pub(crate) fn priority_high(&self) -> u8 {
        byte(self.ram, OAM_PRIORITY_VALUE + 1)
    }

    pub(crate) fn current_pointer(&self) -> u16 {
        word(self.ram, OAM_CUR_PTR)
    }

    pub(crate) fn current_pointer_usize(&self) -> usize {
        usize::from(self.current_pointer())
    }

    pub(crate) fn current_extended_pointer(&self) -> u16 {
        word(self.ram, OAM_EXT_CUR_PTR)
    }

    pub(crate) fn current_extended_pointer_usize(&self) -> usize {
        usize::from(self.current_extended_pointer())
    }

    pub(crate) fn extended_byte(&self, index: usize) -> u8 {
        byte(self.ram, BYTEWISE_EXTENDED_OAM + index)
    }

    pub(crate) fn sprite_sorting_setting(&self) -> u8 {
        byte(self.ram, SORT_SPRITES_SETTING)
    }

    pub(crate) fn sprite_sorting_offset_index(&self) -> usize {
        usize::from(self.sprite_sorting_setting())
    }

    pub(crate) fn has_sprite_sorting(&self) -> bool {
        self.sprite_sorting_setting() != 0
    }

    pub(crate) fn packed_extended_oam_byte(&self, index: usize) -> u8 {
        (self.extended_byte(3 + index * 4) << 6)
            | (self.extended_byte(2 + index * 4) << 4)
            | (self.extended_byte(1 + index * 4) << 2)
            | self.extended_byte(index * 4)
    }

    pub(crate) fn priority_value_2(&self) -> u16 {
        word(self.ram, OAM_PRIORITY_VALUE_2)
    }

    pub(crate) fn sort_sprites_offset(&self) -> u16 {
        word(self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER)
    }

    pub(crate) fn player_oam_computed_value(&self) -> u8 {
        byte(self.ram, VALUE_COMPUTED_FOR_PLAYER_OAM)
    }

    pub(crate) fn turtle_rock_priority_flag(&self) -> u8 {
        byte(self.ram, TURTLE_ROCK_OAM_PRIORITY_FLAG)
    }

    pub(crate) fn entry_x(&self, addr: usize) -> u8 {
        byte(self.ram, addr)
    }

    pub(crate) fn entry_y(&self, addr: usize) -> u8 {
        byte(self.ram, addr + 1)
    }

    pub(crate) fn entry_char(&self, addr: usize) -> u8 {
        byte(self.ram, addr + 2)
    }

    pub(crate) fn entry_flags(&self, addr: usize) -> u8 {
        byte(self.ram, addr + 3)
    }

    /// Reads a byte from the bytewise extended OAM region at a byte
    /// address.
    pub(crate) fn extended_byte_at(&self, addr: usize) -> u8 {
        byte(self.ram, addr)
    }

    pub(crate) fn region_base_word(&self, region: usize) -> u16 {
        word(self.ram, OAM_REGION_BASE + region * 2)
    }

    pub(crate) fn region_alloc_counter(&self, region: usize) -> u16 {
        word(self.ram, OAM_REGION_ALLOC + region * 2)
    }
}

pub(crate) struct OamStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OamStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_priority_word(&mut self, value: u16) {
        write_le_u16(self.ram, OAM_PRIORITY_VALUE, value);
    }

    pub(crate) fn subtract_priority_word(&mut self, value: u16) {
        let priority = read_le_u16(self.ram, OAM_PRIORITY_VALUE);
        write_le_u16(self.ram, OAM_PRIORITY_VALUE, priority.wrapping_sub(value));
    }

    pub(crate) fn set_priority_high(&mut self, value: u8) {
        self.ram[OAM_PRIORITY_VALUE + 1] = value;
    }

    pub(crate) fn set_current_pointer(&mut self, value: u16) {
        write_le_u16(self.ram, OAM_CUR_PTR, value);
    }

    pub(crate) fn add_current_pointer(&mut self, value: u16) {
        let pointer = read_le_u16(self.ram, OAM_CUR_PTR);
        write_le_u16(self.ram, OAM_CUR_PTR, pointer.wrapping_add(value));
    }

    pub(crate) fn subtract_current_pointer(&mut self, value: u16) {
        let pointer = read_le_u16(self.ram, OAM_CUR_PTR);
        write_le_u16(self.ram, OAM_CUR_PTR, pointer.wrapping_sub(value));
    }

    pub(crate) fn set_current_extended_pointer(&mut self, value: u16) {
        write_le_u16(self.ram, OAM_EXT_CUR_PTR, value);
    }

    pub(crate) fn set_sprite_sorting_setting(&mut self, value: u8) {
        self.ram[SORT_SPRITES_SETTING] = value;
    }

    pub(crate) fn set_priority_value_2(&mut self, value: u16) {
        write_le_u16(self.ram, OAM_PRIORITY_VALUE_2, value);
    }

    pub(crate) fn set_sort_sprites_offset(&mut self, value: u16) {
        write_le_u16(self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER, value);
    }

    pub(crate) fn clear_sort_sprites_offset(&mut self) {
        write_le_u16(self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER, 0);
    }

    pub(crate) fn set_player_oam_computed_value(&mut self, value: u8) {
        self.ram[VALUE_COMPUTED_FOR_PLAYER_OAM] = value;
    }

    pub(crate) fn clear_sprite_sorting_setting(&mut self) {
        self.set_sprite_sorting_setting(0);
    }

    pub(crate) fn add_current_extended_pointer(&mut self, value: u16) {
        let pointer = read_le_u16(self.ram, OAM_EXT_CUR_PTR);
        write_le_u16(self.ram, OAM_EXT_CUR_PTR, pointer.wrapping_add(value));
    }

    pub(crate) fn subtract_current_extended_pointer(&mut self, value: u16) {
        let pointer = read_le_u16(self.ram, OAM_EXT_CUR_PTR);
        write_le_u16(self.ram, OAM_EXT_CUR_PTR, pointer.wrapping_sub(value));
    }

    pub(crate) fn set_extended_byte(&mut self, index: usize, value: u8) {
        self.ram[BYTEWISE_EXTENDED_OAM + index] = value;
    }

    /// Writes a byte into the bytewise extended OAM region at a byte
    /// address.
    pub(crate) fn set_extended_byte_at(&mut self, addr: usize, value: u8) {
        self.ram[addr] = value;
    }

    pub(crate) fn set_extended_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, BYTEWISE_EXTENDED_OAM + index, value);
    }

    pub(crate) fn set_packed_extended_oam_byte(&mut self, index: usize, value: u8) {
        self.ram[EXTENDED_OAM + index] = value;
    }

    pub(crate) fn hide_sprite_row(&mut self, oam_index: usize) {
        self.ram[OAM_BUF + oam_index * 4 + 1] = 0xf0;
    }

    /// Writes a full 4-byte OAM entry (x, y, char, flags) at a byte
    /// address into the OAM buffer.
    pub(crate) fn write_entry(&mut self, addr: usize, x: u8, y: u8, charnum: u8, flags: u8) {
        self.ram[addr] = x;
        self.ram[addr + 1] = y;
        self.ram[addr + 2] = charnum;
        self.ram[addr + 3] = flags;
    }

    pub(crate) fn set_entry_x(&mut self, addr: usize, x: u8) {
        self.ram[addr] = x;
    }

    pub(crate) fn set_entry_y(&mut self, addr: usize, y: u8) {
        self.ram[addr + 1] = y;
    }

    pub(crate) fn set_entry_xy(&mut self, addr: usize, x: u8, y: u8) {
        self.ram[addr] = x;
        self.ram[addr + 1] = y;
    }

    /// Writes the char (low byte) and flags (high byte) of an entry as a
    /// single little-endian word.
    pub(crate) fn set_entry_char_flags(&mut self, addr: usize, value: u16) {
        write_le_u16(self.ram, addr + 2, value);
    }

    /// Moves the entry off-screen by setting its y coordinate below the
    /// visible area.
    pub(crate) fn hide_entry(&mut self, addr: usize) {
        self.ram[addr + 1] = 0xf0;
    }

    pub(crate) fn set_entry_char(&mut self, addr: usize, charnum: u8) {
        self.ram[addr + 2] = charnum;
    }

    pub(crate) fn set_entry_flags(&mut self, addr: usize, flags: u8) {
        self.ram[addr + 3] = flags;
    }

    pub(crate) fn or_entry_flags(&mut self, addr: usize, bits: u8) {
        self.ram[addr + 3] |= bits;
    }

    /// Keeps the flag bits selected by `keep_mask` and ors in `bits`.
    pub(crate) fn merge_entry_flags(&mut self, addr: usize, keep_mask: u8, bits: u8) {
        self.ram[addr + 3] = (self.ram[addr + 3] & keep_mask) | bits;
    }

    /// Writes consecutive bytes into the bytewise extended OAM region at
    /// a byte address.
    pub(crate) fn set_extended_bytes_at(&mut self, addr: usize, values: &[u8]) {
        self.ram[addr..addr + values.len()].copy_from_slice(values);
    }

    pub(crate) fn init_credits_region_base(&mut self) {
        write_le_u16(self.ram, OAM_REGION_BASE, 0x30);
        write_le_u16(self.ram, OAM_REGION_BASE + 2, 0x1d0);
        write_le_u16(self.ram, OAM_REGION_BASE + 4, 0);
    }

    pub(crate) fn set_region_base_word(&mut self, region: usize, value: u16) {
        write_le_u16(self.ram, OAM_REGION_BASE + region * 2, value);
    }

    pub(crate) fn set_region_alloc_counter(&mut self, region: usize, value: u16) {
        write_le_u16(self.ram, OAM_REGION_ALLOC + region * 2, value);
    }
}
