use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const DUNGEON_KEY_SLOT_COUNT: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct InventoryState {
    pub(crate) dungeon_key_slots: DungeonKeySlotsState,
    pub(crate) player_resources: PlayerResourcesState,
}

impl InventoryState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dungeon_key_slots: DungeonKeySlotsState::load_from_ram(ram),
            player_resources: PlayerResourcesState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.dungeon_key_slots.write_to_ram(ram);
        self.player_resources.write_to_ram(ram);
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

pub(crate) struct DungeonKeySlotsView<'a> {
    state: &'a DungeonKeySlotsState,
}

impl<'a> DungeonKeySlotsView<'a> {
    pub(crate) fn new(state: &'a DungeonKeySlotsState) -> Self {
        Self { state }
    }

    pub(crate) fn keys_earned(&self, palace_index_x2: u8) -> u8 {
        self.state.keys_earned(palace_index_x2)
    }

    pub(crate) fn keys_earned_slot(&self, slot: usize) -> u8 {
        self.state.keys_earned_slot(slot)
    }
}

pub(crate) struct NativeDungeonKeySlotsBridgeMut<'a> {
    state: &'a mut DungeonKeySlotsState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonKeySlotsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonKeySlotsState, ram: &'a mut [u8]) -> Self {
        *state = DungeonKeySlotsState::load_from_ram(ram);
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
    magic_power: u8,
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
            magic_power: ram_byte(ram, LINK_MAGIC_POWER),
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
        ram[LINK_MAGIC_POWER] = self.magic_power;
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

    pub(crate) fn magic_power(&self) -> u8 {
        self.magic_power
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
        *resources = PlayerResourcesState::load_from_ram(ram);
        Self { resources, ram }
    }

    fn sync(&mut self) {
        self.resources.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.resources,
            PlayerResourcesState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_magic_power(&mut self, value: u8) {
        self.resources.magic_power = value;
        self.sync();
    }

    pub(crate) fn set_magic_consumption_level(&mut self, value: u8) {
        self.resources.magic_consumption_level = value;
        self.sync();
    }

    pub(crate) fn increment_magic_power(&mut self) {
        self.resources.magic_power = self.resources.magic_power.wrapping_add(1);
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
