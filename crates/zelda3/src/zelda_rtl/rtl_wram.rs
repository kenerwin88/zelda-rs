//! WRAM is a wire image. Only the ranges in this registry may be exported
//! from native owners; projecting all of GameState would revive stale mirrors
//! and mode-reused-byte clobbers. Add an owner only after its poison proof.
use super::ZeldaState;
use std::borrow::Cow;

/// One exclusive native range, with inclusive cartridge addresses.
/// The codec may address the whole image, but only this range is exported.
/// An owner's import callback must load only that owner's registered fields.
pub struct NativeOwnedWramRange {
    pub lo: usize,
    pub hi: usize,
    pub owner: &'static str,
    export: fn(&ZeldaState, &mut [u8]),
    import: fn(&mut ZeldaState, &[u8]),
}

/// Sorted, disjoint ranges whose bytes no longer live in ZeldaState::ram.
/// Empty during the export-format pilot: legacy mirrors are NOT sole owners.
pub const NATIVE_OWNED_WRAM: &[NativeOwnedWramRange] = &[];

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
        // Isolate each codec: a broad legacy write_to_ram must never overwrite
        // another owner's bytes (or unowned bytes) in the exported image.
        let mut scratch = self.ram.clone();
        for range in registry {
            scratch.copy_from_slice(&self.ram);
            (range.export)(self, &mut scratch);
            image[range.lo..=range.hi].copy_from_slice(&scratch[range.lo..=range.hi]);
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
        let image = self.wram_image();
        match width {
            1 => u16::from(image[address]),
            2 => u16::from_le_bytes([image[address], image[address + 1]]),
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
            (range.import)(self, image);
            next = range.hi + 1;
        }
        self.ram[next..].copy_from_slice(&image[next..]);
    }

    /// Temporary materialization for an in-place cartridge/cycle-model call.
    /// Pair with import_owned_wram_from_ram before native execution resumes.
    pub(super) fn export_owned_wram_to_ram(&mut self) {
        if NATIVE_OWNED_WRAM.is_empty() { return; }
        let image = self.wram_image().into_owned();
        for range in NATIVE_OWNED_WRAM {
            self.ram[range.lo..=range.hi].copy_from_slice(&image[range.lo..=range.hi]);
        }
    }

    /// Retire raw machine writes at the same boundary as materialization.
    /// Read-only shadow probes use wram_image and never import speculative RAM.
    pub(super) fn import_owned_wram_from_ram(&mut self) {
        if NATIVE_OWNED_WRAM.is_empty() { return; }
        let image = self.ram.clone();
        for range in NATIVE_OWNED_WRAM {
            (range.import)(self, &image);
        }
        self.poison_owned_wram();
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
        export: |state, image| {
            image[0x1a] = state.game_state.frame.frame_counter;
            image[0x1b] = 0xff; // Deliberately overbroad codec, must be contained.
        },
        import: |state, image| { state.game_state.frame.frame_counter = image[0x1a]; },
    }];

    #[test]
    fn empty_registry_is_a_borrowed_identity_and_does_not_refresh_legacy_mirrors() {
        let mut state = ZeldaState::new();
        state.ram[0x1a] = 7;
        state.game_state.frame.frame_counter = 3;
        assert!(matches!(state.wram_image(), Cow::Borrowed(_)));
        assert_eq!(&*state.wram_image(), &state.ram);
        let mut image = state.ram.clone();
        image[0x1a] = 9;
        state.import_wram_image(&image);
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
    #[should_panic(expected = "WRAM image length")]
    fn malformed_import_is_rejected_before_mutation() {
        ZeldaState::new().import_wram_image(&[0; 8]);
    }

    #[test]
    fn live_save_transfer_defers_resource_adoption() {
        use crate::game_state::constants::{LINK_ITEM_BOMBS, LINK_MAGIC_POWER};
        let mut state = ZeldaState::new();
        state.player_resources_mut().set_bombs(7);
        state.player_magic_mut().set_magic_power(41);
        state.clear_live_save();
        // The existing live-save boundary intentionally imports progress
        // only. Immediately adopting resources changes serialized native state;
        // exporting the old native fields instead changes the WRAM image.
        assert_eq!(state.game_state.inventory.player_resources.bombs(), 7);
        assert_eq!(state.game_state.inventory.player_resources.magic.amount(), 41);
        assert_eq!(state.wram_image()[LINK_ITEM_BOMBS], 0);
        assert_eq!(state.wram_image()[LINK_MAGIC_POWER], 0);
        let old_hash = state.canonical_state_hash();
        state.game_state.inventory.player_resources =
            crate::game_state::InventoryState::load_from_ram(&state.ram).player_resources;
        assert_ne!(state.canonical_state_hash(), old_hash);
    }
}
