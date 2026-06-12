use super::ram_byte;
use crate::game_state::constants::*;

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
