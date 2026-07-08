//! Provenance-clean CGRAM mirror.
//!
//! The modern renderer must render without sampling live CGRAM. CGRAM is only
//! ever written by copying the WRAM shadow (`MAIN_PALETTE_BUFFER`), and every
//! shadow write funnels through `PaletteBufferState`. This crate maintains a
//! **mirror** of the shadow whose words are derived exclusively from
//! provenance-clean sources — ROM/asset palette constants and pure integer
//! transforms over other mirror words — never by reading the live shadow.
//! Words the game wrote through a not-yet-annotated path are `Unknown`.
//!
//! Correctness is externally checkable: after each CGRAM commit the mirror
//! must equal the real shadow bit-for-bit with zero `Unknown` words (the
//! `ZELDA3_PALETTE_PROVENANCE_CHECK` replay gate). Once that holds over the
//! full route, the committed mirror is a CGRAM-free color source the renderer
//! can consume in place of `frame.cgram_rgba`.

/// Where a mirror word's value came from. Diagnostic only — the word itself is
/// what the renderer consumes; the tag exists to audit and to burn `Unknown`
/// writes down to zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTag {
    /// A word read from a named palette asset (or ROM palette data).
    Asset,
    /// A compile-time constant the game writes literally (0x0000 clears,
    /// 0x7fff white-filter fills, ...).
    Constant,
    /// Copied from another mirror word (bank-to-bank or intra-bank copies).
    Copied,
    /// Produced by a pure transform over mirror words (palette filters,
    /// restore steps, whiten).
    Computed,
}

/// One CGRAM-shadow word in the mirror: a provenance-clean value or a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MirrorWord {
    Known(u16, SourceTag),
    #[default]
    Unknown,
}

impl MirrorWord {
    pub fn value(self) -> Option<u16> {
        match self {
            MirrorWord::Known(value, _) => Some(value),
            MirrorWord::Unknown => None,
        }
    }

    pub fn is_unknown(self) -> bool {
        matches!(self, MirrorWord::Unknown)
    }
}

pub const PALETTE_WORDS: usize = 256;

/// SNES 15-bit channel stepping table used by `palette_filter_range`
/// (byte-for-byte the game's `kPaletteFilteringBits`).
pub const PALETTE_FILTERING_BITS: [u16; 64] = [
    0xffff, 0xffff, 0xfffe, 0xffff, 0x7fff, 0x7fff, 0x7fdf, 0xfbff, 0x7f7f, 0x7f7f, 0x7df7, 0xefbf,
    0x7bdf, 0x7bdf, 0x77bb, 0xddef, 0x7777, 0x7777, 0x6edd, 0xbb77, 0x6db7, 0x6db7, 0x5b6d, 0xb6db,
    0x5b5b, 0x5b5b, 0x56b6, 0xad6b, 0x5555, 0xad6b, 0x5555, 0xaaab, 0x5555, 0x5555, 0x2a55, 0x5555,
    0x2a55, 0x2a55, 0x294a, 0x5295, 0x2525, 0x2525, 0x2492, 0x4925, 0x1249, 0x1249, 0x1122, 0x4489,
    0x1111, 0x1111, 0x0844, 0x2211, 0x0421, 0x0421, 0x0208, 0x1041, 0x0101, 0x0101, 0x0020, 0x0401,
    0x0001, 0x0001, 0x0000, 0x0001,
];

pub const PALETTE_FILTER_UPPER_BITMASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010,
    0x0008, 0x0004, 0x0002, 0x0001,
];

/// One `palette_filter_range` step for a single word: `main` stepped by
/// `dt` (+1 darkening / -1 lightening encoded as 0xffff) per channel selected
/// by the countdown schedule against the reference `aux` word. Byte-exact
/// port of the game's loop body.
pub fn filter_range_step_word(main: u16, aux: u16, countdown: u16, darkening: bool) -> u16 {
    let load_ptr_offset = usize::from(countdown >= 0x10);
    let mask = PALETTE_FILTER_UPPER_BITMASKS[(countdown & 0x0f) as usize];
    let dt: u16 = if darkening { 1 } else { 0xffff };
    let mut c = main;
    if PALETTE_FILTERING_BITS[load_ptr_offset + ((aux & 0x001f) as usize) * 2] & mask == 0 {
        c = c.wrapping_add(dt);
    }
    if PALETTE_FILTERING_BITS[load_ptr_offset + ((aux & 0x03e0) >> 4) as usize] & mask == 0 {
        c = c.wrapping_add(dt.wrapping_shl(5));
    }
    if PALETTE_FILTERING_BITS[load_ptr_offset + ((aux & 0x7c00) >> 9) as usize] & mask == 0 {
        c = c.wrapping_add(dt.wrapping_shl(10));
    }
    c
}

/// One `PaletteFilter_RestoreAdditive` step: `main` moves +1 per channel that
/// still differs from `aux`.
pub fn restore_additive_step_word(main: u16, aux: u16) -> u16 {
    let mut cx = main;
    if (main & 0x001f) != (aux & 0x001f) {
        cx = cx.wrapping_add(1);
    }
    if (main & 0x03e0) != (aux & 0x03e0) {
        cx = cx.wrapping_add(0x20);
    }
    if (main & 0x7c00) != (aux & 0x7c00) {
        cx = cx.wrapping_add(0x400);
    }
    cx
}

/// One `PaletteFilter_RestoreSubtractive` step: `main` moves -1 per channel
/// that still differs from `aux`.
pub fn restore_subtractive_step_word(main: u16, aux: u16) -> u16 {
    let mut cx = main;
    if (main & 0x001f) != (aux & 0x001f) {
        cx = cx.wrapping_sub(1);
    }
    if (main & 0x03e0) != (aux & 0x03e0) {
        cx = cx.wrapping_sub(0x20);
    }
    if (main & 0x7c00) != (aux & 0x7c00) {
        cx = cx.wrapping_sub(0x400);
    }
    cx
}

/// `filter_majorly_whiten_color`: clamped per-channel add of `amt` (14, or 3
/// with the DIM_FLASHES enhancement).
pub fn whiten_word(color: u16, amt: u16) -> u16 {
    let r = ((color & 0x001f) + amt).min(0x001f);
    let g = ((color & 0x03e0) + (amt << 5)).min(0x03e0);
    let b = ((color & 0x7c00) + (amt << 10)).min(0x7c00);
    r | g | b
}

/// The three shadow banks plus the last committed CGRAM image, all as mirror
/// words. Indices are CGRAM word indices (0..256); the banks correspond to
/// `MAIN_PALETTE_BUFFER`, `AUX_PALETTE_BUFFER`, and `MAPBAK_PALETTE`.
#[derive(Clone, Debug)]
pub struct PaletteMirror {
    pub main: [MirrorWord; PALETTE_WORDS],
    pub aux: [MirrorWord; PALETTE_WORDS],
    pub backup: [MirrorWord; PALETTE_WORDS],
    /// Snapshot of `main` taken at each CGRAM upload; what the renderer may
    /// consume instead of live CGRAM.
    pub cgram: [MirrorWord; PALETTE_WORDS],
}

impl Default for PaletteMirror {
    fn default() -> Self {
        Self {
            main: [MirrorWord::Unknown; PALETTE_WORDS],
            aux: [MirrorWord::Unknown; PALETTE_WORDS],
            backup: [MirrorWord::Unknown; PALETTE_WORDS],
            cgram: [MirrorWord::Unknown; PALETTE_WORDS],
        }
    }
}

/// Which shadow bank a mirror operation targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bank {
    Main,
    Aux,
    Backup,
}

impl PaletteMirror {
    pub fn bank(&self, bank: Bank) -> &[MirrorWord; PALETTE_WORDS] {
        match bank {
            Bank::Main => &self.main,
            Bank::Aux => &self.aux,
            Bank::Backup => &self.backup,
        }
    }

    pub fn bank_mut(&mut self, bank: Bank) -> &mut [MirrorWord; PALETTE_WORDS] {
        match bank {
            Bank::Main => &mut self.main,
            Bank::Aux => &mut self.aux,
            Bank::Backup => &mut self.backup,
        }
    }

    /// A word loaded from ROM/asset palette data.
    pub fn set_asset_word(&mut self, bank: Bank, index: usize, value: u16) {
        if let Some(slot) = self.bank_mut(bank).get_mut(index) {
            *slot = MirrorWord::Known(value, SourceTag::Asset);
        }
    }

    /// A literal constant the game writes (0 clears, 0x7fff fills, ...).
    pub fn set_constant_word(&mut self, bank: Bank, index: usize, value: u16) {
        if let Some(slot) = self.bank_mut(bank).get_mut(index) {
            *slot = MirrorWord::Known(value, SourceTag::Constant);
        }
    }

    /// A write through a not-yet-annotated path: poisons the word.
    pub fn set_unknown_word(&mut self, bank: Bank, index: usize) {
        if let Some(slot) = self.bank_mut(bank).get_mut(index) {
            *slot = MirrorWord::Unknown;
        }
    }

    pub fn set_unknown_range(&mut self, bank: Bank, start_word: usize, words: usize) {
        for index in start_word..(start_word + words).min(PALETTE_WORDS) {
            self.set_unknown_word(bank, index);
        }
    }

    /// Copy a word from another mirror location (never from the live shadow).
    pub fn copy_word(&mut self, from: (Bank, usize), to: (Bank, usize)) {
        let word = match self.bank(from.0).get(from.1) {
            Some(MirrorWord::Known(value, _)) => MirrorWord::Known(*value, SourceTag::Copied),
            _ => MirrorWord::Unknown,
        };
        if let Some(slot) = self.bank_mut(to.0).get_mut(to.1) {
            *slot = word;
        }
    }

    pub fn copy_range(&mut self, from_bank: Bank, to_bank: Bank, start_word: usize, words: usize) {
        for index in start_word..(start_word + words).min(PALETTE_WORDS) {
            self.copy_word((from_bank, index), (to_bank, index));
        }
    }

    pub fn fill_constant_range(&mut self, bank: Bank, start_word: usize, words: usize, value: u16) {
        for index in start_word..(start_word + words).min(PALETTE_WORDS) {
            self.set_constant_word(bank, index, value);
        }
    }

    /// Apply a pure word transform `f(main, aux) -> main` to a range,
    /// reading ONLY mirror words.
    pub fn transform_main_range(
        &mut self,
        start_word: usize,
        end_word: usize,
        mut f: impl FnMut(u16, u16) -> u16,
    ) {
        for index in start_word..end_word.min(PALETTE_WORDS) {
            let next = match (self.main[index], self.aux[index]) {
                (MirrorWord::Known(main, _), MirrorWord::Known(aux, _)) => {
                    MirrorWord::Known(f(main, aux), SourceTag::Computed)
                }
                _ => MirrorWord::Unknown,
            };
            self.main[index] = next;
        }
    }

    /// Snapshot `main` as the committed CGRAM image (mirrors the NMI /
    /// mirror-warp `memcpy(cgram, main_palette_buffer)` uploads).
    pub fn commit_cgram(&mut self) {
        self.cgram = self.main;
    }

    /// Compare a mirror bank against the real shadow bytes (little-endian
    /// words). Returns (mismatched indices, unknown indices).
    pub fn audit_bank(&self, bank: Bank, shadow_bytes: &[u8]) -> BankAudit {
        let mut mismatches = Vec::new();
        let mut unknown = Vec::new();
        for index in 0..PALETTE_WORDS {
            let offset = index * 2;
            if offset + 1 >= shadow_bytes.len() {
                break;
            }
            let actual =
                u16::from(shadow_bytes[offset]) | (u16::from(shadow_bytes[offset + 1]) << 8);
            match self.bank(bank)[index] {
                MirrorWord::Known(value, _) if value == actual => {}
                MirrorWord::Known(value, _) => mismatches.push(WordAudit {
                    index,
                    mirror: Some(value),
                    actual,
                }),
                MirrorWord::Unknown => unknown.push(WordAudit {
                    index,
                    mirror: None,
                    actual,
                }),
            }
        }
        BankAudit {
            mismatches,
            unknown,
        }
    }
}

/// The committed-CGRAM mirror exported to the renderer: provenance-clean
/// words plus a per-word known mask. Once the known mask is all-true over the
/// full route, this is a complete CGRAM replacement derived purely from baked
/// data + semantic parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CgramProvenanceSnapshot {
    /// bgr15 words; unknown slots hold 0.
    pub words: [u16; PALETTE_WORDS],
    pub known: [bool; PALETTE_WORDS],
}

impl CgramProvenanceSnapshot {
    pub fn known_count(&self) -> usize {
        self.known.iter().filter(|known| **known).count()
    }
}

impl PaletteMirror {
    pub fn cgram_snapshot(&self) -> CgramProvenanceSnapshot {
        let mut words = [0u16; PALETTE_WORDS];
        let mut known = [false; PALETTE_WORDS];
        for (index, word) in self.cgram.iter().enumerate() {
            if let MirrorWord::Known(value, _) = word {
                words[index] = *value;
                known[index] = true;
            }
        }
        CgramProvenanceSnapshot { words, known }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WordAudit {
    pub index: usize,
    pub mirror: Option<u16>,
    pub actual: u16,
}

#[derive(Clone, Debug, Default)]
pub struct BankAudit {
    pub mismatches: Vec<WordAudit>,
    pub unknown: Vec<WordAudit>,
}

impl BankAudit {
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty() && self.unknown.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_range_step_matches_reference_shape() {
        // darkening adds toward the mask-selected channels; lightening is the
        // two's-complement inverse.
        let aux = 0x7fff;
        let dark = filter_range_step_word(0x0000, aux, 0, true);
        let light = filter_range_step_word(dark, aux, 0, false);
        assert_eq!(light, 0x0000);
    }

    #[test]
    fn restore_steps_converge_on_aux() {
        let aux = 0x1234;
        let mut main = 0x0000;
        for _ in 0..64 {
            main = restore_additive_step_word(main, aux);
            if main == aux {
                break;
            }
        }
        assert_eq!(main & 0x001f, aux & 0x001f);
    }

    #[test]
    fn whiten_clamps_channels() {
        assert_eq!(whiten_word(0x7fff, 14), 0x7fff);
        assert_eq!(whiten_word(0x0000, 14), (14) | (14 << 5) | (14 << 10));
    }

    #[test]
    fn transform_poisons_on_unknown_input() {
        let mut mirror = PaletteMirror::default();
        mirror.set_asset_word(Bank::Main, 0, 0x0010);
        // aux unknown -> result unknown
        mirror.transform_main_range(0, 1, restore_additive_step_word);
        assert!(mirror.main[0].is_unknown());

        mirror.set_asset_word(Bank::Main, 1, 0x0010);
        mirror.set_asset_word(Bank::Aux, 1, 0x0012);
        mirror.transform_main_range(1, 2, restore_additive_step_word);
        assert_eq!(mirror.main[1].value(), Some(0x0011));
    }

    #[test]
    fn audit_reports_mismatch_and_unknown() {
        let mut mirror = PaletteMirror::default();
        mirror.set_constant_word(Bank::Main, 0, 0x1234);
        mirror.set_constant_word(Bank::Main, 1, 0xdead);
        let mut shadow = vec![0u8; 512];
        shadow[0] = 0x34;
        shadow[1] = 0x12;
        shadow[2] = 0xff;
        shadow[3] = 0xff;
        let audit = mirror.audit_bank(Bank::Main, &shadow);
        assert_eq!(audit.mismatches.len(), 1);
        assert_eq!(audit.mismatches[0].index, 1);
        assert_eq!(audit.unknown.len(), PALETTE_WORDS - 2);
    }

    #[test]
    fn commit_snapshots_main() {
        let mut mirror = PaletteMirror::default();
        mirror.set_constant_word(Bank::Main, 3, 0x7fff);
        mirror.commit_cgram();
        assert_eq!(mirror.cgram[3].value(), Some(0x7fff));
    }
}
