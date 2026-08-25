//! Replaceable semantic contract between timing authorities and translated
//! Zelda gameplay.
//!
//! Temporary emulator adapters may use CPU addresses and registers internally
//! to recognize these facts, but none of that provenance crosses this module.

use crate::DungeonResetSpritesCpuProgress;

/// The 544 OAM bytes consumed by one completed host scanout.
///
/// This is the rendered generation, not Zelda's live shadow or the PPU state
/// visible after a later VBlank DMA. A future native PPU owner can publish the
/// same byte-domain receipt without exposing emulator timing or registers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedOam {
    pub(crate) bytes: Vec<u8>,
}

impl PresentedOam {
    pub const BYTE_COUNT: usize = 544;

    pub fn new(bytes: Vec<u8>) -> Option<Self> {
        (bytes.len() == Self::BYTE_COUNT).then_some(Self { bytes })
    }
}

/// Palette-index pixels for the 64 OBJ tiles actually published by one host
/// scanout. This is a presentation-domain receipt: it exposes neither an
/// emulator cache nor CPU/raster provenance, and a native PPU owner can emit
/// the same 64 unflipped 8x8 tiles when that domain transfers authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedObjTiles {
    pub(crate) tile_pixels: Vec<u8>,
    pub(crate) valid_tiles: Vec<bool>,
}

impl PresentedObjTiles {
    pub const TILE_COUNT: usize = 64;
    pub const PIXELS_PER_TILE: usize = 64;

    pub fn new(tile_pixels: Vec<u8>, valid_tiles: Vec<bool>) -> Option<Self> {
        (tile_pixels.len() == Self::TILE_COUNT * Self::PIXELS_PER_TILE
            && valid_tiles.len() == Self::TILE_COUNT
            && tile_pixels.iter().all(|&pixel| pixel < 16))
        .then_some(Self {
            tile_pixels,
            valid_tiles,
        })
    }
}

/// Palette-index pixels for Zelda's 32-tile animated BG upload as consumed by
/// one completed host scanout.
///
/// Raw VRAM can already contain the following NMI's upload while the completed
/// surface still uses tiles decoded earlier in the field. This receipt names
/// that semantic animated-tile generation without exposing cache addresses or
/// emulator execution state to translated gameplay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedAnimatedBgTiles {
    pub(crate) tile_pixels: Vec<u8>,
    pub(crate) valid_tiles: Vec<bool>,
}

impl PresentedAnimatedBgTiles {
    pub const TILE_COUNT: usize = 32;
    pub const PIXELS_PER_TILE: usize = 64;

    pub fn new(tile_pixels: Vec<u8>, valid_tiles: Vec<bool>) -> Option<Self> {
        (tile_pixels.len() == Self::TILE_COUNT * Self::PIXELS_PER_TILE
            && valid_tiles.len() == Self::TILE_COUNT
            && tile_pixels.iter().all(|&pixel| pixel < 16))
        .then_some(Self {
            tile_pixels,
            valid_tiles,
        })
    }
}

/// The 256 SNES colors that produced one completed host scanout.
///
/// This is presentation state rather than Zelda's live palette buffer. The
/// temporary Snes9x authority and a future native PPU owner can therefore
/// publish the same receipt without exposing emulator timing or register
/// internals to translated gameplay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedCgram {
    pub(crate) colors: Vec<u16>,
}

/// The complete 165-word HUD tilemap published by Zelda's NMI DMA for one
/// completed host scanout.
///
/// The temporary oracle adapter may recognize the underlying DMA registers,
/// but translated gameplay receives only this semantic display domain. A
/// native timing owner can publish the same words without exposing CPU or PPU
/// execution details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedHudTilemap {
    pub(crate) words: Vec<u16>,
}

/// INIDISP scanout state that produced one completed host surface.
///
/// This is deliberately presentation state, not Zelda's live `$2100` mirror:
/// the main thread can author the next brightness while the host is still
/// returning the field rendered with the preceding value. The typed shape is
/// exactly the raster form supported by the native renderer: one brightness,
/// an optional forced-blank prefix, and an optional forced-blank suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentedInidisp {
    pub(crate) brightness: u8,
    pub(crate) forced_blank_prefix: u8,
    pub(crate) forced_blank_suffix_start: Option<u8>,
    /// The visible interval between the blank prefix and suffix is byte-exact
    /// to the preceding completed host surface. This is a presentation-domain
    /// fact, not an emulator timing guess: the temporary oracle derives it
    /// from consecutive returned surfaces, and a native owner can eventually
    /// publish the same fact from its own scanout generations.
    pub(crate) retain_prior_surface: bool,
}

/// Geometry used to turn the PPU's completed scanout surface into the host
/// surface returned for one call.
///
/// SNES overscan is rendered in the hardware scanline domain and the libretro
/// frontend crops an equal top/bottom border when returning a 224-line image.
/// Keeping that translation explicit lets a future native scanout owner emit
/// the same fact without exposing an emulator height or PPU register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentedScanoutGeometry {
    pub(crate) top_crop: u8,
}

impl PresentedScanoutGeometry {
    pub const MAX_TOP_CROP: u8 = 15;

    pub fn new(top_crop: u8) -> Option<Self> {
        (top_crop <= Self::MAX_TOP_CROP).then_some(Self { top_crop })
    }

    pub const fn top_crop(self) -> u8 {
        self.top_crop
    }
}

impl PresentedInidisp {
    pub const VISIBLE_LINES: usize = 224;

    pub fn new(
        brightness: u8,
        forced_blank_prefix: u8,
        forced_blank_suffix_start: Option<u8>,
    ) -> Option<Self> {
        (brightness <= 0x0f
            && usize::from(forced_blank_prefix) <= Self::VISIBLE_LINES
            && forced_blank_suffix_start.is_none_or(|suffix| {
                usize::from(suffix) <= Self::VISIBLE_LINES && suffix >= forced_blank_prefix
            }))
        .then_some(Self {
            brightness,
            forced_blank_prefix,
            forced_blank_suffix_start,
            retain_prior_surface: false,
        })
    }

    pub fn with_retained_prior_surface(mut self, retain: bool) -> Self {
        self.retain_prior_surface = retain;
        self
    }
}

impl PresentedHudTilemap {
    pub const WORD_COUNT: usize = 165;

    pub fn new(words: Vec<u16>) -> Option<Self> {
        (words.len() == Self::WORD_COUNT).then_some(Self { words })
    }
}

/// Interleaved stereo samples returned by one completed host call.
///
/// This is an audio-presentation receipt, not an SPC/DSP execution snapshot.
/// The temporary Snes9x authority and a future native audio owner can publish
/// the same samples without exposing emulator registers, addresses, or phase
/// state to translated gameplay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentedAudio {
    pub(crate) interleaved_stereo: Vec<i16>,
}

impl PresentedAudio {
    pub const CHANNELS: usize = 2;

    pub fn new(interleaved_stereo: Vec<i16>) -> Option<Self> {
        (!interleaved_stereo.is_empty() && interleaved_stereo.len().is_multiple_of(Self::CHANNELS))
            .then_some(Self { interleaved_stereo })
    }

    pub fn sample_frames(&self) -> usize {
        self.interleaved_stereo.len() / Self::CHANNELS
    }
}

/// Result of shadowing one authoritative audio-presentation receipt with the
/// native renderer. Authority remains with the receipt until this reports an
/// exact match and the domain is deliberately transferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTimingAudioShadowResult {
    pub sample_frames: usize,
    pub mismatched_interleaved_samples: usize,
    pub first_mismatch_interleaved: Option<usize>,
}

impl PresentedCgram {
    pub const COLOR_COUNT: usize = 256;

    pub fn new(colors: Vec<u16>) -> Option<Self> {
        (colors.len() == Self::COLOR_COUNT && colors.iter().all(|&color| color <= 0x7fff))
            .then_some(Self { colors })
    }
}

/// Semantic phase of Zelda's shared post-module caller when an NMI was
/// accepted. Timing backends may derive this from private execution details,
/// but translated gameplay sees only the source-level phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainLoopInterruption {
    LinkOam,
    SpritePreparation,
}

/// Source-level progress while one cached dungeon sprite temporarily occupies
/// a live sprite slot. Counts describe completed field publications in C
/// statement order; no CPU address or register provenance crosses the timing
/// authority boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CachedSpriteExecutionProgress {
    Loading { slot: u8, copied_fields: u8 },
    Restoring { slot: u8, live_fields: u8 },
}

/// Source-visible progress through one cached-sprite swap, together with the
/// hardware boundary which exposed that partial C state to the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CachedSpriteExecutionProgressReceipt {
    pub progress: CachedSpriteExecutionProgress,
    pub boundary: OriginalTimingBoundary,
}

/// Hardware boundary which made a source-level progress receipt observable.
///
/// This is deliberately a Zelda scheduling fact, not emulator provenance. A
/// native timing owner can publish the same distinction without exposing a
/// CPU program counter, register, or raster position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginalTimingBoundary {
    /// The host interval returned while the source call was still running;
    /// no interrupt had yet resumed that call stack.
    HostReturn,
    /// An NMI accepted after the reported source statement suspended the call
    /// stack. The translated continuation resumes on the following host.
    NmiAccepted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonResetSpritesProgressReceipt {
    pub progress: DungeonResetSpritesCpuProgress,
    pub boundary: OriginalTimingBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginalTimingSemanticReceipt {
    NmiAccepted,
    /// The main iteration returned from its module, but the following NMI
    /// interrupted a common caller phase before the CPU reached the main wait.
    /// The next host call must resume that phase and must not begin a fresh
    /// main iteration.
    MainLoopInterrupted(MainLoopInterruption),
    CachedSpriteExecutionProgress(CachedSpriteExecutionProgressReceipt),
    DungeonResetSpritesProgress(DungeonResetSpritesProgressReceipt),
    DmaPublicationCompleted {
        channel_mask: u8,
    },
}

/// One timing-authority result for exactly one upcoming host call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginalTimingHostReceipts {
    pub(crate) host_call: u64,
    pub(crate) input_state: u16,
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) presented_animated_bg_tiles: Option<PresentedAnimatedBgTiles>,
    pub(crate) presented_cgram: Option<PresentedCgram>,
    pub(crate) presented_inidisp: Option<PresentedInidisp>,
    pub(crate) presented_scanout_geometry: Option<PresentedScanoutGeometry>,
    pub(crate) presented_hud_tilemap: Option<PresentedHudTilemap>,
    pub(crate) presented_oam: Option<PresentedOam>,
    pub(crate) presented_obj_tiles: Option<PresentedObjTiles>,
    pub(crate) presented_audio: Option<PresentedAudio>,
}

impl OriginalTimingHostReceipts {
    pub fn new(
        host_call: u64,
        input_state: u16,
        semantic: Vec<OriginalTimingSemanticReceipt>,
    ) -> Self {
        Self {
            host_call,
            input_state: sanitize_original_timing_input(input_state),
            semantic,
            presented_animated_bg_tiles: None,
            presented_cgram: None,
            presented_inidisp: None,
            presented_scanout_geometry: None,
            presented_hud_tilemap: None,
            presented_oam: None,
            presented_obj_tiles: None,
            presented_audio: None,
        }
    }

    pub fn with_presented_animated_bg_tiles(mut self, receipt: PresentedAnimatedBgTiles) -> Self {
        self.presented_animated_bg_tiles = Some(receipt);
        self
    }

    pub fn with_presented_cgram(mut self, receipt: PresentedCgram) -> Self {
        self.presented_cgram = Some(receipt);
        self
    }

    pub fn with_presented_inidisp(mut self, receipt: PresentedInidisp) -> Self {
        self.presented_inidisp = Some(receipt);
        self
    }

    pub fn with_presented_scanout_geometry(mut self, receipt: PresentedScanoutGeometry) -> Self {
        self.presented_scanout_geometry = Some(receipt);
        self
    }

    pub fn with_presented_hud_tilemap(mut self, receipt: PresentedHudTilemap) -> Self {
        self.presented_hud_tilemap = Some(receipt);
        self
    }

    pub fn with_presented_oam(mut self, receipt: PresentedOam) -> Self {
        self.presented_oam = Some(receipt);
        self
    }

    pub fn with_presented_obj_tiles(mut self, receipt: PresentedObjTiles) -> Self {
        self.presented_obj_tiles = Some(receipt);
        self
    }

    pub fn with_presented_audio(mut self, receipt: PresentedAudio) -> Self {
        self.presented_audio = Some(receipt);
        self
    }

    pub fn semantic(&self) -> &[OriginalTimingSemanticReceipt] {
        &self.semantic
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginalTimingReceiptInstallError {
    TimingDisabled,
    ActiveHostDispatch,
    ReceiptAlreadyInstalled,
    UnconsumedPresentedAudio,
    DuplicateDungeonResetProgress,
    InvalidDungeonResetProgress,
    DuplicateCachedSpriteExecutionProgress,
    InvalidCachedSpriteExecutionProgress,
    DuplicateMainLoopInterruption,
    OutOfSequence { expected: u64, actual: u64 },
}

/// Canonicalize opposing directions at the authority boundary while retaining
/// every non-direction button. Backends must observe this same host input.
pub(crate) const fn sanitize_original_timing_input(inputs: u16) -> u16 {
    let mut directions = inputs & 0x00f0;
    if directions & 0x0030 == 0x0030 {
        directions &= !0x0010;
    }
    if directions & 0x00c0 == 0x00c0 {
        directions &= !0x0040;
    }
    (inputs & !0x00f0) | directions
}
