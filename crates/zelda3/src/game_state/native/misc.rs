use super::ram_byte;
use crate::game_state::constants::*;

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
