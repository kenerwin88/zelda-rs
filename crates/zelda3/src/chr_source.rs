//! Per-VRAM-slot logical CHR source bookkeeping (Milestone 1 of the
//! animation-modeled asset renderer).
//!
//! Records, for every 8x8 4bpp CHR tile slot in VRAM, the *logical* graphics
//! source that filled it: which decompressed graphics pack + tile offset (BG /
//! sprite), or which Link pose DMA. This is pure game metadata captured at the
//! CHR->VRAM write paths; it never reads the VRAM pixel content and never
//! changes any VRAM/WRAM/game behavior. It is write-only observation used later
//! to drive off-VRAM-pixel asset rendering.
//!
//! VRAM is 0x8000 words; one 4bpp tile = 16 words, so there are
//! `0x8000 / 16 = 0x800` CHR tile slots. The slot for a VRAM word address is
//! `word_addr / 16`.

/// Number of 8x8 4bpp CHR tile slots in VRAM (`0x8000 words / 16`).
pub const VRAM_CHR_SLOTS: usize = 0x800;

/// `LogicalChrSrc::kind` values.
pub const CHR_KIND_NONE: u8 = 0;
pub const CHR_KIND_BG: u8 = 1;
pub const CHR_KIND_SPRITE: u8 = 2;
pub const CHR_KIND_LINK: u8 = 3;
/// The 2bpp BG3 HUD/font layer (mode 1). BG3 tiles are 8 VRAM words, so two of
/// them share one 16-word CHR slot — finer than this per-slot table can record.
/// BG3 graphics are STATIC (the HUD font/icons are loaded once, not re-DMA'd per
/// frame like sprites), so the asset library keys BG3 cells directly by the
/// tilemap entry `(tile_number, palette)` rather than via this slot table; this
/// kind tags such cells in `assets_by_source` and the off-VRAM render path.
pub const CHR_KIND_BG3: u8 = 4;
/// Per-frame animated overworld BG tiles (water/flowers). These reach VRAM via
/// the per-frame animated-tile DMA (`nmi_do_updates` → VRAM 0x3c00), streaming a
/// 32-tile window of a decompressed animation buffer whose window (phase) cycles
/// every few frames. The static `initialize_tilesets` BG tag for those slots is
/// stale once the animation streams over them, so this kind re-tags them at the
/// DMA with `(pack, absolute-buffer-tile-position)`, which is injective: the same
/// `(pack, position)` always decodes the same pixels regardless of phase.
pub const CHR_KIND_BG_ANIM: u8 = 5;

/// The logical source that produced one CHR tile slot.
///
/// * `kind`: 0=none, 1=bg, 2=sprite, 3=link.
/// * `pack`: the graphics pack index (BG/sprite) or the Link DMA graphics index.
/// * `tile_off`: tile offset within that pack / Link pose upload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LogicalChrSrc {
    pub kind: u8,
    pub pack: u16,
    pub tile_off: u16,
}

/// `[LogicalChrSrc; VRAM_CHR_SLOTS]` table, one entry per CHR tile slot.
///
/// Stored as a `Vec` so the type is `Default`-constructible (fixed arrays of
/// this length are not) and cheap to skip during serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VramChrSourceTable {
    entries: Vec<LogicalChrSrc>,
}

impl Default for VramChrSourceTable {
    fn default() -> Self {
        Self {
            entries: vec![LogicalChrSrc::default(); VRAM_CHR_SLOTS],
        }
    }
}

impl VramChrSourceTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read-only view of all slots.
    pub fn as_slice(&self) -> &[LogicalChrSrc] {
        &self.entries
    }

    /// Source recorded for one CHR tile slot (`word_addr / 16`).
    pub fn get(&self, slot: usize) -> LogicalChrSrc {
        self.entries.get(slot).copied().unwrap_or_default()
    }

    /// Record the logical source for `num_tiles` consecutive CHR tile slots
    /// starting at VRAM word address `start_word`.
    ///
    /// `tile_off` is assigned as the tile index within this upload (0-based),
    /// matching the do3->4 / Link DMA write order.
    pub fn record_tiles(&mut self, start_word: usize, num_tiles: usize, kind: u8, pack: u16) {
        let base_slot = start_word / 16;
        for t in 0..num_tiles {
            let slot = base_slot + t;
            if slot < self.entries.len() {
                self.entries[slot] = LogicalChrSrc {
                    kind,
                    pack,
                    tile_off: t as u16,
                };
            }
        }
    }

    /// Record the logical source for the CHR tile slots covered by a VRAM write
    /// of `num_words` words starting at `start_word` (rounds up to whole tiles).
    pub fn record_words(&mut self, start_word: usize, num_words: usize, kind: u8, pack: u16) {
        let num_tiles = num_words.div_ceil(16);
        self.record_tiles(start_word, num_tiles, kind, pack);
    }

    /// Like [`record_tiles`], but the recorded `tile_off` starts at `base_off`
    /// (so a partial upload of a larger logical tile run keeps a stable, global
    /// `tile_off` 0..N matching the full-run tagging).
    ///
    /// Used by the per-frame incremental sprite VRAM upload, which streams a
    /// 64-tile sprite subset to VRAM 16 tiles at a time across several frames:
    /// each 16-tile chunk records `tile_off = base_off + t` so the resulting
    /// `(kind, pack, tile_off)` key is identical to a single full-subset tag.
    pub fn record_tiles_from(
        &mut self,
        start_word: usize,
        num_tiles: usize,
        kind: u8,
        pack: u16,
        base_off: u16,
    ) {
        let base_slot = start_word / 16;
        for t in 0..num_tiles {
            let slot = base_slot + t;
            if slot < self.entries.len() {
                self.entries[slot] = LogicalChrSrc {
                    kind,
                    pack,
                    tile_off: base_off + t as u16,
                };
            }
        }
    }
}
