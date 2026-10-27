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
    NativeOwnedWramRange { lo: 0x00010, hi: 0x00011, owner: "FrameState",
        export_owned: |s, image| s.game_state.frame.export_owned_range(image, 0x00010, 0x00011),
        import_owned: |s, image| s.game_state.frame.import_owned_range(image, 0x00010, 0x00011) },
    NativeOwnedWramRange { lo: 0x00015, hi: 0x00016, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x00015, 0x00016),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x00015, 0x00016) },
    NativeOwnedWramRange { lo: 0x0001a, hi: 0x0001a, owner: "FrameState",
        export_owned: |s, image| s.game_state.frame.export_owned_range(image, 0x0001a, 0x0001a),
        import_owned: |s, image| s.game_state.frame.import_owned_range(image, 0x0001a, 0x0001a) },
    NativeOwnedWramRange { lo: 0x000b0, hi: 0x000b0, owner: "FrameState",
        export_owned: |s, image| s.game_state.frame.export_owned_range(image, 0x000b0, 0x000b0),
        import_owned: |s, image| s.game_state.frame.import_owned_range(image, 0x000b0, 0x000b0) },
    NativeOwnedWramRange { lo: 0x0010a, hi: 0x0010a, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x0010a, 0x0010a),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x0010a, 0x0010a) },
    NativeOwnedWramRange { lo: 0x0010c, hi: 0x0010c, owner: "FrameState",
        export_owned: |s, image| s.game_state.frame.export_owned_range(image, 0x0010c, 0x0010c),
        import_owned: |s, image| s.game_state.frame.import_owned_range(image, 0x0010c, 0x0010c) },
    NativeOwnedWramRange { lo: 0x0012c, hi: 0x00133, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x0012c, 0x00133),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x0012c, 0x00133) },
    NativeOwnedWramRange { lo: 0x00136, hi: 0x00136, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x00136, 0x00136),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x00136, 0x00136) },
    NativeOwnedWramRange { lo: 0x00202, hi: 0x00204, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x00202, 0x00204),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x00202, 0x00204) },
    NativeOwnedWramRange { lo: 0x0040c, hi: 0x0040c, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x0040c, 0x0040c),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x0040c, 0x0040c) },
    NativeOwnedWramRange { lo: 0x004aa, hi: 0x004aa, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x004aa, 0x004aa),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x004aa, 0x004aa) },
    NativeOwnedWramRange { lo: 0x004ca, hi: 0x004ca, owner: "PlayerResourcesState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x004ca, 0x004ca),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x004ca, 0x004ca) },
    NativeOwnedWramRange { lo: 0x00648, hi: 0x00648, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x00648, 0x00648),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x00648, 0x00648) },
    NativeOwnedWramRange { lo: 0x0064a, hi: 0x0064a, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x0064a, 0x0064a),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x0064a, 0x0064a) },
    NativeOwnedWramRange { lo: 0x00654, hi: 0x00654, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x00654, 0x00654),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x00654, 0x00654) },
    NativeOwnedWramRange { lo: 0x00656, hi: 0x00658, owner: "SaveProgressState",
        export_owned: |s, image| s.game_state.inventory.export_owned_range(image, 0x00656, 0x00658),
        import_owned: |s, image| s.game_state.inventory.import_owned_range(image, 0x00656, 0x00658) },
    NativeOwnedWramRange { lo: 0x00cf8, hi: 0x00cf8, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x00cf8, 0x00cf8),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x00cf8, 0x00cf8) },
    NativeOwnedWramRange { lo: 0x00fc1, hi: 0x00fc1, owner: "FrameState",
        export_owned: |s, image| s.game_state.frame.export_owned_range(image, 0x00fc1, 0x00fc1),
        import_owned: |s, image| s.game_state.frame.import_owned_range(image, 0x00fc1, 0x00fc1) },
    NativeOwnedWramRange { lo: 0x0c227, hi: 0x0c228, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x0c227, 0x0c228),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x0c227, 0x0c228) },
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
    NativeOwnedWramRange { lo: 0x1db20, hi: 0x1db39, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x1db20, 0x1db39),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x1db20, 0x1db39) },
    NativeOwnedWramRange { lo: 0x1db60, hi: 0x1db79, owner: "SystemSignalsState",
        export_owned: |s, image| s.game_state.system_signals.export_owned_range(image, 0x1db60, 0x1db79),
        import_owned: |s, image| s.game_state.system_signals.import_owned_range(image, 0x1db60, 0x1db79) },
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
        self.game_state.frame.owned_byte(address).or_else(|| self.game_state.system_signals.owned_byte(address)).or_else(|| self.game_state.inventory.owned_byte(address)).unwrap_or_else(|| self.ram[address])
    }

    pub(crate) fn write_wram_byte(&mut self, address: usize, value: u8) {
        if (0x12c..=0x12f).contains(&address) {
            self.native_time_apu_request(address - 0x12c, value);
        }
        if !self.game_state.frame.write_owned_byte(address, value) && !self.game_state.system_signals.write_owned_byte(address, value) && !self.game_state.inventory.write_owned_byte(address, value) { self.ram[address] = value; }
    }

    pub(crate) fn write_wram_word(&mut self, address: usize, value: u16) {
        for (byte, value) in value.to_le_bytes().into_iter().enumerate() {
            self.write_wram_byte(address + byte, value);
        }
    }

    pub(super) fn poison_owned_wram(&mut self) {
        if NATIVE_OWNED_WRAM.is_empty() { return; }
        if crate::debug_env::var!("ZELDA3_POISON_OWNED_WRAM").is_ok_and(|value| value == "1") {
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
        state.write_wram_byte(0x1a, 8);
        assert_ne!(before, state.canonical_state_hash());
    }

    #[test]
    #[should_panic(expected = "WRAM image length")]
    fn malformed_import_is_rejected_before_mutation() {
        ZeldaState::new().import_wram_image(&[0; 8]);
    }

    #[test]
    fn frame_owner_survives_poison_refresh_and_split_word_stores() {
        let mut state = ZeldaState::new();
        for address in [0x10, 0x11, 0x1a, 0xb0, 0x10c, 0xfc1] {
            state.ram[address] = 0xa5;
            state.write_wram_byte(address, 7);
        }
        state.write_wram_word(0x0f, 0x0903);
        state.write_wram_word(0x11, 0x5502);
        state.sync_native_game_state_from_ram();
        assert_eq!(state.game_state.frame.main_module_word(), 0x0209);
        assert_eq!((state.ram[0x0f], state.ram[0x12]), (3, 0x55));
        let image = state.wram_image();
        for address in [0x10, 0x11, 0x1a, 0xb0, 0x10c, 0xfc1] {
            assert_eq!(state.ram[address], 0xa5);
            assert_eq!(image[address], state.read_wram_byte(address));
        }
    }

    #[test]
    fn mode_gated_publishers_use_frame_owner_with_poisoned_backing() {
        for module in [0, 7] {
            let mut state = ZeldaState::new();
            state.set_main_module(module);
            state.ram[0x10] = 0xa5;
            state.ram[0..2].fill(0x55);
            state.save_load_scratch_mut().set_source_offset(0x1234);
            assert_eq!(&state.ram[0..2], if module == 7 { &[0x55, 0x55] } else { &[0x34, 0x12] });
            state.ram[0xfa8..0xfac].fill(0x55);
            state.dungeon_map_mut().reset_marker_offsets();
            assert_eq!(&state.ram[0xfa8..0xfac], if module == 7 { &[0x55; 4] } else { &[0x40, 0, 0x40, 0] });
        }
    }

    #[test]
    fn system_owner_preserves_partial_words_and_reserved_msu_bytes() {
        let mut state = ZeldaState::new();
        for range in NATIVE_OWNED_WRAM.iter().filter(|r| r.owner == "SystemSignalsState") {
            state.ram[range.lo..=range.hi].fill(0xa5);
        }
        state.write_wram_word(0x14, 0x0701);
        state.write_wram_word(0x16, 0x0905);
        state.write_wram_word(0x648, 0x1301);
        state.write_wram_word(0x1db39, 0x2207);
        state.write_wram_word(0x1db65, 0x4433);
        state.sync_native_game_state_from_ram();
        assert_eq!(state.read_wram_image_word(0x14, 2), 0x0701);
        assert_eq!(state.read_wram_image_word(0x16, 2), 0x0905);
        assert_eq!(state.game_state.system_signals.apui00(), 1);
        assert_eq!(state.ram[0x649], 0x13);
        assert_eq!(state.ram[0x1db3a], 0x22);
        assert_eq!(state.game_state.system_signals.msu_resume_info(
            crate::game_state::MsuResumeSlot::Primary).offset, 0x0044_3300);
        assert_eq!(state.game_state.system_signals.msu_resume_info(
            crate::game_state::MsuResumeSlot::Alternate).actual_track, 7);
        let before = state.canonical_state_hash();
        for range in NATIVE_OWNED_WRAM.iter().filter(|r| r.owner == "SystemSignalsState") {
            assert!(state.ram[range.lo..=range.hi].iter().all(|&b| b == 0xa5));
            state.ram[range.lo..=range.hi].fill(0x5a);
        }
        assert_eq!(before, state.canonical_state_hash());
        for (offset, address) in [(0x1b20, 0x1db20), (0x1b60, 0x1db60)] {
            state.write_wram_word(address, 0x1234);
            assert_eq!(state.clear_intro_wram_block_columns(offset, offset - 2), offset - 2);
            assert_eq!(state.read_wram_image_word(address, 2), 0);
            assert_eq!(&state.ram[address..address + 2], &[0x5a; 2]);
        }
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
