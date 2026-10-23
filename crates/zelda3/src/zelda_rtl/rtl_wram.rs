//! WRAM is a wire image. Only the ranges in this registry may be exported
//! from native owners; projecting all of GameState would revive stale mirrors
//! and mode-reused-byte clobbers. Add an owner only after its poison proof.
use super::ZeldaState;
use std::borrow::Cow;

/// One exclusive native range, with inclusive cartridge addresses.
/// The codec must touch only this registered range in the whole image.
/// An owner's import callback must load only that owner's registered fields.
pub struct NativeOwnedWramRange {
    pub lo: usize,
    pub hi: usize,
    pub owner: &'static str,
    export_owned: fn(&ZeldaState, &mut [u8]),
    import_owned: fn(&mut ZeldaState, &[u8]),
}

/// Sorted, disjoint ranges whose bytes no longer live in ZeldaState::ram.
pub const NATIVE_OWNED_WRAM: &[NativeOwnedWramRange] = &[
    NativeOwnedWramRange { lo: 0x00202, hi: 0x00204, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x00202, 0x00204),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x00202, 0x00204) },
    NativeOwnedWramRange { lo: 0x0040c, hi: 0x0040c, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0040c, 0x0040c),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0040c, 0x0040c) },
    NativeOwnedWramRange { lo: 0x004ca, hi: 0x004ca, owner: "PlayerResourcesState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x004ca, 0x004ca),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x004ca, 0x004ca) },
    NativeOwnedWramRange { lo: 0x00656, hi: 0x00658, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x00656, 0x00658),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x00656, 0x00658) },
    NativeOwnedWramRange { lo: 0x0f000, hi: 0x0f27f, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0f000, 0x0f27f),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0f000, 0x0f27f) },
    NativeOwnedWramRange { lo: 0x0f340, hi: 0x0f377, owner: "InventoryItemsState + PlayerResourcesState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0f340, 0x0f377),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0f340, 0x0f377) },
    NativeOwnedWramRange { lo: 0x0f379, hi: 0x0f37b, owner: "PlayerResourcesState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0f379, 0x0f37b),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0f379, 0x0f37b) },
    NativeOwnedWramRange { lo: 0x0f3c5, hi: 0x0f3ca, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0f3c5, 0x0f3ca),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0f3c5, 0x0f3ca) },
    NativeOwnedWramRange { lo: 0x0f3e7, hi: 0x0f406, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0f3e7, 0x0f406),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0f3e7, 0x0f406) },
    NativeOwnedWramRange { lo: 0x0f4fe, hi: 0x0f4ff, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0f4fe, 0x0f4ff),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0f4fe, 0x0f4ff) },
];

impl ZeldaState {
    /// Complete cartridge-format WRAM, borrowing the backing bytes when there
    /// are no exclusive native owners. Never publishes legacy native mirrors.
    pub fn wram_image(&self) -> Cow<'_, [u8]> {
        self.wram_image_with_registry(NATIVE_OWNED_WRAM)
    }

    fn wram_image_with_registry(&self, registry: &[NativeOwnedWramRange]) -> Cow<'_, [u8]> {
        if registry.is_empty() {
            return Cow::Borrowed(&self.ram);
        }
        let mut image = self.ram.clone();
        // Each codec is narrow by contract and tested against its registry range.
        // One backing copy, with no per-range WRAM scratch copies.
        for range in registry {
            (range.export_owned)(self, &mut image);
        }
        Cow::Owned(image)
    }

    /// Adopt an external cartridge image. Unowned bytes go to backing storage;
    /// owned bytes go only to their native codec. Legacy mirrors retain their
    /// existing explicit refresh schedule; importing an image is not a frame.
    pub fn import_wram_image(&mut self, image: &[u8]) {
        self.import_wram_image_with_registry(image, NATIVE_OWNED_WRAM);
    }

    /// Live source-bus read. Export at the access, rather than keeping a stale
    /// image across native callbacks or an NMI acceptance.
    pub(super) fn read_wram_image_word(&self, address: usize, width: u8) -> u16 {
        match width {
            1 => u16::from(self.read_wram_byte(address)),
            2 => u16::from_le_bytes([self.read_wram_byte(address), self.read_wram_byte(address + 1)]),
            _ => panic!("invalid WRAM bus read width {width}"),
        }
    }

    fn import_wram_image_with_registry(&mut self, image: &[u8], registry: &[NativeOwnedWramRange]) {
        assert_eq!(image.len(), self.ram.len(), "WRAM image length");
        let mut next = 0;
        for range in registry {
            assert!(next <= range.lo && range.lo <= range.hi && range.hi < image.len(),
                "owned WRAM ranges must be sorted, disjoint and in bounds");
            self.ram[next..range.lo].copy_from_slice(&image[next..range.lo]);
            (range.import_owned)(self, image);
            next = range.hi + 1;
        }
        self.ram[next..].copy_from_slice(&image[next..]);
    }

    pub(crate) fn read_wram_byte(&self, address: usize) -> u8 {
        self.game_state.inventory.owned_byte(address).unwrap_or_else(|| self.ram[address])
    }

    pub(crate) fn write_wram_byte(&mut self, address: usize, value: u8) {
        if !self.game_state.inventory.write_owned_byte(address, value) { self.ram[address] = value; }
    }

    pub(crate) fn write_wram_word(&mut self, address: usize, value: u16) {
        for (byte, value) in value.to_le_bytes().into_iter().enumerate() {
            self.write_wram_byte(address + byte, value);
        }
    }

    pub(super) fn poison_owned_wram(&mut self) {
        if NATIVE_OWNED_WRAM.is_empty() { return; }
        if crate::debug_env::var("ZELDA3_POISON_OWNED_WRAM").is_ok_and(|value| value == "1") {
            for range in NATIVE_OWNED_WRAM {
                self.ram[range.lo..=range.hi].fill(0xa5);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A test-only owner exercises nonempty registry paths without claiming
    // that a production subsystem has passed the sole-ownership gate.
    const TEST_OWNER: &[NativeOwnedWramRange] = &[NativeOwnedWramRange {
        lo: 0x1a, hi: 0x1a, owner: "test frame counter",
        export_owned: |state, image| {
            image[0x1a] = state.game_state.frame.frame_counter;
        },
        import_owned: |state, image| { state.game_state.frame.frame_counter = image[0x1a]; },
    }];

    #[test]
    fn empty_registry_is_a_borrowed_identity_and_does_not_refresh_legacy_mirrors() {
        let mut state = ZeldaState::new();
        state.ram[0x1a] = 7;
        state.game_state.frame.frame_counter = 3;
        assert!(matches!(state.wram_image_with_registry(&[]), Cow::Borrowed(_)));
        assert_eq!(&*state.wram_image_with_registry(&[]), &state.ram);
        let mut image = state.ram.clone();
        image[0x1a] = 9;
        state.import_wram_image_with_registry(&image, &[]);
        assert_eq!(state.ram, image);
        assert_eq!(state.game_state.frame.frame_counter, 3);
    }

    #[test]
    fn owned_codec_ignores_poison_and_cannot_export_foreign_bytes() {
        let mut state = ZeldaState::new();
        state.ram[0x1a] = 0xa5;
        state.ram[0x1b] = 6;
        state.game_state.frame.frame_counter = 3;
        let mut image = state.wram_image_with_registry(TEST_OWNER).into_owned();
        assert_eq!((image[0x1a], image[0x1b]), (3, 6));
        assert_eq!(state.ram[0x1a], 0xa5);
        image[0x1a] = 9;
        image[0x1b] = 8;
        state.import_wram_image_with_registry(&image, TEST_OWNER);
        assert_eq!(state.game_state.frame.frame_counter, 9);
        assert_eq!((state.ram[0x1a], state.ram[0x1b]), (0xa5, 8));
        assert_eq!(&*state.wram_image_with_registry(TEST_OWNER), &image);
    }

    #[test]
    fn every_owned_codec_roundtrips_without_writing_foreign_bytes() {
        // Exercise the production registry as it grows, plus a nonempty pilot
        // registry so this contract is tested before any real owner is enabled.
        for registry in [NATIVE_OWNED_WRAM, TEST_OWNER] {
            for range in registry {
                for seed in [0u8, 0x5a, 0xff] {
                    let mut state = ZeldaState::new();
                    let source: Vec<u8> = (0..state.ram.len())
                        .map(|index| (index as u8).wrapping_mul(37).wrapping_add(seed))
                        .collect();
                    (range.import_owned)(&mut state, &source);
                    for sentinel in [0u8, 0xa5, 0xff] {
                        let mut image = vec![sentinel; source.len()];
                        (range.export_owned)(&state, &mut image);
                        assert_eq!(&image[range.lo..=range.hi], &source[range.lo..=range.hi],
                            "{} import/export", range.owner);
                        assert!(image[..range.lo].iter().chain(&image[range.hi + 1..])
                            .all(|&byte| byte == sentinel), "{} wrote foreign bytes", range.owner);
                    }
                }
            }
        }
    }

    #[test]
    fn state_v2_ignores_wram_mirror_refreshes_across_all_substates() {
        let mut state = ZeldaState::new();
        state.ram[0x1a] = 7;
        let before = state.canonical_state_hash();
        state.sync_native_game_state_from_ram();
        assert_eq!(before, state.canonical_state_hash());
        state.ram[0x1a] = 8;
        assert_ne!(before, state.canonical_state_hash());
    }

    #[test]
    #[should_panic(expected = "WRAM image length")]
    fn malformed_import_is_rejected_before_mutation() {
        ZeldaState::new().import_wram_image(&[0; 8]);
    }

    #[test]
    fn properties_stores_retire_owned_bytes_without_backing_publication() {
        use crate::cycle_models::pre_overworld_properties::PropertiesChild;
        let mut state = ZeldaState::new();
        for address in [0x202, 0x203, 0x40c, 0x4ca, 0xf340, 0xf36f] {
            state.ram[address] = 0xa5;
        }
        for (address, value) in [(0x202, 7), (0x203, 9), (0x4ca, 31), (0x7ef340, 2), (0x7ef36f, 4)] {
            state.native_overworld_properties_store(None, address, value, 1).unwrap();
            let offset = if address >= 0x7e0000 { address as usize - 0x7e0000 } else { address as usize };
            assert_eq!(state.read_wram_byte(offset), value as u8);
            assert_eq!(state.ram[offset], 0xa5);
        }
        state.native_overworld_properties_store(Some(PropertiesChild::DungeonExit), 0x40c, 0x9b17, 2).unwrap();
        assert_eq!(state.read_wram_image_word(0x40c, 2), 0x9b17);
        assert_eq!(state.ram[0x40c], 0xa5);
        assert_eq!(state.ram[0x40d], 0x9b);
    }

    #[test]
    fn live_save_transfer_retires_resources_at_each_source_word() {
        let mut state = ZeldaState::new();
        state.player_resources_mut().set_bombs(7);
        state.player_magic_mut().set_magic_power(41);
        state.ram[0xf343] = 0xa5;
        state.ram[0xf36e] = 0xa5;
        state.write_live_save_word(0xf342, 0x0901);
        assert_eq!(state.game_state.inventory.player_resources.bombs(), 9);
        assert_eq!(state.game_state.inventory.player_resources.magic.amount(), 41);
        assert_eq!(state.wram_image()[0xf342..0xf344], [1, 9]);
        assert_eq!(state.ram[0xf343], 0xa5);
        state.clear_live_save();
        assert_eq!(state.game_state.inventory.player_resources.bombs(), 0);
        assert_eq!(state.game_state.inventory.player_resources.magic.amount(), 0);
        assert_eq!(state.ram[0xf36e], 0xa5);
    }
}
