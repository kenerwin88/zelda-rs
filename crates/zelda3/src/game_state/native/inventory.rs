use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const DUNGEON_KEY_SLOT_COUNT: usize = 16;
const DEATH_COUNT_PALACE_SLOTS: usize = 14;
const SAVE_DUNGEON_INFO_LEN: usize = 0x500;
const INVENTORY_ITEM_SLOT_COUNT: usize = 28;
const BOTTLE_SLOT_COUNT: usize = 4;
const EQUIPPED_BUTTON_SLOT_COUNT: usize = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct InventoryState {
    pub(crate) dungeon_key_slots: DungeonKeySlotsState,
    pub(crate) player_resources: PlayerResourcesState,
    pub(crate) mirror_warp: MirrorWarpState,
    pub(crate) save_progress: SaveProgressState,
    pub(crate) items: InventoryItemsState,
}

impl InventoryState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dungeon_key_slots: DungeonKeySlotsState::load_from_ram(ram),
            player_resources: PlayerResourcesState::load_from_ram(ram),
            mirror_warp: MirrorWarpState::load_from_ram(ram),
            save_progress: SaveProgressState::load_from_ram(ram),
            items: InventoryItemsState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.dungeon_key_slots.write_to_ram(ram);
        self.player_resources.write_to_ram(ram);
        self.mirror_warp.write_to_ram(ram);
        self.save_progress.write_to_ram(ram);
        self.items.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct InventoryItemsState {
    item_slots: [u8; INVENTORY_ITEM_SLOT_COUNT],
    bottles: [u8; BOTTLE_SLOT_COUNT],
    equipped_button_items: [u8; EQUIPPED_BUTTON_SLOT_COUNT],
}

impl InventoryItemsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut item_slots = [0; INVENTORY_ITEM_SLOT_COUNT];
        for (index, item) in item_slots.iter_mut().enumerate() {
            *item = ram_byte(ram, LINK_ITEM_BOW + index);
        }

        let mut bottles = [0; BOTTLE_SLOT_COUNT];
        for (index, bottle) in bottles.iter_mut().enumerate() {
            *bottle = ram_byte(ram, LINK_BOTTLE_INFO + index);
        }

        Self {
            item_slots,
            bottles,
            equipped_button_items: [
                ram_byte(ram, HUD_CUR_ITEM),
                ram_byte(ram, HUD_CUR_ITEM_X),
                ram_byte(ram, HUD_CUR_ITEM_L),
                ram_byte(ram, HUD_CUR_ITEM_R),
            ],
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[LINK_ITEM_BOW..LINK_ITEM_BOW + INVENTORY_ITEM_SLOT_COUNT]
            .copy_from_slice(&self.item_slots);
        ram[LINK_BOTTLE_INFO..LINK_BOTTLE_INFO + BOTTLE_SLOT_COUNT].copy_from_slice(&self.bottles);
        ram[HUD_CUR_ITEM] = self.equipped_button_items[0];
        ram[HUD_CUR_ITEM_X] = self.equipped_button_items[1];
        ram[HUD_CUR_ITEM_L] = self.equipped_button_items[2];
        ram[HUD_CUR_ITEM_R] = self.equipped_button_items[3];
    }

    pub(crate) fn inventory_item(&self, index: usize) -> u8 {
        self.item_slots.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn item_memory_value(&self, ram: &[u8], item_memory_addr: usize) -> u8 {
        if (LINK_ITEM_BOW..LINK_ITEM_BOW + INVENTORY_ITEM_SLOT_COUNT).contains(&item_memory_addr) {
            self.inventory_item(item_memory_addr - LINK_ITEM_BOW)
        } else if (LINK_BOTTLE_INFO..LINK_BOTTLE_INFO + BOTTLE_SLOT_COUNT)
            .contains(&item_memory_addr)
        {
            self.bottle(item_memory_addr - LINK_BOTTLE_INFO)
        } else {
            match item_memory_addr {
                HUD_CUR_ITEM => self.equipped_button_item(0),
                HUD_CUR_ITEM_X => self.equipped_button_item(1),
                HUD_CUR_ITEM_L => self.equipped_button_item(2),
                HUD_CUR_ITEM_R => self.equipped_button_item(3),
                _ => ram.get(item_memory_addr).copied().unwrap_or(0),
            }
        }
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
        self.bottles.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn has_bottle(&self, index: usize) -> bool {
        self.bottle(index) != 0
    }

    pub(crate) fn bottle_contents_or(&self) -> u8 {
        self.bottles
            .iter()
            .copied()
            .fold(0, |acc, bottle| acc | bottle)
    }

    pub(crate) fn has_bottle_at_least(&self, value: u8) -> bool {
        self.bottles.iter().any(|bottle| *bottle >= value)
    }

    pub(crate) fn equipped_button_item(&self, button_index: usize) -> u8 {
        self.equipped_button_items
            .get(button_index)
            .copied()
            .unwrap_or_else(|| self.equipped_button_items[EQUIPPED_BUTTON_SLOT_COUNT - 1])
    }

    fn set_inventory_item(&mut self, index: usize, value: u8) {
        if let Some(item) = self.item_slots.get_mut(index) {
            *item = value;
        }
    }

    fn set_bottle(&mut self, index: usize, value: u8) {
        if let Some(bottle) = self.bottles.get_mut(index) {
            *bottle = value;
        }
    }

    fn set_equipped_button_item(&mut self, button_index: usize, value: u8) {
        let index = button_index.min(EQUIPPED_BUTTON_SLOT_COUNT - 1);
        self.equipped_button_items[index] = value;
    }

    fn fill_first_empty_bottle_with(&mut self, value: u8) -> bool {
        if let Some(bottle) = self.bottles.iter_mut().find(|bottle| **bottle < 2) {
            *bottle = value;
            true
        } else {
            false
        }
    }

    fn replace_first_empty_bottle_with(&mut self, value: u8) -> bool {
        if let Some(bottle) = self.bottles.iter_mut().find(|bottle| **bottle == 2) {
            *bottle = value;
            true
        } else {
            false
        }
    }
}

pub(crate) struct NativeInventoryItemsBridgeMut<'a> {
    items: &'a mut InventoryItemsState,
    ram: &'a mut [u8],
}

impl<'a> NativeInventoryItemsBridgeMut<'a> {
    pub(crate) fn new(items: &'a mut InventoryItemsState, ram: &'a mut [u8]) -> Self {
        Self { items, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.items, InventoryItemsState::load_from_ram(self.ram));
    }

    fn absorb_item_memory_byte(&mut self, address: usize) {
        if (LINK_ITEM_BOW..LINK_ITEM_BOW + INVENTORY_ITEM_SLOT_COUNT).contains(&address) {
            self.items
                .set_inventory_item(address - LINK_ITEM_BOW, self.ram[address]);
        } else if (LINK_BOTTLE_INFO..LINK_BOTTLE_INFO + BOTTLE_SLOT_COUNT).contains(&address) {
            self.items
                .set_bottle(address - LINK_BOTTLE_INFO, self.ram[address]);
        } else {
            match address {
                HUD_CUR_ITEM => self.items.set_equipped_button_item(0, self.ram[address]),
                HUD_CUR_ITEM_X => self.items.set_equipped_button_item(1, self.ram[address]),
                HUD_CUR_ITEM_L => self.items.set_equipped_button_item(2, self.ram[address]),
                HUD_CUR_ITEM_R => self.items.set_equipped_button_item(3, self.ram[address]),
                _ => {}
            }
        }
    }

    fn absorb_item_memory_word(&mut self, address: usize) {
        self.absorb_item_memory_byte(address);
        self.absorb_item_memory_byte(address + 1);
    }

    pub(crate) fn set_inventory_item(&mut self, index: usize, value: u8) {
        self.items.set_inventory_item(index, value);
        self.items.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_memory_value(&mut self, item_memory_addr: usize, value: u8) {
        self.ram[item_memory_addr] = value;
        self.absorb_item_memory_byte(item_memory_addr);
    }

    pub(crate) fn or_item_memory_value(&mut self, item_memory_addr: usize, value: u8) -> u8 {
        self.ram[item_memory_addr] |= value;
        self.absorb_item_memory_byte(item_memory_addr);
        self.ram[item_memory_addr]
    }

    pub(crate) fn set_item_memory_value_if_empty(&mut self, item_memory_addr: usize, value: u8) {
        if self.ram[item_memory_addr] == 0 {
            self.ram[item_memory_addr] = value;
            self.absorb_item_memory_byte(item_memory_addr);
        }
    }

    pub(crate) fn or_item_memory_word(&mut self, item_memory_addr: usize, value: u16) {
        let next = read_le_u16(self.ram, item_memory_addr) | value;
        write_le_u16(self.ram, item_memory_addr, next);
        self.absorb_item_memory_word(item_memory_addr);
    }

    pub(crate) fn add_item_memory_value_capped(
        &mut self,
        item_memory_addr: usize,
        add: u8,
        cap: u8,
    ) {
        self.ram[item_memory_addr] = self.ram[item_memory_addr].saturating_add(add).min(cap);
        self.absorb_item_memory_byte(item_memory_addr);
    }

    pub(crate) fn increment_item_memory_value_mod4(&mut self, item_memory_addr: usize) {
        self.ram[item_memory_addr] = self.ram[item_memory_addr].wrapping_add(1) & 3;
        self.absorb_item_memory_byte(item_memory_addr);
    }

    pub(crate) fn set_mushroom(&mut self, value: u8) {
        self.set_inventory_item(4, value);
    }

    pub(crate) fn set_ice_rod(&mut self, value: u8) {
        self.set_inventory_item(6, value);
    }

    pub(crate) fn set_bombos(&mut self, value: u8) {
        self.set_inventory_item(7, value);
    }

    pub(crate) fn set_ether(&mut self, value: u8) {
        self.set_inventory_item(8, value);
    }

    pub(crate) fn set_flute(&mut self, value: u8) {
        self.set_inventory_item(12, value);
    }

    pub(crate) fn set_mirror(&mut self, value: u8) {
        self.set_inventory_item(19, value);
    }

    pub(crate) fn set_boots(&mut self, value: u8) {
        self.set_inventory_item(21, value);
    }

    pub(crate) fn set_moon_pearl(&mut self, value: u8) {
        self.set_inventory_item(23, value);
    }

    pub(crate) fn set_sword_type(&mut self, value: u8) {
        self.set_inventory_item(25, value);
    }

    pub(crate) fn set_shield_type(&mut self, value: u8) {
        self.set_inventory_item(26, value);
    }

    pub(crate) fn set_bottle(&mut self, index: usize, value: u8) {
        self.items.set_bottle(index, value);
        self.items.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_equipped_button_item(&mut self, button_index: usize, value: u8) {
        self.items.set_equipped_button_item(button_index, value);
        self.items.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn fill_first_empty_bottle_with(&mut self, value: u8) -> bool {
        let filled = self.items.fill_first_empty_bottle_with(value);
        if filled {
            self.items.write_to_ram(self.ram);
        }
        self.debug_assert_matches_ram();
        filled
    }

    pub(crate) fn replace_first_empty_bottle_with(&mut self, value: u8) -> bool {
        let replaced = self.items.replace_first_empty_bottle_with(value);
        if replaced {
            self.items.write_to_ram(self.ram);
        }
        self.debug_assert_matches_ram();
        replaced
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MirrorWarpState {
    target_index: u16,
    target_offsets: [u16; 2],
    velocity_deltas: [u16; 2],
    wave_offset: u16,
    displacement: u16,
    subpixel: u16,
    reserved: u16,
    wave_length: u16,
    spacing_a: u16,
    spacing_b: u16,
    load_step_counter: u8,
    animation_counter: u8,
}

impl MirrorWarpState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            target_index: read_le_u16(ram, MIRROR_WARP_TARGET_INDEX),
            target_offsets: [
                read_le_u16(ram, MIRROR_WARP_TARGET_OFFSETS),
                read_le_u16(ram, MIRROR_WARP_TARGET_OFFSETS + 2),
            ],
            velocity_deltas: [
                read_le_u16(ram, MIRROR_WARP_VELOCITY_DELTAS),
                read_le_u16(ram, MIRROR_WARP_VELOCITY_DELTAS + 2),
            ],
            wave_offset: read_le_u16(ram, MIRROR_WARP_WAVE_OFFSET),
            displacement: read_le_u16(ram, MIRROR_WARP_DISPLACEMENT),
            subpixel: read_le_u16(ram, MIRROR_WARP_SUBPIXEL),
            reserved: read_le_u16(ram, MIRROR_WARP_RESERVED),
            wave_length: read_le_u16(ram, MIRROR_WARP_WAVE_LENGTH),
            spacing_a: read_le_u16(ram, MIRROR_WARP_SPACING_A),
            spacing_b: read_le_u16(ram, MIRROR_WARP_SPACING_B),
            load_step_counter: ram_byte(ram, MIRROR_WARP_LOAD_STEP_COUNTER),
            animation_counter: ram_byte(ram, MIRROR_WARP_ANIMATION_COUNTER),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, MIRROR_WARP_TARGET_INDEX, self.target_index);
        write_le_u16(ram, MIRROR_WARP_TARGET_OFFSETS, self.target_offsets[0]);
        write_le_u16(ram, MIRROR_WARP_TARGET_OFFSETS + 2, self.target_offsets[1]);
        write_le_u16(ram, MIRROR_WARP_VELOCITY_DELTAS, self.velocity_deltas[0]);
        write_le_u16(
            ram,
            MIRROR_WARP_VELOCITY_DELTAS + 2,
            self.velocity_deltas[1],
        );
        write_le_u16(ram, MIRROR_WARP_WAVE_OFFSET, self.wave_offset);
        write_le_u16(ram, MIRROR_WARP_DISPLACEMENT, self.displacement);
        write_le_u16(ram, MIRROR_WARP_SUBPIXEL, self.subpixel);
        write_le_u16(ram, MIRROR_WARP_RESERVED, self.reserved);
        write_le_u16(ram, MIRROR_WARP_WAVE_LENGTH, self.wave_length);
        write_le_u16(ram, MIRROR_WARP_SPACING_A, self.spacing_a);
        write_le_u16(ram, MIRROR_WARP_SPACING_B, self.spacing_b);
        ram[MIRROR_WARP_LOAD_STEP_COUNTER] = self.load_step_counter;
        ram[MIRROR_WARP_ANIMATION_COUNTER] = self.animation_counter;
    }

    pub(crate) fn target_index(&self) -> usize {
        usize::from(self.target_index >> 1)
    }

    pub(crate) fn target_offset(&self) -> u16 {
        self.target_offsets
            .get(self.target_index())
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn velocity_delta(&self) -> u16 {
        self.velocity_deltas
            .get(self.target_index())
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn wave_offset(&self) -> u16 {
        self.wave_offset
    }

    pub(crate) fn displacement(&self) -> u16 {
        self.displacement
    }

    pub(crate) fn subpixel(&self) -> u16 {
        self.subpixel
    }

    pub(crate) fn animation_counter(&self) -> u8 {
        self.animation_counter
    }

    pub(crate) fn initialize_hdma_wave_state(&mut self) {
        self.target_index = 0;
        self.wave_offset = 0;
        self.displacement = 0;
        self.subpixel = 0;
        self.reserved = 0;
        self.spacing_a = 8;
        self.spacing_b = 8;
        self.wave_length = 21;
        self.target_offsets = [0xfe00, 0x0200];
        self.velocity_deltas = [0xffc0, 0x0040];
    }

    pub(crate) fn reset_wave_and_subpixel(&mut self) {
        self.wave_offset = 0;
        self.subpixel = 0;
    }

    pub(crate) fn toggle_target_index(&mut self) {
        self.target_index ^= 2;
    }

    pub(crate) fn set_displacement(&mut self, value: u16) {
        self.displacement = value;
    }

    pub(crate) fn set_subpixel_low_from(&mut self, value: u16) {
        self.subpixel = value & 0x00ff;
    }

    pub(crate) fn set_wave_offset(&mut self, value: u16) {
        self.wave_offset = value;
    }

    pub(crate) fn shrink_target_offsets_for_dewaving(&mut self) {
        self.target_offsets = [0xff00, 0x0100];
    }

    pub(crate) fn increment_load_step_counter(&mut self) -> u8 {
        self.load_step_counter = self.load_step_counter.wrapping_add(1);
        self.load_step_counter
    }

    pub(crate) fn reset_load_step_counter(&mut self) {
        self.load_step_counter = 0;
    }

    pub(crate) fn set_animation_counter(&mut self, value: u8) {
        self.animation_counter = value;
    }

    pub(crate) fn decrement_animation_counter(&mut self) -> u8 {
        self.animation_counter = self.animation_counter.wrapping_sub(1);
        self.animation_counter
    }
}

pub(crate) struct NativeMirrorWarpBridgeMut<'a> {
    mirror_warp: &'a mut MirrorWarpState,
    ram: &'a mut [u8],
}

impl<'a> NativeMirrorWarpBridgeMut<'a> {
    pub(crate) fn new(mirror_warp: &'a mut MirrorWarpState, ram: &'a mut [u8]) -> Self {
        Self { mirror_warp, ram }
    }

    fn sync(&mut self) {
        self.mirror_warp.write_to_ram(self.ram);
        debug_assert_eq!(*self.mirror_warp, MirrorWarpState::load_from_ram(self.ram));
    }

    pub(crate) fn initialize_hdma_wave_state(&mut self) {
        self.mirror_warp.initialize_hdma_wave_state();
        self.sync();
    }

    pub(crate) fn reset_wave_and_subpixel(&mut self) {
        self.mirror_warp.reset_wave_and_subpixel();
        self.sync();
    }

    pub(crate) fn toggle_target_index(&mut self) {
        self.mirror_warp.toggle_target_index();
        self.sync();
    }

    pub(crate) fn set_displacement(&mut self, value: u16) {
        self.mirror_warp.set_displacement(value);
        self.sync();
    }

    pub(crate) fn set_subpixel_low_from(&mut self, value: u16) {
        self.mirror_warp.set_subpixel_low_from(value);
        self.sync();
    }

    pub(crate) fn set_wave_offset(&mut self, value: u16) {
        self.mirror_warp.set_wave_offset(value);
        self.sync();
    }

    pub(crate) fn shrink_target_offsets_for_dewaving(&mut self) {
        self.mirror_warp.shrink_target_offsets_for_dewaving();
        self.sync();
    }

    pub(crate) fn increment_load_step_counter(&mut self) -> u8 {
        let value = self.mirror_warp.increment_load_step_counter();
        self.sync();
        value
    }

    pub(crate) fn reset_load_step_counter(&mut self) {
        self.mirror_warp.reset_load_step_counter();
        self.sync();
    }

    pub(crate) fn set_animation_counter(&mut self, value: u8) {
        self.mirror_warp.set_animation_counter(value);
        self.sync();
    }

    pub(crate) fn decrement_animation_counter(&mut self) -> u8 {
        let value = self.mirror_warp.decrement_animation_counter();
        self.sync();
        value
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SaveProgressState {
    palace_index_x2: u8,
    hud_current_items: [u8; 4],
    dungeon_info: Vec<u8>,
    post_message_refresh_flag: u8,
}

impl Default for SaveProgressState {
    fn default() -> Self {
        Self {
            palace_index_x2: 0,
            hud_current_items: [0; 4],
            dungeon_info: vec![0; SAVE_DUNGEON_INFO_LEN],
            post_message_refresh_flag: 0,
        }
    }
}

impl SaveProgressState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut dungeon_info = vec![0; SAVE_DUNGEON_INFO_LEN];
        if let Some(source) = ram.get(SAVE_DUNG_INFO..SAVE_DUNG_INFO + SAVE_DUNGEON_INFO_LEN) {
            dungeon_info.copy_from_slice(source);
        }

        Self {
            palace_index_x2: ram_byte(ram, CUR_PALACE_INDEX_X2),
            hud_current_items: [
                ram_byte(ram, HUD_CUR_ITEM),
                ram_byte(ram, HUD_CUR_ITEM_X),
                ram_byte(ram, HUD_CUR_ITEM_L),
                ram_byte(ram, HUD_CUR_ITEM_R),
            ],
            dungeon_info,
            post_message_refresh_flag: ram_byte(ram, HUD_POST_MESSAGE_REFRESH_FLAG),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + SAVE_DUNGEON_INFO_LEN]
            .copy_from_slice(&self.dungeon_info);
        ram[CUR_PALACE_INDEX_X2] = self.palace_index_x2;
        ram[HUD_CUR_ITEM] = self.hud_current_items[0];
        ram[HUD_CUR_ITEM_X] = self.hud_current_items[1];
        ram[HUD_CUR_ITEM_L] = self.hud_current_items[2];
        ram[HUD_CUR_ITEM_R] = self.hud_current_items[3];
        ram[HUD_POST_MESSAGE_REFRESH_FLAG] = self.post_message_refresh_flag;
    }

    fn save_offset(address: usize) -> usize {
        address - SAVE_DUNG_INFO
    }

    fn save_byte(&self, address: usize) -> u8 {
        self.dungeon_info
            .get(Self::save_offset(address))
            .copied()
            .unwrap_or(0)
    }

    fn set_save_byte(&mut self, address: usize, value: u8) {
        if let Some(byte) = self.dungeon_info.get_mut(Self::save_offset(address)) {
            *byte = value;
        }
    }

    fn save_word(&self, address: usize) -> u16 {
        let offset = Self::save_offset(address);
        if offset + 1 < self.dungeon_info.len() {
            u16::from(self.dungeon_info[offset]) | (u16::from(self.dungeon_info[offset + 1]) << 8)
        } else {
            0
        }
    }

    fn set_save_word(&mut self, address: usize, value: u16) {
        let offset = Self::save_offset(address);
        if offset + 1 < self.dungeon_info.len() {
            self.dungeon_info[offset] = value as u8;
            self.dungeon_info[offset + 1] = (value >> 8) as u8;
        }
    }

    pub(crate) fn palace_index_x2(&self) -> u8 {
        self.palace_index_x2
    }

    pub(crate) fn palace_index_x2_word(&self) -> u16 {
        u16::from(self.palace_index_x2)
    }

    pub(crate) fn palace_index(&self) -> usize {
        usize::from(self.palace_index_x2() >> 1)
    }

    pub(crate) fn progress_indicator(&self) -> u8 {
        self.save_byte(SRAM_PROGRESS_INDICATOR)
    }

    pub(crate) fn progress_indicator_word(&self) -> u16 {
        self.save_word(SRAM_PROGRESS_INDICATOR)
    }

    pub(crate) fn progress_flags(&self) -> u8 {
        self.save_byte(SRAM_PROGRESS_FLAGS)
    }

    pub(crate) fn progress_flags_has(&self, mask: u8) -> bool {
        self.progress_flags() & mask != 0
    }

    pub(crate) fn map_icons_indicator(&self) -> u8 {
        self.save_byte(SAVEGAME_MAP_ICONS_INDICATOR)
    }

    pub(crate) fn dark_world_state(&self) -> u8 {
        self.save_byte(SAVEGAME_IS_DARKWORLD)
    }

    pub(crate) fn is_dark_world(&self) -> bool {
        self.dark_world_state() != 0
    }

    pub(crate) fn dark_world_bit6(&self) -> u8 {
        (self.dark_world_state() >> 6) & 1
    }

    pub(crate) fn hud_current_item(&self) -> u8 {
        self.hud_current_item_slot(0)
    }

    pub(crate) fn hud_current_item_slot(&self, slot: usize) -> u8 {
        self.hud_current_items
            .get(slot)
            .copied()
            .unwrap_or(self.hud_current_items[0])
    }

    pub(crate) fn dungeon_info_word(&self, room: usize) -> u16 {
        let offset = room * 2;
        if offset + 1 < self.dungeon_info.len() {
            u16::from(self.dungeon_info[offset]) | (u16::from(self.dungeon_info[offset + 1]) << 8)
        } else {
            0
        }
    }

    pub(crate) fn death_count_for_palace(&self, palace: usize) -> u16 {
        if palace < DEATH_COUNT_PALACE_SLOTS {
            self.save_word(DEATHS_PER_PALACE + palace * 2)
        } else {
            0
        }
    }

    pub(crate) fn pending_death_save_counter(&self) -> u16 {
        self.save_word(PENDING_DEATH_SAVE_COUNTER)
    }

    pub(crate) fn total_death_save_counter(&self) -> u16 {
        self.save_word(TOTAL_DEATH_SAVE_COUNTER)
    }

    pub(crate) fn total_death_save_counter_is_uninitialized(&self) -> bool {
        self.total_death_save_counter() == 0xffff
    }

    pub(crate) fn dungeon_info_slice(&self) -> &[u8] {
        &self.dungeon_info
    }

    pub(crate) fn which_starting_point(&self) -> u8 {
        self.save_byte(WHICH_STARTING_POINT)
    }

    pub(crate) fn progress_indicator_3(&self) -> u8 {
        self.save_byte(SRAM_PROGRESS_INDICATOR_3)
    }

    pub(crate) fn set_palace_index_x2(&mut self, value: u8) {
        self.palace_index_x2 = value;
    }

    pub(crate) fn set_which_starting_point(&mut self, value: u8) {
        self.set_save_byte(WHICH_STARTING_POINT, value);
    }

    pub(crate) fn xor_palace_index_x2(&mut self, value: u8) {
        self.palace_index_x2 ^= value;
    }

    pub(crate) fn set_progress_indicator(&mut self, value: u8) {
        self.set_save_byte(SRAM_PROGRESS_INDICATOR, value);
    }

    pub(crate) fn or_progress_flags(&mut self, value: u8) {
        self.set_save_byte(SRAM_PROGRESS_FLAGS, self.progress_flags() | value);
    }

    pub(crate) fn or_progress_indicator_3(&mut self, bits: u8) {
        self.set_save_byte(
            SRAM_PROGRESS_INDICATOR_3,
            self.progress_indicator_3() | bits,
        );
    }

    pub(crate) fn set_progress_indicator_3(&mut self, value: u8) {
        self.set_save_byte(SRAM_PROGRESS_INDICATOR_3, value);
    }

    pub(crate) fn clear_progress_indicator_3_bits(&mut self, bits: u8) {
        self.set_save_byte(
            SRAM_PROGRESS_INDICATOR_3,
            self.progress_indicator_3() & !bits,
        );
    }

    pub(crate) fn xor_progress_flags(&mut self, value: u8) {
        self.set_save_byte(SRAM_PROGRESS_FLAGS, self.progress_flags() ^ value);
    }

    pub(crate) fn set_progress_flags(&mut self, value: u8) {
        self.set_save_byte(SRAM_PROGRESS_FLAGS, value);
    }

    pub(crate) fn set_map_icons_indicator(&mut self, value: u8) {
        self.set_save_byte(SAVEGAME_MAP_ICONS_INDICATOR, value);
    }

    pub(crate) fn set_dark_world_state(&mut self, value: u8) {
        self.set_save_byte(SAVEGAME_IS_DARKWORLD, value);
    }

    pub(crate) fn xor_dark_world_state(&mut self, value: u8) {
        self.set_save_byte(SAVEGAME_IS_DARKWORLD, self.dark_world_state() ^ value);
    }

    pub(crate) fn set_hud_current_item(&mut self, value: u8) {
        self.set_hud_current_item_slot(0, value);
    }

    pub(crate) fn set_hud_current_item_slot(&mut self, slot: usize, value: u8) {
        let slot = slot.min(self.hud_current_items.len() - 1);
        self.hud_current_items[slot] = value;
    }

    pub(crate) fn set_death_count_for_palace(&mut self, palace: usize, value: u16) {
        if palace < DEATH_COUNT_PALACE_SLOTS {
            self.set_save_word(DEATHS_PER_PALACE + palace * 2, value);
        }
    }

    pub(crate) fn increment_pending_death_save_counter(&mut self) -> u16 {
        let deaths = self.pending_death_save_counter().wrapping_add(1);
        self.set_save_word(PENDING_DEATH_SAVE_COUNTER, deaths);
        deaths
    }

    pub(crate) fn clear_pending_death_save_counter(&mut self) {
        self.set_save_word(PENDING_DEATH_SAVE_COUNTER, 0);
    }

    pub(crate) fn set_total_death_save_counter(&mut self, value: u16) {
        self.set_save_word(TOTAL_DEATH_SAVE_COUNTER, value);
    }

    pub(crate) fn clear_post_message_refresh_flag(&mut self) {
        self.post_message_refresh_flag = 0;
    }

    pub(crate) fn request_post_message_refresh(&mut self) {
        self.post_message_refresh_flag = 0x80;
    }

    pub(crate) fn clear_dungeon_info(&mut self) {
        self.dungeon_info.fill(0);
    }

    pub(crate) fn copy_dungeon_info_from(&mut self, source: &[u8]) {
        self.dungeon_info.copy_from_slice(source);
    }

    pub(crate) fn set_dungeon_info_word(&mut self, room: usize, value: u16) {
        let offset = room * 2;
        if offset + 1 < self.dungeon_info.len() {
            self.dungeon_info[offset] = value as u8;
            self.dungeon_info[offset + 1] = (value >> 8) as u8;
        }
    }

    pub(crate) fn or_dungeon_info_word(&mut self, room: usize, value: u16) -> u16 {
        let word = self.dungeon_info_word(room) | value;
        self.set_dungeon_info_word(room, word);
        word
    }

    pub(crate) fn set_dungeon_info_checksum(&mut self, value: u16) {
        self.set_dungeon_info_word(0x27f, value);
    }

    pub(crate) fn compute_dungeon_info_checksum(&self) -> u16 {
        let mut checksum = 0x5a5au16;
        for i in (0..0x4fe).step_by(2) {
            let word = u16::from(self.dungeon_info[i]) | (u16::from(self.dungeon_info[i + 1]) << 8);
            checksum = checksum.wrapping_sub(word);
        }
        checksum
    }
}

pub(crate) struct NativeSaveProgressBridgeMut<'a> {
    state: &'a mut SaveProgressState,
    ram: &'a mut [u8],
}

impl<'a> NativeSaveProgressBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SaveProgressState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync_all(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn sync_byte(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
    }

    fn sync_word(&mut self, address: usize, value: u16) {
        write_le_u16(self.ram, address, value);
    }

    fn sync_hud_current_item_slot(&mut self, slot: usize) {
        let address = match slot {
            0 => HUD_CUR_ITEM,
            1 => HUD_CUR_ITEM_X,
            2 => HUD_CUR_ITEM_L,
            _ => HUD_CUR_ITEM_R,
        };
        self.sync_byte(address, self.state.hud_current_item_slot(slot));
    }

    fn sync_dungeon_info_word(&mut self, room: usize) {
        if room * 2 + 1 < SAVE_DUNGEON_INFO_LEN {
            write_le_u16(
                self.ram,
                SAVE_DUNG_INFO + room * 2,
                self.state.dungeon_info_word(room),
            );
        }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, SaveProgressState::load_from_ram(self.ram));
    }

    pub(crate) fn set_palace_index_x2(&mut self, value: u8) {
        self.state.set_palace_index_x2(value);
        self.sync_byte(CUR_PALACE_INDEX_X2, self.state.palace_index_x2());
    }

    pub(crate) fn set_which_starting_point(&mut self, value: u8) {
        self.state.set_which_starting_point(value);
        self.sync_byte(WHICH_STARTING_POINT, self.state.which_starting_point());
    }

    pub(crate) fn xor_palace_index_x2(&mut self, value: u8) {
        self.state.xor_palace_index_x2(value);
        self.sync_byte(CUR_PALACE_INDEX_X2, self.state.palace_index_x2());
    }

    pub(crate) fn set_progress_indicator(&mut self, value: u8) {
        self.state.set_progress_indicator(value);
        self.sync_byte(SRAM_PROGRESS_INDICATOR, self.state.progress_indicator());
    }

    pub(crate) fn or_progress_flags(&mut self, value: u8) {
        self.state.or_progress_flags(value);
        self.sync_byte(SRAM_PROGRESS_FLAGS, self.state.progress_flags());
    }

    pub(crate) fn or_progress_indicator_3(&mut self, bits: u8) {
        self.state.or_progress_indicator_3(bits);
        self.sync_byte(SRAM_PROGRESS_INDICATOR_3, self.state.progress_indicator_3());
    }

    pub(crate) fn set_progress_indicator_3(&mut self, value: u8) {
        self.state.set_progress_indicator_3(value);
        self.sync_byte(SRAM_PROGRESS_INDICATOR_3, self.state.progress_indicator_3());
    }

    pub(crate) fn clear_progress_indicator_3_bits(&mut self, bits: u8) {
        self.state.clear_progress_indicator_3_bits(bits);
        self.sync_byte(SRAM_PROGRESS_INDICATOR_3, self.state.progress_indicator_3());
    }

    pub(crate) fn xor_progress_flags(&mut self, value: u8) {
        self.state.xor_progress_flags(value);
        self.sync_byte(SRAM_PROGRESS_FLAGS, self.state.progress_flags());
    }

    pub(crate) fn set_progress_flags(&mut self, value: u8) {
        self.state.set_progress_flags(value);
        self.sync_byte(SRAM_PROGRESS_FLAGS, self.state.progress_flags());
    }

    pub(crate) fn set_map_icons_indicator(&mut self, value: u8) {
        self.state.set_map_icons_indicator(value);
        self.sync_byte(
            SAVEGAME_MAP_ICONS_INDICATOR,
            self.state.map_icons_indicator(),
        );
    }

    pub(crate) fn set_dark_world_state(&mut self, value: u8) {
        self.state.set_dark_world_state(value);
        self.sync_byte(SAVEGAME_IS_DARKWORLD, self.state.dark_world_state());
    }

    pub(crate) fn xor_dark_world_state(&mut self, value: u8) {
        self.state.xor_dark_world_state(value);
        self.sync_byte(SAVEGAME_IS_DARKWORLD, self.state.dark_world_state());
    }

    pub(crate) fn set_hud_current_item(&mut self, value: u8) {
        self.state.set_hud_current_item(value);
        self.sync_hud_current_item_slot(0);
    }

    pub(crate) fn set_hud_current_item_slot(&mut self, slot: usize, value: u8) {
        self.state.set_hud_current_item_slot(slot, value);
        self.sync_hud_current_item_slot(slot);
    }

    pub(crate) fn set_death_count_for_palace(&mut self, palace: usize, value: u16) {
        self.state.set_death_count_for_palace(palace, value);
        if palace < DEATH_COUNT_PALACE_SLOTS {
            self.sync_word(
                DEATHS_PER_PALACE + palace * 2,
                self.state.death_count_for_palace(palace),
            );
        }
    }

    pub(crate) fn increment_pending_death_save_counter(&mut self) -> u16 {
        let deaths = self.state.increment_pending_death_save_counter();
        self.sync_word(PENDING_DEATH_SAVE_COUNTER, deaths);
        deaths
    }

    pub(crate) fn clear_pending_death_save_counter(&mut self) {
        self.state.clear_pending_death_save_counter();
        self.sync_word(
            PENDING_DEATH_SAVE_COUNTER,
            self.state.pending_death_save_counter(),
        );
    }

    pub(crate) fn set_total_death_save_counter(&mut self, value: u16) {
        self.state.set_total_death_save_counter(value);
        self.sync_word(
            TOTAL_DEATH_SAVE_COUNTER,
            self.state.total_death_save_counter(),
        );
    }

    pub(crate) fn clear_post_message_refresh_flag(&mut self) {
        self.state.clear_post_message_refresh_flag();
        self.sync_byte(HUD_POST_MESSAGE_REFRESH_FLAG, 0);
    }

    pub(crate) fn request_post_message_refresh(&mut self) {
        self.state.request_post_message_refresh();
        self.sync_byte(HUD_POST_MESSAGE_REFRESH_FLAG, 0x80);
    }

    pub(crate) fn clear_dungeon_info(&mut self) {
        self.state.clear_dungeon_info();
        self.sync_all();
    }

    pub(crate) fn copy_dungeon_info_from(&mut self, source: &[u8]) {
        self.state.copy_dungeon_info_from(source);
        self.sync_all();
    }

    pub(crate) fn set_dungeon_info_word(&mut self, room: usize, value: u16) {
        self.state.set_dungeon_info_word(room, value);
        self.sync_dungeon_info_word(room);
    }

    pub(crate) fn or_dungeon_info_word(&mut self, room: usize, value: u16) -> u16 {
        let word = self.state.or_dungeon_info_word(room, value);
        self.sync_dungeon_info_word(room);
        word
    }

    pub(crate) fn set_dungeon_info_checksum(&mut self, value: u16) {
        self.state.set_dungeon_info_checksum(value);
        self.sync_dungeon_info_word(0x27f);
    }

    pub(crate) fn compute_dungeon_info_checksum(&self) -> u16 {
        self.state.compute_dungeon_info_checksum()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonKeySlotsState {
    keys_earned: [u8; DUNGEON_KEY_SLOT_COUNT],
}

impl DungeonKeySlotsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut keys_earned = [0; DUNGEON_KEY_SLOT_COUNT];
        for (slot, keys) in keys_earned.iter_mut().enumerate() {
            *keys = ram_byte(ram, LINK_KEYS_EARNED_PER_DUNGEON + slot);
        }
        Self { keys_earned }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (slot, keys) in self.keys_earned.iter().copied().enumerate() {
            ram[LINK_KEYS_EARNED_PER_DUNGEON + slot] = keys;
        }
    }

    pub(crate) fn keys_earned(&self, palace_index_x2: u8) -> u8 {
        self.keys_earned_slot(usize::from(palace_index_x2 >> 1))
    }

    pub(crate) fn keys_earned_slot(&self, slot: usize) -> u8 {
        self.keys_earned.get(slot).copied().unwrap_or(0)
    }
}

pub(crate) struct NativeDungeonKeySlotsBridgeMut<'a> {
    state: &'a mut DungeonKeySlotsState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonKeySlotsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonKeySlotsState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonKeySlotsState::load_from_ram(self.ram));
    }

    pub(crate) fn set_keys_earned(&mut self, palace_index_x2: u8, keys: u8) {
        self.set_keys_earned_slot(usize::from(palace_index_x2 >> 1), keys);
    }

    pub(crate) fn set_keys_earned_slot(&mut self, slot: usize, keys: u8) {
        if let Some(value) = self.state.keys_earned.get_mut(slot) {
            *value = keys;
            self.sync();
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerResourcesState {
    // link_magic_power (0xf36e) is owned solely by FollowerLinkState; PlayerResources-
    // State must not mirror it (it projects after FollowerLinkState and would clobber
    // item magic-consumption with a stale value). magic_filler stays here.
    magic_consumption_level: u8,
    bombs: u8,
    equipped_bottle_index: u8,
    rupees_goal: u16,
    rupees_actual: u16,
    compass_flags: u16,
    big_key_flags: u16,
    dungeon_map_flags: u16,
    rupees_in_pond: u8,
    heart_pieces: u8,
    health_capacity: u8,
    current_health: u8,
    keys: u8,
    bomb_upgrade_level: u8,
    arrow_upgrade_level: u8,
    heart_filler: u8,
    magic_filler: u8,
    pendant_flags: u8,
    bomb_filler: u8,
    arrow_filler: u8,
    arrows: u8,
    ability_flags: u8,
    crystal_flags: u8,
    low_health_beep_timer: u8,
}

impl PlayerResourcesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            magic_consumption_level: ram_byte(ram, LINK_MAGIC_CONSUMPTION),
            bombs: ram_byte(ram, LINK_ITEM_BOMBS),
            equipped_bottle_index: ram_byte(ram, LINK_ITEM_BOTTLE_INDEX),
            rupees_goal: read_word(ram, LINK_RUPEES_GOAL),
            rupees_actual: read_word(ram, LINK_RUPEES_ACTUAL),
            compass_flags: read_word(ram, LINK_COMPASS),
            big_key_flags: read_word(ram, LINK_BIGKEY),
            dungeon_map_flags: read_word(ram, LINK_DUNGEON_MAP),
            rupees_in_pond: ram_byte(ram, LINK_RUPEES_IN_POND),
            heart_pieces: ram_byte(ram, LINK_HEART_PIECES),
            health_capacity: ram_byte(ram, LINK_HEALTH_CAPACITY),
            current_health: ram_byte(ram, LINK_CURRENT_HEALTH),
            keys: ram_byte(ram, LINK_NUM_KEYS),
            bomb_upgrade_level: ram_byte(ram, LINK_BOMB_UPGRADES),
            arrow_upgrade_level: ram_byte(ram, LINK_ARROW_UPGRADES),
            heart_filler: ram_byte(ram, LINK_HEARTS_FILLER),
            magic_filler: ram_byte(ram, LINK_MAGIC_FILLER),
            pendant_flags: ram_byte(ram, LINK_WHICH_PENDANTS),
            bomb_filler: ram_byte(ram, LINK_BOMB_FILLER),
            arrow_filler: ram_byte(ram, LINK_ARROW_REFILL_COUNTER),
            arrows: ram_byte(ram, LINK_NUM_ARROWS),
            ability_flags: ram_byte(ram, LINK_ABILITY_FLAGS),
            crystal_flags: ram_byte(ram, LINK_HAS_CRYSTALS),
            low_health_beep_timer: ram_byte(ram, LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[LINK_MAGIC_CONSUMPTION] = self.magic_consumption_level;
        ram[LINK_ITEM_BOMBS] = self.bombs;
        ram[LINK_ITEM_BOTTLE_INDEX] = self.equipped_bottle_index;
        write_le_u16(ram, LINK_RUPEES_GOAL, self.rupees_goal);
        write_le_u16(ram, LINK_RUPEES_ACTUAL, self.rupees_actual);
        write_le_u16(ram, LINK_COMPASS, self.compass_flags);
        write_le_u16(ram, LINK_BIGKEY, self.big_key_flags);
        write_le_u16(ram, LINK_DUNGEON_MAP, self.dungeon_map_flags);
        ram[LINK_RUPEES_IN_POND] = self.rupees_in_pond;
        ram[LINK_HEART_PIECES] = self.heart_pieces;
        ram[LINK_HEALTH_CAPACITY] = self.health_capacity;
        ram[LINK_CURRENT_HEALTH] = self.current_health;
        ram[LINK_NUM_KEYS] = self.keys;
        ram[LINK_BOMB_UPGRADES] = self.bomb_upgrade_level;
        ram[LINK_ARROW_UPGRADES] = self.arrow_upgrade_level;
        ram[LINK_HEARTS_FILLER] = self.heart_filler;
        ram[LINK_MAGIC_FILLER] = self.magic_filler;
        ram[LINK_WHICH_PENDANTS] = self.pendant_flags;
        ram[LINK_BOMB_FILLER] = self.bomb_filler;
        ram[LINK_ARROW_REFILL_COUNTER] = self.arrow_filler;
        ram[LINK_NUM_ARROWS] = self.arrows;
        ram[LINK_ABILITY_FLAGS] = self.ability_flags;
        ram[LINK_HAS_CRYSTALS] = self.crystal_flags;
        ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] = self.low_health_beep_timer;
    }

    pub(crate) fn magic_filler(&self) -> u8 {
        self.magic_filler
    }

    pub(crate) fn magic_consumption_level(&self) -> u8 {
        self.magic_consumption_level
    }

    pub(crate) fn bomb_filler(&self) -> u8 {
        self.bomb_filler
    }

    pub(crate) fn bombs(&self) -> u8 {
        self.bombs
    }

    pub(crate) fn bomb_upgrade_level(&self) -> u8 {
        self.bomb_upgrade_level
    }

    pub(crate) fn next_bomb_upgrade_level(&self) -> u8 {
        self.bomb_upgrade_level().wrapping_add(1)
    }

    pub(crate) fn arrow_filler(&self) -> u8 {
        self.arrow_filler
    }

    pub(crate) fn arrows(&self) -> u8 {
        self.arrows
    }

    pub(crate) fn arrow_upgrade_level(&self) -> u8 {
        self.arrow_upgrade_level
    }

    pub(crate) fn next_arrow_upgrade_level(&self) -> u8 {
        self.arrow_upgrade_level().wrapping_add(1)
    }

    pub(crate) fn has_bomb_or_arrow_upgrade(&self) -> bool {
        self.bomb_upgrade_level | self.arrow_upgrade_level != 0
    }

    pub(crate) fn current_health(&self) -> u8 {
        self.current_health
    }

    pub(crate) fn health_capacity(&self) -> u8 {
        self.health_capacity
    }

    pub(crate) fn heart_filler(&self) -> u8 {
        self.heart_filler
    }

    pub(crate) fn low_health_beep_timer(&self) -> u8 {
        self.low_health_beep_timer
    }

    pub(crate) fn equipped_bottle_index(&self) -> u8 {
        self.equipped_bottle_index
    }

    pub(crate) fn rupees_goal(&self) -> u16 {
        self.rupees_goal
    }

    pub(crate) fn rupees_actual(&self) -> u16 {
        self.rupees_actual
    }

    pub(crate) fn compass_flags(&self) -> u16 {
        self.compass_flags
    }

    pub(crate) fn big_key_flags(&self) -> u16 {
        self.big_key_flags
    }

    pub(crate) fn dungeon_map_flags(&self) -> u16 {
        self.dungeon_map_flags
    }

    pub(crate) fn has_compass_mask(&self, mask: u16) -> bool {
        self.compass_flags() & mask != 0
    }

    pub(crate) fn has_big_key_mask(&self, mask: u16) -> bool {
        self.big_key_flags() & mask != 0
    }

    pub(crate) fn lacks_big_key_mask(&self, mask: u16) -> bool {
        !self.has_big_key_mask(mask)
    }

    pub(crate) fn has_dungeon_map_mask(&self, mask: u16) -> bool {
        self.dungeon_map_flags() & mask != 0
    }

    pub(crate) fn has_big_key_at_shift(&self, shift: u8) -> bool {
        (self.big_key_flags() << shift) & 0x8000 != 0
    }

    pub(crate) fn has_dungeon_map_at_shift(&self, shift: u8) -> bool {
        (self.dungeon_map_flags() << shift) & 0x8000 != 0
    }

    pub(crate) fn has_compass_at_shift(&self, shift: u8) -> bool {
        (self.compass_flags() << shift) & 0x8000 != 0
    }

    pub(crate) fn ability_flags(&self) -> u8 {
        self.ability_flags
    }

    pub(crate) fn pendant_flags(&self) -> u8 {
        self.pendant_flags
    }

    pub(crate) fn crystal_flags(&self) -> u8 {
        self.crystal_flags
    }

    pub(crate) fn heart_pieces(&self) -> u8 {
        self.heart_pieces
    }

    pub(crate) fn keys(&self) -> u8 {
        self.keys
    }

    pub(crate) fn rupees_in_pond(&self) -> u8 {
        self.rupees_in_pond
    }
}

pub(crate) struct NativePlayerResourcesBridgeMut<'a> {
    resources: &'a mut PlayerResourcesState,
    ram: &'a mut [u8],
}

impl<'a> NativePlayerResourcesBridgeMut<'a> {
    pub(crate) fn new(resources: &'a mut PlayerResourcesState, ram: &'a mut [u8]) -> Self {
        Self { resources, ram }
    }

    fn sync(&mut self) {
        self.resources.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.resources,
            PlayerResourcesState::load_from_ram(self.ram)
        );
    }

    /// OR a bit into one of the dungeon save-flag words (compass / big-key / dungeon-map)
    /// that this state owns. Used by item receipt (compass 0x32 / map 0x33 / big-key
    /// 0x25): updating the native field — not just RAM — is required or write_to_ram would
    /// re-project the stale value and clobber the just-acquired flag.
    pub(crate) fn or_resource_flag_word(&mut self, addr: usize, mask: u16) {
        match addr {
            LINK_COMPASS => self.resources.compass_flags |= mask,
            LINK_BIGKEY => self.resources.big_key_flags |= mask,
            LINK_DUNGEON_MAP => self.resources.dungeon_map_flags |= mask,
            _ => {
                let next = read_le_u16(self.ram, addr) | mask;
                write_le_u16(self.ram, addr, next);
                return;
            }
        }
        self.sync();
    }

    pub(crate) fn set_magic_consumption_level(&mut self, value: u8) {
        self.resources.magic_consumption_level = value;
        self.sync();
    }

    pub(crate) fn set_magic_filler(&mut self, value: u8) {
        self.resources.magic_filler = value;
        self.sync();
    }

    pub(crate) fn clear_magic_filler(&mut self) {
        self.resources.magic_filler = 0;
        self.sync();
    }

    pub(crate) fn decrement_magic_filler(&mut self) {
        self.resources.magic_filler = self.resources.magic_filler.wrapping_sub(1);
        self.sync();
    }

    pub(crate) fn decrement_bomb_filler(&mut self) {
        self.resources.bomb_filler = self.resources.bomb_filler.wrapping_sub(1);
        self.sync();
    }

    pub(crate) fn set_bomb_filler(&mut self, value: u8) {
        self.resources.bomb_filler = value;
        self.sync();
    }

    pub(crate) fn increment_bomb_filler_by(&mut self, value: u8) {
        self.resources.bomb_filler = self.resources.bomb_filler.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn set_bombs(&mut self, value: u8) {
        self.resources.bombs = value;
        self.sync();
    }

    pub(crate) fn increment_bombs(&mut self) {
        self.resources.bombs = self.resources.bombs.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn decrement_bombs(&mut self) -> u8 {
        self.resources.bombs = self.resources.bombs.wrapping_sub(1);
        let value = self.resources.bombs;
        self.sync();
        value
    }

    pub(crate) fn increment_health_capacity_by(&mut self, value: u8) -> u8 {
        self.resources.health_capacity = self.resources.health_capacity.wrapping_add(value);
        let capacity = self.resources.health_capacity;
        self.sync();
        capacity
    }

    pub(crate) fn increment_heart_filler_by(&mut self, value: u8) {
        self.resources.heart_filler = self.resources.heart_filler.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn increment_heart_filler_word_by(&mut self, value: u16) -> u16 {
        let hearts = self.resources.heart_filler_word().wrapping_add(value);
        self.resources.heart_filler = hearts as u8;
        self.resources.magic_filler = (hearts >> 8) as u8;
        self.sync();
        hearts
    }

    pub(crate) fn increment_magic_filler_by(&mut self, value: u8) {
        self.resources.magic_filler = self.resources.magic_filler.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn add_ability_flags(&mut self, flags: u8) {
        self.resources.ability_flags |= flags;
        self.sync();
    }

    pub(crate) fn add_crystal_flags(&mut self, flags: u8) {
        self.resources.crystal_flags |= flags;
        self.sync();
    }

    pub(crate) fn set_crystal_flags(&mut self, flags: u8) {
        self.resources.crystal_flags = flags;
        self.sync();
    }

    pub(crate) fn set_pendant_flags(&mut self, flags: u8) {
        self.resources.pendant_flags = flags;
        self.sync();
    }

    pub(crate) fn decrement_arrow_filler(&mut self) {
        self.resources.arrow_filler = self.resources.arrow_filler.wrapping_sub(1);
        self.sync();
    }

    pub(crate) fn set_arrow_filler(&mut self, value: u8) {
        self.resources.arrow_filler = value;
        self.sync();
    }

    pub(crate) fn increment_arrow_filler_by(&mut self, value: u8) {
        self.resources.arrow_filler = self.resources.arrow_filler.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn increment_arrows(&mut self) {
        self.resources.arrows = self.resources.arrows.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn increment_arrows_by(&mut self, value: u8) {
        self.resources.arrows = self.resources.arrows.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn set_arrows(&mut self, value: u8) {
        self.resources.arrows = value;
        self.sync();
    }

    pub(crate) fn decrement_arrows(&mut self) -> u8 {
        self.resources.arrows = self.resources.arrows.wrapping_sub(1);
        let value = self.resources.arrows;
        self.sync();
        value
    }

    pub(crate) fn set_current_health(&mut self, value: u8) {
        self.resources.current_health = value;
        self.sync();
    }

    pub(crate) fn increment_current_health_by(&mut self, value: u8) {
        self.resources.current_health = self.resources.current_health.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn decrement_current_health_by(&mut self, value: u8) -> u8 {
        self.resources.current_health = self.resources.current_health.wrapping_sub(value);
        let health = self.resources.current_health;
        self.sync();
        health
    }

    pub(crate) fn set_heart_filler(&mut self, value: u8) {
        self.resources.heart_filler = value;
        self.sync();
    }

    pub(crate) fn decrement_heart_filler_by(&mut self, value: u8) {
        self.resources.heart_filler = self.resources.heart_filler.wrapping_sub(value);
        self.sync();
    }

    pub(crate) fn decrement_low_health_beep_timer(&mut self) {
        self.resources.low_health_beep_timer = self.resources.low_health_beep_timer.wrapping_sub(1);
        self.sync();
    }

    pub(crate) fn set_low_health_beep_timer(&mut self, value: u8) {
        self.resources.low_health_beep_timer = value;
        self.sync();
    }

    pub(crate) fn set_equipped_bottle_index(&mut self, value: u8) {
        self.resources.equipped_bottle_index = value;
        self.sync();
    }

    pub(crate) fn set_rupees_goal(&mut self, value: u16) {
        self.resources.rupees_goal = value;
        self.sync();
    }

    pub(crate) fn set_rupees_actual(&mut self, value: u16) {
        self.resources.rupees_actual = value;
        self.sync();
    }

    pub(crate) fn add_rupees_goal(&mut self, value: u16) -> u16 {
        self.resources.rupees_goal = self.resources.rupees_goal.wrapping_add(value);
        let rupees = self.resources.rupees_goal;
        self.sync();
        rupees
    }

    pub(crate) fn subtract_rupees_goal(&mut self, value: u16) -> u16 {
        self.resources.rupees_goal = self.resources.rupees_goal.wrapping_sub(value);
        let rupees = self.resources.rupees_goal;
        self.sync();
        rupees
    }

    pub(crate) fn set_keys(&mut self, value: u8) {
        self.resources.keys = value;
        self.sync();
    }

    pub(crate) fn increment_keys(&mut self) -> u8 {
        self.resources.keys = self.resources.keys.wrapping_add(1);
        let keys = self.resources.keys;
        self.sync();
        keys
    }

    pub(crate) fn decrement_keys(&mut self) -> u8 {
        self.resources.keys = self.resources.keys.wrapping_sub(1);
        let keys = self.resources.keys;
        self.sync();
        keys
    }

    pub(crate) fn advance_heart_piece_count(&mut self) -> u8 {
        self.resources.heart_pieces = self.resources.heart_pieces.wrapping_add(1) & 3;
        let heart_pieces = self.resources.heart_pieces;
        self.sync();
        heart_pieces
    }

    pub(crate) fn add_rupees_to_pond(&mut self, value: u8) -> u8 {
        self.resources.rupees_in_pond = self.resources.rupees_in_pond.wrapping_add(value);
        let pond = self.resources.rupees_in_pond;
        self.sync();
        pond
    }

    pub(crate) fn subtract_pond_reward_threshold(&mut self) -> u8 {
        self.resources.rupees_in_pond = self.resources.rupees_in_pond.wrapping_sub(100);
        let pond = self.resources.rupees_in_pond;
        self.sync();
        pond
    }

    pub(crate) fn set_bomb_upgrade_level(&mut self, value: u8) {
        self.resources.bomb_upgrade_level = value;
        self.sync();
    }

    pub(crate) fn set_arrow_upgrade_level(&mut self, value: u8) {
        self.resources.arrow_upgrade_level = value;
        self.sync();
    }
}

impl PlayerResourcesState {
    fn heart_filler_word(&self) -> u16 {
        u16::from(self.heart_filler) | (u16::from(self.magic_filler) << 8)
    }
}

fn read_word(ram: &[u8], offset: usize) -> u16 {
    if offset + 1 < ram.len() {
        read_le_u16(ram, offset)
    } else {
        0
    }
}
