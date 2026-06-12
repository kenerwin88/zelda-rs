use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const MEMORIZED_TILE_ENTRY_SLOTS: usize = 0x80;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MemorizedTileState {
    count: u16,
    addresses: Vec<u16>,
    values: Vec<u16>,
}

impl MemorizedTileState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut addresses = vec![0; MEMORIZED_TILE_ENTRY_SLOTS];
        let mut values = vec![0; MEMORIZED_TILE_ENTRY_SLOTS];
        for slot in 0..MEMORIZED_TILE_ENTRY_SLOTS {
            addresses[slot] = read_le_u16(ram, MEMORIZED_TILE_ADDR + slot * 2);
            values[slot] = read_le_u16(ram, MEMORIZED_TILE_VALUE + slot * 2);
        }
        Self {
            count: read_le_u16(ram, NUM_MEMORIZED_TILES),
            addresses,
            values,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, NUM_MEMORIZED_TILES, self.count);
        for slot in 0..MEMORIZED_TILE_ENTRY_SLOTS {
            write_le_u16(ram, MEMORIZED_TILE_ADDR + slot * 2, self.addresses[slot]);
            write_le_u16(ram, MEMORIZED_TILE_VALUE + slot * 2, self.values[slot]);
        }
    }

    pub(crate) fn count(&self) -> u16 {
        self.count
    }

    pub(crate) fn entry_addr(&self, index: usize) -> u16 {
        self.addresses.get(index).copied().unwrap_or_default()
    }

    pub(crate) fn entry_value(&self, index: usize) -> u16 {
        self.values.get(index).copied().unwrap_or_default()
    }

    pub(crate) fn set_count(&mut self, value: u16) {
        self.count = value;
    }

    pub(crate) fn clear_count(&mut self) {
        self.count = 0;
    }

    pub(crate) fn set_entry_addr(&mut self, byte_offset: usize, pos: u16) {
        self.addresses[byte_offset / 2] = pos;
    }

    pub(crate) fn set_entry_value(&mut self, byte_offset: usize, tile: u16) {
        self.values[byte_offset / 2] = tile;
    }

    pub(crate) fn append_entry(&mut self, pos: u16, tile: u16) {
        let byte_offset = usize::from(self.count);
        self.set_entry_value(byte_offset, tile);
        self.set_entry_addr(byte_offset, pos);
        self.set_count(byte_offset as u16 + 2);
    }

    pub(crate) fn clear_entry_addresses(&mut self) {
        self.addresses.fill(0);
    }
}

impl Default for MemorizedTileState {
    fn default() -> Self {
        Self {
            count: 0,
            addresses: vec![0; MEMORIZED_TILE_ENTRY_SLOTS],
            values: vec![0; MEMORIZED_TILE_ENTRY_SLOTS],
        }
    }
}

pub(crate) struct NativeMemorizedTileBridgeMut<'a> {
    memorized_tiles: &'a mut MemorizedTileState,
    ram: &'a mut [u8],
}

impl<'a> NativeMemorizedTileBridgeMut<'a> {
    pub(crate) fn new(memorized_tiles: &'a mut MemorizedTileState, ram: &'a mut [u8]) -> Self {
        *memorized_tiles = MemorizedTileState::load_from_ram(ram);
        Self {
            memorized_tiles,
            ram,
        }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.memorized_tiles,
            MemorizedTileState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_count(&mut self, value: u16) {
        self.memorized_tiles.set_count(value);
        write_le_u16(self.ram, NUM_MEMORIZED_TILES, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_count(&mut self) {
        self.memorized_tiles.clear_count();
        write_le_u16(self.ram, NUM_MEMORIZED_TILES, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_entry_addr(&mut self, byte_offset: usize, pos: u16) {
        self.memorized_tiles.set_entry_addr(byte_offset, pos);
        write_le_u16(self.ram, MEMORIZED_TILE_ADDR + byte_offset, pos);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_entry_value(&mut self, byte_offset: usize, tile: u16) {
        self.memorized_tiles.set_entry_value(byte_offset, tile);
        write_le_u16(self.ram, MEMORIZED_TILE_VALUE + byte_offset, tile);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn append_entry(&mut self, pos: u16, tile: u16) {
        let byte_offset = usize::from(self.memorized_tiles.count());
        self.memorized_tiles.append_entry(pos, tile);
        write_le_u16(self.ram, MEMORIZED_TILE_VALUE + byte_offset, tile);
        write_le_u16(self.ram, MEMORIZED_TILE_ADDR + byte_offset, pos);
        write_le_u16(self.ram, NUM_MEMORIZED_TILES, byte_offset as u16 + 2);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_entry_addresses(&mut self) {
        self.memorized_tiles.clear_entry_addresses();
        self.ram[MEMORIZED_TILE_ADDR..MEMORIZED_TILE_ADDR + 0x100].fill(0);
        self.debug_assert_matches_ram();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonSecretState {
    pending_kind: u8,
    pending_kind_high: u8,
    overworld_substitution_counter: u8,
}

impl DungeonSecretState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            pending_kind: ram_byte(ram, DUNGEON_SECRET_PENDING_KIND),
            pending_kind_high: ram_byte(ram, DUNGEON_SECRET_PENDING_KIND + 1),
            overworld_substitution_counter: ram_byte(ram, OVERWORLD_SECRET_SUBST_CTR),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNGEON_SECRET_PENDING_KIND] = self.pending_kind;
        ram[DUNGEON_SECRET_PENDING_KIND + 1] = self.pending_kind_high;
        ram[OVERWORLD_SECRET_SUBST_CTR] = self.overworld_substitution_counter;
    }

    pub(crate) fn pending_kind(&self) -> u8 {
        self.pending_kind
    }

    pub(crate) fn overworld_subst_counter(&self) -> u8 {
        self.overworld_substitution_counter
    }

    pub(crate) fn has_pending_kind(&self) -> bool {
        self.pending_kind() != 0
    }

    pub(crate) fn is_available(&self) -> bool {
        self.pending_kind() != 0xff
    }

    pub(crate) fn graphics_kind(&self) -> Option<u8> {
        if self.pending_kind & 0x80 != 0 {
            Some(self.pending_kind & 0x7f)
        } else {
            None
        }
    }

    pub(crate) fn clear_pending_kind(&mut self) {
        self.pending_kind = 0;
    }

    pub(crate) fn set_pending_kind(&mut self, value: u8) {
        self.pending_kind = value;
    }

    pub(crate) fn increment_overworld_subst_counter(&mut self) {
        self.overworld_substitution_counter = self.overworld_substitution_counter.wrapping_add(1);
    }

    pub(crate) fn set_powder_pending_kind(&mut self) {
        self.pending_kind = 4;
        self.pending_kind_high = 0;
    }

    pub(crate) fn or_pending_kind(&mut self, value: u8) {
        self.pending_kind |= value;
    }

    pub(crate) fn mark_graphics_kind(&mut self) {
        self.pending_kind |= 0x80;
    }
}

pub(crate) struct NativeDungeonSecretBridgeMut<'a> {
    dungeon_secret: &'a mut DungeonSecretState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonSecretBridgeMut<'a> {
    pub(crate) fn new(dungeon_secret: &'a mut DungeonSecretState, ram: &'a mut [u8]) -> Self {
        *dungeon_secret = DungeonSecretState::load_from_ram(ram);
        Self {
            dungeon_secret,
            ram,
        }
    }

    fn sync(&mut self) {
        self.dungeon_secret.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.dungeon_secret,
            DungeonSecretState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_pending_kind(&mut self) {
        self.dungeon_secret.clear_pending_kind();
        self.sync();
    }

    pub(crate) fn set_pending_kind(&mut self, value: u8) {
        self.dungeon_secret.set_pending_kind(value);
        self.sync();
    }

    pub(crate) fn increment_overworld_subst_counter(&mut self) {
        self.dungeon_secret.increment_overworld_subst_counter();
        self.sync();
    }

    pub(crate) fn set_powder_pending_kind(&mut self) {
        self.dungeon_secret.set_powder_pending_kind();
        self.sync();
    }

    pub(crate) fn or_pending_kind(&mut self, value: u8) {
        self.dungeon_secret.or_pending_kind(value);
        self.sync();
    }

    pub(crate) fn mark_graphics_kind(&mut self) {
        self.dungeon_secret.mark_graphics_kind();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SaveLoadTransferState {
    source_offset: u16,
}

impl SaveLoadTransferState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            source_offset: read_le_u16(ram, SAVE_LOAD_SOURCE_OFFSET),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, SAVE_LOAD_SOURCE_OFFSET, self.source_offset);
    }

    pub(crate) fn source_offset(&self) -> u16 {
        self.source_offset
    }

    pub(crate) fn source_offset_usize(&self) -> usize {
        usize::from(self.source_offset)
    }

    pub(crate) fn set_source_offset(&mut self, value: u16) {
        self.source_offset = value;
    }
}

pub(crate) struct NativeSaveLoadTransferBridgeMut<'a> {
    transfer: &'a mut SaveLoadTransferState,
    ram: &'a mut [u8],
}

impl<'a> NativeSaveLoadTransferBridgeMut<'a> {
    pub(crate) fn new(transfer: &'a mut SaveLoadTransferState, ram: &'a mut [u8]) -> Self {
        *transfer = SaveLoadTransferState::load_from_ram(ram);
        Self { transfer, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.transfer,
            SaveLoadTransferState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_source_offset(&mut self, value: u16) {
        self.transfer.set_source_offset(value);
        write_le_u16(self.ram, SAVE_LOAD_SOURCE_OFFSET, value);
        self.debug_assert_matches_ram();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ArcheryGameState {
    hit_counter: u8,
    arrows_left: u8,
    out_of_arrows: u8,
}

impl ArcheryGameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            hit_counter: ram_byte(ram, ARCHERY_GAME_HIT_COUNTER),
            arrows_left: ram_byte(ram, ARCHERY_GAME_ARROWS_LEFT),
            out_of_arrows: ram_byte(ram, ARCHERY_GAME_OUT_OF_ARROWS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[ARCHERY_GAME_HIT_COUNTER] = self.hit_counter;
        ram[ARCHERY_GAME_ARROWS_LEFT] = self.arrows_left;
        ram[ARCHERY_GAME_OUT_OF_ARROWS] = self.out_of_arrows;
    }

    pub(crate) fn hit_counter(&self) -> u8 {
        self.hit_counter
    }

    pub(crate) fn arrows_left(&self) -> u8 {
        self.arrows_left
    }

    pub(crate) fn out_of_arrows(&self) -> u8 {
        self.out_of_arrows
    }

    pub(crate) fn clear_hit_counter(&mut self) {
        self.hit_counter = 0;
    }

    pub(crate) fn increment_hit_counter(&mut self) {
        self.hit_counter = self.hit_counter.wrapping_add(1);
    }

    pub(crate) fn set_arrows_left(&mut self, value: u8) {
        self.arrows_left = value;
    }

    pub(crate) fn decrement_arrows_left(&mut self) {
        self.arrows_left = self.arrows_left.wrapping_sub(1);
    }

    pub(crate) fn increment_out_of_arrows(&mut self) {
        self.out_of_arrows = self.out_of_arrows.wrapping_add(1);
    }

    pub(crate) fn clear_out_of_arrows(&mut self) {
        self.out_of_arrows = 0;
    }
}

pub(crate) struct NativeArcheryGameBridgeMut<'a> {
    archery_game: &'a mut ArcheryGameState,
    ram: &'a mut [u8],
}

impl<'a> NativeArcheryGameBridgeMut<'a> {
    pub(crate) fn new(archery_game: &'a mut ArcheryGameState, ram: &'a mut [u8]) -> Self {
        *archery_game = ArcheryGameState::load_from_ram(ram);
        Self { archery_game, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.archery_game,
            ArcheryGameState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_hit_counter(&mut self) {
        self.archery_game.clear_hit_counter();
        self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_hit_counter(&mut self) {
        self.archery_game.increment_hit_counter();
        self.ram[ARCHERY_GAME_HIT_COUNTER] = self.ram[ARCHERY_GAME_HIT_COUNTER].wrapping_add(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_arrows_left(&mut self, value: u8) {
        self.archery_game.set_arrows_left(value);
        self.ram[ARCHERY_GAME_ARROWS_LEFT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_arrows_left(&mut self) {
        self.archery_game.decrement_arrows_left();
        self.ram[ARCHERY_GAME_ARROWS_LEFT] = self.ram[ARCHERY_GAME_ARROWS_LEFT].wrapping_sub(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_out_of_arrows(&mut self) {
        self.archery_game.increment_out_of_arrows();
        self.ram[ARCHERY_GAME_OUT_OF_ARROWS] = self.ram[ARCHERY_GAME_OUT_OF_ARROWS].wrapping_add(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_out_of_arrows(&mut self) {
        self.archery_game.clear_out_of_arrows();
        self.ram[ARCHERY_GAME_OUT_OF_ARROWS] = 0;
        self.debug_assert_matches_ram();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteBattleState {
    sprites_killed: u8,
    times_hurt_by_sprites: u8,
    item_drop_luck: u8,
    luck_kill_counter: u8,
    item_drop_counter: u8,
    damage_type_determiner: u8,
    damaging_enemies_timer: u8,
}

impl SpriteBattleState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            sprites_killed: ram_byte(ram, NUM_SPRITES_KILLED),
            times_hurt_by_sprites: ram_byte(ram, TIMES_HURT_BY_SPRITES),
            item_drop_luck: ram_byte(ram, ITEM_DROP_LUCK),
            luck_kill_counter: ram_byte(ram, LUCK_KILL_COUNTER),
            item_drop_counter: ram_byte(ram, ITEM_DROP_COUNTER),
            damage_type_determiner: ram_byte(ram, DAMAGE_TYPE_DETERMINER),
            damaging_enemies_timer: ram_byte(ram, SET_WHEN_DAMAGING_ENEMIES),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[NUM_SPRITES_KILLED] = self.sprites_killed;
        ram[TIMES_HURT_BY_SPRITES] = self.times_hurt_by_sprites;
        ram[ITEM_DROP_LUCK] = self.item_drop_luck;
        ram[LUCK_KILL_COUNTER] = self.luck_kill_counter;
        ram[ITEM_DROP_COUNTER] = self.item_drop_counter;
        ram[DAMAGE_TYPE_DETERMINER] = self.damage_type_determiner;
        ram[SET_WHEN_DAMAGING_ENEMIES] = self.damaging_enemies_timer;
    }

    pub(crate) fn sprites_killed(&self) -> u8 {
        self.sprites_killed
    }

    pub(crate) fn times_hurt_by_sprites(&self) -> u8 {
        self.times_hurt_by_sprites
    }

    pub(crate) fn item_drop_luck(&self) -> u8 {
        self.item_drop_luck
    }

    pub(crate) fn luck_kill_counter(&self) -> u8 {
        self.luck_kill_counter
    }

    pub(crate) fn item_drop_counter(&self) -> u8 {
        self.item_drop_counter
    }

    pub(crate) fn damage_type_determiner(&self) -> u8 {
        self.damage_type_determiner
    }

    pub(crate) fn damaging_enemies_timer(&self) -> u8 {
        self.damaging_enemies_timer
    }
}

pub(crate) struct NativeSpriteBattleBridgeMut<'a> {
    sprite_battle: &'a mut SpriteBattleState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteBattleBridgeMut<'a> {
    pub(crate) fn new(sprite_battle: &'a mut SpriteBattleState, ram: &'a mut [u8]) -> Self {
        *sprite_battle = SpriteBattleState::load_from_ram(ram);
        Self { sprite_battle, ram }
    }

    fn sync(&mut self) {
        self.sprite_battle.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.sprite_battle,
            SpriteBattleState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_sprites_killed(&mut self) {
        self.sprite_battle.sprites_killed = 0;
        self.sync();
    }

    pub(crate) fn clear_times_hurt_by_sprites(&mut self) {
        self.sprite_battle.times_hurt_by_sprites = 0;
        self.sync();
    }

    pub(crate) fn increment_times_hurt_by_sprites(&mut self) {
        self.sprite_battle.times_hurt_by_sprites =
            self.sprite_battle.times_hurt_by_sprites.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn set_item_drop_luck(&mut self, value: u8) {
        self.sprite_battle.item_drop_luck = value;
        self.sync();
    }

    pub(crate) fn clear_luck_kill_counter(&mut self) {
        self.sprite_battle.luck_kill_counter = 0;
        self.sync();
    }

    pub(crate) fn clear_item_drop_counter(&mut self) {
        self.sprite_battle.item_drop_counter = 0;
        self.sync();
    }

    pub(crate) fn increment_sprites_killed(&mut self) {
        self.sprite_battle.sprites_killed = self.sprite_battle.sprites_killed.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn increment_luck_kill_counter(&mut self) {
        self.sprite_battle.luck_kill_counter = self.sprite_battle.luck_kill_counter.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn set_damage_type_determiner(&mut self, value: u8) {
        self.sprite_battle.damage_type_determiner = value;
        self.sync();
    }

    pub(crate) fn set_damaging_enemies_timer(&mut self, value: u8) {
        self.sprite_battle.damaging_enemies_timer = value;
        self.sync();
    }

    pub(crate) fn clear_damaging_enemies_timer(&mut self) {
        self.sprite_battle.damaging_enemies_timer = 0;
        self.sync();
    }

    pub(crate) fn tick_damaging_enemies_timer(&mut self) {
        self.sprite_battle.damaging_enemies_timer =
            if self.sprite_battle.damaging_enemies_timer & 0x7f != 0 {
                self.sprite_battle.damaging_enemies_timer.wrapping_sub(1)
            } else {
                0
            };
        self.sync();
    }

    pub(crate) fn increment_item_drop_counter(&mut self) -> u8 {
        self.sprite_battle.item_drop_counter = self.sprite_battle.item_drop_counter.wrapping_add(1);
        let value = self.sprite_battle.item_drop_counter;
        self.sync();
        value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EnhancedFeaturesState {
    bits: u32,
}

impl EnhancedFeaturesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            bits: u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS))
                | (u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS + 1)) << 8)
                | (u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS + 2)) << 16)
                | (u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS + 3)) << 24),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        let bytes = self.bits.to_le_bytes();
        ram[ENHANCED_FEATURE_FLAGS..ENHANCED_FEATURE_FLAGS + 4].copy_from_slice(&bytes);
    }

    pub(crate) fn bits(&self) -> u32 {
        self.bits
    }

    pub(crate) fn has(&self, mask: u32) -> bool {
        self.bits & mask != 0
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

pub(crate) struct NativeEnhancedFeaturesBridgeMut<'a> {
    enhanced_features: &'a mut EnhancedFeaturesState,
    ram: &'a mut [u8],
}

impl<'a> NativeEnhancedFeaturesBridgeMut<'a> {
    pub(crate) fn new(enhanced_features: &'a mut EnhancedFeaturesState, ram: &'a mut [u8]) -> Self {
        *enhanced_features = EnhancedFeaturesState::load_from_ram(ram);
        Self {
            enhanced_features,
            ram,
        }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.enhanced_features,
            EnhancedFeaturesState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_bits(&mut self, value: u32) {
        self.enhanced_features.bits = value;
        self.enhanced_features.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }
}
