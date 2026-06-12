use super::*;

pub(crate) struct InventoryStateView<'a> {
    ram: &'a [u8],
}

impl<'a> InventoryStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn inventory_item(&self, index: usize) -> u8 {
        byte(self.ram, LINK_ITEM_BOW + index)
    }

    pub(crate) fn item_memory_value(&self, item_memory_addr: usize) -> u8 {
        byte(self.ram, item_memory_addr)
    }

    pub(crate) fn has_inventory_item(&self, index: usize) -> bool {
        self.inventory_item(index) != 0
    }

    pub(crate) fn bow(&self) -> u8 {
        self.inventory_item(0)
    }

    pub(crate) fn has_silver_arrows(&self) -> bool {
        self.bow() & 4 != 0
    }

    pub(crate) fn has_upgraded_bow(&self) -> bool {
        self.bow() >= 3
    }

    pub(crate) fn boomerang(&self) -> u8 {
        self.inventory_item(1)
    }

    pub(crate) fn hookshot(&self) -> u8 {
        self.inventory_item(2)
    }

    pub(crate) fn mushroom(&self) -> u8 {
        self.inventory_item(4)
    }

    pub(crate) fn fire_rod(&self) -> u8 {
        self.inventory_item(5)
    }

    pub(crate) fn ice_rod(&self) -> u8 {
        self.inventory_item(6)
    }

    pub(crate) fn bombos(&self) -> u8 {
        self.inventory_item(7)
    }

    pub(crate) fn ether(&self) -> u8 {
        self.inventory_item(8)
    }

    pub(crate) fn quake(&self) -> u8 {
        self.inventory_item(9)
    }

    pub(crate) fn torch(&self) -> u8 {
        self.inventory_item(10)
    }

    pub(crate) fn hammer(&self) -> u8 {
        self.inventory_item(11)
    }

    pub(crate) fn flute(&self) -> u8 {
        self.inventory_item(12)
    }

    pub(crate) fn bug_net(&self) -> u8 {
        self.inventory_item(13)
    }

    pub(crate) fn book(&self) -> u8 {
        self.inventory_item(14)
    }

    pub(crate) fn cane_somaria(&self) -> u8 {
        self.inventory_item(15)
    }

    pub(crate) fn cane_byrna(&self) -> u8 {
        self.inventory_item(17)
    }

    pub(crate) fn cape(&self) -> u8 {
        self.inventory_item(18)
    }

    pub(crate) fn mirror(&self) -> u8 {
        self.inventory_item(19)
    }

    pub(crate) fn gloves(&self) -> u8 {
        self.inventory_item(20)
    }

    pub(crate) fn boots(&self) -> u8 {
        self.inventory_item(21)
    }

    pub(crate) fn has_boots(&self) -> bool {
        self.boots() != 0
    }

    pub(crate) fn flippers(&self) -> u8 {
        self.inventory_item(22)
    }

    pub(crate) fn moon_pearl(&self) -> u8 {
        self.inventory_item(23)
    }

    pub(crate) fn has_moon_pearl(&self) -> bool {
        self.moon_pearl() != 0
    }

    pub(crate) fn sword_type(&self) -> u8 {
        self.inventory_item(25)
    }

    pub(crate) fn shield_type(&self) -> u8 {
        self.inventory_item(26)
    }

    pub(crate) fn armor(&self) -> u8 {
        self.inventory_item(27)
    }

    pub(crate) fn bottle(&self, index: usize) -> u8 {
        byte(self.ram, LINK_BOTTLE_INFO + index)
    }

    pub(crate) fn has_bottle(&self, index: usize) -> bool {
        self.bottle(index) != 0
    }

    pub(crate) fn bottle_contents_or(&self) -> u8 {
        self.bottle(0) | self.bottle(1) | self.bottle(2) | self.bottle(3)
    }

    pub(crate) fn has_bottle_at_least(&self, value: u8) -> bool {
        (0..4).any(|index| self.bottle(index) >= value)
    }

    pub(crate) fn equipped_button_item(&self, button_index: usize) -> u8 {
        match button_index {
            0 => byte(self.ram, HUD_CUR_ITEM),
            1 => byte(self.ram, HUD_CUR_ITEM_X),
            2 => byte(self.ram, HUD_CUR_ITEM_L),
            _ => byte(self.ram, HUD_CUR_ITEM_R),
        }
    }
}

pub(crate) struct InventoryStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> InventoryStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_inventory_item(&mut self, index: usize, value: u8) {
        self.ram[LINK_ITEM_BOW + index] = value;
    }

    pub(crate) fn set_item_memory_value(&mut self, item_memory_addr: usize, value: u8) {
        self.ram[item_memory_addr] = value;
    }

    pub(crate) fn or_item_memory_value(&mut self, item_memory_addr: usize, value: u8) -> u8 {
        self.ram[item_memory_addr] |= value;
        self.ram[item_memory_addr]
    }

    pub(crate) fn set_item_memory_value_if_empty(&mut self, item_memory_addr: usize, value: u8) {
        if self.ram[item_memory_addr] == 0 {
            self.ram[item_memory_addr] = value;
        }
    }

    pub(crate) fn or_item_memory_word(&mut self, item_memory_addr: usize, value: u16) {
        let next = read_le_u16(self.ram, item_memory_addr) | value;
        write_le_u16(self.ram, item_memory_addr, next);
    }

    pub(crate) fn add_item_memory_value_capped(
        &mut self,
        item_memory_addr: usize,
        add: u8,
        cap: u8,
    ) {
        self.ram[item_memory_addr] = self.ram[item_memory_addr].saturating_add(add).min(cap);
    }

    pub(crate) fn increment_item_memory_value_mod4(&mut self, item_memory_addr: usize) {
        self.ram[item_memory_addr] = self.ram[item_memory_addr].wrapping_add(1) & 3;
    }

    pub(crate) fn set_mushroom(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 4] = value;
    }

    pub(crate) fn set_ice_rod(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 6] = value;
    }

    pub(crate) fn set_bombos(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 7] = value;
    }

    pub(crate) fn set_ether(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 8] = value;
    }

    pub(crate) fn set_flute(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 12] = value;
    }

    pub(crate) fn set_mirror(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 19] = value;
    }

    pub(crate) fn set_boots(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 21] = value;
    }

    pub(crate) fn set_moon_pearl(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 23] = value;
    }

    pub(crate) fn set_sword_type(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 25] = value;
    }

    pub(crate) fn set_shield_type(&mut self, value: u8) {
        self.ram[LINK_ITEM_BOW + 26] = value;
    }

    pub(crate) fn set_bottle(&mut self, index: usize, value: u8) {
        self.ram[LINK_BOTTLE_INFO + index] = value;
    }

    pub(crate) fn set_equipped_button_item(&mut self, button_index: usize, value: u8) {
        match button_index {
            0 => self.ram[HUD_CUR_ITEM] = value,
            1 => self.ram[HUD_CUR_ITEM_X] = value,
            2 => self.ram[HUD_CUR_ITEM_L] = value,
            _ => self.ram[HUD_CUR_ITEM_R] = value,
        }
    }

    pub(crate) fn fill_first_empty_bottle_with(&mut self, value: u8) -> bool {
        for i in 0..4 {
            if self.ram[LINK_BOTTLE_INFO + i] < 2 {
                self.ram[LINK_BOTTLE_INFO + i] = value;
                return true;
            }
        }
        false
    }

    pub(crate) fn replace_first_empty_bottle_with(&mut self, value: u8) -> bool {
        for i in 0..4 {
            if self.ram[LINK_BOTTLE_INFO + i] == 2 {
                self.ram[LINK_BOTTLE_INFO + i] = value;
                return true;
            }
        }
        false
    }
}

pub(crate) struct SaveProgressView<'a> {
    ram: &'a [u8],
}

impl<'a> SaveProgressView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn palace_index_x2(&self) -> u8 {
        byte(self.ram, CUR_PALACE_INDEX_X2)
    }

    pub(crate) fn palace_index_x2_word(&self) -> u16 {
        word(self.ram, CUR_PALACE_INDEX_X2)
    }

    pub(crate) fn palace_index(&self) -> usize {
        usize::from(self.palace_index_x2() >> 1)
    }

    pub(crate) fn progress_indicator(&self) -> u8 {
        byte(self.ram, SRAM_PROGRESS_INDICATOR)
    }

    pub(crate) fn progress_indicator_word(&self) -> u16 {
        word(self.ram, SRAM_PROGRESS_INDICATOR)
    }

    pub(crate) fn progress_flags(&self) -> u8 {
        byte(self.ram, SRAM_PROGRESS_FLAGS)
    }

    pub(crate) fn progress_flags_has(&self, mask: u8) -> bool {
        self.progress_flags() & mask != 0
    }

    pub(crate) fn map_icons_indicator(&self) -> u8 {
        byte(self.ram, SAVEGAME_MAP_ICONS_INDICATOR)
    }

    pub(crate) fn dark_world_state(&self) -> u8 {
        byte(self.ram, SAVEGAME_IS_DARKWORLD)
    }

    pub(crate) fn is_dark_world(&self) -> bool {
        self.dark_world_state() != 0
    }

    pub(crate) fn dark_world_bit6(&self) -> u8 {
        (self.dark_world_state() >> 6) & 1
    }

    pub(crate) fn hud_current_item(&self) -> u8 {
        byte(self.ram, HUD_CUR_ITEM)
    }

    pub(crate) fn hud_current_item_slot(&self, slot: usize) -> u8 {
        let address = match slot {
            1 => HUD_CUR_ITEM_X,
            2 => HUD_CUR_ITEM_L,
            3 => HUD_CUR_ITEM_R,
            _ => HUD_CUR_ITEM,
        };
        byte(self.ram, address)
    }

    pub(crate) fn dungeon_info_word(&self, room: usize) -> u16 {
        word(self.ram, SAVE_DUNG_INFO + room * 2)
    }

    pub(crate) fn death_count_for_palace(&self, palace: usize) -> u16 {
        word(self.ram, DEATHS_PER_PALACE + palace * 2)
    }

    pub(crate) fn pending_death_save_counter(&self) -> u16 {
        word(self.ram, PENDING_DEATH_SAVE_COUNTER)
    }

    pub(crate) fn total_death_save_counter(&self) -> u16 {
        word(self.ram, TOTAL_DEATH_SAVE_COUNTER)
    }

    pub(crate) fn total_death_save_counter_is_uninitialized(&self) -> bool {
        self.total_death_save_counter() == 0xffff
    }

    pub(crate) fn dungeon_info_slice(&self) -> &[u8] {
        &self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500]
    }

    pub(crate) fn which_starting_point(&self) -> u8 {
        byte(self.ram, WHICH_STARTING_POINT)
    }

    pub(crate) fn progress_indicator_3(&self) -> u8 {
        byte(self.ram, SRAM_PROGRESS_INDICATOR_3)
    }
}

pub(crate) struct SaveProgressViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SaveProgressViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_palace_index_x2(&mut self, value: u8) {
        self.ram[CUR_PALACE_INDEX_X2] = value;
    }

    pub(crate) fn set_which_starting_point(&mut self, value: u8) {
        self.ram[WHICH_STARTING_POINT] = value;
    }

    pub(crate) fn xor_palace_index_x2(&mut self, value: u8) {
        self.ram[CUR_PALACE_INDEX_X2] ^= value;
    }

    pub(crate) fn set_progress_indicator(&mut self, value: u8) {
        self.ram[SRAM_PROGRESS_INDICATOR] = value;
    }

    pub(crate) fn or_progress_flags(&mut self, value: u8) {
        self.ram[SRAM_PROGRESS_FLAGS] |= value;
    }

    pub(crate) fn or_progress_indicator_3(&mut self, bits: u8) {
        self.ram[SRAM_PROGRESS_INDICATOR_3] |= bits;
    }

    pub(crate) fn clear_progress_indicator_3_bits(&mut self, bits: u8) {
        self.ram[SRAM_PROGRESS_INDICATOR_3] &= !bits;
    }

    pub(crate) fn xor_progress_flags(&mut self, value: u8) {
        self.ram[SRAM_PROGRESS_FLAGS] ^= value;
    }

    pub(crate) fn set_progress_flags(&mut self, value: u8) {
        self.ram[SRAM_PROGRESS_FLAGS] = value;
    }

    pub(crate) fn set_map_icons_indicator(&mut self, value: u8) {
        self.ram[SAVEGAME_MAP_ICONS_INDICATOR] = value;
    }

    pub(crate) fn set_dark_world_state(&mut self, value: u8) {
        self.ram[SAVEGAME_IS_DARKWORLD] = value;
    }

    pub(crate) fn xor_dark_world_state(&mut self, value: u8) {
        self.ram[SAVEGAME_IS_DARKWORLD] ^= value;
    }

    pub(crate) fn set_hud_current_item(&mut self, value: u8) {
        self.ram[HUD_CUR_ITEM] = value;
    }

    pub(crate) fn set_hud_current_item_slot(&mut self, slot: usize, value: u8) {
        let address = match slot {
            1 => HUD_CUR_ITEM_X,
            2 => HUD_CUR_ITEM_L,
            3 => HUD_CUR_ITEM_R,
            _ => HUD_CUR_ITEM,
        };
        self.ram[address] = value;
    }

    pub(crate) fn set_death_count_for_palace(&mut self, palace: usize, value: u16) {
        write_le_u16(self.ram, DEATHS_PER_PALACE + palace * 2, value);
    }

    pub(crate) fn increment_pending_death_save_counter(&mut self) -> u16 {
        let deaths = word(self.ram, PENDING_DEATH_SAVE_COUNTER).wrapping_add(1);
        write_le_u16(self.ram, PENDING_DEATH_SAVE_COUNTER, deaths);
        deaths
    }

    pub(crate) fn clear_pending_death_save_counter(&mut self) {
        write_le_u16(self.ram, PENDING_DEATH_SAVE_COUNTER, 0);
    }

    pub(crate) fn set_total_death_save_counter(&mut self, value: u16) {
        write_le_u16(self.ram, TOTAL_DEATH_SAVE_COUNTER, value);
    }

    pub(crate) fn clear_post_message_refresh_flag(&mut self) {
        self.ram[HUD_POST_MESSAGE_REFRESH_FLAG] = 0;
    }

    pub(crate) fn request_post_message_refresh(&mut self) {
        self.ram[HUD_POST_MESSAGE_REFRESH_FLAG] = 0x80;
    }

    pub(crate) fn clear_dungeon_info(&mut self) {
        self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500].fill(0);
    }

    pub(crate) fn copy_dungeon_info_from(&mut self, source: &[u8]) {
        self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500].copy_from_slice(source);
    }

    pub(crate) fn set_dungeon_info_word(&mut self, room: usize, value: u16) {
        write_le_u16(self.ram, SAVE_DUNG_INFO + room * 2, value);
    }

    pub(crate) fn or_dungeon_info_word(&mut self, room: usize, value: u16) -> u16 {
        let word = read_le_u16(self.ram, SAVE_DUNG_INFO + room * 2) | value;
        write_le_u16(self.ram, SAVE_DUNG_INFO + room * 2, word);
        word
    }

    pub(crate) fn set_dungeon_info_checksum(&mut self, value: u16) {
        write_le_u16(self.ram, SAVE_DUNG_INFO + 0x4fe, value);
    }

    pub(crate) fn compute_dungeon_info_checksum(&self) -> u16 {
        let mut checksum = 0x5a5au16;
        for i in (0..0x4fe).step_by(2) {
            checksum = checksum.wrapping_sub(read_le_u16(self.ram, SAVE_DUNG_INFO + i));
        }
        checksum
    }
}
