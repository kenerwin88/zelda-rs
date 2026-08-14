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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FilterRangeStepWork {
    pub aux_nonzero: bool,
    pub red_steps: bool,
    pub green_steps: bool,
    pub blue_steps: bool,
}

/// Branch work selected by one iteration of the ROM's palette-filter loop.
/// Keeping this beside the color transform makes timing consumers follow the
/// same lookup-table decisions as the byte-producing implementation.
pub fn filter_range_step_work(aux: u16, countdown: u16) -> FilterRangeStepWork {
    let load_ptr_offset = usize::from(countdown >= 0x10);
    let mask = PALETTE_FILTER_UPPER_BITMASKS[(countdown & 0x0f) as usize];
    FilterRangeStepWork {
        aux_nonzero: aux != 0,
        red_steps: PALETTE_FILTERING_BITS[load_ptr_offset + ((aux & 0x001f) as usize) * 2] & mask
            == 0,
        green_steps: PALETTE_FILTERING_BITS[load_ptr_offset + ((aux & 0x03e0) >> 4) as usize]
            & mask
            == 0,
        blue_steps: PALETTE_FILTERING_BITS[load_ptr_offset + ((aux & 0x7c00) >> 9) as usize] & mask
            == 0,
    }
}

pub fn filter_range_step_word(main: u16, aux: u16, countdown: u16, darkening: bool) -> u16 {
    let work = filter_range_step_work(aux, countdown);
    let dt: u16 = if darkening { 1 } else { 0xffff };
    let mut c = main;
    if work.red_steps {
        c = c.wrapping_add(dt);
    }
    if work.green_steps {
        c = c.wrapping_add(dt.wrapping_shl(5));
    }
    if work.blue_steps {
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

/// What a conditional channel step compares the current channel against to
/// decide whether to move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelReference {
    /// The channel's maximum (all channel bits set).
    Max,
    /// Zero.
    Zero,
    /// The same channel of the reference `aux` word.
    Aux,
}

fn channel_target(aux: u16, channel_mask: u16, reference: ChannelReference) -> u16 {
    match reference {
        ChannelReference::Max => channel_mask,
        ChannelReference::Zero => 0,
        ChannelReference::Aux => aux & channel_mask,
    }
}

/// One whirlpool-filter channel step: if `main`'s masked channel differs from
/// the reference, step the WHOLE word by `step` (the game's whirlpool loops do
/// `color +/- step` without re-masking, so an at-boundary step carries into the
/// neighbouring channel exactly as the original does). Byte-exact port of the
/// `PaletteFilter_Whirlpool*` loop bodies.
pub fn whirlpool_channel_step_word(
    main: u16,
    aux: u16,
    channel_mask: u16,
    step: u16,
    up: bool,
    reference: ChannelReference,
) -> u16 {
    if (main & channel_mask) != channel_target(aux, channel_mask, reference) {
        if up {
            main.wrapping_add(step)
        } else {
            main.wrapping_sub(step)
        }
    } else {
        main
    }
}

/// One Trinexx flash/unflash channel step: rebuild the word from its untouched
/// other channels OR'd with the stepped channel value. The channel arithmetic
/// wraps within the u16 before the OR (matching `(v & !mask) | (chan +/- step)`
/// in the game, which can spill high bits when the channel underflows). Byte-
/// exact port of the `Trinexx_*ShellPalette_*` loop bodies.
pub fn trinexx_channel_step_word(
    main: u16,
    aux: u16,
    channel_mask: u16,
    step: u16,
    up: bool,
    reference: ChannelReference,
) -> u16 {
    let chan = main & channel_mask;
    let delta = if chan != channel_target(aux, channel_mask, reference) {
        step
    } else {
        0
    };
    let new_chan = if up {
        chan.wrapping_add(delta)
    } else {
        chan.wrapping_sub(delta)
    };
    (main & !channel_mask) | new_chan
}

/// The three shadow banks plus the last committed CGRAM image, all as mirror
/// words. Indices are CGRAM word indices (0..256); the banks correspond to
/// `MAIN_PALETTE_BUFFER`, `AUX_PALETTE_BUFFER`, and `MAPBAK_PALETTE`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PaletteMirror {
    #[serde(with = "serde_big_array::BigArray")]
    pub main: [MirrorWord; PALETTE_WORDS],
    #[serde(with = "serde_big_array::BigArray")]
    pub aux: [MirrorWord; PALETTE_WORDS],
    #[serde(with = "serde_big_array::BigArray")]
    pub backup: [MirrorWord; PALETTE_WORDS],
    /// Snapshot of `main` taken at each CGRAM upload; what the renderer may
    /// consume instead of live CGRAM.
    #[serde(with = "serde_big_array::BigArray")]
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

    /// Swap two mirror words, preserving each word's value and source tag.
    pub fn swap_words(&mut self, a: (Bank, usize), b: (Bank, usize)) {
        let wa = self.bank(a.0).get(a.1).copied();
        let wb = self.bank(b.0).get(b.1).copied();
        if let (Some(wa), Some(wb)) = (wa, wb) {
            self.bank_mut(a.0)[a.1] = wb;
            self.bank_mut(b.0)[b.1] = wa;
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

    /// Reconstitute the committed CGRAM image at a full-state restore boundary from the
    /// authoritative restored PPU CGRAM.
    ///
    /// `commit_cgram` (main → cgram) only runs at CGRAM-upload commits. A snapshot restore
    /// bulk-loads `ppu.cgram` directly and may be followed by many frames where
    /// `should_update_cgram` stays false (e.g. a fade), so without this the committed CGRAM image
    /// stays stale — the renderer consumes this image, so it must track what the PPU actually
    /// holds after the restore, not the last pre-restore upload. Tagged `Copied`: a restore is a
    /// full-state reload point (like power-on) where the restored PPU state is authoritative.
    pub fn reconstitute_cgram(&mut self, cgram: &[u16]) {
        for index in 0..PALETTE_WORDS {
            self.cgram[index] =
                MirrorWord::Known(cgram.get(index).copied().unwrap_or(0), SourceTag::Copied);
        }
    }

    /// Compact per-bank source-tag histogram, e.g.
    /// `main{asset=12 const=200 copied=0 computed=44 unknown=0} aux{..} backup{..} cgram{..}`.
    /// Diagnostic only — confirms a restored mirror carries its true derivation tags rather
    /// than an all-`Copied` shadow reconstitution.
    pub fn tag_histogram_line(&self) -> String {
        fn bank_counts(words: &[MirrorWord; PALETTE_WORDS]) -> String {
            let (mut asset, mut constant, mut copied, mut computed, mut unknown) = (0, 0, 0, 0, 0);
            for w in words {
                match w {
                    MirrorWord::Known(_, SourceTag::Asset) => asset += 1,
                    MirrorWord::Known(_, SourceTag::Constant) => constant += 1,
                    MirrorWord::Known(_, SourceTag::Copied) => copied += 1,
                    MirrorWord::Known(_, SourceTag::Computed) => computed += 1,
                    MirrorWord::Unknown => unknown += 1,
                }
            }
            format!(
                "asset={asset} const={constant} copied={copied} computed={computed} unknown={unknown}"
            )
        }
        format!(
            "main{{{}}} aux{{{}}} backup{{{}}} cgram{{{}}}",
            bank_counts(&self.main),
            bank_counts(&self.aux),
            bank_counts(&self.backup),
            bank_counts(&self.cgram),
        )
    }

    /// Audit the committed CGRAM image against the live PPU CGRAM (the values classic renders).
    ///
    /// The main-bank/shadow audit cannot see a stale committed CGRAM image (the image is only
    /// refreshed at upload commits, but the renderer reads it every frame). This closes that
    /// blind spot.
    pub fn audit_cgram(&self, ppu_cgram: &[u16]) -> BankAudit {
        let mut mismatches = Vec::new();
        let mut unknown = Vec::new();
        for index in 0..PALETTE_WORDS {
            let Some(&actual) = ppu_cgram.get(index) else {
                break;
            };
            match self.cgram[index] {
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

impl Default for CgramProvenanceSnapshot {
    fn default() -> Self {
        Self {
            words: [0; PALETTE_WORDS],
            known: [false; PALETTE_WORDS],
        }
    }
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
    fn filter_range_work_reports_the_same_channel_decisions_as_the_transform() {
        let countdown = 28;
        let aux = 0x7fff;
        let work = filter_range_step_work(aux, countdown);
        assert!(work.aux_nonzero);

        let stepped = filter_range_step_word(0, aux, countdown, true);
        assert_eq!(stepped & 0x001f != 0, work.red_steps);
        assert_eq!(stepped & 0x03e0 != 0, work.green_steps);
        assert_eq!(stepped & 0x7c00 != 0, work.blue_steps);

        assert_eq!(
            filter_range_step_work(0, countdown),
            FilterRangeStepWork::default()
        );
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

    /// Words covering channel minimums, maximums, mid-values, and out-of-range
    /// high bits so the wrap/carry edge cases are exercised.
    const SAMPLE_WORDS: [u16; 12] = [
        0x0000, 0x7fff, 0xffff, 0x7c00, 0x03e0, 0x001f, 0x7800, 0x03c0, 0x001e, 0x1234, 0x4210,
        0x8421,
    ];

    #[test]
    fn whirlpool_blue_step_matches_original() {
        // PaletteFilter_WhirlpoolBlue body.
        for &main in &SAMPLE_WORDS {
            let expected = if (main & 0x7c00) != 0x7c00 {
                main.wrapping_add(0x400)
            } else {
                main
            };
            assert_eq!(
                whirlpool_channel_step_word(main, 0, 0x7c00, 0x400, true, ChannelReference::Max),
                expected,
                "main={main:#06x}"
            );
        }
    }

    #[test]
    fn whirlpool_isolate_blue_step_matches_original() {
        // PaletteFilter_IsolateWhirlpoolBlue body: green then red, disjoint
        // channels, so two passes equal the single sequential loop.
        for &main in &SAMPLE_WORDS {
            let mut expected = main;
            if expected & 0x03e0 != 0 {
                expected = expected.wrapping_sub(0x20);
            }
            if expected & 0x001f != 0 {
                expected = expected.wrapping_sub(1);
            }
            let g =
                whirlpool_channel_step_word(main, 0, 0x03e0, 0x20, false, ChannelReference::Zero);
            let gr = whirlpool_channel_step_word(g, 0, 0x001f, 1, false, ChannelReference::Zero);
            assert_eq!(gr, expected, "main={main:#06x}");
        }
    }

    #[test]
    fn whirlpool_restore_blue_step_matches_original() {
        // PaletteFilter_WhirlpoolRestoreBlue body.
        for &main in &SAMPLE_WORDS {
            for &aux in &SAMPLE_WORDS {
                let expected = if (main & 0x7c00) != (aux & 0x7c00) {
                    main.wrapping_sub(0x400)
                } else {
                    main
                };
                assert_eq!(
                    whirlpool_channel_step_word(
                        main,
                        aux,
                        0x7c00,
                        0x400,
                        false,
                        ChannelReference::Aux
                    ),
                    expected,
                    "main={main:#06x} aux={aux:#06x}"
                );
            }
        }
    }

    #[test]
    fn whirlpool_restore_red_green_step_matches_original() {
        // PaletteFilter_WhirlpoolRestoreRedGreen body.
        for &main in &SAMPLE_WORDS {
            for &aux in &SAMPLE_WORDS {
                let mut expected = main;
                if expected & 0x03e0 != aux & 0x03e0 {
                    expected = expected.wrapping_add(0x20);
                }
                if expected & 0x001f != aux & 0x001f {
                    expected = expected.wrapping_add(1);
                }
                let g = whirlpool_channel_step_word(
                    main,
                    aux,
                    0x03e0,
                    0x20,
                    true,
                    ChannelReference::Aux,
                );
                let gr =
                    whirlpool_channel_step_word(g, aux, 0x001f, 1, true, ChannelReference::Aux);
                assert_eq!(gr, expected, "main={main:#06x} aux={aux:#06x}");
            }
        }
    }

    #[test]
    fn trinexx_flash_red_step_matches_original() {
        for &main in &SAMPLE_WORDS {
            let v = main;
            let red = (v & 0x1f).wrapping_add(u16::from((v & 0x1f) != 0x1f));
            let expected = (v & 0xffe0) | red;
            assert_eq!(
                trinexx_channel_step_word(main, 0, 0x1f, 1, true, ChannelReference::Max),
                expected,
                "main={main:#06x}"
            );
        }
    }

    #[test]
    fn trinexx_unflash_red_step_matches_original() {
        for &main in &SAMPLE_WORDS {
            for &aux in &SAMPLE_WORDS {
                let (v, u) = (main, aux);
                let red = (v & 0x1f).wrapping_sub(u16::from((v & 0x1f) != (u & 0x1f)));
                let expected = (v & 0xffe0) | red;
                assert_eq!(
                    trinexx_channel_step_word(main, aux, 0x1f, 1, false, ChannelReference::Aux),
                    expected,
                    "main={main:#06x} aux={aux:#06x}"
                );
            }
        }
    }

    #[test]
    fn trinexx_flash_blue_step_matches_original() {
        for &main in &SAMPLE_WORDS {
            let v = main;
            let blue = (v & 0x7c00).wrapping_add(if (v & 0x7c00) != 0x7c00 { 0x0400 } else { 0 });
            let expected = (v & !0x7c00) | blue;
            assert_eq!(
                trinexx_channel_step_word(main, 0, 0x7c00, 0x400, true, ChannelReference::Max),
                expected,
                "main={main:#06x}"
            );
        }
    }

    #[test]
    fn trinexx_unflash_blue_step_matches_original() {
        for &main in &SAMPLE_WORDS {
            for &aux in &SAMPLE_WORDS {
                let (v, u) = (main, aux);
                let blue = (v & 0x7c00).wrapping_sub(if (v & 0x7c00) != (u & 0x7c00) {
                    0x0400
                } else {
                    0
                });
                let expected = (v & !0x7c00) | blue;
                assert_eq!(
                    trinexx_channel_step_word(
                        main,
                        aux,
                        0x7c00,
                        0x400,
                        false,
                        ChannelReference::Aux
                    ),
                    expected,
                    "main={main:#06x} aux={aux:#06x}"
                );
            }
        }
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

    #[test]
    fn bincode_round_trip_preserves_words_and_tags() {
        // A mirror with a mix of Known (each SourceTag variant) and Unknown words
        // across all four banks must survive a bincode round trip byte-for-byte.
        let mut mirror = PaletteMirror::default();
        mirror.set_asset_word(Bank::Main, 1, 0x1234);
        mirror.set_constant_word(Bank::Main, 2, 0x7fff);
        mirror.set_asset_word(Bank::Aux, 5, 0x0abc);
        mirror.copy_word((Bank::Main, 1), (Bank::Backup, 9));
        mirror.commit_cgram();
        // Also exercise the Computed tag directly so all four SourceTag variants
        // and Unknown are represented in the payload.
        mirror.main[3] = MirrorWord::Known(0x0f0f, SourceTag::Computed);

        let bytes = bincode::serialize(&mirror).expect("serialize mirror");
        let restored: PaletteMirror = bincode::deserialize(&bytes).expect("deserialize mirror");

        for bank in [Bank::Main, Bank::Aux, Bank::Backup] {
            assert_eq!(
                mirror.bank(bank),
                restored.bank(bank),
                "{bank:?} bank differs"
            );
        }
        assert_eq!(mirror.cgram, restored.cgram, "cgram bank differs");
    }
}
