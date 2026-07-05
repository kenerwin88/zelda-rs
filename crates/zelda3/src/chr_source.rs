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
/// Per-frame-streamed dungeon BG CHR (`nmi_update_bg_char*`) is room-specific and
/// re-DMA'd over the same VRAM slots without a stable pack identity, so it is
/// tagged by a 24-bit hash of the streamed pixels. The hash is split into the
/// `(pack, tile_off)` fields so `modern_source_key` encodes it as
/// `(6<<24)|(hash&0xffffff)` with no special key path.
pub const CHR_KIND_BG_STREAM: u8 = 6;
/// Link CHR keyed by the frame-end 4bpp tile content hash. The raw Link source
/// identity (`CHR_KIND_LINK`) records useful DMA provenance, but source offsets are
/// reused across poses and can resolve to stale art. This derived kind is used by the
/// PNG/source-art renderer to pick the exact rendered Link tile.
pub const CHR_KIND_LINK_CONTENT: u8 = 8;

/// FNV-1a 32-bit hash of the little-endian bytes of `words`. Deterministic and
/// identical for identical content (so identical streamed tiles dedup to one
/// asset cell). Full 32 bits round-trip losslessly through the
/// `pack=hash>>16, tile_off=hash&0xffff` key split (see
/// [`VramChrSourceTable::record_tile_content_hash`]) — a narrower hash space
/// (e.g. 24-bit) hits real birthday-paradox collisions once enough distinct
/// streamed patterns exist route-wide (~18.7k for dungeon BG_STREAM).
pub fn chr_content_hash32(words: &[u16]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &w in words {
        for b in [(w & 0xff) as u8, (w >> 8) as u8] {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h
}

/// Link `tile_off` bit that distinguishes the two source buffers a Link CHR tile
/// can be DMA'd from. Link poses are non-injective by `(pack, relative-tile)`
/// alone: several DMA pieces of the same pose (e.g. BodyBottom and HeadBottom)
/// each start their relative tile index at 0, so they collapse to the same key
/// and resolve to the wrong asset cell. The fix keys a Link tile by its *source
/// identity* instead — the tile offset within the buffer it was copied from:
/// `tile_off = src_byte_offset >> 5` (32 bytes per 4bpp tile), with this flag set
/// when the source is the WRAM Link graphics buffer (`copy_ram_bytes_to_vram`) and
/// clear when it is the static Link sprite asset (`copy_asset_bytes_to_vram`,
/// offset relative to `0x8000`). The two raw address spaces overlap, so the flag
/// keeps them distinct. The Link key (`modern_source_key`) carries the full 14-bit
/// `tile_off` (flag + index) rather than the generic 8-bit field.
pub const CHR_LINK_SRC_RAM_FLAG: u16 = 0x2000;

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

impl From<LogicalChrSrc> for (u8, u16, u16) {
    fn from(src: LogicalChrSrc) -> Self {
        (src.kind, src.pack, src.tile_off)
    }
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

    /// Tag a single CHR slot with a content hash (see [`chr_content_hash32`]),
    /// splitting the full 32-bit hash into `pack=hash>>16`, `tile_off=hash&0xffff`
    /// — a lossless round-trip (both fields are u16).
    pub fn record_tile_content_hash(&mut self, slot: usize, kind: u8, hash32: u32) {
        if slot < self.entries.len() {
            self.entries[slot] = LogicalChrSrc {
                kind,
                pack: (hash32 >> 16) as u16,
                tile_off: (hash32 & 0xffff) as u16,
            };
        }
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

#[cfg(test)]
mod bg_stream_tests {
    use super::*;

    #[test]
    fn content_hash_is_deterministic_and_distinguishes_patterns() {
        let a = [0x1234u16; 16];
        let b = [0x1234u16; 16];
        let c = [
            0x1235u16, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234,
            0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234,
        ];
        assert_eq!(chr_content_hash32(&a), chr_content_hash32(&b));
        assert_ne!(chr_content_hash32(&a), chr_content_hash32(&c));
    }

    #[test]
    fn record_tile_content_hash_round_trips_through_key_split() {
        let mut t = VramChrSourceTable::default();
        let hash = 0xabcd_ef12u32;
        t.record_tile_content_hash(3, CHR_KIND_BG_STREAM, hash);
        let e = t.get(3);
        assert_eq!(e.kind, CHR_KIND_BG_STREAM);
        // pack = hash>>16, tile_off = hash&0xffff (lossless: both fields are u16)
        assert_eq!(e.pack, 0xabcd);
        assert_eq!(e.tile_off, 0xef12);
        let recon = ((e.pack as u32) << 16) | (e.tile_off as u32);
        assert_eq!(recon, hash);
    }
}
