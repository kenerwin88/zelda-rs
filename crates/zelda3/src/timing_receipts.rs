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
    pub(crate) presented_cgram: Option<PresentedCgram>,
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
            presented_cgram: None,
            presented_oam: None,
            presented_obj_tiles: None,
            presented_audio: None,
        }
    }

    pub fn with_presented_cgram(mut self, receipt: PresentedCgram) -> Self {
        self.presented_cgram = Some(receipt);
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
