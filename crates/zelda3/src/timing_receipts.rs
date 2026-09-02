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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedOam {
    pub(crate) bytes: Vec<u8>,
}

impl PresentedOam {
    pub const BYTE_COUNT: usize = 544;

    pub fn new(bytes: Vec<u8>) -> Option<Self> {
        (bytes.len() == Self::BYTE_COUNT).then_some(Self { bytes })
    }
}

/// Sparse palette-index pixels for the OBJ tiles actually published by one
/// host scanout. Each entry names its aligned 15-bit VRAM word address; tiles
/// absent from the receipt retain the preceding decoded-cache generation.
/// This is a presentation-domain receipt: it exposes neither an emulator cache
/// nor CPU/raster provenance, and a native PPU owner can emit the same entries
/// when that domain transfers authority.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "PresentedObjTilesUnchecked")]
pub struct PresentedObjTiles {
    pub(crate) tile_word_addresses: Vec<u16>,
    pub(crate) tile_pixels: Vec<u8>,
}

impl PresentedObjTiles {
    pub const WORDS_PER_TILE: usize = 16;
    pub const PIXELS_PER_TILE: usize = 64;
    pub const MAX_TILE_COUNT: usize = 0x800;

    pub fn new(tile_word_addresses: Vec<u16>, tile_pixels: Vec<u8>) -> Option<Self> {
        let receipt = Self {
            tile_word_addresses,
            tile_pixels,
        };
        receipt.is_valid().then_some(receipt)
    }

    fn is_valid(&self) -> bool {
        if self.tile_word_addresses.len() > Self::MAX_TILE_COUNT
            || self.tile_pixels.len() != self.tile_word_addresses.len() * Self::PIXELS_PER_TILE
            || self.tile_pixels.iter().any(|&pixel| pixel >= 16)
        {
            return false;
        }
        let mut addresses = [false; Self::MAX_TILE_COUNT];
        self.tile_word_addresses.iter().all(|&address| {
            let address = usize::from(address);
            if address >= 0x8000 || address % Self::WORDS_PER_TILE != 0 {
                return false;
            }
            let tile = address / Self::WORDS_PER_TILE;
            !std::mem::replace(&mut addresses[tile], true)
        })
    }
}

#[derive(serde::Deserialize)]
struct PresentedObjTilesUnchecked {
    tile_word_addresses: Vec<u16>,
    tile_pixels: Vec<u8>,
}

impl TryFrom<PresentedObjTilesUnchecked> for PresentedObjTiles {
    type Error = &'static str;

    fn try_from(value: PresentedObjTilesUnchecked) -> Result<Self, Self::Error> {
        Self::new(value.tile_word_addresses, value.tile_pixels)
            .ok_or("invalid sparse presented OBJ tile receipt")
    }
}

/// Semantic destination selected by Zelda's animated-background setup.
///
/// These names replace the raw `$3b00`/`$3c00` VRAM addresses at the backend
/// boundary so translated gameplay never depends on emulator address state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PresentedAnimatedBgDestination {
    Dungeon,
    Overworld,
}

impl PresentedAnimatedBgDestination {
    pub(crate) const fn word_address(self) -> usize {
        match self {
            Self::Dungeon => 0x3b00,
            Self::Overworld => 0x3c00,
        }
    }
}

/// Palette-index pixels for Zelda's complete 32-tile animated BG upload as
/// consumed by one completed host scanout.
///
/// The timing backend may derive this from private DMA/VRAM execution, but
/// translated gameplay observes only the semantic destination and complete
/// decoded generation. A future native owner can emit the same receipt.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedAnimatedBgTiles {
    pub(crate) destination: PresentedAnimatedBgDestination,
    pub(crate) tile_pixels: Vec<u8>,
}

impl PresentedAnimatedBgTiles {
    pub const TILE_COUNT: usize = 32;
    pub const PIXELS_PER_TILE: usize = 64;

    pub fn new(destination: PresentedAnimatedBgDestination, tile_pixels: Vec<u8>) -> Option<Self> {
        (tile_pixels.len() == Self::TILE_COUNT * Self::PIXELS_PER_TILE
            && tile_pixels.iter().all(|&pixel| pixel < 16))
        .then_some(Self {
            destination,
            tile_pixels,
        })
    }
}

/// The 256 SNES colors that produced one completed host scanout.
///
/// This is presentation state rather than Zelda's live palette buffer. The
/// temporary Snes9x authority and a future native PPU owner can therefore
/// publish the same receipt without exposing emulator timing or register
/// internals to translated gameplay.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedHudTilemap {
    pub(crate) words: Vec<u16>,
}

/// The 0x3f0 BG3 character words used to draw Zelda's dialogue text in one
/// completed host scanout.
///
/// This is the semantic result of C `NMI_UploadBG3Text`, not its WRAM source,
/// DMA registers, CPU address, or raster provenance. A native publication
/// owner can therefore emit the same receipt after proving its NMI boundary.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedDialogueText {
    pub(crate) words: Vec<u16>,
}

/// One BG name-table generation used by a completed host scanout.
///
/// The receipt names the semantic PPU layer and tilemap geometry, but carries
/// no CPU address, program counter, raster deadline, or emulator-owned cache.
/// A native timing owner can therefore publish the same layer generation when
/// authority for this domain moves out of the temporary Snes9x backend.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedBgTilemapLayer {
    pub(crate) layer: u8,
    pub(crate) word_address: u16,
    pub(crate) wider: bool,
    pub(crate) higher: bool,
    pub(crate) words: Vec<u16>,
}

impl PresentedBgTilemapLayer {
    pub const WORDS_PER_SCREEN: usize = 32 * 32;

    pub fn new(
        layer: u8,
        word_address: u16,
        wider: bool,
        higher: bool,
        words: Vec<u16>,
    ) -> Option<Self> {
        let screen_count = (if wider { 2 } else { 1 }) * (if higher { 2 } else { 1 });
        (layer < 4 && words.len() == Self::WORDS_PER_SCREEN * screen_count).then_some(Self {
            layer,
            word_address,
            wider,
            higher,
            words,
        })
    }
}

/// Complete BG name-table publication for one completed host scanout.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedBgTilemaps {
    pub(crate) layers: Vec<PresentedBgTilemapLayer>,
}

impl PresentedBgTilemaps {
    pub const LAYER_COUNT: usize = 4;

    pub fn new(layers: Vec<PresentedBgTilemapLayer>) -> Option<Self> {
        let complete = layers.len() == Self::LAYER_COUNT
            && layers
                .iter()
                .enumerate()
                .all(|(layer, receipt)| usize::from(receipt.layer) == layer);
        complete.then_some(Self { layers })
    }
}

/// Per-scanline BG scroll registers that produced one completed host scanout.
///
/// This is presentation state rather than Zelda's live `$210d..$2114`
/// mirrors. A timing backend may derive it from private raster execution, but
/// translated gameplay sees only the rendered horizontal/vertical pairs.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedBgScroll {
    pub(crate) scanlines: Vec<[[u16; 2]; 4]>,
}

impl PresentedBgScroll {
    pub const LAYER_COUNT: usize = 4;
    pub const VISIBLE_LINES: usize = 224;

    pub fn new(scanlines: Vec<[[u16; 2]; Self::LAYER_COUNT]>) -> Option<Self> {
        (scanlines.len() == Self::VISIBLE_LINES).then_some(Self { scanlines })
    }

    pub fn scanlines(&self) -> &[[[u16; 2]; Self::LAYER_COUNT]] {
        &self.scanlines
    }
}

/// Per-scanline Mode 7 transform used by one completed host scanout.
///
/// These are renderer-domain values: matrix A/B/C/D, center X/Y, and the
/// horizontal/vertical Mode 7 offsets after the source PPU/HDMA pipeline has
/// selected the generation for each visible row. The temporary oracle may
/// derive them from private PPU state, while translated gameplay sees only
/// the completed transform and can later replace the backend unchanged.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct PresentedMode7Transform {
    pub(crate) scanlines: Vec<[i16; 8]>,
}

impl PresentedMode7Transform {
    pub const FIELD_COUNT: usize = 8;
    pub const VISIBLE_LINES: usize = 224;

    pub fn new(scanlines: Vec<[i16; Self::FIELD_COUNT]>) -> Option<Self> {
        (scanlines.len() == Self::VISIBLE_LINES).then_some(Self { scanlines })
    }

    pub fn scanlines(&self) -> &[[i16; Self::FIELD_COUNT]] {
        &self.scanlines
    }
}

impl<'de> serde::Deserialize<'de> for PresentedMode7Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SerializedPresentedMode7Transform {
            scanlines: Vec<[i16; PresentedMode7Transform::FIELD_COUNT]>,
        }

        let serialized = SerializedPresentedMode7Transform::deserialize(deserializer)?;
        let actual_lines = serialized.scanlines.len();
        Self::new(serialized.scanlines).ok_or_else(|| {
            <D::Error as serde::de::Error>::custom(format_args!(
                "expected exactly {} Mode 7 scanlines, received {actual_lines}",
                Self::VISIBLE_LINES,
            ))
        })
    }
}

/// Per-scanline window boundaries that produced one completed host scanout.
///
/// This is presentation state rather than Zelda's live `$2126..$2129`
/// mirrors or an HDMA table. A timing backend may derive it from private
/// raster execution, while translated gameplay consumes only the two rendered
/// left/right pairs and can replace the backend without changing this type.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PresentedWindowMask {
    pub(crate) scanlines: Vec<[[u8; 2]; 2]>,
    pub(crate) screen_windowed_layers: [u8; 2],
    /// Stable per-layer window predicates for BG1..BG4, OBJ, and color math.
    ///
    /// Each value is semantic rather than register-packed: bit 0 enables
    /// Window 1, bit 1 masks outside Window 1, bit 2 enables Window 2, and bit
    /// 3 masks outside Window 2. The current native renderer supports Zelda's
    /// OR combination when both windows are enabled; the oracle adapter fails
    /// closed before constructing this receipt for another overlap operation.
    pub(crate) layer_predicates: [u8; 6],
}

impl PresentedWindowMask {
    pub const WINDOW_COUNT: usize = 2;
    pub const VISIBLE_LINES: usize = 224;

    pub fn new(
        scanlines: Vec<[[u8; 2]; Self::WINDOW_COUNT]>,
        screen_windowed_layers: [u8; 2],
        layer_predicates: [u8; 6],
    ) -> Option<Self> {
        (scanlines.len() == Self::VISIBLE_LINES
            && screen_windowed_layers
                .iter()
                .all(|&layers| layers & !0x3f == 0)
            && layer_predicates
                .iter()
                .all(|&predicate| predicate & !0x0f == 0))
        .then_some(Self {
            scanlines,
            screen_windowed_layers,
            layer_predicates,
        })
    }

    pub fn scanlines(&self) -> &[[[u8; 2]; Self::WINDOW_COUNT]] {
        &self.scanlines
    }

    pub fn screen_windowed_layers(&self) -> [u8; 2] {
        self.screen_windowed_layers
    }

    pub fn layer_predicates(&self) -> [u8; 6] {
        self.layer_predicates
    }
}

/// INIDISP scanout state that produced one completed host surface.
///
/// This is deliberately presentation state, not Zelda's live `$2100` mirror:
/// the main thread can author the next brightness while the host is still
/// returning the field rendered with the preceding value. The typed shape is
/// exactly the raster form supported by the native renderer: one brightness,
/// an optional forced-blank prefix, and an optional forced-blank suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

impl PresentedDialogueText {
    pub const WORD_COUNT: usize = 0x3f0;

    pub fn new(words: Vec<u16>) -> Option<Self> {
        (words.len() == Self::WORD_COUNT).then_some(Self { words })
    }

    pub fn words(&self) -> &[u16] {
        &self.words
    }
}

/// Interleaved stereo samples returned by one completed host call.
///
/// This is an audio-presentation receipt, not an SPC/DSP execution snapshot.
/// The temporary Snes9x authority and a future native audio owner can publish
/// the same samples without exposing emulator registers, addresses, or phase
/// state to translated gameplay.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Result of shadowing one authoritative BG tilemap presentation with the
/// native NMI/PPU owner. Authority remains with the typed receipt until this
/// domain reports exact matches and is deliberately transferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTimingBgTilemapShadowResult {
    pub mismatched_geometry_layers: usize,
    pub compared_words: usize,
    pub mismatched_words: usize,
    pub first_mismatch: Option<(u8, usize)>,
}

/// Result of shadowing one authoritative BG-scroll presentation with the
/// native publication resolver. Authority remains with the typed receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTimingBgScrollShadowResult {
    pub compared_scanline_layers: usize,
    pub mismatched_scanline_layers: usize,
    pub first_mismatch: Option<(u16, u8)>,
}

/// Result of shadowing one authoritative Mode 7 scanout transform with the
/// native PPU/HDMA resolver. Authority remains with the typed receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTimingMode7TransformShadowResult {
    pub compared_scanline_fields: usize,
    pub mismatched_scanline_fields: usize,
    pub first_mismatch: Option<(u16, u8)>,
}

/// Result of shadowing one authoritative window-mask presentation with the
/// native HDMA publication resolver. Authority remains with the typed receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTimingWindowMaskShadowResult {
    pub compared_scanline_windows: usize,
    pub mismatched_scanline_windows: usize,
    pub mismatched_screen_masks: usize,
    pub first_mismatch: Option<(u16, u8)>,
}

/// Result of shadowing one authoritative dialogue-text publication with the
/// native NMI/PPU owner. Authority remains with the typed receipt until the
/// native owner reports exact generations and is deliberately promoted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalTimingDialogueTextShadowResult {
    pub compared_words: usize,
    pub mismatched_words: usize,
    pub first_mismatch: Option<usize>,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MainLoopInterruption {
    LinkOam,
    SpritePreparation,
    /// `NMI_PrepareSprites` completed every four-byte extended-OAM packing
    /// group above `next_group_start`, then was interrupted while computing
    /// that still-unpublished group. The original loop visits group starts
    /// `28, 24, ..., 0`; exposing that source cursor lets native gameplay
    /// resume the pure packing prefix without replaying the later countdown
    /// and DMA-source publications.
    SpritePreparationExtendedOamPacking {
        next_group_start: u8,
    },
    /// `Sprite_Main` was interrupted before its descending slot loop completed.
    /// `BeforeFirstSlot` means the common prefix returned but slot 15 did not;
    /// `AfterSlot` names the last source slot whose call returned. The timing
    /// backend may derive this from private execution state, but gameplay sees
    /// only the resumable C-loop boundary.
    SpriteMainBeforeFirstSlot,
    SpriteMainAfterSlot(u8),
    /// The active-Cucco branch completed all three source assignments in
    /// `Sprite_MoveX`, but has not entered `Sprite_MoveY`. The timing backend
    /// keeps the instruction boundary private; gameplay resumes at the next
    /// source call in `Sprite_MoveXY`.
    SpriteMainAfterActiveCuccoX {
        slot: u8,
        helper_ordinal: u8,
    },
    /// The active-Cucco branch completed `Sprite_MoveX` and published the
    /// first C assignment in `Sprite_MoveY` (`sprite_y_subpixel = t`). The
    /// matching low/high coordinate assignments and caller suffix remain
    /// pending. The timing backend keeps the instruction boundary private;
    /// gameplay sees only this resumable source statement.
    SpriteMainAfterActiveCuccoYSubpixel {
        slot: u8,
        helper_ordinal: u8,
    },
    /// `Cucco_Flee` completed its movement, Z publication, and optional
    /// velocity retarget, but has not entered `Chicken_IncrSubtype2`. The
    /// temporary timing backend keeps the source call site private; native
    /// gameplay resumes at the pending helper call.
    SpriteMainAfterCuccoFleeMovement {
        slot: u8,
        helper_ordinal: u8,
    },
    /// A Cucco's shared `Chicken_IncrSubtype2` helper published this many of
    /// its byte increments, but not the following graphics store. The helper
    /// has call sites with increments of three, four, and five; exposing the
    /// completed source statements avoids inferring a caller from a shared ROM
    /// instruction address.
    SpriteMainAfterCuccoSubtypeIncrements {
        slot: u8,
        helper_ordinal: u8,
        completed: u8,
    },
    /// A Cucco's shared `Chicken_IncrSubtype2` helper published its graphics
    /// generation, then the accepted NMI interrupted the source
    /// `Sprite_ReturnIfLifted` tail. The timing backend keeps the ROM
    /// instruction boundary private; gameplay resumes the unfinished semantic
    /// tail before advancing to the next slot.
    SpriteMainAfterCuccoGraphicsPublication {
        slot: u8,
        helper_ordinal: u8,
    },
    /// `PrepareEnemyDrop` published the big-key sprite type, then entered the
    /// synchronous graphics loader. The source slot and every lower slot are
    /// still pending; the timing backend keeps the instruction address
    /// private and exposes only this resumable C statement.
    SpriteMainBigKeyDropGraphicsStarted(u8),
    /// A sprite slot completed its ordinary prefix and entered
    /// `Link_ReceiveItem`'s synchronous graphics loader. The caller-specific
    /// item receipt owns the suspended suffix; this boundary tells native
    /// `Sprite_Main` to execute the current slot through that semantic call
    /// instead of stopping at the preceding returned slot.
    SpriteMainItemReceiptGraphicsStarted(u8),
    /// The main-loop module reached Link's movement call, but the accepted NMI
    /// preceded the first coordinate publication. The timing authority keeps
    /// the backend instruction address private; translated gameplay resumes
    /// the source call from this semantic boundary on the following host.
    LinkPositionBeforeCoordinates,
    /// `Link_MovePosition`'s per-axis loop stored the current axis' subpixel
    /// byte but the accepted NMI (or the host boundary) preceded that axis'
    /// coordinate store. `pass` is the loop's X register: 4 = z (airborne
    /// only), 2 = y, 0 = x; earlier passes are complete, later ones pending
    /// (route host 179586, Module0F's dungeon-exit spotlight close).
    LinkPositionAfterSubpixel {
        pass: u8,
    },
    /// The opening or closing iris reached its goal inside
    /// `IrisSpotlight_ConfigureTable` and entered `IrisSpotlight_ResetTable`,
    /// whose 224 table-word stores were interrupted by the accepted NMI (or
    /// the host boundary) after this many stores in source order. The module/
    /// submodule transition, ambient/music publication, and caller suffix all
    /// remain pending (route host 182709, Module10's opening iris).
    SpotlightGoalResetTable {
        completed_stores: u8,
    },
}

impl MainLoopInterruption {
    /// Whether this receipt names a resumable statement inside the shared
    /// descending `Sprite_Main` call. Keep this classification beside the
    /// receipt authority so host-timeline routing cannot omit a newly added
    /// source-level Sprite_Main checkpoint.
    pub const fn is_sprite_main(self) -> bool {
        matches!(
            self,
            Self::SpriteMainBeforeFirstSlot
                | Self::SpriteMainAfterSlot(_)
                | Self::SpriteMainAfterActiveCuccoX { .. }
                | Self::SpriteMainAfterActiveCuccoYSubpixel { .. }
                | Self::SpriteMainAfterCuccoFleeMovement { .. }
                | Self::SpriteMainAfterCuccoSubtypeIncrements { .. }
                | Self::SpriteMainAfterCuccoGraphicsPublication { .. }
                | Self::SpriteMainBigKeyDropGraphicsStarted(_)
                | Self::SpriteMainItemReceiptGraphicsStarted(_)
        )
    }
}

/// The furthest returned statement in one still-active `Sprite_Main` call.
///
/// Unlike [`MainLoopInterruption`], this receipt does not imply that an NMI is
/// currently suspending the call. It survives when an entry NMI resumes and
/// the same source call continues until the libretro host boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteMainProgress {
    BeforeFirstSlot,
    AfterSlot(u8),
    AfterActiveCuccoX {
        slot: u8,
        helper_ordinal: u8,
    },
    AfterActiveCuccoYSubpixel {
        slot: u8,
        helper_ordinal: u8,
    },
    AfterCuccoFleeMovement {
        slot: u8,
        helper_ordinal: u8,
    },
    AfterCuccoSubtypeIncrements {
        slot: u8,
        helper_ordinal: u8,
        completed: u8,
    },
    AfterCuccoGraphicsPublication {
        slot: u8,
        helper_ordinal: u8,
    },
    BigKeyDropGraphicsStarted(u8),
}

/// Source-level dialogue work completed by one host interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DialogueExecutionProgress {
    /// The interval remained inside the existing VWF call stack and did not
    /// begin a fresh `ZeldaRunGameLoop` iteration. The read position is a
    /// Zelda message-decoder endpoint: it lets the native owner prove that it
    /// has already completed the same semantic prefix without importing a CPU
    /// address, register, or raster position.
    ResumedRenderingWithoutMainIteration { message_read_position: u16 },
}

/// Source-level progress while one cached dungeon sprite temporarily occupies
/// a live sprite slot. Counts describe completed field publications in C
/// statement order; no CPU address or register provenance crosses the timing
/// authority boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CachedSpriteExecutionProgress {
    Loading { slot: u8, copied_fields: u8 },
    Restoring { slot: u8, live_fields: u8 },
}

/// Source-visible progress through one cached-sprite swap, together with the
/// hardware boundary which exposed that partial C state to the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CachedSpriteExecutionProgressReceipt {
    pub progress: CachedSpriteExecutionProgress,
    pub boundary: OriginalTimingBoundary,
}

/// Hardware boundary which made a source-level progress receipt observable.
///
/// This is deliberately a Zelda scheduling fact, not emulator provenance. A
/// native timing owner can publish the same distinction without exposing a
/// CPU program counter, register, or raster position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OriginalTimingBoundary {
    /// The host interval returned while the source call was still running;
    /// no interrupt had yet resumed that call stack.
    HostReturn,
    /// An NMI accepted after the reported source statement suspended the call
    /// stack. The translated continuation resumes on the following host.
    NmiAccepted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DungeonResetSpritesProgressReceipt {
    pub progress: DungeonResetSpritesCpuProgress,
    pub boundary: OriginalTimingBoundary,
}

/// Source-level progress through the shared `Sprite_ResetAll` routine.
///
/// The timing backend may use emulator-private call-stack evidence to identify
/// the caller, but translated gameplay receives only this C statement boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteResetAllProgress {
    /// `Sprite_DisableAll` has returned. `Sprite_ResetAll_noDisable` remains
    /// pending and must be resumed without replaying the completed disable.
    SpriteDisableAllCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpriteResetAllProgressReceipt {
    pub progress: SpriteResetAllProgress,
    pub boundary: OriginalTimingBoundary,
}

/// Completion of one source-level `Module08_OverworldLoad` stage.
///
/// The temporary oracle may recognize these boundaries from emulator state,
/// but translated gameplay receives only the C-call outcome. A native timing
/// owner can therefore replace the oracle without exposing CPU or raster data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PreOverworldStageCompletion {
    PropertiesReturned,
    OverlaysReturned,
    ScreenBuildReturned,
}

/// Source-level progress through `ZeldaRunGameLoop` during one host interval.
///
/// The temporary oracle derives this from its private execution state, but the
/// translated runtime sees only whether the C entry statement ran or an
/// interrupted source call stack continued. A native timing owner can publish
/// the same fact without emulating CPU registers or exposing a program counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MainLoopProgress {
    IterationStarted,
    CallStackContinued,
}

/// Backend-neutral execution state which may legitimately span one host-frame
/// boundary while exact original timing is active.
///
/// This is deliberately a separate, versioned sidecar rather than another
/// field in positional `ZeldaState` serialization. It contains Zelda-level
/// semantic continuations only: no CPU registers, program counters, raster
/// positions, or emulator implementation details cross the boundary.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OriginalTimingResumeCheckpoint {
    pub(crate) schema: u32,
    pub(crate) last_consumed_host_call: Option<u64>,
    pub(crate) nmi_publication_pending: bool,
    #[serde(default)]
    pub(crate) pending_nmi_update_gate: Option<NmiUpdateGate>,
    pub(crate) dungeon_exit_spotlight_entry_return_pending: bool,
    pub(crate) pre_dungeon_return_pending: Option<MainLoopProgress>,
    pub(crate) item_receipt_live_link_dma_host: Option<u32>,
}

impl OriginalTimingResumeCheckpoint {
    pub const SCHEMA: u32 = 2;

    pub const fn schema(&self) -> u32 {
        self.schema
    }
}

/// Source-level checkpoint inside one `IrisSpotlight_ConfigureTable` build or
/// its following table projection.
///
/// The temporary oracle derives these values from private CPU state; gameplay
/// receives only the resumable C statement boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpotlightTableBuildCheckpoint {
    /// The current loop iteration has not yet initialized its local circle
    /// value or published either table word. `completed_iterations` identifies
    /// the exact C loop cursor; all work for this iteration remains pending.
    BeforeIterationInitialization,
    /// The circle input has been loaded and `spotlight_var4` has been
    /// conditionally decremented, but the circle value and both table words
    /// are still pending.
    BeforeCircleCalculation { pending_circle_input: u8 },
    /// The circle value and upper table word are published. The lower table
    /// word and loop-cursor update are still pending.
    BeforeLowerTableWrite {
        lower_cursor: u16,
        circle_value: u16,
    },
    /// The row-pair build and off-screen clear are complete. This many words
    /// of the 224-word dynamic table have been copied to the reserved HDMA
    /// table; the remaining projection and caller suffix are still pending.
    ProjectionCopy { copied_words: u16 },
    /// Both table words for the current iteration are published. The source
    /// loop's completion test and, when it continues, its cursor update remain
    /// pending. The cursors are source-level loop values, not CPU registers.
    BeforeLoopCompletionTest {
        upper_cursor: u16,
        lower_cursor: u16,
    },
    /// The loop's completion test was false and the upper cursor was already
    /// incremented. The paired lower-cursor decrement is the only statement
    /// still pending before the next iteration begins.
    BeforeLowerCursorDecrement {
        upper_cursor: u16,
        lower_cursor: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpotlightTableBuildProgress {
    pub completed_iterations: u16,
    pub checkpoint: SpotlightTableBuildCheckpoint,
}

/// A spotlight table build/projection exposed at a hardware boundary while
/// the C caller remained suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpotlightTableBuildProgressReceipt {
    pub progress: SpotlightTableBuildProgress,
    pub boundary: OriginalTimingBoundary,
}

/// Source-level publication progress inside `Sprite_OverworldReloadAll_justLoad`.
///
/// The temporary oracle recognizes these statements from private CPU/WRAM
/// provenance, but gameplay receives only the Zelda semantic result. A native
/// timing owner can therefore replace it without exposing emulator state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OverworldSpriteReloadProgress {
    /// `Overworld_LoadSprites` has published the area's sprite-presence map.
    PresencePublished,
    /// One proximity record has been materialized into a regular sprite slot.
    SpriteActivated {
        block: u16,
        slot: u8,
        sprite_type: u8,
    },
    /// `Module09_LoadNewSprites` completed its source-ordered tail, including
    /// the call to `Overworld_StartScrollTransition`. The receipt deliberately
    /// omits the resulting submodule value: translated gameplay owns that C
    /// mutation, while the replaceable timing authority owns only when the
    /// suspended call returned.
    ReloadReturned,
}

/// Source-level completion state for the first save-menu text initialization.
///
/// The original C call can remain suspended across several host returns while
/// story graphics and the message buffer are prepared.  Translated gameplay
/// needs only to know whether that same C call is still running or has reached
/// its return suffix; CPU addresses, registers, and raster provenance remain
/// private to the temporary timing authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SaveMenuInitializationProgress {
    InProgress,
    Completed,
}

/// Source-level progress of the synchronous item-receipt graphics call.
///
/// The temporary timing backend may use CPU call/return evidence internally,
/// but translated gameplay sees only which semantic C caller owns the call
/// and whether that call is still suspended.  This keeps the receipt usable by
/// a future native timing authority without exposing emulator provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ItemReceiptGraphicsCaller {
    SpriteMain {
        slot: u8,
    },
    /// A sprite slot called `Link_ReceiveItem` directly. The receipt covers
    /// the nested synchronous graphics call, while the existing
    /// `SpriteMainProgress` receipt owns the surrounding descending loop.
    SpriteMainDirect {
        slot: u8,
    },
    /// `Uncle_InPassage` is suspended inside its synchronous
    /// `Link_ReceiveItem` call. The sprite slot is the only caller identity
    /// translated gameplay needs in order to resume the C suffix.
    UnclePassage {
        slot: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SourceCallProgress {
    Suspended,
    Returned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ItemReceiptGraphicsProgressReceipt {
    pub caller: ItemReceiptGraphicsCaller,
    pub progress: SourceCallProgress,
}

/// Zelda-visible controller bytes published by one completed NMI handler.
///
/// This is deliberately not the host call's raw controller input. On original
/// hardware, auto-joy refreshes `$4218` later in VBlank, so an NMI may publish
/// the preceding sample even though the host has already supplied new buttons.
/// A native input/timing owner can emit the same four semantic bytes without
/// exposing PPU counters, CPU locations, or emulator state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JoypadPublication {
    pub high: u8,
    pub low: u8,
    pub high_filtered: u8,
    pub low_filtered: u8,
}

/// Zelda's software update gate sampled when the hardware accepts an NMI.
///
/// `Interrupt_NMI_AudioParts_Locked` and the final PPU-register writes run in
/// both cases. Only an open gate runs `NMI_DoUpdates` and `NMI_ReadJoypads`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NmiUpdateGate {
    Open,
    LatchHeld,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OriginalTimingSemanticReceipt {
    NmiAccepted(NmiUpdateGate),
    /// The accepted NMI handler reached its common completion point.
    /// An open update gate has published DMA/joypad work; a held gate has run
    /// only the unconditional audio and PPU-register portions.
    /// This deliberately does not mean that the interrupted CPU context has
    /// resumed; Zelda can switch stacks and run its main thread first.
    NmiHandlerCompleted,
    JoypadPublication(JoypadPublication),
    MainLoopProgress(MainLoopProgress),
    SpriteMainProgressed(SpriteMainProgress),
    /// The active `Sprite_Main` call completed its descending loop and common
    /// suffix. A temporary backend may observe a private return address;
    /// gameplay consumes only this source-call completion fact.
    SpriteMainReturned,
    /// The active `ZeldaRunGameLoop` iteration returned
    /// through `Module_MainRouting`, `NMI_PrepareSprites`, and the `$12 = 0`
    /// suffix to Zelda's main wait before the host call ended. The temporary
    /// backend may identify the wait from private CPU state; translated
    /// gameplay consumes only this source-level call-completion fact.
    MainLoopIterationReturnedToWait,
    /// The active `ZeldaRunGameLoop` iteration completed its unconditional
    /// `NMI_PrepareSprites(); $12 = 0` suffix. Unlike
    /// `MainLoopIterationReturnedToWait`, the source CPU may immediately run
    /// another cooperative thread and accept an NMI there before the host call
    /// returns. This fact therefore owns the C suffix, not a particular wait
    /// PC or host-return location.
    MainLoopCommonSuffixCompleted,
    /// `Module_PreDungeon` returned to its caller and published the module-7
    /// landing state. This is deliberately separate from `MainLoopProgress`:
    /// the source call can return at a host boundary without beginning the
    /// following `ZeldaRunGameLoop` iteration in that same host interval.
    PreDungeonModuleReturned,
    DialogueExecutionProgress(DialogueExecutionProgress),
    SaveMenuInitializationProgress(SaveMenuInitializationProgress),
    ItemReceiptGraphicsProgress(ItemReceiptGraphicsProgressReceipt),
    /// The active dialogue returned through RenderText_Draw_Finish to Zelda's
    /// already-saved gameplay module. The receipt deliberately omits that
    /// module number: the native C state owns and applies its saved target.
    DialogueClosed,
    /// The main iteration returned from its module, but the following NMI
    /// interrupted a common caller phase before the CPU reached the main wait.
    /// The next host call must resume that phase and must not begin a fresh
    /// main iteration.
    MainLoopInterrupted(MainLoopInterruption),
    SpotlightTableBuildProgress(SpotlightTableBuildProgressReceipt),
    /// The first `Module0F_SpotlightClose` call returned from
    /// `Dungeon_PrepExitWithSpotlight`, including its completed spotlight-table
    /// projection and transition into the recurring close phase.
    DungeonExitSpotlightEntryReturned,
    /// A suspended recurring `Module0F_SpotlightClose` caller completed its
    /// spotlight-table suffix and reached Zelda's main wait before this host
    /// call returned. The temporary timing backend keeps the CPU return
    /// address private; translated gameplay consumes only this C-call
    /// completion fact.
    DungeonExitSpotlightCallerReturnedToMainWait,
    /// A suspended recurring `Module10_SpotlightOpen` goal call completed
    /// `IrisSpotlight_ConfigureTable`, restored the source-owned saved gameplay
    /// module, and returned through `OpenSpotlight_Next2`. The temporary
    /// backend keeps the restored module and CPU return address private; the
    /// native C state owns the same saved target.
    OverworldSpotlightGoalCallerReturned,
    /// `Module09_LoadNewMapAndGFX` completed `SomeTileMapChange`, publishing
    /// the rebuilt map quadrants and entering the remaining screen-map/sprite-
    /// graphics tail. The timing backend owns only this source-call boundary;
    /// translated gameplay owns the submodule mutation and map state.
    OverworldMapQuadrantsPublished,
    /// `Overworld_LoadOverlays2` completed its overlay decode and returned to
    /// Module09. The temporary backend keeps its CPU location private;
    /// translated gameplay consumes only the source-call completion fact.
    WorldMapOverlayReloadReturned,
    /// `Overworld_LoadAmbientOverlay(false)` completed its main-page Map16 to
    /// Map8 conversion and returned to Module09. The temporary backend may
    /// identify that return from private execution state; gameplay consumes
    /// only this source-call completion fact.
    WorldMapAmbientMap8Returned,
    OverworldSpriteReloadProgress(OverworldSpriteReloadProgress),
    CachedSpriteExecutionProgress(CachedSpriteExecutionProgressReceipt),
    DungeonResetSpritesProgress(DungeonResetSpritesProgressReceipt),
    SpriteResetAllProgress(SpriteResetAllProgressReceipt),
    PreOverworldStageCompleted(PreOverworldStageCompletion),
    DmaPublicationCompleted {
        channel_mask: u8,
    },
}

/// One source interruption temporarily owned by a translated C caller during
/// the current host dispatch.
///
/// This is derived from the ordered semantic receipt stream after the outer
/// host-timeline owner has consumed that stream. It is deliberately excluded
/// from serialization: only one in-memory owner may forward or consume it,
/// and a later host cannot inherit the transient boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForwardedMainLoopInterruption {
    interruption: MainLoopInterruption,
    boundary: OriginalTimingBoundary,
}

impl ForwardedMainLoopInterruption {
    pub(crate) const fn interruption(self) -> MainLoopInterruption {
        self.interruption
    }

    pub(crate) const fn boundary(self) -> OriginalTimingBoundary {
        self.boundary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForwardMainLoopInterruptionError {
    existing: ForwardedMainLoopInterruption,
}

impl ForwardMainLoopInterruptionError {
    pub(crate) const fn existing(self) -> ForwardedMainLoopInterruption {
        self.existing
    }
}

/// One timing-authority result for exactly one upcoming host call.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OriginalTimingHostReceipts {
    pub(crate) host_call: u64,
    pub(crate) input_state: u16,
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    /// One interruption temporarily forwarded by the host-timeline owner to
    /// the translated C caller during this same dispatch. The serialized
    /// oracle stream remains the ordered `NmiAccepted`/`MainLoopInterrupted`
    /// authority; this derived boundary is backend-neutral and transient.
    #[serde(skip)]
    forwarded_main_loop_interruption: Option<ForwardedMainLoopInterruption>,
    pub(crate) presented_animated_bg_tiles: Option<PresentedAnimatedBgTiles>,
    pub(crate) presented_cgram: Option<PresentedCgram>,
    pub(crate) presented_inidisp: Option<PresentedInidisp>,
    pub(crate) presented_scanout_geometry: Option<PresentedScanoutGeometry>,
    pub(crate) presented_hud_tilemap: Option<PresentedHudTilemap>,
    pub(crate) presented_dialogue_text: Option<PresentedDialogueText>,
    pub(crate) presented_bg_tilemaps: Option<PresentedBgTilemaps>,
    pub(crate) presented_bg_scroll: Option<PresentedBgScroll>,
    #[serde(default)]
    pub(crate) presented_mode7_transform: Option<PresentedMode7Transform>,
    pub(crate) presented_window_mask: Option<PresentedWindowMask>,
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
            forwarded_main_loop_interruption: None,
            presented_animated_bg_tiles: None,
            presented_cgram: None,
            presented_inidisp: None,
            presented_scanout_geometry: None,
            presented_hud_tilemap: None,
            presented_dialogue_text: None,
            presented_bg_tilemaps: None,
            presented_bg_scroll: None,
            presented_mode7_transform: None,
            presented_window_mask: None,
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

    pub fn with_presented_dialogue_text(mut self, receipt: PresentedDialogueText) -> Self {
        self.presented_dialogue_text = Some(receipt);
        self
    }

    pub fn with_presented_bg_tilemaps(mut self, receipt: PresentedBgTilemaps) -> Self {
        self.presented_bg_tilemaps = Some(receipt);
        self
    }

    pub fn with_presented_bg_scroll(mut self, receipt: PresentedBgScroll) -> Self {
        self.presented_bg_scroll = Some(receipt);
        self
    }

    pub fn with_presented_mode7_transform(mut self, receipt: PresentedMode7Transform) -> Self {
        self.presented_mode7_transform = Some(receipt);
        self
    }

    pub fn with_presented_window_mask(mut self, receipt: PresentedWindowMask) -> Self {
        self.presented_window_mask = Some(receipt);
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

    pub(crate) fn forward_main_loop_interruption(
        &mut self,
        interruption: MainLoopInterruption,
        boundary: OriginalTimingBoundary,
    ) -> Result<(), ForwardMainLoopInterruptionError> {
        if let Some(existing) = self.forwarded_main_loop_interruption {
            return Err(ForwardMainLoopInterruptionError { existing });
        }
        self.forwarded_main_loop_interruption = Some(ForwardedMainLoopInterruption {
            interruption,
            boundary,
        });
        Ok(())
    }

    pub(crate) const fn forwarded_main_loop_interruption(
        &self,
    ) -> Option<ForwardedMainLoopInterruption> {
        self.forwarded_main_loop_interruption
    }

    pub(crate) fn take_forwarded_main_loop_interruption(
        &mut self,
        expected: MainLoopInterruption,
    ) -> Option<OriginalTimingBoundary> {
        let forwarded = self.forwarded_main_loop_interruption?;
        if forwarded.interruption != expected {
            // A translated caller probing for its own phase is not the owner
            // of a differently-phased forwarded interruption; leave it for
            // its real consumer (route host 56395: the module's LinkOam
            // probe must not consume a forwarded extended-OAM-packing
            // boundary owned by the game-loop suffix). The host close still
            // fails if no owner ever consumes it.
            return None;
        }
        self.forwarded_main_loop_interruption = None;
        Some(forwarded.boundary)
    }

    pub(crate) fn discard_forwarded_main_loop_interruption(
        &mut self,
        expected: MainLoopInterruption,
    ) -> bool {
        if self
            .forwarded_main_loop_interruption
            .is_some_and(|forwarded| forwarded.interruption == expected)
        {
            self.forwarded_main_loop_interruption = None;
            true
        } else {
            false
        }
    }

    pub const fn host_call(&self) -> u64 {
        self.host_call
    }

    pub const fn input_state(&self) -> u16 {
        self.input_state
    }

    pub const fn matches_host_call(&self, host_call: u64, raw_input_state: u16) -> bool {
        self.host_call == host_call
            && self.input_state == sanitize_original_timing_input(raw_input_state)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginalTimingReceiptInstallError {
    TimingDisabled,
    ActiveHostDispatch,
    ActiveSpriteMainReturnClaim,
    ReceiptAlreadyInstalled,
    UnconsumedPresentedAudio,
    DuplicateDungeonResetProgress,
    InvalidDungeonResetProgress,
    DuplicateSpriteResetAllProgress,
    InvalidSpriteResetAllProgress,
    DuplicateCachedSpriteExecutionProgress,
    InvalidCachedSpriteExecutionProgress,
    DuplicateDialogueExecutionProgress,
    DuplicateSaveMenuInitializationProgress,
    DuplicateItemReceiptGraphicsProgress,
    InvalidItemReceiptGraphicsProgress,
    DuplicateDialogueClosed,
    DuplicateMainLoopProgress,
    DuplicateSpriteMainProgress,
    InvalidSpriteMainProgress,
    DuplicateSpriteMainReturn,
    DuplicateMainLoopIterationReturn,
    InvalidMainLoopIterationReturn,
    DuplicatePreDungeonModuleReturn,
    DuplicateMainLoopInterruption,
    InvalidMainLoopInterruption,
    DuplicateSpotlightTableBuildProgress,
    InvalidSpotlightTableBuildProgress,
    DuplicateDungeonExitSpotlightEntryReturn,
    DuplicateDungeonExitSpotlightCallerReturn,
    DuplicateOverworldSpotlightGoalCallerReturn,
    DuplicateOverworldMapQuadrantsPublished,
    DuplicateWorldMapOverlayReloadReturn,
    DuplicateWorldMapAmbientMap8Return,
    DuplicateOverworldSpritePresencePublished,
    InvalidOverworldSpriteReloadProgress,
    DuplicatePreOverworldStageCompletion,
    InvalidNmiLifecycle,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize)]
    struct UncheckedObjTiles {
        tile_word_addresses: Vec<u16>,
        tile_pixels: Vec<u8>,
    }

    fn assert_obj_tiles_deserialization_rejects(
        tile_word_addresses: Vec<u16>,
        tile_pixels: Vec<u8>,
    ) {
        let encoded = bincode::serialize(&UncheckedObjTiles {
            tile_word_addresses,
            tile_pixels,
        })
        .unwrap();
        assert!(bincode::deserialize::<PresentedObjTiles>(&encoded).is_err());
    }

    #[test]
    fn presented_obj_tiles_roundtrip_preserves_sparse_word_addresses() {
        let receipt =
            PresentedObjTiles::new(vec![0x0000, 0x5a20, 0x7ff0], vec![3; 3 * 64]).unwrap();
        let encoded = bincode::serialize(&receipt).unwrap();
        let decoded: PresentedObjTiles = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded, receipt);
    }

    #[test]
    fn presented_obj_tiles_reject_invalid_addresses_and_pixels_on_deserialization() {
        assert_obj_tiles_deserialization_rejects(vec![0x4001], vec![0; 64]);
        assert_obj_tiles_deserialization_rejects(vec![0x8000], vec![0; 64]);
        assert_obj_tiles_deserialization_rejects(vec![0x5a20, 0x5a20], vec![0; 128]);
        assert_obj_tiles_deserialization_rejects(vec![0x5a20], vec![0; 63]);
        let mut invalid_pixel = vec![0; 64];
        invalid_pixel[17] = 16;
        assert_obj_tiles_deserialization_rejects(vec![0x5a20], invalid_pixel);
    }

    #[derive(serde::Serialize)]
    struct UncheckedMode7Transform {
        scanlines: Vec<[i16; PresentedMode7Transform::FIELD_COUNT]>,
    }

    fn serialized_mode7_transform(lines: usize) -> Vec<u8> {
        bincode::serialize(&UncheckedMode7Transform {
            scanlines: vec![[0; PresentedMode7Transform::FIELD_COUNT]; lines],
        })
        .unwrap()
    }

    fn assert_mode7_transform_deserialization_rejects(invalid_lines: usize) {
        let error = bincode::deserialize::<PresentedMode7Transform>(&serialized_mode7_transform(
            invalid_lines,
        ))
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("expected exactly 224 Mode 7 scanlines"),
            "unexpected shape error for {invalid_lines} lines: {error}",
        );
    }

    #[test]
    fn presented_mode7_transform_deserialization_rejects_223_scanlines() {
        assert_mode7_transform_deserialization_rejects(223);
    }

    #[test]
    fn presented_mode7_transform_deserialization_rejects_225_scanlines() {
        assert_mode7_transform_deserialization_rejects(225);
    }

    #[test]
    fn presented_mode7_transform_roundtrip_preserves_exact_visible_height() {
        let receipt = PresentedMode7Transform::new(vec![
            [0; PresentedMode7Transform::FIELD_COUNT];
            PresentedMode7Transform::VISIBLE_LINES
        ])
        .unwrap();
        let encoded = bincode::serialize(&receipt).unwrap();
        let decoded: PresentedMode7Transform = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded, receipt);
        assert_eq!(
            decoded.scanlines().len(),
            PresentedMode7Transform::VISIBLE_LINES
        );
    }

    #[test]
    fn forwarded_main_loop_interruption_has_one_transient_owner() {
        let mut receipts = OriginalTimingHostReceipts::new(7, 0, Vec::new());
        receipts
            .forward_main_loop_interruption(
                MainLoopInterruption::LinkOam,
                OriginalTimingBoundary::HostReturn,
            )
            .unwrap();

        let error = receipts
            .forward_main_loop_interruption(
                MainLoopInterruption::SpritePreparation,
                OriginalTimingBoundary::NmiAccepted,
            )
            .unwrap_err();
        assert_eq!(
            error.existing().interruption(),
            MainLoopInterruption::LinkOam
        );
        assert_eq!(
            error.existing().boundary(),
            OriginalTimingBoundary::HostReturn
        );
        assert_eq!(
            receipts.forwarded_main_loop_interruption(),
            Some(error.existing()),
        );
        assert!(!receipts
            .discard_forwarded_main_loop_interruption(MainLoopInterruption::SpritePreparation));
        assert_eq!(
            receipts.take_forwarded_main_loop_interruption(MainLoopInterruption::LinkOam),
            Some(OriginalTimingBoundary::HostReturn),
        );
        assert_eq!(receipts.forwarded_main_loop_interruption(), None);
    }

    #[test]
    fn forwarded_main_loop_interruption_is_not_serialized_authority() {
        let mut receipts = OriginalTimingHostReceipts::new(7, 0, Vec::new());
        let serialized_without_forward = bincode::serialize(&receipts).unwrap();
        receipts
            .forward_main_loop_interruption(
                MainLoopInterruption::LinkOam,
                OriginalTimingBoundary::NmiAccepted,
            )
            .unwrap();

        let encoded = bincode::serialize(&receipts).unwrap();
        assert_eq!(encoded, serialized_without_forward);
        let decoded: OriginalTimingHostReceipts = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded.forwarded_main_loop_interruption(), None);
    }
}
