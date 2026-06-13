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
