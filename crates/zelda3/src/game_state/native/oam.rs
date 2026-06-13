use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const OAM_SHADOW_BYTES: usize = EXTENDED_OAM - OAM_BUF;
const PACKED_EXTENDED_OAM_BYTES: usize = BYTEWISE_EXTENDED_OAM - EXTENDED_OAM;
const BYTEWISE_EXTENDED_OAM_BYTES: usize = 0x80;
const OAM_REGION_COUNT: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OamState {
    priority_value: u16,
    current_pointer: u16,
    current_extended_pointer: u16,
    sprite_sorting_setting: u8,
    priority_value_2: u16,
    sort_sprites_offset: u16,
    player_oam_computed_value: u8,
    turtle_rock_priority_flag: u8,
    region_base: [u16; OAM_REGION_COUNT],
    region_alloc: [u16; OAM_REGION_COUNT],
    shadow_entries: Vec<u8>,
    packed_extended: Vec<u8>,
    bytewise_extended: Vec<u8>,
}

impl Default for OamState {
    fn default() -> Self {
        Self {
            priority_value: 0,
            current_pointer: 0,
            current_extended_pointer: 0,
            sprite_sorting_setting: 0,
            priority_value_2: 0,
            sort_sprites_offset: 0,
            player_oam_computed_value: 0,
            turtle_rock_priority_flag: 0,
            region_base: [0; OAM_REGION_COUNT],
            region_alloc: [0; OAM_REGION_COUNT],
            shadow_entries: vec![0; OAM_SHADOW_BYTES],
            packed_extended: vec![0; PACKED_EXTENDED_OAM_BYTES],
            bytewise_extended: vec![0; BYTEWISE_EXTENDED_OAM_BYTES],
        }
    }
}

impl OamState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self {
            priority_value: read_le_u16(ram, OAM_PRIORITY_VALUE),
            current_pointer: read_le_u16(ram, OAM_CUR_PTR),
            current_extended_pointer: read_le_u16(ram, OAM_EXT_CUR_PTR),
            sprite_sorting_setting: ram_byte(ram, SORT_SPRITES_SETTING),
            priority_value_2: read_le_u16(ram, OAM_PRIORITY_VALUE_2),
            sort_sprites_offset: read_le_u16(ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER),
            player_oam_computed_value: ram_byte(ram, VALUE_COMPUTED_FOR_PLAYER_OAM),
            turtle_rock_priority_flag: ram_byte(ram, TURTLE_ROCK_OAM_PRIORITY_FLAG),
            region_base: [0; OAM_REGION_COUNT],
            region_alloc: [0; OAM_REGION_COUNT],
            shadow_entries: vec![0; OAM_SHADOW_BYTES],
            packed_extended: vec![0; PACKED_EXTENDED_OAM_BYTES],
            bytewise_extended: vec![0; BYTEWISE_EXTENDED_OAM_BYTES],
        };
        for region in 0..OAM_REGION_COUNT {
            state.region_base[region] = read_le_u16(ram, OAM_REGION_BASE + region * 2);
            state.region_alloc[region] = read_le_u16(ram, OAM_REGION_ALLOC + region * 2);
        }
        copy_from_ram(ram, OAM_BUF, &mut state.shadow_entries, OAM_SHADOW_BYTES);
        copy_from_ram(
            ram,
            EXTENDED_OAM,
            &mut state.packed_extended,
            PACKED_EXTENDED_OAM_BYTES,
        );
        copy_from_ram(
            ram,
            BYTEWISE_EXTENDED_OAM,
            &mut state.bytewise_extended,
            BYTEWISE_EXTENDED_OAM_BYTES,
        );
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OAM_PRIORITY_VALUE, self.priority_value);
        write_le_u16(ram, OAM_CUR_PTR, self.current_pointer);
        write_le_u16(ram, OAM_EXT_CUR_PTR, self.current_extended_pointer);
        ram[SORT_SPRITES_SETTING] = self.sprite_sorting_setting;
        write_le_u16(ram, OAM_PRIORITY_VALUE_2, self.priority_value_2);
        write_le_u16(
            ram,
            SORT_SPRITES_OFFSET_INTO_OAM_BUFFER,
            self.sort_sprites_offset,
        );
        ram[VALUE_COMPUTED_FOR_PLAYER_OAM] = self.player_oam_computed_value;
        ram[TURTLE_ROCK_OAM_PRIORITY_FLAG] = self.turtle_rock_priority_flag;
        for region in 0..OAM_REGION_COUNT {
            write_le_u16(ram, OAM_REGION_BASE + region * 2, self.region_base[region]);
            write_le_u16(
                ram,
                OAM_REGION_ALLOC + region * 2,
                self.region_alloc[region],
            );
        }
        ram[OAM_BUF..OAM_BUF + self.shadow_entries.len()].copy_from_slice(&self.shadow_entries);
        ram[EXTENDED_OAM..EXTENDED_OAM + self.packed_extended.len()]
            .copy_from_slice(&self.packed_extended);
        ram[BYTEWISE_EXTENDED_OAM..BYTEWISE_EXTENDED_OAM + self.bytewise_extended.len()]
            .copy_from_slice(&self.bytewise_extended);
    }

    pub(crate) fn priority_word(&self) -> u16 {
        self.priority_value
    }

    pub(crate) fn priority_high(&self) -> u8 {
        (self.priority_value >> 8) as u8
    }

    pub(crate) fn current_pointer(&self) -> u16 {
        self.current_pointer
    }

    pub(crate) fn current_pointer_usize(&self) -> usize {
        usize::from(self.current_pointer)
    }

    pub(crate) fn current_extended_pointer(&self) -> u16 {
        self.current_extended_pointer
    }

    pub(crate) fn current_extended_pointer_usize(&self) -> usize {
        usize::from(self.current_extended_pointer)
    }

    pub(crate) fn extended_byte(&self, index: usize) -> u8 {
        self.bytewise_extended.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn sprite_sorting_setting(&self) -> u8 {
        self.sprite_sorting_setting
    }

    pub(crate) fn sprite_sorting_offset_index(&self) -> usize {
        usize::from(self.sprite_sorting_setting)
    }

    pub(crate) fn has_sprite_sorting(&self) -> bool {
        self.sprite_sorting_setting != 0
    }

    pub(crate) fn packed_extended_oam_byte(&self, index: usize) -> u8 {
        (self.extended_byte(3 + index * 4) << 6)
            | (self.extended_byte(2 + index * 4) << 4)
            | (self.extended_byte(1 + index * 4) << 2)
            | self.extended_byte(index * 4)
    }

    pub(crate) fn priority_value_2(&self) -> u16 {
        self.priority_value_2
    }

    pub(crate) fn sort_sprites_offset(&self) -> u16 {
        self.sort_sprites_offset
    }

    pub(crate) fn player_oam_computed_value(&self) -> u8 {
        self.player_oam_computed_value
    }

    pub(crate) fn turtle_rock_priority_flag(&self) -> u8 {
        self.turtle_rock_priority_flag
    }

    pub(crate) fn set_priority_word(&mut self, value: u16) {
        self.priority_value = value;
    }

    pub(crate) fn subtract_priority_word(&mut self, value: u16) {
        self.priority_value = self.priority_value.wrapping_sub(value);
    }

    pub(crate) fn set_priority_high(&mut self, value: u8) {
        self.priority_value = (self.priority_value & 0x00ff) | (u16::from(value) << 8);
    }

    pub(crate) fn set_current_pointer(&mut self, value: u16) {
        self.current_pointer = value;
    }

    pub(crate) fn add_current_pointer(&mut self, value: u16) {
        self.current_pointer = self.current_pointer.wrapping_add(value);
    }

    pub(crate) fn subtract_current_pointer(&mut self, value: u16) {
        self.current_pointer = self.current_pointer.wrapping_sub(value);
    }

    pub(crate) fn set_current_extended_pointer(&mut self, value: u16) {
        self.current_extended_pointer = value;
    }

    pub(crate) fn add_current_extended_pointer(&mut self, value: u16) {
        self.current_extended_pointer = self.current_extended_pointer.wrapping_add(value);
    }

    pub(crate) fn subtract_current_extended_pointer(&mut self, value: u16) {
        self.current_extended_pointer = self.current_extended_pointer.wrapping_sub(value);
    }

    pub(crate) fn set_sprite_sorting_setting(&mut self, value: u8) {
        self.sprite_sorting_setting = value;
    }

    pub(crate) fn clear_sprite_sorting_setting(&mut self) {
        self.sprite_sorting_setting = 0;
    }

    pub(crate) fn set_priority_value_2(&mut self, value: u16) {
        self.priority_value_2 = value;
    }

    pub(crate) fn set_sort_sprites_offset(&mut self, value: u16) {
        self.sort_sprites_offset = value;
    }

    pub(crate) fn clear_sort_sprites_offset(&mut self) {
        self.sort_sprites_offset = 0;
    }

    pub(crate) fn set_player_oam_computed_value(&mut self, value: u8) {
        self.player_oam_computed_value = value;
    }

    pub(crate) fn entry_x(&self, addr: usize) -> u8 {
        self.shadow_byte(addr)
    }

    pub(crate) fn entry_y(&self, addr: usize) -> u8 {
        self.shadow_byte(addr + 1)
    }

    pub(crate) fn entry_char(&self, addr: usize) -> u8 {
        self.shadow_byte(addr + 2)
    }

    pub(crate) fn entry_flags(&self, addr: usize) -> u8 {
        self.shadow_byte(addr + 3)
    }

    pub(crate) fn extended_byte_at(&self, addr: usize) -> u8 {
        bytewise_extended_offset(addr)
            .and_then(|offset| self.bytewise_extended.get(offset).copied())
            .unwrap_or(0)
    }

    pub(crate) fn region_base_word(&self, region: usize) -> u16 {
        self.region_base.get(region).copied().unwrap_or(0)
    }

    pub(crate) fn region_alloc_counter(&self, region: usize) -> u16 {
        self.region_alloc.get(region).copied().unwrap_or(0)
    }

    fn shadow_byte(&self, addr: usize) -> u8 {
        shadow_oam_offset(addr)
            .and_then(|offset| self.shadow_entries.get(offset).copied())
            .unwrap_or(0)
    }

    fn set_shadow_byte(&mut self, addr: usize, value: u8) -> bool {
        let Some(offset) = shadow_oam_offset(addr) else {
            return false;
        };
        let Some(entry) = self.shadow_entries.get_mut(offset) else {
            return false;
        };
        *entry = value;
        true
    }

    fn set_bytewise_extended_byte(&mut self, addr: usize, value: u8) -> bool {
        let Some(offset) = bytewise_extended_offset(addr) else {
            return false;
        };
        let Some(entry) = self.bytewise_extended.get_mut(offset) else {
            return false;
        };
        *entry = value;
        true
    }

    fn set_extended_byte(&mut self, index: usize, value: u8) -> bool {
        let Some(entry) = self.bytewise_extended.get_mut(index) else {
            return false;
        };
        *entry = value;
        true
    }

    fn set_packed_extended_byte(&mut self, index: usize, value: u8) -> bool {
        let Some(entry) = self.packed_extended.get_mut(index) else {
            return false;
        };
        *entry = value;
        true
    }
}

pub(crate) struct NativeOamStateBridgeMut<'a> {
    state: &'a mut OamState,
    ram: &'a mut [u8],
}

impl<'a> NativeOamStateBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut OamState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, OamState::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_priority_word(&mut self, value: u16) {
        self.state.set_priority_word(value);
        self.sync();
    }

    pub(crate) fn subtract_priority_word(&mut self, value: u16) {
        self.state.subtract_priority_word(value);
        self.sync();
    }

    pub(crate) fn set_priority_high(&mut self, value: u8) {
        self.state.set_priority_high(value);
        self.sync();
    }

    pub(crate) fn set_current_pointer(&mut self, value: u16) {
        self.state.set_current_pointer(value);
        self.sync();
    }

    pub(crate) fn add_current_pointer(&mut self, value: u16) {
        self.state.add_current_pointer(value);
        self.sync();
    }

    pub(crate) fn subtract_current_pointer(&mut self, value: u16) {
        self.state.subtract_current_pointer(value);
        self.sync();
    }

    pub(crate) fn set_current_extended_pointer(&mut self, value: u16) {
        self.state.set_current_extended_pointer(value);
        self.sync();
    }

    pub(crate) fn set_sprite_sorting_setting(&mut self, value: u8) {
        self.state.set_sprite_sorting_setting(value);
        self.sync();
    }

    pub(crate) fn set_priority_value_2(&mut self, value: u16) {
        self.state.set_priority_value_2(value);
        self.sync();
    }

    pub(crate) fn set_sort_sprites_offset(&mut self, value: u16) {
        self.state.set_sort_sprites_offset(value);
        self.sync();
    }

    pub(crate) fn clear_sort_sprites_offset(&mut self) {
        self.state.clear_sort_sprites_offset();
        self.sync();
    }

    pub(crate) fn set_player_oam_computed_value(&mut self, value: u8) {
        self.state.set_player_oam_computed_value(value);
        self.sync();
    }

    pub(crate) fn clear_sprite_sorting_setting(&mut self) {
        self.state.clear_sprite_sorting_setting();
        self.sync();
    }

    pub(crate) fn add_current_extended_pointer(&mut self, value: u16) {
        self.state.add_current_extended_pointer(value);
        self.sync();
    }

    pub(crate) fn subtract_current_extended_pointer(&mut self, value: u16) {
        self.state.subtract_current_extended_pointer(value);
        self.sync();
    }

    pub(crate) fn set_extended_byte(&mut self, index: usize, value: u8) {
        if self.state.set_extended_byte(index, value) {
            self.sync();
        }
    }

    pub(crate) fn set_extended_byte_at(&mut self, addr: usize, value: u8) {
        if self.state.set_bytewise_extended_byte(addr, value) {
            self.sync();
        }
    }

    pub(crate) fn set_extended_word(&mut self, index: usize, value: u16) {
        self.set_extended_byte(index, value as u8);
        self.set_extended_byte(index + 1, (value >> 8) as u8);
    }

    pub(crate) fn set_packed_extended_oam_byte(&mut self, index: usize, value: u8) {
        if self.state.set_packed_extended_byte(index, value) {
            self.sync();
        }
    }

    pub(crate) fn hide_sprite_row(&mut self, oam_index: usize) {
        self.set_entry_y(OAM_BUF + oam_index * 4, 0xf0);
    }

    pub(crate) fn write_entry(&mut self, addr: usize, x: u8, y: u8, charnum: u8, flags: u8) {
        if shadow_oam_offset(addr + 3).is_none() {
            return;
        }
        self.state.set_shadow_byte(addr, x);
        self.state.set_shadow_byte(addr + 1, y);
        self.state.set_shadow_byte(addr + 2, charnum);
        self.state.set_shadow_byte(addr + 3, flags);
        self.sync();
    }

    pub(crate) fn write_entry_with_extended(
        &mut self,
        addr: usize,
        x: u16,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.write_entry(addr, x as u8, y, charnum, flags);
        let ext_index = (addr - OAM_BUF) / 4;
        self.set_extended_byte(ext_index, big | ((x >> 8) as u8 & 1));
    }

    pub(crate) fn write_clipped_entry_with_extended(
        &mut self,
        addr: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        let clipped_y = if y.wrapping_add(0x10) < 0x100 {
            y as u8
        } else {
            0xf0
        };
        self.write_entry_with_extended(addr, x, clipped_y, charnum, flags, big);
    }

    pub(crate) fn write_indexed_entry_with_extended(
        &mut self,
        index: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.write_entry(OAM_BUF + index * 4, x, y, charnum, flags);
        self.set_extended_byte(index, big);
    }

    pub(crate) fn write_indexed_clipped_entry_with_extended(
        &mut self,
        index: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        let addr = OAM_BUF + index * 4;
        self.write_clipped_entry_with_extended(addr, x, y, charnum, flags, big);
    }

    pub(crate) fn write_current_entry_with_extended(
        &mut self,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        extended: u8,
    ) {
        self.write_entry(self.state.current_pointer_usize(), x, y, charnum, flags);
        self.set_extended_byte_at(self.state.current_extended_pointer_usize(), extended);
    }

    pub(crate) fn set_entry_x(&mut self, addr: usize, x: u8) {
        if self.state.set_shadow_byte(addr, x) {
            self.sync();
        }
    }

    pub(crate) fn set_entry_y(&mut self, addr: usize, y: u8) {
        if self.state.set_shadow_byte(addr + 1, y) {
            self.sync();
        }
    }

    pub(crate) fn set_entry_xy(&mut self, addr: usize, x: u8, y: u8) {
        self.set_entry_x(addr, x);
        self.set_entry_y(addr, y);
    }

    pub(crate) fn set_entry_char_flags(&mut self, addr: usize, value: u16) {
        if shadow_oam_offset(addr + 3).is_none() {
            return;
        }
        self.state.set_shadow_byte(addr + 2, value as u8);
        self.state.set_shadow_byte(addr + 3, (value >> 8) as u8);
        self.sync();
    }

    pub(crate) fn hide_entry(&mut self, addr: usize) {
        self.set_entry_y(addr, 0xf0);
    }

    pub(crate) fn set_entry_char(&mut self, addr: usize, charnum: u8) {
        if self.state.set_shadow_byte(addr + 2, charnum) {
            self.sync();
        }
    }

    pub(crate) fn set_entry_flags(&mut self, addr: usize, flags: u8) {
        if self.state.set_shadow_byte(addr + 3, flags) {
            self.sync();
        }
    }

    pub(crate) fn or_entry_flags(&mut self, addr: usize, bits: u8) {
        let flags = self.state.entry_flags(addr) | bits;
        self.set_entry_flags(addr, flags);
    }

    pub(crate) fn merge_entry_flags(&mut self, addr: usize, keep_mask: u8, bits: u8) {
        let flags = (self.state.entry_flags(addr) & keep_mask) | bits;
        self.set_entry_flags(addr, flags);
    }

    pub(crate) fn set_extended_bytes_at(&mut self, addr: usize, values: &[u8]) {
        for (index, value) in values.iter().copied().enumerate() {
            self.set_extended_byte_at(addr + index, value);
        }
    }

    pub(crate) fn init_credits_region_base(&mut self) {
        self.set_region_base_word(0, 0x30);
        self.set_region_base_word(1, 0x1d0);
        self.set_region_base_word(2, 0);
    }

    pub(crate) fn set_region_base_word(&mut self, region: usize, value: u16) {
        let Some(entry) = self.state.region_base.get_mut(region) else {
            return;
        };
        *entry = value;
        self.sync();
    }

    pub(crate) fn set_region_alloc_counter(&mut self, region: usize, value: u16) {
        let Some(entry) = self.state.region_alloc.get_mut(region) else {
            return;
        };
        *entry = value;
        self.sync();
    }
}

fn copy_from_ram(ram: &[u8], start: usize, dst: &mut [u8], len: usize) {
    let available = ram.len().saturating_sub(start).min(len);
    dst[..available].copy_from_slice(&ram[start..start + available]);
}

fn shadow_oam_offset(addr: usize) -> Option<usize> {
    let offset = addr.checked_sub(OAM_BUF)?;
    (offset < OAM_SHADOW_BYTES).then_some(offset)
}

fn bytewise_extended_offset(addr: usize) -> Option<usize> {
    let offset = addr.checked_sub(BYTEWISE_EXTENDED_OAM)?;
    (offset < BYTEWISE_EXTENDED_OAM_BYTES).then_some(offset)
}
