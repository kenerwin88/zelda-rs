use super::player_magic::PlayerMagicState;
use super::ram_byte;
use super::save_progress::SaveProgressState;
use crate::game_state::constants::*;
use crate::game_state::native::ram_target::RamTarget;
use crate::types::read_le_u16;

const DUNGEON_KEY_SLOT_COUNT: usize = 16;
const BOTTLE_SLOT_COUNT: usize = 4;

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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        self.dungeon_key_slots.write_to_ram(ram);
        self.mirror_warp.write_to_ram(ram);
    }
}

// Equipment has semantic fields. Only this compatibility map knows the old
// inventory indices; bomb count and equipped-bottle index belong to resources.
macro_rules! equipment_fields {
    ($($kind:ident => $field:ident = $index:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum EquipmentItem { $($kind),+ }

        impl EquipmentItem {
            pub(crate) fn from_inventory_index(index: usize) -> Option<Self> {
                match index { $($index => Some(Self::$kind),)+ _ => None }
            }
        }

        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub(crate) struct InventoryItemsState {
            $($field: u8,)+
            bottles: [u8; BOTTLE_SLOT_COUNT],
        }

        non_wram_hash! { InventoryItemsState {
            wram: [$($field,)+ bottles], children: [], metadata: [],
        } }

        impl InventoryItemsState {
            pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
                Self {
                    $($field: ram_byte(ram, LINK_ITEM_BOW + $index),)+
                    bottles: std::array::from_fn(|i| ram_byte(ram, LINK_BOTTLE_INFO + i)),
                }
            }
            #[cfg(test)]
            pub(crate) fn export_owned(&self, image: &mut [u8]) {
                $(image[LINK_ITEM_BOW + $index] = self.$field;)+
                image[LINK_BOTTLE_INFO..LINK_BOTTLE_INFO + BOTTLE_SLOT_COUNT].copy_from_slice(&self.bottles);
            }
            fn owned_byte(&self, address: usize) -> Option<u8> {
                match address {
                    $(a if a == LINK_ITEM_BOW + $index => Some(self.$field),)+
                    a if (LINK_BOTTLE_INFO..LINK_BOTTLE_INFO + BOTTLE_SLOT_COUNT).contains(&a) => Some(self.bottles[a - LINK_BOTTLE_INFO]),
                    _ => None,
                }
            }
            fn write_owned_byte(&mut self, address: usize, value: u8) -> bool {
                match address {
                    $(a if a == LINK_ITEM_BOW + $index => self.$field = value,)+
                    a if (LINK_BOTTLE_INFO..LINK_BOTTLE_INFO + BOTTLE_SLOT_COUNT).contains(&a) => self.bottles[a - LINK_BOTTLE_INFO] = value,
                    _ => return false,
                }
                true
            }
            pub(crate) fn equipment(&self, item: EquipmentItem) -> u8 {
                match item { $(EquipmentItem::$kind => self.$field,)+ }
            }
            fn set_equipment(&mut self, item: EquipmentItem, value: u8) {
                match item { $(EquipmentItem::$kind => self.$field = value,)+ }
            }
        }
    };
}

equipment_fields! {
    Bow => bow = 0,
    Boomerang => boomerang = 1,
    Hookshot => hookshot = 2,
    Mushroom => mushroom = 4,
    FireRod => fire_rod = 5,
    IceRod => ice_rod = 6,
    Bombos => bombos = 7,
    Ether => ether = 8,
    Quake => quake = 9,
    Torch => torch = 10,
    Hammer => hammer = 11,
    Flute => flute = 12,
    BugNet => bug_net = 13,
    Book => book = 14,
    CaneSomaria => cane_somaria = 16,
    CaneByrna => cane_byrna = 17,
    Cape => cape = 18,
    Mirror => mirror = 19,
    Gloves => gloves = 20,
    Boots => boots = 21,
    Flippers => flippers = 22,
    MoonPearl => moon_pearl = 23,
    Reserved => reserved_equipment = 24,
    Sword => sword_type = 25,
    Shield => shield_type = 26,
    Armor => armor = 27,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DungeonItem {
    Compass,
    BigKey,
    Map,
}

impl InventoryItemsState {
    /// Compatibility lookup for equipment only. The aggregate inventory reader
    /// routes resource indices to PlayerResourcesState.
    pub(crate) fn inventory_item(&self, index: usize) -> u8 {
        EquipmentItem::from_inventory_index(index).map_or(0, |item| self.equipment(item))
    }

    pub(crate) fn bow(&self) -> u8 {
        self.bow
    }

    pub(crate) fn has_silver_arrows(&self) -> bool {
        self.bow() & 4 != 0
    }

    pub(crate) fn has_upgraded_bow(&self) -> bool {
        self.bow() >= 3
    }

    pub(crate) fn boomerang(&self) -> u8 {
        self.boomerang
    }

    pub(crate) fn hookshot(&self) -> u8 {
        self.hookshot
    }

    pub(crate) fn mushroom(&self) -> u8 {
        self.mushroom
    }

    pub(crate) fn fire_rod(&self) -> u8 {
        self.fire_rod
    }

    pub(crate) fn ice_rod(&self) -> u8 {
        self.ice_rod
    }

    pub(crate) fn bombos(&self) -> u8 {
        self.bombos
    }

    pub(crate) fn ether(&self) -> u8 {
        self.ether
    }

    pub(crate) fn quake(&self) -> u8 {
        self.quake
    }

    pub(crate) fn torch(&self) -> u8 {
        self.torch
    }

    pub(crate) fn hammer(&self) -> u8 {
        self.hammer
    }

    pub(crate) fn flute(&self) -> u8 {
        self.flute
    }

    pub(crate) fn bug_net(&self) -> u8 {
        self.bug_net
    }

    pub(crate) fn book(&self) -> u8 {
        self.book
    }

    pub(crate) fn cane_somaria(&self) -> u8 {
        self.cane_somaria
    }

    pub(crate) fn cape(&self) -> u8 {
        self.cape
    }

    pub(crate) fn mirror(&self) -> u8 {
        self.mirror
    }

    pub(crate) fn gloves(&self) -> u8 {
        self.gloves
    }

    pub(crate) fn boots(&self) -> u8 {
        self.boots
    }

    pub(crate) fn has_boots(&self) -> bool {
        self.boots() != 0
    }

    pub(crate) fn has_flippers(&self) -> bool {
        self.flippers != 0
    }

    pub(crate) fn flippers(&self) -> u8 {
        self.flippers
    }

    pub(crate) fn moon_pearl(&self) -> u8 {
        self.moon_pearl
    }

    pub(crate) fn has_moon_pearl(&self) -> bool {
        self.moon_pearl() != 0
    }

    pub(crate) fn sword_type(&self) -> u8 {
        self.sword_type
    }

    pub(crate) fn shield_type(&self) -> u8 {
        self.shield_type
    }

    pub(crate) fn armor(&self) -> u8 {
        self.armor
    }

    pub(crate) fn bottle(&self, index: usize) -> u8 {
        self.bottles.get(index).copied().unwrap_or(0)
    }

    fn set_inventory_item(&mut self, index: usize, value: u8) {
        if let Some(item) = EquipmentItem::from_inventory_index(index) {
            self.set_equipment(item, value);
        }
    }

    fn set_bottle(&mut self, index: usize, value: u8) {
        if let Some(bottle) = self.bottles.get_mut(index) {
            *bottle = value;
        }
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
}

impl<'a> NativeInventoryItemsBridgeMut<'a> {
    pub(crate) fn new(items: &'a mut InventoryItemsState, _ram: &'a mut [u8]) -> Self {
        Self { items }
    }

    pub(crate) fn grant_equipment(&mut self, item: EquipmentItem, value: u8) {
        self.items.set_equipment(item, value);
    }

    pub(crate) fn grant_equipment_if_empty(&mut self, item: EquipmentItem, value: u8) {
        // Observe the current equipment at the source conditional.
        if self.items.equipment(item) == 0 {
            self.grant_equipment(item, value);
        }
    }

    pub(crate) fn set_inventory_item(&mut self, index: usize, value: u8) {
        self.items.set_inventory_item(index, value);
    }

    pub(crate) fn set_mushroom(&mut self, value: u8) {
        self.set_inventory_item(4, value);
    }

    #[cfg(test)]
    pub(crate) fn set_ice_rod(&mut self, value: u8) {
        self.set_inventory_item(6, value);
    }

    #[cfg(test)]
    pub(crate) fn set_bombos(&mut self, value: u8) {
        self.set_inventory_item(7, value);
    }

    #[cfg(test)]
    pub(crate) fn set_ether(&mut self, value: u8) {
        self.set_inventory_item(8, value);
    }

    pub(crate) fn set_flute(&mut self, value: u8) {
        self.set_inventory_item(12, value);
    }

    #[cfg(test)]
    pub(crate) fn set_mirror(&mut self, value: u8) {
        self.set_inventory_item(19, value);
    }

    #[cfg(test)]
    pub(crate) fn set_boots(&mut self, value: u8) {
        self.set_inventory_item(21, value);
    }

    #[cfg(test)]
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
    }

    pub(crate) fn fill_first_empty_bottle_with(&mut self, value: u8) -> bool {
        let filled = self.items.fill_first_empty_bottle_with(value);
        filled
    }

    pub(crate) fn replace_first_empty_bottle_with(&mut self, value: u8) -> bool {
        let replaced = self.items.replace_first_empty_bottle_with(value);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(MIRROR_WARP_TARGET_INDEX, self.target_index);
        ram.write_word(MIRROR_WARP_TARGET_OFFSETS, self.target_offsets[0]);
        ram.write_word(MIRROR_WARP_TARGET_OFFSETS + 2, self.target_offsets[1]);
        ram.write_word(MIRROR_WARP_VELOCITY_DELTAS, self.velocity_deltas[0]);
        ram.write_word(MIRROR_WARP_VELOCITY_DELTAS + 2, self.velocity_deltas[1]);
        ram.write_word(MIRROR_WARP_WAVE_OFFSET, self.wave_offset);
        ram.write_word(MIRROR_WARP_DISPLACEMENT, self.displacement);
        ram.write_word(MIRROR_WARP_SUBPIXEL, self.subpixel);
        ram.write_word(MIRROR_WARP_RESERVED, self.reserved);
        ram.write_word(MIRROR_WARP_WAVE_LENGTH, self.wave_length);
        ram.write_word(MIRROR_WARP_SPACING_A, self.spacing_a);
        ram.write_word(MIRROR_WARP_SPACING_B, self.spacing_b);
        ram.write_byte(MIRROR_WARP_LOAD_STEP_COUNTER, self.load_step_counter);
        ram.write_byte(MIRROR_WARP_ANIMATION_COUNTER, self.animation_counter);
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

adopting_bridge!(NativeMirrorWarpBridgeMut, mirror_warp: MirrorWarpState);

impl<'a> NativeMirrorWarpBridgeMut<'a> {
    forward_synced! {
        mirror_warp;
        fn initialize_hdma_wave_state();
        fn reset_wave_and_subpixel();
        fn toggle_target_index();
        fn set_displacement(value: u16);
        fn set_subpixel_low_from(value: u16);
        fn set_wave_offset(value: u16);
        fn shrink_target_offsets_for_dewaving();
        fn increment_load_step_counter() -> u8;
        fn reset_load_step_counter();
        fn set_animation_counter(value: u8);
        fn decrement_animation_counter() -> u8;
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        for (slot, keys) in self.keys_earned.iter().copied().enumerate() {
            ram.write_byte(LINK_KEYS_EARNED_PER_DUNGEON + slot, keys);
        }
    }

    pub(crate) fn keys_earned(&self, palace_index_x2: u8) -> u8 {
        self.keys_earned_slot(usize::from(palace_index_x2 >> 1))
    }

    pub(crate) fn keys_earned_slot(&self, slot: usize) -> u8 {
        self.keys_earned.get(slot).copied().unwrap_or(0)
    }
}

adopting_bridge!(NativeDungeonKeySlotsBridgeMut, state: DungeonKeySlotsState);

impl<'a> NativeDungeonKeySlotsBridgeMut<'a> {
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
    pub(crate) magic: PlayerMagicState,
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
    pendant_flags: u8,
    bomb_filler: u8,
    arrow_filler: u8,
    arrows: u8,
    ability_flags: u8,
    crystal_flags: u8,
    low_health_beep_timer: u8,
}

impl PlayerResourcesState {
    #[cfg(test)]
    pub(crate) fn export_owned(&self, image: &mut [u8]) {
        for address in [0x4ca, 0xf343, 0xf34f].into_iter().chain(0xf360..=0xf377).chain(0xf379..=0xf37b) {
            image[address] = self.owned_byte(address).unwrap();
        }
    }

    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            magic: PlayerMagicState::load_from_ram(ram),
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
            pendant_flags: ram_byte(ram, LINK_WHICH_PENDANTS),
            bomb_filler: ram_byte(ram, LINK_BOMB_FILLER),
            arrow_filler: ram_byte(ram, LINK_ARROW_REFILL_COUNTER),
            arrows: ram_byte(ram, LINK_NUM_ARROWS),
            ability_flags: ram_byte(ram, LINK_ABILITY_FLAGS),
            crystal_flags: ram_byte(ram, LINK_HAS_CRYSTALS),
            low_health_beep_timer: ram_byte(ram, LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP),
        }
    }

    pub(crate) fn owned_byte(&self, address: usize) -> Option<u8> {
        match address {
            LINK_ITEM_BOMBS => Some(self.bombs),
            LINK_ITEM_BOTTLE_INDEX => Some(self.equipped_bottle_index),
            a if (LINK_RUPEES_GOAL..LINK_RUPEES_GOAL + 2).contains(&a) => Some(self.rupees_goal.to_le_bytes()[a - LINK_RUPEES_GOAL]),
            a if (LINK_RUPEES_ACTUAL..LINK_RUPEES_ACTUAL + 2).contains(&a) => Some(self.rupees_actual.to_le_bytes()[a - LINK_RUPEES_ACTUAL]),
            a if (LINK_COMPASS..LINK_COMPASS + 2).contains(&a) => Some(self.compass_flags.to_le_bytes()[a - LINK_COMPASS]),
            a if (LINK_BIGKEY..LINK_BIGKEY + 2).contains(&a) => Some(self.big_key_flags.to_le_bytes()[a - LINK_BIGKEY]),
            a if (LINK_DUNGEON_MAP..LINK_DUNGEON_MAP + 2).contains(&a) => Some(self.dungeon_map_flags.to_le_bytes()[a - LINK_DUNGEON_MAP]),
            LINK_RUPEES_IN_POND => Some(self.rupees_in_pond),
            LINK_HEART_PIECES => Some(self.heart_pieces),
            LINK_HEALTH_CAPACITY => Some(self.health_capacity),
            LINK_CURRENT_HEALTH => Some(self.current_health),
            LINK_NUM_KEYS => Some(self.keys),
            LINK_BOMB_UPGRADES => Some(self.bomb_upgrade_level),
            LINK_ARROW_UPGRADES => Some(self.arrow_upgrade_level),
            LINK_HEARTS_FILLER => Some(self.heart_filler),
            LINK_WHICH_PENDANTS => Some(self.pendant_flags),
            LINK_BOMB_FILLER => Some(self.bomb_filler),
            LINK_ARROW_REFILL_COUNTER => Some(self.arrow_filler),
            LINK_NUM_ARROWS => Some(self.arrows),
            LINK_ABILITY_FLAGS => Some(self.ability_flags),
            LINK_HAS_CRYSTALS => Some(self.crystal_flags),
            LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP => Some(self.low_health_beep_timer),
            _ => self.magic.owned_byte(address),
        }
    }

    pub(crate) fn write_owned_byte(&mut self, address: usize, value: u8) -> bool {
        match address {
            LINK_ITEM_BOMBS => self.bombs = value,
            LINK_ITEM_BOTTLE_INDEX => self.equipped_bottle_index = value,
            a if (LINK_RUPEES_GOAL..LINK_RUPEES_GOAL + 2).contains(&a) => {
                let mut bytes = self.rupees_goal.to_le_bytes();
                bytes[a - LINK_RUPEES_GOAL] = value;
                self.rupees_goal = u16::from_le_bytes(bytes);
            }
            a if (LINK_RUPEES_ACTUAL..LINK_RUPEES_ACTUAL + 2).contains(&a) => {
                let mut bytes = self.rupees_actual.to_le_bytes();
                bytes[a - LINK_RUPEES_ACTUAL] = value;
                self.rupees_actual = u16::from_le_bytes(bytes);
            }
            a if (LINK_COMPASS..LINK_COMPASS + 2).contains(&a) => {
                let mut bytes = self.compass_flags.to_le_bytes();
                bytes[a - LINK_COMPASS] = value;
                self.compass_flags = u16::from_le_bytes(bytes);
            }
            a if (LINK_BIGKEY..LINK_BIGKEY + 2).contains(&a) => {
                let mut bytes = self.big_key_flags.to_le_bytes();
                bytes[a - LINK_BIGKEY] = value;
                self.big_key_flags = u16::from_le_bytes(bytes);
            }
            a if (LINK_DUNGEON_MAP..LINK_DUNGEON_MAP + 2).contains(&a) => {
                let mut bytes = self.dungeon_map_flags.to_le_bytes();
                bytes[a - LINK_DUNGEON_MAP] = value;
                self.dungeon_map_flags = u16::from_le_bytes(bytes);
            }
            LINK_RUPEES_IN_POND => self.rupees_in_pond = value,
            LINK_HEART_PIECES => self.heart_pieces = value,
            LINK_HEALTH_CAPACITY => self.health_capacity = value,
            LINK_CURRENT_HEALTH => self.current_health = value,
            LINK_NUM_KEYS => self.keys = value,
            LINK_BOMB_UPGRADES => self.bomb_upgrade_level = value,
            LINK_ARROW_UPGRADES => self.arrow_upgrade_level = value,
            LINK_HEARTS_FILLER => self.heart_filler = value,
            LINK_WHICH_PENDANTS => self.pendant_flags = value,
            LINK_BOMB_FILLER => self.bomb_filler = value,
            LINK_ARROW_REFILL_COUNTER => self.arrow_filler = value,
            LINK_NUM_ARROWS => self.arrows = value,
            LINK_ABILITY_FLAGS => self.ability_flags = value,
            LINK_HAS_CRYSTALS => self.crystal_flags = value,
            LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP => self.low_health_beep_timer = value,
            _ => return self.magic.write_owned_byte(address, value),
        }
        true
    }

    pub(crate) fn magic_filler(&self) -> u8 {
        self.magic.refill()
    }

    pub(crate) fn magic_consumption_level(&self) -> u8 {
        self.magic.consumption_level()
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
}

impl<'a> NativePlayerResourcesBridgeMut<'a> {
    pub(crate) fn new(resources: &'a mut PlayerResourcesState, _ram: &'a mut [u8]) -> Self {
        Self { resources }
    }

    pub(crate) fn grant_dungeon_item(&mut self, item: DungeonItem, mask: u16) {
        match item {
            DungeonItem::Compass => self.resources.compass_flags |= mask,
            DungeonItem::BigKey => self.resources.big_key_flags |= mask,
            DungeonItem::Map => self.resources.dungeon_map_flags |= mask,
        }
    }

    pub(crate) fn set_magic_consumption_level(&mut self, value: u8) {
        self.resources.magic.set_consumption_level(value);
    }

    pub(crate) fn set_magic_filler(&mut self, value: u8) {
        self.resources.magic.set_refill(value);
    }

    #[cfg(feature = "rom-timing")]
    pub(crate) fn clear_magic_filler(&mut self) {
        self.resources.magic.set_refill(0);
    }

    #[cfg(feature = "rom-timing")]
    pub(crate) fn decrement_magic_filler(&mut self) {
        self.resources.magic.decrement_refill();
    }

    pub(crate) fn decrement_bomb_filler(&mut self) {
        self.resources.bomb_filler = self.resources.bomb_filler.wrapping_sub(1);
    }

    pub(crate) fn set_bomb_filler(&mut self, value: u8) {
        self.resources.bomb_filler = value;
    }

    pub(crate) fn increment_bomb_filler_by(&mut self, value: u8) {
        self.resources.bomb_filler = self.resources.bomb_filler.wrapping_add(value);
    }

    pub(crate) fn set_bombs(&mut self, value: u8) {
        self.resources.bombs = value;
    }

    pub(crate) fn increment_bombs(&mut self) {
        self.resources.bombs = self.resources.bombs.wrapping_add(1);
    }

    pub(crate) fn decrement_bombs(&mut self) -> u8 {
        self.resources.bombs = self.resources.bombs.wrapping_sub(1);
        let value = self.resources.bombs;
        value
    }

    pub(crate) fn increment_health_capacity_by(&mut self, value: u8) -> u8 {
        self.resources.health_capacity = self.resources.health_capacity.wrapping_add(value);
        let capacity = self.resources.health_capacity;
        capacity
    }

    pub(crate) fn increment_heart_filler_by(&mut self, value: u8) {
        self.resources.heart_filler = self.resources.heart_filler.wrapping_add(value);
    }

    pub(crate) fn increment_heart_filler_word_by(&mut self, value: u16) -> u16 {
        let hearts = self.resources.heart_filler_word().wrapping_add(value);
        self.resources.heart_filler = hearts as u8;
        self.resources.magic.set_refill((hearts >> 8) as u8);
        hearts
    }

    pub(crate) fn increment_magic_filler_by(&mut self, value: u8) {
        self.resources.magic.add_refill(value);
    }

    pub(crate) fn add_ability_flags(&mut self, flags: u8) {
        self.resources.ability_flags |= flags;
    }

    pub(crate) fn add_crystal_flags(&mut self, flags: u8) {
        self.resources.crystal_flags |= flags;
    }

    #[cfg(test)]
    pub(crate) fn set_crystal_flags(&mut self, flags: u8) {
        self.resources.crystal_flags = flags;
    }

    pub(crate) fn set_pendant_flags(&mut self, flags: u8) {
        self.resources.pendant_flags = flags;
    }

    pub(crate) fn decrement_arrow_filler(&mut self) {
        self.resources.arrow_filler = self.resources.arrow_filler.wrapping_sub(1);
    }

    pub(crate) fn set_arrow_filler(&mut self, value: u8) {
        self.resources.arrow_filler = value;
    }

    pub(crate) fn increment_arrow_filler_by(&mut self, value: u8) {
        self.resources.arrow_filler = self.resources.arrow_filler.wrapping_add(value);
    }

    pub(crate) fn increment_arrows(&mut self) {
        self.resources.arrows = self.resources.arrows.wrapping_add(1);
    }

    pub(crate) fn increment_arrows_by(&mut self, value: u8) {
        self.resources.arrows = self.resources.arrows.wrapping_add(value);
    }

    pub(crate) fn set_arrows(&mut self, value: u8) {
        self.resources.arrows = value;
    }

    pub(crate) fn decrement_arrows(&mut self) -> u8 {
        self.resources.arrows = self.resources.arrows.wrapping_sub(1);
        let value = self.resources.arrows;
        value
    }

    pub(crate) fn set_current_health(&mut self, value: u8) {
        self.resources.current_health = value;
    }

    pub(crate) fn increment_current_health_by(&mut self, value: u8) {
        self.resources.current_health = self.resources.current_health.wrapping_add(value);
    }

    pub(crate) fn decrement_current_health_by(&mut self, value: u8) -> u8 {
        self.resources.current_health = self.resources.current_health.wrapping_sub(value);
        let health = self.resources.current_health;
        health
    }

    pub(crate) fn set_heart_filler(&mut self, value: u8) {
        self.resources.heart_filler = value;
    }

    pub(crate) fn decrement_heart_filler_by(&mut self, value: u8) {
        self.resources.heart_filler = self.resources.heart_filler.wrapping_sub(value);
    }

    pub(crate) fn decrement_low_health_beep_timer(&mut self) {
        self.resources.low_health_beep_timer = self.resources.low_health_beep_timer.wrapping_sub(1);
    }

    pub(crate) fn set_low_health_beep_timer(&mut self, value: u8) {
        self.resources.low_health_beep_timer = value;
    }

    pub(crate) fn set_equipped_bottle_index(&mut self, value: u8) {
        self.resources.equipped_bottle_index = value;
    }

    pub(crate) fn set_rupees_goal(&mut self, value: u16) {
        self.resources.rupees_goal = value;
    }

    pub(crate) fn set_rupees_actual(&mut self, value: u16) {
        self.resources.rupees_actual = value;
    }

    #[cfg(test)]
    pub(crate) fn add_rupees_goal(&mut self, value: u16) -> u16 {
        self.resources.rupees_goal = self.resources.rupees_goal.wrapping_add(value);
        let rupees = self.resources.rupees_goal;
        rupees
    }

    pub(crate) fn subtract_rupees_goal(&mut self, value: u16) -> u16 {
        self.resources.rupees_goal = self.resources.rupees_goal.wrapping_sub(value);
        let rupees = self.resources.rupees_goal;
        rupees
    }

    pub(crate) fn set_keys(&mut self, value: u8) {
        self.resources.keys = value;
    }

    pub(crate) fn increment_keys(&mut self) -> u8 {
        self.resources.keys = self.resources.keys.wrapping_add(1);
        let keys = self.resources.keys;
        keys
    }

    pub(crate) fn decrement_keys(&mut self) -> u8 {
        self.resources.keys = self.resources.keys.wrapping_sub(1);
        let keys = self.resources.keys;
        keys
    }

    pub(crate) fn advance_heart_piece_count(&mut self) -> u8 {
        self.resources.heart_pieces = self.resources.heart_pieces.wrapping_add(1) & 3;
        let heart_pieces = self.resources.heart_pieces;
        heart_pieces
    }

    pub(crate) fn add_rupees_to_pond(&mut self, value: u8) -> u8 {
        self.resources.rupees_in_pond = self.resources.rupees_in_pond.wrapping_add(value);
        let pond = self.resources.rupees_in_pond;
        pond
    }

    pub(crate) fn subtract_pond_reward_threshold(&mut self) -> u8 {
        self.resources.rupees_in_pond = self.resources.rupees_in_pond.wrapping_sub(100);
        let pond = self.resources.rupees_in_pond;
        pond
    }

    pub(crate) fn set_bomb_upgrade_level(&mut self, value: u8) {
        self.resources.bomb_upgrade_level = value;
    }

    pub(crate) fn set_arrow_upgrade_level(&mut self, value: u8) {
        self.resources.arrow_upgrade_level = value;
    }
}

impl PlayerResourcesState {
    fn heart_filler_word(&self) -> u16 {
        u16::from(self.heart_filler) | (u16::from(self.magic.refill()) << 8)
    }
}

fn read_word(ram: &[u8], offset: usize) -> u16 {
    if offset + 1 < ram.len() {
        read_le_u16(ram, offset)
    } else {
        0
    }
}

// State-v2 audit: cartridge fields (including targeted/mode-gated writes)
// are covered by wram_image; recurse only to classify native children.
non_wram_hash! { InventoryState {
    wram: [],
    children: [dungeon_key_slots, player_resources, mirror_warp, save_progress, items],
    metadata: [],
} }
non_wram_hash! { MirrorWarpState {
    wram: [
        target_index, target_offsets, velocity_deltas, wave_offset, displacement, subpixel,
        reserved, wave_length, spacing_a, spacing_b, load_step_counter, animation_counter,
    ],
    children: [],
    metadata: [],
} }
non_wram_hash! { PlayerResourcesState {
    wram: [
        bombs, equipped_bottle_index, rupees_goal, rupees_actual, compass_flags, big_key_flags,
        dungeon_map_flags, rupees_in_pond, heart_pieces, health_capacity, current_health, keys,
        bomb_upgrade_level, arrow_upgrade_level, heart_filler, pendant_flags, bomb_filler,
        arrow_filler, arrows, ability_flags, crystal_flags, low_health_beep_timer,
    ],
    children: [magic],
    metadata: [],
} }
non_wram_hash! { DungeonKeySlotsState {
    wram: [keys_earned],
    children: [],
    metadata: [],
} }

impl InventoryState {
    pub(crate) fn owned_byte(&self, address: usize) -> Option<u8> {
        self.player_resources.owned_byte(address).or_else(|| self.items.owned_byte(address))
            .or_else(|| self.save_progress.owned_byte(address))
    }

    pub(crate) fn write_owned_byte(&mut self, address: usize, value: u8) -> bool {
        self.player_resources.write_owned_byte(address, value) || self.items.write_owned_byte(address, value)
            || self.save_progress.write_owned_byte(address, value)
    }

    pub(crate) fn export_owned_range(&self, image: &mut [u8], lo: usize, hi: usize) {
        for address in lo..=hi { image[address] = self.owned_byte(address).expect("registered inventory byte"); }
    }

    pub(crate) fn import_owned_range(&mut self, image: &[u8], lo: usize, hi: usize) {
        for address in lo..=hi { assert!(self.write_owned_byte(address, image[address])); }
    }
}
