//! Zelda 3 game logic. Port of the C sources under `zelda3/src/`.
//!
//! Holds the reverse-engineered game code (sprites, dungeons, overworld,
//! HUD, etc.). Designed to run in lockstep with `snes::` so each frame's
//! WRAM/SRAM/VRAM can be byte-compared against the original ROM.

#![allow(dead_code)]

pub mod chr_source;
pub mod config;
pub mod game_output;
pub(crate) mod game_state;
pub mod modern_audio;
pub mod modern_audio_sequence;
pub mod modern_music_catalog;
pub mod modern_music_globals;
mod modern_sample_bank;
pub mod modern_sfx_catalog;
pub mod modern_sfx_dsp_catalog;
pub mod modern_sfx_pitch_catalog;
mod raster_timing;
mod rom_cpu_timing;
mod rom_random;
mod spc_driver_clock;
mod timing_receipts;
pub mod types;
pub mod util;
pub mod zelda_rtl;

pub use chr_source::{
    chr_content_hash32, LogicalChrSrc, VramChrSourceTable, CHR_KIND_BG, CHR_KIND_BG3,
    CHR_KIND_BG_ANIM, CHR_KIND_BG_STREAM, CHR_KIND_LINK, CHR_KIND_LINK_CONTENT, CHR_KIND_NONE,
    CHR_KIND_SPRITE, CHR_LINK_SRC_RAM_FLAG, VRAM_CHR_SLOTS,
};
pub use game_state::{
    CachedSpriteCacheField, OverworldMap16LoadState, SmallOverworldMap16ScrollBackupState,
};
pub use rom_random::{parse_rom_random_script, RomRandomSample};
pub use timing_receipts::{
    CachedSpriteExecutionBodyProgress, CachedSpriteExecutionProgress,
    CachedSpriteExecutionProgressReceipt, CreditsEndSequence32ProgressReceipt,
    CreditsSceneLoadProgress, CreditsSceneLoadProgressReceipt, DesertPrayerIrisProgress,
    DialogueExecutionProgress, DungeonFallingEntranceProgress,
    DungeonPegAttributeFlipProgressReceipt, DungeonResetSpritesProgressReceipt,
    ItemReceiptGraphicsCaller, ItemReceiptGraphicsProgressReceipt, JoypadPublication,
    MainLoopInterruption, MainLoopProgress, NmiPpuRegisterOperands, NmiUpdateGate,
    OriginalTimingAudioShadowResult, OriginalTimingBgScrollShadowResult,
    OriginalTimingBgTilemapShadowResult, OriginalTimingBoundary,
    OriginalTimingDialogueTextShadowResult, OriginalTimingHostReceipts,
    OriginalTimingMode7TransformShadowResult, OriginalTimingReceiptInstallError,
    OriginalTimingResumeCheckpoint, OriginalTimingSemanticReceipt,
    OriginalTimingWindowMaskShadowResult, OverworldSpriteReloadProgress,
    PreOverworldStageCompletion, PresentedAnimatedBgDestination, PresentedAnimatedBgTiles,
    PresentedAudio, PresentedBgScroll, PresentedBgTilemapLayer, PresentedBgTilemaps,
    PresentedCgram, PresentedDialogueText, PresentedHudTilemap, PresentedInidisp,
    PresentedMode7Transform, PresentedOam, PresentedObjTiles, PresentedScanoutGeometry,
    PresentedWindowMask, RescuedMaidenInitializationProgressReceipt,
    RescuedMaidenInitializationStage, RescuedMaidenTilemapClearProgressReceipt,
    SaveMenuInitializationProgress, SourceCallProgress, SpotlightTableBuildCheckpoint,
    SpotlightTableBuildProgress, SpotlightTableBuildProgressReceipt, SpriteDynamicSpawnProgress,
    SpriteFollowerGraphicsCaller, SpriteInitializeResetPropertiesPhase, SpriteMainProgress,
    SpriteResetAllProgress, SpriteResetAllProgressReceipt, TriforceRoomCase2PaletteProgressReceipt,
};
pub use zelda3_dialogue as dialogue_ir;
pub use zelda_rtl::{
    Bg3VwfGlyphRun, DungeonLoadSpritesCpuProgress, DungeonResetSpritesCpuProgress,
    DungeonSpriteDisableCpuProgress, DungeonSpriteLoadCheckpoint,
    OriginalTimingResumeCheckpointError, ZeldaState, SRAM_SIZE, VRAM_WORDS,
};

/// Which engine thread(s) a frame step should run (`run_frame_internal`'s
/// `run_what` bitmask): the main game thread and/or the polyhedral thread.
pub const RUN_MAIN: u8 = 1;
pub const RUN_POLY: u8 = 2;
