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

/// Furthest source assignment published by one `Sprite_MoveXY` invocation.
/// The helper stores each axis in subpixel, low-byte, high-byte order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteMoveXYCheckpoint {
    BeforeMovement,
    AfterXSubpixel,
    AfterXLow,
    AfterXHigh,
    AfterYSubpixel,
    AfterYLow,
    AfterYHigh,
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
    /// The current slot's shared `Sprite_TimersAndOam` prefix returned, but
    /// its state-dispatched body and every lower `Sprite_Main` slot remain
    /// pending. This is the generic source-call boundary immediately after
    /// the timer/OAM helper; no timer field or sprite type is inferred.
    SpriteMainAfterTimersAndOam(u8),
    /// The current slot completed every countdown update in
    /// `Sprite_TimersAndOam`; only that helper's final priority publication,
    /// the state-dispatched body, and lower slots remain pending.
    SpriteMainAfterTimerDecrements(u8),
    /// A state-8 Bari initializer loaded the sprite property table, promoted
    /// the slot to state 9, published its fixed Z value, and completed the
    /// room-$ce conditional before entering the RNG-backed delay assignment.
    /// The RNG call and every lower `Sprite_Main` slot remain pending.
    SpriteMainBariBeforeRandom(u8),
    /// Zelda's state-8 sprite initializer completed its generic property
    /// setup and Zelda-specific prefix, then stopped inside the shared two-
    /// sheet follower-graphics decompression. The source slot remains active;
    /// `stage` is the exact committed output prefix.
    SpriteMainFollowerGraphics {
        slot: u8,
        caller: SpriteFollowerGraphicsCaller,
        stage: RescuedMaidenInitializationStage,
    },
    /// A type-$ec throwable-scenery death slot decremented its timers and
    /// published `sprite_state[k] = 0`, then the host boundary interrupted the
    /// pending `Sprite_PrepOamCoordOrDoubleRet`/garnish suffix. The current
    /// slot has not returned to the descending `Sprite_Main` loop.
    SpriteMainAfterThrowableSceneryStateClear(u8),
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
    /// King Zora spawned the purchased flippers and entered
    /// `DecodeAnimatedSpriteTile_variable($11)`. The spawn and all of its
    /// field publications are complete, while the graphics call, the current
    /// slot return, and every lower Sprite_Main slot remain pending.
    SpriteMainKingZoraFlippersGraphicsStarted(u8),
    /// `SpriteDraw_SingleSmall` published the current slot's timer/OAM
    /// prefix, X coordinate, extended-OAM size/X bit, and visible Y
    /// coordinate. The character/flags stores, optional shadow, remainder of
    /// the sprite handler, current slot return, and every lower slot remain
    /// pending.
    SpriteMainAfterSingleSmallDrawPosition(u8),
    /// A Wallmaster carried Link past the room boundary, saved the dungeon
    /// state, disabled every sprite/ancilla/effect slot, and completed the
    /// fixed prefix of `Sprite_ResetAll_noDisable`. The large
    /// `sprite_where_in_room` clear and every later reset/caller statement
    /// remain pending.
    SpriteMainAfterWallmasterResetPrefix(u8),
    /// A sprite slot completed its ordinary prefix and entered
    /// `Link_ReceiveItem`'s synchronous graphics loader. The caller-specific
    /// item receipt owns the suspended suffix; this boundary tells native
    /// `Sprite_Main` to execute the current slot through that semantic call
    /// instead of stopping at the preceding returned slot.
    SpriteMainItemReceiptGraphicsStarted(u8),
    /// `Module0F_SpotlightClose` returned from its submodule-dispatch call,
    /// including a completed dungeon-exit spotlight entry and its submodule
    /// advance, but the accepted NMI preceded the caller's Link movement/OAM
    /// suffix. The timing backend keeps the module-router instruction address
    /// private; translated gameplay resumes the complete suffix without
    /// replaying the entry call.
    DungeonExitSpotlightAfterSubmodule,
    /// `Link_HandleVelocity` cleared actual velocity and movement deltas.
    /// `None` retains speed adjustment before the actual-velocity loop;
    /// `Some(false)` is the first pass and `Some(true)` the second pass.
    /// Vertical publication and movement remain pending in every case.
    LinkActualVelocity {
        horizontal_resolved: Option<bool>,
    },
    /// The main-loop module reached Link's movement call, but the accepted NMI
    /// preceded the first coordinate publication. The timing authority keeps
    /// the backend instruction address private; translated gameplay resumes
    /// the source call from this semantic boundary on the following host.
    LinkPositionBeforeCoordinates,
    /// `Link_MovePosition`'s per-axis loop stored the current axis' subpixel
    /// byte but the accepted NMI (or the host boundary) preceded that axis'
    /// coordinate store. `pass` is the loop's X register: 4 = z (airborne
    /// only), 2 = x, 0 = y; earlier passes are complete, later ones pending
    /// (route host 179586, Module0F's dungeon-exit spotlight close).
    LinkPositionAfterSubpixel {
        pass: u8,
    },
    /// `Link_MovePosition` published the low coordinate byte for the current
    /// axis, but the accepted NMI (or host boundary) preceded its high-byte
    /// store. The typed continuation retains the computed high byte and every
    /// later axis.
    LinkPositionAfterCoordinateLow {
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
    /// Game Over's closing iris reached radius zero and returned through the
    /// generic spotlight goal transition. The caller restored module $12 and
    /// was interrupted while filling six 16-color constant-palette rows;
    /// `completed_stores` counts the exact 16-bit stores in source order.
    GameOverIrisGoalPaletteFill {
        completed_stores: u8,
    },
    /// The current Zazak/Stalfos slot published its source-selected animation
    /// frame. Drawing, activity/recoil checks, movement, AI, and lower slots
    /// remain pending.
    SpriteMainZazakAfterGraphics(u8),
    /// The current slot completed the four leading countdown statements in
    /// `Sprite_TimersAndOam` (`delay_main` and aux1-aux3). The hit-timer,
    /// aux4, final priority publication, state-dispatched body, and lower
    /// slots remain pending.
    SpriteMainAfterPrimaryTimerDecrements(u8),
    /// The current slot completed the `delay_main` and `delay_aux1`
    /// statements in `Sprite_TimersAndOam`. The aux2/aux3, hit-timer, aux4,
    /// priority, dispatch, and lower-slot work remains pending.
    SpriteMainAfterMainAndAux1TimerDecrements(u8),
    /// A state-8 bonk item completed its generic initialization and floor
    /// publication, then entered the synchronous room-$107 animated-sheet
    /// decode. The graphics call, current slot return, and lower slots remain
    /// pending.
    SpriteMainBonkItemGraphicsStarted(u8),
    /// `Link_MovePosition` completed both coordinate-byte stores for the
    /// current axis, but the accepted NMI (or host boundary) preceded the
    /// next axis or the movement tail. `pass` names the completed loop axis:
    /// 4 = z (airborne only), 2 = x, 0 = y. Earlier axes and this axis are
    /// complete; later axes remain pending. For the final Y pass, the source
    /// may already have drained the state-neutral loop indices before the NMI
    /// (route hosts 71903 and 76632, Module0F's dungeon-exit spotlight close).
    LinkPositionAfterCoordinates {
        pass: u8,
    },
    /// A guard probe completed movement, collision/proximity checks, and its
    /// `Sprite_PrepOamCoordOrDoubleRet` call. Only the final off-screen test,
    /// current-slot return, and lower `Sprite_Main` slots remain pending
    /// (route host 76088, a newly spawned lower-slot probe).
    SpriteMainProbeAfterOamCoordinates(u8),
    /// A state-8 slot entered `SpritePrep_LoadProperties` and completed this
    /// many stores from `SpritePrep_ResetProperties`' source-ordered 40-field
    /// clear. `phase` distinguishes the initial property load from a nested
    /// source call made by the initializer after earlier mutations have
    /// already committed. The remaining reset stores and caller suffix remain
    /// pending (route hosts 80273 and 126785).
    SpriteMainInitializeResetProperties {
        slot: u8,
        phase: SpriteInitializeResetPropertiesPhase,
        completed_stores: u8,
    },
    /// A state-8 initializer completed its current 40-store reset and this
    /// many of `SpritePrep_LoadProperties`' ten following source stores.
    SpriteMainInitializeLoadProperties {
        slot: u8,
        phase: SpriteInitializeResetPropertiesPhase,
        completed_stores: u8,
    },
    /// Fire Debirando completed both property loads, converted `$64` to `$63`,
    /// and published the fixed pit-initialization prefix. It has entered
    /// `Sprite_SpawnDynamically`, but no free-slot mutation has committed.
    SpriteMainFireDebirandoBeforeSpawn(u8),
    /// Fire Debirando entered `Sprite_SpawnDynamically`, selected a free
    /// sprite slot, and completed the named source mutation within that
    /// helper. The remaining helper statements, caller suffix, current-slot
    /// return, and lower `Sprite_Main` slots remain pending.
    SpriteMainFireDebirandoSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: SpriteDynamicSpawnProgress,
    },
    /// The dying Trinexx body's `Sprite_MakeBossDeathExplosion_NoSound`
    /// entered `Sprite_SpawnDynamically`, selected `spawned_slot`, and
    /// completed the named source mutation. The explosion's own setup, the
    /// head-direction store, the current-slot return and lower `Sprite_Main`
    /// slots remain pending.
    SpriteMainTrinexxDeathExplosionSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: SpriteDynamicSpawnProgress,
    },
    /// `Sprite_Trinexx_FinalPhase` state 0 entered `Sprite_CheckTileCollision`.
    /// With `probes_completed` the wall-collision byte was cleared and the
    /// single-layer direction probes ran; the tile-property probe, its
    /// consumers and the direction/velocity update remain pending.
    SpriteMainTrinexxFinalPhaseTileCollision {
        slot: u8,
        probes_completed: bool,
    },
    /// `HelmasaurHardHatBeetleCommon` (Helmasaur $13 / Hardhat $26) stopped
    /// inside its `Sprite_CheckTileCollision` call at `stage`; the movement
    /// before it has run, the speed targeting after it is pending.
    SpriteMainHelmasaurHardHatTileCollision {
        slot: u8,
        stage: SpriteTileCollisionStage,
    },
    /// `Lanmola_Draw` stopped inside its graphics/history prologue after
    /// `completed_stores` of its five stores (graphics angle, then the trail
    /// direction, z, y and x bytes); the subtype2 increment, the segment
    /// drawing and the Lanmola body remain pending.
    SpriteMainLanmolaDrawPrefix {
        slot: u8,
        completed_stores: u8,
    },
    /// `Sprite_TrinexxD_Draw` (from `Sprite_Trinexx_FinalPhase`) stopped in
    /// body segment `segment` after `stage` of its eight per-segment steps
    /// (damage check, OAM pointer, OAM ext pointer, OAM flags, flashing
    /// check, graphics, head graphics, draw). `segment == anim_clock` with
    /// stage 0 is the finished loop with its scratch store pending.
    SpriteMainTrinexxFinalPhaseDraw {
        slot: u8,
        segment: u8,
        stage: u8,
    },
    /// `SpriteDraw_Antfairy` published its leading subtype2 increment, but
    /// the animation/draw suffix, caller-specific active body, current-slot
    /// return, and lower `Sprite_Main` slots remain pending.
    SpriteMainAfterAntfairySubtype2Increment(u8),
    /// `Lanmola_Draw` published its graphics/history prefix and leading
    /// subtype2 increment. The remaining draw, Lanmola AI body, current-slot
    /// return, and lower `Sprite_Main` slots remain pending.
    SpriteMainAfterLanmolaSubtype2Increment(u8),
    /// `DesertPrayer_BuildIrisHDMATable` stopped at an exact source statement.
    /// The progress value names every persistent setup/table mutation already
    /// published by the interrupted call; the rest of the builder and its
    /// Module0E caller remain pending.
    DesertPrayerIris {
        source_subsubmodule: u8,
        palette_countdown: u8,
        radius: u16,
        progress: DesertPrayerIrisProgress,
    },
    /// `ApplyPaletteFilter` in Desert Prayer state 3 completed every source
    /// range before `next_color`, then stopped before loading that color. The
    /// remaining palette stores, filter-state transition, iris build, and
    /// Module0E caller suffix remain pending.
    DesertPrayerPaletteFilterBeforeColor {
        countdown: u8,
        next_color: u8,
    },
    /// Wish Pond case 2 removed the selected inventory item, spawned its
    /// tossed-item ancilla, and entered that helper's synchronous animated-
    /// sheet decode. The graphics call, helper/case suffix, current slot, and
    /// lower `Sprite_Main` slots remain pending.
    SpriteMainWishPondTossedItemGraphicsStarted(u8),
    /// A state-8 standard guard completed property initialization and entered
    /// the first of `SpritePrep_TrooperAndArcherSoldier`'s two nested
    /// `SpriteActive_Main` calls. The guard draw has published both weapon
    /// entries through the final entry's character byte; its flags/extended
    /// OAM fields, the rest of the first active call, the second active call,
    /// and the initializer/slot-loop suffix remain pending.
    SpriteMainGuardPrepWeaponFlagsPending(u8),
    /// A state-8 Mini Moldorm completed generic property initialization and
    /// this many of its 32 history entries' four byte stores. Stores are
    /// counted in ROM order: Y low, Y high, X low, X high.
    SpriteMainMiniMoldormHistory {
        slot: u8,
        completed_stores: u8,
    },
    /// `HelmasaurHardHatBeetleCommon` returned from its inactive check and
    /// published the shared subtype2 increment. Its recoil check, movement /
    /// collision body, current-slot return, and lower Sprite_Main slots remain
    /// pending.
    SpriteMainAfterHelmasaurHardHatBeetleSubtype2Increment(u8),
    /// A type-$62 master-sword light beam completed its draw and reached the
    /// named assignment in `Sprite_MoveXY`. The remainder of movement, its
    /// frame-gated caller suffix, and all lower Sprite_Main slots are pending.
    SpriteMainMasterSwordLightBeamMovement {
        slot: u8,
        checkpoint: SpriteMoveXYCheckpoint,
    },
    /// An indoor Boulder ($C2) published its OAM flags, ran `Sprite_MoveZ`,
    /// and reached the named `Sprite_MoveXY` assignment; the remaining
    /// movement and its frame-gated damage/tile-collision suffix are pending.
    SpriteMainBoulderMovement {
        slot: u8,
        checkpoint: SpriteMoveXYCheckpoint,
    },
    /// A type-$62 master-sword light beam completed movement, entered its
    /// frame-gated replacement spawn, selected `spawned_slot`, and published
    /// the named source mutation. The remainder of the shared dynamic-spawn
    /// helper, caller suffix, and lower Sprite_Main slots are pending.
    SpriteMainMasterSwordLightBeamSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: SpriteDynamicSpawnProgress,
    },
    /// A nested guard initializer call completed drawing and parry hitbox
    /// setup. Damage checks and the remaining active calls are pending.
    SpriteMainGuardPrepParryHitbox {
        slot: u8,
        active_call: u8,
    },
    /// An active guard's weapon entry published its coordinate words while
    /// its temporary animation pose remains live in the drawing caller.
    SpriteMainGuardAnimation {
        slot: u8,
        checkpoint: GuardAnimationCheckpoint,
    },
    /// Hog Spear Man completed its active body and both subtype increments;
    /// the shared animation helper's graphics store remains pending.
    SpriteMainHogSpearBodyGraphicsPending(u8),
    /// A nested guard initializer call reached the patrol delay branch;
    /// movement is published and the patrol/initializer suffix is pending.
    SpriteMainGuardPrepPatrolDelay {
        slot: u8,
        active_call: u8,
    },
    /// Generic properties and state promotion completed; type-specific prep
    /// has not begun in the initializer's jump-table caller.
    SpriteMainInitializePrepPending(u8),
    SpriteMainGuardPrepTileCollisionReturned {
        slot: u8,
        active_call: u8,
    },
    SpriteMainAbsorbableHorizontalTileLookup(u8),
    /// Descending reset stores completed before the suspended Wallmaster caller.
    SpriteMainWallmasterResetClear {
        slot: u8,
        cleared_bytes: u16,
    },
    SpriteMainAfterHitTimer(u8),
    SpriteMainPengatorSlidePending(u8),
    SpriteMainAntifairyBouncePending(u8),
    SpriteMainKholdstareDamagePending(u8),
    SpriteMainAfterMainTimerDecrement(u8),
    SpriteMainAfterZeroHitTimerClear(u8),
    /// Both actual velocity components are stored; airborne defaults and
    /// position integration remain pending in Link_HandleVelocity.
    LinkActualVelocityCompleted,
    /// Recurring Module0F finished its iris table and radius update, before
    /// the control clears and Link/OAM caller suffix.
    DungeonExitSpotlightTableCompleted,
    SpriteMainAbsorbableVerticalTileLookup(u8),
    SpriteMainAbsorbableVerticalTileAttributeLoaded(u8),
    SpriteMainSwamolaHeadDraw(u8),
    SpriteMainSwamolaHeadDrawCompleted(u8),
    SpriteMainSwamolaSegmentDraw {
        slot: u8,
        segment: u8,
    },
    SpriteMainVitreousDamagePending(u8),
    SpriteMainVitreousAiPending(u8),
    SpriteMainVitreousPlayerDamagePending(u8),
    SpriteMainMoblinCollisionGeometry(u8),
    SpriteMainMoblinAttributeLoaded(u8),
    SpriteMainHappinessPondRupeeGraphicsStarted(u8),
    SpriteMainMiniMoldormAiPending(u8),
    LinkVelocityClearProgress {
        completed: u8,
    },
    SpriteMainBuzzblobAfterXSubpixel(u8),
    SpriteMainCatfishMedallionGraphicsStarted(u8),
    SpriteMainTrinexxHeadDrawSetup(u8),
    SpriteMainWaterfallGtCutsceneGraphicsStarted(u8),
    SpriteMainTrinexxBreathTileCollisionReturned(u8),
    SpriteMainTrinexxHeadDraw {
        slot: u8,
        segment: u8,
    },
    /// Sprite_Sidenexx's state-2 neck-target loop ($1D:BA07..BA53) or its
    /// completion test. `step` counts the loop's compare/count steps
    /// (`6 * segment + 2 * pass + counted`), 54 is the loop done with the
    /// change count pending and 55 the state store done with its random
    /// delay pending.
    SpriteMainSidenexxNeckTargetLoop {
        slot: u8,
        step: u8,
    },
    SpriteMainTrinexxHeadFrontPart {
        slot: u8,
        completed_stores: u8,
    },
}

/// Persistent source progress within `DesertPrayer_BuildIrisHDMATable`.
/// Scratch-register-only instruction boundaries collapse to the nearest C
/// statement: only writes which can survive the interrupt are represented.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DesertPrayerIrisProgress {
    /// Setup writes are ordered as lower Y, upper Y, X center, radial cursor.
    Setup { completed_writes: u8 },
    /// The loop is about to calculate one scanline. All prior iterations are
    /// complete and `scanline` is the source's direct-page `$04` value.
    BeforeIteration { scanline: u16 },
    /// The current scanline has been calculated but its primary table word has
    /// not been published yet.
    BeforePrimaryTableWrite { table_word: u16, y_buffer: u8 },
    /// The primary table statement is complete; the optional mirrored write,
    /// radial-cursor advance, and loop test remain pending.
    AfterPrimaryTableWrite { table_word: u16, y_buffer: u8 },
    /// Both table statements and cursor advances for the preceding iteration
    /// are complete. `next_scanline` is the next `$04` value.
    AfterIteration { next_scanline: u16, y_buffer: u8 },
    /// The scanline loop is complete, before the state-4 input/radius tail.
    LoopComplete,
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
                | Self::SpriteMainAfterTimersAndOam(_)
                | Self::SpriteMainAfterTimerDecrements(_)
                | Self::SpriteMainBariBeforeRandom(_)
                | Self::SpriteMainFollowerGraphics { .. }
                | Self::SpriteMainAfterThrowableSceneryStateClear(_)
                | Self::SpriteMainAfterActiveCuccoX { .. }
                | Self::SpriteMainAfterActiveCuccoYSubpixel { .. }
                | Self::SpriteMainAfterCuccoFleeMovement { .. }
                | Self::SpriteMainAfterCuccoSubtypeIncrements { .. }
                | Self::SpriteMainAfterCuccoGraphicsPublication { .. }
                | Self::SpriteMainBigKeyDropGraphicsStarted(_)
                | Self::SpriteMainKingZoraFlippersGraphicsStarted(_)
                | Self::SpriteMainCatfishMedallionGraphicsStarted(_)
                | Self::SpriteMainTrinexxHeadDrawSetup(_)
                | Self::SpriteMainWaterfallGtCutsceneGraphicsStarted(_)
                | Self::SpriteMainTrinexxBreathTileCollisionReturned(_)
                | Self::SpriteMainHappinessPondRupeeGraphicsStarted(_)
                | Self::SpriteMainAfterSingleSmallDrawPosition(_)
                | Self::SpriteMainAfterWallmasterResetPrefix(_)
                | Self::SpriteMainWallmasterResetClear { .. }
                | Self::SpriteMainZazakAfterGraphics(_)
                | Self::SpriteMainItemReceiptGraphicsStarted(_)
                | Self::SpriteMainAfterPrimaryTimerDecrements(_)
                | Self::SpriteMainAfterHitTimer(_)
                | Self::SpriteMainAfterMainAndAux1TimerDecrements(_)
                | Self::SpriteMainAfterMainTimerDecrement(_)
                | Self::SpriteMainAfterZeroHitTimerClear(_)
                | Self::SpriteMainBonkItemGraphicsStarted(_)
                | Self::SpriteMainProbeAfterOamCoordinates(_)
                | Self::SpriteMainInitializeResetProperties { .. }
                | Self::SpriteMainInitializeLoadProperties { .. }
                | Self::SpriteMainFireDebirandoBeforeSpawn(_)
                | Self::SpriteMainFireDebirandoSpawn { .. }
                | Self::SpriteMainTrinexxDeathExplosionSpawn { .. }
                | Self::SpriteMainTrinexxFinalPhaseTileCollision { .. }
                | Self::SpriteMainHelmasaurHardHatTileCollision { .. }
                | Self::SpriteMainLanmolaDrawPrefix { .. }
                | Self::SpriteMainTrinexxFinalPhaseDraw { .. }
                | Self::SpriteMainAfterAntfairySubtype2Increment(_)
                | Self::SpriteMainAfterLanmolaSubtype2Increment(_)
                | Self::SpriteMainAfterHelmasaurHardHatBeetleSubtype2Increment(_)
                | Self::SpriteMainBuzzblobAfterXSubpixel(_)
                | Self::SpriteMainHogSpearBodyGraphicsPending(_)
                | Self::SpriteMainAbsorbableHorizontalTileLookup(_)
                | Self::SpriteMainAbsorbableVerticalTileLookup(_)
                | Self::SpriteMainAbsorbableVerticalTileAttributeLoaded(_)
                | Self::SpriteMainSwamolaHeadDraw(_)
                | Self::SpriteMainSwamolaHeadDrawCompleted(_)
                | Self::SpriteMainMoblinAttributeLoaded(_)
                | Self::SpriteMainMoblinCollisionGeometry(_)
                | Self::SpriteMainVitreousDamagePending(_)
                | Self::SpriteMainVitreousAiPending(_)
                | Self::SpriteMainMiniMoldormAiPending(_)
                | Self::SpriteMainVitreousPlayerDamagePending(_)
                | Self::SpriteMainSwamolaSegmentDraw { .. }
                | Self::SpriteMainTrinexxHeadDraw { .. }
                | Self::SpriteMainSidenexxNeckTargetLoop { .. }
                | Self::SpriteMainTrinexxHeadFrontPart { .. }
                | Self::SpriteMainPengatorSlidePending(_)
                | Self::SpriteMainAntifairyBouncePending(_)
                | Self::SpriteMainKholdstareDamagePending(_)
                | Self::SpriteMainInitializePrepPending(_)
                | Self::SpriteMainGuardPrepPatrolDelay { .. }
                | Self::SpriteMainGuardPrepTileCollisionReturned { .. }
                | Self::SpriteMainWishPondTossedItemGraphicsStarted(_)
                | Self::SpriteMainGuardPrepWeaponFlagsPending(_)
                | Self::SpriteMainGuardPrepParryHitbox { .. }
                | Self::SpriteMainGuardAnimation { .. }
                | Self::SpriteMainMiniMoldormHistory { .. }
                | Self::SpriteMainMasterSwordLightBeamMovement { .. }
                | Self::SpriteMainBoulderMovement { .. }
                | Self::SpriteMainMasterSwordLightBeamSpawn { .. }
        )
    }
}

/// The furthest completed source statement in one still-active `Sprite_Main` call.
///
/// Unlike [`MainLoopInterruption`], this receipt does not imply that an NMI is
/// currently suspending the call. It survives when an entry NMI resumes and
/// the same source call continues until the libretro host boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteMainProgress {
    BeforeFirstSlot,
    AfterSlot(u8),
    AfterTimersAndOam(u8),
    AfterTimerDecrements(u8),
    BariBeforeRandom(u8),
    FollowerGraphics {
        slot: u8,
        caller: SpriteFollowerGraphicsCaller,
        stage: RescuedMaidenInitializationStage,
    },
    AfterThrowableSceneryStateClear(u8),
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
    KingZoraFlippersGraphicsStarted(u8),
    AfterSingleSmallDrawPosition(u8),
    AfterWallmasterResetPrefix(u8),
    /// Keep new checkpoint variants at the end: progress receipts are
    /// checkpoint-serialized.
    ZazakAfterGraphics(u8),
    /// The four leading `Sprite_TimersAndOam` countdown statements returned;
    /// hit-timer handling and every later statement remain pending.
    AfterPrimaryTimerDecrements(u8),
    /// The `delay_main` and `delay_aux1` countdown statements returned;
    /// aux2/aux3 and every later statement remain pending.
    AfterMainAndAux1TimerDecrements(u8),
    /// A state-8 bonk item completed its generic initialization and floor
    /// publication, then entered the synchronous room-$107 animated-sheet
    /// decode. The graphics call, current slot return, and lower slots remain
    /// pending.
    BonkItemGraphicsStarted(u8),
    /// A guard probe completed movement, collision/proximity checks, and its
    /// OAM-coordinate preparation. The final off-screen test and the rest of
    /// the descending slot loop remain pending.
    ProbeAfterOamCoordinates(u8),
    /// A state-8 slot completed a source-ordered prefix of the shared
    /// `SpritePrep_ResetProperties` clear.
    InitializeResetProperties {
        slot: u8,
        phase: SpriteInitializeResetPropertiesPhase,
        completed_stores: u8,
    },
    /// The reset returned and this many of the shared property loader's ten
    /// source stores committed.
    InitializeLoadProperties {
        slot: u8,
        phase: SpriteInitializeResetPropertiesPhase,
        completed_stores: u8,
    },
    /// Fire Debirando entered its dynamic-spawn call after publishing the
    /// initializer prefix, before the callee mutated a free sprite slot.
    FireDebirandoBeforeSpawn(u8),
    /// Fire Debirando's dynamic-spawn helper selected `spawned_slot` and
    /// published the source statements named by `progress`.
    FireDebirandoSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: SpriteDynamicSpawnProgress,
    },
    /// The dying Trinexx body's explosion spawn selected `spawned_slot` and
    /// published the source statements named by `progress`.
    TrinexxDeathExplosionSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: SpriteDynamicSpawnProgress,
    },
    TrinexxFinalPhaseTileCollision {
        slot: u8,
        probes_completed: bool,
    },
    HelmasaurHardHatTileCollision {
        slot: u8,
        stage: SpriteTileCollisionStage,
    },
    LanmolaDrawPrefix {
        slot: u8,
        completed_stores: u8,
    },
    TrinexxFinalPhaseDraw {
        slot: u8,
        segment: u8,
        stage: u8,
    },
    /// The current active slot completed `SpriteDraw_Antfairy`'s leading
    /// subtype2 increment. Its draw/body suffix and lower slots are pending.
    AfterAntfairySubtype2Increment(u8),
    /// The current Lanmola slot completed the source prefix through its
    /// subtype2 increment. Its remaining draw/body suffix and lower slots are
    /// pending.
    AfterLanmolaSubtype2Increment(u8),
    /// Wish Pond case 2 is suspended inside the tossed-item ancilla's
    /// animated-sheet decoder after all item-removal and spawn-prefix writes.
    WishPondTossedItemGraphicsStarted(u8),
    /// The first nested active call in a state-8 standard-guard initializer
    /// published the final weapon entry through its character byte. Its flags
    /// and extended-OAM stores are the next source mutations.
    GuardPrepWeaponFlagsPending(u8),
    /// A state-8 Mini Moldorm completed generic property initialization and
    /// this many of its 128 source-ordered history byte stores.
    MiniMoldormHistory {
        slot: u8,
        completed_stores: u8,
    },
    /// The shared Mini Helmasaur / Hardhat Beetle body completed its inactive
    /// check and subtype2 increment. The rest of that body and lower slots are
    /// pending.
    AfterHelmasaurHardHatBeetleSubtype2Increment(u8),
    /// Keep new checkpoint variants at the end: progress receipts are
    /// checkpoint-serialized.
    MasterSwordLightBeamMovement {
        slot: u8,
        checkpoint: SpriteMoveXYCheckpoint,
    },
    BoulderMovement {
        slot: u8,
        checkpoint: SpriteMoveXYCheckpoint,
    },
    /// The current master-sword light beam entered its frame-gated
    /// replacement spawn and published the named shared-helper mutation.
    MasterSwordLightBeamSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: SpriteDynamicSpawnProgress,
    },
    /// One of the initializer's two active calls reached parry hitbox setup.
    GuardPrepParryHitbox {
        slot: u8,
        active_call: u8,
    },
    GuardAnimation {
        slot: u8,
        checkpoint: GuardAnimationCheckpoint,
    },
    HogSpearBodyGraphicsPending(u8),
    GuardPrepPatrolDelay {
        slot: u8,
        active_call: u8,
    },
    InitializePrepPending(u8),
    GuardPrepTileCollisionReturned {
        slot: u8,
        active_call: u8,
    },
    AbsorbableHorizontalTileLookup(u8),
    WallmasterResetClear {
        slot: u8,
        cleared_bytes: u16,
    },
    AfterHitTimer(u8),
    PengatorSlidePending(u8),
    AntifairyBouncePending(u8),
    KholdstareDamagePending(u8),
    AfterMainTimerDecrement(u8),
    AfterZeroHitTimerClear(u8),
    AbsorbableVerticalTileLookup(u8),
    AbsorbableVerticalTileAttributeLoaded(u8),
    SwamolaHeadDraw(u8),
    SwamolaHeadDrawCompleted(u8),
    SwamolaSegmentDraw {
        slot: u8,
        segment: u8,
    },
    VitreousDamagePending(u8),
    VitreousAiPending(u8),
    VitreousPlayerDamagePending(u8),
    MoblinCollisionGeometry(u8),
    MoblinAttributeLoaded(u8),
    HappinessPondRupeeGraphicsStarted(u8),
    MiniMoldormAiPending(u8),
    BuzzblobAfterXSubpixel(u8),
    CatfishMedallionGraphicsStarted(u8),
    TrinexxHeadDrawSetup(u8),
    WaterfallGtCutsceneGraphicsStarted(u8),
    TrinexxBreathTileCollisionReturned(u8),
    TrinexxHeadDraw {
        slot: u8,
        segment: u8,
    },
    SidenexxNeckTargetLoop {
        slot: u8,
        step: u8,
    },
    TrinexxHeadFrontPart {
        slot: u8,
        completed_stores: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuardAnimationCheckpoint {
    HeadCharacterPending,
    HeadFlagsPending,
    WeaponCoordinates {
        entry: u8,
    },
    BodyBeforeEntry {
        entry: u8,
    },
    BodyCoordinates {
        entry: u8,
    },
    BodyFlagsPending {
        entry: u8,
    },
    WeaponBeforeCoordinates {
        entry: u8,
    },
    /// Guard_Animate returned; the caller has not restored its saved pose.
    DrawReturned,
    HeadExtendedPending,
    /// The state-8 trooper initializer is inside a nested Hog Spear draw,
    /// after its last body OAM entry and before its weapon draw.
    HogSpearInitializerBodyReturned {
        active_call: u8,
    },
}

impl GuardAnimationCheckpoint {
    pub const fn is_valid(self) -> bool {
        matches!(
            self,
            Self::HeadCharacterPending
                | Self::DrawReturned
                | Self::HeadExtendedPending
                | Self::HeadFlagsPending
                | Self::HogSpearInitializerBodyReturned { active_call: 1 | 2 }
                | Self::WeaponCoordinates { entry: 0 | 1 }
                | Self::WeaponBeforeCoordinates { entry: 0 | 1 }
                | Self::BodyBeforeEntry { entry: 0..=3 }
                | Self::BodyCoordinates { entry: 0..=3 }
                | Self::BodyFlagsPending { entry: 0..=3 }
        )
    }
}

/// Source call site owning an interrupted `SpritePrep_ResetProperties`.
///
/// A state-8 Fire Debirando performs one ordinary property load, increments
/// its state, converts type `$64` to `$63`, and then performs a second nested
/// property load. Both calls share the same reset helper and return address,
/// so the call site is part of the semantic checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteInitializeResetPropertiesPhase {
    InitialPropertyLoad,
    FireDebirandoTypeConversion,
}

/// Furthest persistent source mutation completed by
/// `Sprite_SpawnDynamicallyEx` after it selects a free slot.
///
/// Coordinate capture is scratch-only until the caller consumes the returned
/// `SpriteSpawnInfo`, so it deliberately does not introduce a checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteDynamicSpawnProgress {
    TypePublished,
    StatePublished,
    ResetProperties { completed_stores: u8 },
    LoadProperties { completed_stores: u8 },
    IdentityPublished,
    FloorPublished,
    DirectionPublished,
    DieActionCleared,
    SubtypeCleared,
}

/// Where a host boundary fell inside `Sprite_CheckTileCollision` on the
/// single-layer path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteTileCollisionStage {
    /// The wall-collision clear is still pending.
    Entered,
    /// The wall-collision byte is cleared and no direction probe finished.
    Cleared,
    /// The vertical probe (when the vertical velocity is nonzero) finished;
    /// the horizontal probe or the tile-property probe is pending.
    VerticalProbeDone,
    /// Both direction probes finished; the `$68` property probe is pending.
    ProbesCompleted,
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
    /// The interval returned from inside the current `VWF_RenderSingle` call
    /// after its function prefix had committed, but before the decoder cursor
    /// advanced. In particular, the optional glyph click and line-transition
    /// state already belong to the source call and must not be replayed when
    /// the translated continuation resumes the drawing body.
    ResumedRenderingWithCurrentGlyphStarted { message_read_position: u16 },
}

impl DialogueExecutionProgress {
    pub const fn message_read_position(self) -> u16 {
        match self {
            Self::ResumedRenderingWithoutMainIteration {
                message_read_position,
            }
            | Self::ResumedRenderingWithCurrentGlyphStarted {
                message_read_position,
            } => message_read_position,
        }
    }

    pub const fn current_glyph_started(self) -> bool {
        matches!(self, Self::ResumedRenderingWithCurrentGlyphStarted { .. })
    }
}

/// Source-level progress while one cached dungeon sprite temporarily occupies
/// a live sprite slot. Counts describe completed field publications in C
/// statement order; no CPU address or register provenance crosses the timing
/// authority boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CachedSpriteExecutionProgress {
    Loading {
        slot: u8,
        copied_fields: u8,
    },
    Executing {
        slot: u8,
        progress: CachedSpriteExecutionBodyProgress,
    },
    Restoring {
        slot: u8,
        live_fields: u8,
    },
}

/// Source statements completed by the cached sprite handler after the live
/// slot swap and before the displaced sprite is restored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CachedSpriteExecutionBodyProgress {
    AfterAntfairySubtype2Increment,
}

/// Source-order cursor inside `Dungeon_FlipCrystalPegAttribute`. The ROM
/// visits four 0x800-byte attribute banks at each descending `index` before
/// decrementing it; `completed_banks` names the completed prefix at the
/// current index. `index == 0xffff` is the literal exhausted X-register state
/// after the final DEX, with no current bank prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DungeonPegAttributeFlipProgressReceipt {
    pub index: u16,
    pub completed_banks: u8,
    pub boundary: OriginalTimingBoundary,
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

/// Source-visible control-state publications made while
/// `Module11_02_LoadEntrance` remains suspended inside its long room load.
///
/// These are statement boundaries, not elapsed-frame estimates. A temporary
/// CPU-backed oracle may identify the corresponding stores from private
/// execution state, while translated gameplay receives only their semantic
/// effect and can therefore use any timing backend which publishes the same
/// source facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DungeonFallingEntranceProgress {
    /// `Dungeon_LoadAndDrawRoom` completed the room-parser tail which clears
    /// `subsubmodule_index` before the caller restores its saved phase.
    RoomParserClearedSubsubmodule,
    /// `Module11_02_LoadEntrance` restored and advanced its saved
    /// `subsubmodule_index` after the room draw returned.
    RoomLoadAdvancedSubsubmodule,
    /// The caller published `submodule_index = 7`; only the following dungeon
    /// song-bank transfer remains inside the suspended source call.
    SongBankTailEntered,
}

/// Source-level progress through the rescued-maiden room-tilemap clear.
///
/// The assembly clears eight 1,024-word regions for each even X cursor: four
/// BG2 quadrants followed by four BG1 quadrants.  `completed_stores` is the
/// exact prefix of that 8,192-store source order which committed before the
/// boundary.  It deliberately exposes neither a CPU address nor a raster
/// position, so any timing backend can publish the same C-visible state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RescuedMaidenTilemapClearProgressReceipt {
    pub completed_stores: u16,
    pub boundary: OriginalTimingBoundary,
}

/// Source-level progress through the two sprite-sheet decompressions inside
/// `CrystalCutscene_SpawnMaiden`'s synchronous `LoadFollowerGraphics` call.
///
/// Each sheet expands to exactly 1,536 bytes. The cursor names the committed
/// output prefix at a host return; it exposes neither the decompressor's CPU
/// registers nor its private program counter. The translated caller can
/// therefore preserve the exact partially-written scratch buffers while the
/// source call stack remains suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RescuedMaidenInitializationStage {
    FirstFollowerSheet {
        completed_bytes: u16,
    },
    SecondFollowerSheet {
        completed_bytes: u16,
    },
    /// Both sheets are complete. `completed_stores` counts the exact prefix
    /// of the 512 source-order 16-bit stores which expand 32 tiles from 3bpp
    /// into the shared 4bpp buffer.
    Conversion {
        completed_stores: u16,
    },
}

/// The sprite initializer which owns a suspended shared follower-graphics
/// load.  The loader itself is common, but each caller has a distinct prefix
/// and suffix which translated gameplay must resume exactly once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteFollowerGraphicsCaller {
    /// `SpritePrep_BlindMaiden`'s state-8 initialization call.
    BlindMaiden,
    Zelda,
    /// `Sprite_B7_BlindMaiden`'s state-9 transition into a follower.
    /// Keep new variants at the end: these receipts are checkpoint-serialized.
    BlindMaidenBody,
    /// `SpritePrep_OldMan`'s state-8 initialization call.
    OldMan,
    /// The purple chest's state-9 transition into follower 12.
    PurpleChest,
    /// Bomb Shop purchase, after payment and before becoming follower 13.
    SuperBomb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RescuedMaidenInitializationProgressReceipt {
    pub stage: RescuedMaidenInitializationStage,
    pub boundary: OriginalTimingBoundary,
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
    /// The upper table word is published, but the source has not yet reached
    /// the guarded lower-word store. `lower_cursor` identifies the current C
    /// loop iteration; native gameplay recomputes the pure circle value from
    /// its matching loop state instead of importing a CPU accumulator.
    AfterUpperTableWrite { lower_cursor: u16 },
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

/// Source-level progress through Module19 case 2's special-area palette load.
///
/// The song upload can return late enough in a host for the caller to enter
/// `Overworld_EnterSpecialArea` and publish only a prefix of the OWBG2 palette
/// before the host boundary. The temporary timing backend derives the count
/// from private CPU state; translated gameplay receives only the number of
/// source-order palette words already committed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TriforceRoomCase2PaletteProgressReceipt {
    pub completed_ow_bg2_words: u8,
    pub boundary: OriginalTimingBoundary,
}

/// Source-level progress through a blocking Module1A credits scene load.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CreditsSceneLoadProgress {
    /// The scene loader published its module/submodule advance and returned
    /// to `Credits_LoadNextScene_*`, before the ending text append.
    SceneLoadCompleted,
    /// The ending text append wrote its header and this many payload bytes.
    EndingTextPayloadBytes(u16),
    /// The scene loader and ending text append returned; only the shared
    /// ZeldaRunGameLoop suffix remains suspended.
    EndingTextCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreditsSceneLoadProgressReceipt {
    pub progress: CreditsSceneLoadProgress,
    pub boundary: OriginalTimingBoundary,
}

/// Source-level progress through the credits finale's `SaveGameFile` call.
///
/// Both SRAM save-block mirrors have been copied before this loop begins.
/// `completed_checksum_words` identifies the live save-block words already
/// accumulated by the checksum loop when the host boundary exposed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreditsEndSequence32ProgressReceipt {
    pub completed_checksum_words: u16,
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
    /// `Sprite_ActivateAllProxima` returned to the host while its temporary
    /// horizontal scan coordinate was still live in `$E2`. The value is the
    /// source-visible scratch coordinate at that boundary, not a display
    /// prediction.
    ProximityScanSuspended { bg2_h: u16 },
    /// `Sprite_ReloadAll_Overworld` returned to its concrete
    /// `Overworld_LoadOverlays` caller. The remainder of the overlay load is
    /// still executing; this receipt exposes only the source call boundary.
    GenerationReturned,
    /// `Module09_LoadNewSprites` completed its source-ordered tail, including
    /// the call to `Overworld_StartScrollTransition`. The receipt deliberately
    /// omits the resulting submodule value: translated gameplay owns that C
    /// mutation, while the replaceable timing authority owns only when the
    /// suspended call returned.
    ReloadReturned,
    /// The generation returned, but its mirror-warp caller is suspended in
    /// interactive cleanup before this slot's pickup test and type clear.
    GenerationReturnedAtInteractiveCleanup { slot: u8 },
    /// The current pickup test returned; its object-type clear is pending.
    GenerationReturnedAtInteractiveTypeClear { slot: u8 },
    /// The mirror portal owns a dynamic spawn suspended in its property reset.
    GenerationReturnedAtPortalReset { slot: u8, completed_stores: u8 },
    /// The reset returned; the mirror portal's property-table loads remain in flight.
    GenerationReturnedAtPortalLoadProperties { slot: u8, completed_stores: u8 },
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
    /// An ancilla (the falling milestone item, `Ancilla_MilestoneItemReceipt`)
    /// called `Link_ReceiveItem` from Sprite_Main's prefix, before the
    /// descending slot loop began. The ancilla slot identifies the caller.
    SpriteMainAncilla {
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

/// Zelda software-register operands sampled when hardware accepts an NMI.
///
/// `WritePpuRegisters` runs even when Zelda's update latch is held.  A source
/// host may return while that handler is still pending, after the translated
/// main-loop body has already advanced these mirrors.  Keeping the complete
/// operand generation beside the acceptance receipt prevents those later
/// native writes from being mistaken for the interrupted source generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NmiPpuRegisterOperands {
    pub window_selection: [u8; 3],
    pub color_window_selection: u8,
    pub color_math_control: u8,
    pub fixed_color: [u8; 3],
    pub screen_layers: [u8; 4],
    pub bg_scroll: [u16; 6],
    pub screen_brightness: u8,
    pub mosaic: u8,
    pub bg_mode: u8,
    pub mode7_center: [u16; 2],
}

/// Cumulative source prefix of `Intro_ValidateSram`'s final 768-byte clear.
///
/// The pinned ROM lowers the clear to one descending 16-bit loop. For each
/// even `word_offset`, it stores zero to the `$0d`, `$0e`, and `$0f` pages in
/// that order before decrementing the cursor by two. All words above
/// `word_offset` have therefore completed all three page stores; this many
/// stores have completed for the current word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileSelectGraphicsLowWramClearProgress {
    pub word_offset: u8,
    pub completed_page_stores: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OriginalTimingSemanticReceipt {
    NmiAccepted(NmiUpdateGate),
    /// The preemptive dungeon/Triforce poly worker entered its source-level
    /// render call. The temporary timing backend identifies the entry from
    /// private CPU state; translated gameplay consumes only the fact that the
    /// worker started the next frame in this host interval.
    PreemptivePolyhedralRenderStarted,
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
    DungeonPegAttributeFlipProgress(DungeonPegAttributeFlipProgressReceipt),
    DungeonResetSpritesProgress(DungeonResetSpritesProgressReceipt),
    SpriteResetAllProgress(SpriteResetAllProgressReceipt),
    PreOverworldStageCompleted(PreOverworldStageCompletion),
    DungeonFallingEntranceProgress(DungeonFallingEntranceProgress),
    RescuedMaidenTilemapClearProgress(RescuedMaidenTilemapClearProgressReceipt),
    RescuedMaidenInitializationProgress(RescuedMaidenInitializationProgressReceipt),
    DmaPublicationCompleted {
        channel_mask: u8,
    },
    /// `Death_Func15`'s save-quit branch returned from `Death_Func31` and
    /// published its module, Link-coordinate, and scroll-reset prefix before
    /// entering the source-ordered dungeon-info clear. The later clear/song
    /// upload remain owned by the suspended caller.
    SaveQuitResetStatePublished,
    /// `Intro_ValidateSram` committed a source-ordered prefix of its final
    /// low-WRAM clear while the file-select graphics caller remains suspended.
    /// The range aliases live sprite arrays, so every host-visible store is
    /// gameplay state rather than private decompressor timing.
    FileSelectGraphicsLowWramClearProgress(FileSelectGraphicsLowWramClearProgress),
    /// The same source loop completed all of WRAM `$0d00-$0fff`.
    FileSelectGraphicsLowWramCleared,
    /// Starting-point entrance has published its room and working vertical
    /// scroll, before copying that scroll to the NMI display mirrors.
    SelectedGameEntranceScrollPublished,
    /// `Main_ShowTextMessage` returned inside Module05's Message destination.
    /// Module 14 and submodule 2 are now observable, while palette loading and
    /// Module05's final Module1B publication remain in the suspended caller.
    SelectedGameLoadMessageInterfacePublished,
    TriforceRoomCase2PaletteProgress(TriforceRoomCase2PaletteProgressReceipt),
    CreditsSceneLoadProgress(CreditsSceneLoadProgressReceipt),
    CreditsEndSequence32Progress(CreditsEndSequence32ProgressReceipt),
    /// Module0B/$24 completed its first animated-sprite decode and
    /// `LoadOverworldFromSpecialOverworld`, then entered the second decode.
    /// The saved overworld coordinates and scroll state are now live, while
    /// the second decode and submodule advance remain in the suspended caller.
    OverworldSpecialExitMosaicRestored,
    /// Module0B/$24 returned from its second animated-sprite decode and
    /// advanced to the next submodule. The timing backend identifies the
    /// enclosing source-stage return; gameplay owns every restored value.
    OverworldSpecialExitMosaicReturned,
    /// LinkOam_Main reached a source checkpoint while its temporary stair
    /// coordinate is live. The remaining drawing suffix owns its restoration.
    LinkOamStairProgress(LinkOamStairProgress),
    /// Dungeon_LoadEntrance finished its backup/reset prefix, before choosing
    /// the entrance table or publishing the new room and Link placement.
    SelectedGameEntranceBeforeSelection,
    /// The selected-game caller returned from Dungeon_LoadEntrance.
    SelectedGameEntranceReturned,
    DungeonFallingFadeInPaletteDirectionToggled,
    /// Save-and-quit's intro-memory initialization returned; palette/reset
    /// work remains inside the suspended Death_Func31 caller.
    SaveQuitIntroMemoryReturned,
    /// Module 7 reached its push-block draw call after saving and applying
    /// the caller's scroll values; drawing and Sprite_Main remain pending.
    DungeonPushBlocksPending,
    /// Module 7's Dungeon_PushBlock_Handler loop was interrupted: the misc
    /// objects before word offset `next_index` have run; the rest of the loop,
    /// the scroll copies, drawing and Sprite_Main remain pending.
    DungeonPushBlocksInProgress {
        next_index: u16,
    },
    /// Module 7's Dungeon_PushBlock_Handler returned and OrientLampLightCone
    /// was entered but has stored nothing; the lamp cone, scroll copies,
    /// drawing and Sprite_Main remain pending.
    DungeonPushBlocksHandled,
    /// Module09 has applied three saved scroll pairs; BG1 vertical and
    /// the ensuing Sprite_Main call remain in its native caller frame.
    Module09FinalScrollPairPending,
    PreDungeonSpriteDisableThrough {
        slot: u8,
        boundary: OriginalTimingBoundary,
    },
    /// Sprite_ResetAll's disable call cleared garnish slots 29 through slot.
    PreDungeonGarnishDisableThrough {
        slot: u8,
        boundary: OriginalTimingBoundary,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LinkOamStairProgress {
    PoseSelected,
    EquipmentSelection,
    BodySelection,
    ShadowSelection,
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

/// Source progress of one RenderText scroll call during a host interval.
/// Each pass denotes a completed native pixel-copy operation, not oracle
/// buffer contents or an absolute scroll-position value. Multiple calls in
/// one host retain their source order as separate receipts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DialogueScrollProgressReceipt {
    pub entered: bool,
    pub completed_pixel_passes: u8,
    pub returned: bool,
}

/// One timing-authority result for exactly one upcoming host call.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OriginalTimingHostReceipts {
    pub(crate) host_call: u64,
    pub(crate) input_state: u16,
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    /// One complete `WritePpuRegisters` operand generation per ordered
    /// `NmiAccepted` receipt.  The live adapter validates this alignment
    /// before constructing the host receipt; schema-gated cached evidence
    /// therefore cannot silently fall back to post-interruption native RAM.
    pub(crate) nmi_acceptance_ppu_register_operands: Vec<NmiPpuRegisterOperands>,
    /// Native DSP-sample offsets of source `APUI00` song-end polls in this
    /// host interval, in C call order. The timing authority supplies only the
    /// point within the current audio window; Zelda's own SPC clock remains
    /// authoritative for the value observed at that point.
    #[serde(default)]
    pub(crate) song_end_poll_native_sample_offsets: Vec<u16>,
    #[serde(default)]
    pub(crate) dialogue_scroll_progress: Vec<DialogueScrollProgressReceipt>,
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
            nmi_acceptance_ppu_register_operands: Vec::new(),
            song_end_poll_native_sample_offsets: Vec::new(),
            dialogue_scroll_progress: Vec::new(),
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

    pub fn with_nmi_acceptance_ppu_register_operands(
        mut self,
        operands: Vec<NmiPpuRegisterOperands>,
    ) -> Self {
        self.nmi_acceptance_ppu_register_operands = operands;
        self
    }

    pub fn with_song_end_poll_native_sample_offsets(mut self, offsets: Vec<u16>) -> Self {
        self.song_end_poll_native_sample_offsets = offsets;
        self
    }

    pub fn with_dialogue_scroll_progress(
        mut self,
        progress: Vec<DialogueScrollProgressReceipt>,
    ) -> Self {
        self.dialogue_scroll_progress = progress;
        self
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
    DuplicateDungeonPegAttributeFlipProgress,
    InvalidDungeonPegAttributeFlipProgress,
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
    DuplicateSaveQuitResetStatePublished,
    DuplicateFileSelectGraphicsLowWramCleared,
    InvalidFileSelectGraphicsLowWramClearProgress,
    DuplicateSelectedGameLoadMessageInterfacePublished,
    DuplicateTriforceRoomCase2PaletteProgress,
    InvalidTriforceRoomCase2PaletteProgress,
    DuplicateCreditsSceneLoadProgress,
    InvalidCreditsSceneLoadProgress,
    DuplicateCreditsEndSequence32Progress,
    InvalidCreditsEndSequence32Progress,
    InvalidOverworldSpriteReloadProgress,
    DuplicatePreOverworldStageCompletion,
    DuplicateDungeonFallingEntranceProgress,
    DuplicateRescuedMaidenTilemapClearProgress,
    InvalidRescuedMaidenTilemapClearProgress,
    DuplicateRescuedMaidenInitializationProgress,
    InvalidRescuedMaidenInitializationProgress,
    InvalidNmiLifecycle,
    InvalidNmiPpuRegisterOperands,
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
