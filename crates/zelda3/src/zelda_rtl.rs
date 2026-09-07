//! Minimal runtime environment from `src/zelda_rtl.c`.
//!
//! This is deliberately a skeleton: it owns the memory regions the oracle
//! compares and exposes the same frame entry point that later module ports
//! will fill in.

#![allow(non_snake_case)]

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

use snes::{DmaChannel, DmaState, PpuState, HDMA_START_CYCLE, WRAM_SIZE};

use crate::config::config_value_bytes;
#[cfg(test)]
use crate::game_state::constants::messaging::MODULE as MESSAGING_MODULE;
use crate::game_state::constants::nmi::{
    BG_CHAR_BUFFER_1 as NMI_BG_CHAR_BUFFER_1, BG_CHAR_HALF_BUFFER as NMI_BG_CHAR_HALF_BUFFER,
};
use crate::game_state::constants::{
    ANIMATED_TILE_DATA_SRC, ANIMATED_TILE_VRAM_ADDR, BG2_X_SCROLL, BG2_Y_SCROLL,
    CACHED_SPRITE_LIVE_FIELDS, CRYSTAL_ROTATION_COUNTER, DARKENING_OR_LIGHTENING_SCREEN,
    DUNGEON_ROOM, FRAME_COUNTER, HDMA_TABLE_DYNAMIC, MAIN_MODULE, MESSAGING_BUF_LOAD_GFX,
    MOVING_WALL_REPLACEMENT_BUFFER, NMI_BOOLEAN, OVERWORLD_SCROLL_X_END, OVERWORLD_SCROLL_X_START,
    OVERWORLD_SCROLL_Y_END, PALETTE_FILTER_COUNTDOWN, RESERVED_HDMA_TABLE, SUBMODULE, SUBSUBMODULE,
    VWF_ARR,
};
use crate::game_state::{
    lanmola_flat_trail_entry_from_ram, loaded_room_data_word, Bg1MovementAccumulatorState,
    BirdTravelDestinationState, BlastWallExplosionSlotState, BlastWallFireballSlotState,
    BlastWallFragmentSlotState, BombosBlastState, BombosFireColumnState, BossHomePositionRead,
    CachedSpriteRead, CompatibilityBytesView, CompatibilityBytesViewMut, DungeonStairList,
    FollowerLinkState, GameState, GraphicsDecompressionScratch, HappinessPondRupeeSlotState,
    HappinessPondRupeeSnapshot, HistoryPositionState, HudStateRead, HudTilemapState,
    IntroActorRead, LanmolaFlatTrailEntry, LanmolaSegmentMotionState, LinkDmaSourceSlot,
    LinkDmaSources, MsuResumeInfoState, MsuResumeSlot, MultiselectChoiceRead,
    NativeAncillaSlotBridgeMut, NativeAncillaSlotView, NativeArcheryGameBridgeMut,
    NativeArmosKnightHomePositionBridgeMut, NativeArrghusPuffHomePositionBridgeMut,
    NativeAttractSceneBridgeMut, NativeAttractVramDestinationBridgeMut,
    NativeBeamosLaserHistoryBridgeMut, NativeBg1MovementAccumulatorBridgeMut,
    NativeBirdTravelDestinationBridgeMut, NativeBlastWallBridgeMut,
    NativeBlastWallExplosionBridgeMut, NativeBlastWallFireballBridgeMut,
    NativeBlastWallFragmentBridgeMut, NativeBombosBlastBridgeMut, NativeBombosFireColumnBridgeMut,
    NativeBombosSpellBridgeMut, NativeCachedSpriteBridgeMut, NativeChainChompHistoryBridgeMut,
    NativeDecodedMessageTextBridgeMut, NativeDialogueMessageIndexBridgeMut,
    NativeDialogueNumberBridgeMut, NativeDialogueSourceOffsetBridgeMut,
    NativeDiggingGamePrizeBridgeMut, NativeDisplayStateBridgeMut, NativeDoorDebrisBridgeMut,
    NativeDualLayerTileCacheBridgeMut, NativeDungeonBg2AttributeBridgeMut,
    NativeDungeonDoorBridgeMut, NativeDungeonEntranceBackupBridgeMut,
    NativeDungeonEnvironmentBridgeMut, NativeDungeonHeaderBridgeMut,
    NativeDungeonKeySlotsBridgeMut, NativeDungeonMapDisplayBridgeMut,
    NativeDungeonMovableBlockBridgeMut, NativeDungeonMovingFloorBridgeMut,
    NativeDungeonObjectTrackingBridgeMut, NativeDungeonRoomDoorSetupBridgeMut,
    NativeDungeonRoomEffectsBridgeMut, NativeDungeonRoomItemBridgeMut,
    NativeDungeonRoomLoadBridgeMut, NativeDungeonRoomParserBridgeMut,
    NativeDungeonRoomRuntimeBridgeMut, NativeDungeonRoomTilemapBridgeMut,
    NativeDungeonRoomTrackingBridgeMut, NativeDungeonSavegameBridgeMut,
    NativeDungeonScratchWordBridgeMut, NativeDungeonSecretBridgeMut,
    NativeDungeonStairListsBridgeMut, NativeDungeonStairMovementBridgeMut,
    NativeDungeonTorchBridgeMut, NativeEffectAngleScratchBridgeMut, NativeEndingCreditBridgeMut,
    NativeEnemyDamageSubclassTableBridgeMut, NativeEnhancedFeaturesBridgeMut,
    NativeEtherOrbitBridgeMut, NativeFailedSpinSparkleSpawnBridgeMut, NativeFollowerLinkBridgeMut,
    NativeFollowerRuntimeBridgeMut, NativeFrameStateBridgeMut, NativeGarnishRuntimeBridgeMut,
    NativeGarnishSlotBridgeMut, NativeGarnishSlotView, NativeHappinessPondRupeeBridgeMut,
    NativeHudInventoryOrderBridgeMut, NativeHudStateBridgeMut, NativeIntroActorBridgeMut,
    NativeIntroSceneBridgeMut, NativeIntroSwordBridgeMut, NativeInventoryItemsBridgeMut,
    NativeLanmolaSegmentMotionBridgeMut, NativeMazeGameTimerBridgeMut,
    NativeMemorizedTileBridgeMut, NativeMessagingRenderBufferBridgeMut,
    NativeMessagingRuntimeBridgeMut, NativeMinigameBridgeMut, NativeMirrorWarpBridgeMut,
    NativeMoldormHistoryBridgeMut, NativeMultiselectChoiceBridgeMut, NativeOamStateBridgeMut,
    NativeOverlordSlotBridgeMut, NativeOverlordSlotView, NativeOverworldConfigTableBridgeMut,
    NativeOverworldEntranceBridgeMut, NativeOverworldEventInfoBridgeMut,
    NativeOverworldExitBridgeMut, NativeOverworldMap16BridgeMut, NativeOverworldMapUiBridgeMut,
    NativeOverworldMapZoomBridgeMut, NativeOverworldPaletteBackupBridgeMut,
    NativeOverworldScreenSizeBridgeMut, NativeOverworldScrollDeltaBridgeMut,
    NativeOverworldSpriteLoadedBridgeMut, NativeOverworldSpritePresenceBridgeMut,
    NativeOverworldTransitionBridgeMut, NativePaletteBufferBridgeMut, NativePaletteFilterBridgeMut,
    NativePlayerResourcesBridgeMut, NativePolyFaceCoordsBridgeMut,
    NativePolyProjectedVerticesBridgeMut, NativePolyRasterEdgeBridgeMut,
    NativePolyRuntimeBridgeMut, NativePpuScrollCopyBridgeMut, NativePrizeDropCycleBridgeMut,
    NativePushedBlockBridgeMut, NativeQuakeBoltBridgeMut, NativeQuakeSpellBridgeMut,
    NativeRoomBoundsBridgeMut, NativeSaveLoadTransferBridgeMut, NativeSaveProgressBridgeMut,
    NativeScratchCounterBridgeMut, NativeSelectFileMenuBridgeMut,
    NativeSharedMessageTimerBridgeMut, NativeSkullWoodsFireBridgeMut,
    NativeSkullWoodsFireSlotBridgeMut, NativeSpecialExitPositionBridgeMut,
    NativeSpotlightHdmaBridgeMut, NativeSpriteBattleBridgeMut,
    NativeSpriteDrawWorkPositionBridgeMut, NativeSpriteHitboxWorkOffsetBridgeMut,
    NativeSpriteSlotBridgeMut, NativeSpriteSlotView, NativeSpriteSystemBridgeMut,
    NativeSpriteWorkspaceBridgeMut, NativeSwamolaHistoryBridgeMut, NativeSwamolaTargetBridgeMut,
    NativeSwimAccelerationBridgeMut, NativeSystemSignalsBridgeMut, NativeTagalongSlotBridgeMut,
    NativeTileDetectionBridgeMut, NativeTowerSealBridgeMut, NativeTowerSealOrbitBridgeMut,
    NativeTowerSealSparkleBridgeMut, NativeTrinexxPaletteBridgeMut,
    NativeVramUploadBufferBridgeMut, NativeVwfRenderBridgeMut, NativeWaterHdmaWindowBridgeMut,
    NativeWeatherVaneBridgeMut, NativeWeatherVaneDebrisBridgeMut,
    NativeWorldCameraBoundariesBridgeMut, NativeWorldLocationBridgeMut,
    NativeWorldPaletteThemeBridgeMut, NativeWorldRegionBridgeMut, NativeWorldScrollBridgeMut,
    NativeWorldTransientBridgeMut, OverworldConfigTableRead, OverworldMap16Decode,
    OverworldMap16DecodeScratch, OverworldMap16LoadState, OverworldMap16SourcePage,
    PpuScrollCopyState, QuakeBoltSlotState, RamPlayerStateView, RamPlayerStateViewMut,
    SkullWoodsFireSlotState, SmallOverworldMap16ScrollBackupState, SpotlightHdmaState,
    SpriteSlotsState, SystemSignalsState, SystemWorkArea, TagalongSlotRead, TowerSealOrbitState,
    TowerSealSparkleState, WeatherVaneDebrisSlotState,
};
use crate::raster_timing::{
    attract_map_projection_current_word_is_visible,
    straight_interroom_fadeout_suffix_crosses_vblank, SpriteMainTimingWorkload,
    ATTRACT_MAP_PROJECTION_WORDS,
};
use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
use crate::timing_receipts::{
    sanitize_original_timing_input, CachedSpriteExecutionProgressReceipt,
    CreditsEndSequence32ProgressReceipt, CreditsSceneLoadProgressReceipt,
    DungeonResetSpritesProgressReceipt, FileSelectGraphicsLowWramClearProgress,
    ItemReceiptGraphicsCaller, ItemReceiptGraphicsProgressReceipt, NmiUpdateGate,
    OriginalTimingBoundary, OriginalTimingHostReceipts, OriginalTimingReceiptInstallError,
    OriginalTimingSemanticReceipt, SaveMenuInitializationProgress, SourceCallProgress,
    SpotlightTableBuildProgress, SpotlightTableBuildProgressReceipt, SpriteResetAllProgress,
    SpriteResetAllProgressReceipt, TriforceRoomCase2PaletteProgressReceipt,
};
use crate::types::{read_le_u16, write_le_u16, xy, MemBlk};
use crate::util::{find_index_in_memblk, ByteArray, ByteArray_AppendByte, ByteArray_AppendData};

// Snes9x reaches the ROM's first initialized WRAM state after 81 complete
// libretro frames.  The pixels written by that work are published separately
// at the pre-NMI display boundary below; delaying the CPU for an extra frame
// only happened to hide that boundary error during the earliest fade.
const ROM_RESET_FRAME_DELAY: u8 = 81;
const ROM_INTRO_CLEAR_1KB_CONTINUATION_FRAMES: u8 = 1;
const ROM_INTRO_MESSAGE_POINTER_CONTINUATION_FRAMES: u8 = 48;
const ROM_INTRO_ITEM_GFX_CONTINUATION_FRAMES: u8 = 15;
const ROM_INTRO_FOLLOWER_GFX_CONTINUATION_FRAMES: u8 = 3;
const ROM_INTRO_MEMORY_INITIALIZATION_FRAMES: u8 = 41;
// The asset-backed body/head/hand DMA is the first Link OBJ batch in NMI.
// During a long NMI this batch can become visible while the later equipment,
// animated-tile, pointer, and travel-bird uploads still belong to the retained
// scanout generation.
const EARLY_LINK_OBJ_DMA_TRANSFERS: [(usize, LinkDmaSourceSlot, usize); 6] = [
    (0x4100, LinkDmaSourceSlot::BodyBottom, 0x40),
    (0x4120, LinkDmaSourceSlot::HeadBottom, 0x40),
    (0x4140, LinkDmaSourceSlot::HandRight, 0x20),
    (0x4000, LinkDmaSourceSlot::BodyTop, 0x40),
    (0x4020, LinkDmaSourceSlot::HeadTop, 0x40),
    (0x4040, LinkDmaSourceSlot::HandLeft, 0x20),
];

fn compose_early_link_obj_cache(
    base_vram: &[u16],
    sources: LinkDmaSources,
    link_graphics: Option<&[u8]>,
) -> Vec<u16> {
    let mut obj_cache_vram = base_vram.to_vec();
    for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
        let source_address = usize::from(sources.source(source));
        let source_offset = source_address.saturating_sub(0x8000);
        let destination_end = destination + len / 2;
        let Some(source_bytes) = link_graphics.and_then(|graphics| {
            (source_address >= 0x8000 && source_offset + len <= graphics.len())
                .then_some(&graphics[source_offset..source_offset + len])
        }) else {
            continue;
        };
        for (word, bytes) in obj_cache_vram[destination..destination_end]
            .iter_mut()
            .zip(source_bytes.chunks_exact(2))
        {
            *word = u16::from_le_bytes([bytes[0], bytes[1]]);
        }
    }
    obj_cache_vram
}

fn link_obj_cache_sources_for_publication(
    completed_dma: Option<CompletedLinkObjDma>,
    explicit_obj_cache_owner: bool,
) -> Option<LinkDmaSources> {
    // C changes the Link OBJ page only in NMI_DoUpdates. A scanout-generation
    // plan describes which already-completed page is visible; it is not proof
    // that another DMA ran. Refine the captured decoded page only from the
    // exact leading-NMI receipt which records those six transfers.
    completed_dma
        .filter(|completed| {
            completed.source_generation == GraphicsDmaGeneration::LiveAfterMain
                && !explicit_obj_cache_owner
        })
        .map(|completed| completed.sources)
}

fn copy_bytes_into_obj_cache(vram: &mut [u16], destination: usize, source: &[u8]) {
    for (word, bytes) in vram[destination..destination + source.len() / 2]
        .iter_mut()
        .zip(source.chunks_exact(2))
    {
        *word = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
}

fn compose_complete_link_obj_cache(
    base_vram: &[u16],
    sources: LinkDmaSources,
    link_graphics: Option<&[u8]>,
    ram: &[u8],
) -> Vec<u16> {
    let mut obj_cache_vram = compose_early_link_obj_cache(base_vram, sources, link_graphics);
    for (destination, source, len) in [
        (0x4050, LinkDmaSourceSlot::SwordUpper, 0x40),
        (0x4070, LinkDmaSourceSlot::ShieldUpper, 0x40),
        (0x4090, LinkDmaSourceSlot::AuxUpper, 0x40),
        (0x40b0, LinkDmaSourceSlot::AnimatedTileUpper, 0x20),
        (0x40c0, LinkDmaSourceSlot::PushUpper, 0x40),
        (0x4150, LinkDmaSourceSlot::SwordLower, 0x40),
        (0x4170, LinkDmaSourceSlot::ShieldLower, 0x40),
        (0x4190, LinkDmaSourceSlot::AuxLower, 0x40),
        (0x41b0, LinkDmaSourceSlot::AnimatedTileLower, 0x20),
        (0x41c0, LinkDmaSourceSlot::PushLower, 0x40),
        (0x4200, LinkDmaSourceSlot::HeadPointerUpper, 0x40),
        (0x4220, LinkDmaSourceSlot::BodyPointerUpper, 0x40),
        (0x4300, LinkDmaSourceSlot::HeadPointerLower, 0x40),
        (0x4320, LinkDmaSourceSlot::BodyPointerLower, 0x40),
    ] {
        let source_address = usize::from(sources.source(source));
        let Some(source_bytes) = ram.get(source_address..source_address + len) else {
            continue;
        };
        copy_bytes_into_obj_cache(&mut obj_cache_vram, destination, source_bytes);
    }
    if let Some(high_planes) = ram.get(
        LINK_DMA_EXPANDED_HIGH_PLANES_START
            ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN,
    ) {
        copy_bytes_into_obj_cache(
            &mut obj_cache_vram,
            0x4240,
            &high_planes[..LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN],
        );
        copy_bytes_into_obj_cache(
            &mut obj_cache_vram,
            0x4340,
            &high_planes[LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN..],
        );
    }
    obj_cache_vram
}

// File-select graphics completes after 56 interrupted CPU slices. The next
// host frame resumes the caller without consuming another display boundary.
const FILE_SELECT_GRAPHICS_NMI_SLICES: u8 = 56;
// The original CPU reaches Module_PreDungeon's audio prefix after 19 complete
// NMI slices. The twentieth CPU slice writes the command before its NMI, then
// 57 more interrupted slices finish the selected-game load.
const SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES: u8 = 20;
const SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES: u8 = 57;
const SELECTED_GAME_LOAD_NMI_SLICES: u8 = SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES
    + SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES;
// Module_PreDungeon's audio prefix returns before the interruptible entrance
// load begins. Clean Snes9x NMI-PC traces divide the remaining caller into
// semantic workloads: ten boundaries in entrance/room construction, four in
// animated-tile decompression, ten in the attribute table, 33 in tileset
// decompression/conversion, and one in the final sprite reset/caller suffix.
// Keep the total derived from those workloads so later room-specific timing
// can refine a stage instead of growing a route/frame exception.
const PRE_DUNGEON_ROOM_CONSTRUCTION_NMI_SLICES: u8 = 10;
const PRE_DUNGEON_ANIMATED_TILES_NMI_SLICES: u8 = 4;
const PRE_DUNGEON_ATTRIBUTE_TABLE_NMI_SLICES: u8 = 10;
const PRE_DUNGEON_TILESETS_NMI_SLICES: u8 = 33;
const PRE_DUNGEON_RETURN_SUFFIX_NMI_SLICES: u8 = 1;
const PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES: u8 = PRE_DUNGEON_ROOM_CONSTRUCTION_NMI_SLICES
    + PRE_DUNGEON_ANIMATED_TILES_NMI_SLICES
    + PRE_DUNGEON_ATTRIBUTE_TABLE_NMI_SLICES
    + PRE_DUNGEON_TILESETS_NMI_SLICES
    + PRE_DUNGEON_RETURN_SUFFIX_NMI_SLICES;
// Module_PreDungeon publishes module $07/$0f, starts LoadSongBank by writing
// $ff to APUI0, then remains inside the $00:8888 transfer loop for 22 host
// boundaries before its caller can release the main-loop NMI latch.
const PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES: u8 = 22;
// The ROM enters AttractScene_ThroneRoom on frame 5939 and does not return
// from its dungeon-room construction work until frame 5981. NMI continues to
// run while that main-CPU call is in flight, so model the interruption slices
// themselves instead of delaying the later dialogue state.
const ATTRACT_THRONE_ROOM_NMI_SLICES: u8 = 42;
// Room 0x73 uses the same resumable dungeon construction path as the throne
// room, minus the extra common-sprite upload. The ROM returns after 40 NMIs.
const ATTRACT_ZELDA_PRISON_NMI_SLICES: u8 = 40;
// Room 0x75 includes the room build plus its distinct palette setup and
// completes one NMI later than the prison-room preparation.
const ATTRACT_MAIDEN_WARP_NMI_SLICES: u8 = 41;
// The conclusion transition darkens memory and reloads the overworld palettes
// before returning to the intro module. The ROM's main CPU resumes after 45
// intervening NMIs; keep that work attached to the transition itself.
const ATTRACT_END_OF_STORY_NMI_SLICES: u8 = 45;
// Module11_02_LoadEntrance enters the room/tile construction call on host
// frame 8452 and returns through Module_MainRouting on frame 8508. The entry
// slice performs the prefix before the first interrupt, leaving 56 subsequent
// NMI boundaries while the original CPU remains inside that semantic work.
const DUNGEON_FALLING_ENTRANCE_ROOM_LOAD_NMI_SLICES: u8 = 56;
// The immediately following LoadNewSpriteGFXSet/dungeon_reset_sprites call
// begins on frame 8509 and returns on frame 8512.
const DUNGEON_FALLING_ENTRANCE_SPRITE_GFX_NMI_SLICES: u8 = 3;
// The standard animated-item path decompresses packs $5b and $5a, then
// expands the selected high-plane tiles while the main-loop NMI latch remains
// set. Both the measured $14 chest receipt and $06 scripted receipt return to
// the main-loop epilogue after four intervening vblanks.
const ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES: u8 = 4;
// A big-key enemy drop calls DecodeAnimatedSpriteTile_variable($22), which
// decompresses the fixed $5b/$5a sheet pair. The entry main slice reaches the
// decompressor before vblank; four following host boundaries pass before the
// call returns through the sprite loop and Module 7 caller suffix.
const BIG_KEY_DROP_GRAPHICS_NMI_SLICES: u8 = 4;
const ROM_TEXT_DECODE_FIRST_SLICE_CURSOR: u16 = 94;
const POLY_WORKER_TWO_FRAME_CYCLE_THRESHOLD: u32 = 28_250;
const SNES9X_INTRO_POLY_BOOTSTRAP_STEPS: u8 = 0;
const SNES9X_INTRO_THREAD_START_DELAY: u8 = 0;
const SNES9X_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER: u8 = 0x42;
const SNES9X_NMI_POLY_UPLOAD_DEFER_FRAMES: u8 = 3;

const fn resolve_active_display_blanking_scanout(
    captured_retain_prior_surface: bool,
    live_suffix_start_scanline: Option<u8>,
    live_retain_prior_surface: bool,
) -> ActiveDisplayBlankingScanout {
    match live_suffix_start_scanline {
        Some(line) => ActiveDisplayBlankingScanout {
            suffix_start_scanline: Some(line),
            retain_prior_surface: captured_retain_prior_surface || live_retain_prior_surface,
        },
        None => ActiveDisplayBlankingScanout {
            suffix_start_scanline: None,
            retain_prior_surface: captured_retain_prior_surface || live_retain_prior_surface,
        },
    }
}

const fn live_forced_blank_for_scanout(
    published_ppu_is_blank: bool,
    completed_nmi_forced_blank_write: Option<bool>,
    active_display_force_blank_write: Option<u8>,
    staged_scanout_owns_following_generation: bool,
) -> bool {
    (published_ppu_is_blank
        || matches!(completed_nmi_forced_blank_write, Some(true))
        || active_display_force_blank_write.is_some())
        && !staged_scanout_owns_following_generation
}

const fn dungeon_map_terminal_fade_blank_scanline(
    main_module: u8,
    submodule: u8,
    overworld_map_state: u8,
    brightness: u8,
) -> Option<u8> {
    if main_module == 14 && submodule == 3 && overworld_map_state == 1 && brightness == 1 {
        // Crystal-4 route frame 27,888 writes INIDISP=$80 at V=49, H=320.
        // Active output row zero is hardware V=1, so rows 0..47 retain the
        // brightness-1 surface and forced blank begins at output row 48.
        Some(48)
    } else {
        None
    }
}

fn nmi_active_display_blanking_for_pending_work(
    core_updates_disabled: bool,
    forced_blank: bool,
    bg_vram_load_mode: u8,
    stripe_work: StripeUploadWork,
) -> NmiActiveDisplayBlanking {
    // Module_NamePlayer_2 publishes asset 100 through mode 5: 47 stripe
    // packets representing 1,936 transferred bytes. With normal core updates
    // enabled, instrumented Snes9x reaches the ROM's INIDISP copy at scanline
    // 50, dot 1228 (clean-route frame 1061), so scanlines 0..49 retain forced
    // blank. Classify the actual NMI workload rather than its route position.
    if forced_blank
        && !core_updates_disabled
        && bg_vram_load_mode == 5
        && stripe_work.transfer_bytes == 1_936
    {
        return NmiActiveDisplayBlanking {
            prefix_scanlines: 50,
            suffix_start_scanline: None,
        };
    }
    // This workload returns from NMI with enough main-thread work remaining
    // that the ROM's transition write to INIDISP occurs at V=1, H=698. Snes9x
    // has already rendered scanline zero at that point, so the write blanks the
    // suffix beginning at scanline one. Keep the direction of the transition
    // explicit instead of folding it into the forced-blank prefix above.
    if !forced_blank
        && !core_updates_disabled
        && bg_vram_load_mode == 1
        && stripe_work
            == (StripeUploadWork {
                packets: 6,
                transfer_bytes: 216,
                fixed_source_packets: 4,
                vertical_packets: 0,
            })
    {
        NmiActiveDisplayBlanking {
            prefix_scanlines: 0,
            suffix_start_scanline: Some(1),
        }
    } else {
        NmiActiveDisplayBlanking {
            prefix_scanlines: 0,
            suffix_start_scanline: None,
        }
    }
}

const fn hud_and_full_tilemap_nmi_forced_blank_prefix(
    hud_upload_pending: bool,
    full_tilemap_upload_pending: bool,
) -> u8 {
    // C NMI_DoUpdates first uploads 165 HUD words when
    // flag_update_hud_in_nmi is set, then subroutine 1 uploads the complete
    // $800-byte tilemap staging buffer. With both transfers pending, Snes9x
    // reaches the subroutine DMA at ROM $008cdb on V=249 and does not restore
    // INIDISP until V=1 H=870, so output row zero is black. Subroutine 1 alone
    // returns at V=261 and does not cross into the next field.
    if hud_upload_pending && full_tilemap_upload_pending {
        1
    } else {
        0
    }
}

fn stripe_upload_work(mut stripes: &[u8]) -> StripeUploadWork {
    let mut work = StripeUploadWork::default();
    while stripes.first().copied().unwrap_or(0x80) & 0x80 == 0 {
        if stripes.len() < 4 {
            break;
        }
        let flags = stripes[2];
        let len = ((((u16::from(flags)) << 8) | u16::from(stripes[3])) & 0x3fff) as usize + 1;
        work.packets = work.packets.saturating_add(1);
        work.transfer_bytes = work.transfer_bytes.saturating_add(len);
        work.fixed_source_packets = work
            .fixed_source_packets
            .saturating_add(usize::from(flags & 0x40 != 0));
        work.vertical_packets = work
            .vertical_packets
            .saturating_add(usize::from(flags & 0x80 != 0));
        let stored = if flags & 0x40 != 0 { 2 } else { len };
        if stripes.len() < 4 + stored {
            break;
        }
        stripes = &stripes[4 + stored..];
    }
    work
}

fn stripe_upload_clears_dialogue_box(stripes: &[u8]) -> bool {
    let Some(packet) = stripes.get(..8) else {
        return false;
    };
    let destination = u16::from_be_bytes([packet[0], packet[1]]);
    matches!(destination, 0x6125 | 0x6244) && packet[2..] == [0x42, 0x2e, 0x7f, 0x38, 0xff, 0xff]
}

const fn attract_throne_room_nmi_slices(retained_sprite_subset_2: u8) -> u8 {
    // Sprite tileset 0x7e deliberately leaves subset 2 unchanged. On a cold
    // attract pass that retained pack is 19; after the story restart it is 66.
    // The latter compressed stream makes InitializeTilesets execute 20,282
    // additional ROM instructions and span two more NMIs. Budget from the
    // actual retained asset identity, not from which attract loop is running.
    ATTRACT_THRONE_ROOM_NMI_SLICES + if retained_sprite_subset_2 == 66 { 2 } else { 0 }
}

pub(crate) const fn rom_intro_poly_thread_is_active(main_module: u8, submodule: u8) -> bool {
    main_module == 0 && matches!(submodule, 3 | 4 | 5 | 7 | 9 | 11)
}

const fn rom_intro_poly_initialization_is_active(main_module: u8, submodule: u8) -> bool {
    // Submodule 2 is the cold-start path; submodule 10 restarts the same
    // Triforce worker after the attract story. Both execute the same expensive
    // sprite/graphics initialization and can cross NMI boundaries.
    main_module == 0 && matches!(submodule, 2 | 10)
}

const fn rom_full_tilemap_scanout_retains_uploaded_region(
    pending_full_tilemap_upload: bool,
    forced_blank_prefix_scanlines: u8,
) -> bool {
    // NMI subroutine 1 uploads the complete $800-byte staging buffer. A request
    // present in the pre-NMI snapshot was authored for the following vblank, so
    // this scanout retains that destination's tilemap generation already in
    // VRAM. Other NMI transfers still publish normally: the SNES has one VRAM,
    // but scanout ownership belongs to each DMA transaction rather than to the
    // entire address space. The initial menu upload is the measured overrun
    // exception: its DMA ends at V=249 and INIDISP returns at V=1, making the
    // new tilemap visible from scanline one while only the first line retains
    // forced blank.
    pending_full_tilemap_upload && forced_blank_prefix_scanlines == 0
}

const fn rom_display_memory_publication_is_deferred(
    main_module: u8,
    submodule: u8,
    text_render_state: u8,
    pending_main_thread_stripe: bool,
) -> bool {
    // A mode-1 stripe packet pending at the capture boundary was authored by
    // the main thread after the active frame's hardware NMI. It is consumed by
    // the following NMI, so publishing live post-NMI memory here would expose
    // every menu stripe (file select, naming, copy, erase) one frame early.
    // Dialogue character tiles use their own BG3 NMI packet but share that
    // next-publication cadence while RenderText_Draw_MessageCharacters owns
    // the CPU buffer (text state 3). RenderText_Draw_Finish is state 4: its
    // completed display generation publishes at the boundary instead of being
    // hidden by the broader Module 14/submodule 2 rule this replaces.
    // WorldMap_HandleSprites likewise authors the map marker after the active
    // frame's OAM DMA; it appears at the following NMI rather than immediately
    // in Module 14/submodule 7.
    pending_main_thread_stripe || (main_module == 14 && submodule == 2 && text_render_state != 4)
}

const fn rom_display_oam_publication_is_deferred(
    main_module: u8,
    submodule: u8,
    text_render_state: u8,
    active_display_nmi_overrun: bool,
    pending_main_thread_stripe: bool,
) -> bool {
    // Normal gameplay authors the OAM shadow during the main loop. NMI uploads
    // that shadow after the active frame's capture boundary, so the frame being
    // presented must retain the OAM image uploaded by the preceding NMI.
    // An NMI that runs into active display necessarily precedes the resumed
    // main-thread sprite authoring as well; its partial scanout therefore uses
    // the preceding OAM generation regardless of module identity. The steady
    // name-player loop has the same ordinary main-then-next-NMI cadence for its
    // cursor and underline sprites, including input-driven row transitions.
    active_display_nmi_overrun
        || rom_display_memory_publication_is_deferred(
            main_module,
            submodule,
            text_render_state,
            pending_main_thread_stripe,
        )
        || (main_module == 4 && submodule == 3)
        || (main_module == 14 && submodule == 7)
        || matches!(
            rom_graphics_dma_plan(main_module, submodule).oam_scanout,
            OamScanoutSource::RetainCapturedBeforeNmi
        )
}

const fn dungeon_exit_spotlight_entry_return_obj_scanout() -> ObjScanoutGenerations {
    // The resumed Module 15 caller finishes LinkOam_Main before the following
    // vblank. That NMI consumes the completed OAM shadow, while Link's OBJ CHR
    // upload still uses the operands captured at the suspended host boundary.
    ObjScanoutGenerations {
        oam: OamScanoutSource::ComposeCompletedWorkAfterNmi,
        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
    }
}

const fn dungeon_subtile_palette_filter_return_obj_scanout() -> ObjScanoutGenerations {
    // The resumed Module07 suffix authors a new OAM shadow after the resident
    // table was already DMAed, but its prepared Link sources are consumed by
    // the ensuing NMI before sprite evaluation. Keep those domains separate.
    ObjScanoutGenerations {
        oam: OamScanoutSource::RetainResidentPpuOam,
        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
    }
}

const fn straight_interroom_fadeout_return_obj_scanout() -> ObjScanoutGenerations {
    ObjScanoutGenerations {
        oam: OamScanoutSource::RetainResidentPpuOam,
        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
    }
}

const fn straight_interroom_fadeout_obj_scanout() -> ObjScanoutGenerations {
    ObjScanoutGenerations {
        oam: OamScanoutSource::RetainCapturedBeforeNmi,
        link_obj: GraphicsDmaGeneration::LiveAfterMain,
        link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
    }
}

const fn straight_interroom_fadeout_following_obj_scanout() -> ObjScanoutGenerations {
    ObjScanoutGenerations {
        oam: OamScanoutSource::RetainResidentPpuOam,
        link_obj: GraphicsDmaGeneration::LiveAfterMain,
        link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
    }
}

const fn atomic_item_graphics_return_obj_scanout(
    continuation: ItemReceiptGraphicsContinuation,
) -> ObjScanoutGenerations {
    // The decompressor's final interrupt retains Link OBJ CHR. Its caller then
    // prepares the next OAM shadow before the following display boundary, so
    // that boundary legitimately combines live OAM with resident Link tiles.
    let link_obj = match continuation {
        // The enemy-drop pickup's $22 sheet has completed its last OBJ upload at
        // this boundary. Unlike ordinary equipment receipts, there is no
        // resident Link-sheet tail to retain after that upload returns.
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x14 | 0x22, ..
        } => GraphicsDmaGeneration::LiveAfterMain,
        _ => GraphicsDmaGeneration::HostBoundaryBeforeMain,
    };
    let oam = match continuation {
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14, .. } => {
            OamScanoutSource::ComposeLivePlayerOamAfterMain
        }
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. } => {
            OamScanoutSource::ComposePublishedShadowDma
        }
        ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
        | ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. }
        | ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. } => {
            OamScanoutSource::ComposeLiveAfterNmi
        }
    };
    ObjScanoutGenerations {
        oam,
        link_obj,
        link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
    }
}

const fn rom_graphics_dma_plan(main_module: u8, submodule: u8) -> GraphicsDmaPlan {
    // These phase rules describe where the hardware NMI falls relative to the
    // native main-thread slice. OAM, Link OBJ CHR, and animated-BG DMA do not
    // always use the same generation, so keep the domains explicit while
    // deriving the complete plan in one place.
    let dungeon_entrance_nmi_precedes_main = main_module == 0x11 && submodule == 7;
    // Intra-room and straight inter-room stairs run their Link movement and
    // animation update after the vblank that uploads OAM and OBJ CHR. The
    // shadow and source words authored by that main-thread work therefore
    // belong to the following NMI.
    let dungeon_stairs_nmi_precedes_link_animation =
        main_module == 7 && matches!(submodule, 8 | 0x10 | 0x12);
    let dungeon_spiral_stairs_oam_nmi_precedes_main = main_module == 7 && submodule == 0x0e;
    // Module0E's dungeon-map handler authors its OAM shadow during main. The
    // trailing NMI publishes that shadow for the following scanout, so the
    // active map frame still displays the host-boundary OAM generation.
    let dungeon_map_oam_scanout_uses_host_boundary = main_module == 14 && submodule == 3;
    let dungeon_main_nmi_precedes_main = main_module == 7 && submodule == 0;
    // The Game Over choice fairy is authored after the menu frame's leading
    // OAM DMA. The active scanout receives the host-boundary shadow; the new
    // fairy frame remains software-only until the following vblank.
    let game_over_menu_oam_nmi_precedes_main = main_module == 0x12 && submodule == 9;
    // Shutter control runs after the frame's leading OAM DMA. Its CPU slice can
    // prepare Link's next shadow while the independently scheduled Link-CHR
    // upload remains on the ordinary live generation.
    let dungeon_shutter_oam_nmi_precedes_main = main_module == 7 && submodule == 5;
    // The overworld transition load pipeline (submodules 1..=4: LoadAuxGFX,
    // FinishTransGfx, LoadNewMapAndGFX, LoadNewSprites) holds the main loop
    // across its load slices, so hardware keeps scanning out the pre-hold OAM
    // generation. Measured at route frames 6913 (submodule 1), 6931
    // (submodule 3), and 6944 (submodule 4): Link's entries match the
    // captured boundary image while the frame counter stands still.
    let player_link_obj_scanout_uses_host_boundary = (submodule == 0
        && matches!(main_module, 9 | 11))
        || (main_module == 9 && matches!(submodule, 1..=8 | 0x0a));
    // The overworld screen-transition submodule keeps the ordinary retained
    // cadence: its loader owns the main slice past the leading NMI (route
    // frame 6913: Link's camera-dragged coordinates split the live composition
    // from the boundary shadow hardware scans out, and the held load frames
    // that follow keep the oracle on the same generation). The previous
    // live-scanout exclusion here dated to a window where nothing moved during
    // the load, making live and published indistinguishable.
    let player_oam_scanout_uses_host_boundary = player_link_obj_scanout_uses_host_boundary;
    let oam_scanout_uses_host_boundary = dungeon_entrance_nmi_precedes_main
        || player_oam_scanout_uses_host_boundary
        || dungeon_stairs_nmi_precedes_link_animation
        || dungeon_spiral_stairs_oam_nmi_precedes_main
        || dungeon_map_oam_scanout_uses_host_boundary;
    let animated_bg_nmi_precedes_scanout =
        dungeon_entrance_nmi_precedes_main || dungeon_main_nmi_precedes_main;

    GraphicsDmaPlan {
        oam_operands: if dungeon_main_nmi_precedes_main
            || dungeon_shutter_oam_nmi_precedes_main
            || game_over_menu_oam_nmi_precedes_main
        {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else {
            GraphicsDmaGeneration::LiveAfterMain
        },
        oam_scanout: if oam_scanout_uses_host_boundary {
            OamScanoutSource::RetainCapturedBeforeNmi
        } else {
            OamScanoutSource::ComposeLiveAfterNmi
        },
        link_obj_scanout: if player_link_obj_scanout_uses_host_boundary {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else {
            GraphicsDmaGeneration::LiveAfterMain
        },
        link_obj_operands: if dungeon_entrance_nmi_precedes_main
            || dungeon_main_nmi_precedes_main
            || dungeon_stairs_nmi_precedes_link_animation
        {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else {
            GraphicsDmaGeneration::LiveAfterMain
        },
        animated_bg_operands: if dungeon_entrance_nmi_precedes_main
            || dungeon_main_nmi_precedes_main
            || (main_module == 9 && submodule == 5)
        {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else {
            GraphicsDmaGeneration::LiveAfterMain
        },
        animated_bg_scanout: if animated_bg_nmi_precedes_scanout {
            // A leading NMI completes its VRAM uploads before the active
            // display, so the overlapping animated-background region must use
            // that same live generation.
            AnimatedBgScanoutGeneration::LiveAfterNmi
        } else {
            AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
        },
    }
}

const fn animated_bg_operands_for_dungeon_landing(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    link_last_direction: u8,
    default: GraphicsDmaGeneration,
) -> GraphicsDmaGeneration {
    if dungeon_room == 0x72
        && link_last_direction & 0x0f == 8
        && frame.main_module == 7
        && frame.submodule == 1
    {
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    } else {
        default
    }
}

const fn rom_dungeon_transition_oam_scanout_uses_host_boundary(
    main_module: u8,
    submodule: u8,
    subsubmodule: u8,
) -> bool {
    // The landing sequence runs between consecutive OAM uploads: its initial
    // scroll steps, landing search, doorway movement, palette restore, and
    // shutter trigger author the following shadow while the active scanout
    // keeps the resident OAM image captured at the host boundary.
    main_module == 7
        && ((submodule == 1 && matches!(subsubmodule, 1..=7))
            || (submodule == 2 && matches!(subsubmodule, 1 | 3)))
}

const fn rom_dungeon_transition_link_obj_scanout_uses_host_boundary(
    main_module: u8,
    submodule: u8,
    subsubmodule: u8,
) -> bool {
    main_module == 7 && submodule == 2 && matches!(subsubmodule, 1 | 3)
}

fn rom_graphics_dma_plan_at_host_boundary(frame: crate::game_state::FrameState) -> GraphicsDmaPlan {
    let mut plan = rom_graphics_dma_plan(frame.main_module, frame.submodule);
    if rom_dungeon_transition_oam_scanout_uses_host_boundary(
        frame.main_module,
        frame.submodule,
        frame.subsubmodule,
    ) {
        plan.oam_scanout = OamScanoutSource::RetainResidentPpuOam;
    }
    if rom_dungeon_transition_link_obj_scanout_uses_host_boundary(
        frame.main_module,
        frame.submodule,
        frame.subsubmodule,
    ) {
        plan.link_obj_scanout = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    if frame.main_module == 7 && frame.submodule == 1 && matches!(frame.subsubmodule, 3..=7) {
        // Subtile transition movement runs after the leading NMI, so Link's
        // OBJ upload must not see the post-main operands. Animated BG DMA
        // retains the ordinary plan: v1.0.0 kept its independently captured
        // operand and scanout generations here.
        plan.link_obj_operands = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    plan
}

fn animated_bg_scanout_across_main(
    entry: GraphicsDmaPlan,
    exit: GraphicsDmaPlan,
) -> AnimatedBgScanoutGeneration {
    // When main changes CPU/NMI phases, the NMI after that main slice belongs
    // to the following active frame. Its DMA may legitimately consume the
    // exit phase's operands, but the scanout that just completed still owns
    // the VRAM resident at the host boundary. Combining the entry scanout rule
    // with the exit operand rule creates a generation that never existed on
    // hardware.
    if entry.animated_bg_scanout != exit.animated_bg_scanout {
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    } else {
        entry.animated_bg_scanout
    }
}

const fn dungeon_supertile_scroll_nmi_precedes_link_animation(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    // Module07_02 runs Link_HandleMovingAnimation_FullLongEntry before the
    // state-specific body. State 7 then enters state 8 through the palette
    // setup, and recurring state 8 advances the scroll, but both CPU slices
    // run after the NMI that supplied the active field's Link OBJ tiles. Their
    // NMI_PrepareSprites results belong to the following field.
    entry.main_module == 7
        && entry.submodule == 2
        && matches!(entry.subsubmodule, 7 | 8)
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 8
}

const fn link_obj_operands_across_main(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    exit_operands: GraphicsDmaGeneration,
) -> GraphicsDmaGeneration {
    let entering_dungeon_spiral_stairs = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 7
        && exit.submodule == 0x0e;
    let entering_dungeon_supertile_transition = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 0;
    // From subsubmodule $01 onward, main authors the next Link OBJ source words
    // after the NMI has already consumed the host-boundary generation. Measured
    // at route frame 28837 (entry $0e/$00, exit $0e/$01): the slice advances the
    // body-pointer words $0af0/$0af2 by 0x40, and Snes9x's OBJ CHR at VRAM
    // $4220/$4320 still holds the pre-advance data. Unchanged animation frames
    // make the two generations look identical, so keep this keyed to the actual
    // phase instead of the visible four-frame animation cadence.
    let dungeon_spiral_stairs_nmi_precedes_link_animation =
        exit.main_module == 7 && exit.submodule == 0x0e && exit.subsubmodule >= 1;
    let entering_dungeon_supertile_scroll = entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 0
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 1;
    let dungeon_subtile_scroll_nmi_precedes_link_animation = entry.main_module == 7
        && entry.submodule == 1
        && matches!(entry.subsubmodule, 3..=7)
        && exit.main_module == 7
        && exit.submodule == 1;
    if entering_dungeon_spiral_stairs
        || entering_dungeon_supertile_transition
        || dungeon_spiral_stairs_nmi_precedes_link_animation
        || entering_dungeon_supertile_scroll
        || dungeon_subtile_scroll_nmi_precedes_link_animation
        || dungeon_supertile_scroll_nmi_precedes_link_animation(entry, exit)
    {
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    } else {
        exit_operands
    }
}

/// Both source facts prove the same unconditional ZeldaRunGameLoop suffix.
/// The legacy fact additionally proves that the CPU reached the main wait;
/// the narrower fact permits a following cooperative thread to run first.
const fn original_timing_receipt_completes_main_loop_common_suffix(
    receipt: &OriginalTimingSemanticReceipt,
) -> bool {
    matches!(
        receipt,
        OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait
            | OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
    )
}

/// Whether an observed Link/OAM interruption is still the caller's final
/// source boundary for this host interval.
///
/// The ordered authority may report an earlier Link/OAM interruption followed
/// by NMI publication and then the same caller's return to main wait. A later
/// caller-return receipt completes that suspended source stack and therefore
/// supersedes the earlier interruption.
const fn spotlight_caller_remains_interrupted_in_link_oam(
    caller_returned_to_main_wait: bool,
    link_oam_interruption_observed: bool,
) -> bool {
    link_oam_interruption_observed && !caller_returned_to_main_wait
}

fn try_classify_original_timing_nmi_phases(
    publication_pending_at_entry: bool,
    phases: &[OriginalTimingNmiPhase],
) -> Option<OriginalTimingNmiPhaseClassification> {
    use OriginalTimingNmiHandlerCompletionOwner::{
        AcceptedInThisSequence, None, PendingAtSequenceEntry,
    };
    let classification = |handler_completion, publication_pending_at_exit| {
        Some(OriginalTimingNmiPhaseClassification {
            handler_completion,
            publication_pending_at_exit,
        })
    };
    match (publication_pending_at_entry, phases) {
        (false, []) => classification(None, false),
        (false, [OriginalTimingNmiPhase::Accepted(_)]) => classification(None, true),
        (
            false,
            [OriginalTimingNmiPhase::Accepted(_), OriginalTimingNmiPhase::HandlerCompleted],
        ) => classification(AcceptedInThisSequence, false),
        (
            false,
            [OriginalTimingNmiPhase::Accepted(_), OriginalTimingNmiPhase::HandlerCompleted, OriginalTimingNmiPhase::Accepted(_)],
        ) => classification(AcceptedInThisSequence, true),
        (true, [OriginalTimingNmiPhase::HandlerCompleted]) => {
            classification(PendingAtSequenceEntry, false)
        }
        (true, [OriginalTimingNmiPhase::HandlerCompleted, OriginalTimingNmiPhase::Accepted(_)]) => {
            classification(PendingAtSequenceEntry, true)
        }
        _ => Option::None,
    }
}

#[track_caller]
fn classify_original_timing_nmi_phases_with_ownership(
    publication_pending_at_entry: bool,
    phases: &[OriginalTimingNmiPhase],
) -> OriginalTimingNmiPhaseClassification {
    try_classify_original_timing_nmi_phases(publication_pending_at_entry, phases)
        .unwrap_or_else(|| {
            panic!(
                "unsupported source NMI phase sequence: pending={publication_pending_at_entry} phases={phases:?}",
            )
        })
}

#[track_caller]
fn classify_original_timing_nmi_phases(
    publication_pending_at_entry: bool,
    phases: &[OriginalTimingNmiPhase],
) -> (bool, bool) {
    let classification =
        classify_original_timing_nmi_phases_with_ownership(publication_pending_at_entry, phases);
    (
        classification.handler_completion.completed(),
        classification.publication_pending_at_exit,
    )
}

fn assert_original_timing_carry_in_handler_has_receptive_display(
    classification: OriginalTimingNmiPhaseClassification,
    receptive_display_snapshot: bool,
) {
    if classification.handler_completion
        == OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry
    {
        assert!(
            receptive_display_snapshot,
            "a carry-in NMI handler requires the receptive display snapshot returned by its acceptance host",
        );
    }
}

const fn link_obj_dma_generations_for_cpu_phase(
    resumed_call_stack_is_before_nmi: bool,
    module_generation: GraphicsDmaGeneration,
) -> LinkObjDmaPhaseGenerations {
    if resumed_call_stack_is_before_nmi {
        LinkObjDmaPhaseGenerations {
            presented: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            following_nmi: GraphicsDmaGeneration::LiveAfterMain,
        }
    } else {
        LinkObjDmaPhaseGenerations {
            presented: module_generation,
            following_nmi: module_generation,
        }
    }
}

const fn dungeon_dialogue_entry_uses_host_oam_operands(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 0x0e
        && matches!(exit.submodule, 1 | 2)
        && exit.frame_counter == entry.frame_counter.wrapping_add(1)
}

const fn oam_operands_across_main(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    exit_operands: GraphicsDmaGeneration,
) -> GraphicsDmaGeneration {
    if dungeon_dialogue_entry_uses_host_oam_operands(entry, exit) {
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    } else {
        exit_operands
    }
}

const fn dungeon_gameplay_submodule_handoff_publishes_entry_shadow(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 7
        && matches!(
            exit.submodule,
            1 | 2 | 4 | 5 | 0x0a | 0x0e | 0x11 | 0x12 | 0x13
        )
}

const fn game_over_publishes_entry_obj_generation(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 0x12
        && exit.main_module == 0x12
        && ((entry.submodule == 4 && matches!(exit.submodule, 4 | 5))
            // The moving letters are authored by the main thread after this
            // frame's leading NMI. Active display therefore still owns the
            // OAM shadow captured at the host boundary, not the newly moved
            // positions prepared for the following vblank.
            || (entry.submodule == 7 && matches!(exit.submodule, 7 | 8)))
}

const fn game_over_menu_retains_resident_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    scanout_source: OamScanoutSource,
) -> bool {
    entry.main_module == 0x12
        && entry.submodule == 9
        && following.main_module == 0x12
        && following.submodule == 9
        && matches!(scanout_source, OamScanoutSource::RetainCapturedBeforeNmi)
}

fn staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    staircase_index: u8,
    link_obj_scanout: GraphicsDmaGeneration,
    link_obj_sources: GraphicsDmaGeneration,
) -> bool {
    staircase_index == 0x34
        && dungeon_gameplay_submodule_handoff_publishes_entry_shadow(entry, exit)
        && link_obj_scanout == GraphicsDmaGeneration::HostBoundaryBeforeMain
        && link_obj_sources == GraphicsDmaGeneration::HostBoundaryBeforeMain
}

fn room_82_staircase_30_gameplay_handoff_uses_live_obj_cache(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    room: u8,
    staircase_index: u8,
    link_obj_scanout: GraphicsDmaGeneration,
    link_obj_sources: GraphicsDmaGeneration,
) -> bool {
    room == 0x82
        && staircase_index == 0x30
        && dungeon_gameplay_submodule_handoff_publishes_entry_shadow(entry, exit)
        && link_obj_scanout == GraphicsDmaGeneration::HostBoundaryBeforeMain
        && link_obj_sources == GraphicsDmaGeneration::HostBoundaryBeforeMain
}

const fn oam_scanout_across_main(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    entry_scanout: OamScanoutSource,
) -> OamScanoutSource {
    // Module07 can enter the dungeon-exit spotlight module from its ordinary
    // gameplay tail. The C caller still finishes Sprite_Main/LinkOam_Main
    // before returning to ZeldaRunGameLoop, so the trailing NMI consumes that
    // completed entry shadow; Module0F's later work belongs to the next field.
    let dungeon_exit_entry_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 0x0f
        && exit.submodule == 0;
    // The lethal gameplay slice crosses its leading NMI before Death_Func1
    // advances into Module12. Snes9x has already consumed gameplay's completed
    // OAM shadow for this scanout; the death initializer's newly authored
    // coordinates belong to the following NMI.
    let dungeon_game_over_entry_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 0x12
        && exit.submodule == 1;
    // Death_Func1 finishes after the next leading NMI and advances to the
    // pre-iris delay. That NMI consumes the submodule-1 entry shadow; the
    // initializer's submodule-2 OAM becomes visible one boundary later.
    let game_over_pre_iris_entry_publishes_entry_shadow = entry.main_module == 0x12
        && entry.submodule == 1
        && exit.main_module == 0x12
        && exit.submodule == 2;
    // Module 12's post-iris fade animates Link's shadow OAM periodically, but
    // the main-thread write follows the leading NMI. The changed coordinates
    // therefore become visible on the next host boundary (oracle changes at
    // frame counters 127, 133, 139, ...), never in the slice that authors them.
    // The same entry generation owns the scanout that advances into state 5.
    // Item-receipt dismissal similarly crosses the leading NMI after its OAM
    // shadow has hidden the receipt sprites, before the main slice advances to
    // the dungeon submodule-$0a handoff.
    // A dungeon interaction enters Module0E after the leading NMI has already
    // consumed gameplay's completed OAM shadow. Dialogue initialization can
    // author the next sprite positions in the same coarse host slice, but
    // those coordinates do not become resident until the following NMI.
    let dungeon_dialogue_entry_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 0x0e
        && exit.submodule == 1;
    let dungeon_supertile_scroll_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 0
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 1;
    let dungeon_subtile_scroll_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 1
        && entry.subsubmodule == 0
        && exit.main_module == 7
        && exit.submodule == 1
        && exit.subsubmodule == 1;
    let dungeon_supertile_scroll_tail_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 8
        && exit.main_module == 7
        && exit.submodule == 2
        && matches!(exit.subsubmodule, 8 | 9);
    // Spiral stairs' first steady slice (subsubmodule 0 -> 1) authors Link's OAM
    // shadow during main, but its trailing NMI has already DMAed the pre-main
    // host-boundary generation. Retaining the *previous* frame's published shadow
    // (the ordinary RetainCapturedBeforeNmi source) leaves the scanout one
    // generation too stale once Link begins moving up the stairs; publishing this
    // frame's captured host-boundary shadow matches hardware. This is the missing
    // sibling of the submodule 1/2 `0 -> 1` publish-entry-shadow rules above.
    // Measured at route frame 28837 (entry $0e/$00 -> exit $0e/$01): every
    // diverging OAM slot matched `pre_main_graphics_dma.oam_shadow`, not the
    // stale published shadow. Bug class 5 (timed side effect on the wrong side of
    // the host boundary).
    let dungeon_spiral_stairs_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 0x0e
        && entry.subsubmodule == 0
        && exit.main_module == 7
        && exit.submodule == 0x0e
        && exit.subsubmodule == 1;
    if dungeon_exit_entry_publishes_entry_shadow
        || dungeon_game_over_entry_publishes_entry_shadow
        || game_over_pre_iris_entry_publishes_entry_shadow
        || game_over_publishes_entry_obj_generation(entry, exit)
        || dungeon_gameplay_submodule_handoff_publishes_entry_shadow(entry, exit)
        || dungeon_dialogue_entry_publishes_entry_shadow
        || dungeon_subtile_scroll_publishes_entry_shadow
        || dungeon_supertile_scroll_publishes_entry_shadow
        || dungeon_supertile_scroll_tail_publishes_entry_shadow
        || dungeon_spiral_stairs_publishes_entry_shadow
    {
        OamScanoutSource::ComposePublishedShadowDma
    } else {
        entry_scanout
    }
}

const fn rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    hud_upload_pending: bool,
) -> bool {
    // Room $01's short spiral state-8 slice completes the queued HUD DMA
    // before active scanout. The new floor label is therefore visible while
    // the remaining VRAM domains still retain their pre-NMI generation.
    hud_upload_pending
        && dungeon_room == 0x01
        && frame.main_module == 7
        && frame.submodule == 0x0e
        && frame.subsubmodule == 8
}

fn resumed_dungeon_spiral_state_7_publishes_audio_after_main(
    resume: PreMainNmiResume,
    dungeon_room: u8,
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    // The suspended spiral-graphics caller returns into state 7 behind a
    // leading display NMI. That resumed slice changes floors and authors the
    // stair blip before the same host boundary samples the audio ports. Keep
    // this exception tied to the measured continuation and phase transition;
    // ordinary state-7 iterations still use the pre-main audio boundary.
    matches!(resume, PreMainNmiResume::DungeonSupertileQuadrantUploads)
        && dungeon_room == 1
        && entry.main_module == 7
        && entry.submodule == 0x0e
        && entry.subsubmodule == 7
        && exit.main_module == 7
        && exit.submodule == 0x0e
        && exit.subsubmodule == 8
}

const fn dungeon_subtile_landing_enters_shutter(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 7
        && entry.submodule == 1
        && matches!(entry.subsubmodule, 3..=7)
        && exit.main_module == 7
        && exit.submodule == 5
}

fn room_71_subtile_shutter_publishes_live_link_head_obj_cache(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    link_dma_countdown: u16,
) -> bool {
    dungeon_room == 0x71
        && entry.main_module == 7
        && entry.submodule == 1
        && entry.subsubmodule == 7
        && following.main_module == 7
        && following.submodule == 5
        && following.subsubmodule == 0
        && link_dma_countdown == 5
}

const fn room_72_northward_subtile_palette_tail_uses_live_obj_cache(
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    link_last_direction: u8,
) -> bool {
    dungeon_room == 0x72
        && following.main_module == 7
        && following.submodule == 1
        && following.subsubmodule == 7
        && link_last_direction & 0x0f == 8
}

const fn room_72_northward_subtile_shutter_retains_presented_obj_cache(
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    link_last_direction: u8,
) -> bool {
    dungeon_room == 0x72
        && following.main_module == 7
        && following.submodule == 5
        && following.subsubmodule == 0
        && link_last_direction & 0x0f == 8
}

const fn straight_interroom_fadeout_uses_live_decoded_obj_cache(
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    // The straight-stair palette caller can cross vblank without publishing
    // the new raw OBJ page. Snes9x nevertheless invalidates and re-decodes the
    // Link tiles uploaded by that NMI, so scanout owns a live renderer cache
    // independently from the retained raw-VRAM generation.
    dungeon_room == 0x51
        && following.main_module == 7
        && following.submodule == 0x12
        && following.subsubmodule == 1
        && staircase_index == 0x30
}

const fn straight_interroom_fadeout_main_slice_publishes_host_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
    scanout_source: OamScanoutSource,
) -> bool {
    straight_interroom_fadeout_uses_live_decoded_obj_cache(following, dungeon_room, staircase_index)
        && entry.frame_counter != following.frame_counter
        && matches!(scanout_source, OamScanoutSource::RetainResidentPpuOam)
}

const fn straight_interroom_fadeout_no_nmi_caller_retains_presented_display(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
    nmi_update_latch: u8,
    link_obj_scanout_generation: GraphicsDmaGeneration,
) -> bool {
    straight_interroom_fadeout_uses_live_decoded_obj_cache(following, dungeon_room, staircase_index)
        && entry.frame_counter == following.frame_counter
        && nmi_update_latch == 0
        && matches!(
            link_obj_scanout_generation,
            GraphicsDmaGeneration::LiveAfterMain
        )
}

const fn straight_interroom_post_sprite_graphics_uses_host_link_obj_cache(
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    // The sprite-sheet workload returns before the Link DMA tail finishes.
    // Snes9x has decoded the early body/head/hand batch using the operands
    // captured at the host boundary, while raw OBJ VRAM remains independently
    // retained through the following palette states.
    dungeon_room == 0x51
        && following.main_module == 7
        && following.submodule == 0x12
        && matches!(following.subsubmodule, 0x0d..=0x0f)
        && staircase_index == 0x30
}

const fn dungeon_dialogue_render_entry_uses_host_link_obj_cache(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
) -> bool {
    // Gameplay's leading NMI has already decoded the early Link DMA batch
    // before dialogue initialization advances directly into RenderText. The
    // active OAM/raw-VRAM generation remains independently selected, so only
    // the renderer's OBJ cache consumes the host-boundary source operands.
    entry.main_module == 7
        && entry.submodule == 0
        && following.main_module == 0x0e
        && following.submodule == 2
}

const fn straight_interroom_quadrant_pipeline_publishes_host_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    dungeon_room == 0x51
        && staircase_index == 0x30
        && entry.main_module == 7
        && entry.submodule == 0x12
        && matches!(entry.subsubmodule, 0x0c..=0x0e)
        && following.main_module == 7
        && following.submodule == 0x12
        && following.subsubmodule == entry.subsubmodule + 1
}

const fn straight_interroom_palette_caller_retains_presented_display(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    // State $0f alternates completed palette uploads with caller-only host
    // slices. An unchanged frame counter means no new CGRAM upload reached
    // scanout even though the translated palette buffer already advanced.
    dungeon_room == 0x51
        && following.main_module == 7
        && following.submodule == 0x12
        && following.subsubmodule == 0x0f
        && staircase_index == 0x30
        && entry.frame_counter == following.frame_counter
}

const fn straight_interroom_palette_completion_retains_presented_cgram(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    // The final palette workload slice advances the CPU state to $10 after
    // the host frame's NMI has already sampled CGRAM. Its completed palette
    // therefore becomes visible on the next scanout, not this one.
    dungeon_room == 0x51
        && staircase_index == 0x30
        && entry.main_module == 7
        && entry.submodule == 0x12
        && entry.subsubmodule == 0x0f
        && following.main_module == 7
        && following.submodule == 0x12
        && following.subsubmodule == 0x10
}

const fn straight_interroom_palette_filter_retains_presented_oam(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 0x12
        && frame.subsubmodule == 5
        && dungeon_room == 0x51
        && staircase_index == 0x30
}

const fn straight_interroom_palette_filter_retains_captured_oam(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 0x12
        && matches!(frame.subsubmodule, 6..=8)
        && dungeon_room == 0x51
        && staircase_index == 0x30
}

fn publish_live_link_head_obj_cache(captured: &mut [u16], resident: &[u16]) {
    if captured.len() < 0x4140 || resident.len() < 0x4140 {
        return;
    }
    for range in [0x4020..0x4040, 0x4120..0x4140] {
        captured[range.clone()].copy_from_slice(&resident[range]);
    }
}

const fn link_obj_scanout_across_main(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    entry_scanout: GraphicsDmaGeneration,
) -> GraphicsDmaGeneration {
    // The post-iris fade's OAM coordinates and Link's decoded OBJ tiles are a
    // single leading-NMI generation. Retaining only one side produces a
    // coherent table with the wrong animation pixels.
    if game_over_publishes_entry_obj_generation(entry, exit) {
        return GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    // Gameplay enters straight inter-room stairs after the leading NMI, then
    // state 0 completes the first Link upload before active OBJ evaluation.
    // OAM still belongs to gameplay's entry shadow, so keep this explicit as
    // a Link-only live publication edge.
    if entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 7
        && exit.submodule == 0x12
    {
        return GraphicsDmaGeneration::LiveAfterMain;
    }
    // The leading NMI consumes both the OAM shadow and Link's decoded OBJ
    // operands before gameplay advances into a dungeon transition submodule.
    // Keep these independent publication lanes on the same measured edge.
    if dungeon_gameplay_submodule_handoff_publishes_entry_shadow(entry, exit) {
        return GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    // The state-7 palette entry and recurring state-8 scroll both run after
    // the NMI that supplied the active field's Link tiles.
    if dungeon_supertile_scroll_nmi_precedes_link_animation(entry, exit) {
        return GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    // The subtile landing tail enters room-load/shutter control after the
    // leading NMI. That scanout keeps the Link tiles resident at host entry,
    // even though the independently scheduled animated-BG upload is live.
    if dungeon_subtile_landing_enters_shutter(entry, exit) {
        return GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    // State 1 retains the host-boundary Link tiles throughout the supertile
    // scroll. Its final main slice advances to state 2 after the NMI has
    // completed the live upload, so that exit scanout owns the live raw OBJ
    // generation instead of carrying state 1's retention one frame farther.
    if entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 1
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 2
    {
        GraphicsDmaGeneration::LiveAfterMain
    } else {
        entry_scanout
    }
}

const fn dialogue_text_frame_holds_published_oam(
    published: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
    captured_text_render_state: u8,
) -> bool {
    published.main_module == 14
        && published.submodule == 2
        && captured.main_module == 14
        && captured.submodule == 2
        && captured_text_render_state == 3
        && published.frame_counter == captured.frame_counter
}

const fn rom_dungeon_exit_entry_crosses_nmi_boundary(
    snapshot_main_module: u8,
    snapshot_submodule: u8,
    live_main_module: u8,
    live_submodule: u8,
    spotlight_entry_build_in_flight: bool,
) -> bool {
    // The first Module 15/submodule 0 boundary only authors the doorway scroll
    // and hidden-sprite shadow for the next NMI. Once the interruptible
    // spotlight-entry build is in flight, that trailing NMI has published both
    // domains. The completed build's return into submodule 1 owns the same live
    // generation.
    snapshot_main_module == 0x0f
        && snapshot_submodule == 0
        && live_main_module == 0x0f
        && (live_submodule == 1 || (live_submodule == 0 && spotlight_entry_build_in_flight))
}

const fn rom_dungeon_falling_entry_retains_published_obj_generation(
    published_main_module: u8,
    published_submodule: u8,
    current_main_module: u8,
    current_submodule: u8,
) -> bool {
    // The overworld main loop hides the pit marker in its next OAM shadow and
    // then switches to Module 11. Snes9x returns at the intervening vblank
    // before that shadow reaches hardware, so the falling-entrance entry
    // scanout still owns the OAM generation published by Module 9.
    published_main_module == 9
        && published_submodule == 0
        && current_main_module == 0x11
        && current_submodule == 0
}

const fn interface_exit_bg_upload_misses_current_scanout(
    entry_main_module: u8,
    current_main_module: u8,
    bg_vram_upload_is_pending: bool,
) -> bool {
    // Interface modules author their exit stripe after the active frame has
    // started. The next vblank consumes it: for example, the save-menu erase
    // is armed at V=42 while Snes9x is already scanning out the final menu
    // frame, then reaches NMI at V=225.
    entry_main_module == 0x0e && current_main_module != 0x0e && bg_vram_upload_is_pending
}

const fn rom_overworld_bad_weather_scroll_is_live(
    snapshot_main_module: u8,
    snapshot_submodule: u8,
    live_main_module: u8,
    live_submodule: u8,
    snapshot_bg1_h: u16,
    snapshot_bg1_v: u16,
    snapshot_bg2_h: u16,
    snapshot_bg2_v: u16,
    live_bg1_h: u16,
    live_bg1_v: u16,
    live_bg2_h: u16,
    live_bg2_v: u16,
) -> bool {
    // The resumed Module09 caller suffix runs OverworldOverlay_HandleRain,
    // which shakes only BG1 by $0100/$1100. Snes9x publishes that weather
    // scroll on the first submodule-6 scanout. A real transition step moves
    // BG1 and BG2 together and must keep the ordinary coherent snapshot cadence.
    snapshot_main_module == 9
        && snapshot_submodule == 6
        && live_main_module == 9
        && live_submodule == 6
        && (snapshot_bg1_h != live_bg1_h || snapshot_bg1_v != live_bg1_v)
        && snapshot_bg2_h == live_bg2_h
        && snapshot_bg2_v == live_bg2_v
}

const fn rom_dungeon_item_hold_to_dialogue_publishes_live_animated_bg(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
) -> bool {
    // The item-hold release enters the dialogue module after the leading NMI
    // has already uploaded the next dungeon animated-tile batch. OAM and
    // ordinary VRAM retain their independent generations; only the animated
    // BG DMA is live for this first module-14 scanout.
    entry.main_module == 7
        && entry.submodule == 0
        && captured.main_module == 14
        && captured.submodule == 2
}

const fn rom_dungeon_subtile_return_publishes_live_animated_bg(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
) -> bool {
    // The last subtile landing slice enters room-load/shutter control after
    // the leading NMI has already uploaded the next dungeon animated-tile
    // batch. The module change cannot move that completed DMA behind main.
    dungeon_subtile_landing_enters_shutter(entry, captured)
}

const fn rom_dungeon_subtile_direction_one_publishes_live_animated_bg(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
    screen_transition: u8,
) -> bool {
    // Direction $01's subtile movement runs after the leading NMI has
    // uploaded the next dungeon animation page. Direction $00 reaches the
    // same coarse state before that upload owns scanout, so the direction is
    // part of the hardware publication boundary rather than incidental input.
    screen_transition == 1
        && entry.main_module == 7
        && entry.submodule == 1
        && matches!(entry.subsubmodule, 1..=7)
        && captured.main_module == 7
        && captured.submodule == 1
        && matches!(captured.subsubmodule, 1..=7)
}

const fn rom_room_82_sprite_conversion_defers_trailing_nmi(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    room: u8,
) -> bool {
    // Room $82's state-$02 filter returns after the current host scanout but
    // before the following vblank. Snes9x therefore keeps $12 clear and the
    // resident Link OBJ page visible on this boundary; the deferred NMI runs
    // during the first state-$03 host slice and uploads the newly authored
    // Link operands.
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 2
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 3
}

const fn rom_room_82_deferred_nmi_retains_resident_oam(
    entry: crate::game_state::FrameState,
    room: u8,
    nmi_update_is_latched: bool,
    screen_transition: u8,
) -> bool {
    // The first state-$03 slice of the direction-$00 transition owns the
    // vblank deferred by state $02, but its active OBJ list remains on the
    // resident state-$02 OAM generation. The horizontal direction-$02 path
    // completes OAM DMA during that deferred vblank and must publish the new
    // sorted table. Later state-$03 slices enter with $12 set and need no
    // override.
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 3
        && !nmi_update_is_latched
        && screen_transition == 0
}

const fn room_82_horizontal_deferred_nmi_publishes_entry_shadow_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    room: u8,
    screen_transition: u8,
    deferred_nmi: bool,
    oam_scanout_source: OamScanoutSource,
) -> bool {
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 2
        && following.main_module == 7
        && following.submodule == 2
        && following.subsubmodule == 3
        && screen_transition == 2
        && deferred_nmi
        && matches!(oam_scanout_source, OamScanoutSource::ComposeLiveAfterNmi)
}

const fn room_82_horizontal_state3_followup_publishes_entry_shadow_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    room: u8,
    screen_transition: u8,
    deferred_nmi: bool,
    oam_scanout_source: OamScanoutSource,
) -> bool {
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 3
        && following.main_module == 7
        && following.submodule == 2
        && following.subsubmodule == 3
        && screen_transition == 2
        && !deferred_nmi
        && matches!(oam_scanout_source, OamScanoutSource::RetainResidentPpuOam)
}

const fn room_82_horizontal_sprite_conversion_return_retains_last_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    room: u8,
    screen_transition: u8,
    oam_scanout_source: OamScanoutSource,
) -> bool {
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && matches!(entry.subsubmodule, 3 | 4)
        && following.main_module == 7
        && following.submodule == 2
        && following.subsubmodule == 4
        && screen_transition == 2
        && matches!(
            oam_scanout_source,
            OamScanoutSource::RetainResidentPpuOam | OamScanoutSource::ComposeLiveAfterNmi
        )
}

const fn room_82_horizontal_quadrant_filter_entry_publishes_host_boundary_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    room: u8,
    screen_transition: u8,
    oam_scanout_source: OamScanoutSource,
) -> bool {
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 5
        && following.main_module == 7
        && following.submodule == 2
        && following.subsubmodule == 6
        && screen_transition == 2
        && matches!(
            oam_scanout_source,
            OamScanoutSource::ComposePublishedShadowDma
        )
}

const fn room_82_horizontal_first_scroll_publishes_host_boundary_oam(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    room: u8,
    screen_transition: u8,
    oam_scanout_source: OamScanoutSource,
    retain_captured_oam: bool,
) -> bool {
    room == 0x82
        && entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 7
        && following.main_module == 7
        && following.submodule == 2
        && following.subsubmodule == 8
        && screen_transition == 2
        && matches!(oam_scanout_source, OamScanoutSource::ComposeLiveAfterNmi)
        && retain_captured_oam
}

const fn rom_dungeon_module_iteration_runs_after_leading_nmi(
    frame: crate::game_state::FrameState,
    room: u8,
) -> bool {
    if frame.main_module != 7 {
        return false;
    }

    // Room $41's final supertile state returns on the preceding host boundary.
    // State 15 and its shutter-control tail then run after each host's leading
    // NMI, leaving the ordinary game-loop epilogue's cleared $12 visible.
    if room == 0x41 && ((frame.submodule == 2 && frame.subsubmodule == 15) || frame.submodule == 5)
    {
        return true;
    }

    if frame.submodule != 2 {
        return false;
    }

    // Room $61 returns from its long room-load caller at the preceding NMI.
    // The next host call consumes the ordinary leading NMI, then completes one
    // fresh state-$02 iteration without reaching another vblank.
    if room == 0x61 && frame.subsubmodule == 2 {
        return true;
    }

    // Once state 8 is active, vblank publishes the scroll authored by the
    // preceding iteration and the CPU then prepares the next 4-pixel step.
    // Rooms $71/$72 use their explicit interrupted-return scheduler instead.
    frame.subsubmodule == 8 && !matches!(room, 0x71 | 0x72)
}

fn post_landing_player_control_runs_after_leading_nmi(
    cadence_room: Option<u8>,
    frame: crate::game_state::FrameState,
    room: u8,
) -> bool {
    cadence_room == Some(room) && frame.main_module == 7 && frame.submodule == 0
}

const fn straight_interroom_upload_pipeline_runs_after_leading_nmi(
    frame: crate::game_state::FrameState,
    leading_nmi_upload_pipeline_active: bool,
) -> bool {
    // Destination prep, the three quadrant-build states, and the initial idle
    // palette slice run after their host's leading NMI. Each CPU state leaves
    // its newly queued VRAM upload pending for the following boundary; the
    // active scanout still owns the display captured before that leading NMI.
    leading_nmi_upload_pipeline_active
        && frame.main_module == 7
        && frame.submodule == 0x12
        && matches!(frame.subsubmodule, 0x0b..=0x0f)
}

const fn game_over_upload_pipeline_runs_after_leading_nmi(
    frame: crate::game_state::FrameState,
    menu_animation_timer: u8,
    palette_filter_countdown: u8,
    pending_nmi_subroutine: u8,
) -> bool {
    if frame.main_module != 0x12 {
        return false;
    }
    match frame.submodule {
        // The final subtractive palette step queues the GAME OVER tilemap DMA
        // after this host's leading vblank.
        5 => {
            menu_animation_timer == 0
                && palette_filter_countdown == 31
                && pending_nmi_subroutine == 0
        }
        // State 6 begins only after that tilemap DMA, then queues the half-slot
        // character upload for the following leading vblank.
        6 => pending_nmi_subroutine == 22,
        // The first letter-animation iteration starts only after the queued
        // character half-slot is resident.
        7 => pending_nmi_subroutine == 11,
        _ => false,
    }
}

const ANIMATED_TILE_BUFFER_FIRST_SOURCE: usize = 0xa680;
const ANIMATED_TILE_SOURCE_CYCLE_BYTES: usize = 0x0c00;

const fn rom_spiral_stairs_suspended_animated_bg_source_address(
    frame: crate::game_state::FrameState,
    host_main_prefix_did_not_advance: bool,
    countdown: u16,
    current_source: usize,
) -> Option<usize> {
    if frame.main_module == 7
        && frame.submodule == 0x0e
        && host_main_prefix_did_not_advance
        && countdown == 1
        && current_source >= ANIMATED_TILE_BUFFER_FIRST_SOURCE
        && current_source < ANIMATED_TILE_BUFFER_FIRST_SOURCE + ANIMATED_TILE_SOURCE_CYCLE_BYTES
    {
        Some(
            ANIMATED_TILE_BUFFER_FIRST_SOURCE
                + (current_source - ANIMATED_TILE_BUFFER_FIRST_SOURCE + 0x400)
                    % ANIMATED_TILE_SOURCE_CYCLE_BYTES,
        )
    } else {
        None
    }
}

const fn rom_display_snapshot_publication(
    main_module: u8,
    submodule: u8,
) -> DisplaySnapshotPublication {
    // The dungeon-exit entry setup authors its first circle before NMI enables
    // the window controls, so retain the preceding display once for submodule
    // zero. During the active close, Snes9x PC/V-counter traces show the ROM
    // rebuilding the table while HDMA consumes it in that same scanout; those
    // submodule-one frames must publish the live table instead.
    //
    // The landing wipe and overworld-entry open retain their independently
    // measured following-frame publication boundaries.
    if rom_dungeon_landing_wipe_is_active(main_module, submodule)
        || (main_module == 0x0f && submodule == 0)
        || (main_module == 0x10 && submodule == 1)
    {
        DisplaySnapshotPublication::AdvanceStaged
    } else {
        DisplaySnapshotPublication::PublishCaptured
    }
}

const fn rom_attract_world_map_display_is_one_frame_deferred(
    main_module: u8,
    submodule: u8,
    sequence: u8,
    attract_state: u8,
) -> bool {
    main_module == 20 && submodule == 0 && sequence == 1 && attract_state >= 4
}

const fn rom_attract_world_map_mode7_brightness_is_early_published(
    main_module: u8,
    submodule: u8,
    sequence: u8,
    attract_state: u8,
) -> bool {
    // Only Attract_FadeInSequence publishes the new INIDISP step ahead of the
    // deferred CGRAM generation. Once Attract_EnactStory takes over (state 5),
    // the presented PPU brightness owns scanout again.
    main_module == 20 && submodule == 0 && sequence == 1 && attract_state == 4
}

const fn rom_intro_wait_player_tears_down_poly_thread(
    main_module: u8,
    submodule: u8,
    nmi_thread_active: bool,
) -> bool {
    main_module == 0 && submodule == 8 && nmi_thread_active
}

const fn legacy_poly_scheduler_is_active(
    bugs_fixed: u8,
    timed_poly_worker_active: bool,
    nmi_thread_active: bool,
) -> bool {
    bugs_fixed < BUGFIX_POLY_RENDERER && !timed_poly_worker_active && nmi_thread_active
}

const fn rom_file_select_teardown_runs_with_outgoing_poly_worker(
    main_module: u8,
    submodule: u8,
    nmi_thread_active: bool,
    nmi_thread_uses_poly_stack: bool,
) -> bool {
    main_module == 1 && submodule == 0 && nmi_thread_active && nmi_thread_uses_poly_stack
}

const fn rom_intro_title_fade_runs_main(poly_phase: u8) -> bool {
    poly_phase < 2
}

const fn rom_intro_title_fade_should_yield_suffix(poly_phase: u8) -> bool {
    poly_phase == 1
}

const fn rom_intro_bg_fade_main_decision(carry_frames: u8, poly_phase: u8) -> (bool, bool, u8, u8) {
    if carry_frames < 2 {
        (true, false, carry_frames + 1, poly_phase)
    } else {
        (
            poly_phase < 4,
            poly_phase == 3,
            carry_frames,
            (poly_phase + 1) % 5,
        )
    }
}

const fn rom_intro_bg_fade_should_yield_suffix(
    scheduled_yield: bool,
    sword_animation_step: u8,
    sword_sparkle_step: u8,
) -> bool {
    scheduled_yield && sword_animation_step == 2 && sword_sparkle_step >= 4
}

const fn rom_intro_poly_init_decision(phase: u8) -> (bool, bool, u8) {
    match phase {
        3 => (true, false, 2),
        2 => (false, false, 1),
        1 => (false, true, 0),
        _ => (false, false, 0),
    }
}

const fn rom_attract_init_graphics_decision(phase: u8) -> (bool, u8) {
    match phase {
        4 => (false, 3),
        3 => (false, 2),
        2 => (true, 1),
        1 => (false, 0),
        _ => (false, 0),
    }
}

const fn rom_attract_story_render_nmi_slices(sequence: u8) -> u8 {
    // The throne-room loader returns on the NMI boundary that also consumes
    // the first story continuation slice. The opening polka-dot sequence has
    // an additional NMI boundary between text initialization and its first
    // character render.
    match sequence {
        0 => 7,
        2 => 5,
        _ => 6,
    }
}

const fn rom_item_receipt_graphics_nmi_slices(gfx: u8) -> u8 {
    match load_gfx::animated_sprite_tile_secondary_sheet(gfx) {
        // Timing belongs to the compressed sheets, not an individual route's
        // item ID. $5b, $5c, and the separately packed $5d (the
        // heart-container receipt, route host 102905) all cross four NMI
        // boundaries.
        0x5b | 0x5c | 0x5d => ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
        _ => 0,
    }
}

const SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES: u8 = 1;
const DUNGEON_EXIT_SPOTLIGHT_GOAL_CALLER_NMI_SLICES: u8 = 2;
const TRAILING_NMI_FORCE_BLANK_SCANLINE: u8 = 224;
// PreOverworld_LoadProperties enters at $02:83c7. The C call stack reaches
// Sprite_DisableAll's sprite-state clear at $09:c244 after 37 NMI crossings;
// the uninterrupted reload prefix then resets the old sprite workspace. Three
// more crossings load/activate overworld sprites and complete the caller tail.
const PRE_OVERWORLD_PROPERTIES_TO_SPRITE_RESET_NMI_SLICES: u8 = 37;
const PRE_OVERWORLD_PROPERTIES_AFTER_SPRITE_RESET_NMI_SLICES: u8 = 3;
const PRE_OVERWORLD_OVERLAYS_NMI_SLICES: u8 = 6;
const WORLD_MAP_LIGHT_LOAD_NMI_SLICES: u8 = 5;
// Attract_DramatizeWorldMap enters the tilemap erase immediately after
// vblank, but the clear crosses the next scanout boundary before the caller
// can advance the attract sequence.
const ATTRACT_WORLD_MAP_EXIT_NMI_SLICES: u8 = 1;
const OVERWORLD_SPRITE_RECORD_TIMING_UNITS: usize = 3;
const OVERWORLD_SPRITE_RELOAD_SAME_FRAME_BUDGET_UNITS: usize = 39;
// From the $02:af1e -> $02:fd0d `Overworld_LoadOverlays2` trace: the overlay
// map32 decode and Map16ToMap8 conversion cross four NMI boundaries before
// returning to the Module09 caller. Live semantic return authority may finish
// earlier or later; this is only the standalone native fallback.
const OVERWORLD_LOAD_OVERLAYS_OVERLAY_NMI_SLICES: u8 = 4;

const fn overworld_aux_graphics_timing(
    workload: OverworldAuxGraphicsWorkload,
) -> OverworldAuxGraphicsTiming {
    // Sprite decompression and conversion consume the eleven-boundary base.
    // Each nonzero auxiliary background pack adds one $600-byte decompression;
    // clean Snes9x traces measure two additional NMI slices per pack.
    OverworldAuxGraphicsTiming {
        load_nmi_slices: 11 + workload.background_packs_to_decompress as u8 * 2,
    }
}

const fn overworld_map_and_sprite_graphics_timing(
    workload: OverworldMapGraphicsWorkload,
) -> OverworldMapAndSpriteGraphicsTiming {
    // The four quadrants always expand 1,024 map32 cells. The expensive branch
    // reloads twelve map16 definition bytes whenever the aligned definition
    // changes. PC/V-counter traces measure 670 changes on screen $1b as 13
    // boundaries and 796 changes on screen $2b as 14. The C callers then
    // diverge: Overworld_LoadAmbientOverlay runs Map16ToMap8 and returns after
    // three crossings, while Module09_LoadNewMapAndGFX runs
    // CreateInitialNewScreenMapToScroll plus LoadNewSpriteGFXSet and returns
    // after four. Keep those distinct call paths explicit instead of sharing
    // a route-dependent tail adjustment.
    OverworldMapAndSpriteGraphicsTiming {
        quadrant_load_nmi_slices: 8 + (workload.map32_definition_changes / 128) as u8,
        map16_to_map8_tail_nmi_slices: 3,
        scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
    }
}

const fn pre_main_caller_uses_host_boundary_shadow_oam(
    continuation: Option<PreMainCallerContinuation>,
    palette_filter_countdown: u8,
) -> bool {
    matches!(
        continuation,
        Some(PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass { .. })
    ) && palette_filter_countdown == 1
}

const fn dungeon_faded_filter_first_pass_oam_scanout() -> OamScanoutSource {
    // The first palette walk suspends the Module 7 stack before Sprite_Main.
    // Its cleared live OAM shadow therefore belongs to the following CPU
    // generation; scanout keeps the completed shadow DMA prepared by the
    // preceding resumed suffix on every interrupted walk.
    OamScanoutSource::RetainCapturedBeforeNmi
}

const fn interrupted_dungeon_faded_filter_uses_host_link_obj_cache(
    frame: crate::game_state::FrameState,
    continuation: Option<PreMainCallerContinuation>,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 2
        && matches!(frame.subsubmodule, 2 | 14)
        && matches!(
            continuation,
            Some(PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass { .. })
        )
}

fn dungeon_supertile_quadrant_cpu_advance_for_resume(
    resume: PreMainNmiResume,
    state: &ZeldaState,
) -> (
    Option<DungeonQuadrantCpuAdvance>,
    Option<DungeonModuleCpuAdvance>,
) {
    let frame = state.game_state.frame;
    if !matches!(resume, PreMainNmiResume::DungeonSupertileQuadrantUploads)
        || frame.main_module != 7
        || frame.submodule != 2
    {
        return (None, None);
    }
    if matches!(frame.subsubmodule, 4..=7) {
        // Start at the live main-thread wait and execute the real leading NMI,
        // main-loop prefix, Module 7 body, and caller continuously. A fixed
        // post-handler raster envelope loses several scanlines of provenance
        // when the preceding caller reached WAI at a different CPU phase.
        let exact = begin_dungeon_module_cpu_advance_after_leading_nmi(state)
            .expect("dungeon quadrant phase was validated above");
        let coarse = match exact.phase {
            ModuleCpuPhase::InterruptedBeforeSubmodule | ModuleCpuPhase::InterruptedInSubmodule => {
                DungeonQuadrantCpuAdvance::InterruptedInModule
            }
            ModuleCpuPhase::InterruptedAfterModule => {
                DungeonQuadrantCpuAdvance::InterruptedAfterModule
            }
            ModuleCpuPhase::CompleteBeforeNmi
            | ModuleCpuPhase::InterruptedBeforeSpriteMain
            | ModuleCpuPhase::InterruptedInSpriteMain
            | ModuleCpuPhase::InterruptedAfterSpriteMain
            | ModuleCpuPhase::InterruptedInLinkOam
            | ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites
            | ModuleCpuPhase::InterruptedInNmiPrepareSprites => {
                DungeonQuadrantCpuAdvance::CompleteBeforeNmi
            }
        };
        if crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
            eprintln!(
                "dungeon_quadrant_cpu_advance host={} room={:04x} state={} phase={:?} resumed={:?} sprite_boundary={:?} cached_boundary={:?}",
                state.frame_ctr_dbg,
                state.game_state.world.location.dungeon_room_index(),
                frame.subsubmodule,
                exact.phase,
                exact.resumed_phase,
                exact.sprite_main_boundary,
                exact.cached_sprite_interruption,
            );
        }
        return (Some(coarse), Some(exact));
    }

    // The filtered state-14 completion has a distinct entry phase and is not
    // part of the ordinary state-4..7 quadrant dispatcher model above.
    (
        (state.game_state.world.location.dungeon_room_index() == 0x22 && frame.subsubmodule == 14)
            .then_some(DungeonQuadrantCpuAdvance::InterruptedInModule),
        None,
    )
}

// Instrumented 65816 traces put the ordinary dungeon state dispatcher at
// V=248 after the leading NMI. The state-10 filtered path starts four lines
// later because it resumes through the preceding caller-only boundary.
const DUNGEON_STATE_9_CPU_ENTRY: CpuRasterPosition = CpuRasterPosition::new(248, 1_258);
const DUNGEON_STATE_10_CPU_ENTRY: CpuRasterPosition = CpuRasterPosition::new(252, 312);
const DUNGEON_STATE_11_CPU_ENTRY: CpuRasterPosition = CpuRasterPosition::new(252, 338);
// After an interrupted subtile palette walk resumes and returns, the next
// fresh Module 7 iteration reaches the common main-loop dispatcher here.
// Instrumented Snes9x traces put repeated state-1 entries at V=254,
// master-cycle 628..668; the small input-dependent prefix variation cannot
// cross either measured palette/return boundary.
const DUNGEON_SUBTILE_PALETTE_CPU_ENTRY: CpuRasterPosition = CpuRasterPosition::new(254, 640);
const DUNGEON_QUADRANT_PREPARE_MASTER_CYCLES: u32 = 123_464;
const DUNGEON_STATE_9_FILTERED_FIXED_MASTER_CYCLES: u32 = 126_342;
const DUNGEON_STATE_10_FILTERED_FIXED_MASTER_CYCLES: u32 = 126_380;
const DUNGEON_STATE_11_FILTERED_FIXED_MASTER_CYCLES: u32 = 124_024;
const DUNGEON_MODULE_CALLER_SUFFIX_MASTER_CYCLES: u32 = 59_584;
const DUNGEON_HDMA_STALL_MASTER_CYCLES: u16 = 42;

const PALETTE_FILTER_ZERO_AUX_WORD_MASTER_CYCLES: u32 = 224;
const PALETTE_FILTER_NONZERO_AUX_EXTRA_MASTER_CYCLES: u32 = 678;
const PALETTE_FILTER_RED_STEP_EXTRA_MASTER_CYCLES: u32 = 90;
const PALETTE_FILTER_GREEN_STEP_EXTRA_MASTER_CYCLES: u32 = 90;
const PALETTE_FILTER_BLUE_STEP_EXTRA_MASTER_CYCLES: u32 = 104;

const DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    // Begin at the real frame-counter increment immediately before the OAM
    // clear and module router. Starting at $8053 left the ROM shadow's $1a one
    // frame stale and hardcoded the flags produced by this increment. Both
    // change Sprite_Main's variable workload, so the entire caller phase must
    // derive them from live WRAM instead.
    entry_pc: 0x00_8051,
    stop_pc: 0x00_8036,
    a: 0xb701,
    x: 0,
    y: 20,
    sp: 0x01ff,
    dp: 0,
    db: 0,
    carry: false,
    zero: false,
    overflow: false,
    negative: false,
    interrupt_disable: false,
    decimal: false,
    accumulator_is_8_bit: true,
    index_is_8_bit: true,
    emulation: false,
    waiting: false,
    stack_address: 0,
    stack_bytes: &[],
};

const DUNGEON_MAIN_WAIT_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    // The game thread is asleep in `LDA $12 / BEQ` when vblank asserts NMI.
    // NMI returns here, the loop observes the handler's set latch, and only
    // then does execution branch to the frame-counter increment at $8051.
    entry_pc: 0x00_8034,
    a: 0xb700,
    zero: true,
    waiting: true,
    ..DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT
};

const DUNGEON_EXIT_SPOTLIGHT_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    // Module0F_SpotlightClose at the indirect module-router call. Cold Snes9x
    // traces at both exercised exits enter with this register/stack image; the
    // saved return address is the final byte of JSL [$0080], so RTL stops at
    // $00:805a.
    entry_pc: 0x02_9982,
    stop_pc: 0x00_805a,
    a: 0xb702,
    x: 0x00e0,
    y: 0x000f,
    sp: 0x01fc,
    dp: 0,
    db: 0,
    carry: false,
    zero: false,
    overflow: false,
    negative: false,
    interrupt_disable: false,
    decimal: false,
    accumulator_is_8_bit: true,
    index_is_8_bit: true,
    emulation: false,
    waiting: false,
    stack_address: 0x01fd,
    stack_bytes: &[0x59, 0x80, 0x00],
};

const OVERWORLD_SPOTLIGHT_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    // Begin at ZeldaRunGameLoop's frame-counter increment, before ClearOamBuffer
    // and the module router. Sprite_Main's cycle count depends on both the new
    // frame counter and the OAM allocation state, so beginning at Module10
    // would inherit translated scratch state instead of executing the C caller.
    entry_pc: 0x00_8051,
    stop_pc: 0x00_805a,
    a: 0xb701,
    x: 0,
    y: 0x0010,
    sp: 0x01ff,
    dp: 0,
    db: 0,
    carry: false,
    zero: false,
    overflow: false,
    negative: false,
    interrupt_disable: false,
    decimal: false,
    accumulator_is_8_bit: true,
    index_is_8_bit: true,
    emulation: false,
    waiting: false,
    stack_address: 0,
    stack_bytes: &[],
};

// Cold traces at both standard-route openings reach ZeldaRunGameLoop's $8051
// frame-counter increment at V=248, C=1142..1208. The ROM shadow executes the
// complete ClearOamBuffer/module-router caller from this measured envelope.
const OVERWORLD_SPOTLIGHT_CPU_ENTRY_EARLIEST: CpuRasterPosition =
    CpuRasterPosition::new(248, 1_142);
const OVERWORLD_SPOTLIGHT_CPU_ENTRY_LATEST: CpuRasterPosition = CpuRasterPosition::new(248, 1_208);

// The two cold route entries observed through the current frontier reach
// Module0F at V=255, master cycle 480..598. Execute both measured ends and
// require the same semantic plan so input-dependent prefix jitter cannot turn
// into a route-specific publication rule.
const DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST: CpuRasterPosition =
    CpuRasterPosition::new(255, 480);
const DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST: CpuRasterPosition = CpuRasterPosition::new(255, 598);

const SPOTLIGHT_VISIBLE_SCANLINES: usize = 224;

const MODULE0E_DIALOGUE_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    // Entry to Module0E_Interface through the common module router. The
    // translated caller invokes the timing shadow at this same semantic
    // boundary, before Sprite_Main mutates any of its operands. Module_MainRouting
    // tail-calls the selected module through `jmp [$03]`; Module0E's RTL therefore
    // returns directly to ZeldaRunGameLoop at $00:805a, immediately before
    // NMI_PrepareSprites. Stop there to measure the caller return itself. The
    // timing run deliberately continues afterward to observe the following NMI.
    entry_pc: 0x00_f800,
    stop_pc: 0x00_805a,
    a: 0xb700,
    x: 0x00e0,
    y: 0x000e,
    sp: 0x01fc,
    dp: 0,
    db: 0,
    carry: false,
    zero: true,
    overflow: false,
    negative: false,
    interrupt_disable: false,
    decimal: false,
    accumulator_is_8_bit: true,
    index_is_8_bit: true,
    emulation: false,
    waiting: false,
    stack_address: 0x01fd,
    // JSL return address $00:8059 (+1 = $805a) INCLUDING its bank byte: a
    // foreign (Snes9x-seeded) WRAM image holds $80 at $01FF because the ROM
    // runs its bank-$00 code from the FastROM mirror, and an RTL into $80:805a
    // would never reach `stop_pc` (oracle-seeded boundary 5, frame 72136).
    stack_bytes: &[0x59, 0x80, 0x00],
};

/// `Module1B_SpawnSelect` ($02:8586, the save-quit "where to start" prompt)
/// hosts the same RenderText/Text_Initialize call as Module0E_Interface and
/// returns to ZeldaRunGameLoop through the same router tail-call.
/// The polyhedral thread's per-frame render in the Triforce room: from the
/// thread loop's `JSL $09:FD04` (after it observed `intro_did_run_step` set
/// and no pending upload) to the `STA $0C` that publishes the frame. The
/// thread was created by `Polyhedral_InitializeThread` with the RTI frame it
/// copies from `$09:F810`: DB `$09`, DP `$1F00`, P `$30`, and its own stack
/// below `$1F3F`.
const TRIFORCE_ROOM_POLY_RENDER_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    entry_pc: 0x09_f825,
    stop_pc: 0x09_f83b,
    a: 0,
    x: 0,
    y: 0,
    sp: 0x1f3e,
    dp: 0x1f00,
    db: 0x09,
    carry: false,
    zero: true,
    overflow: false,
    negative: false,
    interrupt_disable: false,
    decimal: false,
    accumulator_is_8_bit: true,
    index_is_8_bit: true,
    emulation: false,
    waiting: false,
    stack_address: 0x1f3f,
    stack_bytes: &[],
};

const MODULE1B_DIALOGUE_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    entry_pc: 0x02_8586,
    ..MODULE0E_DIALOGUE_CPU_CHECKPOINT
};

/// Effective raster position from which the crystal poly thread owns the CPU
/// up to the NMI acceptance in a rendering field, as seen by the main thread
/// during the maiden's dialogue initialization (route hosts 414025-414033,
/// 415930-415939). The ROM's V-IRQ trigger itself is `$FF` = 48, but the NMI
/// and IRQ swap the two threads unconditionally, so which thread holds the
/// IRQ-to-NMI slice alternates with every idle yield; the main thread's
/// measured loss per rendering field is close to the vblank-plus-top slice
/// (~97 scanlines), which this constant reproduces.
const POLY_THREAD_EFFECTIVE_IRQ_SCANLINE: u16 = 128;

pub(super) fn dialogue_initialization_cpu_plan(
    state: &ZeldaState,
    entry: (u16, u16),
) -> DialogueInitializationCpuPlan {
    let timing_dma = state.dma_with_native_hdma_enable();
    let checkpoint = if state.game_state.frame.main_module == 27 {
        MODULE1B_DIALOGUE_CPU_CHECKPOINT
    } else {
        MODULE0E_DIALOGUE_CPU_CHECKPOINT
    };
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        checkpoint,
    )
    .expect("dialogue CPU timing requires the loaded Zelda ROM");
    run.restore_original_dialogue_pointer_table()
        .expect("dialogue CPU timing requires the original compressed message table");
    let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
        CpuRasterPosition::new(entry.0, entry.1),
        CpuBusWorkload::with_dynamic_hdma(),
        CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0),
    );
    if state.game_state.display.nmi_thread_active {
        // The crystal maiden's dialogue opens while her poly thread runs
        // (route hosts 414025, 793479); the Module0E frame's Sprite_Main
        // keeps re-arming the thread flag inside the shadow. The shadow's
        // NMI must not switch onto the poly stack (its contents are not
        // modeled); the budget instead hands the effective IRQ-to-NMI slice
        // of every rendering field to the poly thread.
        run.disable_nmi_thread_switch()
            .expect("dialogue CPU timing requires the ROM's NMI thread switch");
        budget = budget.with_poly_thread_irq(
            POLY_THREAD_EFFECTIVE_IRQ_SCANLINE,
            state.dungeon_poly_thread_free_host_offsets(),
        );
    }
    let mut nmi_crossings = 0u8;
    let mut final_interrupted_pc = 0;
    let mut entered_text_initialize = false;
    let mut prefix_nmi_crossings = None;
    let mut completed_return = None;

    for _ in 0..5_000_000 {
        if run.pc() == 0x0e_c483 {
            entered_text_initialize = true;
        }
        if run.pc() == 0x0e_c4e2 && prefix_nmi_crossings.is_none() {
            prefix_nmi_crossings = Some(nmi_crossings);
        }
        if run.is_complete() && completed_return.is_none() {
            assert!(
                entered_text_initialize,
                "Module0E dialogue timing returned without entering Text_Initialize"
            );
            let (return_scanline, return_master_cycle) = budget.raster_position().coordinates();
            let animated_tile_source = u16::from(run.ram_byte(ANIMATED_TILE_DATA_SRC))
                | (u16::from(run.ram_byte(ANIMATED_TILE_DATA_SRC + 1)) << 8);
            completed_return = Some((return_scanline, return_master_cycle, animated_tile_source));
        }
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        if advance_rom_cpu_step(&mut run, &mut budget)
            .reached_boundary()
            .is_some()
        {
            if let Some((return_scanline, return_master_cycle, return_animated_tile_source)) =
                completed_return
            {
                // Bank00's NMI runs NMI_DoUpdates only when $12 is clear. Keep
                // executing through any skipped interrupt until the next NMI
                // that can actually consume animated-tile DMA operands.
                if run.ram_byte(NMI_BOOLEAN) == 0 {
                    let nmi_animated_tile_source = u16::from(run.ram_byte(ANIMATED_TILE_DATA_SRC))
                        | (u16::from(run.ram_byte(ANIMATED_TILE_DATA_SRC + 1)) << 8);
                    return DialogueInitializationCpuPlan {
                        prefix_nmi_crossings: prefix_nmi_crossings.expect(
                            "dialogue CPU timing returned without entering Text_LoadCharacterBuffer",
                        ),
                        nmi_crossings,
                        final_interrupted_pc,
                        return_scanline,
                        return_master_cycle,
                        // ZeldaRunGameLoop calls NMI_PrepareSprites immediately
                        // before clearing $12. If this NMI still sees the source
                        // present at the preceding main-wait boundary, it
                        // interrupted that caller before the animation-source
                        // update. Its active-scanout receipt must therefore use
                        // captured host-boundary operands while the atomic
                        // port's following live PPU write remains independent.
                        following_main_nmi_uses_host_animated_bg_operands: nmi_animated_tile_source
                            == return_animated_tile_source,
                    };
                }
            } else {
                assert!(
                    entered_text_initialize,
                    "Module0E reached NMI before Text_Initialize"
                );
                if crate::debug_env::var_os("ZELDA3_DEBUG_DIALOGUE_CPU_PLAN").is_some() {
                    eprintln!(
                        "[DLG-CPU] nmi #{nmi_crossings} at pc={:06x} module={:02x}/{:02x} $12={:02x} text_state={:02x} read_pos={:02x}{:02x} sp={:04x}",
                        run.pc(),
                        run.ram_byte(0x10),
                        run.ram_byte(0x11),
                        run.ram_byte(NMI_BOOLEAN),
                        run.ram_byte(0x1cd8),
                        run.ram_byte(0x1cda),
                        run.ram_byte(0x1cd9),
                        run.stack_pointer(),
                    );
                }
                nmi_crossings = nmi_crossings.checked_add(1).unwrap_or_else(|| {
                    panic!(
                        "dialogue NMI count overflowed at pc={:06x} module={:02x}/{:02x} $12={:02x} text_state={:02x}",
                        run.pc(),
                        run.ram_byte(0x10),
                        run.ram_byte(0x11),
                        run.ram_byte(NMI_BOOLEAN),
                        run.ram_byte(0x1cd8),
                    )
                });
                final_interrupted_pc = run.pc();
            }
            advance_rom_cpu_through_nmi(&mut run, &mut budget);
        }
    }
    panic!(
        "dialogue ROM timing did not reach the following core-update NMI; stopped at {:06x}",
        run.pc()
    );
}

// State 13 and 14 enter the post-NMI main loop within these measured raster
// intervals. The live prefix varies slightly, so both ends are executed and
// reduced to a semantic continuation that is safe across the whole interval.
const DUNGEON_LANDING_UNFILTERED_CPU_ENTRY_EARLIEST: CpuRasterPosition =
    CpuRasterPosition::new(248, 1_200);
const DUNGEON_LANDING_UNFILTERED_CPU_ENTRY_LATEST: CpuRasterPosition =
    CpuRasterPosition::new(248, 1_294);
const DUNGEON_LANDING_FILTERED_CPU_ENTRY_EARLIEST: CpuRasterPosition =
    CpuRasterPosition::new(251, 1_178);
const DUNGEON_LANDING_FILTERED_CPU_ENTRY_LATEST: CpuRasterPosition =
    CpuRasterPosition::new(252, 376);
// Instrumented Snes9x traces for repeated state-3 entries reach the common
// frame-counter increment at V=248, master cycle 1216..1240. The translated
// display scheduler can retain DMA generations that the hardware has already
// consumed, so replay the real leading NMI for its WRAM side effects, then
// anchor the long fixed-size sprite conversion at this observed caller phase.
const DUNGEON_SPRITE_CONVERSION_CPU_ENTRY_EARLIEST: CpuRasterPosition =
    CpuRasterPosition::new(248, 1_200);
const DUNGEON_SPRITE_CONVERSION_CPU_ENTRY_LATEST: CpuRasterPosition =
    CpuRasterPosition::new(248, 1_260);

const DUNGEON_ROOM_LOAD_CPU_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
    // Module07_02_SupertileTransition, immediately before dispatching state 1.
    // The caller stack is shared with the later state-12 checkpoint.
    entry_pc: 0x02_8a26,
    stop_pc: 0x00_8036,
    a: 0xb704,
    x: 4,
    y: 7,
    sp: 0x01fa,
    dp: 0,
    db: 0,
    carry: false,
    zero: false,
    overflow: false,
    negative: false,
    interrupt_disable: false,
    decimal: false,
    accumulator_is_8_bit: true,
    index_is_8_bit: true,
    emulation: false,
    waiting: false,
    stack_address: 0x01fb,
    // Inner RTS return ($87ae) then the outer JSL return ($8059) with its
    // bank byte (see MODULE0E_DIALOGUE_CPU_CHECKPOINT).
    stack_bytes: &[0xae, 0x87, 0x59, 0x80, 0x00],
};

// Every state-1 entry observed by Snes9x through the recorded route frontier
// falls in this 84-master-cycle interval around the field wrap. Run both
// endpoints and require the semantic continuation schedule to agree, so small
// input/handler variations cannot be mistaken for a room-specific rule.
const DUNGEON_ROOM_LOAD_CPU_ENTRY_EARLIEST: CpuRasterPosition = CpuRasterPosition::new(258, 1320);
const DUNGEON_ROOM_LOAD_CPU_ENTRY_LATEST: CpuRasterPosition = CpuRasterPosition::new(259, 40);

/// Debug accounting of the HDMA master cycles the ROM CPU shadow charged
/// (`ZELDA3_DEBUG_POLY` slot receipts): (init cycles, per-line cycles, lines).
pub(crate) static ROM_CPU_SHADOW_HDMA_DEBUG: [std::sync::atomic::AtomicU64; 3] = [
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
];

fn advance_rom_cpu_step(run: &mut RomCpuTimingRun, budget: &mut CpuCycleBudget) -> CpuWorkAdvance {
    use std::sync::atomic::Ordering::Relaxed;
    let timing = run.step();
    let instruction =
        budget.advance_instruction_with_hdma(timing.master_cycles, |event, scanline| match event {
            CpuBusEvent::HdmaInit => {
                run.set_raster_position(scanline, 20);
                let cycles = run.run_hdma_init_master_cycles();
                ROM_CPU_SHADOW_HDMA_DEBUG[0].fetch_add(u64::from(cycles), Relaxed);
                cycles
            }
            CpuBusEvent::HdmaStart => {
                run.set_raster_position(scanline, 1_106);
                let cycles = run.run_hdma_scanline_master_cycles();
                ROM_CPU_SHADOW_HDMA_DEBUG[1].fetch_add(u64::from(cycles), Relaxed);
                ROM_CPU_SHADOW_HDMA_DEBUG[2].fetch_add(1, Relaxed);
                cycles
            }
            CpuBusEvent::WramRefresh => unreachable!(),
        });
    let dma_master_cycles = run.drain_started_dma_master_cycles();
    budget.advance_started_general_dma(instruction, dma_master_cycles)
}

fn advance_rom_cpu_through_nmi(run: &mut RomCpuTimingRun, budget: &mut CpuCycleBudget) {
    let return_pc = run.pc();
    let return_sp = run.stack_pointer();
    budget.begin_nmi_handler();
    run.request_nmi();

    let trace = crate::debug_env::var_os("ZELDA3_DEBUG_ROM_CPU_NMI_TRACE").is_some();
    for step in 0..100_000 {
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        if trace {
            eprintln!(
                "[ROMCPU-NMI] step={step} pc={:06x} sp={:04x} v={scanline} h={master_cycle}",
                run.pc(),
                run.stack_pointer(),
            );
        }
        assert_eq!(
            advance_rom_cpu_step(run, budget),
            CpuWorkAdvance::Complete,
            "ROM NMI handler crossed the next vblank at {:06x}",
            run.pc(),
        );
        if run.pc() == return_pc && run.stack_pointer() == return_sp {
            return;
        }
    }
    panic!("ROM NMI handler did not return to {return_pc:06x}");
}

fn dungeon_exit_spotlight_cpu_plan_at(
    state: &ZeldaState,
    entry: CpuRasterPosition,
) -> Option<DungeonExitSpotlightCpuPlan> {
    // The C port relocates its 240-word working table to $1dba0. The original
    // routine at $00:f37d authors $7f:7000 and the copy loop at $00:f3b7
    // publishes it to the hardware table at $7e:1b00. Seed both original-ROM
    // buffers from the translated live owner before running the isolated CPU.
    let table_bytes = SPOTLIGHT_VISIBLE_SCANLINES * 2;
    let mut rom_ram = state.ram.clone();
    // The translated caller has just executed this increment. Rewind only the
    // cloned timing image so the ROM owns the full ZeldaRunGameLoop prefix;
    // ClearOamBuffer is idempotent and is intentionally executed by the ROM.
    rom_ram[FRAME_COUNTER] = rom_ram[FRAME_COUNTER].wrapping_sub(1);
    let live_table = state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + table_bytes].to_vec();
    rom_ram[0x1b00..0x1b00 + table_bytes].copy_from_slice(&live_table);
    rom_ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + table_bytes].copy_from_slice(&live_table);

    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &rom_ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        DUNGEON_EXIT_SPOTLIGHT_CPU_CHECKPOINT,
    )
    .expect("dungeon-exit spotlight CPU timing requires the loaded Zelda ROM");
    let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
        entry,
        CpuBusWorkload::with_dynamic_hdma(),
        CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0),
    );
    let mut iterations = 0usize;
    let mut iterations_before_nmi = None;
    let mut interrupted_pc = None;
    let mut interrupted_return_address = None;
    let mut returned_to_main_wait_before_first_nmi = None;
    let mut main_loop_sprite_preparation_completed = false;
    let mut main_loop_sprite_preparation_completed_before_second_nmi = None;
    let mut active_window_words = [None; SPOTLIGHT_VISIBLE_SCANLINES];
    let mut completed_active_window_words = None;
    let mut following_window_words = [None; SPOTLIGHT_VISIBLE_SCANLINES];
    let mut completed_following_window_words = None;
    let mut next_entry_after_second_nmi = None;
    let mut module_ended_after_second_nmi = false;

    for _ in 0..5_000_000 {
        // ZeldaRunGameLoop calls NMI_PrepareSprites at $00:805a. Reaching
        // $00:805d proves the original subroutine returned; its complete DMA
        // operand/countdown side effects now belong to this C iteration.
        if run.pc() == 0x00_805d {
            main_loop_sprite_preparation_completed = true;
        }
        if completed_active_window_words.is_some() {
            if next_entry_after_second_nmi.is_none()
                && run.pc() == DUNGEON_EXIT_SPOTLIGHT_CPU_CHECKPOINT.entry_pc
            {
                next_entry_after_second_nmi = Some(budget.raster_position());
            }
            if run.pc() == 0x00_8036 && run.ram_byte(MAIN_MODULE) != 0x0f {
                module_ended_after_second_nmi = true;
            }
        }
        if let (Some(active_window_words), Some(following_window_words)) = (
            completed_active_window_words,
            completed_following_window_words,
        ) {
            if next_entry_after_second_nmi.is_some() || module_ended_after_second_nmi {
                let iterations_before_nmi =
                    iterations_before_nmi.expect("completed plan must retain its loop count");
                let interrupted_pc =
                    interrupted_pc.expect("interrupted plan must retain its ROM PC");
                return Some(DungeonExitSpotlightCpuPlan {
                    interrupted_pc,
                    interrupted_return_address: interrupted_return_address
                        .expect("interrupted plan must retain its ROM return address"),
                    iterations_before_nmi,
                    // Player_MovePosition1_ has completed both coordinate
                    // integrations at these instruction boundaries. The
                    // earliest/latest measured envelope straddles only the
                    // idempotent final high-byte store for this live state;
                    // moving-floor, conveyor, drag, animation, and Link OAM
                    // remain on the suspended Module0F call stack.
                    link_position_integrated_before_first_nmi: matches!(
                        interrupted_pc,
                        // $e3cd..$e3d0 is the final coordinate-loop store/
                        // epilogue; $e595 is Link_HandleMovingFloor's entry,
                        // before any instruction in the remaining tail ran.
                        0x07_e3cd | 0x07_e3cf | 0x07_e3d0 | 0x07_e595
                    ),
                    returned_to_main_wait_before_first_nmi: returned_to_main_wait_before_first_nmi
                        .expect("completed plan must classify its main-loop return"),
                    main_loop_sprite_preparation_completed_before_second_nmi:
                        main_loop_sprite_preparation_completed_before_second_nmi
                            .expect("completed plan must classify its main-loop suffix"),
                    active_window_words,
                    following_window_words,
                    next_entry_earliest: next_entry_after_second_nmi,
                    next_entry_latest: next_entry_after_second_nmi,
                });
            }
        }

        if run.is_complete() && iterations_before_nmi.is_none() {
            return None;
        }

        if completed_active_window_words.is_none() && run.pc() == 0x00_f39b {
            // $f39b compares the just-authored upper cursor with the vertical
            // center, so each visit is one complete C loop iteration.
            iterations += 1;
        }

        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        let timing = run.step();
        let active_scanout =
            iterations_before_nmi.is_some() && completed_active_window_words.is_none();
        let following_scanout =
            completed_active_window_words.is_some() && completed_following_window_words.is_none();
        let instruction = budget.advance_instruction_with_hdma(
            timing.master_cycles,
            |event, scanline| match event {
                CpuBusEvent::HdmaInit => {
                    run.set_raster_position(scanline, 20);
                    run.run_hdma_init_master_cycles()
                }
                CpuBusEvent::HdmaStart => {
                    run.set_raster_position(scanline, 1_106);
                    let cycles = run.run_hdma_scanline_master_cycles();
                    let scanline = usize::from(scanline);
                    if active_scanout && scanline < SPOTLIGHT_VISIBLE_SCANLINES {
                        let (left, right) = run.window1_bounds();
                        active_window_words[scanline] =
                            Some(u16::from(left) | (u16::from(right) << 8));
                    } else if following_scanout && scanline < SPOTLIGHT_VISIBLE_SCANLINES {
                        let (left, right) = run.window1_bounds();
                        following_window_words[scanline] =
                            Some(u16::from(left) | (u16::from(right) << 8));
                    }
                    cycles
                }
                CpuBusEvent::WramRefresh => unreachable!(),
            },
        );
        let advance =
            budget.advance_started_general_dma(instruction, run.drain_started_dma_master_cycles());
        if advance.reached_boundary().is_some() {
            if iterations_before_nmi.is_none() {
                iterations_before_nmi = Some(iterations);
                interrupted_pc = Some(run.pc());
                interrupted_return_address = Some(run.stack_return_address());
                returned_to_main_wait_before_first_nmi =
                    Some(matches!(run.pc(), 0x00_8034 | 0x00_8036));
            } else if completed_active_window_words.is_none() {
                main_loop_sprite_preparation_completed_before_second_nmi =
                    Some(main_loop_sprite_preparation_completed);
                completed_active_window_words =
                    Some(complete_spotlight_window_words(&run, &active_window_words));
            } else if completed_following_window_words.is_none() {
                completed_following_window_words = Some(complete_spotlight_window_words(
                    &run,
                    &following_window_words,
                ));
            }
            advance_rom_cpu_through_nmi(&mut run, &mut budget);
        }
    }
    panic!(
        "dungeon-exit spotlight ROM timing did not return from Module0F; stopped at {:06x}",
        run.pc(),
    );
}

fn complete_spotlight_window_words(
    run: &RomCpuTimingRun,
    captured: &[Option<u16>; SPOTLIGHT_VISIBLE_SCANLINES],
) -> [u16; SPOTLIGHT_VISIBLE_SCANLINES] {
    std::array::from_fn(|scanline| {
        captured[scanline].unwrap_or_else(|| {
            let address = 0x1b00 + scanline * 2;
            u16::from(run.ram_byte(address)) | (u16::from(run.ram_byte(address + 1)) << 8)
        })
    })
}

fn dungeon_exit_spotlight_cpu_plan(
    state: &ZeldaState,
    entry_earliest: CpuRasterPosition,
    entry_latest: CpuRasterPosition,
) -> Option<DungeonExitSpotlightCpuPlan> {
    let mut earliest = dungeon_exit_spotlight_cpu_plan_at(state, entry_earliest)
        .map(DungeonExitSpotlightCpuPlan::normalized_interruption_phase);
    let latest = dungeon_exit_spotlight_cpu_plan_at(state, entry_latest)
        .map(DungeonExitSpotlightCpuPlan::normalized_interruption_phase);
    assert_eq!(
        earliest.map(DungeonExitSpotlightCpuPlan::current_boundary_key),
        latest.map(DungeonExitSpotlightCpuPlan::current_boundary_key),
        "measured Module0F entry raster envelope changed the spotlight CPU/HDMA plan",
    );
    if let (Some(earliest), Some(latest)) = (&mut earliest, latest) {
        earliest.next_entry_latest = latest.next_entry_latest;
    }
    earliest
}

fn overworld_spotlight_cpu_plan_at(
    state: &ZeldaState,
    entry: CpuRasterPosition,
) -> Option<OverworldSpotlightCpuPlan> {
    let table_bytes = SPOTLIGHT_VISIBLE_SCANLINES * 2;
    let mut rom_ram = state.ram.clone();
    // The translated frame entry has already performed ZeldaRunGameLoop's
    // increment. This isolated run begins at that instruction so it also owns
    // ClearOamBuffer and the full C caller; rewind only the cloned timing RAM.
    rom_ram[FRAME_COUNTER] = rom_ram[FRAME_COUNTER].wrapping_sub(1);
    let live_table = state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + table_bytes].to_vec();
    rom_ram[0x1b00..0x1b00 + table_bytes].copy_from_slice(&live_table);
    rom_ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + table_bytes].copy_from_slice(&live_table);

    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &rom_ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        OVERWORLD_SPOTLIGHT_CPU_CHECKPOINT,
    )
    .expect("overworld spotlight CPU timing requires the loaded Zelda ROM");
    let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
        entry,
        CpuBusWorkload::with_dynamic_hdma(),
        CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0),
    );
    let mut iterations = 0usize;
    let mut interrupted_pc = None;
    let mut interrupted_return_address = None;
    let mut iterations_before_nmi = None;
    let mut nmis_elapsed_after_interruption = 0u8;
    let mut nmis_before_module_exit = None;
    let mut active_window_words = [None; SPOTLIGHT_VISIBLE_SCANLINES];
    let mut completed_active_window_words = None;
    let mut following_window_words = [None; SPOTLIGHT_VISIBLE_SCANLINES];
    let mut completed_following_window_words = None;
    let mut next_entry = None;
    let mut module_ended = false;

    for _ in 0..5_000_000 {
        if run.is_complete() && iterations_before_nmi.is_none() {
            return None;
        }
        if iterations_before_nmi.is_some()
            && next_entry.is_none()
            && run.pc() == OVERWORLD_SPOTLIGHT_CPU_CHECKPOINT.entry_pc
        {
            next_entry = Some(budget.raster_position());
        }
        if iterations_before_nmi.is_some()
            && run.pc() == 0x00_8036
            && run.ram_byte(MAIN_MODULE) != 0x10
        {
            module_ended = true;
            nmis_before_module_exit.get_or_insert(nmis_elapsed_after_interruption);
        }
        if let (Some(active_window_words), Some(following_window_words)) = (
            completed_active_window_words,
            completed_following_window_words,
        ) {
            if next_entry.is_some() || module_ended {
                return Some(OverworldSpotlightCpuPlan {
                    interrupted_pc: interrupted_pc
                        .expect("interrupted opening plan must retain its ROM PC"),
                    interrupted_return_address: interrupted_return_address
                        .expect("interrupted opening plan must retain its ROM return address"),
                    iterations_before_nmi: iterations_before_nmi
                        .expect("completed opening plan must retain its loop count"),
                    nmis_before_module_exit,
                    active_window_words,
                    following_window_words,
                    next_entry_earliest: next_entry,
                    next_entry_latest: next_entry,
                });
            }
        }
        if run.pc() == 0x00_f39b && iterations_before_nmi.is_none() {
            iterations += 1;
        }
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        let timing = run.step();
        let active_scanout =
            iterations_before_nmi.is_some() && completed_active_window_words.is_none();
        let following_scanout =
            completed_active_window_words.is_some() && completed_following_window_words.is_none();
        let instruction = budget.advance_instruction_with_hdma(
            timing.master_cycles,
            |event, scanline| match event {
                CpuBusEvent::HdmaInit => {
                    run.set_raster_position(scanline, 20);
                    run.run_hdma_init_master_cycles()
                }
                CpuBusEvent::HdmaStart => {
                    run.set_raster_position(scanline, 1_106);
                    let cycles = run.run_hdma_scanline_master_cycles();
                    let scanline = usize::from(scanline);
                    if active_scanout && scanline < SPOTLIGHT_VISIBLE_SCANLINES {
                        let (left, right) = run.window1_bounds();
                        active_window_words[scanline] =
                            Some(u16::from(left) | (u16::from(right) << 8));
                    } else if following_scanout && scanline < SPOTLIGHT_VISIBLE_SCANLINES {
                        let (left, right) = run.window1_bounds();
                        following_window_words[scanline] =
                            Some(u16::from(left) | (u16::from(right) << 8));
                    }
                    cycles
                }
                CpuBusEvent::WramRefresh => unreachable!(),
            },
        );
        let advance =
            budget.advance_started_general_dma(instruction, run.drain_started_dma_master_cycles());
        if advance.reached_boundary().is_some() {
            if iterations_before_nmi.is_none() {
                iterations_before_nmi = Some(iterations);
                interrupted_pc = Some(run.pc());
                interrupted_return_address = Some(run.stack_return_address());
            } else if completed_active_window_words.is_none() {
                completed_active_window_words =
                    Some(complete_spotlight_window_words(&run, &active_window_words));
            } else if completed_following_window_words.is_none() {
                completed_following_window_words = Some(complete_spotlight_window_words(
                    &run,
                    &following_window_words,
                ));
            }
            nmis_elapsed_after_interruption = nmis_elapsed_after_interruption
                .checked_add(1)
                .expect("overworld spotlight NMI count overflowed");
            advance_rom_cpu_through_nmi(&mut run, &mut budget);
        }
    }
    panic!(
        "overworld spotlight ROM timing did not reach NMI or return; stopped at {:06x}",
        run.pc(),
    );
}

fn overworld_spotlight_cpu_plan(
    state: &ZeldaState,
    entry_earliest: CpuRasterPosition,
    entry_latest: CpuRasterPosition,
) -> Option<OverworldSpotlightCpuPlan> {
    let mut earliest = overworld_spotlight_cpu_plan_at(state, entry_earliest)
        .map(OverworldSpotlightCpuPlan::normalized_interruption_phase);
    let latest = overworld_spotlight_cpu_plan_at(state, entry_latest)
        .map(OverworldSpotlightCpuPlan::normalized_interruption_phase);
    assert_eq!(
        earliest.map(OverworldSpotlightCpuPlan::current_boundary_key),
        latest.map(OverworldSpotlightCpuPlan::current_boundary_key),
        "measured Module10 entry raster envelope changed the spotlight CPU phase",
    );
    if let (Some(earliest), Some(latest)) = (&mut earliest, latest) {
        earliest.next_entry_latest = latest.next_entry_latest;
    }
    earliest
}

/// Emit the state-12 ROM timing shadow's post-leading-NMI main checkpoint.
///
/// This hook is deliberately limited to the `dispatcher_entry == None` path:
/// that path begins at the current host's main wait and therefore owns an
/// unambiguous absolute comparison-host coordinate. Envelope predictions for a
/// future CPU slice must not be mislabeled as checkpoints from the capture
/// host. The comparison harness owns the window-relative `retro_run`; the game
/// cannot derive it after checkpoint resume, so this trace must not claim to.
fn trace_dungeon_cpu_checkpoint(
    state: &ZeldaState,
    run: &RomCpuTimingRun,
    budget: &CpuCycleBudget,
) {
    let Some(path) = crate::debug_env::var_os("ZELDA3_CPU_CHECKPOINT_TRACE") else {
        return;
    };
    let host_frame = state.frame_ctr_dbg;
    let (scanline, master_cycle) = budget.raster_position().coordinates();
    let room =
        u16::from(run.ram_byte(DUNGEON_ROOM)) | (u16::from(run.ram_byte(DUNGEON_ROOM + 1)) << 8);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .unwrap_or_else(|error| {
            panic!(
                "failed to open CPU checkpoint trace {}: {error}",
                Path::new(&path).display()
            )
        });
    writeln!(
        file,
        concat!(
            "{{\"schema\":2,\"event\":\"rust-cpu-checkpoint\",",
            "\"coordinate\":\"absolute comparison host frame\",",
            "\"host_frame\":{},\"pc\":{},\"v\":{},\"cycles\":{},",
            "\"main\":{},\"sub\":{},\"subsub\":{},\"frame_counter\":{},",
            "\"room\":{},\"lights_out\":{},\"palette_countdown\":{},",
            "\"palette_direction\":{},",
            "\"link_y\":{},\"link_x\":{},\"bg2_v\":{},\"bg2_h\":{}}}"
        ),
        host_frame,
        run.pc(),
        scanline,
        master_cycle,
        run.ram_byte(MAIN_MODULE),
        run.ram_byte(SUBMODULE),
        run.ram_byte(SUBSUBMODULE),
        run.ram_byte(FRAME_COUNTER),
        room,
        run.ram_byte(DUNG_WANT_LIGHTS_OUT) | run.ram_byte(DUNG_WANT_LIGHTS_OUT_COPY),
        run.ram_byte(PALETTE_FILTER_COUNTDOWN),
        run.ram_byte(DARKENING_OR_LIGHTENING_SCREEN),
        u16::from(run.ram_byte(LINK_Y_COORD)) | (u16::from(run.ram_byte(LINK_Y_COORD + 1)) << 8),
        u16::from(run.ram_byte(LINK_X_COORD)) | (u16::from(run.ram_byte(LINK_X_COORD + 1)) << 8),
        u16::from(run.ram_byte(BG2_Y_SCROLL)) | (u16::from(run.ram_byte(BG2_Y_SCROLL + 1)) << 8),
        u16::from(run.ram_byte(BG2_X_SCROLL)) | (u16::from(run.ram_byte(BG2_X_SCROLL + 1)) << 8),
    )
    .unwrap_or_else(|error| {
        panic!(
            "failed to append CPU checkpoint trace {}: {error}",
            Path::new(&path).display()
        )
    });
}

fn dungeon_room_load_cpu_plan(
    state: &ZeldaState,
    entry: CpuRasterPosition,
) -> DungeonRoomLoadCpuPlan {
    const ROOM_LOAD_RETURN_PC: u32 = 0x02_8a5f;
    const AUXILIARY_GRAPHICS_ENTRY_PC: u32 = 0x02_8a63;
    const AUXILIARY_GRAPHICS_RETURN_PC: u32 = 0x02_8a67;
    const SPRITE_MAIN_ENTRY_PC: u32 = 0x06_8328;
    // Return address of Module 7's JSL Sprite_Main. When the first caller NMI
    // lands inside a sprite slot, every slice through this checkpoint belongs
    // to that same translated Sprite_Main continuation; it is not a generic
    // caller delay. Snes9x room-$12 resumes at $00:e7f8/$00:e7f2/$00:d5d8
    // inside slot initialization before reaching this address at the fourth
    // boundary.
    const SPRITE_MAIN_RETURN_PC: u32 = 0x02_8842;
    const SPRITE_EXECUTE_SINGLE_ENTRY_PC: u32 = 0x06_84e2;
    // The loop reaches this instruction after each Sprite_ExecuteSingle
    // returns. It is a per-slot semantic commit point, not the end of the
    // whole 16-slot walk.
    const SPRITE_SLOT_RETURN_PC: u32 = 0x06_83a7;
    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        DUNGEON_ROOM_LOAD_CPU_CHECKPOINT,
    )
    .expect("dungeon room-load CPU timing requires the loaded Zelda ROM");
    let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
        entry,
        CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL_MASTER_CYCLES),
        CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0),
    );
    let mut nmis = 0u8;
    let mut room_load_return = None;
    let mut auxiliary_graphics_entry = None;
    let mut auxiliary_graphics_return = None;
    let mut room_load_return_nmis = 0;
    let mut auxiliary_graphics_entry_nmis = 0;
    let mut auxiliary_graphics_return_nmis = 0;
    let mut sprite_main_first_nmi_observed = false;
    let mut sprite_main_current_slot = None;
    let mut sprite_main_last_completed_slot = None;
    let mut initialized_sprite_slot = None;
    let mut sprite_main_boundary = None;
    let mut sprite_main_entry_nmis: Option<u8> = None;
    let mut sprite_main_return_nmis: Option<u8> = None;

    run.enable_cpu_write_trace();

    for _ in 0..5_000_000 {
        let pc = run.pc();
        let raster = budget.raster_position();
        if pc == SPRITE_EXECUTE_SINGLE_ENTRY_PC && auxiliary_graphics_return.is_some() {
            let slot = run.ram_byte(CUR_OBJECT_INDEX);
            assert!(slot < 16, "Sprite_Main entered an invalid slot {slot}");
            sprite_main_current_slot = Some(slot);
        }
        if pc == SPRITE_SLOT_RETURN_PC && auxiliary_graphics_return.is_some() {
            sprite_main_last_completed_slot = sprite_main_current_slot;
        }
        match pc {
            ROOM_LOAD_RETURN_PC if room_load_return.is_none() => {
                room_load_return = Some(raster);
                room_load_return_nmis = nmis;
            }
            AUXILIARY_GRAPHICS_ENTRY_PC if auxiliary_graphics_entry.is_none() => {
                auxiliary_graphics_entry = Some(raster);
                auxiliary_graphics_entry_nmis = nmis;
            }
            AUXILIARY_GRAPHICS_RETURN_PC if auxiliary_graphics_return.is_none() => {
                auxiliary_graphics_return = Some(raster);
                auxiliary_graphics_return_nmis = nmis;
            }
            SPRITE_MAIN_ENTRY_PC
                if auxiliary_graphics_return.is_some() && sprite_main_entry_nmis.is_none() =>
            {
                sprite_main_entry_nmis = Some(nmis);
            }
            SPRITE_MAIN_RETURN_PC
                if auxiliary_graphics_return.is_some() && sprite_main_return_nmis.is_none() =>
            {
                sprite_main_return_nmis = Some(nmis);
            }
            _ => {}
        }
        if run.is_complete() {
            let room_load_return =
                room_load_return.expect("room-load timing missed its return checkpoint");
            let auxiliary_graphics_entry = auxiliary_graphics_entry
                .expect("room-load timing missed the auxiliary-graphics entry checkpoint");
            let auxiliary_graphics_return = auxiliary_graphics_return
                .expect("room-load timing missed the auxiliary-graphics return checkpoint");
            let sprite_main_return_nmis = sprite_main_return_nmis
                .expect("room-load timing missed the Sprite_Main return checkpoint");
            let sprite_main_entry_nmis = sprite_main_entry_nmis
                .expect("room-load timing missed the Sprite_Main entry checkpoint");
            if sprite_main_return_nmis == sprite_main_entry_nmis {
                // The first caller NMI belongs to the Module 7 suffix after
                // Sprite_Main has already returned. Any last live-slot entry
                // observed earlier is stale at that boundary and must not be
                // turned into a Sprite_Main continuation.
                sprite_main_boundary = None;
            }
            let caller_sprite_main_nmis =
                sprite_main_return_nmis.saturating_sub(sprite_main_entry_nmis);
            let caller_suffix_nmis = nmis.saturating_sub(sprite_main_return_nmis);
            return DungeonRoomLoadCpuPlan {
                room_load_return,
                auxiliary_graphics_entry,
                auxiliary_graphics_return,
                caller_return: raster,
                room_load_nmis: room_load_return_nmis,
                auxiliary_graphics_nmis: auxiliary_graphics_return_nmis
                    .saturating_sub(auxiliary_graphics_entry_nmis),
                caller_nmis: nmis.saturating_sub(auxiliary_graphics_return_nmis),
                caller_prefix_nmis: sprite_main_entry_nmis
                    .saturating_sub(auxiliary_graphics_return_nmis),
                caller_sprite_main_nmis,
                caller_suffix_nmis,
                sprite_main_boundary,
            };
        }

        let (scanline, master_cycle) = raster.coordinates();
        run.set_raster_position(scanline, master_cycle);
        let advance = advance_rom_cpu_step(&mut run, &mut budget);
        for (address, value) in run.take_cpu_wram_writes() {
            if auxiliary_graphics_return.is_some()
                && (SPRITE_STATE..SPRITE_STATE + 16).contains(&address)
                && value == 9
            {
                initialized_sprite_slot = Some((address - SPRITE_STATE) as u8);
            }
        }
        if advance.reached_boundary().is_some() {
            if sprite_main_entry_nmis.is_some() && !sprite_main_first_nmi_observed {
                sprite_main_first_nmi_observed = true;
                // The ROM writes $0fa0 immediately before every
                // Sprite_ExecuteSingle call, then reaches $06:83a7 after that
                // call returns. NMI may interrupt anywhere inside the current
                // slot, so the safe translated continuation boundary is the
                // preceding slot which completed, not the slot merely entered.
                // This remains valid when state was already 9 and no 8-to-9
                // write exists to reveal the frontier.
                sprite_main_boundary = sprite_main_cpu_interruption_boundary(
                    sprite_main_current_slot,
                    sprite_main_last_completed_slot,
                    initialized_sprite_slot,
                    run.stack_return_address(),
                );
            }
            nmis = nmis.checked_add(1).unwrap_or_else(|| {
                panic!(
                    "room-load NMI count overflowed at pc={:06x} module={:02x}/{:02x} $12={:02x}",
                    run.pc(),
                    run.ram_byte(0x10),
                    run.ram_byte(0x11),
                    run.ram_byte(NMI_BOOLEAN),
                )
            });
            advance_rom_cpu_through_nmi(&mut run, &mut budget);
        }
    }
    panic!(
        "room-load ROM timing did not reach stop PC from {:06x}; stopped at {:06x}",
        DUNGEON_ROOM_LOAD_CPU_CHECKPOINT.entry_pc,
        run.pc(),
    );
}

fn dungeon_room_load_cpu_schedule(state: &ZeldaState) -> DungeonRoomLoadCpuSchedule {
    let earliest =
        dungeon_room_load_cpu_plan(state, DUNGEON_ROOM_LOAD_CPU_ENTRY_EARLIEST).schedule();
    let latest = dungeon_room_load_cpu_plan(state, DUNGEON_ROOM_LOAD_CPU_ENTRY_LATEST).schedule();
    if earliest != latest
        && matches!(state.original_timing_owner, OriginalTimingOwnerState::Live)
        && (DungeonRoomLoadCpuSchedule {
            sprite_main_boundary: None,
            ..earliest
        }) == (DungeonRoomLoadCpuSchedule {
            sprite_main_boundary: None,
            ..latest
        })
    {
        // The entry envelope straddles a raster boundary only inside the
        // caller's Sprite_Main slot loop (route host 593817: AfterSlot(0) vs
        // AfterSlot(1)). The live wire's SpriteMainProgressed / interruption
        // receipts refine that armed boundary when the checkpoint host
        // arrives, so either candidate's shared budget is the same plan.
        return earliest;
    }
    assert_eq!(
        earliest,
        latest,
        "room-load CPU continuation changed across the observed Module 7 entry-phase interval: module={:02x}/{:02x}/{:02x} earliest={earliest:?} latest={latest:?}",
        state.game_state.frame.main_module,
        state.game_state.frame.submodule,
        state.game_state.frame.subsubmodule,
    );
    earliest
}

/// Follow a real long-running Module 7 submodule from the main-loop wait
/// through its return. A translated submodule can remain atomic while its
/// suspension boundary comes from the ROM's semantic subsubmodule write rather
/// than a room or staircase identity. Later NMIs are divided at Sprite_Main's
/// stable slot/return and cached-sprite checkpoints so caller execution is
/// resumed, not delayed as an opaque lump.
/// Ring buffer of the most recent installed host receipt vectors, so a
/// fail-closed panic anywhere in the timing machinery can print the wire
/// window without a dedicated `ZELDA3_DEBUG_INSTALL_RECEIPTS` re-run. Written
/// once per host at install; read only by the compare binary's panic hook.
static RECENT_HOST_RECEIPT_VECTORS: std::sync::Mutex<Option<std::collections::VecDeque<String>>> =
    std::sync::Mutex::new(None);

fn record_recent_host_receipt_vector(line: String) {
    if let Ok(mut ring) = RECENT_HOST_RECEIPT_VECTORS.lock() {
        let ring = ring.get_or_insert_with(std::collections::VecDeque::new);
        if ring.len() >= 12 {
            ring.pop_front();
        }
        ring.push_back(line);
    }
}

/// The most recent installed host receipt vectors, oldest first. Used by the
/// compare binary's panic hook to make timing failures self-describing.
pub fn recent_host_receipt_vectors() -> Vec<String> {
    RECENT_HOST_RECEIPT_VECTORS
        .lock()
        .ok()
        .and_then(|ring| ring.as_ref().map(|ring| ring.iter().cloned().collect()))
        .unwrap_or_default()
}

/// `ZELDA3_DEBUG_HOST_PATH=1`: name the host-body early return a host took,
/// so an "unowned semantic control receipts" close panic can be traced to
/// the lane that skipped consumption (route hosts 202669, 256364).
fn debug_host_path_early_return(host: u32, line: u32) {
    if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
        eprintln!("[HOSTPATH] host={host} early-return line {line}");
    }
}

fn dungeon_submodule_cpu_schedule(state: &ZeldaState) -> DungeonSubmoduleCpuSchedule {
    if state.game_state.frame.main_module == 7
        && state.game_state.frame.submodule == 2
        && state.game_state.frame.subsubmodule == 3
    {
        let earliest = dungeon_submodule_cpu_schedule_plan(
            state,
            Some(DUNGEON_SPRITE_CONVERSION_CPU_ENTRY_EARLIEST),
        );
        let latest = dungeon_submodule_cpu_schedule_plan(
            state,
            Some(DUNGEON_SPRITE_CONVERSION_CPU_ENTRY_LATEST),
        );
        if earliest != latest {
            // The entry envelope straddles a raster boundary for this room.
            // The live wire's cached-sprite checkpoint names the exact copy
            // statement the vblank landed on; exactly one candidate may match
            // it (route host 281197, copied_fields 8 vs 7).
            let wire_interruption =
                matches!(state.original_timing_owner, OriginalTimingOwnerState::Live)
                    .then(|| state.original_timing_semantic_receipts.as_ref())
                    .flatten()
                    .and_then(|receipts| {
                        receipts
                            .semantic()
                            .iter()
                            .find_map(|receipt| match receipt {
                                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                                    receipt,
                                ) => Some(receipt.progress.into()),
                                _ => None,
                            })
                    });
            if let Some(wire_interruption) = wire_interruption {
                let earliest_matches =
                    earliest.cached_sprite_interruption == Some(wire_interruption);
                let latest_matches = latest.cached_sprite_interruption == Some(wire_interruption);
                if earliest_matches != latest_matches {
                    return if earliest_matches { earliest } else { latest };
                }
            }
            // The checkpoint's acceptance arrives on a later host; when the
            // candidates differ only in that copy cursor, the live receipt
            // refines the armed continuation there, so either candidate's
            // shared budget is the same plan.
            let earliest_without_checkpoint = DungeonSubmoduleCpuSchedule {
                cached_sprite_interruption: None,
                ..earliest
            };
            let latest_without_checkpoint = DungeonSubmoduleCpuSchedule {
                cached_sprite_interruption: None,
                ..latest
            };
            if matches!(state.original_timing_owner, OriginalTimingOwnerState::Live)
                && earliest_without_checkpoint == latest_without_checkpoint
            {
                return earliest;
            }
            // The wire's slot boundary supersedes the estimated one (route
            // host 1490403); when the candidates differ only in which slot
            // the caller's Sprite_Main crosses, the plan is otherwise the
            // same and the live receipt refines the boundary (route host
            // 1330354: AfterSlot(2) against AfterSlot(3)).
            let earliest_without_boundary = DungeonSubmoduleCpuSchedule {
                sprite_main_boundary: None,
                ..earliest_without_checkpoint
            };
            let latest_without_boundary = DungeonSubmoduleCpuSchedule {
                sprite_main_boundary: None,
                ..latest_without_checkpoint
            };
            if matches!(state.original_timing_owner, OriginalTimingOwnerState::Live)
                && earliest_without_boundary == latest_without_boundary
            {
                return earliest;
            }
        }
        assert_eq!(
            earliest, latest,
            "sprite-conversion continuation changed across its observed dispatcher-entry interval",
        );
        earliest
    } else {
        dungeon_submodule_cpu_schedule_plan(state, None)
    }
}

fn dungeon_submodule_cpu_schedule_plan(
    state: &ZeldaState,
    dispatcher_entry: Option<CpuRasterPosition>,
) -> DungeonSubmoduleCpuSchedule {
    const SPRITE_MAIN_RETURN_PC: u32 = 0x02_8842;
    const SPRITE_EXECUTE_SINGLE_ENTRY_PC: u32 = 0x06_84e2;
    const SPRITE_SLOT_RETURN_PC: u32 = 0x06_83a7;

    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        DUNGEON_MAIN_WAIT_CPU_CHECKPOINT,
    )
    .expect("dungeon submodule CPU timing requires the loaded Zelda ROM");
    let field_timing = CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0);
    let mut budget =
        CpuCycleBudget::at_nmi_acceptance(CpuBusWorkload::with_dynamic_hdma(), field_timing);
    advance_rom_cpu_through_nmi(&mut run, &mut budget);
    while run.pc() != DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.entry_pc {
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        assert_eq!(
            advance_rom_cpu_step(&mut run, &mut budget),
            CpuWorkAdvance::Complete,
            "spiral-room main wait-loop exit crossed the next vblank at {:06x}",
            run.pc(),
        );
    }
    if let Some(entry) = dispatcher_entry {
        budget = CpuCycleBudget::until_next_nmi_acceptance(
            entry,
            CpuBusWorkload::with_dynamic_hdma(),
            field_timing,
        );
    }
    if crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        eprintln!(
            "dungeon_submodule_dispatch_entry host={} raster={scanline}:{master_cycle} nmi_latch={:02x}",
            state.frame_ctr_dbg,
            run.ram_byte(0x12),
        );
    }

    let entry_subsubmodule = run.ram_byte(SUBSUBMODULE);
    let mut nmis = 0u8;
    let mut submodule_nmis = None;
    let mut caller_first_nmi_observed = false;
    let mut sprite_main_current_slot = None;
    let mut sprite_main_last_completed_slot = None;
    let mut initialized_sprite_slot = None;
    let mut sprite_main_boundary = None;
    let mut sprite_main_return_nmis: Option<u8> = None;
    let mut link_oam_started = false;
    let mut nmi_prepare_sprites_started = false;

    let mut caller_first_nmi_phase = None;
    let mut cached_sprite_copy: Option<CachedSpriteCpuProgress> = None;
    let mut cached_sprite_interruption = None;

    run.enable_cpu_write_trace();

    for _ in 0..5_000_000 {
        if run.is_complete() {
            let submodule_nmis = submodule_nmis
                .expect("dungeon ROM timing missed the subsubmodule completion write");
            let sprite_main_return_nmis = sprite_main_return_nmis
                .expect("dungeon submodule timing missed the Sprite_Main return checkpoint");
            if sprite_main_return_nmis == submodule_nmis {
                sprite_main_boundary = None;
            }
            let mut reenters_main_loop_before_nmi = false;
            for _ in 0..64 {
                let (scanline, master_cycle) = budget.raster_position().coordinates();
                run.set_raster_position(scanline, master_cycle);
                if advance_rom_cpu_step(&mut run, &mut budget)
                    .reached_boundary()
                    .is_some()
                {
                    break;
                }
                if run.pc() == DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.entry_pc {
                    reenters_main_loop_before_nmi = true;
                    break;
                }
                if run.pc() == DUNGEON_MAIN_WAIT_CPU_CHECKPOINT.entry_pc {
                    break;
                }
            }
            return DungeonSubmoduleCpuSchedule {
                submodule_nmis,
                caller_nmis: nmis.saturating_sub(submodule_nmis),
                caller_sprite_main_nmis: sprite_main_return_nmis.saturating_sub(submodule_nmis),
                caller_suffix_nmis: nmis.saturating_sub(sprite_main_return_nmis),
                caller_first_nmi_phase,
                sprite_main_boundary,
                cached_sprite_interruption,
                reenters_main_loop_before_nmi,
            };
        }

        let pc = run.pc();
        if submodule_nmis.is_some() && pc == SPRITE_EXECUTE_SINGLE_ENTRY_PC {
            let slot = run.ram_byte(CUR_OBJECT_INDEX);
            assert!(slot < 16, "Sprite_Main entered an invalid slot {slot}");
            sprite_main_current_slot = Some(slot);
        }
        if submodule_nmis.is_some() && pc == SPRITE_SLOT_RETURN_PC {
            sprite_main_last_completed_slot = sprite_main_current_slot;
        }
        if submodule_nmis.is_some()
            && pc == SPRITE_MAIN_RETURN_PC
            && sprite_main_return_nmis.is_none()
        {
            sprite_main_return_nmis = Some(nmis);
        }
        if sprite_main_return_nmis.is_some() && pc == 0x0d_a18e {
            link_oam_started = true;
        }
        if submodule_nmis.is_some() && pc == 0x00_85fc {
            nmi_prepare_sprites_started = true;
        }
        if submodule_nmis.is_some() && pc == 0x1d_ea00 {
            cached_sprite_copy = Some(CachedSpriteCpuProgress::at_entry(
                run.ram_byte(CUR_OBJECT_INDEX),
            ));
        }
        if let Some(copy) = cached_sprite_copy.as_mut() {
            copy.observe_pc(pc);
        }
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        let prior_subsubmodule = run.ram_byte(SUBSUBMODULE);
        let advance = advance_rom_cpu_step(&mut run, &mut budget);
        for (address, value) in run.take_cpu_wram_writes() {
            if let Some(copy) = cached_sprite_copy.as_mut() {
                copy.observe_wram_write(address);
            }
            if submodule_nmis.is_some()
                && (SPRITE_STATE..SPRITE_STATE + 16).contains(&address)
                && value == 9
            {
                initialized_sprite_slot = Some((address - SPRITE_STATE) as u8);
            }
        }
        if submodule_nmis.is_none()
            && prior_subsubmodule == entry_subsubmodule
            && run.ram_byte(SUBSUBMODULE) != entry_subsubmodule
        {
            submodule_nmis = Some(nmis);
        }
        if advance.reached_boundary().is_some() {
            if submodule_nmis.is_some() && !caller_first_nmi_observed {
                caller_first_nmi_observed = true;
                sprite_main_boundary = sprite_main_cpu_interruption_boundary(
                    sprite_main_current_slot,
                    sprite_main_last_completed_slot,
                    initialized_sprite_slot,
                    run.stack_return_address(),
                );
                cached_sprite_interruption =
                    cached_sprite_copy.and_then(CachedSpriteCpuProgress::interruption);
                caller_first_nmi_phase = Some(if nmi_prepare_sprites_started {
                    ModuleCpuPhase::InterruptedInNmiPrepareSprites
                } else if link_oam_started {
                    ModuleCpuPhase::InterruptedInLinkOam
                } else if sprite_main_return_nmis.is_some() {
                    ModuleCpuPhase::InterruptedAfterSpriteMain
                } else {
                    ModuleCpuPhase::InterruptedInSpriteMain
                });
                if crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
                    let slot = sprite_main_current_slot.unwrap_or(0);
                    eprintln!(
                        "dungeon_submodule_first_caller_nmi pc={:06x} cached_progress={cached_sprite_copy:?} live_slot={} state={:02x} type={:02x} x={:02x}",
                        run.pc(),
                        slot,
                        run.ram_byte(SPRITE_STATE + usize::from(slot)),
                        run.ram_byte(SPRITE_TYPE + usize::from(slot)),
                        run.ram_byte(SPRITE_X_LO + usize::from(slot)),
                    );
                }
            }
            nmis = nmis
                .checked_add(1)
                .expect("spiral-room NMI count overflowed");
            advance_rom_cpu_through_nmi(&mut run, &mut budget);
        }
    }
    panic!(
        "dungeon submodule ROM timing did not reach the main wait; stopped at {:06x}",
        run.pc(),
    );
}

fn module09_cpu_schedule(state: &ZeldaState) -> Module09CpuSchedule {
    const MODULE09_ENTRY_PC: u32 = 0x02_a475;
    const SPRITE_MAIN_RETURN_PC: u32 = 0x02_a4b5;
    const SPRITE_EXECUTE_SINGLE_ENTRY_PC: u32 = 0x06_84e2;
    const SPRITE_SLOT_RETURN_PC: u32 = 0x06_83a7;
    const LINK_OAM_ENTRY_PC: u32 = 0x0d_a18e;
    const NMI_PREPARE_SPRITES_ENTRY_PC: u32 = 0x00_85fc;

    let entry = state.game_state.frame;
    assert_eq!(entry.main_module, 9);
    assert!(matches!(entry.submodule, 0x20 | 0x21));

    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        DUNGEON_MAIN_WAIT_CPU_CHECKPOINT,
    )
    .expect("Module09 CPU timing requires the loaded Zelda ROM");
    let field_timing = CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0);
    let mut budget =
        CpuCycleBudget::at_nmi_acceptance(CpuBusWorkload::with_dynamic_hdma(), field_timing);
    advance_rom_cpu_through_nmi(&mut run, &mut budget);
    while run.pc() != MODULE09_ENTRY_PC {
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        assert_eq!(
            advance_rom_cpu_step(&mut run, &mut budget),
            CpuWorkAdvance::Complete,
            "main-loop prefix crossed another NMI before Module09 at {:06x}",
            run.pc(),
        );
    }

    let entry_submodule = run.ram_byte(SUBMODULE);
    assert_eq!(entry_submodule, entry.submodule);
    let mut nmis = 0u8;
    let mut submodule_nmis = None;
    let mut sprite_main_return_nmis: Option<u8> = None;
    let mut sprite_main_current_slot = None;
    let mut sprite_main_last_completed_slot = None;
    let mut initialized_sprite_slot = None;
    let mut sprite_main_boundary = None;
    let mut caller_first_nmi_phase = None;
    let mut link_oam_started = false;
    let mut nmi_prepare_sprites_started = false;

    run.enable_cpu_write_trace();

    for _ in 0..5_000_000 {
        if run.is_complete() {
            let submodule_return_nmis =
                submodule_nmis.expect("Module09 CPU timing missed the submodule completion write");
            let sprite_main_return_nmis = sprite_main_return_nmis
                .expect("Module09 CPU timing missed the Sprite_Main return checkpoint");
            let caller_nmis = nmis.saturating_sub(submodule_return_nmis);
            if sprite_main_return_nmis == submodule_return_nmis {
                sprite_main_boundary = None;
            }
            // If the body returns and Module09 is interrupted before this
            // host's trailing NMI, that boundary is shared: it is both the
            // body's final scheduled slice and the caller's first crossing.
            // A caller which reaches the main wait first has no shared slice.
            let submodule_nmis = submodule_return_nmis + u8::from(caller_nmis != 0);
            return Module09CpuSchedule {
                submodule_nmis,
                caller_nmis,
                caller_sprite_main_nmis: sprite_main_return_nmis
                    .saturating_sub(submodule_return_nmis),
                caller_suffix_nmis: nmis.saturating_sub(sprite_main_return_nmis),
                caller_first_nmi_phase,
                sprite_main_boundary,
            };
        }

        let pc = run.pc();
        if submodule_nmis.is_some() && pc == SPRITE_EXECUTE_SINGLE_ENTRY_PC {
            let slot = run.ram_byte(CUR_OBJECT_INDEX);
            assert!(slot < 16, "Sprite_Main entered an invalid slot {slot}");
            sprite_main_current_slot = Some(slot);
        }
        if submodule_nmis.is_some() && pc == SPRITE_SLOT_RETURN_PC {
            sprite_main_last_completed_slot = sprite_main_current_slot;
        }
        if submodule_nmis.is_some()
            && pc == SPRITE_MAIN_RETURN_PC
            && sprite_main_return_nmis.is_none()
        {
            sprite_main_return_nmis = Some(nmis);
        }
        if sprite_main_return_nmis.is_some() && pc == LINK_OAM_ENTRY_PC {
            link_oam_started = true;
        }
        if submodule_nmis.is_some() && pc == NMI_PREPARE_SPRITES_ENTRY_PC {
            nmi_prepare_sprites_started = true;
        }

        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        let prior_submodule = run.ram_byte(SUBMODULE);
        let advance = advance_rom_cpu_step(&mut run, &mut budget);
        for (address, value) in run.take_cpu_wram_writes() {
            if submodule_nmis.is_some()
                && (SPRITE_STATE..SPRITE_STATE + 16).contains(&address)
                && value == 9
            {
                initialized_sprite_slot = Some((address - SPRITE_STATE) as u8);
            }
        }
        if submodule_nmis.is_none()
            && prior_submodule == entry_submodule
            && run.ram_byte(SUBMODULE) != entry_submodule
        {
            submodule_nmis = Some(nmis);
        }
        if advance.reached_boundary().is_some() {
            if submodule_nmis.is_some() && caller_first_nmi_phase.is_none() {
                sprite_main_boundary = sprite_main_cpu_interruption_boundary(
                    sprite_main_current_slot,
                    sprite_main_last_completed_slot,
                    initialized_sprite_slot,
                    run.stack_return_address(),
                );
                caller_first_nmi_phase = Some(if nmi_prepare_sprites_started {
                    ModuleCpuPhase::InterruptedInNmiPrepareSprites
                } else if link_oam_started {
                    ModuleCpuPhase::InterruptedInLinkOam
                } else if sprite_main_return_nmis.is_some() {
                    ModuleCpuPhase::InterruptedAfterSpriteMain
                } else {
                    ModuleCpuPhase::InterruptedInSpriteMain
                });
            }
            nmis = nmis.checked_add(1).expect("Module09 NMI count overflowed");
            advance_rom_cpu_through_nmi(&mut run, &mut budget);
        }
    }
    panic!(
        "Module09 ROM timing did not reach the main wait; stopped at {:06x}",
        run.pc(),
    );
}

pub(super) fn palette_filter_bounce_loop_master_cycles(state: &ZeldaState) -> u32 {
    palette_filter_bounce_loop_master_cycles_for_countdown(
        state,
        state.game_state.display.palette_filter.countdown_word(),
    )
}

fn palette_filter_bounce_loop_master_cycles_for_countdown(
    state: &ZeldaState,
    countdown: u16,
) -> u32 {
    [0..1, 0x20..0xd8, 0xe0..0xf0]
        .into_iter()
        .flatten()
        .map(|index| {
            let work = zelda3_palette::filter_range_step_work(
                state.game_state.display.palette_buffer.aux_color(index),
                countdown,
            );
            PALETTE_FILTER_ZERO_AUX_WORD_MASTER_CYCLES
                + u32::from(work.aux_nonzero) * PALETTE_FILTER_NONZERO_AUX_EXTRA_MASTER_CYCLES
                + u32::from(work.red_steps) * PALETTE_FILTER_RED_STEP_EXTRA_MASTER_CYCLES
                + u32::from(work.green_steps) * PALETTE_FILTER_GREEN_STEP_EXTRA_MASTER_CYCLES
                + u32::from(work.blue_steps) * PALETTE_FILTER_BLUE_STEP_EXTRA_MASTER_CYCLES
        })
        .sum()
}

fn dungeon_supertile_state_9_cpu_advance(
    palette_filter_loop_master_cycles: Option<u32>,
) -> CpuPhaseSequenceAdvance {
    let body = palette_filter_loop_master_cycles
        .map_or(DUNGEON_QUADRANT_PREPARE_MASTER_CYCLES, |palette_work| {
            DUNGEON_STATE_9_FILTERED_FIXED_MASTER_CYCLES + palette_work
        });
    let mut budget = CpuCycleBudget::until_next_vblank_publication(
        DUNGEON_STATE_9_CPU_ENTRY,
        CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL_MASTER_CYCLES),
        CpuFieldTiming::NON_INTERLACE_EVEN,
    );
    budget.advance_phases(&[body, DUNGEON_MODULE_CALLER_SUFFIX_MASTER_CYCLES])
}

fn dungeon_supertile_state_9_caller_continuation(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    palette_filter_loop_master_cycles: Option<u32>,
) -> Option<PreMainNmiResume> {
    (entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 9
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 10
        && matches!(
            dungeon_supertile_state_9_cpu_advance(palette_filter_loop_master_cycles),
            CpuPhaseSequenceAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::VblankPublication,
                phase_index: 1,
                ..
            }
        ))
    .then_some(PreMainNmiResume::DungeonSupertileCallerReturnNmi)
}

fn dungeon_supertile_state_10_cpu_advance(
    palette_filter_loop_master_cycles: u32,
) -> CpuPhaseSequenceAdvance {
    let mut budget = CpuCycleBudget::until_next_vblank_publication(
        DUNGEON_STATE_10_CPU_ENTRY,
        CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL_MASTER_CYCLES),
        CpuFieldTiming::NON_INTERLACE_EVEN,
    );
    budget.advance_phases(&[
        DUNGEON_STATE_10_FILTERED_FIXED_MASTER_CYCLES + palette_filter_loop_master_cycles
    ])
}

fn dungeon_supertile_state_11_cpu_advance(
    palette_filter_loop_master_cycles: u32,
) -> CpuPhaseSequenceAdvance {
    let mut budget = CpuCycleBudget::until_next_vblank_publication(
        DUNGEON_STATE_11_CPU_ENTRY,
        CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL_MASTER_CYCLES),
        CpuFieldTiming::NON_INTERLACE_EVEN,
    );
    budget.advance_phases(&[
        DUNGEON_STATE_11_FILTERED_FIXED_MASTER_CYCLES + palette_filter_loop_master_cycles
    ])
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Module09AfterSpriteMain {
    #[default]
    Ordinary,
    FinishOverworldSpriteReload {
        post_return_hold_nmi_slices: u8,
        epilogue_phase: NmiPhase,
        resume_scanout: OverworldSpriteReloadResumeScanout,
    },
}

// Return address immediately after SpritePrep_Zelda's JSL to the shared
// follower-graphics loader. At this point C has completed the generic state-8
// initialization prefix and Zelda's prep prefix, but the current slot has not
// returned to Sprite_Main yet.
const ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS: u32 = 0x05_ebf5;

fn sprite_main_cpu_interruption_boundary(
    current_slot: Option<u8>,
    last_completed_slot: Option<u8>,
    initialized_slot: Option<u8>,
    stack_return_address: u32,
) -> Option<SpriteMainCpuBoundary> {
    if current_slot == initialized_slot
        && stack_return_address == ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS
    {
        initialized_slot.map(SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics)
    } else {
        last_completed_slot.map(SpriteMainCpuBoundary::AfterSlot)
    }
}

/// Whether a suspended direct item-receipt graphics call from `slot` sits in
/// the active invocation named by a `Sprite_Main` checkpoint. Coarse walk
/// checkpoints identify the next slot after the last completed one; typed
/// partial checkpoints identify the in-flight slot directly.
fn direct_item_receipt_slot_pairs_with_boundary(slot: u8, boundary: SpriteMainCpuBoundary) -> bool {
    match boundary {
        SpriteMainCpuBoundary::BeforeFirstSlot
        | SpriteMainCpuBoundary::Module09FinalScrollPairPending => slot == 15,
        SpriteMainCpuBoundary::AfterSlot(after_slot) => slot.checked_add(1) == Some(after_slot),
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterTimerDecrements {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterPrimaryTimerDecrements {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterHitTimer {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterMainTimerDecrement {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterZeroHitTimerClear {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::BariBeforeRandom(active_slot)
        | SpriteMainCpuBoundary::FollowerGraphics {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(active_slot)
        | SpriteMainCpuBoundary::AfterActiveCuccoX {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::BoulderMovement {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::InitializePrepMoveY {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterCuccoFleeMovement {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(active_slot)
        | SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(active_slot)
        | SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(active_slot)
        | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(active_slot)
        | SpriteMainCpuBoundary::TrinexxHeadDrawSetup(active_slot)
        | SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(active_slot)
        | SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(active_slot)
        | SpriteMainCpuBoundary::LaserEyeDrawPrologue(active_slot)
        | SpriteMainCpuBoundary::ZoraFireballMovement {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(active_slot)
        | SpriteMainCpuBoundary::WishPondTossedItemGraphics {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(active_slot)
        | SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::BonkItemGraphicsEntered(active_slot)
        | SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterWallmasterResetPrefix(active_slot)
        | SpriteMainCpuBoundary::WallmasterResetClear {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::ZazakAfterGraphics(active_slot)
        | SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::InitializeResetProperties {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::InitializeLoadProperties {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(active_slot)
        | SpriteMainCpuBoundary::FireDebirandoSpawn {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::HelmasaurHardHatTileCollision {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::LanmolaDrawPrefix {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment {
            slot: active_slot,
        }
        | SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot: active_slot }
        | SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot: active_slot }
        | SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot: active_slot }
        | SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot: active_slot }
        | SpriteMainCpuBoundary::SwamolaHeadDraw { slot: active_slot }
        | SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot: active_slot }
        | SpriteMainCpuBoundary::MoblinAttributeLoaded { slot: active_slot }
        | SpriteMainCpuBoundary::MoblinCollisionGeometry { slot: active_slot }
        | SpriteMainCpuBoundary::VitreousDamagePending { slot: active_slot }
        | SpriteMainCpuBoundary::VitreousAiPending { slot: active_slot }
        | SpriteMainCpuBoundary::MiniMoldormAiPending { slot: active_slot }
        | SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot: active_slot }
        | SpriteMainCpuBoundary::SwamolaSegmentDraw {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::TrinexxHeadDraw {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::TrinexxHeadFrontPart {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::PengatorSlidePending { slot: active_slot }
        | SpriteMainCpuBoundary::AntifairyBouncePending { slot: active_slot }
        | SpriteMainCpuBoundary::KholdstareDamagePending { slot: active_slot }
        | SpriteMainCpuBoundary::InitializePrepPending { slot: active_slot }
        | SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::GuardPrepParryHitbox {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::GuardPrepPatrolDelay {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::GuardAnimation {
            slot: active_slot, ..
        }
        | SpriteMainCpuBoundary::MiniMoldormHistory {
            slot: active_slot, ..
        } => active_slot == slot,
    }
}

/// Whether a scheduled caller return may run its parked dungeon Sprite_Main
/// remainder before the host's leading NMI handler (route host 1415870).
fn scheduled_caller_return_runs_dungeon_sprite_main_before_leading_nmi(
    work: GameWorkContinuation,
) -> bool {
    matches!(
        work,
        GameWorkContinuation::FinishSpriteMain {
            caller: SpriteMainCpuCaller::DungeonModule07
                | SpriteMainCpuCaller::DungeonModule07Live { .. },
            ..
        }
    )
}

fn sprite_main_cpu_boundary_from_interruption(
    interruption: crate::MainLoopInterruption,
) -> Option<SpriteMainCpuBoundary> {
    match interruption {
        crate::MainLoopInterruption::SpriteMainBeforeFirstSlot => {
            Some(SpriteMainCpuBoundary::BeforeFirstSlot)
        }
        crate::MainLoopInterruption::SpriteMainAfterSlot(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterSlot(slot))
        }
        crate::MainLoopInterruption::SpriteMainAfterTimersAndOam(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main timer/OAM receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainAfterTimerDecrements(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main timer decrement receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterTimerDecrements { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main primary timer decrement receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainAfterHitTimer(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main hit timer decrement receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterHitTimer { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main main/aux1 timer decrement receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainAfterMainTimerDecrement(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main main timer decrement receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterMainTimerDecrement { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainAfterZeroHitTimerClear(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main main timer decrement receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterZeroHitTimerClear { slot, state: None })
        }
        crate::MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source bonk-item graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::BonkItemGraphicsEntered(slot))
        }
        crate::MainLoopInterruption::SpriteMainBariBeforeRandom(slot) => {
            assert!(
                slot < 16,
                "source Bari prep receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::BariBeforeRandom(slot))
        }
        crate::MainLoopInterruption::SpriteMainFollowerGraphics {
            slot,
            caller,
            stage,
        } => {
            assert!(
                slot < 16,
                "source Zelda follower-graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller,
                prefix_completed: false,
                saved_follower_indicator: None,
                stage,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterThrowableSceneryStateClear(slot) => {
            assert!(
                slot < 16,
                "source throwable-scenery receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(slot))
        }
        crate::MainLoopInterruption::SpriteMainAfterActiveCuccoX {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco X receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterActiveCuccoX {
                slot,
                helper_ordinal,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterActiveCuccoYSubpixel {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco movement receipt used invalid slot {slot}",
            );
            Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
                y_low: None,
                y_high: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainMasterSwordLightBeamMovement {
            slot,
            checkpoint,
        } => {
            assert!(
                slot < 16,
                "source master-sword movement used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                slot,
                checkpoint,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainBoulderMovement { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source Boulder movement used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::BoulderMovement {
                slot,
                checkpoint,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainInitializePrepMoveY { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source state-8 prep Sprite_MoveY used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::InitializePrepMoveY {
                slot,
                checkpoint,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainMasterSwordLightBeamSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            Some(SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
                slot,
                spawned_slot,
                progress,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterCuccoFleeMovement {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco flee receipt used invalid slot {slot}",
            );
            Some(SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements {
            slot,
            helper_ordinal,
            completed,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco subtype receipt used invalid slot {slot}",
            );
            assert!((1..=5).contains(&completed));
            Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                slot,
                helper_ordinal,
                completed,
                total: 0,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco receipt used invalid slot {slot}",
            );
            Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source big-key graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(slot))
        }
        crate::MainLoopInterruption::SpriteMainKingZoraFlippersGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source King Zora flippers graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(slot))
        }
        crate::MainLoopInterruption::SpriteMainHappinessPondRupeeGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Happiness Pond rupee graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(
                slot,
            ))
        }
        crate::MainLoopInterruption::SpriteMainCatfishMedallionGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Catfish medallion graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(slot))
        }
        crate::MainLoopInterruption::SpriteMainTrinexxHeadDrawSetup(slot) => {
            assert!(
                slot < 16,
                "source Trinexx medallion graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::TrinexxHeadDrawSetup(slot))
        }
        crate::MainLoopInterruption::SpriteMainWaterfallGtCutsceneGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Waterfall GT cutscene graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(
                slot,
            ))
        }
        crate::MainLoopInterruption::SpriteMainTrinexxBreathTileCollisionReturned(slot) => {
            assert!(
                slot < 16,
                "source Trinexx breath tile-collision receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(
                slot,
            ))
        }
        crate::MainLoopInterruption::SpriteMainLaserEyeDrawPrologue(slot) => {
            assert!(
                slot < 16,
                "source Trinexx breath tile-collision receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::LaserEyeDrawPrologue(slot))
        }
        crate::MainLoopInterruption::SpriteMainZoraFireballMovement { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source Zora fireball movement used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::ZoraFireballMovement {
                slot,
                checkpoint,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterSingleSmallDrawPosition(slot) => {
            assert!(
                slot < 16,
                "source single-small draw receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                slot,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainWallmasterResetClear {
            slot,
            cleared_bytes,
        } => {
            assert!(slot < 16 && cleared_bytes <= 0x1000);
            Some(SpriteMainCpuBoundary::WallmasterResetClear {
                slot,
                cleared_bytes,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterWallmasterResetPrefix(slot) => {
            assert!(
                slot < 16,
                "source Wallmaster reset receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterWallmasterResetPrefix(slot))
        }
        crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source item-receipt graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(slot))
        }
        crate::MainLoopInterruption::SpriteMainWishPondTossedItemGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Wish Pond graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::WishPondTossedItemGraphics {
                slot,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainZazakAfterGraphics(slot) => {
            assert!(
                slot < 16,
                "source Zazak graphics receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::ZazakAfterGraphics(slot))
        }
        crate::MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(slot) => {
            assert!(
                slot < 16,
                "source guard-probe receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                slot,
                oam_position: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainInitializeResetProperties {
            slot,
            phase,
            completed_stores,
        } => {
            assert!(
                slot < 16,
                "source sprite-init reset receipt used invalid slot {slot}"
            );
            assert!(
                completed_stores <= 40,
                "source sprite-init reset receipt exceeded 40 stores: {completed_stores}",
            );
            Some(SpriteMainCpuBoundary::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            })
        }
        crate::MainLoopInterruption::SpriteMainInitializeLoadProperties {
            slot,
            phase,
            completed_stores,
        } => {
            assert!(
                slot < 16,
                "source property-load receipt used invalid slot {slot}"
            );
            assert!(
                completed_stores <= 10,
                "source property-load receipt exceeded 10 stores: {completed_stores}",
            );
            Some(SpriteMainCpuBoundary::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            })
        }
        crate::MainLoopInterruption::SpriteMainFireDebirandoBeforeSpawn(slot) => {
            assert!(
                slot < 16,
                "source Fire Debirando receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(slot))
        }
        crate::MainLoopInterruption::SpriteMainFireDebirandoSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            Some(SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            })
        }
        crate::MainLoopInterruption::SpriteMainTrinexxDeathExplosionSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            Some(SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
                slot,
                spawned_slot,
                progress,
            })
        }
        crate::MainLoopInterruption::SpriteMainAgahnimMotionBlurSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            Some(SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
                slot,
                spawned_slot,
                progress,
                bound: false,
            })
        }
        crate::MainLoopInterruption::SpriteMainTrinexxFinalPhaseTileCollision {
            slot,
            probes_completed,
        } => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
                slot,
                probes_completed,
            })
        }
        crate::MainLoopInterruption::SpriteMainHelmasaurHardHatTileCollision { slot, stage } => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::HelmasaurHardHatTileCollision { slot, stage })
        }
        crate::MainLoopInterruption::SpriteMainLanmolaDrawPrefix {
            slot,
            completed_stores,
        } => {
            assert!(slot < 16 && completed_stores <= 5);
            Some(SpriteMainCpuBoundary::LanmolaDrawPrefix {
                slot,
                completed_stores,
            })
        }
        crate::MainLoopInterruption::SpriteMainTrinexxFinalPhaseDraw {
            slot,
            segment,
            stage,
        } => {
            assert!(valid_trinexx_final_phase_draw_stage(segment, stage));
            Some(SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
                slot,
                segment,
                stage,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(slot) => {
            assert!(
                slot < 16,
                "source Antfairy subtype2 receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                slot,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterLanmolaSubtype2Increment(slot) => {
            assert!(
                slot < 16,
                "source Lanmola subtype2 receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                slot,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainAfterHelmasaurHardHatBeetleSubtype2Increment(
            slot,
        ) => {
            assert!(
                slot < 16,
                "source Helmasaur/Hardhat subtype2 receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment { slot })
        }
        crate::MainLoopInterruption::SpriteMainGuardPrepPatrolDelay { slot, active_call } => {
            assert!(slot < 16 && (1..=2).contains(&active_call));
            Some(SpriteMainCpuBoundary::GuardPrepPatrolDelay {
                slot,
                active_call,
                saved_submodule: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainGuardPrepTileCollisionReturned {
            slot,
            active_call,
        } => {
            assert!(slot < 16 && (1..=2).contains(&active_call));
            Some(SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
                slot,
                active_call,
                saved_submodule: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainInitializePrepPending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::InitializePrepPending { slot })
        }
        crate::MainLoopInterruption::SpriteMainHogSpearBodyGraphicsPending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot })
        }
        crate::MainLoopInterruption::SpriteMainBuzzblobAfterXSubpixel(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
                slot,
                pending: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainAbsorbableHorizontalTileLookup(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot })
        }
        crate::MainLoopInterruption::SpriteMainAbsorbableVerticalTileLookup(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot })
        }
        crate::MainLoopInterruption::SpriteMainAbsorbableVerticalTileAttributeLoaded(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot })
        }
        crate::MainLoopInterruption::SpriteMainSwamolaHeadDraw(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::SwamolaHeadDraw { slot })
        }
        crate::MainLoopInterruption::SpriteMainSwamolaHeadDrawCompleted(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot })
        }
        crate::MainLoopInterruption::SpriteMainMoblinAttributeLoaded(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::MoblinAttributeLoaded { slot })
        }
        crate::MainLoopInterruption::SpriteMainMoblinCollisionGeometry(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::MoblinCollisionGeometry { slot })
        }
        crate::MainLoopInterruption::SpriteMainVitreousDamagePending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::VitreousDamagePending { slot })
        }
        crate::MainLoopInterruption::SpriteMainVitreousAiPending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::VitreousAiPending { slot })
        }
        crate::MainLoopInterruption::SpriteMainMiniMoldormAiPending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::MiniMoldormAiPending { slot })
        }
        crate::MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot })
        }
        crate::MainLoopInterruption::SpriteMainSwamolaSegmentDraw { slot, segment } => {
            assert!(slot < 16 && segment < 4);
            Some(SpriteMainCpuBoundary::SwamolaSegmentDraw { slot, segment })
        }
        crate::MainLoopInterruption::SpriteMainTrinexxHeadDraw { slot, segment } => {
            assert!(slot < 16 && segment < 9);
            Some(SpriteMainCpuBoundary::TrinexxHeadDraw {
                slot,
                segment,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainSidenexxNeckTargetLoop { slot, step } => {
            assert!(slot < 16 && step < 56);
            Some(SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
                slot,
                step,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainTrinexxHeadFrontPart {
            slot,
            completed_stores,
        } => {
            assert!(slot < 16 && completed_stores <= 30);
            Some(SpriteMainCpuBoundary::TrinexxHeadFrontPart {
                slot,
                completed_stores,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainPengatorSlidePending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::PengatorSlidePending { slot })
        }
        crate::MainLoopInterruption::SpriteMainAntifairyBouncePending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::AntifairyBouncePending { slot })
        }
        crate::MainLoopInterruption::SpriteMainKholdstareDamagePending(slot) => {
            assert!(slot < 16);
            Some(SpriteMainCpuBoundary::KholdstareDamagePending { slot })
        }
        crate::MainLoopInterruption::SpriteMainGuardPrepWeaponFlagsPending(slot) => {
            assert!(
                slot < 16,
                "source guard-prep weapon receipt used invalid slot {slot}"
            );
            Some(SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                slot,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainGuardPrepParryHitbox { slot, active_call } => {
            assert!(slot < 16 && (1..=2).contains(&active_call));
            Some(SpriteMainCpuBoundary::GuardPrepParryHitbox {
                slot,
                active_call,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainGuardAnimation { slot, checkpoint } => {
            assert!(slot < 16 && checkpoint.is_valid());
            Some(SpriteMainCpuBoundary::GuardAnimation {
                slot,
                checkpoint,
                continuation: None,
            })
        }
        crate::MainLoopInterruption::SpriteMainMiniMoldormHistory {
            slot,
            completed_stores,
        } => {
            assert!(slot < 16);
            assert!(completed_stores <= 128);
            Some(SpriteMainCpuBoundary::MiniMoldormHistory {
                slot,
                completed_stores,
            })
        }
        _ => None,
    }
}

/// `Sprite_TrinexxD_Draw` stages: 0..8 are the per-segment steps; 16..30 are
/// segment 4's flashing-segment damage check after that many of its stores.
const fn valid_trinexx_final_phase_draw_stage(segment: u8, stage: u8) -> bool {
    segment < 24 && (stage < 8 || (segment == 4 && stage >= 16 && stage < 30))
}

const fn valid_dynamic_spawn_progress(progress: crate::SpriteDynamicSpawnProgress) -> bool {
    match progress {
        crate::SpriteDynamicSpawnProgress::ResetProperties { completed_stores } => {
            completed_stores <= 40
        }
        crate::SpriteDynamicSpawnProgress::LoadProperties { completed_stores } => {
            completed_stores <= 10
        }
        _ => true,
    }
}

const fn valid_sprite_main_interruption(interruption: crate::MainLoopInterruption) -> bool {
    match interruption {
        crate::MainLoopInterruption::SpriteMainSwamolaSegmentDraw { slot, segment } => {
            slot < 16 && segment < 4
        }
        crate::MainLoopInterruption::SpriteMainTrinexxHeadDraw { slot, segment } => {
            slot < 16 && segment < 9
        }
        crate::MainLoopInterruption::SpriteMainSidenexxNeckTargetLoop { slot, step } => {
            slot < 16 && step < 56
        }
        crate::MainLoopInterruption::SpriteMainTrinexxHeadFrontPart {
            slot,
            completed_stores,
        } => slot < 16 && completed_stores <= 30,
        crate::MainLoopInterruption::SpriteMainBeforeFirstSlot => true,
        crate::MainLoopInterruption::SpriteMainAfterSlot(slot)
        | crate::MainLoopInterruption::SpriteMainAfterTimersAndOam(slot)
        | crate::MainLoopInterruption::SpriteMainAfterTimerDecrements(slot)
        | crate::MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(slot)
        | crate::MainLoopInterruption::SpriteMainAfterHitTimer(slot)
        | crate::MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(slot)
        | crate::MainLoopInterruption::SpriteMainAfterMainTimerDecrement(slot)
        | crate::MainLoopInterruption::SpriteMainAfterZeroHitTimerClear(slot)
        | crate::MainLoopInterruption::SpriteMainBariBeforeRandom(slot)
        | crate::MainLoopInterruption::SpriteMainAfterThrowableSceneryStateClear(slot)
        | crate::MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainKingZoraFlippersGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainHappinessPondRupeeGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainCatfishMedallionGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainTrinexxHeadDrawSetup(slot)
        | crate::MainLoopInterruption::SpriteMainWaterfallGtCutsceneGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainTrinexxBreathTileCollisionReturned(slot)
        | crate::MainLoopInterruption::SpriteMainLaserEyeDrawPrologue(slot)
        | crate::MainLoopInterruption::SpriteMainZoraFireballMovement { slot, .. }
        | crate::MainLoopInterruption::SpriteMainAfterSingleSmallDrawPosition(slot)
        | crate::MainLoopInterruption::SpriteMainAfterWallmasterResetPrefix(slot)
        | crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainWishPondTossedItemGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainZazakAfterGraphics(slot)
        | crate::MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(slot)
        | crate::MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(slot)
        | crate::MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(slot)
        | crate::MainLoopInterruption::SpriteMainAfterLanmolaSubtype2Increment(slot)
        | crate::MainLoopInterruption::SpriteMainAfterHelmasaurHardHatBeetleSubtype2Increment(
            slot,
        )
        | crate::MainLoopInterruption::SpriteMainBuzzblobAfterXSubpixel(slot)
        | crate::MainLoopInterruption::SpriteMainHogSpearBodyGraphicsPending(slot)
        | crate::MainLoopInterruption::SpriteMainAbsorbableHorizontalTileLookup(slot)
        | crate::MainLoopInterruption::SpriteMainAbsorbableVerticalTileLookup(slot)
        | crate::MainLoopInterruption::SpriteMainAbsorbableVerticalTileAttributeLoaded(slot)
        | crate::MainLoopInterruption::SpriteMainSwamolaHeadDraw(slot)
        | crate::MainLoopInterruption::SpriteMainSwamolaHeadDrawCompleted(slot)
        | crate::MainLoopInterruption::SpriteMainMoblinAttributeLoaded(slot)
        | crate::MainLoopInterruption::SpriteMainMoblinCollisionGeometry(slot)
        | crate::MainLoopInterruption::SpriteMainVitreousDamagePending(slot)
        | crate::MainLoopInterruption::SpriteMainVitreousAiPending(slot)
        | crate::MainLoopInterruption::SpriteMainMiniMoldormAiPending(slot)
        | crate::MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(slot)
        | crate::MainLoopInterruption::SpriteMainPengatorSlidePending(slot)
        | crate::MainLoopInterruption::SpriteMainAntifairyBouncePending(slot)
        | crate::MainLoopInterruption::SpriteMainKholdstareDamagePending(slot)
        | crate::MainLoopInterruption::SpriteMainInitializePrepPending(slot)
        | crate::MainLoopInterruption::SpriteMainGuardPrepWeaponFlagsPending(slot) => slot < 16,
        crate::MainLoopInterruption::SpriteMainWallmasterResetClear {
            slot,
            cleared_bytes,
        } => slot < 16 && cleared_bytes <= 0x1000,
        crate::MainLoopInterruption::SpriteMainGuardPrepPatrolDelay { slot, active_call }
        | crate::MainLoopInterruption::SpriteMainGuardPrepTileCollisionReturned {
            slot,
            active_call,
        }
        | crate::MainLoopInterruption::SpriteMainGuardPrepParryHitbox { slot, active_call } => {
            slot < 16 && active_call >= 1 && active_call <= 2
        }
        crate::MainLoopInterruption::SpriteMainGuardAnimation { slot, checkpoint } => {
            slot < 16 && checkpoint.is_valid()
        }
        crate::MainLoopInterruption::SpriteMainMasterSwordLightBeamMovement {
            slot,
            checkpoint: _,
        } => slot < 16,
        crate::MainLoopInterruption::SpriteMainBoulderMovement { slot, .. } => slot < 16,
        crate::MainLoopInterruption::SpriteMainInitializePrepMoveY { slot, .. } => slot < 16,
        crate::MainLoopInterruption::SpriteMainMasterSwordLightBeamSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::MainLoopInterruption::SpriteMainMiniMoldormHistory {
            slot,
            completed_stores,
        } => slot < 16 && completed_stores <= 128,
        crate::MainLoopInterruption::SpriteMainInitializeResetProperties {
            slot,
            phase: _,
            completed_stores,
        } => slot < 16 && completed_stores <= 40,
        crate::MainLoopInterruption::SpriteMainInitializeLoadProperties {
            slot,
            completed_stores,
            ..
        } => slot < 16 && completed_stores <= 10,
        crate::MainLoopInterruption::SpriteMainFireDebirandoBeforeSpawn(slot) => slot < 16,
        crate::MainLoopInterruption::SpriteMainFireDebirandoSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::MainLoopInterruption::SpriteMainTrinexxDeathExplosionSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::MainLoopInterruption::SpriteMainAgahnimMotionBlurSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::MainLoopInterruption::SpriteMainTrinexxFinalPhaseTileCollision { slot, .. } => {
            slot < 16
        }
        crate::MainLoopInterruption::SpriteMainHelmasaurHardHatTileCollision { slot, .. } => {
            slot < 16
        }
        crate::MainLoopInterruption::SpriteMainLanmolaDrawPrefix {
            slot,
            completed_stores,
        } => slot < 16 && completed_stores <= 5,
        crate::MainLoopInterruption::SpriteMainTrinexxFinalPhaseDraw {
            slot,
            segment,
            stage,
        } => valid_trinexx_final_phase_draw_stage(segment, stage) && slot < 16,
        crate::MainLoopInterruption::SpriteMainFollowerGraphics { slot, stage, .. } => {
            slot < 16
                && match stage {
                    crate::RescuedMaidenInitializationStage::FirstFollowerSheet {
                        completed_bytes,
                    }
                    | crate::RescuedMaidenInitializationStage::SecondFollowerSheet {
                        completed_bytes,
                    } => completed_bytes <= 0x0600,
                    crate::RescuedMaidenInitializationStage::Conversion { completed_stores } => {
                        completed_stores <= 512
                    }
                }
        }
        crate::MainLoopInterruption::SpriteMainAfterActiveCuccoX { slot, .. }
        | crate::MainLoopInterruption::SpriteMainAfterActiveCuccoYSubpixel { slot, .. }
        | crate::MainLoopInterruption::SpriteMainAfterCuccoFleeMovement { slot, .. }
        | crate::MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication { slot, .. } => {
            slot < 16
        }
        crate::MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements {
            slot,
            completed,
            ..
        } => slot < 16 && completed >= 1 && completed <= 5,
        crate::MainLoopInterruption::LinkOam
        | crate::MainLoopInterruption::SpritePreparation
        | crate::MainLoopInterruption::DungeonExitSpotlightAfterSubmodule
        | crate::MainLoopInterruption::LinkActualVelocity { .. }
        | crate::MainLoopInterruption::LinkActualVelocityCompleted
        | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
        | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
        | crate::MainLoopInterruption::LinkPositionBeforeCoordinates => true,
        crate::MainLoopInterruption::LinkPositionAfterSubpixel { pass }
        | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { pass, .. }
        | crate::MainLoopInterruption::LinkPositionAfterCoordinates { pass } => {
            matches!(pass, 0 | 2 | 4)
        }
        crate::MainLoopInterruption::SpotlightGoalResetTable { completed_stores } => {
            completed_stores <= 224
        }
        crate::MainLoopInterruption::GameOverIrisGoalPaletteFill { completed_stores } => {
            completed_stores <= 96
        }
        crate::MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule,
            radius,
            progress,
            ..
        } => {
            source_subsubmodule >= 2
                && source_subsubmodule <= 4
                && radius >= 1
                && radius <= 0xbf
                && match progress {
                    crate::DesertPrayerIrisProgress::Setup { completed_writes } => {
                        completed_writes <= 4
                    }
                    crate::DesertPrayerIrisProgress::BeforeIteration { scanline } => {
                        scanline >= 0x8000 || scanline < 225
                    }
                    crate::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                        y_buffer, ..
                    }
                    | crate::DesertPrayerIrisProgress::AfterPrimaryTableWrite {
                        y_buffer, ..
                    }
                    | crate::DesertPrayerIrisProgress::AfterIteration { y_buffer, .. } => {
                        y_buffer >= 1 && (y_buffer as u16) <= radius + 1
                    }
                    crate::DesertPrayerIrisProgress::LoopComplete => true,
                }
        }
        crate::MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor {
            next_color, ..
        } => {
            matches!(next_color, 0 | 1)
                || (next_color >= 0x20 && next_color <= 0xd8)
                || (next_color >= 0xe0 && next_color <= 0xf0)
        }
        crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { next_group_start } => {
            next_group_start <= 28 && next_group_start & 3 == 0
        }
    }
}

const fn valid_sprite_main_progress(progress: crate::SpriteMainProgress) -> bool {
    match progress {
        crate::SpriteMainProgress::SwamolaSegmentDraw { slot, segment } => slot < 16 && segment < 4,
        crate::SpriteMainProgress::TrinexxHeadDraw { slot, segment } => slot < 16 && segment < 9,
        crate::SpriteMainProgress::SidenexxNeckTargetLoop { slot, step } => slot < 16 && step < 56,
        crate::SpriteMainProgress::TrinexxHeadFrontPart {
            slot,
            completed_stores,
        } => slot < 16 && completed_stores <= 30,
        crate::SpriteMainProgress::BeforeFirstSlot => true,
        crate::SpriteMainProgress::AfterSlot(slot)
        | crate::SpriteMainProgress::AfterTimersAndOam(slot)
        | crate::SpriteMainProgress::AfterTimerDecrements(slot)
        | crate::SpriteMainProgress::AfterPrimaryTimerDecrements(slot)
        | crate::SpriteMainProgress::AfterHitTimer(slot)
        | crate::SpriteMainProgress::AfterMainAndAux1TimerDecrements(slot)
        | crate::SpriteMainProgress::AfterMainTimerDecrement(slot)
        | crate::SpriteMainProgress::AfterZeroHitTimerClear(slot)
        | crate::SpriteMainProgress::BariBeforeRandom(slot)
        | crate::SpriteMainProgress::AfterThrowableSceneryStateClear(slot)
        | crate::SpriteMainProgress::BigKeyDropGraphicsStarted(slot)
        | crate::SpriteMainProgress::KingZoraFlippersGraphicsStarted(slot)
        | crate::SpriteMainProgress::HappinessPondRupeeGraphicsStarted(slot)
        | crate::SpriteMainProgress::CatfishMedallionGraphicsStarted(slot)
        | crate::SpriteMainProgress::TrinexxHeadDrawSetup(slot)
        | crate::SpriteMainProgress::WaterfallGtCutsceneGraphicsStarted(slot)
        | crate::SpriteMainProgress::TrinexxBreathTileCollisionReturned(slot)
        | crate::SpriteMainProgress::LaserEyeDrawPrologue(slot)
        | crate::SpriteMainProgress::ZoraFireballMovement { slot, .. }
        | crate::SpriteMainProgress::WishPondTossedItemGraphicsStarted(slot)
        | crate::SpriteMainProgress::AfterSingleSmallDrawPosition(slot)
        | crate::SpriteMainProgress::ZazakAfterGraphics(slot)
        | crate::SpriteMainProgress::BonkItemGraphicsStarted(slot)
        | crate::SpriteMainProgress::ProbeAfterOamCoordinates(slot)
        | crate::SpriteMainProgress::AfterAntfairySubtype2Increment(slot)
        | crate::SpriteMainProgress::AfterLanmolaSubtype2Increment(slot)
        | crate::SpriteMainProgress::AfterHelmasaurHardHatBeetleSubtype2Increment(slot)
        | crate::SpriteMainProgress::BuzzblobAfterXSubpixel(slot)
        | crate::SpriteMainProgress::HogSpearBodyGraphicsPending(slot)
        | crate::SpriteMainProgress::AbsorbableHorizontalTileLookup(slot)
        | crate::SpriteMainProgress::AbsorbableVerticalTileLookup(slot)
        | crate::SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(slot)
        | crate::SpriteMainProgress::SwamolaHeadDraw(slot)
        | crate::SpriteMainProgress::SwamolaHeadDrawCompleted(slot)
        | crate::SpriteMainProgress::MoblinAttributeLoaded(slot)
        | crate::SpriteMainProgress::MoblinCollisionGeometry(slot)
        | crate::SpriteMainProgress::VitreousDamagePending(slot)
        | crate::SpriteMainProgress::VitreousAiPending(slot)
        | crate::SpriteMainProgress::MiniMoldormAiPending(slot)
        | crate::SpriteMainProgress::VitreousPlayerDamagePending(slot)
        | crate::SpriteMainProgress::PengatorSlidePending(slot)
        | crate::SpriteMainProgress::AntifairyBouncePending(slot)
        | crate::SpriteMainProgress::KholdstareDamagePending(slot)
        | crate::SpriteMainProgress::InitializePrepPending(slot)
        | crate::SpriteMainProgress::GuardPrepWeaponFlagsPending(slot) => slot < 16,
        crate::SpriteMainProgress::GuardPrepPatrolDelay { slot, active_call }
        | crate::SpriteMainProgress::GuardPrepTileCollisionReturned { slot, active_call }
        | crate::SpriteMainProgress::GuardPrepParryHitbox { slot, active_call } => {
            slot < 16 && active_call >= 1 && active_call <= 2
        }
        crate::SpriteMainProgress::GuardAnimation { slot, checkpoint } => {
            slot < 16 && checkpoint.is_valid()
        }
        crate::SpriteMainProgress::MiniMoldormHistory {
            slot,
            completed_stores,
        } => slot < 16 && completed_stores <= 128,
        crate::SpriteMainProgress::InitializeResetProperties {
            slot,
            phase: _,
            completed_stores,
        } => slot < 16 && completed_stores <= 40,
        crate::SpriteMainProgress::InitializeLoadProperties {
            slot,
            completed_stores,
            ..
        } => slot < 16 && completed_stores <= 10,
        crate::SpriteMainProgress::FireDebirandoBeforeSpawn(slot) => slot < 16,
        crate::SpriteMainProgress::FireDebirandoSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::SpriteMainProgress::TrinexxDeathExplosionSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::SpriteMainProgress::AgahnimMotionBlurSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
        crate::SpriteMainProgress::TrinexxFinalPhaseTileCollision { slot, .. } => slot < 16,
        crate::SpriteMainProgress::HelmasaurHardHatTileCollision { slot, .. } => slot < 16,
        crate::SpriteMainProgress::LanmolaDrawPrefix {
            slot,
            completed_stores,
        } => slot < 16 && completed_stores <= 5,
        crate::SpriteMainProgress::TrinexxFinalPhaseDraw {
            slot,
            segment,
            stage,
        } => valid_trinexx_final_phase_draw_stage(segment, stage) && slot < 16,
        crate::SpriteMainProgress::FollowerGraphics { slot, stage, .. } => {
            slot < 16
                && match stage {
                    crate::RescuedMaidenInitializationStage::FirstFollowerSheet {
                        completed_bytes,
                    }
                    | crate::RescuedMaidenInitializationStage::SecondFollowerSheet {
                        completed_bytes,
                    } => completed_bytes <= 0x0600,
                    crate::RescuedMaidenInitializationStage::Conversion { completed_stores } => {
                        completed_stores <= 512
                    }
                }
        }
        crate::SpriteMainProgress::AfterWallmasterResetPrefix(slot) => slot < 16,
        crate::SpriteMainProgress::WallmasterResetClear {
            slot,
            cleared_bytes,
        } => slot < 16 && cleared_bytes <= 0x1000,
        crate::SpriteMainProgress::AfterActiveCuccoX { slot, .. }
        | crate::SpriteMainProgress::AfterActiveCuccoYSubpixel { slot, .. }
        | crate::SpriteMainProgress::AfterCuccoFleeMovement { slot, .. }
        | crate::SpriteMainProgress::AfterCuccoGraphicsPublication { slot, .. } => slot < 16,
        crate::SpriteMainProgress::AfterCuccoSubtypeIncrements {
            slot, completed, ..
        } => slot < 16 && completed >= 1 && completed <= 5,
        crate::SpriteMainProgress::MasterSwordLightBeamMovement {
            slot,
            checkpoint: _,
        } => slot < 16,
        crate::SpriteMainProgress::BoulderMovement { slot, .. } => slot < 16,
        crate::SpriteMainProgress::InitializePrepMoveY { slot, .. } => slot < 16,
        crate::SpriteMainProgress::MasterSwordLightBeamSpawn {
            slot,
            spawned_slot,
            progress,
        } => slot < 16 && spawned_slot < 16 && valid_dynamic_spawn_progress(progress),
    }
}

fn sprite_main_cpu_boundary_from_progress(
    progress: crate::SpriteMainProgress,
) -> SpriteMainCpuBoundary {
    match progress {
        crate::SpriteMainProgress::BeforeFirstSlot => SpriteMainCpuBoundary::BeforeFirstSlot,
        crate::SpriteMainProgress::AfterSlot(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterSlot(slot)
        }
        crate::SpriteMainProgress::AfterTimersAndOam(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main timer/OAM progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None }
        }
        crate::SpriteMainProgress::AfterTimerDecrements(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main timer decrement progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterTimerDecrements { slot, state: None }
        }
        crate::SpriteMainProgress::AfterPrimaryTimerDecrements(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main primary timer decrement progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot, state: None }
        }
        crate::SpriteMainProgress::AfterHitTimer(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main hit timer decrement progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterHitTimer { slot, state: None }
        }
        crate::SpriteMainProgress::AfterMainAndAux1TimerDecrements(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main main/aux1 timer decrement progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { slot, state: None }
        }
        crate::SpriteMainProgress::AfterMainTimerDecrement(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main main timer decrement progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterMainTimerDecrement { slot, state: None }
        }
        crate::SpriteMainProgress::AfterZeroHitTimerClear(slot) => {
            assert!(
                slot < 16,
                "source Sprite_Main main timer decrement progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterZeroHitTimerClear { slot, state: None }
        }
        crate::SpriteMainProgress::BonkItemGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source bonk-item graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::BonkItemGraphicsEntered(slot)
        }
        crate::SpriteMainProgress::BariBeforeRandom(slot) => {
            assert!(
                slot < 16,
                "source Bari prep progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::BariBeforeRandom(slot)
        }
        crate::SpriteMainProgress::FollowerGraphics {
            slot,
            caller,
            stage,
        } => {
            assert!(
                slot < 16,
                "source Zelda follower-graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller,
                prefix_completed: false,
                saved_follower_indicator: None,
                stage,
            }
        }
        crate::SpriteMainProgress::AfterThrowableSceneryStateClear(slot) => {
            assert!(
                slot < 16,
                "source throwable-scenery progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(slot)
        }
        crate::SpriteMainProgress::AfterActiveCuccoX {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco X progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterActiveCuccoX {
                slot,
                helper_ordinal,
            }
        }
        crate::SpriteMainProgress::AfterActiveCuccoYSubpixel {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco movement progress used invalid slot {slot}",
            );
            SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
                y_low: None,
                y_high: None,
            }
        }
        crate::SpriteMainProgress::MasterSwordLightBeamMovement { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source master-sword movement used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                slot,
                checkpoint,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::BoulderMovement { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source Boulder movement used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::BoulderMovement {
                slot,
                checkpoint,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::InitializePrepMoveY { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source state-8 prep Sprite_MoveY used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::InitializePrepMoveY {
                slot,
                checkpoint,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::MasterSwordLightBeamSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
                slot,
                spawned_slot,
                progress,
            }
        }
        crate::SpriteMainProgress::AfterCuccoFleeMovement {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco flee progress used invalid slot {slot}",
            );
            SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            }
        }
        crate::SpriteMainProgress::AfterCuccoSubtypeIncrements {
            slot,
            helper_ordinal,
            completed,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco subtype progress used invalid slot {slot}",
            );
            assert!((1..=5).contains(&completed));
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                slot,
                helper_ordinal,
                completed,
                total: 0,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::AfterCuccoGraphicsPublication {
            slot,
            helper_ordinal,
        } => {
            assert!(
                slot < 16,
                "source Sprite_Main Cucco progress used invalid slot {slot}",
            );
            SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::BigKeyDropGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source big-key graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(slot)
        }
        crate::SpriteMainProgress::KingZoraFlippersGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source King Zora flippers graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(slot)
        }
        crate::SpriteMainProgress::HappinessPondRupeeGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Happiness Pond rupee graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(slot)
        }
        crate::SpriteMainProgress::CatfishMedallionGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Catfish medallion graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(slot)
        }
        crate::SpriteMainProgress::TrinexxHeadDrawSetup(slot) => {
            assert!(
                slot < 16,
                "source Trinexx medallion graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::TrinexxHeadDrawSetup(slot)
        }
        crate::SpriteMainProgress::WaterfallGtCutsceneGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Waterfall GT cutscene graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(slot)
        }
        crate::SpriteMainProgress::TrinexxBreathTileCollisionReturned(slot) => {
            assert!(
                slot < 16,
                "source Trinexx breath tile-collision progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(slot)
        }
        crate::SpriteMainProgress::LaserEyeDrawPrologue(slot) => {
            assert!(
                slot < 16,
                "source Trinexx breath tile-collision progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::LaserEyeDrawPrologue(slot)
        }
        crate::SpriteMainProgress::ZoraFireballMovement { slot, checkpoint } => {
            assert!(
                slot < 16,
                "source Zora fireball movement used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::ZoraFireballMovement {
                slot,
                checkpoint,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::WishPondTossedItemGraphicsStarted(slot) => {
            assert!(
                slot < 16,
                "source Wish Pond graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::WishPondTossedItemGraphics {
                slot,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::AfterSingleSmallDrawPosition(slot) => {
            assert!(
                slot < 16,
                "source single-small draw progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                slot,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::WallmasterResetClear {
            slot,
            cleared_bytes,
        } => {
            assert!(slot < 16 && cleared_bytes <= 0x1000);
            SpriteMainCpuBoundary::WallmasterResetClear {
                slot,
                cleared_bytes,
            }
        }
        crate::SpriteMainProgress::AfterWallmasterResetPrefix(slot) => {
            assert!(
                slot < 16,
                "source Wallmaster reset progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterWallmasterResetPrefix(slot)
        }
        crate::SpriteMainProgress::ZazakAfterGraphics(slot) => {
            assert!(
                slot < 16,
                "source Zazak graphics progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::ZazakAfterGraphics(slot)
        }
        crate::SpriteMainProgress::ProbeAfterOamCoordinates(slot) => {
            assert!(
                slot < 16,
                "source guard-probe progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                slot,
                oam_position: None,
            }
        }
        crate::SpriteMainProgress::InitializeResetProperties {
            slot,
            phase,
            completed_stores,
        } => {
            assert!(
                slot < 16,
                "source sprite-init reset progress used invalid slot {slot}"
            );
            assert!(
                completed_stores <= 40,
                "source sprite-init reset progress exceeded 40 stores: {completed_stores}",
            );
            SpriteMainCpuBoundary::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            }
        }
        crate::SpriteMainProgress::InitializeLoadProperties {
            slot,
            phase,
            completed_stores,
        } => {
            assert!(
                slot < 16,
                "source property-load progress used invalid slot {slot}"
            );
            assert!(completed_stores <= 10);
            SpriteMainCpuBoundary::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            }
        }
        crate::SpriteMainProgress::FireDebirandoBeforeSpawn(slot) => {
            assert!(
                slot < 16,
                "source Fire Debirando progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(slot)
        }
        crate::SpriteMainProgress::FireDebirandoSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            }
        }
        crate::SpriteMainProgress::TrinexxDeathExplosionSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
                slot,
                spawned_slot,
                progress,
            }
        }
        crate::SpriteMainProgress::AgahnimMotionBlurSpawn {
            slot,
            spawned_slot,
            progress,
        } => {
            assert!(slot < 16 && spawned_slot < 16);
            assert!(valid_dynamic_spawn_progress(progress));
            SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
                slot,
                spawned_slot,
                progress,
                bound: false,
            }
        }
        crate::SpriteMainProgress::TrinexxFinalPhaseTileCollision {
            slot,
            probes_completed,
        } => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
                slot,
                probes_completed,
            }
        }
        crate::SpriteMainProgress::HelmasaurHardHatTileCollision { slot, stage } => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::HelmasaurHardHatTileCollision { slot, stage }
        }
        crate::SpriteMainProgress::LanmolaDrawPrefix {
            slot,
            completed_stores,
        } => {
            assert!(slot < 16 && completed_stores <= 5);
            SpriteMainCpuBoundary::LanmolaDrawPrefix {
                slot,
                completed_stores,
            }
        }
        crate::SpriteMainProgress::TrinexxFinalPhaseDraw {
            slot,
            segment,
            stage,
        } => {
            assert!(valid_trinexx_final_phase_draw_stage(segment, stage));
            SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
                slot,
                segment,
                stage,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::AfterAntfairySubtype2Increment(slot) => {
            assert!(
                slot < 16,
                "source Antfairy subtype2 progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                slot,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::AfterLanmolaSubtype2Increment(slot) => {
            assert!(
                slot < 16,
                "source Lanmola subtype2 progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                slot,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::AfterHelmasaurHardHatBeetleSubtype2Increment(slot) => {
            assert!(
                slot < 16,
                "source Helmasaur/Hardhat subtype2 progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment { slot }
        }
        crate::SpriteMainProgress::GuardPrepPatrolDelay { slot, active_call } => {
            assert!(slot < 16 && (1..=2).contains(&active_call));
            SpriteMainCpuBoundary::GuardPrepPatrolDelay {
                slot,
                active_call,
                saved_submodule: None,
            }
        }
        crate::SpriteMainProgress::GuardPrepTileCollisionReturned { slot, active_call } => {
            assert!(slot < 16 && (1..=2).contains(&active_call));
            SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
                slot,
                active_call,
                saved_submodule: None,
            }
        }
        crate::SpriteMainProgress::InitializePrepPending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::InitializePrepPending { slot }
        }
        crate::SpriteMainProgress::HogSpearBodyGraphicsPending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot }
        }
        crate::SpriteMainProgress::BuzzblobAfterXSubpixel(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
                slot,
                pending: None,
            }
        }
        crate::SpriteMainProgress::AbsorbableHorizontalTileLookup(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot }
        }
        crate::SpriteMainProgress::AbsorbableVerticalTileLookup(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot }
        }
        crate::SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot }
        }
        crate::SpriteMainProgress::SwamolaHeadDraw(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::SwamolaHeadDraw { slot }
        }
        crate::SpriteMainProgress::SwamolaHeadDrawCompleted(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot }
        }
        crate::SpriteMainProgress::MoblinAttributeLoaded(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::MoblinAttributeLoaded { slot }
        }
        crate::SpriteMainProgress::MoblinCollisionGeometry(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::MoblinCollisionGeometry { slot }
        }
        crate::SpriteMainProgress::VitreousDamagePending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::VitreousDamagePending { slot }
        }
        crate::SpriteMainProgress::VitreousAiPending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::VitreousAiPending { slot }
        }
        crate::SpriteMainProgress::MiniMoldormAiPending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::MiniMoldormAiPending { slot }
        }
        crate::SpriteMainProgress::VitreousPlayerDamagePending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot }
        }
        crate::SpriteMainProgress::SwamolaSegmentDraw { slot, segment } => {
            assert!(slot < 16 && segment < 4);
            SpriteMainCpuBoundary::SwamolaSegmentDraw { slot, segment }
        }
        crate::SpriteMainProgress::TrinexxHeadDraw { slot, segment } => {
            assert!(slot < 16 && segment < 9);
            SpriteMainCpuBoundary::TrinexxHeadDraw {
                slot,
                segment,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::SidenexxNeckTargetLoop { slot, step } => {
            assert!(slot < 16 && step < 56);
            SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
                slot,
                step,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::TrinexxHeadFrontPart {
            slot,
            completed_stores,
        } => {
            assert!(slot < 16 && completed_stores <= 30);
            SpriteMainCpuBoundary::TrinexxHeadFrontPart {
                slot,
                completed_stores,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::PengatorSlidePending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::PengatorSlidePending { slot }
        }
        crate::SpriteMainProgress::AntifairyBouncePending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::AntifairyBouncePending { slot }
        }
        crate::SpriteMainProgress::KholdstareDamagePending(slot) => {
            assert!(slot < 16);
            SpriteMainCpuBoundary::KholdstareDamagePending { slot }
        }
        crate::SpriteMainProgress::GuardPrepWeaponFlagsPending(slot) => {
            assert!(
                slot < 16,
                "source guard-prep weapon progress used invalid slot {slot}"
            );
            SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                slot,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::GuardPrepParryHitbox { slot, active_call } => {
            assert!(slot < 16 && (1..=2).contains(&active_call));
            SpriteMainCpuBoundary::GuardPrepParryHitbox {
                slot,
                active_call,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::GuardAnimation { slot, checkpoint } => {
            assert!(slot < 16 && checkpoint.is_valid());
            SpriteMainCpuBoundary::GuardAnimation {
                slot,
                checkpoint,
                continuation: None,
            }
        }
        crate::SpriteMainProgress::MiniMoldormHistory {
            slot,
            completed_stores,
        } => {
            assert!(slot < 16);
            assert!(completed_stores <= 128);
            SpriteMainCpuBoundary::MiniMoldormHistory {
                slot,
                completed_stores,
            }
        }
    }
}

const fn module_cpu_phase_from_main_loop_interruption(
    interruption: crate::MainLoopInterruption,
) -> Option<ModuleCpuPhase> {
    if interruption.is_sprite_main() {
        return Some(ModuleCpuPhase::InterruptedInSpriteMain);
    }
    match interruption {
        crate::MainLoopInterruption::LinkOam => Some(ModuleCpuPhase::InterruptedInLinkOam),
        crate::MainLoopInterruption::SpritePreparation
        | crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. } => {
            Some(ModuleCpuPhase::InterruptedInNmiPrepareSprites)
        }
        crate::MainLoopInterruption::SpriteMainBeforeFirstSlot
        | crate::MainLoopInterruption::SpriteMainAfterSlot(_)
        | crate::MainLoopInterruption::SpriteMainAfterTimersAndOam(_)
        | crate::MainLoopInterruption::SpriteMainAfterTimerDecrements(_)
        | crate::MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(_)
        | crate::MainLoopInterruption::SpriteMainAfterHitTimer(_)
        | crate::MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(_)
        | crate::MainLoopInterruption::SpriteMainAfterMainTimerDecrement(_)
        | crate::MainLoopInterruption::SpriteMainAfterZeroHitTimerClear(_)
        | crate::MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainBariBeforeRandom(_)
        | crate::MainLoopInterruption::SpriteMainFollowerGraphics { .. }
        | crate::MainLoopInterruption::SpriteMainAfterThrowableSceneryStateClear(_)
        | crate::MainLoopInterruption::SpriteMainAfterActiveCuccoX { .. }
        | crate::MainLoopInterruption::SpriteMainAfterActiveCuccoYSubpixel { .. }
        | crate::MainLoopInterruption::SpriteMainAfterCuccoFleeMovement { .. }
        | crate::MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements { .. }
        | crate::MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication { .. }
        | crate::MainLoopInterruption::SpriteMainMasterSwordLightBeamMovement { .. }
        | crate::MainLoopInterruption::SpriteMainBoulderMovement { .. }
        | crate::MainLoopInterruption::SpriteMainInitializePrepMoveY { .. }
        | crate::MainLoopInterruption::SpriteMainMasterSwordLightBeamSpawn { .. }
        | crate::MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainKingZoraFlippersGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainHappinessPondRupeeGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainCatfishMedallionGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainTrinexxHeadDrawSetup(_)
        | crate::MainLoopInterruption::SpriteMainWaterfallGtCutsceneGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainTrinexxBreathTileCollisionReturned(_)
        | crate::MainLoopInterruption::SpriteMainLaserEyeDrawPrologue(_)
        | crate::MainLoopInterruption::SpriteMainZoraFireballMovement { .. }
        | crate::MainLoopInterruption::SpriteMainAfterSingleSmallDrawPosition(_)
        | crate::MainLoopInterruption::SpriteMainAfterWallmasterResetPrefix(_)
        | crate::MainLoopInterruption::SpriteMainWallmasterResetClear { .. }
        | crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainWishPondTossedItemGraphicsStarted(_)
        | crate::MainLoopInterruption::SpriteMainZazakAfterGraphics(_)
        | crate::MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(_)
        | crate::MainLoopInterruption::SpriteMainInitializeResetProperties { .. }
        | crate::MainLoopInterruption::SpriteMainInitializeLoadProperties { .. }
        | crate::MainLoopInterruption::SpriteMainFireDebirandoBeforeSpawn(_)
        | crate::MainLoopInterruption::SpriteMainFireDebirandoSpawn { .. }
        | crate::MainLoopInterruption::SpriteMainTrinexxDeathExplosionSpawn { .. }
        | crate::MainLoopInterruption::SpriteMainAgahnimMotionBlurSpawn { .. }
        | crate::MainLoopInterruption::SpriteMainTrinexxFinalPhaseTileCollision { .. }
        | crate::MainLoopInterruption::SpriteMainHelmasaurHardHatTileCollision { .. }
        | crate::MainLoopInterruption::SpriteMainLanmolaDrawPrefix { .. }
        | crate::MainLoopInterruption::SpriteMainTrinexxFinalPhaseDraw { .. }
        | crate::MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(_)
        | crate::MainLoopInterruption::SpriteMainAfterLanmolaSubtype2Increment(_)
        | crate::MainLoopInterruption::SpriteMainAfterHelmasaurHardHatBeetleSubtype2Increment(_)
        | crate::MainLoopInterruption::SpriteMainBuzzblobAfterXSubpixel(_)
        | crate::MainLoopInterruption::SpriteMainHogSpearBodyGraphicsPending(_)
        | crate::MainLoopInterruption::SpriteMainAbsorbableHorizontalTileLookup(_)
        | crate::MainLoopInterruption::SpriteMainAbsorbableVerticalTileLookup(_)
        | crate::MainLoopInterruption::SpriteMainAbsorbableVerticalTileAttributeLoaded(_)
        | crate::MainLoopInterruption::SpriteMainSwamolaHeadDraw(_)
        | crate::MainLoopInterruption::SpriteMainSwamolaHeadDrawCompleted(_)
        | crate::MainLoopInterruption::SpriteMainMoblinAttributeLoaded(_)
        | crate::MainLoopInterruption::SpriteMainMoblinCollisionGeometry(_)
        | crate::MainLoopInterruption::SpriteMainVitreousDamagePending(_)
        | crate::MainLoopInterruption::SpriteMainVitreousAiPending(_)
        | crate::MainLoopInterruption::SpriteMainMiniMoldormAiPending(_)
        | crate::MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(_)
        | crate::MainLoopInterruption::SpriteMainSwamolaSegmentDraw { .. }
        | crate::MainLoopInterruption::SpriteMainTrinexxHeadDraw { .. }
        | crate::MainLoopInterruption::SpriteMainSidenexxNeckTargetLoop { .. }
        | crate::MainLoopInterruption::SpriteMainTrinexxHeadFrontPart { .. }
        | crate::MainLoopInterruption::SpriteMainPengatorSlidePending(_)
        | crate::MainLoopInterruption::SpriteMainAntifairyBouncePending(_)
        | crate::MainLoopInterruption::SpriteMainKholdstareDamagePending(_)
        | crate::MainLoopInterruption::SpriteMainInitializePrepPending(_)
        | crate::MainLoopInterruption::SpriteMainGuardPrepWeaponFlagsPending(_)
        | crate::MainLoopInterruption::SpriteMainGuardPrepParryHitbox { .. }
        | crate::MainLoopInterruption::SpriteMainGuardPrepPatrolDelay { .. }
        | crate::MainLoopInterruption::SpriteMainGuardPrepTileCollisionReturned { .. }
        | crate::MainLoopInterruption::SpriteMainGuardAnimation { .. }
        | crate::MainLoopInterruption::SpriteMainMiniMoldormHistory { .. } => {
            unreachable!()
        }
        crate::MainLoopInterruption::DungeonExitSpotlightAfterSubmodule
        | crate::MainLoopInterruption::LinkActualVelocity { .. }
        | crate::MainLoopInterruption::LinkActualVelocityCompleted
        | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
        | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
        | crate::MainLoopInterruption::LinkPositionBeforeCoordinates
        | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
        | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
        | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
        | crate::MainLoopInterruption::SpotlightGoalResetTable { .. }
        | crate::MainLoopInterruption::GameOverIrisGoalPaletteFill { .. }
        | crate::MainLoopInterruption::DesertPrayerIris { .. }
        | crate::MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor { .. } => None,
    }
}

/// Compare the source statement named by two Sprite_Main checkpoints without
/// treating native-only resume data as part of the wire identity. Source
/// receipts deliberately omit saved dispatch state, reconstructed movement,
/// and typed C continuations; those values are bound only after the native
/// body reaches the named statement.
fn same_sprite_main_source_checkpoint(
    left: SpriteMainCpuBoundary,
    right: SpriteMainCpuBoundary,
) -> bool {
    match (left, right) {
        (
            SpriteMainCpuBoundary::AfterTimersAndOam { slot: left, .. },
            SpriteMainCpuBoundary::AfterTimersAndOam { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterTimerDecrements { slot: left, .. },
            SpriteMainCpuBoundary::AfterTimerDecrements { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot: left, .. },
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterHitTimer { slot: left, .. },
            SpriteMainCpuBoundary::AfterHitTimer { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { slot: left, .. },
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterMainTimerDecrement { slot: left, .. },
            SpriteMainCpuBoundary::AfterMainTimerDecrement { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterZeroHitTimerClear { slot: left, .. },
            SpriteMainCpuBoundary::AfterZeroHitTimerClear { slot: right, .. },
        ) => left == right,
        (
            SpriteMainCpuBoundary::FollowerGraphics {
                slot: left,
                caller: left_caller,
                stage: left_stage,
                ..
            },
            SpriteMainCpuBoundary::FollowerGraphics {
                slot: right,
                caller: right_caller,
                stage: right_stage,
                ..
            },
        ) => left == right && left_caller == right_caller && left_stage == right_stage,
        (
            SpriteMainCpuBoundary::TrinexxHeadDraw {
                slot: left,
                segment: left_segment,
                ..
            },
            SpriteMainCpuBoundary::TrinexxHeadDraw {
                slot: right,
                segment: right_segment,
                ..
            },
        ) => left == right && left_segment == right_segment,
        (
            SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
                slot: left,
                step: left_step,
                ..
            },
            SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
                slot: right,
                step: right_step,
                ..
            },
        ) => left == right && left_step == right_step,
        (
            SpriteMainCpuBoundary::TrinexxHeadFrontPart {
                slot: left,
                completed_stores: a,
                ..
            },
            SpriteMainCpuBoundary::TrinexxHeadFrontPart {
                slot: right,
                completed_stores: b,
                ..
            },
        ) => left == right && a == b,
        (
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition { slot: left, .. },
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition { slot: right, .. },
        ) => left == right,
        (
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates { slot: left, .. },
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates { slot: right, .. },
        ) => left == right,
        (
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment { slot: left, .. },
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment { slot: left, .. },
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment { slot: left },
            SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment { slot: right },
        )
        | (
            SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot: left },
            SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot: right },
        )
        | (
            SpriteMainCpuBoundary::BuzzblobAfterXSubpixel { slot: left, .. },
            SpriteMainCpuBoundary::BuzzblobAfterXSubpixel { slot: right, .. },
        )
        | (
            SpriteMainCpuBoundary::InitializePrepPending { slot: left },
            SpriteMainCpuBoundary::InitializePrepPending { slot: right },
        ) => left == right,
        (
            SpriteMainCpuBoundary::InitializeResetProperties {
                slot: left_slot,
                phase: left_phase,
                completed_stores: left_stores,
            },
            SpriteMainCpuBoundary::InitializeResetProperties {
                slot: right_slot,
                phase: right_phase,
                completed_stores: right_stores,
            },
        ) => left_slot == right_slot && left_phase == right_phase && left_stores == right_stores,
        (
            SpriteMainCpuBoundary::InitializeLoadProperties {
                slot: left_slot,
                phase: left_phase,
                completed_stores: left_stores,
            },
            SpriteMainCpuBoundary::InitializeLoadProperties {
                slot: right_slot,
                phase: right_phase,
                completed_stores: right_stores,
            },
        ) => left_slot == right_slot && left_phase == right_phase && left_stores == right_stores,
        (
            SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(left),
            SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(right),
        ) => left == right,
        (
            SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
                slot: left,
                segment: left_segment,
                stage: left_stage,
                ..
            },
            SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
                slot: right,
                segment: right_segment,
                stage: right_stage,
                ..
            },
        ) => left == right && left_segment == right_segment && left_stage == right_stage,
        (
            SpriteMainCpuBoundary::LanmolaDrawPrefix {
                slot: left,
                completed_stores: left_stores,
            },
            SpriteMainCpuBoundary::LanmolaDrawPrefix {
                slot: right,
                completed_stores: right_stores,
            },
        ) => left == right && left_stores == right_stores,
        (
            SpriteMainCpuBoundary::HelmasaurHardHatTileCollision {
                slot: left,
                stage: left_stage,
            },
            SpriteMainCpuBoundary::HelmasaurHardHatTileCollision {
                slot: right,
                stage: right_stage,
            },
        ) => left == right && left_stage == right_stage,
        (
            SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
                slot: left,
                probes_completed: left_probes,
            },
            SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
                slot: right,
                probes_completed: right_probes,
            },
        ) => left == right && left_probes == right_probes,
        (
            SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot: left_slot,
                spawned_slot: left_spawned,
                progress: left_progress,
            },
            SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot: right_slot,
                spawned_slot: right_spawned,
                progress: right_progress,
            },
        )
        | (
            SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
                slot: left_slot,
                spawned_slot: left_spawned,
                progress: left_progress,
            },
            SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
                slot: right_slot,
                spawned_slot: right_spawned,
                progress: right_progress,
            },
        ) => {
            left_slot == right_slot
                && left_spawned == right_spawned
                && left_progress == right_progress
        }
        (
            SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
                slot: left_slot,
                spawned_slot: left_spawned,
                progress: left_progress,
                ..
            },
            SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
                slot: right_slot,
                spawned_slot: right_spawned,
                progress: right_progress,
                ..
            },
        ) => {
            left_slot == right_slot
                && left_spawned == right_spawned
                && left_progress == right_progress
        }
        (
            SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                slot: left_slot,
                helper_ordinal: left_helper,
                ..
            },
            SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                slot: right_slot,
                helper_ordinal: right_helper,
                ..
            },
        )
        | (
            SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot: left_slot,
                helper_ordinal: left_helper,
                ..
            },
            SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot: right_slot,
                helper_ordinal: right_helper,
                ..
            },
        ) => left_slot == right_slot && left_helper == right_helper,
        (
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                slot: left_slot,
                helper_ordinal: left_helper,
                completed: left_completed,
                ..
            },
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                slot: right_slot,
                helper_ordinal: right_helper,
                completed: right_completed,
                ..
            },
        ) => {
            left_slot == right_slot
                && left_helper == right_helper
                && left_completed == right_completed
        }
        (
            SpriteMainCpuBoundary::WishPondTossedItemGraphics {
                slot: left_slot, ..
            },
            SpriteMainCpuBoundary::WishPondTossedItemGraphics {
                slot: right_slot, ..
            },
        ) => left_slot == right_slot,
        (
            SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                slot: left_slot, ..
            },
            SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                slot: right_slot, ..
            },
        ) => left_slot == right_slot,
        (
            SpriteMainCpuBoundary::GuardPrepParryHitbox {
                slot: left_slot,
                active_call: left_call,
                ..
            },
            SpriteMainCpuBoundary::GuardPrepParryHitbox {
                slot: right_slot,
                active_call: right_call,
                ..
            },
        )
        | (
            SpriteMainCpuBoundary::GuardPrepPatrolDelay {
                slot: left_slot,
                active_call: left_call,
                ..
            },
            SpriteMainCpuBoundary::GuardPrepPatrolDelay {
                slot: right_slot,
                active_call: right_call,
                ..
            },
        )
        | (
            SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
                slot: left_slot,
                active_call: left_call,
                ..
            },
            SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
                slot: right_slot,
                active_call: right_call,
                ..
            },
        ) => left_slot == right_slot && left_call == right_call,
        (
            SpriteMainCpuBoundary::GuardAnimation {
                slot: left_slot,
                checkpoint: left_entry,
                ..
            },
            SpriteMainCpuBoundary::GuardAnimation {
                slot: right_slot,
                checkpoint: right_entry,
                ..
            },
        ) => left_slot == right_slot && left_entry == right_entry,
        (
            SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                slot: left_slot,
                checkpoint: left_checkpoint,
                ..
            },
            SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                slot: right_slot,
                checkpoint: right_checkpoint,
                ..
            },
        ) => left_slot == right_slot && left_checkpoint == right_checkpoint,
        (
            SpriteMainCpuBoundary::BoulderMovement {
                slot: left_slot,
                checkpoint: left_checkpoint,
                ..
            },
            SpriteMainCpuBoundary::BoulderMovement {
                slot: right_slot,
                checkpoint: right_checkpoint,
                ..
            },
        ) => left_slot == right_slot && left_checkpoint == right_checkpoint,
        (
            SpriteMainCpuBoundary::InitializePrepMoveY {
                slot: left_slot,
                checkpoint: left_checkpoint,
                ..
            },
            SpriteMainCpuBoundary::InitializePrepMoveY {
                slot: right_slot,
                checkpoint: right_checkpoint,
                ..
            },
        ) => left_slot == right_slot && left_checkpoint == right_checkpoint,
        (
            SpriteMainCpuBoundary::ZoraFireballMovement {
                slot: left_slot,
                checkpoint: left_checkpoint,
                ..
            },
            SpriteMainCpuBoundary::ZoraFireballMovement {
                slot: right_slot,
                checkpoint: right_checkpoint,
                ..
            },
        ) => left_slot == right_slot && left_checkpoint == right_checkpoint,
        (
            SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
                slot: left_slot,
                spawned_slot: left_spawned,
                progress: left_progress,
            },
            SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
                slot: right_slot,
                spawned_slot: right_spawned,
                progress: right_progress,
            },
        ) => {
            left_slot == right_slot
                && left_spawned == right_spawned
                && left_progress == right_progress
        }
        _ => left == right,
    }
}

fn same_optional_sprite_main_source_checkpoint(
    left: Option<SpriteMainCpuBoundary>,
    right: Option<SpriteMainCpuBoundary>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => same_sprite_main_source_checkpoint(left, right),
        (None, None) => true,
        _ => false,
    }
}

/// Monotonic source order through `Sprite_Main`'s descending slot loop.
/// Partial-slot checkpoints sit between the preceding returned slot and the
/// current slot's return. Different partial checkpoints at the same rank are
/// intentionally not interchangeable.
const fn sprite_main_cpu_boundary_order(boundary: SpriteMainCpuBoundary) -> u8 {
    match boundary {
        SpriteMainCpuBoundary::BeforeFirstSlot
        | SpriteMainCpuBoundary::Module09FinalScrollPairPending => 0,
        SpriteMainCpuBoundary::AfterSlot(slot) => 2 * (16 - slot),
        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication { slot, .. }
        | SpriteMainCpuBoundary::AfterTimersAndOam { slot, .. }
        | SpriteMainCpuBoundary::AfterTimerDecrements { slot, .. }
        | SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot, .. }
        | SpriteMainCpuBoundary::AfterHitTimer { slot, .. }
        | SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { slot, .. }
        | SpriteMainCpuBoundary::AfterMainTimerDecrement { slot, .. }
        | SpriteMainCpuBoundary::AfterZeroHitTimerClear { slot, .. }
        | SpriteMainCpuBoundary::BariBeforeRandom(slot)
        | SpriteMainCpuBoundary::FollowerGraphics { slot, .. }
        | SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(slot)
        | SpriteMainCpuBoundary::AfterActiveCuccoX { slot, .. }
        | SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel { slot, .. }
        | SpriteMainCpuBoundary::MasterSwordLightBeamMovement { slot, .. }
        | SpriteMainCpuBoundary::BoulderMovement { slot, .. }
        | SpriteMainCpuBoundary::InitializePrepMoveY { slot, .. }
        | SpriteMainCpuBoundary::MasterSwordLightBeamSpawn { slot, .. }
        | SpriteMainCpuBoundary::AfterCuccoFleeMovement { slot, .. }
        | SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements { slot, .. }
        | SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(slot)
        | SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(slot)
        | SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(slot)
        | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(slot)
        | SpriteMainCpuBoundary::TrinexxHeadDrawSetup(slot)
        | SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(slot)
        | SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(slot)
        | SpriteMainCpuBoundary::LaserEyeDrawPrologue(slot)
        | SpriteMainCpuBoundary::ZoraFireballMovement { slot, .. }
        | SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(slot)
        | SpriteMainCpuBoundary::WishPondTossedItemGraphics { slot, .. }
        | SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(slot)
        | SpriteMainCpuBoundary::AfterZeldaFollowerGraphics { slot, .. }
        | SpriteMainCpuBoundary::BonkItemGraphicsEntered(slot)
        | SpriteMainCpuBoundary::AfterSingleSmallDrawPosition { slot, .. }
        | SpriteMainCpuBoundary::ZazakAfterGraphics(slot)
        | SpriteMainCpuBoundary::ProbeAfterOamCoordinates { slot, .. }
        | SpriteMainCpuBoundary::InitializeResetProperties { slot, .. }
        | SpriteMainCpuBoundary::InitializeLoadProperties { slot, .. }
        | SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(slot)
        | SpriteMainCpuBoundary::FireDebirandoSpawn { slot, .. }
        | SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn { slot, .. }
        | SpriteMainCpuBoundary::AgahnimMotionBlurSpawn { slot, .. }
        | SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision { slot, .. }
        | SpriteMainCpuBoundary::HelmasaurHardHatTileCollision { slot, .. }
        | SpriteMainCpuBoundary::LanmolaDrawPrefix { slot, .. }
        | SpriteMainCpuBoundary::TrinexxFinalPhaseDraw { slot, .. }
        | SpriteMainCpuBoundary::AfterAntfairySubtype2Increment { slot, .. }
        | SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment { slot, .. }
        | SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment { slot }
        | SpriteMainCpuBoundary::BuzzblobAfterXSubpixel { slot, .. }
        | SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot }
        | SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot }
        | SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot }
        | SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot }
        | SpriteMainCpuBoundary::SwamolaHeadDraw { slot }
        | SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot }
        | SpriteMainCpuBoundary::MoblinAttributeLoaded { slot }
        | SpriteMainCpuBoundary::MoblinCollisionGeometry { slot }
        | SpriteMainCpuBoundary::VitreousDamagePending { slot }
        | SpriteMainCpuBoundary::VitreousAiPending { slot }
        | SpriteMainCpuBoundary::MiniMoldormAiPending { slot }
        | SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot }
        | SpriteMainCpuBoundary::SwamolaSegmentDraw { slot, .. }
        | SpriteMainCpuBoundary::TrinexxHeadDraw { slot, .. }
        | SpriteMainCpuBoundary::SidenexxNeckTargetLoop { slot, .. }
        | SpriteMainCpuBoundary::TrinexxHeadFrontPart { slot, .. }
        | SpriteMainCpuBoundary::PengatorSlidePending { slot }
        | SpriteMainCpuBoundary::AntifairyBouncePending { slot }
        | SpriteMainCpuBoundary::KholdstareDamagePending { slot }
        | SpriteMainCpuBoundary::InitializePrepPending { slot }
        | SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending { slot, .. }
        | SpriteMainCpuBoundary::GuardPrepParryHitbox { slot, .. }
        | SpriteMainCpuBoundary::GuardPrepPatrolDelay { slot, .. }
        | SpriteMainCpuBoundary::GuardPrepTileCollisionReturned { slot, .. }
        | SpriteMainCpuBoundary::GuardAnimation { slot, .. }
        | SpriteMainCpuBoundary::MiniMoldormHistory { slot, .. } => 2 * (16 - slot) - 1,
        SpriteMainCpuBoundary::AfterWallmasterResetPrefix(slot)
        | SpriteMainCpuBoundary::WallmasterResetClear { slot, .. } => 2 * (16 - slot) - 1,
    }
}

fn spotlight_reset_prefix_scanlines(
    rows_reset_before_hdma: &[Option<bool>; SPOTLIGHT_VISIBLE_SCANLINES],
    nmi_crossings_before_caller_return: u8,
) -> Option<usize> {
    // A second submodule NMI retires the reset field before the translated
    // caller returns. Its later publication owns the fully reset next field,
    // not the discarded in-flight race.
    if nmi_crossings_before_caller_return != 1
        || !rows_reset_before_hdma.iter().any(Option::is_some)
    {
        return None;
    }
    assert!(
        rows_reset_before_hdma.iter().all(Option::is_some),
        "ROM spotlight reset returned with an incomplete table-write trace",
    );
    let prefix = rows_reset_before_hdma
        .iter()
        .take_while(|reset_before_hdma| reset_before_hdma == &&Some(false))
        .count();
    assert!(
        rows_reset_before_hdma[prefix..]
            .iter()
            .all(|reset_before_hdma| reset_before_hdma == &Some(true)),
        "single-NMI ROM spotlight reset produced a non-contiguous HDMA generation",
    );
    Some(prefix)
}

fn dungeon_module_7_cpu_advance_at(
    state: &ZeldaState,
    checkpoint: RomCpuCheckpoint,
    entry: CpuRasterPosition,
) -> DungeonModuleCpuAdvance {
    dungeon_module_7_cpu_advance(state, checkpoint, Some(entry), false)
}

fn dungeon_module_7_cpu_advance_after_leading_nmi(
    state: &ZeldaState,
    checkpoint: RomCpuCheckpoint,
) -> DungeonModuleCpuAdvance {
    dungeon_module_7_cpu_advance(state, checkpoint, None, true)
}

fn dungeon_module_7_cpu_advance(
    state: &ZeldaState,
    checkpoint: RomCpuCheckpoint,
    dispatcher_entry: Option<CpuRasterPosition>,
    trace_host_checkpoint: bool,
) -> DungeonModuleCpuAdvance {
    dungeon_module_7_cpu_timing(state, checkpoint, dispatcher_entry, trace_host_checkpoint).advance
}

fn dungeon_module_7_cpu_timing(
    state: &ZeldaState,
    checkpoint: RomCpuCheckpoint,
    dispatcher_entry: Option<CpuRasterPosition>,
    trace_host_checkpoint: bool,
) -> DungeonModuleCpuTiming {
    let leading_nmi_checkpoint =
        if checkpoint.entry_pc == DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.entry_pc {
            DUNGEON_MAIN_WAIT_CPU_CHECKPOINT
        } else {
            checkpoint
        };
    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        leading_nmi_checkpoint,
    )
    .expect("Module 7 CPU timing requires the loaded Zelda ROM");
    run.enable_cpu_write_trace();
    // The measured dispatcher-entry envelope begins after the leading NMI.
    // The native host scheduler may fold that NMI's state mutation into its
    // trailing boundary, so `state.ram` can still be the pre-NMI generation
    // even though `entry` is a post-NMI raster position. Advance an isolated
    // ROM shadow through the real handler first; its mutations affect only
    // the timing run, while the translated engine remains the state owner.
    let field_timing = CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0);
    let mut leading_nmi_budget =
        CpuCycleBudget::at_nmi_acceptance(CpuBusWorkload::with_dynamic_hdma(), field_timing);
    advance_rom_cpu_through_nmi(&mut run, &mut leading_nmi_budget);
    while run.pc() != checkpoint.entry_pc {
        let (scanline, master_cycle) = leading_nmi_budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        assert_eq!(
            advance_rom_cpu_step(&mut run, &mut leading_nmi_budget),
            CpuWorkAdvance::Complete,
            "main wait-loop exit crossed the next vblank at {:06x}",
            run.pc(),
        );
    }
    if trace_host_checkpoint {
        debug_assert!(dispatcher_entry.is_none());
        trace_dungeon_cpu_checkpoint(state, &run, &leading_nmi_budget);
    }
    let mut budget = dispatcher_entry.map_or(leading_nmi_budget, |entry| {
        CpuCycleBudget::until_next_nmi_acceptance(
            entry,
            CpuBusWorkload::with_dynamic_hdma(),
            field_timing,
        )
    });
    const SUBMODULE_DISPATCH_PC: u32 = 0x02_87ac;
    // Some timing runs begin directly at the supertile submodule target.
    const SUBMODULE_ENTRY_PC: u32 = 0x02_8a26;
    const SPRITE_MAIN_ENTRY_PC: u32 = 0x06_8328;
    const SPRITE_MAIN_RETURN_PC: u32 = 0x02_8842;
    const SPRITE_EXECUTE_SINGLE_ENTRY_PC: u32 = 0x06_84e2;
    const SPRITE_SLOT_RETURN_PC: u32 = 0x06_83a7;
    const LINK_OAM_ENTRY_PC: u32 = 0x0d_a18e;
    const MODULE_7_RETURN_PC: u32 = 0x02_885a;
    const NMI_PREPARE_SPRITES_ENTRY_PC: u32 = 0x00_85fc;
    let mut submodule_started = checkpoint.entry_pc == SUBMODULE_ENTRY_PC;
    let mut submodule_entry_sp = submodule_started.then(|| run.stack_pointer());
    let mut submodule_returned = false;
    let mut sprite_main_started = false;
    let mut sprite_main_returned = false;
    let mut sprite_main_current_slot = None;
    let mut sprite_main_last_completed_slot = None;
    let mut link_oam_started = false;
    let mut link_oam_entry_sp = None;
    let mut link_oam_returned = false;
    let mut module_returned = false;
    let mut nmi_prepare_sprites_entry_sp = None;
    let mut nmi_prepare_sprites_returned = false;
    let mut first_interruption: Option<DungeonModuleCpuAdvance> = None;
    let mut cached_sprite_copy: Option<CachedSpriteCpuProgress> = None;
    let mut spotlight_reset_rows_before_hdma = [None; SPOTLIGHT_VISIBLE_SCANLINES];
    let debug_cpu_phases = crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_PHASES").is_some();
    let mut spotlight_instruction_steps = None;

    for _ in 0..200_000 {
        if run.is_complete() {
            if debug_cpu_phases {
                let (scanline, master_cycle) = budget.raster_position().coordinates();
                eprintln!(
                    "dungeon_cpu_phase host={} marker=complete pc={:06x} v={} cycles={} first_interruption={:?}",
                    state.frame_ctr_dbg,
                    run.pc(),
                    scanline,
                    master_cycle,
                    first_interruption.as_ref().map(|advance| advance.phase),
                );
            }
            let completed = DungeonModuleCpuAdvance {
                phase: ModuleCpuPhase::CompleteBeforeNmi,
                resumed_phase: None,
                submodule_nmi_slices: 0,
                subsubmodule: run.ram_byte(SUBSUBMODULE),
                palette_countdown: run.ram_byte(PALETTE_FILTER_COUNTDOWN),
                sprite_main_boundary: None,
                cached_sprite_interruption: None,
            };
            let timing = DungeonModuleCpuTiming {
                advance: completed,
                spotlight_reset_prefix_scanlines: None,
            };
            if let Some(mut first) = first_interruption {
                first.resumed_phase = Some(completed.phase);
                return DungeonModuleCpuTiming {
                    spotlight_reset_prefix_scanlines: spotlight_reset_prefix_scanlines(
                        &spotlight_reset_rows_before_hdma,
                        first.submodule_nmi_slices,
                    ),
                    advance: first,
                };
            }
            return timing;
        }
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        if run.pc() == 0x00_f312 {
            spotlight_instruction_steps = Some(0u32);
        }
        if let Some(steps) = spotlight_instruction_steps.as_mut() {
            *steps += 1;
        }
        if debug_cpu_phases
            && matches!(
                run.pc(),
                0x02_87a2
                    | 0x02_931d
                    | 0x02_932d
                    | 0x00_f295
                    | 0x00_f312
                    | 0x00_f4cc
                    | SUBMODULE_ENTRY_PC
                    | SPRITE_MAIN_ENTRY_PC
                    | SPRITE_MAIN_RETURN_PC
                    | LINK_OAM_ENTRY_PC
                    | MODULE_7_RETURN_PC
                    | NMI_PREPARE_SPRITES_ENTRY_PC
            )
        {
            eprintln!(
                "dungeon_cpu_phase host={} marker=pc pc={:06x} v={} cycles={} sp={:04x}",
                state.frame_ctr_dbg,
                run.pc(),
                scanline,
                master_cycle,
                run.stack_pointer(),
            );
        }
        if debug_cpu_phases && run.pc() == LINK_OAM_ENTRY_PC {
            eprintln!(
                "dungeon_cpu_phase host={} marker=spotlight-steps instructions={}",
                state.frame_ctr_dbg,
                spotlight_instruction_steps.unwrap_or(0),
            );
            spotlight_instruction_steps = None;
        }
        let executing_submodule_dispatch = run.pc() == SUBMODULE_DISPATCH_PC;
        let executing_submodule_entry = run.pc() == SUBMODULE_ENTRY_PC;
        if executing_submodule_entry && submodule_entry_sp.is_none() {
            submodule_entry_sp = Some(run.stack_pointer());
        }
        let executing_sprite_main_entry = run.pc() == SPRITE_MAIN_ENTRY_PC;
        let executing_sprite_main_return = run.pc() == SPRITE_MAIN_RETURN_PC;
        if sprite_main_started
            && !sprite_main_returned
            && run.pc() == SPRITE_EXECUTE_SINGLE_ENTRY_PC
        {
            let slot = run.ram_byte(CUR_OBJECT_INDEX);
            assert!(slot < 16, "Sprite_Main entered an invalid slot {slot}");
            sprite_main_current_slot = Some(slot);
        }
        if sprite_main_started && !sprite_main_returned && run.pc() == SPRITE_SLOT_RETURN_PC {
            sprite_main_last_completed_slot = sprite_main_current_slot;
        }
        let executing_link_oam_entry = run.pc() == LINK_OAM_ENTRY_PC;
        if sprite_main_returned && executing_link_oam_entry && link_oam_entry_sp.is_none() {
            link_oam_entry_sp = Some(run.stack_pointer());
        }
        let executing_nmi_prepare_sprites_entry = run.pc() == NMI_PREPARE_SPRITES_ENTRY_PC;
        let executing_module_7_return = run.pc() == MODULE_7_RETURN_PC;
        if run.pc() == 0x1d_ea00 {
            cached_sprite_copy = Some(CachedSpriteCpuProgress::at_entry(
                run.ram_byte(CUR_OBJECT_INDEX),
            ));
        }
        if let Some(copy) = cached_sprite_copy.as_mut() {
            copy.observe_pc(run.pc());
        }
        if executing_nmi_prepare_sprites_entry && nmi_prepare_sprites_entry_sp.is_none() {
            nmi_prepare_sprites_entry_sp = Some(run.stack_pointer());
        }
        let executing_pc = run.pc();
        let advance = advance_rom_cpu_step(&mut run, &mut budget);
        let cpu_writes = run.take_cpu_wram_writes();
        if let Some(copy) = cached_sprite_copy.as_mut() {
            for &(address, _) in &cpu_writes {
                copy.observe_wram_write(address);
            }
        }
        if (0x00_f427..0x00_f44b).contains(&executing_pc) {
            let (write_scanline, write_master_cycle) = budget.raster_position().coordinates();
            for &(address, _) in &cpu_writes {
                if !(0x1b00..0x1b00 + SPOTLIGHT_VISIBLE_SCANLINES * 2).contains(&address) {
                    continue;
                }
                let row = (address - 0x1b00) / 2;
                // Vblank stores precede every row of the following field. In
                // active display, compare the completed CPU store with that
                // row's physical HDMA read at master cycle 1106.
                spotlight_reset_rows_before_hdma[row] = Some(
                    write_scanline >= 225
                        || write_scanline < row as u16
                        || (write_scanline == row as u16 && write_master_cycle < 1_106),
                );
            }
        }
        // Reaching the dispatch target and actually executing its first
        // instruction are distinct CPU boundaries. An NMI may become pending
        // on the indirect-call instruction itself, in which case none of the
        // translated submodule body is allowed to run yet.
        if executing_submodule_dispatch {
            // JSR (abs,X) has just pushed the common dispatcher return and
            // selected the live submodule target. Record the target's stack
            // depth instead of naming every target routine individually.
            submodule_started = true;
            submodule_entry_sp = Some(run.stack_pointer());
        }
        submodule_started |= executing_submodule_entry;
        submodule_returned |= submodule_started
            && submodule_entry_sp.is_some_and(|entry_sp| run.stack_pointer() > entry_sp);
        sprite_main_started |= submodule_returned && executing_sprite_main_entry;
        sprite_main_returned |= sprite_main_started && executing_sprite_main_return;
        // Sprite routines may call LinkOam_Main themselves. Only the call
        // reached after the common Sprite_Main return belongs to Module 7's
        // caller suffix and can arm its continuation.
        link_oam_started |= sprite_main_returned && executing_link_oam_entry;
        link_oam_returned |= link_oam_started
            && link_oam_entry_sp.is_some_and(|entry_sp| run.stack_pointer() > entry_sp);
        // The common main loop calls NMI_PrepareSprites immediately after
        // Module_MainRouting returns. This PC is a stable semantic boundary
        // for both checkpoints; comparing stack depth against the dispatcher
        // checkpoint is invalid when a run starts one caller earlier at $8053.
        module_returned |= submodule_returned && executing_nmi_prepare_sprites_entry;
        nmi_prepare_sprites_returned |=
            nmi_prepare_sprites_entry_sp.is_some_and(|entry_sp| run.stack_pointer() > entry_sp);
        if advance.reached_boundary().is_some() {
            let phase = if module_returned && !nmi_prepare_sprites_returned {
                ModuleCpuPhase::InterruptedInNmiPrepareSprites
            } else if module_returned {
                ModuleCpuPhase::InterruptedAfterModule
            } else if executing_module_7_return || run.pc() == MODULE_7_RETURN_PC {
                ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites
            } else if link_oam_started && !link_oam_returned {
                ModuleCpuPhase::InterruptedInLinkOam
            } else if sprite_main_returned {
                ModuleCpuPhase::InterruptedAfterSpriteMain
            } else if sprite_main_started {
                ModuleCpuPhase::InterruptedInSpriteMain
            } else if submodule_returned {
                ModuleCpuPhase::InterruptedBeforeSpriteMain
            } else if submodule_started {
                ModuleCpuPhase::InterruptedInSubmodule
            } else {
                ModuleCpuPhase::InterruptedBeforeSubmodule
            };
            let interrupted = DungeonModuleCpuAdvance {
                phase,
                resumed_phase: None,
                submodule_nmi_slices: u8::from(phase == ModuleCpuPhase::InterruptedInSubmodule),
                subsubmodule: run.ram_byte(SUBSUBMODULE),
                palette_countdown: run.ram_byte(PALETTE_FILTER_COUNTDOWN),
                sprite_main_boundary: match phase {
                    ModuleCpuPhase::InterruptedBeforeSpriteMain => {
                        Some(SpriteMainCpuBoundary::BeforeFirstSlot)
                    }
                    ModuleCpuPhase::InterruptedInSpriteMain => {
                        Some(sprite_main_last_completed_slot.map_or(
                            SpriteMainCpuBoundary::BeforeFirstSlot,
                            SpriteMainCpuBoundary::AfterSlot,
                        ))
                    }
                    _ => None,
                },
                cached_sprite_interruption: cached_sprite_copy
                    .and_then(CachedSpriteCpuProgress::interruption),
            };
            if debug_cpu_phases {
                let (scanline, master_cycle) = budget.raster_position().coordinates();
                eprintln!(
                    "dungeon_cpu_phase host={} marker=interrupt pc={:06x} v={} cycles={} phase={:?} sp={:04x}",
                    state.frame_ctr_dbg,
                    run.pc(),
                    scanline,
                    master_cycle,
                    interrupted.phase,
                    run.stack_pointer(),
                );
            }
            if let Some(mut first) = first_interruption {
                if interrupted.phase == ModuleCpuPhase::InterruptedInSubmodule {
                    first.submodule_nmi_slices = first
                        .submodule_nmi_slices
                        .checked_add(1)
                        .expect("Module 7 submodule crossed more than 255 NMIs");
                    first_interruption = Some(first);
                    advance_rom_cpu_through_nmi(&mut run, &mut budget);
                    continue;
                }
                first.resumed_phase = Some(interrupted.phase);
                return DungeonModuleCpuTiming {
                    spotlight_reset_prefix_scanlines: spotlight_reset_prefix_scanlines(
                        &spotlight_reset_rows_before_hdma,
                        first.submodule_nmi_slices.saturating_add(1),
                    ),
                    advance: first,
                };
            }
            if interrupted.phase == ModuleCpuPhase::InterruptedInSubmodule {
                first_interruption = Some(interrupted);
                advance_rom_cpu_through_nmi(&mut run, &mut budget);
                continue;
            }
            return DungeonModuleCpuTiming {
                spotlight_reset_prefix_scanlines: spotlight_reset_prefix_scanlines(
                    &spotlight_reset_rows_before_hdma,
                    1,
                ),
                advance: interrupted,
            };
        }
    }
    panic!(
        "Module 7 ROM timing did not reach stop PC from {:06x}; stopped at {:06x}",
        checkpoint.entry_pc,
        run.pc(),
    );
}

/// Whether `next` is a later statement of the same descending Sprite_Main slot
/// loop than the suspension at `held` (the loop runs slot 15 down to 0).
fn sprite_main_slot_loop_advances(
    held: SpriteMainCpuBoundary,
    next: SpriteMainCpuBoundary,
) -> bool {
    matches!(next, SpriteMainCpuBoundary::AfterSlot(_))
        && sprite_main_cpu_boundary_order(next) > sprite_main_cpu_boundary_order(held)
}

/// Whether a suspended Sprite_Main call reached a later typed checkpoint
/// without returning its current slot. Besides the descending slot loop, a
/// synchronous follower-graphics load can cross several host boundaries while
/// advancing through its two decompressions and conversion.
fn sprite_main_in_flight_checkpoint_advances(
    held: SpriteMainCpuBoundary,
    next: SpriteMainCpuBoundary,
) -> bool {
    if sprite_main_slot_loop_advances(held, next) {
        return true;
    }
    let stage_cursor = |stage: crate::RescuedMaidenInitializationStage| match stage {
        crate::RescuedMaidenInitializationStage::FirstFollowerSheet { completed_bytes } => {
            (0u8, completed_bytes)
        }
        crate::RescuedMaidenInitializationStage::SecondFollowerSheet { completed_bytes } => {
            (1u8, completed_bytes)
        }
        crate::RescuedMaidenInitializationStage::Conversion { completed_stores } => {
            (2u8, completed_stores)
        }
    };
    match (held, next) {
        (
            SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(completed_slot)
            | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(completed_slot)
            | SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(completed_slot),
            SpriteMainCpuBoundary::AfterTimersAndOam { slot, .. },
        ) => slot < completed_slot,
        (
            SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(completed_slot)
            | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(completed_slot)
            | SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(completed_slot),
            SpriteMainCpuBoundary::AfterSlot(slot),
        ) => slot <= completed_slot,
        (
            SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller,
                stage,
                ..
            },
            SpriteMainCpuBoundary::FollowerGraphics {
                slot: next_slot,
                caller: next_caller,
                stage: next_stage,
                ..
            },
        ) => {
            slot == next_slot
                && caller == next_caller
                && stage_cursor(next_stage) > stage_cursor(stage)
        }
        _ => false,
    }
}

fn dungeon_module_7_cpu_advance_across_envelope(
    state: &ZeldaState,
    checkpoint: RomCpuCheckpoint,
    earliest_entry: CpuRasterPosition,
    latest_entry: CpuRasterPosition,
    context: &str,
) -> DungeonModuleCpuTiming {
    let earliest = dungeon_module_7_cpu_timing(state, checkpoint, Some(earliest_entry), false);
    let latest = dungeon_module_7_cpu_timing(state, checkpoint, Some(latest_entry), false);
    let earliest_advance = earliest.advance;
    let latest_advance = latest.advance;
    let earliest_key = (
        earliest_advance.phase,
        earliest_advance.resumed_phase,
        earliest_advance.submodule_nmi_slices,
        earliest.spotlight_reset_prefix_scanlines,
    );
    let latest_key = (
        latest_advance.phase,
        latest_advance.resumed_phase,
        latest_advance.submodule_nmi_slices,
        latest.spotlight_reset_prefix_scanlines,
    );
    if earliest_key != latest_key {
        // The calibrated entry envelope straddles a raster boundary for this
        // room's cost (route host 85231, room $BA). A live wire settles which
        // side the ROM actually took: a host whose vector completes the
        // shared main-loop suffix without a trailing Held acceptance proves
        // the iteration finished in-host; a mid-module suspension leaves the
        // suffix unfinished. Exactly one candidate may match that
        // disposition — anything else stays fail-closed.
        let wire_completed = matches!(state.original_timing_owner, OriginalTimingOwnerState::Live)
            .then(|| state.original_timing_semantic_receipts.as_ref())
            .flatten()
            .map(|receipts| {
                let suffix_completed = receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                    )
                });
                let trailing_held = receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld)
                    )
                });
                suffix_completed && !trailing_held
            });
        // The wire may also name the interrupted phase directly: an
        // interruption receipt (NMI_PrepareSprites at route host 204995,
        // LinkOam, or a Sprite_Main slot) or a Sprite_Main return followed by
        // a Held acceptance with no named boundary (InterruptedAfterSpriteMain).
        let wire_phase = matches!(state.original_timing_owner, OriginalTimingOwnerState::Live)
            .then(|| state.original_timing_semantic_receipts.as_ref())
            .flatten()
            .and_then(|receipts| {
                let interruption = receipts
                    .semantic()
                    .iter()
                    .find_map(|receipt| match receipt {
                        OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption) => {
                            Some(*interruption)
                        }
                        _ => None,
                    });
                if let Some(interruption) = interruption {
                    return module_cpu_phase_from_main_loop_interruption(interruption);
                }
                let sprite_main_returned = receipts
                    .semantic()
                    .iter()
                    .any(|receipt| *receipt == OriginalTimingSemanticReceipt::SpriteMainReturned);
                let trailing_held = receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld)
                    )
                });
                let suffix_completed = receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                    )
                });
                (sprite_main_returned && trailing_held && !suffix_completed)
                    .then_some(ModuleCpuPhase::InterruptedAfterSpriteMain)
            })
            // The raw vector is drained into the retained host facts before
            // some callers reach this envelope (route host 394502, state 13
            // landing with a LinkOam interruption); read the same facts back.
            .or_else(|| {
                if !matches!(state.original_timing_owner, OriginalTimingOwnerState::Live) {
                    return None;
                }
                if let Some(interruption) = state.original_timing_main_loop_interruption() {
                    return module_cpu_phase_from_main_loop_interruption(interruption);
                }
                let trailing_held = state
                    .original_timing_expected_nmi_update_gates
                    .contains(&NmiUpdateGate::LatchHeld);
                (state.original_timing_owes_sprite_main_return()
                    && trailing_held
                    && state.original_timing_live_suffix_outstanding())
                .then_some(ModuleCpuPhase::InterruptedAfterSpriteMain)
            });
        if let Some(wire_phase) = wire_phase {
            let earliest_matches = earliest_advance.phase == wire_phase;
            let latest_matches = latest_advance.phase == wire_phase;
            if earliest_matches != latest_matches {
                return if earliest_matches { earliest } else { latest };
            }
        }
        if let Some(wire_completed) = wire_completed {
            let earliest_completed = earliest_advance.phase == ModuleCpuPhase::CompleteBeforeNmi;
            let latest_completed = latest_advance.phase == ModuleCpuPhase::CompleteBeforeNmi;
            assert_ne!(
                earliest_completed, latest_completed,
                "{context} envelope split is not a completion-boundary split: {earliest_key:?} vs {latest_key:?}",
            );
            return if earliest_completed == wire_completed {
                earliest
            } else {
                latest
            };
        }
    }
    assert_eq!(
        earliest_key,
        latest_key,
        "{context} CPU continuation changes inside the observed dispatcher-entry envelope: \
         host_frame={} module={:02x}/{:02x}/{:02x} room={:04x} countdown={} scheduler={:?}",
        state.frame_ctr_dbg,
        state.game_state.frame.main_module,
        state.game_state.frame.submodule,
        state.game_state.frame.subsubmodule,
        state.game_state.world.location.dungeon_room(),
        state.game_state.display.palette_filter.countdown(),
        state.game_execution_scheduler,
    );
    if !matches!(
        earliest_advance.phase,
        ModuleCpuPhase::InterruptedBeforeSubmodule
            | ModuleCpuPhase::InterruptedInSubmodule
            | ModuleCpuPhase::InterruptedBeforeSpriteMain
            | ModuleCpuPhase::InterruptedInSpriteMain
            | ModuleCpuPhase::InterruptedAfterSpriteMain
            | ModuleCpuPhase::InterruptedInLinkOam
            | ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites
            | ModuleCpuPhase::InterruptedInNmiPrepareSprites
    ) {
        assert_eq!(
            (
                earliest_advance.subsubmodule,
                earliest_advance.palette_countdown,
            ),
            (
                latest_advance.subsubmodule,
                latest_advance.palette_countdown,
            ),
            "{context} ROM result changes inside the observed dispatcher-entry envelope"
        );
    }
    let sprite_main_boundary = match (
        earliest_advance.sprite_main_boundary,
        latest_advance.sprite_main_boundary,
    ) {
        (left, right) if left == right => left,
        (
            Some(SpriteMainCpuBoundary::AfterSlot(left)),
            Some(SpriteMainCpuBoundary::AfterSlot(right)),
        ) => {
            // Sprite_Main executes slots from 15 down to 0. If the raster
            // interval straddles a slot return, stop after the higher slot:
            // every possible entry has completed through that boundary, and
            // no possible entry has begun the resumed lower suffix.
            Some(SpriteMainCpuBoundary::AfterSlot(left.max(right)))
        }
        boundaries => panic!(
            "{context} CPU continuation crosses incompatible Sprite_Main boundaries: \
             {boundaries:?}"
        ),
    };
    DungeonModuleCpuTiming {
        advance: DungeonModuleCpuAdvance {
            sprite_main_boundary,
            ..earliest_advance
        },
        spotlight_reset_prefix_scanlines: earliest.spotlight_reset_prefix_scanlines,
    }
}

fn begin_dungeon_supertile_state_12_cpu_advance(state: &ZeldaState) -> DungeonModuleCpuAdvance {
    // State 12's palette workload makes the continuation sensitive to a few
    // thousand master cycles, so an observed dispatcher-entry envelope is not
    // a stable model. Start at the main-loop checkpoint instead: the isolated
    // CPU run executes the leading NMI and ordinary main prefix continuously,
    // deriving the Module 7 entry phase from the live NMI/DMA workload.
    dungeon_module_7_cpu_advance_after_leading_nmi(state, DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT)
}

/// Follow the real post-leading-NMI instruction stream far enough to learn
/// whether hardware interrupted a cached-sprite WRAM copy. The shadow is used
/// only in ROM-timing parity mode; the translated runtime consumes the result
/// as a semantic slot/field boundary and never reads ROM bytes itself.
fn begin_dungeon_cached_sprite_cpu_advance_after_leading_nmi(
    state: &ZeldaState,
) -> Option<DungeonModuleCpuAdvance> {
    let frame = state.game_state.frame;
    if !state.rom_startup_timing()
        || frame.main_module != 7
        || frame.submodule != 2
        || state.game_state.sprites.system.alt_sprites_flag() == 0
        || !(0..16).any(|slot| state.cached_sprite_slot(slot).is_active())
    {
        return None;
    }
    let advance =
        dungeon_module_7_cpu_advance(state, DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT, None, false);
    if debug_cached_sprite_cpu_for_host(state.frame_ctr_dbg) {
        eprintln!(
            "cached_sprite_cpu host={} entry={:02x}/{:02x}/{:02x} phase={:?} interruption={:?}",
            state.frame_ctr_dbg,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            advance.phase,
            advance.cached_sprite_interruption,
        );
    }
    Some(advance)
}

fn begin_dungeon_module_cpu_advance_after_leading_nmi(
    state: &ZeldaState,
) -> Option<DungeonModuleCpuAdvance> {
    begin_dungeon_module_cpu_timing_after_leading_nmi(state).map(|timing| timing.advance)
}

fn begin_dungeon_module_cpu_timing_after_leading_nmi(
    state: &ZeldaState,
) -> Option<DungeonModuleCpuTiming> {
    let frame = state.game_state.frame;
    (state.rom_startup_timing()
        && frame.main_module == 7
        && ((frame.submodule == 2 && matches!(frame.subsubmodule, 4..=7))
            || frame.submodule == 0x0f))
        .then(|| {
            dungeon_module_7_cpu_timing(state, DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT, None, false)
        })
}

fn begin_dungeon_landing_cpu_advance(state: &ZeldaState) -> DungeonModuleCpuTiming {
    let timing = rom_dungeon_landing_cpu_advance(state);
    let advance = timing.advance;
    if debug_cached_sprite_cpu_for_host(state.frame_ctr_dbg) {
        eprintln!(
            "cached_sprite_landing_cpu host={} entry={:02x}/{:02x}/{:02x} phase={:?} resumed={:?} submodule_nmis={} reset_prefix={:?} interruption={:?}",
            state.frame_ctr_dbg,
            state.game_state.frame.main_module,
            state.game_state.frame.submodule,
            state.game_state.frame.subsubmodule,
            advance.phase,
            advance.resumed_phase,
            advance.submodule_nmi_slices,
            timing.spotlight_reset_prefix_scanlines,
            advance.cached_sprite_interruption,
        );
    }
    timing
}

pub(super) fn debug_cached_sprite_cpu_for_host(host: u32) -> bool {
    let Some(value) = crate::debug_env::var_os("ZELDA3_DEBUG_CACHED_SPRITE_CPU") else {
        return false;
    };
    let value = value.to_string_lossy();
    value == "1"
        || value
            .split(',')
            .filter_map(|part| part.trim().parse::<u32>().ok())
            .any(|selected| selected == host)
}

fn rom_dungeon_landing_cpu_advance(state: &ZeldaState) -> DungeonModuleCpuTiming {
    let filtered = state.game_state.dungeon.torch.any_lights_out_request() != 0;
    let (earliest, latest) = if filtered {
        (
            DUNGEON_LANDING_FILTERED_CPU_ENTRY_EARLIEST,
            DUNGEON_LANDING_FILTERED_CPU_ENTRY_LATEST,
        )
    } else {
        (
            DUNGEON_LANDING_UNFILTERED_CPU_ENTRY_EARLIEST,
            DUNGEON_LANDING_UNFILTERED_CPU_ENTRY_LATEST,
        )
    };
    dungeon_module_7_cpu_advance_across_envelope(
        state,
        DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT,
        earliest,
        latest,
        "landing state",
    )
}

fn dungeon_palette_caller_cpu_advance(
    state: &ZeldaState,
    entry: CpuRasterPosition,
) -> DungeonPaletteCpuAdvance {
    let timing_dma = state.dma_with_native_hdma_enable();
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &timing_dma,
        state.zelda_audio_apu_output_ports(),
        DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT,
    )
    .expect("dungeon palette CPU timing requires the loaded Zelda ROM");
    let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
        entry,
        CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL_MASTER_CYCLES),
        CpuFieldTiming::non_interlace(state.frame_ctr_dbg & 1 == 0),
    );

    for _ in 0..200_000 {
        if run.is_complete() {
            let advance = DungeonPaletteCpuAdvance {
                work: CpuWorkAdvance::Complete,
                pc: run.pc(),
                subsubmodule: run.ram_byte(SUBSUBMODULE),
                palette_countdown: run.ram_byte(PALETTE_FILTER_COUNTDOWN),
            };
            return advance;
        }
        let (scanline, master_cycle) = budget.raster_position().coordinates();
        run.set_raster_position(scanline, master_cycle);
        let timing = run.step();
        let work = budget.advance_instruction(timing.master_cycles);
        if work.reached_boundary().is_some() {
            let advance = DungeonPaletteCpuAdvance {
                work,
                pc: run.pc(),
                subsubmodule: run.ram_byte(SUBSUBMODULE),
                palette_countdown: run.ram_byte(PALETTE_FILTER_COUNTDOWN),
            };
            return advance;
        }
    }
    panic!(
        "dungeon palette ROM timing did not reach stop PC from {:06x}; stopped at {:06x}",
        DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.entry_pc,
        run.pc(),
    );
}

const fn room_61_sprite_conversion_retains_resident_oam(
    work: DungeonSupertileTransitionWork,
    dungeon_room: u8,
) -> bool {
    dungeon_room == 0x61
        && matches!(
            work,
            DungeonSupertileTransitionWork::SpriteConversion
                | DungeonSupertileTransitionWork::SpriteConversionCallerResume
        )
}

const fn rescue_follower_message_obj_scanout(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    event_bit: u8,
    message: u16,
) -> Option<ObjScanoutGenerations> {
    if frame.main_module == 7
        && frame.submodule == 0
        && dungeon_room == 0x61
        && event_bit == 2
        && message == 0x21
    {
        let link_obj = if frame.frame_counter == 0xff {
            GraphicsDmaGeneration::LiveAfterMain
        } else {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        };
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj,
            link_obj_sources: link_obj,
        })
    } else {
        None
    }
}

const fn overworld_sprite_reload_timing(
    workload: OverworldSpriteReloadWorkload,
    entry_phase: OverworldSpriteReloadEntryPhase,
) -> OverworldSpriteReloadTiming {
    // The ROM loader is interruptible, so its return frame depends on the
    // actual area workload. Snes9x PC/V-counter traces show screen $2b
    // processing two sprite records and 18 in-bounds proximity checks, then
    // returning from $09:c55e to $02:ac27 at V=213 before that frame's NMI.
    // Screen $2c processes four records and 90 in-bounds checks, crosses NMI
    // inside $09:c6f6, and returns at V=9 on the following frame. A sprite-list
    // record costs about three proximity branches in the measured 65816 loop.
    let timing_units = workload
        .in_bounds_proximity_checks
        .saturating_add(workload.sprite_records * OVERWORLD_SPRITE_RECORD_TIMING_UNITS);
    let returns_before_nmi = timing_units <= OVERWORLD_SPRITE_RELOAD_SAME_FRAME_BUDGET_UNITS;
    let return_phase = if returns_before_nmi {
        NmiPhase::BeforeNmi
    } else {
        NmiPhase::AfterNmi
    };
    if matches!(
        entry_phase,
        OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail
    ) {
        // The graphics tail enters Module09_LoadNewSprites at the vblank edge.
        // The reload spans two host NMI boundaries. The returned scanout owns
        // the register generation at this CPU-slice entry: the transition's
        // direct BG2 adjustment has happened, while the provisional rain
        // suffix has not yet authored the following BG1 mirror generation.
        OverworldSpriteReloadTiming {
            load_nmi_slices: 2,
            post_return_hold_nmi_slices: 0,
            return_phase,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
        }
    } else if returns_before_nmi {
        OverworldSpriteReloadTiming {
            load_nmi_slices: 3,
            // The light loader returns before NMI, but its next Module09
            // iteration does not reach Overworld_StartScrollTransition until
            // V=255 of the following scanout.
            post_return_hold_nmi_slices: 1,
            return_phase,
            epilogue_phase: NmiPhase::AfterNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::ByReturnPhase(return_phase),
        }
    } else {
        OverworldSpriteReloadTiming {
            load_nmi_slices: 4,
            post_return_hold_nmi_slices: 0,
            return_phase,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::ByReturnPhase(return_phase),
        }
    }
}
// WorldMap_ExitMap enters InitializeTilesets while forced blank. From the
// From the first interrupted tileset-load frame through the boundary where the
// ROM writes music control $f3 and returns as module $09/$20, clean Snes9x
// state probes observe 33 later NMI slices (clean-route frames 17751..17783).
const WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES: u8 = 33;
// Dungeon-map initialization enters InitializeTilesets after the terminal
// fade and remains on that suspended 65816 stack for 31 complete vblanks.
// The caller returns after the NMI at route frame 14318.
const DUNGEON_MAP_GRAPHICS_PREPARATION_NMI_SLICES: u8 = 31;
// Drawing both dungeon-map room planes crosses the vblank at route frame
// 14322. The 65816 resumes the interrupted routine on the following host
// frame, so the marker/fade state cannot begin until one boundary later.
const DUNGEON_MAP_ROOM_DRAWING_NMI_SLICES: u8 = 1;
// DungeonMap_RecoverGFX restores the dungeon tilesets and rebuilds all eight
// room quadrants while the 65816 caller remains suspended. Instrumented ROM
// frames 2521..2556 keep the NMI latch set; the caller returns at frame 2557.
const DUNGEON_MAP_RECOVERY_NMI_SLICES: u8 = 36;
const SPOTLIGHT_MIXED_SCANOUT_LIVE_TAIL_START: usize = 221;

// Module07_02's room transition is one uninterrupted 65816 call stack, but it
// has three useful semantic return boundaries. A clean Snes9x PC/V-counter
// trace enters Dungeon_LoadRoom at $01:873a and crosses six NMIs before
// returning to $02:8a5f. LoadTransAuxGFX_sprite then crosses seven more before
// returning to $02:8a67. State 3's fixed-size LoadNewSpriteGFXSet conversion
// crosses three NMIs in Do3To4Low16Bit before its caller suffix can run.
// The auxiliary return lands at V=206, advances the room state, and reaches
// Dungeon_LoadSprites before NMI interrupts at $09:c1dd. Snes9x returns that
// host call at the boundary; the interrupted Module 7 sprite/HUD suffix resumes
// on the following call. Model that host-visible return explicitly so sprite
// initialization (including RNG) belongs to the same generation as the ROM.
//
// Keep these costs attached to the ROM routines rather than a route frame,
// room number, or rendered symptom. The typed stages also preserve which
// caller suffix owns each return.
const DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES: u8 = 6;
const DUNGEON_SUPERTILE_AUX_SPRITE_GFX_NMI_SLICES: u8 = 7;
const DUNGEON_SUPERTILE_SPRITE_CONVERSION_NMI_SLICES: u8 = 3;
const DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES: u8 = 1;
const DUNGEON_SUPERTILE_QUADRANT_TILEMAP_NMI_SLICES: u8 = 1;
const DUNGEON_STRAIGHT_INTERROOM_ROOM_INITIALIZATION_NMI_SLICES: u8 = 19;
const DUNGEON_STRAIGHT_INTERROOM_BG_CHARACTERS_34_NMI_SLICES: u8 = 4;
const DUNGEON_STRAIGHT_INTERROOM_SPRITE_GRAPHICS_NMI_SLICES: u8 = 4;
const DUNGEON_FALLING_BG_CHARACTERS_34_NMI_SLICES: u8 = 4;
const DUNGEON_FALLING_SPRITE_GRAPHICS_NMI_SLICES: u8 = 4;
const DUNGEON_SPIRAL_BG_CHARACTERS_34_NMI_SLICES: u8 = 3;
const DUNGEON_SPIRAL_SPRITE_GRAPHICS_NMI_SLICES: u8 = 3;

// IrisSpotlight_ConfigureTable's cost grows with the generated row-pair count.
// The 239-row table is the maximum `spotlight_table_row_pairs` can produce
// (vertical center at the bottom door line, e.g. leaving Link's house); with
// it, Snes9x traces show the module-15 ENTRY build and the $70 circle build
// also crossing vblank, one radius step beyond the 189-row calibration below.
const SPOTLIGHT_MAX_TABLE_ROW_PAIRS: u16 = 239;

const fn rom_dungeon_exit_spotlight_table_needs_entry_slice(
    radius: u16,
    vertical_center: u16,
) -> bool {
    // Snes9x PC traces show the $7e and $77 circle builds crossing vblank
    // inside IrisSpotlight_ConfigureTable. From $70 downward the next table is
    // far enough along at the first boundary to publish in that same slice —
    // except with the maximal 239-row table, where the $70 build still
    // crosses (measured on the Link's-house exit, vertical center 238).
    radius >= 0x77
        || (radius >= 0x70
            && spotlight_table_row_pairs(vertical_center) >= SPOTLIGHT_MAX_TABLE_ROW_PAIRS)
}

// IrisSpotlight_ConfigureTable waits for V=192, copies its 448-byte table, and
// only then writes the next radius at $00:f3cf. This boundary remains useful
// for classifying which table rows HDMA can consume in the same field.
const SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX: u16 = 0x38;

const fn spotlight_close_next_radius(radius: u16) -> u16 {
    radius.saturating_sub(load_gfx::SPOTLIGHT_RADIUS_STEP)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpiralStaircasePaletteTail {
    BuildQuadrantForVram,
    PrepareNextQuadrant,
    FallingFadeInAfterDirectionToggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LinkActualVelocityCheckpoint {
    BeforeSelection,
    BeforeX,
    BeforeY,
    AfterBoth,
    Clearing { completed: u8 },
}

impl From<Option<bool>> for LinkActualVelocityCheckpoint {
    fn from(horizontal_resolved: Option<bool>) -> Self {
        match horizontal_resolved {
            None => Self::BeforeSelection,
            Some(false) => Self::BeforeX,
            Some(true) => Self::BeforeY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectedGameLoadDestination {
    Dungeon,
    DarkWorldOverworld,
    Message,
}

const fn game_over_spotlight_build_uses_live_oam(work: Option<GameWorkContinuation>) -> bool {
    matches!(
        work,
        Some(GameWorkContinuation::FinishGameOverSpotlightBuild { .. })
    )
}

const fn game_over_spotlight_build_entry_uses_live_oam(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 0x12
        && entry.submodule == 3
        && captured.main_module == 0x12
        && captured.submodule == 3
        && entry.frame_counter != captured.frame_counter
}

const fn game_over_spotlight_return_boundary_uses_live_oam(
    frame: crate::game_state::FrameState,
    radius: u16,
    work: Option<GameWorkContinuation>,
) -> bool {
    frame.main_module == 0x12 && frame.submodule == 3 && radius <= 0x70 && work.is_none()
}

const fn game_over_iris_goal_scanout_is_closed(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 0x12
        && entry.submodule == 3
        && captured.main_module == 0x12
        && captured.submodule == 4
}

const fn scheduled_work_completion_clears_nmi_latch_after_interrupt(step: GameWorkStep) -> bool {
    matches!(
        step,
        GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSupertileTransition { work })
            if matches!(
                work,
                DungeonSupertileTransitionWork::State13CallerReturn
                    | DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn
                    | DungeonSupertileTransitionWork::FadedFilterCallerReturn
            )
    )
}

fn faded_filter_palette_completion_clears_nmi_latch_after_interrupt(
    completion_host_frame: Option<u32>,
    current_host_frame: u32,
    scheduler_is_idle: bool,
) -> bool {
    completion_host_frame == Some(current_host_frame) && scheduler_is_idle
}

/// The part of a held Module1A credits iteration the ROM completes in the
/// iteration's entry host, before its first multi-host decompression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CreditsEntryHostPrefix {
    /// The whole body runs where the wire returns the iteration.
    None,
    /// `Credits_LoadScene_Overworld_PrepGFX` through the music/ambient clears.
    OverworldPrepGfx,
    /// `Credits_LoadScene_Dungeon` through the torch/lantern clears.
    DungeonScene,
}

pub(super) fn spotlight_following_field_publication(
    active: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    following: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    has_queued_receipt: bool,
) -> SpotlightFollowingFieldPublication {
    if active == following || has_queued_receipt {
        SpotlightFollowingFieldPublication::AfterCompletionCapture
    } else {
        SpotlightFollowingFieldPublication::WithCompletionCapture
    }
}

const DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES: u8 = 1;

const fn rom_dungeon_landing_wipe_is_active(main_module: u8, submodule: u8) -> bool {
    main_module == 7 && submodule == 15
}

const fn rom_spiral_palette_completion_publishes_live_cgram_and_oam(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 0x0e
        && dungeon_room == 1
        && staircase_index == 0x30
}

const fn rom_spiral_second_palette_return_publishes_live_cgram(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    rom_spiral_palette_completion_publishes_live_cgram_and_oam(frame, dungeon_room, staircase_index)
        && frame.subsubmodule == 0x0f
}

const fn rom_spiral_stairs_second_palette_return_uses_host_animated_bg_operands(
    frame: crate::game_state::FrameState,
    host_main_prefix_did_not_advance: bool,
    countdown: u16,
    current_source: usize,
) -> bool {
    // Snes9x's NMI runs with the global core-update gate clear at these
    // boundaries: Link OBJ DMA advances normally. Only the animated-BG source
    // is the host-boundary generation. Keep that operand ownership local to
    // its DMA instead of suppressing every core transfer through $0710.
    current_source != ANIMATED_TILE_BUFFER_FIRST_SOURCE + 0x400
        && rom_spiral_stairs_suspended_animated_bg_source_address(
            frame,
            host_main_prefix_did_not_advance,
            countdown,
            current_source,
        )
        .is_some()
}

const fn rom_spiral_palette_slice_retains_previous_cgram(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
    palette_filter_countdown: u16,
) -> bool {
    entry.main_module == 7
        && entry.submodule == 0x0e
        && entry.subsubmodule == 0x0f
        && matches!(captured.subsubmodule, 0x0f | 0x10)
        && entry.frame_counter == captured.frame_counter
        && ((palette_filter_countdown < 0x1b && palette_filter_countdown & 1 == 1)
            || (palette_filter_countdown == 0 && captured.subsubmodule == 0x10))
        && !rom_spiral_palette_completion_publishes_live_cgram_and_oam(
            captured,
            dungeon_room,
            staircase_index,
        )
}

const fn rom_spiral_stair_landing_publishes_live_display(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
    y_button_action_step: u8,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 0x0e
        && frame.subsubmodule == 0x11
        && dungeon_room == 1
        && staircase_index == 0x30
        && y_button_action_step != 0
}

const fn rom_spiral_stair_motion_publishes_live_oam(
    frame: crate::game_state::FrameState,
    dungeon_room: u8,
    staircase_index: u8,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 0x0e
        && matches!(frame.subsubmodule, 0x12 | 0x13)
        && dungeon_room == 1
        && staircase_index == 0x30
}

// Cartridge-state sweeps and Snes9x PC/V-counter traces place the exact
// CPU/NMI workload crossover at 184 generated row pairs, symmetrically at
// vertical centers 41/42 and 182/183. Below it, the table and its $80:f427
// goal reset reach the next hardware publication boundary; at and above it,
// the calculation remains on the preceding table generation for one more
// scanout.
const SPOTLIGHT_LONG_NMI_WORKLOAD_MIN_ROW_PAIRS: u16 = 184;
// With the dungeon landing's 189-row workload, the completed table copy can
// still reach HDMA's scanline-221 read through radius $3f. The following ROM
// step ($46) misses that consumer.
const SPOTLIGHT_OPENING_LIVE_TAIL_MAX_RADIUS: u16 = 0x3f;
const fn spotlight_vertical_center(link_y: u16, bg2_y: u16) -> u16 {
    link_y.wrapping_sub(bg2_y).wrapping_add(12)
}

const fn spotlight_table_row_pairs(vertical_center: u16) -> u16 {
    let doubled_center = vertical_center.wrapping_mul(2);
    let lower_cursor = if doubled_center < 224 {
        224
    } else {
        doubled_center
    };
    lower_cursor.wrapping_sub(vertical_center).wrapping_add(1)
}

const fn spotlight_table_has_long_nmi_workload(vertical_center: u16) -> bool {
    spotlight_table_row_pairs(vertical_center) >= SPOTLIGHT_LONG_NMI_WORKLOAD_MIN_ROW_PAIRS
}

const fn spotlight_mixed_scanout_live_tail_start(vertical_center: u16, radius: u16) -> usize {
    // The 189-row close finishes its reserved-table copy while scanline 221
    // can still consume the new words. On the maximal 239-row close, the
    // $3f->$38 build does not finish until V=224; once the smaller $31 table
    // is being built, the copy again reaches the last three active lines.
    if spotlight_table_row_pairs(vertical_center) >= SPOTLIGHT_MAX_TABLE_ROW_PAIRS
        && radius >= SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX
    {
        224
    } else {
        SPOTLIGHT_MIXED_SCANOUT_LIVE_TAIL_START
    }
}

const fn spotlight_opening_projects_live_tail_before_hdma(
    radius: u16,
    vertical_center: u16,
) -> bool {
    spotlight_table_has_long_nmi_workload(vertical_center)
        && radius <= SPOTLIGHT_OPENING_LIVE_TAIL_MAX_RADIUS
}

const fn rom_dungeon_landing_goal_transition_waits_for_caller_return(
    main_module: u8,
    submodule: u8,
) -> bool {
    rom_dungeon_landing_wipe_is_active(main_module, submodule)
}

const DUNGEON_ANIMATED_BG_VRAM_DESTINATION: usize = 0x3b00;
const OVERWORLD_ANIMATED_BG_VRAM_DESTINATION: usize = 0x3c00;

const ASSET_SIGNATURE_PREFIX: &[u8; 16] = b"Zelda3_v0     \n\0";
const DIALOGUE_SOURCE_SIDECAR_ASSET_NAME: &str = "kDialogueSourceSemantic";
const SPC_DRIVER_TIMING_ASSET_NAME: &str = "kSpcDriverTimingProgram";
const DIALOGUE_SOURCE_SIDECAR_MAGIC: &[u8; 16] = b"Z3DLGSRCv1\0\0\0\0\0\0";
const REFERENCE_SAVE_NAMES: [&str; 13] = [
    "Chapter 1 - Zelda's Rescue.sav",
    "Chapter 2 - After Eastern Palace.sav",
    "Chapter 3 - After Desert Palace.sav",
    "Chapter 4 - After Tower of Hera.sav",
    "Chapter 5 - After Hyrule Castle Tower.sav",
    "Chapter 6 - After Dark Palace.sav",
    "Chapter 7 - After Swamp Palace.sav",
    "Chapter 8 - After Skull Woods.sav",
    "Chapter 9 - After Gargoyle's Domain.sav",
    "Chapter 10 - After Ice Palace.sav",
    "Chapter 11 - After Misery Mire.sav",
    "Chapter 12 - After Turtle Rock.sav",
    "Chapter 13 - After Ganon's Tower.sav",
];
const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

const PALETTE_ASSET_SNES_RANGES: &[(u32, usize)] = &[
    (PALETTE_MAIN_SPRITE_SNES_ADDR, 80),
    (PALETTE_ARMOR_AND_GLOVES_SNES_ADDR, 81),
    (PALETTE_SWORD_SNES_ADDR, 82),
    (PALETTE_SHIELD_SNES_ADDR, 83),
    (PALETTE_SPRITE_AUX3_SNES_ADDR, 84),
    (PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR, 85),
    (PALETTE_SPRITE_AUX1_SNES_ADDR, 86),
    (HUD_PALETTE_SNES_ADDR, 92),
    (PALETTE_DUNGEON_BG_MAIN_SNES_ADDR, 79),
    (PALETTE_PALACE_MAP_SPRITE_SNES_ADDR, 91),
    (PALETTE_PALACE_MAP_BG_SNES_ADDR, 90),
    (PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR, 87),
    (PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 88),
    (PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR, 89),
];

const UPPER_BITMASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010,
    0x0008, 0x0004, 0x0002, 0x0001,
];

const RTL_RECEIVE_ITEM_OAM_EXT_SIZES: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const RTL_RECEIVE_ITEM_DRAW_Y_OFFSETS: [i8; 76] = [
    -5, -5, -5, -5, -5, -4, -4, -5, -5, -4, -4, -4, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -5, -4, -4, -4, -4, -4, -4, -2, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -2, -2, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -2, -2, -4, -2, -4, -4,
    -4, -5, -4, -4,
];
const RTL_RECEIVE_ITEM_PALETTE_BITS: [u8; 76] = [
    4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
];
const GIVE_ITEM_MEMORY_LOCATIONS: [usize; 76] = [
    0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342, 0xf340,
    0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350, 0xf35c, 0xf36b,
    0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a, 0xf34d, 0xf35b, 0xf35b,
    0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c, 0xf35c, 0xf35c, 0xf36d, 0xf36e,
    0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360, 0xf374, 0xf374, 0xf374, 0xf340, 0xf340,
    0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360, 0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360,
    0xf35c, 0xf359, 0xf34c, 0xf355,
];
const GIVE_ITEM_VALUES: [u8; 76] = [
    1, 2, 3, 4, 1, 2, 3, 1, 1, 1, 1, 1, 1, 2, 0xff, 1, 1, 1, 1, 1, 2, 1, 0xff, 0xff, 1, 1, 2, 1, 2,
    1, 1, 1, 0xff, 1, 0xff, 2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 2, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xfb, 0xec, 0xff, 0xff, 0xff, 1, 3, 0xff, 0xff, 0xff, 0xff, 0x9c,
    0xce, 0xff, 1, 10, 0xff, 0xff, 0xff, 0xff, 1, 3, 1,
];

fn configured_rom_reset_frame_delay() -> u8 {
    env::var("ZELDA3_ROM_RESET_FRAME_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(ROM_RESET_FRAME_DELAY)
}

pub(super) fn configured_intro_memory_initialization_frames() -> u8 {
    env::var("ZELDA3_ROM_INTRO_MEMORY_INITIALIZATION_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(ROM_INTRO_MEMORY_INITIALIZATION_FRAMES)
}

const fn intro_memory_initialization_delay_for_owner(
    live_source_owner: bool,
    configured_fallback: u8,
) -> Option<u8> {
    if live_source_owner {
        Some(1)
    } else if configured_fallback != 0 {
        Some(configured_fallback)
    } else {
        None
    }
}

pub(super) fn configured_intro_poly_bootstrap_steps() -> u8 {
    env::var("ZELDA3_SNES9X_INTRO_POLY_BOOTSTRAP_STEPS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(SNES9X_INTRO_POLY_BOOTSTRAP_STEPS)
}

pub(super) fn configured_intro_thread_start_delay() -> u8 {
    env::var("ZELDA3_SNES9X_INTRO_THREAD_START_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(SNES9X_INTRO_THREAD_START_DELAY)
}

fn configured_nmi_poly_upload_defer_frames() -> u8 {
    env::var("ZELDA3_SNES9X_NMI_POLY_UPLOAD_DEFER_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(SNES9X_NMI_POLY_UPLOAD_DEFER_FRAMES)
}

fn configured_poly_upload_defer_until_frame_counter() -> u8 {
    env::var("ZELDA3_SNES9X_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER")
        .ok()
        .and_then(|value| {
            u8::from_str_radix(value.trim_start_matches("0x"), 16)
                .or_else(|_| value.parse())
                .ok()
        })
        .unwrap_or(SNES9X_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER)
}

fn non_empty_path(value: Option<OsString>) -> Option<PathBuf> {
    let value = value?;
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

#[path = "ancilla.rs"]
mod ancilla;
#[path = "attract.rs"]
mod attract;
#[path = "audio.rs"]
mod audio;
#[path = "dungeon.rs"]
mod dungeon;
#[path = "ending.rs"]
mod ending;
#[path = "game_execution_scheduler.rs"]
mod game_execution_scheduler;
use game_execution_scheduler::*;
#[path = "hud.rs"]
mod hud;
#[path = "load_gfx.rs"]
mod load_gfx;
#[path = "messaging.rs"]
mod messaging;
#[path = "misc.rs"]
mod misc;
#[path = "nmi.rs"]
mod nmi;
#[path = "overlord.rs"]
mod overlord;
#[path = "overworld.rs"]
mod overworld;
#[path = "player.rs"]
mod player;
#[path = "player_oam.rs"]
mod player_oam;
#[path = "poly.rs"]
mod poly;
#[path = "select_file.rs"]
mod select_file;
#[path = "sprite.rs"]
mod sprite;
pub use crate::game_state::CachedSpriteCacheField;
pub use sprite::{
    DungeonLoadSpritesCpuProgress, DungeonResetSpritesCpuProgress, DungeonSpriteDisableCpuProgress,
    DungeonSpriteLoadCheckpoint,
};
#[path = "sprite_main.rs"]
mod sprite_main;
#[path = "sprite_main_blind.rs"]
mod sprite_main_blind;
#[path = "sprite_main_draw.rs"]
mod sprite_main_draw;
#[path = "sprite_main_dungeon_npcs.rs"]
mod sprite_main_dungeon_npcs;
#[path = "sprite_main_ganon.rs"]
mod sprite_main_ganon;
#[path = "sprite_main_guard.rs"]
mod sprite_main_guard;
#[path = "sprite_main_helmasaur_king.rs"]
mod sprite_main_helmasaur_king;
#[path = "sprite_main_hinox_shop.rs"]
mod sprite_main_hinox_shop;
#[path = "sprite_main_mothula.rs"]
mod sprite_main_mothula;
#[path = "sprite_main_npcs.rs"]
mod sprite_main_npcs;
#[path = "sprite_main_prep.rs"]
mod sprite_main_prep;
#[path = "sprite_main_small_bosses.rs"]
mod sprite_main_small_bosses;
#[path = "sprite_main_world.rs"]
mod sprite_main_world;
#[path = "tagalong.rs"]
mod tagalong;
#[path = "tile_detect.rs"]
mod tile_detect;

pub const SRAM_SIZE: usize = 0x2000;
pub const VRAM_WORDS: usize = 0x8000;

const SPIN_ATTACK_DELAYS: [u8; 18] = [1, 0, 0, 0, 0, 3, 0, 0, 1, 0, 3, 3, 3, 3, 4, 4, 1, 5];

/// The ROM page starting at the spin-attack delay table (`$07:9CBF`). The
/// ROM indexes that table with the raw `button_b_frames` byte, which carries
/// the charged-spin flag in bit 7 (route host 240019 read index $91 = $A0),
/// so out-of-table indices read the code bytes that follow it.
const SPIN_ATTACK_DELAY_ROM_PAGE: [u8; 256] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x01, 0x00, 0x03, 0x03, 0x03, 0x03, 0x04, 0x04,
    0x01, 0x05, 0x01, 0x02, 0x03, 0x04, 0x00, 0x09, 0x12, 0x1b, 0xa5, 0x3b, 0x29, 0x10, 0xd0, 0xdf,
    0x24, 0x3a, 0x30, 0x20, 0x24, 0xf4, 0x10, 0xd7, 0xa6, 0x6c, 0xf0, 0x0b, 0x20, 0x3e, 0xd7, 0xa5,
    0x0e, 0x29, 0x30, 0x49, 0x30, 0xf0, 0xc8, 0xa9, 0x80, 0x04, 0x3a, 0x20, 0x66, 0x9c, 0xa9, 0x01,
    0x04, 0x50, 0x64, 0x2e, 0x24, 0xf0, 0x30, 0x04, 0xa9, 0x01, 0x04, 0x3a, 0x20, 0x65, 0xae, 0xa5,
    0x67, 0x29, 0xf0, 0x85, 0x67, 0xc6, 0x3d, 0x10, 0x56, 0xe6, 0x3c, 0xa5, 0x3c, 0xc9, 0x09, 0xb0,
    0x52, 0xaa, 0xbd, 0xbf, 0x9c, 0x85, 0x3d, 0xe0, 0x05, 0xd0, 0x2d, 0xaf, 0x59, 0xf3, 0x7e, 0xf0,
    0x10, 0xc9, 0x01, 0xf0, 0x0c, 0xc9, 0xff, 0xf0, 0x08, 0xa0, 0x04, 0xa9, 0x26, 0x22, 0xc2, 0x93,
    0x09, 0xa0, 0x01, 0xaf, 0x59, 0xf3, 0x7e, 0xf0, 0x26, 0xc9, 0xff, 0xf0, 0x22, 0xc9, 0x01, 0xf0,
    0x02, 0xa0, 0x06, 0x20, 0x77, 0xd0, 0x80, 0x17, 0xe0, 0x04, 0x90, 0x13, 0xa5, 0x3a, 0x29, 0x01,
    0xf0, 0x0d, 0x24, 0xf0, 0x10, 0x09, 0xa5, 0x3a, 0x29, 0xfe, 0x85, 0x3a, 0x82, 0xf8, 0xfe, 0x20,
    0x63, 0x9e, 0x60, 0x24, 0xf0, 0x30, 0x29, 0xa5, 0x79, 0xc9, 0x30, 0x90, 0x08, 0x20, 0x84, 0x9d,
    0x64, 0x79, 0x82, 0xf6, 0x09, 0x64, 0x5e, 0xa5, 0x48, 0x29, 0xf6, 0x85, 0x48, 0x64, 0x3d, 0x64,
    0x3c, 0xa5, 0x3a, 0x29, 0x7e, 0x85, 0x3a, 0xa5, 0x50, 0x29, 0xfe, 0x85, 0x50, 0x82, 0xc3, 0x00,
    0x24, 0x48, 0x30, 0x06, 0xa5, 0x48, 0x29, 0x09, 0xd0, 0x08, 0xa5, 0x47, 0xf0, 0x58, 0xc9, 0x01,
    0xf0, 0xd3, 0xa5, 0x3c, 0xc9, 0x09, 0xd0, 0x09, 0xa2, 0x0a, 0x86, 0x3c, 0xbd, 0xbf, 0x9c, 0x85,
];

/// `kSpinAttackDelays[button_b_frames]` exactly as the ROM reads it.
pub(super) const fn spin_attack_delay_for_frames(frames: u8) -> u8 {
    SPIN_ATTACK_DELAY_ROM_PAGE[frames as usize]
}
const FIRE_BEAM_SOUNDS: [u8; 8] = [1, 2, 3, 4, 0, 9, 18, 27];
const LINK_SPIN_GRAPHICS_BY_DIR: [u8; 48] = [
    10, 11, 10, 6, 7, 8, 9, 2, 3, 4, 5, 10, 0, 1, 0, 2, 3, 4, 5, 6, 7, 8, 9, 0, 12, 13, 12, 4, 5,
    6, 7, 8, 9, 2, 3, 12, 14, 15, 14, 8, 9, 2, 3, 4, 5, 6, 7, 14,
];
const LINK_SPIN_DELAYS: [u8; 12] = [1, 5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5];
const HOP_SOUTH_Y: [i8; 2] = [-8, 8];
const HOP_SOUTH_Y2: [i8; 2] = [-16, 16];
const HOP_HORIZ_VEL_Z: [u8; 8] = [32, 32, 32, 40, 48, 56, 64, 72];
const HOP_HORIZ_VEL_X: [u8; 8] = [16, 28, 28, 28, 28, 28, 28, 28];
const HOP_HORIZ_X_STEP: [i8; 2] = [-8, 8];
const HOP_HORIZ_X_FALLBACK: [i8; 2] = [-32, 32];
const HOP_HORIZ_X_FINAL: [i8; 2] = [-16, 16];
const HOP_HORIZ_X_VEL: [u8; 24] = [
    20, 20, 20, 24, 24, 24, 24, 28, 28, 36, 36, 36, 36, 36, 36, 38, 38, 38, 38, 38, 38, 38, 40, 40,
];
const HOP_HORIZ_Z_VEL: [u8; 24] = [
    20, 20, 20, 20, 20, 20, 20, 24, 24, 32, 32, 32, 36, 36, 36, 38, 38, 38, 38, 38, 38, 38, 40, 40,
];
const LEDGE_DOWN_X_VEL: [u8; 24] = [
    4, 4, 4, 10, 10, 10, 11, 18, 18, 18, 20, 20, 20, 20, 22, 22, 26, 26, 26, 26, 28, 28, 28, 28,
];
const LEDGE_DIAG_DX: [i8; 2] = [-8, 8];
const LEDGE_DIAG_DY: [i8; 2] = [-9, 9];
const LEDGE_DIAG_BITS: [u8; 2] = [6, 3];
const LEDGE_DIAG_DY2: [i8; 2] = [-24, 24];
const FALL_HOLE_PIT_DIRS: [u8; 4] = [12, 3, 10, 5];
const FALL_HOLE_DIRS: [u8; 8] = [5, 6, 9, 10, 4, 8, 1, 2];
const FALL_HOLE_DIRS2: [u8; 8] = [10, 9, 6, 5, 8, 4, 2, 1];
const GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];
const GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];
const GRAB_WALL_ANIM_STEPS2: [u8; 10] = [0, 1, 2, 3, 4, 0, 1, 2, 3, 0x20];

const LINK_Y_COORD: usize = 0x20;
const LINK_X_COORD: usize = 0x22;
const LINK_Z_COORD: usize = 0x24;
const LINK_DIRECTION_LAST: usize = 0x26;
const ATTRACT_NEXT_LEGEND_GFX: usize = 0x26;
const NMI_LOAD_BG_FROM_VRAM: usize = 0x14;
const NMI_COPY_PACKETS_FLAG: usize = 0x18;
const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x15;
const FLAG_UPDATE_HUD_IN_NMI: usize = 0x16;
const HUD_TILEMAP_NMI_WORDS: usize = 165;
const HUD_TILEMAP_VRAM_DESTINATION: usize = 0x6040;
const FULL_TILEMAP_NMI_WORDS: usize = 0x400;
// Shared zero-page scratch; NES_Ver2 aliases include BMWORK/CRTNL/CRTNR, but these slots
// are reused by unrelated player, overworld, and tile-detection code paths.
const SCRATCH_0: usize = 0x72;
const SCRATCH_A: usize = 0x73;
const SCRATCH_1: usize = 0x74;
const LINK_SUBPIXEL_Y: usize = 0x2a;
const LINK_SUBPIXEL_X: usize = 0x2b;
const LINK_SUBPIXEL_Z: usize = 0x2c;
// NES_Ver2: PYFLCH, player frame-change counter.
const LINK_FRAME_CHANGE_COUNTER: usize = 0x2d;
const LINK_ANIMATION_STEPS: usize = 0x2e;
const LINK_Y_VEL: usize = 0x30;
const LINK_X_VEL: usize = 0x31;
const LINK_Y_COORD_ORIGINAL: usize = 0x32;
const LINK_Y_COORD_SAFE_RETURN_LO: usize = 0x3e;
const LINK_X_COORD_SAFE_RETURN_LO: usize = 0x3f;
const LINK_Y_COORD_SAFE_RETURN_HI: usize = 0x40;
const LINK_X_COORD_SAFE_RETURN_HI: usize = 0x41;
const BUTTON_MASK_B_Y: usize = 0x3a;
// NES_Ver2: KENKYL, "y key flag".
const Y_BUTTON_ACTION_FLAGS: usize = 0x3b;
const BUTTON_B_FRAMES: usize = 0x3c;
const LINK_DELAY_TIMER_SPIN_ATTACK: usize = 0x3d;
const LINK_DIRECTION_MASK_A: usize = 0x42;
const LINK_DIRECTION_MASK_B: usize = 0x43;
const SET_WHEN_DAMAGING_ENEMIES: usize = 0x47;
// NES_Ver2: HANIFG1, "sword defense flag".
const PLAYER_DEFENSE_FLAGS: usize = 0x48;
const FORCE_MOVE_ANY_DIRECTION: usize = 0x49;
const LINK_VISIBILITY_STATUS: usize = 0x4b;
const CAPE_DECREMENT_COUNTER: usize = 0x4c;
const INDEX_OF_DASHING_SFX: usize = 0x4f;
const LINK_SPRITE_OAM_STATE_TIMER: usize = 0x5c;
const LINK_CANT_CHANGE_DIRECTION: usize = 0x50;
const TILEDETECT_WHICH_Y_POS: usize = 0x51;
const LINK_CAPE_MODE: usize = 0x55;
const LINK_IS_BUNNY: usize = 0x56;
const LINK_SPEED_MODIFIER: usize = 0x57;
// NES_Ver2: BKONFG/DRMKFG, tile-detect block and door direction flags.
const GRAVESTONE_PUSH_TIMEOUT: usize = 0x61;
const LINK_LAST_DIRECTION_MOVED_TOWARDS: usize = 0x66;
const FLAG_IS_LINK_IMMOBILIZED: usize = 0x2e4;
const LINK_Y_PAGE_MOVEMENT_DELTA: usize = 0x68;
const LINK_X_PAGE_MOVEMENT_DELTA: usize = 0x69;
const OVERWORLD_SCROLL_DELTA: usize = 0x69e;
const LINK_NUM_ORTHOGONAL_DIRECTIONS: usize = 0x6a;
const LINK_MOVING_AGAINST_DIAG_TILE: usize = 0x6b;
const MOVING_AGAINST_DIAG_DEADLOCKED: usize = 0x6d;
const LINK_DIRECTION: usize = 0x67;
const INDEX_OF_INTERACTING_TILE: usize = 0x76;
const ALLOW_SCROLL_Z: usize = 0x78;
const LINK_SPIN_ATTACK_STEP_COUNTER: usize = 0x79;
const BG1_X_OFFSET: usize = 0x11a;
const BG1_Y_OFFSET: usize = 0x11c;
const FLAG_CUSTOM_SPELL_ANIM_ACTIVE: usize = 0x112;
const OAM_CUR_PTR: usize = 0x90;
const OAM_EXT_CUR_PTR: usize = 0x92;
const OVERLAY_INDEX: usize = 0x8c;
const LAST_LIGHT_VS_DARK_WORLD: usize = 0x7b;
const DUNG_DRAW_WIDTH_INDICATOR: usize = 0xb2;
const DUNG_DRAW_HEIGHT_INDICATOR: usize = 0xb4;
const DUNG_LINE_PTRS_ROW0: usize = 0xbf;
const DUNG_LOAD_PTR_OFFS: usize = 0xba;
const DUNG_CUR_FLOOR: usize = 0xa4;
const QUADRANT_FULLSIZE_X: usize = 0xa6;
const QUADRANT_FULLSIZE_Y: usize = 0xa7;
const COMPOSITE_OF_LAYOUT_AND_QUADRANT: usize = 0xa8;
const DUNG_HDR_TAG: usize = 0xae;
const LINK_QUADRANT_X: usize = 0xa9;
const LINK_QUADRANT_Y: usize = 0xaa;
const IS_STANDING_IN_DOORWAY: usize = 0x6c;
const TILEMAP_LOCATION_CALC_MASK: usize = 0xec;
const ROOM_TRANSITIONING_FLAGS: usize = 0xef;
const DUNG_HDR_COLLISION_2: usize = 0xad;
const LINK_RECOIL_Z_VEL: usize = 0xc7;
const KSRM_OFFS_GLOVES: usize = 0x354;
const KSRM_OFFS_DIED_COUNTER: usize = 0x405;
const KSRM_OFFS_HEALTH: usize = 0x36c;
const KSRM_OFFS_SWORD: usize = 0x359;
const KSRM_OFFS_SHIELD: usize = 0x35a;
const KSRM_OFFS_ARMOR: usize = 0x35b;
const KSRM_OFFS_NAME: usize = 0x3d9;
const INTRO_SWORD_YPOS: usize = 0xc8;
const INTRO_SWORD_18: usize = 0xca;
const INTRO_SWORD_19: usize = 0xcb;
const INTRO_SWORD_20: usize = 0xcc;
const INTRO_SWORD_21: usize = 0xcd;
const INTRO_SWORD_24: usize = 0xd0;
const LINK_DMA_GRAPHICS_INDEX: usize = 0x100;
const LINK_DMA_LEFT_SPRITE_BANK_INDEX: usize = 0x102;
const LINK_DMA_RIGHT_SPRITE_BANK_INDEX: usize = 0x104;
// NES_Ver2: KENCPT/TATCPT, sword and shield graphics DMA indices.
const LINK_DMA_SWORD_GRAPHICS_INDEX: usize = 0x107;
const LINK_DMA_SHIELD_GRAPHICS_INDEX: usize = 0x108;
const LINK_TILE_BELOW: usize = 0x114;
const CHEAT_WALK_THROUGH_WALLS: usize = 0x37f;
const JOYPAD1H_LAST: usize = 0xf0;
const JOYPAD1L_LAST: usize = 0xf2;
const FILTERED_JOYPAD_H: usize = 0xf4;
const FILTERED_JOYPAD_L: usize = 0xf6;
const JOYPAD1H_LAST2: usize = 0xf8;
const JOYPAD1L_LAST2: usize = 0xfa;
const WHICH_ENTRANCE: usize = 0x10e;
const OVERWORLD_HOLE_SCAN_STEP: usize = 0x10f;
const OAM_PRIORITY_VALUE: usize = 0x64;
// NES_Ver2: GOVRCFG, game-over check flag.
const GAME_OVER_CHECK_FLAG: usize = 0x10a;
const MAPBAK_TM: usize = 0x0c211;
const MAPBAK_TS: usize = 0x0c212;
const LINK_Y_COORD_SPEXIT: usize = 0x0c108;
const LINK_X_COORD_SPEXIT: usize = 0x0c10a;
const MAPBAK_CGWSEL: usize = 0x0c225;
const MAPBAK_HDMAEN: usize = 0x0c229;
// NES_Ver2: BKMODE, "block mode flag".
const PUSHED_BLOCK_MODE: usize = 0x2c3;
const LINK_INCAPACITATED_CAMERA_TIMER: usize = 0x2c5;
const SWIMMING_COUNTDOWN: usize = 0x2cb;
const TAGALONG_DATA_INDEX: usize = 0x2cf;
const TIMER_TAGALONG_REACQUIRE: usize = 0x2d2;
const SHARED_MESSAGE_TIMER: usize = 0x2cd;
const SWIM_STROKE_ANIM_STEP: usize = 0x2cc;
const TAGALONG_SHARED_STATE_A: usize = 0x2d4;
const TAGALONG_JUMP_TIMER: usize = 0x2d6;
const TAGALONG_ANIM_FRAME_COUNTER: usize = 0x2d7;
const TILE_INTERACTION_SHARED_FLAG: usize = 0x223;
const LINK_POSE_FOR_ITEM: usize = 0x2da;
const LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE: usize = 0x2db;
const LINK_X_COORD_COPY: usize = 0x2dc;
const LINK_Y_COORD_COPY: usize = 0x2de;
const LINK_IS_BUNNY_MIRROR: usize = 0x2e0;
const LINK_IS_TRANSFORMING: usize = 0x2e1;
const LINK_BUNNY_TRANSFORM_TIMER: usize = 0x2e2;
const LINK_SWORD_DELAY_TIMER: usize = 0x2e3;
// NES_Ver2: HLMKCT, pit/hole correction timer.
const PIT_CORRECTION_TIMER: usize = 0x2ca;
const FALL_HOLE_SCAN_INDEX: usize = 0x2c9;
const ITEM_RECEIPT_METHOD: usize = 0x2e9;
const TILEDETECT_INROOM_STAIRCASE: usize = 0x2c0;
const LINK_RECEIVEITEM_INDEX: usize = 0x2d8;
// NES_Ver2: ATMTTM, item holding timer.
const LINK_ITEM_HOLDING_TIMER: usize = 0x2d9;
const FLAG_IS_ANCILLA_TO_PICK_UP: usize = 0x2ec;
const ITEM_PICKUP_IN_PROGRESS_FLAG: usize = 0x2ed;
const FLAG_IS_SPRITE_TO_PICK_UP_CACHED: usize = 0x2f4;
const TILEDETECT_MISC_TILES: usize = 0x2f6;
const MESSAGE_OR_SPRITE_STATE_CACHE: usize = 0x2f0;
const TAGALONG_EVENT_FLAGS: usize = 0x2f2;
const LINK_WANT_MAKE_NOISE_WHEN_DASHED: usize = 0x2f8;
const TAGALONG_APPEARANCE_NONE_FLAG: usize = 0x2f9;
const LINK_IS_NEAR_MOVEABLE_STATUE: usize = 0x2fa;
const PLAYER_HANDLER_TIMER: usize = 0x300;
const OVERWORLD_MUSIC: usize = 0x15b00;
// NES_Ver2: PKYNOT, player key-not flag; Rust call sites use it to gate pit correction.
const PIT_CORRECTION_ACTIVE_FLAG: usize = 0x302;
const CURRENT_ITEM_Y: usize = 0x303;
const CURRENT_ITEM_ACTIVE: usize = 0x304;
const EQ_SELECTED_ROD: usize = 0x307;
const CACHED_TILE_ACTION_INDEX: usize = 0x306;
const DUNG_FLOOR_Y_VEL: usize = 0x310;
const DUNG_FLOOR_X_VEL: usize = 0x312;
const OVERWORLD_SCREEN_TRANS_DIR_BITS: usize = 0x410;
const OVERWORLD_SCREEN_TRANS_DIR_BITS2: usize = 0x416;
const OVERWORLD_SCREEN_TRANSITION: usize = 0x418;
const LINK_IS_ON_LOWER_LEVEL_MIRROR: usize = 0x476;
// NES_Ver2: PYDMMD/PYDMFM, Y-button action mode and frame counter.
const Y_BUTTON_ACTION_STEP: usize = 0x30a;
const Y_BUTTON_ACTION_TIMER: usize = 0x30b;
const STATE_FOR_SPIN_ATTACK: usize = 0x31c;
const STEP_COUNTER_FOR_SPIN_ATTACK: usize = 0x31d;
const SPIN_ATTACK_SOUND_LATCH: usize = 0x324;
const LINK_SPIN_OFFSETS: usize = 0x31e;
const COUNTDOWN_FOR_BLINK: usize = 0x31f;
const RELATED_TO_MOVING_FLOOR_Y: usize = 0x318;
const RELATED_TO_MOVING_FLOOR_X: usize = 0x31a;
const LINK_DIRECTION_FACING_MIRROR: usize = 0x323;
// NES_Ver2 swim RAM block: frame counter, mode, active flag, max speed, direction, acceleration.
const SWIM_STROKE_FRAME_COUNTER: usize = 0x326;
const LINK_MAYBE_SWIM_FASTER: usize = 0x32a;
const DUNGEON_TORCH_ATTR: usize = 0x333;
const TILEDETECT_DEEPWATER: usize = 0x341;
const TILEDETECT_NORMAL_TILES: usize = 0x343;
const LINK_IS_IN_DEEP_WATER: usize = 0x345;
const LINK_PALETTE_BITS_OF_OAM: usize = 0x346;
const LINK_FLAG_MOVING: usize = 0x34a;
const FLAG_IS_SPRITE_TO_PICK_UP: usize = 0x314;
const LINK_SWIM_HARD_STROKE: usize = 0x34f;
const SORT_SPRITES_OFFSET_INTO_OAM_BUFFER: usize = 0x352;
const VALUE_COMPUTED_FOR_PLAYER_OAM: usize = 0x354;
const OAM_PRIORITY_VALUE_2: usize = 0x35d;
const LINK_DEBUG_VALUE_2: usize = 0x350;
const FLAG_FOR_BOOMERANG_IN_PLACE: usize = 0x35f;
const LINK_ELECTROCUTE_ON_TOUCH: usize = 0x360;
const LINK_ACTUAL_VEL_Z_MIRROR: usize = 0x362;
const LINK_ACTUAL_VEL_Z_COPY_MIRROR: usize = 0x363;
const LINK_Z_COORD_MIRROR: usize = 0x364;
const LIFTABLE_TILE_ACTION_INDEX_SECONDARY: usize = 0x369;
const TILEDETECT_THICK_GRASS: usize = 0x357;
const LINK_ACTUAL_VEL_Z_COPY: usize = 0x2c7;
const LINK_RECOILMODE_TIMER: usize = 0x2c6;
const LIFTABLE_TILE_ACTION_INDEX_PRIMARY: usize = 0x368;
const LINK_TIMER_PUSH_GET_TIRED: usize = 0x371;
const LINK_TIMER_JUMP_LEDGE: usize = 0x375;
const LINK_COUNTDOWN_FOR_DASH: usize = 0x374;
const PLAYER_SLEEP_IN_BED_STATE: usize = 0x37c;
const LINK_POSE_DURING_OPENING: usize = 0x37d;
const LINK_DASH_CTR: usize = 0x2f1;
const LINK_GIVE_DAMAGE: usize = 0x373;
// NES_Ver2: HIKUFG, "pull set flag".
const LINK_PULL_ACTION_STATE: usize = 0x377;
const TILE_ACTION_INDEX: usize = 0x36c;
const TILEDETECT_VERTICAL_LEDGE: usize = 0x36d;
const LIFTABLE_TILE_DETECTED_INDEX_DOUBLED: usize = 0x36a;
const DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ: usize = 0x36e;
const PLAYER_POSE_DRAW_COUNTER: usize = 0x379;
const LINK_DISABLE_SPRITE_DAMAGE: usize = 0x37b;
const ANCILLA_K: usize = 0x380;
const ANCILLA_L: usize = 0x385;
const ANCILLA_A: usize = 0x38a;
const ANCILLA_B: usize = 0x38f;
const ANCILLA_G: usize = 0x394;
const LINK_SOMETHING_WITH_HOOKSHOT: usize = 0x3e9;
const LINK_FORCE_HOLD_SWORD_UP: usize = 0x3ef;
const FLUTE_COUNTDOWN: usize = 0x3f0;
// NES_Ver2: BELFLG, moving-floor BG check flags.
const MOVING_FLOOR_BG_CHECK_FLAGS: usize = 0x3f1;
// NES_Ver2: BOGNTM, hookshot/bowgun BG check-off timer.
const HOOKSHOT_BG_CHECK_OFF_TIMER: usize = 0x3f9;
const LINK_ON_CONVEYOR_BELT: usize = 0x3f3;
const SOMARIA_BLOCK_BG_CHECK_FLAG: usize = 0x3f4;
const TILE_COLL_FLAG: usize = 0x315;
const TILE_COLLISION_BITS_PRIMARY: usize = 0x316;
const TILE_COLLISION_BITS_SECONDARY: usize = 0x317;
const DUNG_HDR_COLLISION: usize = 0x46c;
const LINK_TIMER_TEMPBUNNY: usize = 0x3f5;
const LINK_NEED_FOR_POOF_FOR_TRANSFORM: usize = 0x3f7;
const LINK_NEED_FOR_PULLFORRUPEES_SPRITE: usize = 0x3f8;
const BIT9_OF_XCOORD: usize = 0x3fa;
const IS_ARCHER_OR_SHOVEL_GAME: usize = 0x3fc;
const PLAYER_SPECIAL_DRAW_FLAG: usize = 0x3fd;
const DUNG_SAVEGAME_STATE_BITS: usize = 0x402;
const DUNG_QUADRANTS_VISITED: usize = 0x408;
const DUNG_LAYOUT_AND_STARTING_QUADRANT: usize = 0x40e;
// NES_Ver2: BG1MBF, "BG.1 move calc. buffer".
const BG1_MOVE_CALC_BUFFER: usize = 0x41c;
const DUNG_CUR_DOOR_IDX: usize = 0x460;
const DUNG_DOOR_OPENED: usize = 0x400;
const INVISIBLE_DOOR_DIR_AND_INDEX_X2: usize = 0x436;
const DUNG_FLOOR_X_OFFS: usize = 0x422;
const DUNG_FLOOR_Y_OFFS: usize = 0x424;
const DUNG_HDR_COLLISION_2_MIRROR: usize = 0x428;
const DUNGEON_ROOM_INDEX2: usize = 0x48e;
const OVERWORLD_HOLE_TILEMAP_POS: usize = 0x4b2;
const GANON_TORCH_COUNT: usize = 0x4c5;
const SUPER_BOMB_INDICATOR_TIMER: usize = 0x4b4;
const SUPER_BOMB_INDICATOR_COUNTER: usize = 0x4b5;
const CUR_PALACE_INDEX_X2: usize = 0x40c;
const DUNG_HDR_BG2_PROPERTIES: usize = 0x414;
const HDR_DUNGEON_DARK_WITH_LANTERN: usize = 0x458;
const DUNG_MISC_OBJS_INDEX: usize = 0x42c;
const DUNG_INDEX_OF_TORCHES: usize = 0x42e;
const DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS: usize = 0x43a;
const KIND_OF_IN_ROOM_STAIRCASE: usize = 0x44a;
const DUNG_NUM_LIT_TORCHES: usize = 0x45a;
const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x45c;
// NES_Ver2: CWLFLG, crush-wall check/progress flag.
const CRUSH_WALL_PROGRESS: usize = 0x454;
const DUNG_FLOOR_2_FILLER_TILES: usize = 0x46a;
const DUNG_FLOOR_1_FILLER_TILES: usize = 0x490;
const ABOUT_TO_JUMP_OFF_LEDGE: usize = 0x47a;
const NUM_MEMORIZED_TILES: usize = 0x4ac;
// NES_Ver2: RESTSFG, restart check flag.
const RESTART_CHECK_FLAG: usize = 0x04aa;
const HUD_FLOOR_CHANGED_TIMER: usize = 0x04a0;
const FLAG_SKIP_CALL_TAG_ROUTINES: usize = 0x4c7;
const LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP: usize = 0x04ca;
const DUNG_LOADE_BGOFFS_H_COPY: usize = 0x62c;
const DUNG_LOADE_BGOFFS_V_COPY: usize = 0x62e;
// NES_Ver2 WN* window/iris work RAM: X center, Y buffer, radius, and wipe state.
const SPOTLIGHT_WINDOW_X_CENTER: usize = 0x670;
const SPOTLIGHT_Y_LOWER: usize = 0x674;
const SPOTLIGHT_Y_UPPER: usize = 0x676;
const SPOTLIGHT_WINDOW_Y_BUFFER: usize = 0x67a;
const SPOTLIGHT_WINDOW_RADIUS: usize = 0x67c;
const SPOTLIGHT_WINDOW_STATE: usize = 0x67e;
const OVERWORLD_OFFSET_BASE_Y: usize = 0x708;
const OVERWORLD_OFFSET_MASK_Y: usize = 0x70a;
const OVERWORLD_OFFSET_BASE_X: usize = 0x70c;
const OVERWORLD_OFFSET_MASK_X: usize = 0x70e;
const SPRITE_LIMIT_INSTANCE: usize = 0x0b6a;
const SPRITE_STUNNED: usize = 0x0b58;
const LINK_PREVENT_FROM_MOVING: usize = 0x0b7b;
const DRAG_PLAYER_X: usize = 0x0b7c;
const DRAG_PLAYER_Y: usize = 0x0b7e;
const DUNGEON_ROOM_HISTORY: usize = 0x0b80;
const ARCHERY_GAME_HIT_COUNTER: usize = 0x0b88;
const ITEM_DROP_COUNTER: usize = 0x0b9b;
const ENHANCED_FEATURES0: usize = 0x064c;
const RAM_BUGS_FIXED: usize = 0x064a;
const DUNG_FLAG_SOMARIA_BLOCK_SWITCH: usize = 0x646;
const BUGFIX_POLY_RENDERER: u8 = 1;
const BUGFIX_LATEST: u8 = 1;
const FEATURES0_SKIP_INTRO_ON_KEYPRESS: u32 = 128;
const SPRITE_ROOM_ORIGIN_X_HI: usize = 0x0fb0;
const SPRITE_SHARED_WORK_A: usize = 0x0fb6;
const FLAG_BLOCK_LINK_MENU: usize = 0x0ffc;
const SPRCOLL_X_SIZE: usize = 0x0fb8;
const SPRCOLL_Y_SIZE: usize = 0x0fba;
const SPRITE_CHR_HALFSLOT_STATE: usize = 0x0fc6;
const LINK_X_COORD_PREV: usize = 0x0fc2;
const LINK_Y_COORD_PREV: usize = 0x0fc4;
const SPRITE_ALERT_FLAG: usize = 0x0fdc;
const HAUNTED_GROVE_FLUTE_EVENT_LATCH: usize = 0x0fdd;
const OVERWORLD_BOULDER_TRAP_COUNT: usize = 0x0ffd;
const OVERWORLD_BOULDER_TRAP_TIMER: usize = 0x0ffe;
const ALT_SPRITES_FLAG: usize = 0x0ffa;
const CUR_OBJECT_INDEX: usize = 0x0fa0;
const ARCHERY_GAME_ARROWS_LEFT: usize = 0x0b99;
const ARCHERY_GAME_OUT_OF_ARROWS: usize = 0x0b9a;
const PUSHEDBLOCKS_X_HI: usize = 0x5e0;
const PUSHEDBLOCKS_X_LO: usize = 0x5e4;
const PUSHEDBLOCKS_TARGET: usize = 0x5e8;
const PUSHEDBLOCKS_Y_HI: usize = 0x5ec;
const PUSHEDBLOCKS_Y_LO: usize = 0x5f0;
const PUSHEDBLOCKS_SUBPIXEL: usize = 0x5f4;
const INDEX_OF_CHANGABLE_DUNGEON_OBJS: usize = 0x5fc;
const OAM_ALLOC_ARR1: usize = 0x0fec;
const ANCILLA_OBJPRIO: usize = 0x280;
const ANCILLA_U: usize = 0x28a;
const ANCILLA_Z_VEL: usize = 0x294;
const ANCILLA_Z: usize = 0x29e;
const ANCILLA_AUX_TIMER: usize = 0x3b1;
const ANCILLA_H: usize = 0x3c5;
const ANCILLA_FLOOR2: usize = 0x3ca;
const ANCILLA_Y_LO: usize = 0x0bfa;
const ANCILLA_X_LO: usize = 0x0c04;
const ANCILLA_Y_HI: usize = 0x0c0e;
const ANCILLA_X_HI: usize = 0x0c18;
const ANCILLA_Y_VEL: usize = 0x0c22;
const ANCILLA_X_VEL: usize = 0x0c2c;
const ANCILLA_Y_SUBPIXEL: usize = 0x0c36;
const ANCILLA_X_SUBPIXEL: usize = 0x0c40;
const ANCILLA_STEP: usize = 0x0c54;
const ANCILLA_ITEM_TO_LINK: usize = 0x0c5e;
const ANCILLA_TIMER: usize = 0x0c68;
const ANCILLA_DIR: usize = 0x0c72;
const ANCILLA_FLOOR: usize = 0x0c7c;
const ANCILLA_NUMSPR: usize = 0x0c90;
const TAGALONG_Y_LO: usize = 0x1a00;
const TAGALONG_Y_HI: usize = 0x1a14;
const TAGALONG_X_LO: usize = 0x1a28;
const TAGALONG_X_HI: usize = 0x1a3c;
const TAGALONG_LAYERBITS: usize = 0x1a64;
const SPRITE_WHERE_IN_ROOM: usize = 0x1df80;
const OVERWORLD_SPRITE_WAS_LOADED: usize = 0x1ef80;
const DUNG_INDEX_OF_TORCHES_START: usize = 0x478;
const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS: usize = 0x47e;
const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS: usize = 0x480;
const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2: usize = 0x482;
const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2: usize = 0x484;
const DUNG_NUM_CHESTS_X2: usize = 0x496;
const DUNG_NUM_BIGKEY_LOCKS_X2: usize = 0x498;
const DUNG_OVERLAY_TO_LOAD: usize = 0x4ba;
const DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS: usize = 0x4a2;
const DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS: usize = 0x4a4;
const DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS: usize = 0x4a6;
const DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS: usize = 0x4a8;
const DUNG_OBJECT_POS_IN_OBJDATA: usize = 0x520;
const DUNG_OBJECT_TILEMAP_POS: usize = 0x540;
const REPLACEMENT_TILEMAP_UL: usize = 0x560;
const REPLACEMENT_TILEMAP_LL: usize = 0x580;
const REPLACEMENT_TILEMAP_UR: usize = 0x5a0;
const REPLACEMENT_TILEMAP_LR: usize = 0x5c0;
const DUNG_INTER_STARCASES: usize = 0x6b0;
const DUNG_STAIRS_TABLE_1: usize = 0x6b8;
const DUNG_CHEST_LOCATIONS: usize = 0x6e0;
const MAIN_TILE_THEME_INDEX: usize = 0x0aa1;
const AUX_TILE_THEME_INDEX: usize = 0x0aa2;
const SPRITE_GRAPHICS_INDEX: usize = 0x0aa3;
const MISC_SPRITES_GRAPHICS_INDEX: usize = 0x0aa4;
const HUD_CUR_ITEM: usize = 0x0202;
const HUD_MODULE_TICK_COUNTER: usize = 0x0206;
const TIMER_FOR_FLASHING_CIRCLE: usize = 0x0207;
const ANIMATE_HEART_REFILL_COUNTDOWN: usize = 0x0208;
const HUD_CUR_ITEM_X: usize = 0x0656;
const HUD_CUR_ITEM_L: usize = 0x0657;
const HUD_CUR_ITEM_R: usize = 0x0658;
const HUD_TMP1: usize = 0x0bd;
const BOTTLE_MENU_EXPAND_ROW: usize = 0x0205;
const ANIMATE_HEART_REFILL_COUNTDOWN_SUBPOS: usize = 0x0209;
const IS_DOING_HEART_ANIMATION: usize = 0x020a;
const EQUIPMENT_MENU_EXIT_STATE: usize = 0x034b;
const OVERWORLD_PALETTE_AUX1_BP2TO4_HI: usize = 0x0ab4;
const PALETTE_MAIN_INDOORS_COPY: usize = 0x0ab7;
const EXTENDED_OAM: usize = 0x0a00;
const LINK_ITEM_BOW: usize = 0x0f340;
const LINK_ITEM_BOOMERANG: usize = 0x0f341;
const LINK_ITEM_HOOKSHOT: usize = 0x0f342;
const LINK_ITEM_BOMBS: usize = 0x0f343;
const LINK_ITEM_MUSHROOM: usize = 0x0f344;
const LINK_ITEM_FIRE_ROD: usize = 0x0f345;
const LINK_ITEM_ICE_ROD: usize = 0x0f346;
const LINK_ITEM_BOMBOS: usize = 0x0f347;
const LINK_ITEM_ETHER: usize = 0x0f348;
const LINK_ITEM_QUAKE: usize = 0x0f349;
const LINK_ITEM_TORCH: usize = 0x0f34a;
const LINK_ITEM_HAMMER: usize = 0x0f34b;
const LINK_ITEM_FLUTE: usize = 0x0f34c;
const LINK_ITEM_BUG_NET: usize = 0x0f34d;
const LINK_ITEM_BOOK: usize = 0x0f34e;
const LINK_ITEM_BOTTLE_INDEX: usize = 0x0f34f;
const LINK_ITEM_CANE_SOMARIA: usize = 0x0f350;
const LINK_ITEM_CANE_BYRNA: usize = 0x0f351;
const LINK_ITEM_BOTTLE_INFO: usize = 0x0f35c;
const LINK_ITEM_FLIPPERS: usize = 0x0f356;
const LINK_ITEM_GLOVES: usize = 0x0f354;
const LINK_ITEM_BOOTS: usize = 0x0f355;
const LINK_ITEM_CAPE: usize = 0x0f352;
const LINK_ITEM_MIRROR: usize = 0x0f353;
const LINK_ITEM_MOON_PEARL: usize = 0x0f357;
const SRAM_PROGRESS_INDICATOR: usize = 0x0f3c5;
const SRAM_PROGRESS_FLAGS: usize = 0x0f3c6;
const WHICH_STARTING_POINT: usize = 0x0f3c8;
const SAVEGAME_IS_DARKWORLD: usize = 0x0f3ca;
const LINK_SWORD_TYPE: usize = 0x0f359;
const LINK_SHIELD_TYPE: usize = 0x0f35a;
const LINK_BOTTLE_INFO: usize = 0x0f35c;
const LINK_RUPEES_GOAL: usize = 0x0f360;
const LINK_RUPEES_ACTUAL: usize = 0x0f362;
const LINK_HEART_PIECES: usize = 0x0f36b;
const LINK_HEALTH_CAPACITY: usize = 0x0f36c;
const LINK_HEALTH_CURRENT: usize = 0x0f36d;
const LINK_MAGIC_POWER: usize = 0x0f36e;
const LINK_NUM_KEYS: usize = 0x0f36f;
const LINK_BOMB_UPGRADES: usize = 0x0f370;
const LINK_ARROW_UPGRADES: usize = 0x0f371;
const LINK_HEARTS_FILLER: usize = 0x0f372;
const LINK_MAGIC_FILLER: usize = 0x0f373;
const LINK_WHICH_PENDANTS: usize = 0x0f374;
const LINK_BOMB_FILLER: usize = 0x0f375;
const LINK_ARROW_REFILL_COUNTER: usize = 0x0f376;
const LINK_NUM_ARROWS: usize = 0x0f377;
const LINK_MAGIC_CONSUMPTION: usize = 0x0f37b;
const LINK_HAS_CRYSTALS: usize = 0x0f37a;
const NUMBER_OF_TIMES_HURT_BY_SPRITES: usize = 0x0cfc;
const LINK_ARMOR: usize = 0x0f35b;
const SAVE_DUNG_INFO: usize = 0x0f000;
const LINK_KEYS_EARNED_PER_DUNGEON: usize = 0x0f37c;
const LINK_COMPASS: usize = 0x0f364;
const LINK_BIGKEY: usize = 0x0f366;
const LINK_DUNGEON_MAP: usize = 0x0f368;
const OVERWORLD_SPRITE_GFX: usize = 0x0fcc0;
const OVERWORLD_SPRITE_PALETTES: usize = 0x0fd40;
const ATTRIBUTES_FOR_TILE: usize = 0x0fe00;
const ENEMY_DAMAGE_DATA: usize = 0x16000;
const VWF_TILE_BUFFER: usize = 0x1300;
const PEG_TILE_GFX_BUFFER: usize = 0xb340;
const ATTRACT_LEGEND_FLAG: usize = 0x27;
const ATTRACT_PRISON_ZELDA_Y_BASE: usize = 0x2b;
const ATTRACT_VRAM_DST: usize = 0x30;
const ATTRACT_ANIM_STEP_COUNTER: usize = 0x32;
const ATTRACT_SOLDIER_ANIM_STEP: usize = 0x33;
// Reuses NES_Ver2 SPYPS as the low byte of a prison soldier X sentinel.
const ATTRACT_PRISON_SOLDIER_X_LO: usize = 0x34;
const ATTRACT_SCENE_FRAME_COUNTER: usize = 0x50;
const ATTRACT_SCENE_DONE_FLAG: usize = 0x5d;
const ATTRACT_LEGEND_CTR: usize = 0x200;
const ATTRACT_BG2_VOFS_BACKUP: usize = 0x20;
const ATTRACT_THRONE_FADE_TIMER: usize = 0x2c;
const ATTRACT_FADE_IN_COMPLETE_FLAG: usize = 0x52;
const ATTRACT_FADE_IN_DONE_FLAG: usize = 0x5f;
const ATTRACT_SUBSTEP_DELAY_COUNTER: usize = 0x61;
const ATTRACT_MAIDEN_WARP_TIMER_A: usize = 0x62;
const ATTRACT_MAIDEN_WARP_TIMER_B: usize = 0x63;
const OVERWORLD_MAP_STATE: usize = 0x200;
const LINK_DEBUG_VALUE_1: usize = 0x20b;
const HUD_INVENTORY_ORDER: usize = 0x0225;
// NES_Ver2: OPTHPT/OPTBPT, option head/body DMA pointers.
const SPRITE_N: usize = 0x0bc0;
const RAW_SFX_PAN_VALUE: usize = 0x0cf8;
const RUPEE_SFX_SOUND_DELAY: usize = 0x0cfd;
const OVERWORLD_TILE_THEME_INDEX: usize = 0x0aa0;
const SPRITE_FLAGS5: usize = 0x0be0;
const ANCILLA_OAM_IDX: usize = 0x0c86;
const SPRITE_ROOM: usize = 0x0c9a;
const SPRITE_DEFL_BITS: usize = 0x0caa;
const SPRITE_DIE_ACTION: usize = 0x0cba;
const SPRITE_Y_LO: usize = 0x0d00;
const SPRITE_X_LO: usize = 0x0d10;
const SPRITE_Y_HI: usize = 0x0d20;
const SPRITE_X_HI: usize = 0x0d30;
const SPRITE_Y_VEL: usize = 0x0d40;
const SPRITE_X_VEL: usize = 0x0d50;
const SPRITE_Y_SUBPIXEL: usize = 0x0d60;
const SPRITE_X_SUBPIXEL: usize = 0x0d70;
const SPRITE_AI_STATE: usize = 0x0d80;
const SPRITE_A: usize = 0x0d90;
const SPRITE_B: usize = 0x0da0;
const SPRITE_C: usize = 0x0db0;
const SPRITE_OBJ_PRIO: usize = 0x0b89;
const SPRITE_GRAPHICS: usize = 0x0dc0;
const SPRITE_STATE: usize = 0x0dd0;
const SPRITE_D: usize = 0x0de0;
const SPRITE_DELAY_MAIN: usize = 0x0df0;
const SPRITE_DELAY_AUX1: usize = 0x0e00;
const SPRITE_IGNORE_PROJECTILE: usize = 0x0ba0;
const SPRITE_SUBTYPE: usize = 0x0e30;
const SPRITE_TYPE: usize = 0x0e20;
const SPRITE_FLAGS2: usize = 0x0e40;
const SPRITE_FLAGS3: usize = 0x0e60;
const SPRITE_SUBTYPE2: usize = 0x0e80;
const SPRITE_E: usize = 0x0e90;
const SPRITE_HEAD_DIR: usize = 0x0eb0;
const SPRITE_PAUSE: usize = 0x0f00;
const SPRITE_DELAY_AUX2: usize = 0x0e10;
const SPRITE_DELAY_AUX4: usize = 0x0f10;
const SPRITE_FLOOR: usize = 0x0f20;
const SPRITE_X_RECOIL: usize = 0x0f40;
const SPRITE_OAM_FLAGS: usize = 0x0f50;
const SPRITE_FLAGS4: usize = 0x0f60;
const SPRITE_Z: usize = 0x0f70;
const SPRITE_Z_VEL: usize = 0x0f80;
const SPRITE_Z_SUBPOS: usize = 0x0f90;
const SPRITE_F: usize = 0x0ea0;
const SPRITE_G: usize = 0x0ed0;
const SPRITE_FLAGS: usize = 0x0b6b;
const SPRITE_HEALTH: usize = 0x0e50;
const SPRITE_WALLCOLL: usize = 0x0e70;
const SPRITE_ANIM_CLOCK: usize = 0x0ec0;
const SPRITE_HIT_TIMER: usize = 0x0ef0;
const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
const OVERLORD_GEN1: usize = 0x0b28;
const OVERLORD_GEN2: usize = 0x0b30;
const REPULSESPARK_TIMER: usize = 0x0fac;
const REPULSESPARK_X_LO: usize = 0x0fad;
const REPULSESPARK_Y_LO: usize = 0x0fae;
const BLIND_HEAD_ANIM_COUNTER: usize = 0x0b69;
// NES_Ver2: MEMSTT, bird-travel status.
const BIRDTRAVEL_STATUS: usize = 0x1af0;
const RNG_SEED: usize = 0x0fa1;
const SPRITE_ROOM_ORIGIN_Y_HI: usize = 0x0fb1;
const CUR_SPRITE_X: usize = 0x0fd8;
const CUR_SPRITE_Y: usize = 0x0fda;
const GARNISH_TYPE: usize = 0x1f800;
const BEAMOS_X_HI: usize = 0x1fe00;
const LINK_DMA_SOURCE_OFFSET: usize = 0x0c00f;
const LINK_DMA_COUNTDOWN: usize = 0x0c013;
const LINK_DMA_TILE_OFFSET: usize = 0x0c015;
const OVERWORLD_FIXED_COLOR_PLUSMINUS: usize = 0x0c017;
const DUNG_WANT_LIGHTS_OUT: usize = 0x0c005;
const DUNG_WANT_LIGHTS_OUT_COPY: usize = 0x0c006;
const AGAHNIM_PAL_SETTING: usize = 0x0c019;
const TIMER_FOR_MODE7_ZOOM: usize = 0x637;
const MODE7_ZOOM_STEP_COUNTER: usize = 0x635;
const OVERWORLD_MAP_FLAGS: usize = 0x636;
const DEBUG_ROOM_BOUNDS_TOP: usize = 0x600;
const UP_DOWN_SCROLL_TARGET: usize = 0x610;
const UP_DOWN_SCROLL_TARGET_END: usize = 0x612;
const LEFT_RIGHT_SCROLL_TARGET: usize = 0x614;
const LEFT_RIGHT_SCROLL_TARGET_END: usize = 0x616;
const CAMERA_Y_COORD_SCROLL_LOW: usize = 0x618;
const CAMERA_Y_COORD_SCROLL_HI: usize = 0x61a;
const CAMERA_X_COORD_SCROLL_LOW: usize = 0x61c;
const CAMERA_X_COORD_SCROLL_HI: usize = 0x61e;
const DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED: usize = 0x641;
const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x642;
const LINK_Y_COORD_CACHED: usize = 0x0c184;
const LINK_X_COORD_CACHED: usize = 0x0c186;
const CACHED_ROOM_BOUNDS_Y_START: usize = 0x0c188;
const CACHED_ROOM_BOUNDS_Y_END: usize = 0x0c18a;
const CACHED_ROOM_BOUNDS_X_START: usize = 0x0c18c;
const CACHED_ROOM_BOUNDS_X_END: usize = 0x0c18e;
const UP_DOWN_SCROLL_TARGET_CACHED: usize = 0x0c190;
const UP_DOWN_SCROLL_TARGET_END_CACHED: usize = 0x0c192;
const LEFT_RIGHT_SCROLL_TARGET_CACHED: usize = 0x0c194;
const LEFT_RIGHT_SCROLL_TARGET_END_CACHED: usize = 0x0c196;
const CAMERA_Y_COORD_SCROLL_LOW_CACHED: usize = 0x0c198;
const CAMERA_X_COORD_SCROLL_LOW_CACHED: usize = 0x0c19a;
const QUADRANT_FULLSIZE_X_CACHED: usize = 0x0c19c;
const QUADRANT_FULLSIZE_Y_CACHED: usize = 0x0c19d;
const LINK_QUADRANT_X_CACHED: usize = 0x0c19e;
const LINK_QUADRANT_Y_CACHED: usize = 0x0c19f;
const LINK_DIRECTION_FACING_CACHED: usize = 0x0c1a6;
const LINK_IS_ON_LOWER_LEVEL_CACHED: usize = 0x0c1a7;
const LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED: usize = 0x0c1a8;
const IS_STANDING_IN_DOORWAY_CACHED: usize = 0x0c1a9;
const DUNG_CUR_FLOOR_CACHED: usize = 0x0c1aa;
const OVERWORLD_EXIT_TILE_THEME_INDEX: usize = 0x0c164;
const OVERWORLD_PAL_MAIN_INDOORS_BACKUP: usize = 0x0c20a;
const OVERWORLD_PAL_AUX3_BP7_BACKUP: usize = 0x0c20b;
const OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP: usize = 0x0c20c;
const OW_ENTRANCE_VALUE: usize = 0x696;
const DOOR_OPEN_CLOSED_COUNTER: usize = 0x692;
const BIG_ROCK_STARTING_ADDRESS: usize = 0x698;
const DOOR_DEBRIS_X: usize = 0x3b6;
const DOOR_DEBRIS_Y: usize = 0x3ba;
const DUNG_HDR_HOLE_TELEPORTER_PLANE: usize = 0x63c;
const DUNG_DOOR_OPENED_INCL_ADJACENT: usize = 0x68c;
const DUNGEON_TRAP_TRIGGER_LATCH: usize = 0x0b9e;
const ORANGE_BLUE_BARRIER_STATE: usize = 0x0c172;
const AUX_BG_SUBSET_0: usize = 0x0c2f8;
const AUX_BG_SUBSET_1: usize = 0x0c2f9;
const AUX_BG_SUBSET_2: usize = 0x0c2fa;
const AUX_BG_SUBSET_3: usize = 0x0c2fb;
const SPRITE_GFX_SUBSET_0: usize = 0x0c2fc;
const SPRITE_GFX_SUBSET_1: usize = 0x0c2fd;
const SPRITE_GFX_SUBSET_2: usize = 0x0c2fe;
const SPRITE_GFX_SUBSET_3: usize = 0x0c2ff;
const AUX_PALETTE_BUFFER: usize = 0x0c300;
const MAIN_PALETTE_BUFFER: usize = 0x0c500;
const HUD_TILE_INDICES_BUFFER: usize = 0x0c700;
const OAM_BUF: usize = 0x0800;
const BYTEWISE_EXTENDED_OAM: usize = 0x0a20;
const LINK_ABILITY_FLAGS: usize = 0xf379;
const SAVEGAME_MAP_ICONS_INDICATOR: usize = 0x0f3c7;
const SELECTED_SAVE_SLOT_X2: usize = 0x1ffe;
const TEXT_DIALOGUE_POINTERS: usize = 0x171c0;
const FOLLOWER_INDICATOR: usize = 0x0f3cc;
const FOLLOWER_DROPPED: usize = 0x0f3d3;
const DUNG_HDR_TRAVEL_DESTINATIONS: usize = 0x0c000;
const INTRO_STEP_TIMER: usize = 0x1e01;
const INTRO_SPRITE_ALLOC: usize = 0x1e08;
const POLY_CONFIG_COLOR_MODE: usize = 0x1f01;
const POLY_CONFIG1: usize = 0x1f02;
const POLY_WHICH_MODEL: usize = 0x1f03;
const POLY_A: usize = 0x1f04;
const POLY_B: usize = 0x1f05;
const POLY_BASE_X: usize = 0x1f06;
const POLY_BASE_Y: usize = 0x1f07;
const POLY_CONFIG_NUM_VERTEX: usize = 0x1f3f;
const POLY_CONFIG_NUM_POLYS: usize = 0x1f40;
const POLY_FROMLUT_PTR2: usize = 0x1f41;
const POLY_FROMLUT_PTR4: usize = 0x1f43;
const POLY_FROMLUT_Z: usize = 0x1f45;
const POLY_FROMLUT_Y: usize = 0x1f46;
const POLY_FROMLUT_X: usize = 0x1f47;
const POLY_F0: usize = 0x1f48;
const POLY_F1: usize = 0x1f4a;
const POLY_F2: usize = 0x1f4c;
const POLY_NUM_VERTEX_IN_POLY: usize = 0x1f4e;
const POLY_RASTER_COLOR_CONFIG: usize = 0x1f4f;
const POLY_SIN_A: usize = 0x1f50;
const POLY_COS_A: usize = 0x1f52;
const POLY_SIN_B: usize = 0x1f54;
const POLY_COS_B: usize = 0x1f56;
const POLY_E0: usize = 0x1f58;
const POLY_E2: usize = 0x1f5a;
const POLY_E3: usize = 0x1f5c;
const POLY_E1: usize = 0x1f5e;
const POLY_TMP0: usize = 0x1fb0;
const POLY_TMP1: usize = 0x1fb2;
const POLY_RASTER_COLOR0: usize = 0x1fb5;
const POLY_RASTER_COLOR1: usize = 0x1fb7;
const POLY_RASTER_DST_PTR: usize = 0x1fb9;
const POLY_TMP2: usize = 0x1fbc;
const POLY_XY_COORDS: usize = 0x1fc0;
const POLY_TOTAL_NUM_STEPS: usize = 0x1fe0;
const POLY_X0_CUR: usize = 0x1fe1;
const POLY_Y0_CUR: usize = 0x1fe2;
const POLY_X0_TARGET: usize = 0x1fe3;
const POLY_Y0_TRIG: usize = 0x1fe4;
const POLY_X0_FRAC: usize = 0x1fe5;
const POLY_X0_STEP: usize = 0x1fe7;
const POLY_CUR_VERTEX_IDX0: usize = 0x1fe9;
const POLY_X1_CUR: usize = 0x1fea;
const POLY_Y1_CUR: usize = 0x1feb;
const POLY_X1_TARGET: usize = 0x1fec;
const POLY_Y1_TRIG: usize = 0x1fed;
const POLY_X1_FRAC: usize = 0x1fee;
const POLY_X1_STEP: usize = 0x1ff0;
const POLY_CUR_VERTEX_IDX1: usize = 0x1ff2;
const POLY_RASTER_NUMFULL: usize = 0x1ffa;
const POLYHEDRAL_BUFFER: usize = 0xe800;

const COMP_SPRITE_PTRS: [u32; 108] = [
    0x10f000, 0x10f600, 0x10fc00, 0x118200, 0x118800, 0x118e00, 0x119400, 0x119a00, 0x11a000,
    0x11a600, 0x11ac00, 0x11b200, 0x14fffc, 0x1585d4, 0x158ab6, 0x158fbe, 0x1593f8, 0x1599a6,
    0x159f32, 0x15a3d7, 0x15a8f1, 0x15aec6, 0x15b418, 0x15b947, 0x15bed0, 0x15c449, 0x15c975,
    0x15ce7c, 0x15d394, 0x15d8ac, 0x15ddc0, 0x15e34c, 0x15e8e8, 0x15ee31, 0x15f3a6, 0x15f92d,
    0x15feba, 0x1682ff, 0x1688e0, 0x168e41, 0x1692df, 0x169883, 0x169cd0, 0x16a26e, 0x16a275,
    0x16a787, 0x16aa06, 0x16ae9d, 0x16b3ff, 0x16b87e, 0x16be6b, 0x16c13d, 0x16c619, 0x16cbbb,
    0x16d0f1, 0x16d641, 0x16d95a, 0x16dd99, 0x16e278, 0x16e760, 0x16ed25, 0x16f20f, 0x16f6b7,
    0x16fa5f, 0x16fd29, 0x1781cd, 0x17868d, 0x178b62, 0x178fd5, 0x179527, 0x17994b, 0x179ea7,
    0x17a30e, 0x17a805, 0x17acf8, 0x17b2a2, 0x17b7f9, 0x17bc93, 0x17c237, 0x17c78e, 0x17cd55,
    0x17d2bc, 0x17d82f, 0x17dcec, 0x17e1cc, 0x17e36b, 0x17e842, 0x17eb38, 0x17ed58, 0x17f06c,
    0x17f4fd, 0x17fa39, 0x17ff86, 0x18845c, 0x1889a1, 0x188d64, 0x18919d, 0x189610, 0x189857,
    0x189b24, 0x189dd2, 0x18a03f, 0x18a4ed, 0x18a7ba, 0x18aedf, 0x18af0d, 0x18b520, 0x18b953,
];
const GRAPHICS_HALF_SLOT_PACKS: [u8; 20] =
    [1, 1, 8, 8, 9, 9, 2, 2, 2, 2, 3, 3, 4, 4, 5, 5, 8, 8, 8, 8];

const PALETTE_MAIN_SPRITE_SNES_ADDR: u32 = 0x9bd218;
const PALETTE_ARMOR_AND_GLOVES_SNES_ADDR: u32 = 0x9bd308;
const PALETTE_SPRITE_AUX3_SNES_ADDR: u32 = 0x9bd39e;
const PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR: u32 = 0x9bd446;
const PALETTE_SPRITE_AUX1_SNES_ADDR: u32 = 0x9bd4e0;
const PALETTE_SWORD_SNES_ADDR: u32 = 0x9bd630;
const PALETTE_SHIELD_SNES_ADDR: u32 = 0x9bd648;
const PALETTE_DUNGEON_BG_MAIN_SNES_ADDR: u32 = 0x9bd734;
const PALETTE_PALACE_MAP_SPRITE_SNES_ADDR: u32 = 0x9bd70a;
const PALETTE_PALACE_MAP_BG_SNES_ADDR: u32 = 0x9be544;
const PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR: u32 = 0x9be6c8;
const PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR: u32 = 0x9be86c;
const PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR: u32 = 0x9be604;
const HUD_PALETTE_SNES_ADDR: u32 = 0x9bd660;

const DUNGEON_DRAW_OBJECT_OFFSETS_BG1: [u8; 33] = [
    0, 0x20, 0x7e, 2, 0x20, 0x7e, 4, 0x20, 0x7e, 6, 0x20, 0x7e, 0x80, 0x20, 0x7e, 0x82, 0x20, 0x7e,
    0x84, 0x20, 0x7e, 0x86, 0x20, 0x7e, 0, 0x21, 0x7e, 0x80, 0x21, 0x7e, 0, 0x22, 0x7e,
];
const DUNGEON_DRAW_OBJECT_OFFSETS_BG2: [u8; 33] = [
    0, 0x40, 0x7e, 2, 0x40, 0x7e, 4, 0x40, 0x7e, 6, 0x40, 0x7e, 0x80, 0x40, 0x7e, 0x82, 0x40, 0x7e,
    0x84, 0x40, 0x7e, 0x86, 0x40, 0x7e, 0, 0x41, 0x7e, 0x80, 0x41, 0x7e, 0, 0x42, 0x7e,
];
const DUNGEON_QUADRANT_OFFSETS: [usize; 4] = [0x0000, 0x0040, 0x1000, 0x1040];
const DOOR_TYPE_AND_SLOT: usize = 0x1980;
const DUNG_DOOR_TILEMAP_ADDRESS: usize = 0x19a0;
const DUNG_DOOR_DIRECTION: usize = 0x19c0;
const DOOR_TYPE_REGULAR: u8 = 0;
const DOOR_TYPE_EXIT_TO_OW: u8 = 18;
const DOOR_TYPE_SHUTTERS_TWO_WAY: u8 = 24;
const DOOR_TYPE_THRONE_ROOM: u8 = 20;
const DOOR_TYPE_SLASHABLE: u8 = 50;
const DOOR_TYPE_36: u8 = 54;
const DOOR_TYPE_38: u8 = 56;
const DUNG_EXIT_DOOR_COUNT: usize = 0x19e0;
const DUNG_EXIT_DOOR_ADDRESSES: usize = 0x19e2;
const RESERVED_GFX_CONFIG_WORD: usize = 0x0aa6;
const DOOR_POSITION_UP: [u16; 12] = [
    0x21c, 0x23c, 0x25c, 0x39c, 0x3bc, 0x3dc, 0x121c, 0x123c, 0x125c, 0x139c, 0x13bc, 0x13dc,
];
const DOOR_POSITION_DOWN: [u16; 12] = [
    0xd1c, 0xd3c, 0xd5c, 0xb9c, 0xbbc, 0xbdc, 0x1d1c, 0x1d3c, 0x1d5c, 0x1b9c, 0x1bbc, 0x1bdc,
];
const DOOR_TYPE_SRC_UP: [u16; 52] = [
    0x2716, 0x272e, 0x272e, 0x2746, 0x2746, 0x2746, 0x2746, 0x2746, 0x2746, 0x275e, 0x275e, 0x275e,
    0x275e, 0x2776, 0x278e, 0x27a6, 0x27be, 0x27be, 0x27d6, 0x27d6, 0x27ee, 0x2806, 0x2806, 0x281e,
    0x2836, 0x2836, 0x2836, 0x2836, 0x284e, 0x2866, 0x2866, 0x2866, 0x2866, 0x287e, 0x2896, 0x28ae,
    0x28c6, 0x28de, 0x28f6, 0x28f6, 0x28f6, 0x290e, 0x2926, 0x2958, 0x2978, 0x2990, 0x2990, 0x2990,
    0x2990, 0x29a8, 0x29c0, 0x29d8,
];
const DOOR_TYPE_SRC_DOWN: [u16; 48] = [
    0x29f0, 0x2a08, 0x2a08, 0x2a20, 0x2a20, 0x2a20, 0x2a20, 0x2a20, 0x2a20, 0x2a38, 0x2a38, 0x2a38,
    0x2a38, 0x2a50, 0x2a68, 0x2a80, 0x2a98, 0x2a98, 0x2a98, 0x2a98, 0x2a98, 0x2ab0, 0x2ac8, 0x2ae0,
    0x2af8, 0x2af8, 0x2af8, 0x2af8, 0x2b10, 0x2b28, 0x2b28, 0x2b28, 0x2b28, 0x2b40, 0x2b58, 0x2b70,
    0x2b88, 0x2ba0, 0x2bb8, 0x2bb8, 0x2bb8, 0x2bd0, 0x2be8, 0x2c1a, 0x2c3a, 0x2c52, 0x2c6a, 0x2c6a,
];
const UPLOAD_BG_SRCS: [usize; 16] = [
    0x0000, 0x1000, 0x0000, 0x0040, 0x0040, 0x1040, 0x1000, 0x1040, 0x1000, 0x0000, 0x0040, 0x0000,
    0x1040, 0x0040, 0x1040, 0x1000,
];
const UPLOAD_BG_DSTS: [u8; 16] = [1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15, 4, 8, 12, 16];
const NMI_VRAM_ADDRS: [usize; 35] = [
    0, 0, 4, 8, 12, 8, 12, 0, 4, 0, 8, 4, 12, 4, 12, 0, 8, 16, 20, 24, 28, 24, 28, 16, 20, 16, 24,
    20, 28, 20, 28, 16, 24, 96, 104,
];

fn full_tilemap_nmi_vram_region(target_page: u8) -> Option<(usize, usize)> {
    let destination = *NMI_VRAM_ADDRS.get(usize::from(target_page))? << 8;
    let end = destination.checked_add(FULL_TILEMAP_NMI_WORDS)?;
    (end <= 0x8000).then_some((destination, FULL_TILEMAP_NMI_WORDS))
}

const ATTRACT_LEGEND_TILEMAP_BYTES_0: [u8; 158] = [
    0x61, 0x65, 0x40, 0x28, 0, 0x35, 0x61, 0x85, 0x40, 0x28, 0x10, 0x35, 0x61, 0xa5, 0, 0x29, 1,
    0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 3, 0x31, 3, 0x71,
    2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 2, 0x35, 1, 0x35, 0x61,
    0xc5, 0, 0x29, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11,
    0x35, 0x12, 0x35, 0x11, 0x35, 0x13, 0x35, 0x13, 0x75, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11,
    0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x12, 0x35, 0x11, 0x35, 0x61, 0xe5, 0,
    0x29, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21,
    0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21,
    0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x21, 0x35, 0x20, 0x35, 0x62, 5, 0x40, 0x28, 0, 0xb5,
    0xff, 0x61,
];

const ATTRACT_LEGEND_TILEMAP_BYTES_1: [u8; 238] = [
    0x61, 0x65, 0x40, 0x28, 0, 0x35, 0x61, 0x85, 0, 0x13, 0x10, 0x35, 0x4e, 0x75, 0x6e, 0x35, 0x10,
    0x35, 0x4e, 0x35, 0x10, 0x35, 0x4c, 0x35, 0x10, 0x35, 0x4e, 0x75, 0x49, 0x35, 0x61, 0x8f, 0x40,
    8, 0x10, 0x35, 0x61, 0x94, 0, 0x0b, 0x4e, 0x75, 0x6e, 0x35, 0x10, 0x35, 0x4e, 0x35, 0x10, 0x35,
    0x4c, 0x35, 0x61, 0xa5, 0, 0x29, 0x5f, 0x75, 0x5e, 0x75, 0x7e, 0x35, 0x7f, 0x35, 0x5e, 0x35,
    0x5f, 0x35, 0x4d, 0x35, 0x5f, 0x75, 0x5e, 0x75, 0x4a, 0x35, 0x4b, 0x35, 0x10, 0x35, 0x49, 0x75,
    0x10, 0x35, 0x5f, 0x75, 0x5e, 0x75, 0x7e, 0x35, 0x7f, 0x35, 0x5e, 0x35, 0x5f, 0x35, 0x4d, 0x35,
    0x61, 0xc5, 0, 0x29, 0x50, 0x35, 0x51, 0x35, 0x52, 0x35, 0x53, 0x35, 0x54, 0x35, 0x55, 0x35,
    0x56, 0x35, 0x57, 0x35, 0x58, 0x35, 0x59, 0x35, 0x5a, 0x35, 0x5b, 0x35, 0x5c, 0x35, 0x5d, 0x35,
    0x50, 0x35, 0x51, 0x35, 0x52, 0x35, 0x53, 0x35, 0x54, 0x35, 0x55, 0x35, 0x56, 0x35, 0x61, 0xe5,
    0, 0x29, 0x60, 0x35, 0x61, 0x35, 0x62, 0x35, 0x63, 0x35, 0x64, 0x35, 0x65, 0x35, 0x66, 0x35,
    0x67, 0x35, 0x68, 0x35, 0x69, 0x35, 0x6a, 0x35, 0x6b, 0x35, 0x6c, 0x35, 0x6d, 0x35, 0x60, 0x35,
    0x61, 0x35, 0x62, 0x35, 0x63, 0x35, 0x64, 0x35, 0x65, 0x35, 0x66, 0x35, 0x62, 5, 0, 0x29, 0x70,
    0x35, 0x71, 0x35, 0x72, 0x35, 0x73, 0x35, 0x74, 0x35, 0x75, 0x35, 0x76, 0x35, 0x77, 0x35, 0x78,
    0x35, 0x79, 0x35, 0x7a, 0x35, 0x7b, 0x35, 0x7c, 0x35, 0x7d, 0x35, 0x70, 0x35, 0x71, 0x35, 0x72,
    0x35, 0x73, 0x35, 0x74, 0x35, 0x75, 0x35, 0x76, 0x35, 0xff, 0x61,
];

const ATTRACT_LEGEND_TILEMAP_BYTES_2: [u8; 200] = [
    0x61, 0x65, 0x40, 0x28, 0, 0x35, 0x61, 0x85, 0x40, 0x28, 0x10, 0x35, 0x61, 0xa5, 0, 0x1d, 0x22,
    0x35, 0x23, 0x35, 0x10, 0x35, 0x22, 0x35, 0x23, 0x35, 0x10, 0x35, 0x22, 0x35, 0x23, 0x35, 0x10,
    0x35, 0x22, 0x35, 0x23, 0x35, 0x10, 0x35, 0x10, 0x75, 0x23, 0x75, 0x22, 0x75, 0x61, 0xb4, 0x40,
    6, 0x10, 0x35, 0x61, 0xb8, 0, 3, 0x23, 0x75, 0x22, 0x75, 0x61, 0xc5, 0, 0x29, 4, 0x35, 5, 0x35,
    6, 0x35, 4, 0x35, 5, 0x35, 6, 0x35, 4, 0x35, 5, 0x35, 6, 0x35, 4, 0x35, 5, 0x35, 6, 0x35, 6,
    0x75, 5, 0x75, 4, 0x75, 0x10, 0x75, 0x23, 0x75, 0x22, 0x75, 6, 0x75, 5, 0x75, 4, 0x75, 0x61,
    0xe5, 0, 0x29, 0x14, 0x35, 0x15, 0x35, 0x16, 0x35, 0x14, 0x35, 0x15, 0x35, 0x16, 0x35, 0x14,
    0x35, 0x15, 0x35, 0x16, 0x35, 0x14, 0x35, 0x15, 0x35, 0x16, 0x35, 0x16, 0x75, 0x15, 0x75, 0x14,
    0x75, 6, 0x75, 5, 0x75, 4, 0x75, 0x16, 0x75, 0x15, 0x75, 0x14, 0x75, 0x62, 5, 0, 0x29, 0x24,
    0x35, 0x25, 0x35, 0x26, 0x35, 0x24, 0x35, 0x25, 0x35, 0x26, 0x35, 0x24, 0x35, 0x25, 0x35, 0x26,
    0x35, 0x24, 0x35, 0x25, 0x35, 0x26, 0x35, 0x26, 0x75, 0x25, 0x75, 0x24, 0x75, 0x26, 0x75, 0x25,
    0x75, 0x24, 0x75, 0x26, 0x75, 0x25, 0x75, 0x24, 0x75, 0xff, 0x61,
];

const ATTRACT_LEGEND_TILEMAP_BYTES_3: [u8; 266] = [
    0x61, 0x65, 0, 0x29, 0, 0x35, 0, 0x35, 0x1b, 0x35, 0x30, 0x35, 0x31, 0x35, 0x32, 0x35, 0, 0x35,
    0, 0x35, 0, 0x35, 0x33, 0x35, 0x41, 0x35, 0x41, 0x75, 0x33, 0x75, 0, 0x75, 0, 0x75, 0, 0x75,
    0x32, 0x75, 0x31, 0x75, 0x30, 0x75, 0x1b, 0x75, 0, 0x75, 0x61, 0x85, 0x40, 0x1e, 0x10, 0x35,
    0x61, 0x86, 0, 9, 0x34, 0x35, 0x0b, 0x35, 0x40, 0x35, 0x41, 0x35, 0x42, 0x35, 0x61, 0x95, 0, 9,
    0x42, 0x75, 0x41, 0x75, 0x40, 0x75, 0x0b, 0x75, 0x34, 0x75, 0x61, 0xa5, 0, 0x29, 0x43, 0x35,
    0x44, 0x35, 7, 0x35, 8, 0x35, 9, 0x35, 0x0a, 0x35, 0x10, 0x35, 0x0c, 0x35, 0x0d, 0x35, 0x0e,
    0x35, 0x0f, 0x35, 0x0f, 0x75, 0x0e, 0x75, 0x0d, 0x75, 0x0c, 0x75, 0x10, 0x75, 0x0a, 0x75, 9,
    0x75, 8, 0x75, 7, 0x75, 0x44, 0x75, 0x61, 0xc5, 0, 0x29, 0x35, 0x35, 0x36, 0x35, 0x17, 0x35,
    0x18, 0x35, 0x19, 0x35, 0x1a, 0x35, 0x10, 0x35, 0x1c, 0x35, 0x1d, 0x35, 0x1e, 0x35, 0x1f, 0x35,
    0x1f, 0x75, 0x1e, 0x75, 0x1d, 0x75, 0x1c, 0x75, 0x10, 0x75, 0x1a, 0x75, 0x19, 0x75, 0x18, 0x75,
    0x17, 0x75, 0x36, 0x75, 0x61, 0xe5, 0, 0x29, 0x45, 0x35, 0x46, 0x35, 0x27, 0x35, 0x28, 0x35,
    0x29, 0x35, 0x2a, 0x35, 0x2b, 0x35, 0x2c, 0x35, 0x2d, 0x35, 0x2e, 0x35, 0x2f, 0x35, 0x2f, 0x75,
    0x2e, 0x75, 0x2d, 0x75, 0x2c, 0x75, 0x2b, 0x75, 0x2a, 0x75, 0x29, 0x75, 0x28, 0x75, 0x27, 0x75,
    0x46, 0x75, 0x62, 5, 0, 0x29, 0x47, 0x35, 0x48, 0x35, 0x37, 0x35, 0x38, 0x35, 0x39, 0x35, 0x3a,
    0x35, 0x3b, 0x35, 0x3c, 0x35, 0x3d, 0x35, 0x3e, 0x35, 0x3f, 0x35, 0x3f, 0x75, 0x3e, 0x75, 0x3d,
    0x75, 0x3c, 0x75, 0x3b, 0x75, 0x3a, 0x75, 0x39, 0x75, 0x38, 0x75, 0x37, 0x75, 0x48, 0x75, 0xff,
    0,
];

const SIMPLE_HDMA_B_ADR_OFFSETS: [[u8; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];
const SIMPLE_HDMA_TRANSFER_LENGTH: [usize; 8] = [1, 2, 2, 4, 4, 4, 2, 4];
const DMA_SAVELOAD_SLOT_SIZE: usize = snes::dma::DmaState::C_SAVELOAD_SIZE;
const PPU_SAVELOAD_SLOT_SIZE: usize = snes::ppu::PpuState::C_SAVELOAD_SIZE;
const APU_RAM_SAVELOAD_SIZE: usize = 0x10000;
const DSP_SAVELOAD_SIZE: usize = 3024;
const PPU_SIDE_SPACE_LIMIT: u16 = snes::consts::PPU_EXTRA_LEFT_RIGHT as u16;
const ATTRACT_BG_DMA_SETUP: [u8; 13] = [
    0x20, 0xff, 0x00, 0x50, 0x18, 0xe0, 0x50, 0x18, 0xe0, 1, 0xff, 0x00, 0,
];
const ATTRACT_TILEMAP_DMA_SETUP: [u8; 10] = [0x48, 0xff, 0x00, 0x30, 0x30, 0xd8, 1, 0xff, 0x00, 0];
const ENDING_HDMA_SETUP: [u8; 19] = [
    0x52, 0x00, 0x06, 8, 0xe2, 0x00, 8, 0x02, 0x06, 5, 0x04, 0x06, 0x10, 0x06, 0x06, 0x81, 0xe2,
    0x00, 0,
];
const SPOTLIGHT_INDIRECT_HDMA_SETUP: [u8; 7] = [0xf8, 0x00, 0x1b, 0xf8, 0xf0, 0x1b, 0];
const MAP_MODE_HDMA_SETUP_NEAR: [u8; 7] = [0xf0, 0x27, 0xdd, 0xf0, 0x07, 0xde, 0];
const MAP_MODE_HDMA_SETUP_FAR: [u8; 7] = [0xf0, 0xe7, 0xde, 0xf0, 0xc7, 0xdf, 0];
const ATTRACT_INDIRECT_HDMA_SETUP: [u8; 7] = [0xf0, 0x00, 0x1b, 0xf0, 0xe0, 0x1b, 0];
const PRAYING_SCENE_HDMA_SETUP: [u8; 7] = [0xf8, 0x00, 0x1b, 0xf8, 0xf0, 0x1b, 0];
const MAP_MODE_PERSPECTIVE_ZOOMS_NEAR: [u16; 240] = [
    375, 374, 373, 373, 372, 371, 371, 370, 369, 369, 368, 367, 367, 366, 365, 365, 364, 363, 363,
    361, 361, 360, 359, 359, 358, 357, 357, 356, 355, 355, 354, 354, 353, 352, 352, 351, 351, 350,
    349, 349, 348, 348, 347, 346, 346, 345, 345, 344, 343, 343, 342, 342, 341, 341, 340, 339, 339,
    338, 338, 337, 337, 336, 335, 335, 334, 334, 333, 333, 332, 332, 331, 331, 330, 330, 328, 327,
    327, 326, 326, 325, 325, 324, 324, 323, 323, 322, 322, 321, 321, 320, 320, 319, 319, 318, 318,
    317, 317, 316, 316, 315, 315, 314, 314, 313, 313, 312, 312, 311, 311, 310, 310, 309, 309, 309,
    308, 308, 307, 307, 306, 306, 305, 305, 304, 304, 303, 303, 303, 302, 302, 301, 301, 300, 300,
    299, 299, 299, 298, 298, 297, 297, 295, 295, 294, 294, 294, 293, 293, 292, 292, 292, 291, 291,
    290, 290, 289, 289, 289, 288, 288, 287, 287, 287, 286, 286, 285, 285, 285, 284, 284, 283, 283,
    283, 282, 282, 281, 281, 281, 280, 280, 279, 279, 279, 278, 278, 278, 277, 277, 276, 276, 276,
    275, 275, 275, 274, 274, 273, 273, 273, 272, 272, 272, 271, 271, 271, 270, 270, 269, 269, 269,
    268, 268, 268, 267, 267, 267, 266, 266, 266, 265, 265, 265, 264, 264, 264, 263, 263, 262, 262,
    262, 261, 261, 261, 260, 260, 260, 259, 259, 259, 258, 258,
];
const MAP_MODE_PERSPECTIVE_ZOOMS_FAR: [u16; 240] = [
    136, 136, 135, 135, 135, 135, 135, 134, 134, 134, 133, 133, 133, 133, 132, 132, 132, 132, 132,
    131, 131, 131, 130, 130, 130, 130, 130, 129, 129, 129, 129, 129, 128, 128, 128, 127, 127, 127,
    127, 127, 126, 126, 126, 126, 126, 125, 125, 125, 124, 124, 124, 124, 124, 124, 123, 123, 123,
    123, 123, 122, 122, 122, 121, 121, 121, 121, 121, 121, 120, 120, 120, 120, 120, 120, 119, 119,
    119, 118, 118, 118, 118, 118, 118, 117, 117, 117, 117, 117, 117, 116, 116, 116, 116, 115, 115,
    115, 115, 115, 115, 114, 114, 114, 114, 114, 114, 113, 113, 113, 113, 112, 112, 112, 112, 112,
    112, 112, 111, 111, 111, 111, 111, 111, 110, 110, 110, 110, 110, 109, 109, 109, 109, 109, 109,
    108, 108, 108, 108, 108, 108, 108, 107, 107, 107, 107, 107, 106, 106, 106, 106, 106, 106, 106,
    105, 105, 105, 105, 105, 105, 105, 104, 104, 104, 104, 104, 103, 103, 103, 103, 103, 103, 103,
    103, 102, 102, 102, 102, 102, 102, 102, 101, 101, 101, 101, 101, 101, 100, 100, 100, 100, 100,
    100, 100, 100, 99, 99, 99, 99, 99, 99, 99, 99, 98, 98, 98, 98, 98, 97, 97, 97, 97, 97, 97, 97,
    97, 97, 96, 96, 96, 96, 96, 96, 96, 96, 96, 95, 95, 95, 95, 95, 95, 95, 94, 94, 94, 94, 94, 94,
    94, 94, 94,
];

#[derive(Clone, Default)]
struct SimpleHdma {
    table: Option<Vec<u8>>,
    table_pos: usize,
    indir: Vec<u8>,
    indir_pos: usize,
    rep_count: u8,
    mode: u8,
    ppu_addr: u8,
    indir_bank: u8,
}

enum SaveLoadFunc<'a, 'b> {
    Save(&'a mut ByteArray),
    Load(&'a mut LoadFuncState<'b>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateRecorder {
    pub last_inputs: u16,
    pub frames_since_last: u32,
    pub total_frames: u32,
    pub replay_pos: u32,
    pub replay_pos_last_complete: u32,
    pub replay_frame_counter: u32,
    pub replay_next_cmd_at: u32,
    pub replay_cmd: u8,
    pub replay_mode: bool,
    pub log: ByteArray,
    pub base_snapshot: ByteArray,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRecoderMultiPatch {
    pub count: u32,
    pub addr: u32,
    pub vals: [u8; 256],
}

impl Default for StateRecoderMultiPatch {
    fn default() -> Self {
        Self {
            count: 0,
            addr: 0,
            vals: [0; 256],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveLoadCommand {
    Save = 0,
    Load = 1,
    Replay = 2,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AssetPack {
    data: Vec<u8>,
    ranges: Vec<(usize, usize)>,
    #[serde(default)]
    names: Vec<String>,
    #[serde(skip)]
    dialogue_source_ir_table:
        std::sync::OnceLock<Option<Vec<Vec<crate::dialogue_ir::DialogueIrOp>>>>,
}

impl AssetPack {
    fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 88 || &data[..16] != ASSET_SIGNATURE_PREFIX {
            return Err("invalid zelda3_assets.dat signature".to_string());
        }

        let count = read_le_u32(data, 80)? as usize;
        let key_sig_len = read_le_u32(data, 84)? as usize;
        let sizes_start = 88usize;
        let key_sig_start = sizes_start
            .checked_add(count.checked_mul(4).ok_or("asset count overflow")?)
            .ok_or("asset header overflow")?;
        let mut offset = key_sig_start
            .checked_add(key_sig_len)
            .ok_or("asset key signature overflow")?;
        if key_sig_start > data.len() || offset > data.len() {
            return Err("asset header extends past file".to_string());
        }
        let names = data[key_sig_start..offset]
            .split(|byte| *byte == 0)
            .filter(|name| !name.is_empty())
            .map(|name| String::from_utf8(name.to_vec()).map_err(|_| "asset name is not utf8"))
            .collect::<Result<Vec<_>, _>>()?;
        if names.len() != count {
            return Err(format!(
                "asset key signature has {} names, expected {count}",
                names.len()
            ));
        }

        let mut ranges = Vec::with_capacity(count);
        for i in 0..count {
            let size = read_le_u32(data, sizes_start + i * 4)? as usize;
            offset = (offset + 3) & !3;
            let end = offset.checked_add(size).ok_or("asset range overflow")?;
            if end > data.len() {
                return Err("asset range extends past file".to_string());
            }
            ranges.push((offset, end));
            offset = end;
        }

        let data = data.to_vec();
        let dialogue_source_ir_table =
            Self::parse_dialogue_source_ir_table_result(&data, &ranges, &names)?;
        if names.iter().any(|name| name == "kDialogue") && dialogue_source_ir_table.is_none() {
            return Err(format!(
                "asset pack contains kDialogue but is missing required {DIALOGUE_SOURCE_SIDECAR_ASSET_NAME}"
            ));
        }

        Ok(Self {
            data,
            ranges,
            names,
            dialogue_source_ir_table: std::sync::OnceLock::from(dialogue_source_ir_table),
        })
    }

    fn from_data_ranges(data: Vec<u8>, ranges: Vec<(usize, usize)>) -> Self {
        Self::from_named_data_ranges(data, ranges, Vec::new())
    }

    fn from_named_data_ranges(
        data: Vec<u8>,
        ranges: Vec<(usize, usize)>,
        names: Vec<String>,
    ) -> Self {
        let dialogue_source_ir_table = Self::parse_dialogue_source_ir_table(&data, &ranges, &names);
        Self {
            data,
            ranges,
            names,
            dialogue_source_ir_table: std::sync::OnceLock::from(dialogue_source_ir_table),
        }
    }

    fn asset(&self, index: usize) -> Option<&[u8]> {
        let (start, end) = *self.ranges.get(index)?;
        Some(&self.data[start..end])
    }

    fn asset_by_name(&self, name: &str) -> Option<&[u8]> {
        let index = self.names.iter().position(|candidate| candidate == name)?;
        self.asset(index)
    }

    fn dialogue_source_sidecar_in<'a>(
        data: &'a [u8],
        ranges: &[(usize, usize)],
        names: &[String],
    ) -> Result<Option<&'a [u8]>, String> {
        let Some(index) = names
            .iter()
            .position(|name| name == DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)
        else {
            return Ok(None);
        };
        let (start, end) = *ranges
            .get(index)
            .ok_or_else(|| format!("{DIALOGUE_SOURCE_SIDECAR_ASSET_NAME} range is missing"))?;
        let asset = data
            .get(start..end)
            .ok_or_else(|| format!("{DIALOGUE_SOURCE_SIDECAR_ASSET_NAME} range is invalid"))?;
        let payload = asset
            .strip_prefix(DIALOGUE_SOURCE_SIDECAR_MAGIC)
            .ok_or_else(|| {
                format!("{DIALOGUE_SOURCE_SIDECAR_ASSET_NAME} has invalid semantic sidecar magic")
            })?;
        if payload.is_empty() {
            return Err(format!(
                "{DIALOGUE_SOURCE_SIDECAR_ASSET_NAME} has empty semantic sidecar payload"
            ));
        }
        Ok(Some(payload))
    }

    fn parse_dialogue_source_ir_table(
        data: &[u8],
        ranges: &[(usize, usize)],
        names: &[String],
    ) -> Option<Vec<Vec<crate::dialogue_ir::DialogueIrOp>>> {
        Self::parse_dialogue_source_ir_table_result(data, ranges, names)
            .ok()
            .flatten()
    }

    fn parse_dialogue_source_ir_table_result(
        data: &[u8],
        ranges: &[(usize, usize)],
        names: &[String],
    ) -> Result<Option<Vec<Vec<crate::dialogue_ir::DialogueIrOp>>>, String> {
        let Some(payload) = Self::dialogue_source_sidecar_in(data, ranges, names)? else {
            return Ok(None);
        };
        let table = bincode::deserialize(payload).map_err(|err| {
            format!("failed to deserialize {DIALOGUE_SOURCE_SIDECAR_ASSET_NAME}: {err}")
        })?;
        Ok(Some(table))
    }

    fn source_dialogue_ir_for_message(
        &self,
        message_id: u16,
    ) -> Option<Vec<crate::dialogue_ir::DialogueIrOp>> {
        let table = self
            .dialogue_source_ir_table
            .get_or_init(|| {
                Self::parse_dialogue_source_ir_table(&self.data, &self.ranges, &self.names)
            })
            .as_ref()?;
        table.get(usize::from(message_id)).cloned()
    }

    fn asset_mut(&mut self, index: usize) -> Option<&mut [u8]> {
        let (start, end) = *self.ranges.get(index)?;
        Some(&mut self.data[start..end])
    }
}

const fn oam_scanout_source_for_staged_promotion(
    main_iteration_completed: bool,
    staged: OamScanoutSource,
) -> OamScanoutSource {
    if !main_iteration_completed && staged.active_nmi_dma_is_presented() {
        // Advancing the display queue while resuming the same C main-loop
        // iteration promotes a field whose sprite evaluation already began.
        // A trailing NMI may make a newer OAM table resident, but it belongs to
        // the following field. If ZeldaRunGameLoop advanced its frame counter,
        // the staged snapshot's recorded source remains authoritative instead.
        OamScanoutSource::RetainPreviousPresented
    } else {
        staged
    }
}

fn spotlight_hdma_tables_from_ram(ram: &[u8]) -> [Vec<u8>; 2] {
    [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
        .map(|table_base| ram[table_base..table_base + ZeldaState::HDMA_DYNAMIC_TABLE_LEN].to_vec())
}

fn retain_captured_oam_for_scanout(
    obj_generation: &DisplayObjGeneration,
    scanout_source: OamScanoutSource,
) -> bool {
    obj_generation.retained_oam().is_some()
        || matches!(
            scanout_source,
            OamScanoutSource::RetainCapturedBeforeNmi
                | OamScanoutSource::RetainImmutableCapturedPpu
                | OamScanoutSource::RetainPreviousPresented
                | OamScanoutSource::RetainResidentPpuOam
        )
}

const fn is_dungeon_item_hold_entry(
    entry_frame: crate::game_state::FrameState,
    exit_frame: crate::game_state::FrameState,
    entry_link_handler_state: u8,
    exit_link_handler_state: u8,
) -> bool {
    const LINK_HANDLER_HOLD_ITEM: u8 = 21;
    entry_frame.main_module == 7
        && entry_frame.submodule == 0
        && exit_frame.main_module == 7
        && exit_frame.submodule == 0
        && entry_link_handler_state != LINK_HANDLER_HOLD_ITEM
        && exit_link_handler_state == LINK_HANDLER_HOLD_ITEM
}

const fn oam_scanout_for_dungeon_item_hold_entry(
    module_scanout: OamScanoutSource,
    dungeon_item_hold_entry: bool,
) -> OamScanoutSource {
    if dungeon_item_hold_entry {
        OamScanoutSource::ComposeLivePlayerOamAfterMain
    } else {
        module_scanout
    }
}

const fn dungeon_item_hold_publishes_live_scroll(
    frame: crate::game_state::FrameState,
    link_handler_state: u8,
    dungeon_item_hold_entry_scanout: bool,
) -> bool {
    frame.main_module == 7
        && frame.submodule == 0
        && link_handler_state == 21
        && !dungeon_item_hold_entry_scanout
}

/// Value `BG_TILE_ANIMATION_COUNTDOWN` reloads to when the dungeon animation
/// advances to its next page.
const BG_TILE_ANIMATION_COUNTDOWN_RELOAD: u8 = 9;

/// Whether the dungeon brightness phase scans out the live animated-BG page.
///
/// The animated-tile DMA runs on every frame of module 7/`$0a`, but only the
/// frame whose countdown has just reloaded carries a freshly advanced page,
/// and that upload lands too late for the current scanout. Snes9x therefore
/// keeps the host-boundary generation on the reload frame (f28358, countdown
/// `0x01` -> `0x09`) and shows the live post-NMI generation on the remaining
/// frames of the cycle (f28602, countdown `0x09` -> `0x08`). The two frames
/// share room `$41` and module 7/`$0a`/0, so the countdown phase is the only
/// thing that separates them.
const fn dungeon_brightness_animated_bg_is_live(
    host_entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    animation_countdown: u8,
) -> bool {
    dungeon_brightness_screen_layers_are_live(host_entry, following)
        && animation_countdown != BG_TILE_ANIMATION_COUNTDOWN_RELOAD
}

const fn dungeon_brightness_screen_layers_are_live(
    host_entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
) -> bool {
    host_entry.main_module == 7
        && host_entry.submodule == 0x0a
        && following.main_module == 7
        && following.submodule == 0x0a
}

fn dialogue_scroll_phase(
    continuation: DialogueScrollContinuation,
    publication: DialogueScanoutOwnership,
    has_frozen_scanout: bool,
    has_completion_scanout: bool,
    has_staged_completion: bool,
) -> DialogueScrollPhase {
    if continuation.is_copying_remaining_pixels() {
        debug_assert!(has_frozen_scanout);
        debug_assert!(!has_completion_scanout);
        debug_assert!(!has_staged_completion);
        return DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: continuation.completion_timing(),
        };
    }
    if continuation.is_return_only() {
        debug_assert!(has_frozen_scanout);
        debug_assert!(!has_completion_scanout);
        debug_assert!(!has_staged_completion);
        return DialogueScrollPhase::ReturnOnly;
    }
    if continuation.is_completion_pending_publication() {
        debug_assert!(has_frozen_scanout);
        debug_assert!(!has_completion_scanout);
        debug_assert!(!has_staged_completion);
        return DialogueScrollPhase::CompletionPendingPublication;
    }
    if has_staged_completion {
        debug_assert!(!has_completion_scanout);
        if publication == DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN {
            debug_assert!(has_frozen_scanout);
            return DialogueScrollPhase::CompletionStagedAfterFrozenScanout;
        }
        debug_assert_eq!(
            publication,
            DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_SNAPSHOT
        );
        return DialogueScrollPhase::CompletionStagedAfterSnapshot;
    }
    if has_completion_scanout {
        if publication == DialogueScanoutOwnership::COMPLETED_SCROLL {
            return DialogueScrollPhase::CompletedScroll;
        }
        debug_assert_eq!(publication, DialogueScanoutOwnership::RETIRED_TEXT_DMA);
        return DialogueScrollPhase::RetiredTextDma;
    }
    debug_assert!(publication.is_snapshot());
    DialogueScrollPhase::Idle
}

/// Reconstruct the unconditional `WritePpuRegisters` result from the machine
/// state captured when a cross-host NMI was accepted. A translated caller can
/// advance its native mirrors before the following host resumes that handler;
/// those later values are not source operands.
fn nmi_ppu_register_operands_from_snapshot(
    snapshot: &DisplaySnapshot,
) -> crate::NmiPpuRegisterOperands {
    let display = crate::game_state::DisplayState::load_from_ram(&snapshot.ram);
    crate::NmiPpuRegisterOperands {
        window_selection: [
            display.bg12_window_selection,
            display.bg34_window_selection,
            display.object_color_window_selection,
        ],
        color_window_selection: display.palette_filter.color_window_selection(),
        color_math_control: display.palette_filter.color_math_control(),
        fixed_color: [
            display.palette_filter.fixed_color_red(),
            display.palette_filter.fixed_color_green(),
            display.palette_filter.fixed_color_blue(),
        ],
        screen_layers: [
            display.main_screen_layers,
            display.sub_screen_layers,
            display.main_screen_window_layers,
            display.sub_screen_window_layers,
        ],
        bg_scroll: [
            display.ppu_scroll_copy.bg1_h_copy(),
            display.ppu_scroll_copy.bg1_v_copy(),
            display.ppu_scroll_copy.bg2_h_copy(),
            display.ppu_scroll_copy.bg2_v_copy(),
            display.ppu_scroll_copy.bg3_h_copy2(),
            display.ppu_scroll_copy.bg3_v_copy2(),
        ],
        screen_brightness: display.screen_brightness,
        mosaic: display.mosaic_copy,
        bg_mode: display.bg_mode,
        mode7_center: [
            display.ppu_scroll_copy.mode7_center_x(),
            display.ppu_scroll_copy.mode7_center_y(),
        ],
    }
}

fn nmi_ppu_register_scanout_from_acceptance_snapshot(
    snapshot: &DisplaySnapshot,
    exact_operands: Option<crate::NmiPpuRegisterOperands>,
) -> NmiPpuRegisterScanout {
    let operands =
        exact_operands.unwrap_or_else(|| nmi_ppu_register_operands_from_snapshot(snapshot));
    let mut ppu = snapshot.ppu.clone();
    for (address, value) in [
        (0x23, operands.window_selection[0]),
        (0x24, operands.window_selection[1]),
        (0x25, operands.window_selection[2]),
        (0x30, operands.color_window_selection),
        (0x31, operands.color_math_control),
        (0x32, operands.fixed_color[0]),
        (0x32, operands.fixed_color[1]),
        (0x32, operands.fixed_color[2]),
        (0x2c, operands.screen_layers[0]),
        (0x2d, operands.screen_layers[1]),
        (0x2e, operands.screen_layers[2]),
        (0x2f, operands.screen_layers[3]),
        (0x0d, operands.bg_scroll[0] as u8),
        (0x0d, (operands.bg_scroll[0] >> 8) as u8),
        (0x0e, operands.bg_scroll[1] as u8),
        (0x0e, (operands.bg_scroll[1] >> 8) as u8),
        (0x0f, operands.bg_scroll[2] as u8),
        (0x0f, (operands.bg_scroll[2] >> 8) as u8),
        (0x10, operands.bg_scroll[3] as u8),
        (0x10, (operands.bg_scroll[3] >> 8) as u8),
        (0x11, operands.bg_scroll[4] as u8),
        (0x11, (operands.bg_scroll[4] >> 8) as u8),
        (0x12, operands.bg_scroll[5] as u8),
        (0x12, (operands.bg_scroll[5] >> 8) as u8),
        (0x00, operands.screen_brightness),
        (0x06, operands.mosaic),
        (0x05, operands.bg_mode),
    ] {
        ppu.write(address, value);
    }
    if operands.bg_mode & 7 == 7 {
        for (address, value) in [
            (0x1c, 0),
            (0x1c, 0),
            (0x1d, 0),
            (0x1d, 0),
            (0x1f, operands.mode7_center[0] as u8),
            (0x1f, (operands.mode7_center[0] >> 8) as u8),
            (0x20, operands.mode7_center[1] as u8),
            (0x20, (operands.mode7_center[1] >> 8) as u8),
        ] {
            ppu.write(address, value);
        }
    }
    ppu.write(0x0b, 0x22);
    ppu.write(0x0c, 0x07);
    NmiPpuRegisterScanout::capture(&ppu)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ZeldaState {
    pub ram: Vec<u8>,
    #[serde(default)]
    pub(crate) game_state: GameState,
    /// Optional hardware-random input for exact ROM replays. The cartridge's
    /// `$8dba71` routine samples the PPU beam counter, so controller input alone
    /// cannot reproduce it in an atomic translated engine.
    #[serde(skip)]
    rom_random_replay: crate::rom_random::RomRandomReplay,
    pub sram: Vec<u8>,
    pub ppu: PpuState,
    /// Per-VRAM-slot logical CHR source bookkeeping (animation-modeled asset
    /// renderer M1). Write-only observation; never affects game/VRAM behavior,
    /// so it is excluded from serialization (recomputed at every CHR upload).
    #[serde(skip)]
    pub vram_chr_source: crate::chr_source::VramChrSourceTable,
    /// Raw source identity for authoring/preview tooling. Unlike `vram_chr_source`,
    /// sprite entries are not overwritten by content hashes, so offline tools can
    /// still map observed sprite palette usage back to `kSprGfx` pack/tile IDs.
    #[serde(skip)]
    pub vram_chr_preview_source: crate::chr_source::VramChrSourceTable,
    /// ROM graphics pack last decompressed into the animated-tile buffer (0xa680).
    /// Used only to tag the per-frame animated-tile DMA's VRAM slots with an
    /// injective logical CHR source (`CHR_KIND_BG_ANIM`). Pure render-bookkeeping,
    /// excluded from serialization like `vram_chr_source`.
    #[serde(skip)]
    pub animated_tile_pack: u16,
    /// Exact dynamic BG3 VWF glyph placements in message-buffer pixel space.
    /// Unlike tile provenance, this can represent packed/unaligned glyphs.
    #[serde(skip)]
    bg3_vwf_glyph_runs: Vec<Bg3VwfGlyphRun>,
    bg3_vwf_glyph_run_dialogue_offsets: Vec<u16>,
    #[serde(skip)]
    bg3_vwf_glyph_run_dialogue_message_id: u16,
    /// Continuation phase of an in-flight message-line scroll call. A value of
    /// 2 resumes the remaining three pixel copies; 1 resumes the return-only
    /// caller suffix. The main loop and frame counter run only on the initial
    /// two-pixel slice.
    #[serde(default)]
    pub(crate) dialogue_scroll_continuation: DialogueScrollContinuation,
    /// Set by `RenderText_Draw_MessageCharacters` when this frame's fast-forward
    /// render stopped mid-line at the per-frame budget; consumed at the START of
    /// the next `zelda_run_game_loop` to hold that frame's core update (frame
    /// counter + sprite/Link update), matching the ROM's core_update_disable.
    #[serde(skip)]
    pub(crate) dialogue_fast_forward_hold_pending: bool,
    #[serde(skip)]
    pub(crate) dialogue_fast_forward_hold_active: bool,
    /// The wire completed the save-menu initialization inside a continued
    /// caller host whose native module runs on the following iteration; the
    /// next `Module0E_0B_SaveMenu` call consumes this one-shot completion.
    #[serde(skip)]
    pub(crate) save_menu_initialization_completed_pending: bool,
    /// Set only while `advance_suspended_vwf_to_handler_completion` drives the
    /// character loop. The wire's suffix-completed terminal proves the
    /// RenderText caller returned to `ZeldaRunGameLoop`, which a begun scroll
    /// copy cannot do, so the loop yields with a `TEXT_CMD_SCROLL` command
    /// unconsumed; the next module iteration owns the scroll start (route
    /// host 20765).
    #[serde(skip)]
    pub(crate) dialogue_vwf_completion_stops_before_scroll: bool,
    /// The authoritative decoder endpoint was reached by a command which
    /// returns RenderText_Draw_MessageCharacters (a line command, a paced
    /// glyph): the ROM's handler is complete and only its caller suffix
    /// crosses to the terminal host, which must render nothing more (route
    /// host 163369/163370: `[line2]` ends the held group at read position 33).
    #[serde(skip)]
    pub(crate) dialogue_vwf_handler_completed_at_endpoint: bool,
    /// Zelda message-decoder endpoint published by the temporary Live timing
    /// authority for this host interval. This is a transient work boundary,
    /// not persisted game state; the native VWF owner consumes it while
    /// executing the same C command loop.
    #[serde(skip)]
    pub(crate) dialogue_live_message_read_position_target: Option<u16>,
    /// Number of calls still outstanding in the post-death save-options
    /// `Text_Render` loop. The original 65816 call stack can remain inside one
    /// of these calls across several vblanks; keeping the outer-loop program
    /// counter explicit prevents the translated `for` loop from retiring the
    /// Game Over module several host frames early.
    #[serde(skip)]
    pub(crate) game_over_text_render_calls_remaining: u8,
    /// True while the current post-death `Text_Render` invocation has not yet
    /// returned to its five-call outer loop.
    #[serde(skip)]
    pub(crate) game_over_text_render_call_in_flight: bool,
    /// The 65816 Y register as `Sprite_CheckDamageToPlayer` ($06:F145) leaves
    /// it when a path inside clobbers it: `Some(3 * (bump_damage & 0x0f) +
    /// armor)` after the damage table lookup, `Some(sprite_D)` on the shield
    /// path. Callers that (as ROM bugs) keep using Y as an index after the
    /// call clear it before calling and consult it afterwards.
    #[serde(skip)]
    pub(crate) rom_damage_check_y_register: Option<u8>,
    /// CPU entry phase for the next fresh VWF handler iteration. A caller
    /// suffix that returned in its own host slice reaches the following module
    /// iteration earlier than an ordinary game-loop entry.
    #[serde(skip)]
    pub(crate) dialogue_vwf_handler_entry_phase: messaging::VwfHandlerEntryPhase,
    /// CPU phase of an interruptible VWF glyph. `Entering` has not reached the
    /// ROM's dialogue-click store; `Drawing` has already performed entry-time
    /// effects and owns only pixel-loop work.
    #[serde(skip)]
    pub(crate) dialogue_vwf_glyph_cpu_phase: messaging::VwfGlyphCpuPhase,
    /// Semantic VWF metadata follows the same NMI publication boundary as the
    /// hardware text VRAM. CPU-authored glyphs stay private until subroutine 2
    /// uploads the completed buffer.
    #[serde(skip)]
    published_bg3_vwf_glyph_runs: Vec<Bg3VwfGlyphRun>,
    #[serde(skip)]
    published_bg3_vwf_glyph_run_dialogue_offsets: Vec<u16>,
    #[serde(skip)]
    published_dialogue_msg_read_pos: u16,
    #[serde(skip)]
    published_dialogue_message_id: u16,
    /// Coherent BG3 text VRAM and semantic glyph metadata owned by the current
    /// interrupted scroll. Copy slices retain this published display generation
    /// until the measured caller-completion boundary.
    #[serde(default, alias = "dialogue_scroll_frozen_text")]
    pub(crate) dialogue_scroll_frozen_scanout: Option<DialogueTextScanout>,
    /// Selects the single coherent BG3 text generation presented at this
    /// display boundary. The transparent byte preserves compatibility with
    /// snapshots whose old boolean field encoded snapshot/frozen ownership.
    #[serde(default, alias = "dialogue_scroll_stale_scanout")]
    pub(crate) dialogue_scanout_ownership: DialogueScanoutOwnership,
    /// Dedicated one-frame override presenting the freshly completed coherent
    /// scanout on the group-completion frame (see the lag handler). Separate
    /// from the frozen state to avoid cascading into adjacent scroll groups.
    #[serde(skip)]
    pub(crate) dialogue_scroll_completion_scanout: Option<DialogueTextScanout>,
    #[serde(skip)]
    pub(crate) dialogue_scroll_completion_staged: Option<DialogueTextScanout>,
    pub dma: DmaState,
    pub frame_ctr_dbg: u32,
    /// Legacy serialized host-input history. Retained for z3state compatibility;
    /// Snes9x libretro resolves opposing directions from its fixed report order,
    /// not from the preceding frame.
    #[serde(default)]
    previous_host_controller_input: u16,
    rom: Vec<u8>,
    assets: Option<AssetPack>,
    #[serde(default = "default_gloves_color")]
    gloves_color: [u16; 2],
    initialized: bool,
    apply_links_movement_to_camera_called: bool,
    /// Armed only across the ground handler's `link_handle_a_press` call so a
    /// chest-open item receipt scheduled inside it defers the handler's
    /// iteration tail to the receipt's completion slice (set and cleared
    /// within one translated call; never live across a frame boundary).
    #[serde(skip)]
    ground_apress_defers_atomic_item_receipt: bool,
    /// Host frame whose completion arm finished a ground A-press item
    /// receipt. The following NMI consumes live Link OBJ operands: the next
    /// iteration's LinkOam recomputes the pose pointers the receipt just
    /// changed, and the core's vblank after that iteration uploads them
    /// (f4587: the captured host boundary still held the pre-receipt pose).
    #[serde(skip)]
    item_receipt_completion_live_link_dma_host: Option<u32>,
    pub wanted_zelda_features: u32,
    pub state_recorder: StateRecorder,
    dialogue_blk_index: usize,
    dialogue_font_blk_index: usize,
    dialogue_flags: u8,
    #[serde(default)]
    #[serde(skip)]
    rom_startup_timing: bool,
    /// Runtime-only exact original timing machine and lifecycle provenance.
    /// Positional ZeldaState/bincode checkpoints remain byte-identical; bound
    /// exact persistence belongs to a separately versioned timing sidecar.
    #[serde(skip)]
    original_timing_owner: OriginalTimingOwnerState,
    /// Provenance latch distinguishing a genuinely fresh `new`/reset boundary
    /// from a restored frame-zero state after the public policy is disabled.
    /// Like the owner, this is runtime-only and absent from checkpoint bytes.
    #[serde(skip)]
    original_timing_cold_start_eligible: bool,
    /// Re-entrancy guard spanning the public host wrapper's callback/internal
    /// dispatch. A callback which invokes the public internal entry point must
    /// observe the already-owned source interval and never advance it twice.
    #[serde(skip)]
    original_timing_host_dispatch_active: bool,
    /// External pinned-Snes9x semantic receipt for exactly one upcoming host
    /// dispatch. The emulator remains outside ZeldaState and this receipt is
    /// absent from Zelda checkpoints.
    #[serde(skip)]
    original_timing_semantic_receipts: Option<OriginalTimingHostReceipts>,
    /// Linear Sprite_Main returns claimed by the immutable host execution plan
    /// for the one synchronous native caller body currently executing. Each
    /// true slot-zero loop boundary consumes one receipt; the scope must be
    /// empty again before that body returns.
    #[serde(skip)]
    original_timing_sprite_main_return_claims_remaining: Option<u8>,
    /// An NMI was accepted at the end of the preceding source host call, but
    /// that call returned before the handler completed. The next host must run
    /// the handler before resuming the interrupted C instruction. This is a
    /// backend-neutral execution phase; the temporary oracle's PC never enters
    /// Zelda gameplay state.
    #[serde(skip)]
    original_timing_nmi_publication_pending: bool,
    /// The source-sampled `$12` disposition belonging to the unfinished NMI.
    /// Kept beside the legacy boolean while checkpoint schema migration is
    /// explicit and fail-closed.
    #[serde(skip)]
    original_timing_pending_nmi_update_gate: Option<NmiUpdateGate>,
    /// Exact software-register generation sampled with the unfinished NMI.
    /// Like the pending display snapshot, this is runtime-only and cannot be
    /// reconstructed from native mirrors after the interrupted body advances.
    #[serde(skip)]
    original_timing_pending_nmi_ppu_register_operands: Option<crate::NmiPpuRegisterOperands>,
    /// Accepted gate dispositions for handlers owned by the active host.
    /// This transient queue is populated from the typed receipt stream before
    /// any translated handler runs and never enters Zelda save-state bytes.
    #[serde(skip)]
    original_timing_expected_nmi_update_gates: Vec<NmiUpdateGate>,
    /// Acceptance operands aligned one-for-one with the gate queue. Synthetic
    /// unit receipts may leave an entry absent and exercise the legacy local
    /// snapshot path; schema-gated live evidence always supplies every entry.
    #[serde(skip)]
    original_timing_expected_nmi_ppu_register_operands: Vec<Option<crate::NmiPpuRegisterOperands>>,
    /// A scheduled source caller accepted its following NMI at this host's
    /// return boundary. The ordered timeline is consumed before translated
    /// execution chooses among many early-return branches, so retain this one
    /// transient fact until the common internal-frame wrapper closes the host.
    /// It then becomes `original_timing_nmi_publication_pending` exactly once.
    #[serde(skip)]
    original_timing_scheduled_nmi_accepted_at_host_return: bool,
    /// The authority returned from the Module15 entry call before the native
    /// scheduler had materialized that same call's continuation. This token
    /// is bound to Module15 submodule 0 and is consumed exactly once by
    /// `FinishDungeonExitSpotlightEntry`. If native execution has already
    /// reached submodule 1 and its caller-return continuation, the source
    /// return is already represented and is not stored here. Unrelated host
    /// receipts still expire at the end of their dispatch.
    #[serde(skip)]
    original_timing_dungeon_exit_spotlight_entry_return_pending: bool,
    /// `Module_PreDungeon` returned before the delayed native Module 6 call
    /// began. The final source return dominates its earlier Sprite_ResetAll
    /// and Dungeon_ResetSprites interruption receipts, so the eventual native
    /// call executes the complete C body once without scheduling another
    /// timing wait.
    #[serde(skip)]
    original_timing_pre_dungeon_return_pending: Option<crate::MainLoopProgress>,
    /// One completed authority-owned audio presentation awaiting the native
    /// renderer's shadow pass. It is consumed exactly once by audio output.
    #[serde(skip)]
    original_timing_presented_audio: Option<crate::PresentedAudio>,
    /// Most recent native-vs-authority audio comparison. Diagnostic only;
    /// authority remains with the typed presentation receipt.
    #[serde(skip)]
    original_timing_audio_shadow_result: Option<crate::OriginalTimingAudioShadowResult>,
    /// Most recent native-vs-authority BG name-table comparison. Diagnostic
    /// only; the typed presentation receipt remains authoritative.
    #[serde(skip)]
    original_timing_bg_tilemap_shadow_result: Option<crate::OriginalTimingBgTilemapShadowResult>,
    /// Most recent native-vs-authority dialogue-text character comparison.
    /// Diagnostic only; the typed completed-scanout receipt remains authoritative.
    #[serde(skip)]
    original_timing_dialogue_text_shadow_result:
        Option<crate::OriginalTimingDialogueTextShadowResult>,
    /// Most recent native-vs-authority BG scroll comparison. Diagnostic only;
    /// the typed completed-scanout receipt remains authoritative.
    #[serde(skip)]
    original_timing_bg_scroll_shadow_result: Option<crate::OriginalTimingBgScrollShadowResult>,
    /// Most recent native-vs-authority Mode 7 transform comparison. Diagnostic
    /// only; the typed completed-scanout receipt remains authoritative.
    #[serde(skip)]
    original_timing_mode7_transform_shadow_result:
        Option<crate::OriginalTimingMode7TransformShadowResult>,
    /// Most recent native-vs-authority window-mask comparison. Diagnostic
    /// only; the typed completed-scanout receipt remains authoritative.
    #[serde(skip)]
    original_timing_window_mask_shadow_result: Option<crate::OriginalTimingWindowMaskShadowResult>,
    /// Scanline scroll receipt active only while the renderer consumes one
    /// immutable display snapshot. It never mutates Zelda's live PPU mirrors.
    #[serde(skip)]
    active_presented_bg_scroll: Option<crate::PresentedBgScroll>,
    /// Scanline Mode 7 receipt active only while the renderer consumes one
    /// immutable display snapshot. It never mutates Zelda's live PPU mirrors.
    #[serde(skip)]
    active_presented_mode7_transform: Option<crate::PresentedMode7Transform>,
    /// Scanline window receipt active only while the renderer consumes one
    /// immutable display snapshot. It never mutates live PPU/HDMA mirrors.
    #[serde(skip)]
    active_presented_window_mask: Option<crate::PresentedWindowMask>,
    /// Last pinned-Snes9x host-call receipt consumed by this live owner. The
    /// next receipt must be its exact successor, so identical controller input
    /// cannot make a stale receipt replayable.
    #[serde(skip)]
    original_timing_last_oracle_host_call: Option<u64>,
    /// Decoded OBJ cache generation present when an authoritative NMI receipt
    /// suspended the common sprite-preparation caller.
    #[serde(skip)]
    interrupted_nmi_prepare_obj_cache_vram: Option<Vec<u16>>,
    // Set on a frame whose ROM NMI is PARTIAL because a heavy load runs on the
    // main thread past the vblank (Snes9x-verified per site): the intro
    // message-pointer generation step, and the module-5 selected-game
    // load-initiation frame. On such a frame the ROM skips
    // Main_PrepSpritesForNmi, so rust must not advance the BG-tile / Link
    // animation countdowns either (else 0xc00d/0xc013 gain one decrement,
    // permanently phase-shifting the dungeon animated tile and cascading the
    // 14661+ tail). Consumed (taken) in zelda_run_game_loop.
    #[serde(skip)]
    rom_load_partial_nmi_this_frame: bool,
    #[serde(skip)]
    game_over_iris_goal_scanout_closed_pending: bool,
    #[serde(skip)]
    intro_initialization_work_frames_pending: u8,
    #[serde(skip)]
    intro_initialization_reset_obj_control_pending: bool,
    #[serde(skip)]
    rom_reset_frame_delay: u8,
    #[serde(skip)]
    intro_memory_darken_frame_delay: u8,
    save_quit_reset_hold: bool,
    #[serde(skip)]
    save_quit_reset_state_published: bool,
    save_quit_reset_writes_applied: bool,
    /// A Module09/0B submodule handler's trailing module/submodule advance,
    /// deferred while its fresh iteration is suspended before Sprite_Main's
    /// first slot: the ROM writes the advance at the end of the long submodule
    /// work, in the host where Sprite_Main then runs (Overworld_Func18/19 span
    /// route hosts 165774-165777 and 165778-165793).
    #[serde(skip)]
    pending_module09_frame_advance: Option<(u8, u8)>,
    /// The fully rebuilt Module09 sprite array is used to stage the held
    /// transition's OAM, but does not become CPU-visible until the source
    /// loader reaches its return host.
    #[serde(skip)]
    pending_overworld_sprite_reload_slots: Option<SpriteSlotsState>,
    /// Native entrance selection retained across its vertical-scroll stores.
    #[serde(skip)]
    pending_selected_game_entrance: Option<dungeon::SelectedGameEntranceContinuation>,
    /// Native loader publications in source order. A later allocation may
    /// reuse a slot before the scan returns, so the final array is insufficient.
    #[serde(skip)]
    pending_overworld_sprite_activations:
        Option<std::collections::VecDeque<(u8, SpriteSlotsState)>>,
    /// Source `Sprite_ActivateAllProxima` locals retained while any translated
    /// overworld reload scan crosses host boundaries: the routine temporarily
    /// walks BG2 H and forces the horizontal delta byte, then restores both at
    /// return.
    #[serde(skip)]
    overworld_proximity_scan_saved_scroll: Option<(u16, u8)>,
    #[serde(skip)]
    intro_poly_thread_initialization_phase: u8,
    #[serde(skip)]
    attract_init_graphics_phase: u8,
    #[serde(skip)]
    attract_first_story_render_delay: u8,
    #[serde(skip)]
    game_execution_scheduler: GameExecutionScheduler,
    /// Return frame for a `Sprite_Main` call made by Module 7. Long ROM
    /// subroutines take this frame into their continuation so the translated
    /// call stack resumes at the same semantic boundary.
    #[serde(skip)]
    active_dungeon_sprite_main_return: Option<DungeonSpriteMainReturn>,
    /// Return frame for a `Sprite_Main` call made by Module 9. Synchronous
    /// sprite handlers move this frame into their continuation so the native
    /// caller restores its four stack-local scroll values exactly once.
    #[serde(skip)]
    active_module09_sprite_main_return: Option<Module09ItemReceiptCallerReturn>,
    /// Exact semantic boundary derived from the continuous post-leading-NMI
    /// CPU shadow. `execute_cached_sprites` consumes it while the corresponding
    /// native Module 7 iteration is active.
    #[serde(skip)]
    dungeon_cached_sprite_cpu_interruption_pending: Option<CachedSpriteCpuInterruption>,
    /// Hardware boundary paired with the pending semantic progress receipt.
    /// Legacy shadow timing leaves this unset.
    #[serde(skip)]
    dungeon_cached_sprite_cpu_interruption_boundary: Option<OriginalTimingBoundary>,
    /// Source-order cursor and exact caller for a suspended crystal-peg
    /// attribute flip. The shared dungeon-caller continuation finishes the
    /// helper, resumes that caller, then enters Sprite_Main and the main-loop
    /// suffix.
    #[serde(skip)]
    pub(super) dungeon_peg_attribute_flip_pending: Option<DungeonPegAttributeFlipContinuation>,
    /// The state-12 ROM instruction stream completed its translated module
    /// suffix at the NMI edge. The next host first services that hardware NMI,
    /// then resumes the main loop at the following dungeon-state iteration.
    #[serde(skip)]
    dungeon_state_12_caller_suffix_nmi_pending: bool,
    /// Instruction-level result captured at the common Module07_02 dispatcher
    /// before the translated state 13/14 prefix mutates WRAM.
    #[serde(skip)]
    dungeon_landing_cpu_advance_pending: Option<DungeonModuleCpuAdvance>,
    /// ROM-timed part of the one-NMI landing reset field that remains owned
    /// by the corresponding caller-return publication.
    #[serde(skip)]
    dungeon_landing_spotlight_reset_prefix_scanlines: Option<usize>,
    /// Reset-field provenance moved out of the timing queue with the exact
    /// Module 7 iteration currently executing.
    #[serde(skip)]
    active_dungeon_landing_spotlight_reset_prefix_scanlines: Option<usize>,
    /// The instruction-timed Module 7 shadow reached NMI after Sprite_Main
    /// returned, either before or inside LinkOam_Main. This is a semantic
    /// caller phase, not a room/frame publication rule.
    #[serde(skip)]
    dungeon_post_sprite_main_return_pending: bool,
    /// The instruction-timed Module 7 shadow completed the shared module
    /// suffix, then reached vblank inside Main_PrepSpritesForNmi. Keep that
    /// caller phase live until the translated suffix reaches the same point.
    #[serde(skip)]
    dungeon_nmi_prepare_sprites_return_pending: bool,
    /// The translated dungeon palette body has reached the same WRAM
    /// milestones as the ROM shadow, while the instruction-level result
    /// records whether its common Module 7 caller returned before NMI.
    #[serde(skip)]
    dungeon_palette_cpu_advance_pending: Option<DungeonPaletteCpuAdvance>,
    /// ROM-shadow continuation schedule captured at the common state-1 entry.
    /// It stays live across the room-load and auxiliary-graphics suspensions.
    #[serde(skip)]
    dungeon_room_load_cpu_schedule: Option<DungeonRoomLoadCpuSchedule>,
    /// ROM-shadow continuation schedule captured from pre-NMI state before
    /// Module07_0E state 3 mutates the translated room model.
    #[serde(skip)]
    dungeon_submodule_cpu_schedule: Option<DungeonSubmoduleCpuSchedule>,
    /// Original-ROM timing for the interruptible Module09 world-map overlay
    /// conversion and its caller. Captured at the real main-wait NMI so both
    /// body crossings and the Sprite_Main return boundary share one source.
    #[serde(skip)]
    module09_cpu_schedule: Option<Module09CpuSchedule>,
    #[serde(skip)]
    sprite_main_cpu_boundary: Option<SpriteMainCpuBoundary>,
    #[serde(skip)]
    sprite_main_cpu_nmi_slices: u8,
    #[serde(skip)]
    sprite_main_cpu_caller: SpriteMainCpuCaller,
    /// A suspended Sprite_Main/cached-sprite stack whose entry phase came from
    /// the continuous dungeon-quadrant ROM shadow. Nested timed work inherits
    /// this provenance until the common Module 7 caller reaches its main wait.
    #[serde(skip)]
    dungeon_quadrant_cpu_continuation_active: bool,
    /// NMI crossings after the ROM's Sprite_Main return but before the Module
    /// 7 caller reaches its main-loop wait. Sprite_Main schedules this phase at
    /// its semantic return boundary so pre-NMI sprite work is not deferred.
    #[serde(skip)]
    dungeon_room_load_module_suffix_nmi_slices: u8,
    /// The room-$01 spiral return reaches the following player-control call
    /// after Module 7's current sprite/Link OAM suffix but before vblank.
    /// Keeping this as a one-shot continuation preserves that CPU ordering.
    #[serde(skip)]
    spiral_stair_return_player_control_pending: bool,
    /// All render/capture consumers in this host frame must observe the same
    /// completed spiral-return OAM shadow. A consumable boolean is incorrect:
    /// diagnostics may render before the actual frontend does.
    #[serde(skip)]
    spiral_stair_return_oam_publication_host_frame: Option<u32>,
    /// Host frame whose room-$41 state-13 body was entered by resuming the
    /// quadrant-upload caller after its leading NMI. Only that first body
    /// publishes its live palette/register generation; ordinary recurring
    /// bodies publish the palette consumed by their preceding NMI instead.
    #[serde(skip)]
    dungeon_state_13_pre_main_publication_host_frame: Option<u32>,
    /// Host frame whose state-13 body scheduled its measured caller return.
    /// This remains explicit because the final body advances to state 14
    /// before display publication inspects the resulting snapshot.
    #[serde(skip)]
    dungeon_state_13_recurring_main_publication_host_frame: Option<u32>,
    /// Host frame occupied only by the interrupted state-13 caller return.
    /// The return keeps the previous scanout palette/OAM while selecting the
    /// Link tile generation captured at the CPU slice entry.
    #[serde(skip)]
    dungeon_state_13_caller_return_publication_host_frame: Option<u32>,
    /// Host frame where the measured state-13 body and common Module 7 suffix
    /// both complete before vblank. Its scanout retains the entry OAM and Link
    /// graphics generations without retaining the preceding palette.
    #[serde(skip)]
    dungeon_state_13_atomic_caller_return_publication_host_frame: Option<u32>,
    /// Host frame where a state-14 landing palette pass completes. The decoded
    /// OBJ cache still belongs to the host-entry Link generation whether this
    /// is the penultimate pass or the final pass that schedules its return.
    #[serde(skip)]
    dungeon_faded_filter_palette_completion_host_frame: Option<u32>,
    /// Host frame occupied only by the state-14-to-15 interrupted common
    /// Module 7 caller return. No display-memory domain publishes during this
    /// slice, so the preceding presented palette, OAM, and decoded OBJ cache
    /// remain authoritative until the next fresh main iteration.
    #[serde(skip)]
    dungeon_faded_filter_caller_return_publication_host_frame: Option<u32>,
    /// Room whose completed supertile landing shifted subsequent player-control
    /// iterations onto a leading-NMI cadence. The phase ends when the next
    /// supertile transition initializes, preventing room-local timing from
    /// leaking into later transitions.
    #[serde(skip)]
    dungeon_post_landing_leading_nmi_room: Option<u8>,
    /// Player OAM authored by the returning Module 7 suffix before the ROM
    /// re-enters player control and performs the following Link OAM pass.
    #[serde(skip)]
    spiral_stair_return_player_oam_scanout: Option<[u16; 24]>,
    #[serde(skip)]
    next_display_vram_generation: DisplayVramGeneration,
    #[serde(skip)]
    next_display_cgram_override: Option<Vec<u16>>,
    #[serde(skip)]
    publish_live_hud_vram_on_next_capture: bool,
    #[serde(skip)]
    next_display_animated_bg_scanout_generation: Option<AnimatedBgScanoutGeneration>,
    #[serde(skip)]
    next_display_bg_scroll_generation: DisplayBgScrollGeneration,
    #[serde(skip)]
    next_display_obj_scanout_generation: Option<ObjScanoutGenerations>,
    /// Source location of the last `set_next_display_obj_scanout` caller. Lets a
    /// display probe answer "which handler staged this frame's OBJ scanout
    /// generation" without grepping every setter. Runtime-only diagnostic.
    #[serde(skip)]
    next_display_obj_scanout_provenance: Option<&'static core::panic::Location<'static>>,
    #[serde(skip)]
    next_display_obj_memory_generation: Option<DisplayObjGeneration>,
    /// One-shot decoded OBJ cache publication, independent from raw OBJ VRAM.
    #[serde(skip)]
    next_display_obj_cache_vram: Option<Vec<u16>>,
    #[serde(skip)]
    next_display_interrupted_item_receipt_obj_cache: bool,
    #[serde(skip)]
    enemy_drop_item_graphics_live_extended_oam_pending: bool,
    /// The enemy-drop item decompressor reaches its final `$0f` receipt sound
    /// store only after the measured multi-NMI call returns. The translated
    /// atomic caller may publish its other state early for scanout parity, but
    /// this order-sensitive audio command must retire at the real return edge.
    #[serde(skip)]
    enemy_drop_item_graphics_deferred_sound_effect_2: Option<u8>,
    #[serde(skip)]
    link_obj_dma_completed_this_frame: bool,
    /// Whether this suspended C main-loop iteration has already returned
    /// through `NMI_PrepareSprites`. A continuation may cross one or more
    /// host frames, but the C caller owns exactly one preparation pass.
    #[serde(skip)]
    main_loop_sprite_preparation_completed: bool,
    /// Typed source continuation retained across host calls when a receipt
    /// proves that `Module_MainRouting` has not yet returned. Ordinary Zelda
    /// save states cannot serialize an active 65816 call stack, so paired
    /// checkpoints reject this runtime-only owner until it has retired.
    #[serde(skip)]
    pending_main_loop_common_suffix: Option<MainLoopCommonSuffixContinuation>,
    #[serde(skip)]
    next_display_spotlight_scanout: Option<LiveSpotlightScanout>,
    /// Spotlight program authored after the current field's HDMA initialization.
    /// It becomes eligible only after that already-active field is captured.
    #[serde(skip)]
    spotlight_scanout_after_active_field: Option<LiveSpotlightScanout>,
    /// Register/HDMA effects of a held-latch NMI which interrupted Module 7.
    /// Snes9x may return the following internal field from the same host call;
    /// apply this receipt at the next publication boundary, not to the image
    /// which was already visible when the interrupt occurred.
    #[serde(skip)]
    interrupted_dungeon_submodule_publication: Option<InterruptedDungeonSubmodulePublication>,
    /// Atomic image of a spotlight table whose C builder is still suspended
    /// across additional NMIs. It may become publishable only when the
    /// measured call stack actually returns.
    #[serde(skip)]
    interrupted_dungeon_spotlight_build_in_flight: Option<LiveSpotlightScanout>,
    /// Last table whose interrupted C builder reached its caller return. A new
    /// atomic landing iteration can overwrite WRAM before its translated call
    /// is allowed to return, so suspended captures retain this completed
    /// hardware generation explicitly.
    #[serde(skip)]
    last_completed_interrupted_dungeon_spotlight_scanout: Option<LiveSpotlightScanout>,
    /// The state-0 `Spotlight_open` call began after the leading NMI had
    /// already selected the active field. This entry provenance governs the
    /// completed-table publication cadence for the lifetime of that wipe.
    #[serde(skip)]
    dungeon_landing_entry_started_after_leading_nmi: bool,
    /// The landing iris reached its goal and returned through Module 7 during
    /// active display. Retire the final Module 7 scanout before publishing the
    /// new module's cleared window controls.
    #[serde(skip)]
    dungeon_landing_goal_display_handoff: DungeonLandingGoalDisplayHandoff,
    #[serde(skip)]
    active_display_obj_generation: DisplayObjGeneration,
    #[serde(skip)]
    next_overworld_sprite_reload_entry_phase: Option<OverworldSpriteReloadEntryPhase>,
    #[serde(skip)]
    joypad_sampled_before_main: bool,
    #[serde(skip)]
    audio_nmi_processed_before_main: bool,
    /// Ambient APUI01 state sampled by a real C NMI after the ordinary host
    /// audio batch was published. The following audio callbacks retain that
    /// port read until the SPC exposes the matching acknowledgement.
    #[serde(skip)]
    audio_after_publication_ambient_nmi: Option<(u8, u8)>,
    #[serde(skip)]
    dungeon_exit_spotlight_cpu_entry_envelope: Option<(CpuRasterPosition, CpuRasterPosition)>,
    #[serde(skip)]
    overworld_spotlight_cpu_entry_envelope: Option<(CpuRasterPosition, CpuRasterPosition)>,
    #[serde(skip)]
    dungeon_landing_goal_transition_pending: bool,
    #[serde(skip)]
    normal_dialogue_following_main_nmi_uses_host_animated_bg_operands: Option<bool>,
    /// Operand generation observed by the instruction-timed ROM shadow at the
    /// core-update NMI following a deferred dialogue caller return. The NMI's
    /// decoded-BG effect belongs to the active scanout even though the atomic
    /// port reaches its live PPU write on the following host boundary.
    #[serde(skip)]
    next_core_nmi_active_scanout_uses_host_animated_bg_operands: Option<bool>,
    #[serde(skip)]
    pending_dialogue_initialization_schedule: Option<(u8, u8, Option<bool>)>,

    #[serde(skip)]
    intro_poly_upload_delay: u8,
    #[serde(skip)]
    display_snapshot: Option<Box<DisplaySnapshot>>,
    /// Monotonic identity shared by transient display snapshots and the exact
    /// NMI receipts attached to them. Both owners are serde-skipped.
    #[serde(skip)]
    display_snapshot_epoch: u64,
    /// Actual PPU memory destinations written while an explicit leading NMI
    /// executes. A value-diff is insufficient here: DMA can rewrite a word
    /// with the same live value, and that write still replaces an older
    /// presented generation.
    #[serde(skip)]
    active_effective_dma_writes: Option<EffectiveDmaWriteSet>,
    /// Last payload installed by a real $7e0800->$2104 OAM DMA. Display
    /// composition temporarily swaps `ppu` with immutable scanout snapshots,
    /// so `ppu.oam` alone cannot carry resident hardware state across that
    /// swap. This latch advances only at `complete_oam_dma_from_source`.
    #[serde(skip)]
    resident_oam_dma: Option<Vec<u16>>,
    /// OAM evaluated for the most recently rendered scanout. Interrupted CPU
    /// graphics work retains this exact hardware generation while NMI-owned
    /// register domains continue advancing.
    #[serde(skip)]
    last_presented_oam: Option<Vec<u16>>,
    /// OAM-law lanes (diagnostic; consumer-less by default — see the "OAM law
    /// audit" tracing envs in CLAUDE.md). The hardware rule they model,
    /// trace-verified against the pinned core: a $0800→$2104 transfer runs at
    /// each vblank whose main completed, carries the software shadow as it
    /// stands at that vblank, and becomes visible from the FOLLOWING scanout.
    /// `pending` holds the latest modeled transfer; a scanout capture promotes
    /// it to `visible`.
    #[serde(skip)]
    oam_law_pending: Option<Vec<u16>>,
    #[serde(skip)]
    oam_law_visible: Option<Vec<u16>>,
    /// Frame counter observed at the current host frame's entry. A capture
    /// whose frame did not advance the counter models a held vblank (the poly
    /// worker or an overrunning iteration owned it): hardware ran no
    /// NMI_DoUpdates there, so a pending transfer stays pending across it
    /// instead of becoming visible one scanout early.
    #[serde(skip)]
    oam_law_entry_frame_counter: Option<u8>,

    /// CGRAM evaluated for the most recently rendered scanout. Long palette
    /// walks can author the next PPU palette before the current field consumes
    /// it, independently of the other display domains.
    #[serde(skip)]
    last_presented_cgram: Option<Vec<u16>>,
    /// Complete VRAM generation backing the decoded OBJ cache used by the most
    /// recently rendered scanout. Snes9x caches both OBJ name pages, so this
    /// cannot be represented by Link's first 64 tiles alone.
    #[serde(skip)]
    last_presented_obj_vram: Option<Vec<u16>>,
    /// Host frame owning the staged presentation below. Repeated classic,
    /// modern, and diagnostic captures of one frame must all compose from the
    /// same prior scanout generation.
    #[serde(skip)]
    presented_history_host_frame: Option<u32>,
    #[serde(skip)]
    staged_presented_oam: Option<Vec<u16>>,
    #[serde(skip)]
    staged_presented_cgram: Option<Vec<u16>>,
    #[serde(skip)]
    staged_presented_obj_vram: Option<Vec<u16>>,
    #[serde(skip)]
    last_presented_vram_chr_source: Option<crate::chr_source::VramChrSourceTable>,
    #[serde(skip)]
    last_presented_vram_chr_preview_source: Option<crate::chr_source::VramChrSourceTable>,
    #[serde(skip)]
    staged_presented_vram_chr_source: Option<crate::chr_source::VramChrSourceTable>,
    #[serde(skip)]
    staged_presented_vram_chr_preview_source: Option<crate::chr_source::VramChrSourceTable>,
    #[serde(skip)]
    deferred_display_snapshot: Option<Box<DisplaySnapshot>>,
    /// Dynamic Mode 7 table generation before the ROM begins its descending
    /// projection loop. Captured separately because HDMA can consume the old
    /// and new generations within one field.
    #[serde(skip)]
    pub(super) attract_map_hdma_projection_before: Option<Vec<u8>>,
    /// Graphics DMA operands as they existed at the host vblank boundary.
    /// Snes9x resumes a pending NMI before the following main slice can advance
    /// the animated-BG or Link OBJ sources.
    #[serde(skip)]
    pre_main_graphics_dma: Option<PreMainGraphicsDma>,
    /// Per-capture parity diagnostics; never participates in simulation state.
    #[serde(skip)]
    debug_display_publication_candidates: Vec<DebugDisplayPublicationCandidate>,
    #[serde(skip)]
    debug_display_publication_context: Option<DebugDisplayPublicationContext>,
    #[serde(skip)]
    debug_display_cgram_candidates: Vec<DebugDisplayCgramCandidate>,
    /// Animated-BG VRAM as it existed when the host entered this frame, before
    /// the frame's NMI could upload a newly selected animation phase.
    #[serde(skip)]
    pre_nmi_animated_bg_scanout: Option<AnimatedBgScanout>,
    #[serde(default)]
    nmi_forced_blank_scanlines_pending: u8,
    /// Positional compatibility field for snapshots written before active-display
    /// blanking edges were made field-local. Old values are discarded at the
    /// next display capture instead of leaking a raster edge into another field.
    #[serde(alias = "nmi_forced_blank_from_scanline_pending")]
    legacy_nmi_forced_blank_from_scanline_pending: Option<u8>,
    #[serde(default)]
    nmi_active_display_blanking_candidate: NmiActiveDisplayBlanking,
    #[serde(skip)]
    active_display_force_blank_event: Option<u8>,
    /// Measured raster position of the next FileSelect_EraseTriforce
    /// EnableForceBlank request when its preceding C caller crossed into the
    /// active field. This is CPU-workload provenance, not display state, so it
    /// is consumed exactly once by the following file-select iteration.
    #[serde(default)]
    pending_file_select_force_blank_output_scanline: Option<u8>,
    /// Work performed by the most recent `Sprite_Main` call in this host
    /// frame. Consumers use it only through measured raster timing models.
    #[serde(skip)]
    last_sprite_main_timing_workload: Option<SpriteMainTimingWorkload>,
    #[serde(skip)]
    nmi_poly_upload_deferred: u8,
    #[serde(skip)]
    nmi_poly_upload_started: bool,
    #[serde(skip)]
    nmi_poly_deferred_upload_bypasses_latch: bool,
    #[serde(skip)]
    nmi_poly_upload_from_deferred: bool,
    #[serde(skip)]
    obj_vram_latch_generation: u64,
    /// Pre-upload CGRAM image latched when this frame's NMI performed the
    /// main-palette-buffer upload: hardware scanout only shows that upload on
    /// the NEXT frame, so the display compose prefers this image (see
    /// `with_display_snapshot`). Cleared at each display-snapshot capture.
    #[serde(skip)]
    cgram_upload_latch: Option<Vec<u16>>,
    #[serde(skip)]
    snes9x_poly_scheduler_counter: u8,
    #[serde(skip)]
    snes9x_hold_intro_step_this_frame: bool,
    #[serde(skip)]
    snes9x_intro_step_carry_phase_active: bool,
    #[serde(skip)]
    snes9x_intro_step_hold_alternate: bool,
    #[serde(skip)]
    last_poly_work: PolyWorkMetrics,
    #[serde(skip)]
    poly_job_in_flight: bool,
    /// Hosts the crystal-maiden cutscene's IRQ poly thread takes to start its
    /// first frame after activation (route host 413549: activation, config
    /// writes at +4, first frame complete at +7; later frames every 4 hosts).
    #[serde(skip)]
    poly_dungeon_thread_startup_hold: Option<u8>,
    /// `RenderText_Draw_Scroll` ran a whole scroll call inside a Triforce-room
    /// iteration; the ROM's main thread only owns the lines between the
    /// V-IRQ and vblank there, so the iteration stays held for the hosts the
    /// wire proves (route hosts 1558261-1558267: five 88.5-line passes in an
    /// 81-line slot).
    #[serde(skip)]
    triforce_room_scroll_this_iteration: bool,
    /// The ROM CPU shadow's WRAM after the last Triforce-room render: the
    /// thread's own working state (fill-loop carry words, edge tables, the
    /// bitmap buffer) that the next render reads, kept apart from the native
    /// rasterizer's state so the shadow reproduces the ROM thread's paths.
    #[serde(skip)]
    triforce_poly_shadow_ram: Option<Vec<u8>>,
    /// Crystal frames the dungeon poly thread has rendered since activation.
    #[serde(skip)]
    poly_dungeon_frames_rendered: u8,
    /// The ROM thread's in-flight render, advanced one host slot at a time
    /// (from that host's NMI swap to the V-IRQ line) so each host's actual NMI
    /// duration bounds the thread's time.
    #[serde(skip)]
    poly_shadow_run: Option<RomCpuTimingRun>,
    /// Master cycles the in-flight shadow render has consumed.
    #[serde(skip)]
    poly_shadow_master: u64,
    /// Hosts the in-flight shadow render has spanned.
    #[serde(skip)]
    poly_shadow_hosts: u8,
    /// A dungeon / Triforce-room poly frame completed inside this host's
    /// thread slot. The slot follows this host's NMI swap chronologically, so
    /// the completed bitmap is uploaded by the NEXT host's NMI (oracle: the
    /// crystal maiden's config-$96 frame completes at line 12 of run 413572
    /// and reaches VRAM at run 413573's NMI, route f413572). The bitmap is
    /// held as completed so the next slot may already start the following
    /// frame in the shared WRAM buffer.
    #[serde(skip)]
    poly_completed_upload: Option<(u32, Vec<u8>)>,
    /// Master cycles from the most recent NMI acceptance to the handler's
    /// thread swap ($00:82C7), measured by the shadow on the RAM state at
    /// that NMI (held latch → fast path at V≈227, poly upload → V≈256, text
    /// DMA → V≈248-250); the next host's thread slot begins there.
    #[serde(skip)]
    poly_next_host_swap_master: Option<u64>,
    /// `(latch_held, poly_upload_pending)` as seen by the most recent NMI;
    /// the next host's swap is measured on the top-of-host RAM with these
    /// two bytes ($12, $1F0C) restored to what that NMI saw.
    #[serde(skip)]
    poly_next_host_nmi_state: Option<(Option<bool>, bool)>,
    /// `(first, last)` NMI acceptance dispositions (held?) of the previous
    /// and current hosts' receipt vectors, refreshed at the top of every
    /// thread host: the handler that begins this host's run was accepted
    /// either at the end of the previous run (prev.last) or, when the run
    /// boundary fell just before it, at the head of this run (cur.first).
    #[serde(skip)]
    poly_receipt_gates_prev: Option<(Option<bool>, Option<bool>)>,
    #[serde(skip)]
    poly_receipt_gates_cur: Option<(Option<bool>, Option<bool>)>,
    /// Host that activated the dungeon poly thread.
    #[serde(skip)]
    poly_dungeon_activation_host: u32,
    /// The installed host began inside the previous iteration's common
    /// suffix; its leading completion retires the pending suffix before the
    /// host's own iteration (route host 511525).
    #[serde(skip)]
    original_timing_carried_suffix_completion_pending: bool,
    /// Diagnostic: where the active Sprite_Main return claim scope was opened.
    #[serde(skip)]
    original_timing_sprite_main_return_claim_scope_site:
        Option<&'static std::panic::Location<'static>>,
    /// The installed host's wire completes ZeldaRunGameLoop's common suffix
    /// with no held acceptance after its main-loop progress: every
    /// synchronous call of the iteration returned without crossing an NMI.
    #[serde(skip)]
    original_timing_host_iteration_uninterrupted: bool,
    /// Host slices of the crystal frame currently (or most recently) rendered.
    #[serde(skip)]
    poly_dungeon_current_frame_slices: u8,
    #[serde(skip)]
    poly_job_hold_frames: u8,
    #[serde(skip)]
    intro_title_fade_poly_phase: u8,
    #[serde(skip)]
    intro_title_fade_defer_suffix_this_frame: bool,
    #[serde(skip)]
    intro_title_fade_suffix_pending: bool,
    #[serde(skip)]
    intro_bg_fade_carry_frames: u8,
    #[serde(skip)]
    intro_bg_fade_poly_phase: u8,
    #[serde(skip)]
    intro_bg_fade_defer_suffix_this_frame: bool,
    #[serde(skip)]
    intro_bg_fade_suffix_pending: bool,
    #[serde(skip)]
    intro_zelda_fade_transition_pending: bool,
    #[serde(skip)]
    intro_poly_thread_teardown_pending: bool,
    ending_coords: sprite::PrepOamCoordsRet,
    #[serde(skip)]
    intro_poly_vram_history: Vec<(u8, Vec<u16>, Vec<u16>)>,
    audio: audio::AudioState,
    #[serde(skip)]
    emu_memory_ptr: Option<Vec<u8>>,
    #[serde(skip)]
    emu_runframe: Option<ZeldaRunFrameFunc>,
    #[serde(skip)]
    emu_syncall: Option<ZeldaSyncAllFunc>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct PolyWorkMetrics {
    pub divide_calls: u32,
    pub divide_shifts: u32,
    pub faces: u32,
    pub visible_faces: u32,
    pub edge_segments: u32,
    pub scanlines: u32,
    pub span_words: u32,
}

impl PolyWorkMetrics {
    pub fn estimated_65816_cycles(self) -> u32 {
        // The rasterizer's inner-loop cost changes with the number of visible
        // faces: sparse faces spend proportionally more time walking scanlines,
        // while dense faces spend more time in span writes.  Model those real
        // control-flow shapes separately instead of using a route/frame table.
        match self.visible_faces {
            1 => {
                20_106
                    + 16 * self.divide_shifts
                    + 124 * self.edge_segments
                    + 117 * self.scanlines
                    + 40 * self.span_words
            }
            2 => {
                20_223
                    + 32 * self.divide_shifts
                    + 50 * self.edge_segments
                    + 199 * self.scanlines
                    + 26 * self.span_words
            }
            3 => {
                20_975
                    + 15 * self.divide_shifts
                    + 96 * self.edge_segments
                    + 170 * self.scanlines
                    + 30 * self.span_words
            }
            _ => {
                19_317
                    + 21 * self.divide_shifts
                    + 472 * self.visible_faces
                    + 108 * self.edge_segments
                    + 135 * self.scanlines
                    + 44 * self.span_words
            }
        }
    }

    fn worker_frames(self) -> u8 {
        if self.estimated_65816_cycles() >= POLY_WORKER_TWO_FRAME_CYCLE_THRESHOLD {
            2
        } else {
            1
        }
    }
}

impl EffectiveDmaWriteSet {
    #[cfg(test)]
    fn new(vram_words: usize, active_snapshot_accepts_receipt: bool) -> Self {
        Self::new_for_snapshot(vram_words, active_snapshot_accepts_receipt, None)
    }

    fn new_for_snapshot(
        vram_words: usize,
        active_snapshot_accepts_receipt: bool,
        active_snapshot_epoch: Option<u64>,
    ) -> Self {
        Self {
            active_snapshot_accepts_receipt,
            active_snapshot_epoch,
            completed_ppu_registers_own_active_scanout: false,
            vram_words: vec![false; vram_words],
            completed_oam: None,
            completed_link_obj_dma: None,
            completed_cgram: None,
            completed_ppu_registers: None,
            completed_dialogue_metadata: None,
        }
    }
}

fn parity_trace_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/parity-traces")
        .join(file_name)
}

fn append_parity_trace(file_name: &str, trace: &str) {
    let trace_path = parity_trace_path(file_name);
    if trace_path
        .parent()
        .is_some_and(|directory| fs::create_dir_all(directory).is_ok())
    {
        if let Ok(mut file) = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(trace_path)
        {
            let _ = writeln!(file, "{trace}");
        }
    }
}

const LINK_DMA_EXPANDED_HIGH_PLANES_START: usize = 0xbd40;
const LINK_DMA_EXPANDED_HIGH_PLANES_LEN: usize = 0x80;
const LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN: usize = LINK_DMA_EXPANDED_HIGH_PLANES_LEN / 2;

pub type ZeldaRunFrameFunc = fn(&mut ZeldaState, u16, i32);
pub type ZeldaSyncAllFunc = fn(&mut ZeldaState);

fn default_gloves_color() -> [u16; 2] {
    [0x52f6, 0x0376]
}

fn wram_patch_addr(addr: usize) -> u32 {
    debug_assert!(addr < WRAM_SIZE);
    addr as u32
}

macro_rules! zelda_ppu_scroll_copy_methods {
    (
        $(
            fn $name:ident($($arg:ident: $ty:ty),*);
        )*
    ) => {
        $(
            pub(crate) fn $name(&mut self, $($arg: $ty),*) {
                self.ppu_scroll_copy_mut().$name($($arg),*);
            }
        )*
    };
}

macro_rules! zelda_world_camera_boundary_methods {
    (
        $(
            fn $name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)?;
        )*
    ) => {
        $(
            pub(crate) fn $name(&mut self, $($arg: $ty),*) $(-> $ret)? {
                self.world_camera_boundaries_mut().$name($($arg),*)
            }
        )*
    };
}

macro_rules! zelda_bridge_accessors {
    (
        $(
            $vis:vis fn $name:ident() -> $bridge:ident { $($target:tt)+ }
        )*
    ) => {
        $(
            $vis fn $name(&mut self) -> $bridge<'_> {
                $bridge::new(&mut self.$($target)+, &mut self.ram)
            }
        )*
    };
}

fn oam_entry_bytes(oam: &[u16], entry: usize) -> [u8; 4] {
    let first = oam[entry * 2].to_le_bytes();
    let second = oam[entry * 2 + 1].to_le_bytes();
    [first[0], first[1], second[0], second[1]]
}

const LINK_OAM_ENTRIES: [usize; 5] = [102, 103, 107, 110, 111];
const HOST_BOUNDARY_LINK_OAM_ENTRIES: [usize; 7] = [102, 103, 107, 110, 111, 112, 113];

fn compose_published_oam_entries<const N: usize>(
    oam: &mut [u16],
    published_shadow_oam: Option<&[u16]>,
    entries: [usize; N],
) {
    let Some(published) = published_shadow_oam.filter(|published| published.len() == oam.len())
    else {
        return;
    };
    for entry in entries {
        let start = entry * 2;
        oam[start..start + 2].copy_from_slice(&published[start..start + 2]);

        let high_word = 256 + entry / 8;
        let high_shift = (entry % 8) * 2;
        let high_mask = 0b11u16 << high_shift;
        oam[high_word] = (oam[high_word] & !high_mask) | (published[high_word] & high_mask);
    }
}

fn compose_host_boundary_link_oam(oam: &mut [u16], host_boundary_oam: Option<&[u16]>) {
    compose_published_oam_entries(oam, host_boundary_oam, HOST_BOUNDARY_LINK_OAM_ENTRIES);
}

fn publish_oam_shadow(oam: &mut [u16], shadow: &[u8]) -> bool {
    let byte_len = oam.len().saturating_mul(2);
    let Some(shadow) = shadow.get(..byte_len) else {
        return false;
    };
    for (word, bytes) in oam.iter_mut().zip(shadow.chunks_exact(2)) {
        *word = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
    true
}

fn dungeon_brightness_entry_retains_presented_player_graphics(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
) -> bool {
    dungeon_room_index == 0x41
        && entry.main_module == 7
        && entry.submodule == 0
        && following.main_module == 7
        && following.submodule == 0x0a
}

fn dungeon_transition_retains_presented_link_vram(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
) -> bool {
    following.main_module == 7
        && ((dungeon_room_index == 0x71
            && ((following.submodule == 2 && matches!(following.subsubmodule, 13 | 14))
                || (entry.main_module == 7
                    && entry.submodule == 2
                    && entry.subsubmodule == 15
                    && following.submodule == 5)))
            || dungeon_brightness_entry_retains_presented_player_graphics(
                entry,
                following,
                dungeon_room_index,
            ))
}

fn room_71_room_load_uses_live_obj_cache(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
    link_obj_scanout_generation: GraphicsDmaGeneration,
    link_obj_source_generation: GraphicsDmaGeneration,
    oam_scanout_source: OamScanoutSource,
    visible_blue_guard_workload: bool,
) -> bool {
    dungeon_room_index == 0x71
        && ((entry.main_module == 7
            && entry.submodule == 2
            && entry.subsubmodule == 15)
            // The measured visible-guard Sprite_Main workload reaches the
            // equivalent force-blank boundary at row 35. Snes9x has decoded
            // the live Link page by that later boundary even though raw OBJ
            // VRAM remains captured.
            || visible_blue_guard_workload)
        && following.main_module == 7
        && following.submodule == 5
        && link_obj_scanout_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
        && link_obj_source_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
        && oam_scanout_source == OamScanoutSource::RetainResidentPpuOam
}

fn room_71_item_graphics_return_crosses_completed_nmi(
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
    link_obj_scanout_generation: GraphicsDmaGeneration,
    oam_scanout_source: OamScanoutSource,
) -> bool {
    dungeon_room_index == 0x71
        && following.main_module == 7
        && following.submodule == 0
        && following.subsubmodule == 0
        && link_obj_scanout_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
        && oam_scanout_source == OamScanoutSource::ComposePublishedShadowDma
}

/// `ZELDA3_DIAGNOSTIC_TOLERATE_SONG_END_POLL=1`: development-only escape for
/// oracle-seeded or resumed probes whose audio driver state is not the
/// oracle's (see `finish_original_timing_host_dispatch`). Never set by the
/// authoritative gate; a run with it set is not parity evidence.
fn tolerate_unconsumed_song_end_poll_for_diagnostics() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("ZELDA3_DIAGNOSTIC_TOLERATE_SONG_END_POLL").is_some())
}


impl ZeldaState {
    pub(crate) fn compatibility_state_len(&self) -> usize {
        self.ram.len()
    }

    zelda_bridge_accessors! {
        pub(crate) fn follower_link_state_mut() -> NativeFollowerLinkBridgeMut {
            game_state.player.follower_link
        }
        pub(crate) fn enhanced_features_mut() -> NativeEnhancedFeaturesBridgeMut {
            game_state.enhanced_features
        }
        pub(crate) fn system_signals_mut() -> NativeSystemSignalsBridgeMut {
            game_state.system_signals
        }
        pub(crate) fn special_exit_position_mut() -> NativeSpecialExitPositionBridgeMut {
            game_state.player.special_exit_position
        }
        pub(crate) fn swim_acceleration_mut() -> NativeSwimAccelerationBridgeMut {
            game_state.player.swim_acceleration
        }
        pub(crate) fn bg1_move_calc_mut() -> NativeBg1MovementAccumulatorBridgeMut {
            game_state.player.bg1_movement_accumulator
        }
        pub(crate) fn tile_detect_position_mut() -> NativeTileDetectionBridgeMut {
            game_state.player.tile_detection
        }
        pub(crate) fn ppu_scroll_copy_mut() -> NativePpuScrollCopyBridgeMut {
            game_state.display.ppu_scroll_copy
        }
        pub(crate) fn attract_scene_mut() -> NativeAttractSceneBridgeMut {
            game_state.ending.attract_scene
        }
        pub(crate) fn dialogue_message_index_mut() -> NativeDialogueMessageIndexBridgeMut {
            game_state.messaging.dialogue_message_index
        }
        pub(crate) fn multiselect_choice_mut() -> NativeMultiselectChoiceBridgeMut {
            game_state.messaging.multiselect_choice
        }
        pub(crate) fn pushed_block_mut() -> NativePushedBlockBridgeMut {
            game_state.player.pushed_block
        }
        pub(crate) fn inventory_items_mut() -> NativeInventoryItemsBridgeMut {
            game_state.inventory.items
        }
        pub(crate) fn player_resources_mut() -> NativePlayerResourcesBridgeMut {
            game_state.inventory.player_resources
        }
        pub(crate) fn frame_state_mut() -> NativeFrameStateBridgeMut {
            game_state.frame
        }
        fn world_location_mut() -> NativeWorldLocationBridgeMut {
            game_state.world.location
        }
        pub(crate) fn world_scroll_mut() -> NativeWorldScrollBridgeMut {
            game_state.world.scroll
        }
        pub(crate) fn world_palette_theme_mut() -> NativeWorldPaletteThemeBridgeMut {
            game_state.world.palette_theme
        }
        pub(crate) fn world_region_mut() -> NativeWorldRegionBridgeMut {
            game_state.world.region
        }
        pub(crate) fn world_transient_mut() -> NativeWorldTransientBridgeMut {
            game_state.world.transient
        }
        pub(crate) fn overworld_map_ui_mut() -> NativeOverworldMapUiBridgeMut {
            game_state.world.overworld.map_ui
        }
        pub(crate) fn overworld_map_zoom_mut() -> NativeOverworldMapZoomBridgeMut {
            game_state.world.overworld.map_zoom
        }
        pub(crate) fn overworld_screen_size_mut() -> NativeOverworldScreenSizeBridgeMut {
            game_state.world.overworld.screen_size
        }
        pub(crate) fn overworld_scroll_delta_mut() -> NativeOverworldScrollDeltaBridgeMut {
            game_state.world.overworld.scroll_delta
        }
        pub(crate) fn overworld_entrance_mut() -> NativeOverworldEntranceBridgeMut {
            game_state.world.overworld.entrance
        }
        pub(crate) fn overworld_exit_mut() -> NativeOverworldExitBridgeMut {
            game_state.world.overworld.exit
        }
        pub(crate) fn overworld_transition_mut() -> NativeOverworldTransitionBridgeMut {
            game_state.world.overworld.transition
        }
        fn attract_vram_destination_bridge_mut() -> NativeAttractVramDestinationBridgeMut {
            game_state.display
        }
        pub(crate) fn display_core_mut() -> NativeDisplayStateBridgeMut {
            game_state.display
        }
        pub(crate) fn dungeon_secret_scratch_mut() -> NativeDungeonSecretBridgeMut {
            game_state.dungeon_secret
        }
        pub(crate) fn temp_counter_mut() -> NativeScratchCounterBridgeMut {
            game_state.scratch_counter
        }
        pub(crate) fn overworld_event_info_mut() -> NativeOverworldEventInfoBridgeMut {
            game_state.world.overworld.event_info
        }
        fn overworld_config_table_mut() -> NativeOverworldConfigTableBridgeMut {
            game_state.world.overworld.config_table
        }
        pub(crate) fn palette_buffer_mut() -> NativePaletteBufferBridgeMut {
            game_state.display
        }
        pub(crate) fn palette_filter_mut() -> NativePaletteFilterBridgeMut {
            game_state.display
        }
        pub(crate) fn hud_mut() -> NativeHudStateBridgeMut {
            game_state.display
        }
        fn hud_inventory_order_bridge_mut() -> NativeHudInventoryOrderBridgeMut {
            game_state.display
        }
        pub(crate) fn archery_game_mut() -> NativeArcheryGameBridgeMut {
            game_state.archery_game
        }
        pub(crate) fn minigame_state_mut() -> NativeMinigameBridgeMut {
            game_state.minigame
        }
        pub(crate) fn sprite_battle_mut() -> NativeSpriteBattleBridgeMut {
            game_state.sprite_battle
        }
        fn shared_message_timer_bridge_mut() -> NativeSharedMessageTimerBridgeMut {
            game_state.messaging.shared_message_timer
        }
        fn intro_scene_bridge_mut() -> NativeIntroSceneBridgeMut {
            game_state.ending.intro_scene
        }
        fn ending_credit_bridge_mut() -> NativeEndingCreditBridgeMut {
            game_state.ending.credits
        }
        pub(crate) fn intro_sword_mut() -> NativeIntroSwordBridgeMut {
            game_state.intro_sword
        }
        pub(crate) fn room_bounds_mut() -> NativeRoomBoundsBridgeMut {
            game_state.world.room_bounds
        }
        fn vram_upload_mut() -> NativeVramUploadBufferBridgeMut {
            game_state.display
        }
        pub(crate) fn poly_runtime_mut() -> NativePolyRuntimeBridgeMut {
            game_state.poly.runtime
        }
        pub(crate) fn poly_projected_vertex_mut() -> NativePolyProjectedVerticesBridgeMut {
            game_state.poly.projected_vertices
        }
        pub(crate) fn poly_face_coords_mut() -> NativePolyFaceCoordsBridgeMut {
            game_state.poly.face_coords
        }
        pub(crate) fn poly_raster_edge_mut() -> NativePolyRasterEdgeBridgeMut {
            game_state.poly.raster_edge
        }
        pub(crate) fn effect_angle_scratch_mut() -> NativeEffectAngleScratchBridgeMut {
            game_state.effects.angle_scratch
        }
        pub(crate) fn quake_spell_scratch_mut() -> NativeQuakeSpellBridgeMut {
            game_state.effects.quake_spell
        }
        pub(crate) fn bombos_spell_scratch_mut() -> NativeBombosSpellBridgeMut {
            game_state.effects.bombos_spell
        }
        pub(crate) fn tower_seal_scratch_mut() -> NativeTowerSealBridgeMut {
            game_state.effects.tower_seal
        }
        pub(crate) fn blast_wall_scratch_mut() -> NativeBlastWallBridgeMut {
            game_state.effects.entrance_effects
        }
        pub(crate) fn skull_woods_fire_scratch_mut() -> NativeSkullWoodsFireBridgeMut {
            game_state.effects.entrance_effects
        }
        fn weather_vane_bridge_mut() -> NativeWeatherVaneBridgeMut {
            game_state.world.overworld.weather_vane
        }
        fn bird_travel_destination_bridge_mut() -> NativeBirdTravelDestinationBridgeMut {
            game_state.world.overworld.bird_travel_destinations
        }
        pub(crate) fn door_debris_mut() -> NativeDoorDebrisBridgeMut {
            game_state.sprites.ancilla_slots
        }
        pub(crate) fn digging_game_prize_mut() -> NativeDiggingGamePrizeBridgeMut {
            game_state.effects.digging_game_prize
        }
        pub(crate) fn dialogue_number_mut() -> NativeDialogueNumberBridgeMut {
            game_state.messaging.dialogue_number
        }
        pub(crate) fn messaging_state_mut() -> NativeMessagingRuntimeBridgeMut {
            game_state.messaging
        }
        pub(crate) fn messaging_text_mut() -> NativeDecodedMessageTextBridgeMut {
            game_state.messaging
        }
        fn messaging_render_buffer_mut() -> NativeMessagingRenderBufferBridgeMut {
            game_state.messaging.render_buffer
        }
        fn vwf_render_mut() -> NativeVwfRenderBridgeMut {
            game_state.messaging.vwf_render
        }
        pub(crate) fn dialogue_source_offset_mut() -> NativeDialogueSourceOffsetBridgeMut {
            game_state.messaging.dialogue_source_offset
        }
        fn select_file_menu_mut() -> NativeSelectFileMenuBridgeMut {
            game_state.messaging.select_file_menu
        }
        pub(crate) fn follower_state_mut() -> NativeFollowerRuntimeBridgeMut {
            game_state.sprites.follower_runtime
        }
        pub(crate) fn chain_chomp_history_mut() -> NativeChainChompHistoryBridgeMut {
            game_state.sprites.chain_chomp_history
        }
        pub(crate) fn maze_game_timer_mut() -> NativeMazeGameTimerBridgeMut {
            game_state.sprites.maze_game_timer
        }
        pub(crate) fn ether_orbit_mut() -> NativeEtherOrbitBridgeMut {
            game_state.sprites.ether_orbit
        }
        pub(crate) fn prize_drop_cycle_mut() -> NativePrizeDropCycleBridgeMut {
            game_state.sprites.prize_drop_cycle
        }
        pub(crate) fn dual_layer_tile_cache_mut() -> NativeDualLayerTileCacheBridgeMut {
            game_state.sprites.dual_layer_tile_cache
        }
        pub(crate) fn garnish_state_mut() -> NativeGarnishRuntimeBridgeMut {
            game_state.sprites.garnish_runtime
        }
        pub(crate) fn oam_state_mut() -> NativeOamStateBridgeMut {
            game_state.oam
        }
        fn overworld_sprite_presence_mut() -> NativeOverworldSpritePresenceBridgeMut {
            game_state.sprites.overworld_sprite_presence
        }
        pub(crate) fn memorized_tile_mut() -> NativeMemorizedTileBridgeMut {
            game_state.memorized_tiles
        }
        fn overworld_sprite_loaded_mut() -> NativeOverworldSpriteLoadedBridgeMut {
            game_state.sprites.overworld_sprite_loaded
        }
        fn trinexx_palette_bridge_mut() -> NativeTrinexxPaletteBridgeMut {
            game_state.display
        }
        pub(crate) fn spotlight_hdma_mut() -> NativeSpotlightHdmaBridgeMut {
            game_state.display.spotlight_hdma
        }
        pub(crate) fn water_hdma_window_mut() -> NativeWaterHdmaWindowBridgeMut {
            game_state.display
        }
        pub(crate) fn overworld_map16_mut() -> NativeOverworldMap16BridgeMut {
            game_state.world.overworld.map16
        }
    }

    /// LINK_MAGIC_CONSUMPTION (0xf37b) read live from RAM, as C does at every consumer
    /// (`kCapeDepletionTimers[link_magic_consumption]`,
    /// `kLinkItem_MagicCosts[x * 3 + link_magic_consumption]`,
    /// `kCaneSpark_Magic[link_magic_consumption]`). PlayerResourcesState is the sole
    /// native owner -- it holds the only writer, the magic-shop 1/2-magic upgrade -- but
    /// it is not resynced from RAM at handler entry the way follower_link is, so reading
    /// the byte is both faithful to C and immune to that asymmetry.
    pub(crate) fn magic_consumption_level_live(&self) -> u8 {
        self.ram[crate::game_state::constants::LINK_MAGIC_CONSUMPTION]
    }

    pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
        self.system_signals_mut().set_ambient_sound_effect(value);
    }

    pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
        self.system_signals_mut().set_sound_effect_1(value);
    }

    #[track_caller]
    pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
        if crate::debug_env::var("ZELDA3_DEBUG_AUDIO_COMMAND_FRAME")
            .ok()
            .and_then(|frame| frame.parse::<u32>().ok())
            .is_some_and(|frame| frame == self.frame_ctr_dbg)
        {
            let caller = std::panic::Location::caller();
            eprintln!(
                "audio_command host={} latch=sfx2 old={:02x} new={value:02x} sprite_slot={} caller={}:{}",
                self.frame_ctr_dbg,
                self.game_state.system_signals.sound_effect_2(),
                self.game_state.sprites.system.cur_object_index(),
                caller.file(),
                caller.line(),
            );
        }
        self.system_signals_mut().set_sound_effect_2(value);
    }

    pub(crate) fn set_msu_volume(&mut self, value: u8) {
        self.system_signals_mut().set_msu_volume(value);
    }

    pub(crate) fn set_sound_effect_1_word(&mut self, value: u16) {
        self.system_signals_mut().set_sound_effect_1_word(value);
    }

    pub(crate) fn set_ambient_sound_effect_word(&mut self, value: u16) {
        self.system_signals_mut()
            .set_ambient_sound_effect_word(value);
    }

    pub(crate) fn clear_sound_effect_1(&mut self) {
        self.system_signals_mut().clear_sound_effect_1();
    }

    pub(crate) fn clear_sound_effect_2(&mut self) {
        self.system_signals_mut().clear_sound_effect_2();
    }

    pub(crate) fn clear_ambient_sound_effect(&mut self) {
        self.system_signals_mut().clear_ambient_sound_effect();
    }

    pub(crate) fn queue_sound_effect_1_if_empty(&mut self, value: u8) -> bool {
        self.system_signals_mut()
            .queue_sound_effect_1_if_empty(value)
    }

    pub(crate) fn queue_sound_effect_2_if_empty(&mut self, value: u8) -> bool {
        self.system_signals_mut()
            .queue_sound_effect_2_if_empty(value)
    }

    pub(crate) fn increment_hud_update_flag(&mut self) -> u8 {
        self.system_signals_mut().increment_hud_update_flag()
    }

    pub(crate) fn clear_hud_update_flag(&mut self) {
        self.system_signals_mut().clear_hud_update_flag();
    }

    pub(crate) fn set_bugs_fixed(&mut self, value: u8) {
        self.system_signals_mut().set_bugs_fixed(value);
    }

    pub(crate) fn clear_game_over_check_flag(&mut self) {
        self.system_signals_mut().clear_game_over_check_flag();
    }

    pub(crate) fn clear_restart_check_flag(&mut self) {
        self.system_signals_mut().clear_restart_check_flag();
    }

    pub(crate) fn set_restart_check_flag(&mut self, value: u8) {
        self.system_signals_mut().set_restart_check_flag(value);
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u8) {
        self.system_signals_mut().set_game_over_check_flag(value);
    }

    pub(crate) fn increment_game_over_check_flag(&mut self) {
        self.system_signals_mut().increment_game_over_check_flag();
    }

    fn compatibility_ram_range(&self, offset: usize, len: usize) -> &[u8] {
        CompatibilityBytesView::new(&self.ram).range(offset, len)
    }

    fn set_compatibility_ram_byte(&mut self, offset: usize, value: u8) {
        CompatibilityBytesViewMut::new(&mut self.ram).set_byte_at(offset, value);
    }

    zelda_ppu_scroll_copy_methods! {
        fn set_mapbak_tm(value: u8);
        fn set_mapbak_ts(value: u8);
        fn set_mapbak_tm_word(value: u16);
        fn set_bg1_h_high(value: u8);
        fn set_bg1_h_copy(value: u16);
        fn set_bg1_v_copy(value: u16);
        fn set_bg2_h_copy(value: u16);
        fn set_bg2_v_copy(value: u16);
        fn set_bg1_h_copy_low(value: u8);
        fn set_bg1_v_copy_low(value: u8);
        fn set_bg2_h_copy_low(value: u8);
        fn set_bg2_v_copy_low(value: u8);
        fn set_bg1_h_copy2(value: u16);
        fn set_bg1_v_copy2(value: u16);
        fn set_bg2_h_copy2(value: u16);
        fn set_bg2_v_copy2(value: u16);
        fn set_bg3_h_copy2(value: u16);
        fn set_bg3_v_copy2(value: u16);
        fn set_bg3_v_copy2_low(value: u8);
        fn set_mode7_center_x(value: u16);
        fn set_mode7_center_y(value: u16);
        fn set_mode7_center(x: u16, y: u16);
        fn set_bg1_h_live_and_copy(value: u16);
        fn set_bg1_v_live_and_copy(value: u16);
        fn set_bg2_h_live_and_copy(value: u16);
        fn set_bg2_v_live_and_copy(value: u16);
        fn set_bg1_bg2_h_live_and_copy(value: u16);
        fn set_bg1_bg2_v_live_and_copy(value: u16);
        fn set_bg1_bg2_live_and_copy(bg2_h: u16, bg2_v: u16, bg1_h: u16, bg1_v: u16);
        fn set_bg2_h_copy2_cached(value: u16);
        fn set_bg2_v_copy2_cached(value: u16);
        fn cache_bg2_live_scroll();
        fn cache_bg2_live_scroll_from(bg2_h: u16, bg2_v: u16);
        fn save_special_exit_bg2_live_scroll();
        fn save_exit_bg2_live_scroll();
        fn restore_special_exit_bg2_scroll_to_all_layers();
        fn restore_exit_bg2_scroll_to_all_layers();
        fn set_all_layer_h_scrolls(value: u16);
        fn set_all_layer_v_scrolls(value: u16);
        fn set_map_backup_scrolls(bg1_h: u16, bg2_h: u16, bg1_v: u16, bg2_v: u16);
        fn clear_bg3_h_copy2();
        fn clear_bg3_v_copy2();
        fn add_bg1_h_copy_low(value: u8);
        fn add_bg1_v_copy_low(value: u8);
        fn add_bg2_v_copy_low(value: u8);
        fn subtract_bg2_h_copy_low(value: u8);
        fn add_bg2_h_copy2_signed(value: i8);
        fn add_bg2_v_copy2_signed(value: i8);
        fn add_bg3_v_copy2_signed(value: i8);
        fn clear_bg1_scroll_subpixels();
        fn add_bg1_h_live_subpixel(subpixel: u16, scroll: u16);
        fn add_bg1_v_live_subpixel(subpixel: u16, scroll: u16);
        fn subtract_bg1_v_live_subpixel(value: u32);
        fn add_bg1_h_copy2_subpixel(subpixel: u16, scroll: u16);
        fn add_bg1_v_copy2_subpixel(subpixel: u16, scroll: u16);
        fn subtract_bg1_v_copy2_subpixel(subpixel: u16, scroll: u16);
        fn set_bg1_h_subpixel(value: u16);
        fn set_bg1_v_subpixel(value: u16);
        fn step_bg2_h_copy2_toward_cached();
        fn step_bg2_v_copy2_toward_cached();
        fn add_bg2_h_copy2(value: u16);
        fn add_bg2_v_copy2(value: u16);
        fn add_bg2_copy2_for_axis_signed(vertical: bool, value: i16);
        fn copy_bg1_live_to_ppu_copy();
        fn copy_bg2_live_to_ppu_copy();
        fn copy_live_to_ppu_copy();
        fn copy_bg2_live_to_bg1_live();
        fn copy_bg2_h_live_to_bg1_h_live();
        fn copy_bg2_v_live_to_bg1_v_live();
        fn set_mapbak_main_tile_theme_index(value: u8);
        fn set_mapbak_sprite_graphics_index(value: u8);
        fn set_mapbak_aux_tile_theme_index(value: u8);
        fn set_mapbak_bg1_x_offset(value: u16);
        fn set_mapbak_bg1_y_offset(value: u16);
        fn set_mapbak_cgwsel(value: u8);
        fn set_mapbak_cgwsel_word(value: u16);
        fn set_mapbak_hdmaen(value: u8);
    }

    pub(crate) fn multiselect_choice(&self) -> MultiselectChoiceRead<'_> {
        MultiselectChoiceRead::new(
            &self.game_state.messaging.multiselect_choice,
            &self.game_state.messaging.runtime,
        )
    }

    pub(crate) fn item_memory_value(&self, item_memory_addr: usize) -> u8 {
        self.game_state
            .inventory
            .items
            .item_memory_value(&self.ram, item_memory_addr)
    }

    pub(crate) fn set_main_module(&mut self, value: u8) {
        self.frame_state_mut().set_main_module(value);
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        self.frame_state_mut().set_main_module_word(value);
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        self.frame_state_mut().set_submodule(value);
    }

    #[track_caller]
    pub(crate) fn increment_submodule(&mut self) {
        self.frame_state_mut().increment_submodule();
    }

    #[track_caller]
    pub(crate) fn decrement_submodule(&mut self) {
        self.frame_state_mut().decrement_submodule();
    }

    #[track_caller]
    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.frame_state_mut().set_subsubmodule(value);
    }

    #[track_caller]
    pub(crate) fn increment_subsubmodule(&mut self) {
        self.frame_state_mut().increment_subsubmodule();
    }

    #[track_caller]
    pub(crate) fn decrement_subsubmodule(&mut self) {
        self.frame_state_mut().decrement_subsubmodule();
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.frame_state_mut().set_frame_counter(value);
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        self.frame_state_mut().increment_frame_counter();
    }

    pub(crate) fn clear_modal_pause_flag(&mut self) {
        self.frame_state_mut().clear_modal_pause_flag();
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        self.frame_state_mut().set_modal_pause_flag(value);
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        self.frame_state_mut().increment_modal_pause_flag()
    }

    zelda_world_camera_boundary_methods! {
        fn set_camera_y_coord_scroll_low(value: u16);
        fn set_camera_y_coord_scroll_hi(value: u16);
        fn set_camera_x_coord_scroll_low(value: u16);
        fn set_camera_x_coord_scroll_hi(value: u16);
        fn add_camera_scroll_for_axis(horizontal: bool, delta: i16) -> u16;
        fn set_camera_scroll_from_link_for_axis(horizontal: bool, value: u16);
        fn set_up_down_scroll_target(value: u16);
        fn set_up_down_scroll_target_end(value: u16);
        fn set_left_right_scroll_target(value: u16);
        fn set_left_right_scroll_target_end(value: u16);
        fn cache_scroll_targets();
        fn cache_camera_scroll();
        fn restore_scroll_targets_from_cached();
        fn set_overworld_scroll_up_counter(value: u16);
        fn set_overworld_scroll_down_counter(value: u16);
        fn set_overworld_scroll_left_counter(value: u16);
        fn set_overworld_scroll_right_counter(value: u16);
        fn set_overworld_scroll_counter_for_axis(ya: usize, value: u16);
        fn clear_opposed_scroll_counters(ya: usize);
        fn set_opposed_scroll_counter_pair(ya: usize, value: u16);
        fn set_special_exit_room_bounds(y_start: u16, y_end: u16, x_start: u16, x_end: u16);
        fn save_exit_room_bounds(y_start: u16, y_end: u16, x_start: u16, x_end: u16);
        fn copy_spexit_scroll_targets();
        fn copy_spexit_scroll_counters();
        fn restore_spexit_scroll_targets();
        fn restore_spexit_scroll_counters();
        fn copy_exit_scroll_targets();
        fn copy_exit_scroll_counters();
        fn restore_exit_scroll_targets();
        fn restore_exit_scroll_counters();
        fn save_spexit_camera_coords();
        fn save_exit_camera_coords();
        fn restore_exit_camera_scroll();
        fn restore_special_exit_camera_scroll();
        fn restore_camera_y_from_cached_indoor();
        fn restore_camera_x_from_cached_indoor();
        fn update_camera_hi_outdoor();
    }

    pub(crate) fn set_rng_seed(&mut self, value: u8) {
        self.world_region_mut().set_rng_seed(value);
    }

    pub(crate) fn set_dark_world_region_index(&mut self, value: u8) {
        self.world_region_mut().set_dark_world_region_index(value);
    }

    pub(crate) fn set_which_entrance(&mut self, value: u16) {
        self.world_region_mut().set_which_entrance(value);
    }

    pub(crate) fn set_which_entrance_byte(&mut self, value: u8) {
        self.world_region_mut().set_which_entrance_byte(value);
    }

    pub(crate) fn clear_overlay_index_word(&mut self) {
        self.world_region_mut().clear_overlay_index_word();
    }

    pub(crate) fn set_overlay_index_word(&mut self, value: u16) {
        self.world_region_mut().set_overlay_index_word(value);
    }

    pub(crate) fn set_overlay_high(&mut self, value: u8) {
        self.world_region_mut().set_overlay_high(value);
    }

    pub(crate) fn set_prev_screen_index_word(&mut self, value: u16) {
        self.world_region_mut().set_prev_screen_index_word(value);
    }

    pub(crate) fn set_ow_entrance_value(&mut self, value: u16) {
        self.world_region_mut().set_ow_entrance_value(value);
    }

    pub(crate) fn ow_entrance_value(&self) -> u16 {
        self.game_state.world.region.ow_entrance_value()
    }

    pub(crate) fn clear_custom_spell_animation(&mut self) {
        self.world_transient_mut().clear_custom_spell_animation();
    }

    pub(crate) fn set_custom_spell_animation_active(&mut self) {
        self.world_transient_mut()
            .set_custom_spell_animation_active();
    }

    pub(crate) fn set_flag_travel_bird(&mut self, value: u8) {
        // FLAG_TRAVEL_BIRD (0xaf4) is one byte but was modeled by TWO native fields:
        // world.travel_bird_flag (set here, no readers) and display.travel_bird_tile_offset
        // (read for the DMA tile source in misc.rs / has_travel_bird_tile_upload, and projected
        // LAST in GameState::write_to_ram). Writing world.travel_bird_flag let display's stale
        // copy re-project over the duck's per-frame cycling value (f533517, travel-bird duck).
        // Target the display field that actually owns the byte so the write survives.
        self.set_travel_bird_tile_offset(value);
    }

    pub(crate) fn clear_tile_interaction_shared_flag(&mut self) {
        self.world_transient_mut()
            .clear_tile_interaction_shared_flag();
    }

    pub(crate) fn clear_hud_floor_changed_timer(&mut self) {
        // HUD_FLOOR_CHANGED_TIMER (0x4a0) is owned by display.hud_tilemap (see
        // set_hud_floor_changed_timer / hud_floor_indicator), not world_transient. Clear the
        // low byte there so the write reaches RAM and is not re-clobbered by a stale projection.
        self.clear_floor_changed_timer_low();
    }

    pub(crate) fn cache_quadrant_fullsize_state(&mut self) {
        self.world_transient_mut().cache_quadrant_fullsize_state();
    }

    pub(crate) fn set_quadrant_fullsize_x(&mut self, value: u8) {
        self.world_transient_mut().set_quadrant_fullsize_x(value);
    }

    pub(crate) fn set_quadrant_fullsize_y(&mut self, value: u8) {
        self.world_transient_mut().set_quadrant_fullsize_y(value);
    }

    pub(crate) fn apply_reset_xy_quadrant_overrides(&mut self, reset_xy_flags: u16) {
        self.world_transient_mut()
            .apply_reset_xy_quadrant_overrides(reset_xy_flags);
    }

    pub(crate) fn increment_move_overlay_ctr(&mut self) -> u8 {
        self.world_transient_mut().increment_move_overlay_ctr()
    }

    pub(crate) fn set_dung_replacement_tile_state(&mut self, index: usize, value: u16) {
        self.world_transient_mut()
            .set_dung_replacement_tile_state(index, value);
    }

    pub(crate) fn birdtravel_status(&self) -> u8 {
        self.game_state.world.overworld.map_ui.birdtravel_status()
    }

    pub(crate) fn birdtravel_status_word(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .map_ui
            .birdtravel_status_word()
    }

    pub(crate) fn set_birdtravel_status(&mut self, value: u8) {
        self.overworld_map_ui_mut().set_birdtravel_status(value);
    }

    pub(crate) fn set_birdtravel_status_word(&mut self, value: u16) {
        self.overworld_map_ui_mut()
            .set_birdtravel_status_word(value);
    }

    pub(crate) fn and_birdtravel_status(&mut self, value: u8) {
        self.overworld_map_ui_mut().and_birdtravel_status(value);
    }

    pub(crate) fn decrement_birdtravel_status(&mut self) {
        self.overworld_map_ui_mut().decrement_birdtravel_status();
    }

    pub(crate) fn increment_birdtravel_status(&mut self) {
        self.overworld_map_ui_mut().increment_birdtravel_status();
    }

    pub(crate) fn clear_bird_travel_stop_status(&mut self, slot: usize) {
        self.overworld_map_ui_mut()
            .clear_bird_travel_stop_status(slot);
    }

    pub(crate) fn increment_bird_travel_stop_status(&mut self, slot: usize) {
        self.overworld_map_ui_mut()
            .increment_bird_travel_stop_status(slot);
    }

    pub(crate) fn special_entrance_trigger(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .entrance
            .special_entrance_trigger
    }

    pub(crate) fn set_special_entrance_trigger(&mut self, value: u8) {
        self.overworld_entrance_mut()
            .set_special_entrance_trigger(value);
    }

    pub(crate) fn clear_special_entrance_trigger(&mut self) {
        self.overworld_entrance_mut()
            .clear_special_entrance_trigger();
    }

    pub(crate) fn entrance_sequence_counter(&self) -> u8 {
        self.game_state.world.overworld.entrance.sequence_counter
    }

    pub(crate) fn set_entrance_sequence_counter(&mut self, value: u8) {
        self.overworld_entrance_mut().set_sequence_counter(value);
    }

    pub(crate) fn clear_entrance_sequence_counter(&mut self) {
        self.overworld_entrance_mut().clear_sequence_counter();
    }

    pub(crate) fn increment_entrance_sequence_counter(&mut self) -> u8 {
        self.overworld_entrance_mut().increment_sequence_counter()
    }

    pub(crate) fn decrement_entrance_sequence_counter(&mut self) -> u8 {
        self.overworld_entrance_mut().decrement_sequence_counter()
    }

    pub(crate) fn exit_screen_index(&self) -> u16 {
        self.game_state.world.overworld.exit.exit_screen
    }

    pub(crate) fn set_exit_screen_index(&mut self, value: u16) {
        self.overworld_exit_mut().set_exit_screen(value);
    }

    pub(crate) fn special_exit_screen_index(&self) -> u16 {
        self.game_state.world.overworld.exit.special_exit_screen
    }

    pub(crate) fn set_special_exit_screen_index(&mut self, value: u16) {
        self.overworld_exit_mut().set_special_exit_screen(value);
    }

    pub(crate) fn screen_transition_direction_bits(&self) -> u8 {
        self.game_state.world.overworld.transition.direction_bits()
    }

    pub(crate) fn screen_transition_direction_bits_word(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .transition
            .direction_bits_word()
    }

    pub(crate) fn has_screen_transition_direction_bits(&self) -> bool {
        self.game_state
            .world
            .overworld
            .transition
            .has_direction_bits()
    }

    pub(crate) fn edge_transition_direction_bits(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .transition
            .edge_direction_bits()
    }

    pub(crate) fn set_edge_transition_direction_bits(&mut self, value: u8) {
        self.overworld_transition_mut()
            .set_edge_direction_bits(value);
    }

    pub(crate) fn clear_edge_transition_direction_bits(&mut self) {
        self.overworld_transition_mut().clear_edge_direction_bits();
    }

    pub(crate) fn set_screen_transition_direction_bits(&mut self, value: u8) {
        self.overworld_transition_mut().set_direction_bits(value);
    }

    pub(crate) fn set_screen_transition_direction_bits_word(&mut self, value: u16) {
        self.overworld_transition_mut()
            .set_direction_bits_word(value);
    }

    pub(crate) fn clear_screen_transition_direction_bits(&mut self) {
        self.overworld_transition_mut().clear_direction_bits();
    }

    pub(crate) fn clear_screen_transition_direction_bits_word(&mut self) {
        self.overworld_transition_mut().clear_direction_bits_word();
    }

    pub(crate) fn and_screen_transition_direction_bits(&mut self, value: u8) {
        self.overworld_transition_mut().and_direction_bits(value);
    }

    pub(crate) fn or_screen_transition_direction_bits(&mut self, value: u8) {
        self.overworld_transition_mut().or_direction_bits(value);
    }

    pub(crate) fn or_screen_transition_direction_bits_word(&mut self, value: u16) -> u16 {
        self.overworld_transition_mut()
            .or_direction_bits_word(value)
    }

    pub(crate) fn transition_direction_enum(&self) -> u8 {
        self.game_state.world.overworld.transition.direction_enum()
    }

    pub(crate) fn set_transition_direction_enum(&mut self, value: u8) {
        self.overworld_transition_mut().set_direction_enum(value);
    }

    pub(crate) fn screen_transition(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .transition
            .screen_transition()
    }

    pub(crate) fn screen_transition_word(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .transition
            .screen_transition_word()
    }

    pub(crate) fn set_screen_transition(&mut self, value: u8) {
        self.overworld_transition_mut().set_screen_transition(value);
    }

    pub(crate) fn set_screen_transition_word(&mut self, value: u16) {
        self.overworld_transition_mut()
            .set_screen_transition_word(value);
    }

    pub(crate) fn clear_screen_transition(&mut self) {
        self.overworld_transition_mut().clear_screen_transition();
    }

    pub(crate) fn transition_counter(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .transition
            .transition_counter
    }

    pub(crate) fn set_transition_counter(&mut self, value: u8) {
        self.overworld_transition_mut()
            .set_transition_counter(value);
    }

    pub(crate) fn increment_transition_counter(&mut self) -> u8 {
        self.overworld_transition_mut()
            .increment_transition_counter()
    }

    pub(crate) fn previous_screen_transition(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .transition
            .previous_screen_transition
    }

    pub(crate) fn set_previous_screen_transition(&mut self, value: u8) {
        self.overworld_transition_mut()
            .set_previous_screen_transition(value);
    }

    pub(crate) fn set_screen_brightness(&mut self, value: u8) {
        self.display_core_mut().set_screen_brightness(value);
    }

    pub(crate) fn increment_screen_brightness(&mut self) -> u8 {
        self.display_core_mut().increment_screen_brightness()
    }

    pub(crate) fn decrement_screen_brightness(&mut self) -> u8 {
        self.display_core_mut().decrement_screen_brightness()
    }

    pub(crate) fn set_core_update_disable_flag(&mut self, value: u8) {
        self.display_core_mut().set_core_update_disable_flag(value);
    }

    pub(crate) fn set_core_update_disable_flag_word(&mut self, value: u16) {
        self.display_core_mut()
            .set_core_update_disable_flag_word(value);
    }

    pub(crate) fn clear_core_update_disable_flag(&mut self) {
        self.display_core_mut().clear_core_update_disable_flag();
    }

    pub(crate) fn increment_core_update_disable_flag(&mut self) -> u8 {
        self.display_core_mut().increment_core_update_disable_flag()
    }

    pub(crate) fn set_bg_mode(&mut self, value: u8) {
        self.display_core_mut().set_bg_mode(value);
    }

    pub(crate) fn set_main_screen_layers(&mut self, value: u8) {
        self.display_core_mut().set_main_screen_layers(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn and_main_screen_layers(&mut self, value: u8) {
        self.display_core_mut().and_main_screen_layers(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn or_main_screen_layers(&mut self, value: u8) {
        self.display_core_mut().or_main_screen_layers(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn set_sub_screen_layers(&mut self, value: u8) {
        self.display_core_mut().set_sub_screen_layers(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn clear_sub_screen_layers_word(&mut self) {
        self.display_core_mut().clear_sub_screen_layers_word();
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn and_sub_screen_layers(&mut self, value: u8) {
        self.display_core_mut().and_sub_screen_layers(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn or_sub_screen_layers(&mut self, value: u8) {
        self.display_core_mut().or_sub_screen_layers(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn set_layer_masks_word(&mut self, value: u16) {
        self.display_core_mut().set_layer_masks_word(value);
        self.mirror_display_layer_masks_to_world_transient();
    }

    pub(crate) fn set_irq_control_flag(&mut self, value: u8) {
        self.display_core_mut().set_irq_control_flag(value);
    }

    pub(crate) fn clear_irq_control_flag(&mut self) {
        self.display_core_mut().clear_irq_control_flag();
    }

    pub(crate) fn set_vertical_irq_trigger(&mut self, value: u8) {
        self.display_core_mut().set_vertical_irq_trigger(value);
    }

    pub(crate) fn advance_crystal_rotation_counter(&mut self, amount: u8) -> bool {
        self.display_core_mut()
            .advance_crystal_rotation_counter(amount)
    }

    pub(crate) fn set_mosaic_copy(&mut self, value: u8) {
        self.display_core_mut().set_mosaic_copy(value);
    }

    pub(crate) fn set_mosaic_copy_from_level_or(&mut self, mask: u8) {
        self.display_core_mut().set_mosaic_copy_from_level_or(mask);
    }

    pub(crate) fn set_mosaic_level(&mut self, value: u8) {
        self.display_core_mut().set_mosaic_level(value);
    }

    pub(crate) fn clear_mosaic_level(&mut self) {
        self.display_core_mut().clear_mosaic_level();
    }

    pub(crate) fn clear_mosaic_level_word(&mut self) {
        self.display_core_mut().clear_mosaic_level_word();
    }

    pub(crate) fn increment_mosaic_level_by(&mut self, value: u8) -> u8 {
        self.display_core_mut().increment_mosaic_level_by(value)
    }

    pub(crate) fn decrement_mosaic_level_by(&mut self, value: u8) -> u8 {
        self.display_core_mut().decrement_mosaic_level_by(value)
    }

    pub(crate) fn set_mosaic_target_level(&mut self, value: u8) {
        self.display_core_mut().set_mosaic_target_level(value);
    }

    pub(crate) fn set_mosaic_target_level_word(&mut self, value: u16) {
        self.display_core_mut().set_mosaic_target_level_word(value);
    }

    pub(crate) fn clear_mosaic_target_level(&mut self) {
        self.display_core_mut().clear_mosaic_target_level();
    }

    pub(crate) fn clear_mosaic_target_level_word(&mut self) {
        self.display_core_mut().clear_mosaic_target_level_word();
    }

    pub(crate) fn set_mosaic_direction(&mut self, value: u8) {
        self.display_core_mut().set_mosaic_direction(value);
    }

    pub(crate) fn clear_mosaic_direction(&mut self) {
        self.display_core_mut().clear_mosaic_direction();
    }

    pub(crate) fn set_travel_bird_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_travel_bird_dma_sources(upper, lower);
    }

    pub(crate) fn reset_bg_tile_animation_countdown(&mut self, value: u16) {
        self.display_core_mut()
            .reset_bg_tile_animation_countdown(value);
    }

    pub(crate) fn decrement_bg_tile_animation_countdown(&mut self) -> u16 {
        self.display_core_mut()
            .decrement_bg_tile_animation_countdown()
    }

    pub(crate) fn set_animated_tile_data_source_address(&mut self, value: u16) {
        self.display_core_mut()
            .set_animated_tile_data_source_address(value);
    }

    pub(crate) fn set_travel_bird_tile_offset(&mut self, value: u8) {
        self.display_core_mut().set_travel_bird_tile_offset(value);
    }

    zelda_bridge_accessors! {
        pub(crate) fn save_progress_mut() -> NativeSaveProgressBridgeMut {
            game_state.inventory.save_progress
        }
        pub(crate) fn mirror_warp_scratch_mut() -> NativeMirrorWarpBridgeMut {
            game_state.inventory.mirror_warp
        }
        pub(crate) fn dungeon_entrance_backup_mut() -> NativeDungeonEntranceBackupBridgeMut {
            game_state.dungeon.entrance_backup
        }
        pub(crate) fn dungeon_header_mut() -> NativeDungeonHeaderBridgeMut {
            game_state.dungeon.header
        }
        pub(crate) fn dungeon_key_slots_mut() -> NativeDungeonKeySlotsBridgeMut {
            game_state.inventory.dungeon_key_slots
        }
        pub(crate) fn dungeon_torch_mut() -> NativeDungeonTorchBridgeMut {
            game_state.dungeon.torch
        }
        pub(crate) fn dungeon_savegame_state_mut() -> NativeDungeonSavegameBridgeMut {
            game_state.dungeon.savegame_state
        }
    }

    zelda_bridge_accessors! {
        pub(crate) fn dungeon_bg2_attributes_mut() -> NativeDungeonBg2AttributeBridgeMut {
            game_state.dungeon.bg2_attributes
        }
        pub(crate) fn dungeon_stair_lists_mut() -> NativeDungeonStairListsBridgeMut {
            game_state.dungeon.stair_lists
        }
        pub(crate) fn dungeon_stair_movement_mut() -> NativeDungeonStairMovementBridgeMut {
            game_state.dungeon.stair_movement
        }
        pub(crate) fn dungeon_moving_floor_mut() -> NativeDungeonMovingFloorBridgeMut {
            game_state.dungeon.moving_floor
        }
        pub(crate) fn dungeon_room_tracking_mut() -> NativeDungeonRoomTrackingBridgeMut {
            game_state.dungeon.room_tracking
        }
        pub(crate) fn dungeon_object_tracking_mut() -> NativeDungeonObjectTrackingBridgeMut {
            game_state.dungeon.object_tracking
        }
        pub(crate) fn dungeon_doors_mut() -> NativeDungeonDoorBridgeMut {
            game_state.dungeon.doors
        }
        pub(crate) fn dungeon_room_load_mut() -> NativeDungeonRoomLoadBridgeMut {
            game_state.dungeon.room_load
        }
        pub(crate) fn dungeon_environment_mut() -> NativeDungeonEnvironmentBridgeMut {
            game_state.dungeon.environment
        }
        pub(crate) fn dungeon_room_tilemaps_mut() -> NativeDungeonRoomTilemapBridgeMut {
            game_state.dungeon.room_tilemaps
        }
        pub(crate) fn dungeon_room_items_mut() -> NativeDungeonRoomItemBridgeMut {
            game_state.dungeon.room_items
        }
        pub(crate) fn dungeon_room_effects_mut() -> NativeDungeonRoomEffectsBridgeMut {
            game_state.dungeon.room_effects
        }
        pub(crate) fn dungeon_room_parser_mut() -> NativeDungeonRoomParserBridgeMut {
            game_state.dungeon.room_parser
        }
        pub(crate) fn dungeon_room_doors_mut() -> NativeDungeonRoomDoorSetupBridgeMut {
            game_state.dungeon.door_setup
        }
        pub(crate) fn dungeon_room_runtime_mut() -> NativeDungeonRoomRuntimeBridgeMut {
            game_state.dungeon.room_runtime
        }
        pub(crate) fn dungeon_movable_blocks_mut() -> NativeDungeonMovableBlockBridgeMut {
            game_state.dungeon.movable_blocks
        }
        pub(crate) fn dungeon_map_mut() -> NativeDungeonMapDisplayBridgeMut {
            game_state.dungeon_map_display
        }
        pub(crate) fn scratch_word_mut() -> NativeDungeonScratchWordBridgeMut {
            game_state.dungeon.scratch_word
        }
        pub(crate) fn ending_scratch_mut() -> NativeDungeonScratchWordBridgeMut {
            game_state.dungeon.scratch_word
        }
        pub(crate) fn save_load_scratch_mut() -> NativeSaveLoadTransferBridgeMut {
            game_state.save_load_transfer
        }
    }

    #[track_caller]
    pub(crate) fn set_main_color(&mut self, index: usize, value: u16) {
        self.palette_buffer_mut().set_main_color(index, value);
    }

    #[track_caller]
    pub(crate) fn set_aux_color(&mut self, index: usize, value: u16) {
        self.palette_buffer_mut().set_aux_color(index, value);
    }

    /// Palette word read from ROM or a palette asset (baked constant data).
    pub(crate) fn set_main_color_asset(&mut self, index: usize, value: u16) {
        self.palette_buffer_mut().set_main_color_asset(index, value);
    }

    pub(crate) fn set_aux_color_asset(&mut self, index: usize, value: u16) {
        self.palette_buffer_mut().set_aux_color_asset(index, value);
    }

    /// Literal constant the game writes (0 clears, white fills, fixed colors).
    pub(crate) fn set_main_color_constant(&mut self, index: usize, value: u16) {
        self.palette_buffer_mut()
            .set_main_color_constant(index, value);
    }

    pub(crate) fn set_aux_color_constant(&mut self, index: usize, value: u16) {
        self.palette_buffer_mut()
            .set_aux_color_constant(index, value);
    }

    /// Copy one palette word between shadow banks, mirroring provenance.
    pub(crate) fn copy_color(
        &mut self,
        from: (zelda3_palette::Bank, usize),
        to: (zelda3_palette::Bank, usize),
    ) {
        self.palette_buffer_mut().copy_color(from, to);
    }

    /// Swap two palette words within one shadow bank, mirroring provenance.
    pub(crate) fn swap_colors(
        &mut self,
        a: (zelda3_palette::Bank, usize),
        b: (zelda3_palette::Bank, usize),
    ) {
        self.palette_buffer_mut().swap_colors(a, b);
    }

    /// Apply one of the game's pure palette transforms to a main-bank word
    /// range, updating shadow, RAM, and mirror with the same math.
    pub(crate) fn transform_main_range(
        &mut self,
        from_word: usize,
        to_word: usize,
        transform: crate::game_state::PaletteTransform,
    ) {
        self.palette_buffer_mut()
            .transform_main_range(from_word, to_word, transform);
    }

    pub(crate) fn clear_main_full(&mut self) {
        self.palette_buffer_mut().clear_main_full();
    }

    #[track_caller]
    pub(crate) fn copy_aux_visible_from(&mut self, palette: &[u8]) {
        self.palette_buffer_mut().copy_aux_visible_from(palette);
    }

    pub(crate) fn copy_aux_visible_from_tagged(
        &mut self,
        palette: &[u8],
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .copy_aux_visible_from_tagged(palette, source);
    }

    #[track_caller]
    pub(crate) fn copy_aux_full_from(&mut self, palette: &[u8]) {
        self.palette_buffer_mut().copy_aux_full_from(palette);
    }

    pub(crate) fn copy_aux_full_from_tagged(
        &mut self,
        palette: &[u8],
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .copy_aux_full_from_tagged(palette, source);
    }

    #[track_caller]
    pub(crate) fn copy_main_full_from(&mut self, palette: &[u8]) {
        self.palette_buffer_mut().copy_main_full_from(palette);
    }

    pub(crate) fn copy_main_full_from_tagged(
        &mut self,
        palette: &[u8],
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .copy_main_full_from_tagged(palette, source);
    }

    pub(crate) fn set_sp0l(&mut self, value: u8) {
        self.palette_buffer_mut().set_sp0l(value);
    }

    pub(crate) fn set_sp5l(&mut self, value: u8) {
        self.palette_buffer_mut().set_sp5l(value);
    }

    pub(crate) fn set_sp6l(&mut self, value: u8) {
        self.palette_buffer_mut().set_sp6l(value);
    }

    pub(crate) fn set_bg_tile_animation_countdown(&mut self, value: u16) {
        self.palette_buffer_mut()
            .set_bg_tile_animation_countdown(value);
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.palette_filter_mut().set_countdown(value);
    }

    pub(crate) fn increment_countdown(&mut self) {
        self.palette_filter_mut().increment_countdown();
    }

    pub(crate) fn decrement_countdown(&mut self) {
        self.palette_filter_mut().decrement_countdown();
    }

    pub(crate) fn set_countdown_word(&mut self, value: u16) {
        self.palette_filter_mut().set_countdown_word(value);
    }

    pub(crate) fn set_darkening_or_lightening_screen(&mut self, value: u8) {
        self.palette_filter_mut()
            .set_darkening_or_lightening_screen(value);
    }

    pub(crate) fn xor_darkening_or_lightening_screen(&mut self, value: u8) {
        self.palette_filter_mut()
            .xor_darkening_or_lightening_screen(value);
    }

    pub(crate) fn set_darkening_or_lightening_screen_word(&mut self, value: u16) {
        self.palette_filter_mut()
            .set_darkening_or_lightening_screen_word(value);
    }

    pub(crate) fn set_color_math_control(&mut self, value: u8) {
        self.palette_filter_mut().set_color_math_control(value);
    }

    pub(crate) fn set_fixed_color_red(&mut self, value: u8) {
        self.palette_filter_mut().set_fixed_color_red(value);
    }

    pub(crate) fn or_fixed_color_red(&mut self, value: u8) {
        self.palette_filter_mut().or_fixed_color_red(value);
    }

    pub(crate) fn subtract_fixed_color_red(&mut self, value: u8) {
        self.palette_filter_mut().subtract_fixed_color_red(value);
    }

    pub(crate) fn set_fixed_color_green(&mut self, value: u8) {
        self.palette_filter_mut().set_fixed_color_green(value);
    }

    pub(crate) fn or_fixed_color_green(&mut self, value: u8) {
        self.palette_filter_mut().or_fixed_color_green(value);
    }

    pub(crate) fn subtract_fixed_color_green(&mut self, value: u8) {
        self.palette_filter_mut().subtract_fixed_color_green(value);
    }

    pub(crate) fn set_fixed_color_blue(&mut self, value: u8) {
        self.palette_filter_mut().set_fixed_color_blue(value);
    }

    pub(crate) fn or_fixed_color_blue(&mut self, value: u8) {
        self.palette_filter_mut().or_fixed_color_blue(value);
    }

    pub(crate) fn subtract_fixed_color_blue(&mut self, value: u8) {
        self.palette_filter_mut().subtract_fixed_color_blue(value);
    }

    pub(crate) fn set_fixed_color_component(&mut self, index: usize, value: u8) {
        self.palette_filter_mut()
            .set_fixed_color_component(index, value);
    }

    pub(crate) fn or_fixed_color_component(&mut self, index: usize, value: u8) {
        self.palette_filter_mut()
            .or_fixed_color_component(index, value);
    }

    pub(crate) fn hud_state(&self) -> HudStateRead<'_> {
        HudStateRead::new(
            &self.game_state.display.hud_runtime,
            &self.game_state.display.hud_tilemap,
        )
    }

    pub(crate) fn set_super_bomb_indicator_timer(&mut self, value: u8) {
        self.hud_mut().set_super_bomb_indicator_timer(value);
    }

    pub(crate) fn set_super_bomb_indicator_counter(&mut self, value: u8) {
        self.hud_mut().set_super_bomb_indicator_counter(value);
    }

    pub(crate) fn set_is_doing_heart_animation(&mut self, value: u8) {
        self.hud_mut().set_is_doing_heart_animation(value);
    }

    pub(crate) fn clear_is_doing_heart_animation(&mut self) {
        self.hud_mut().clear_is_doing_heart_animation();
    }

    pub(crate) fn set_heart_refill_countdown(&mut self, value: u8) {
        self.hud_mut().set_heart_refill_countdown(value);
    }

    pub(crate) fn set_heart_refill_anim_subpos(&mut self, value: u8) {
        self.hud_mut().set_heart_refill_anim_subpos(value);
    }

    pub(crate) fn set_flashing_circle_timer(&mut self, value: u8) {
        self.hud_mut().set_flashing_circle_timer(value);
    }

    pub(crate) fn set_prev_joypad_h(&mut self, value: u8) {
        self.hud_mut().set_prev_joypad_h(value);
    }

    pub(crate) fn clear_prev_joypad_h(&mut self) {
        self.hud_mut().clear_prev_joypad_h();
    }

    pub(crate) fn set_equipment_menu_exit_state(&mut self, value: u8) {
        self.hud_mut().set_equipment_menu_exit_state(value);
    }

    pub(crate) fn set_bottle_menu_row(&mut self, value: u8) {
        self.hud_mut().set_bottle_menu_row(value);
    }

    pub(crate) fn decrement_bottle_menu_row(&mut self) -> u8 {
        self.hud_mut().decrement_bottle_menu_row()
    }

    pub(crate) fn set_tick_counter(&mut self, value: u8) {
        self.hud_mut().set_tick_counter(value);
    }

    pub(crate) fn set_hud_floor_changed_timer(&mut self, value: u16) {
        self.game_state
            .display
            .hud_tilemap
            .set_floor_changed_timer(value);
        write_le_u16(&mut self.ram, HUD_FLOOR_CHANGED_TIMER, value);
        self.debug_assert_hud_tilemap_matches_ram();
    }

    pub(crate) fn clear_floor_changed_timer_low(&mut self) {
        self.game_state
            .display
            .hud_tilemap
            .clear_floor_changed_timer_low();
        self.ram[HUD_FLOOR_CHANGED_TIMER] = 0;
        self.debug_assert_hud_tilemap_matches_ram();
    }

    pub(crate) fn set_hud_tile_word(&mut self, tile: usize, value: u16) {
        let offset = tile * 2;
        if offset + 1 < MOVING_WALL_REPLACEMENT_BUFFER - HUD_TILE_INDICES_BUFFER {
            self.game_state
                .display
                .hud_tilemap
                .set_tile_word(tile, value);
            write_le_u16(&mut self.ram, HUD_TILE_INDICES_BUFFER + offset, value);
            self.debug_assert_hud_tilemap_matches_ram();
        }
    }

    pub(crate) fn initialize_default_hud_inventory_order(&mut self, count: usize) {
        self.hud_inventory_order_bridge_mut()
            .initialize_default_order(count);
    }

    pub(crate) fn swap_hud_inventory_order_items(&mut self, old_pos: usize, new_pos: usize) {
        self.hud_inventory_order_bridge_mut()
            .swap_items(old_pos, new_pos);
    }

    pub(crate) fn pause_intro_triangle_motion(&mut self) {
        self.intro_scene_bridge_mut().pause_triangle_motion();
    }

    pub(crate) fn start_triforce_countdown(&mut self, value: u16) {
        self.intro_scene_bridge_mut().set_triforce_countdown(value);
    }

    pub(crate) fn decrement_triforce_countdown(&mut self) {
        self.intro_scene_bridge_mut().decrement_triforce_countdown();
    }

    pub(crate) fn clear_ending_palace_death_count_digit_step(&mut self) {
        self.ending_credit_bridge_mut()
            .clear_palace_death_count_digit_step();
    }

    pub(crate) fn set_ending_palace_death_count_digit_step(&mut self, value: u16) {
        self.ending_credit_bridge_mut()
            .set_palace_death_count_digit_step(value);
    }

    pub(crate) fn advance_ending_palace_death_count_digit_step(&mut self) {
        self.ending_credit_bridge_mut()
            .advance_palace_death_count_digit_step();
    }

    pub(crate) fn set_ending_death_count_digit_tile_base(&mut self, value: u16) {
        self.ending_credit_bridge_mut()
            .set_death_count_digit_tile_base(value);
    }

    pub(crate) fn set_aux_bg_subset_pack(&mut self, index: usize, value: u8) {
        self.ram[AUX_BG_SUBSET_0 + index] = value;
    }

    pub(crate) fn copy_to_primary_decompression_buffer(&mut self, data: &[u8]) {
        GraphicsDecompressionScratch::copy_to_primary_buffer(&mut self.ram, data);
    }

    pub(crate) fn animated_tile_dma_source_bytes(&self) -> &[u8] {
        self.game_state
            .display
            .animated_tile_dma_source_bytes(&self.ram)
    }

    pub(crate) fn secondary_stripe_upload_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .secondary_stripe_upload_buffer(&self.ram)
    }

    pub(crate) fn background_character_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .background_character_buffer(&self.ram)
    }

    pub(crate) fn background_character_secondary_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .background_character_secondary_buffer(&self.ram)
    }

    pub(crate) fn background_character_half_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .background_character_half_buffer(&self.ram)
    }

    pub(crate) fn intro_actor(&self, slot: usize) -> IntroActorRead<'_> {
        IntroActorRead::new(&self.game_state.ending.intro_actors, slot)
    }

    pub(crate) fn intro_actor_mut(&mut self, slot: usize) -> NativeIntroActorBridgeMut<'_> {
        NativeIntroActorBridgeMut::new(
            &mut self.game_state.ending.intro_actors,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn quake_bolt(&self, slot: usize) -> QuakeBoltSlotState {
        self.game_state.effects.quake_bolts.slot(slot)
    }

    pub(crate) fn quake_bolt_mut(&mut self, slot: usize) -> NativeQuakeBoltBridgeMut<'_> {
        NativeQuakeBoltBridgeMut::new(
            &mut self.game_state.effects.quake_bolts,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn happiness_pond_rupee(&self, slot: usize) -> HappinessPondRupeeSlotState {
        self.game_state.effects.happiness_pond_rupees.rupee(slot)
    }

    pub(crate) fn happiness_pond_rupee_mut(
        &mut self,
        slot: usize,
    ) -> NativeHappinessPondRupeeBridgeMut<'_> {
        NativeHappinessPondRupeeBridgeMut::new(
            &mut self.game_state.effects.happiness_pond_rupees,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn bird_travel_destination(&self, slot: usize) -> BirdTravelDestinationState {
        self.game_state
            .world
            .overworld
            .bird_travel_destinations
            .destination(slot)
    }

    pub(crate) fn set_bird_travel_destination(&mut self, slot: usize, x: u16, y: u16) {
        self.bird_travel_destination_bridge_mut()
            .set_destination(slot, x, y);
    }

    pub(crate) fn clear_bird_travel_destination(&mut self, slot: usize) {
        self.bird_travel_destination_bridge_mut()
            .clear_destination(slot);
    }

    pub(crate) fn moldorm_history(&self, slot: usize) -> HistoryPositionState {
        self.game_state
            .effects
            .sprite_histories
            .moldorm_history(slot)
    }

    pub(crate) fn moldorm_history_mut(&mut self, slot: usize) -> NativeMoldormHistoryBridgeMut<'_> {
        NativeMoldormHistoryBridgeMut::new(
            &mut self.game_state.effects.sprite_histories,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn beamos_laser_history(&self, slot: usize) -> HistoryPositionState {
        self.game_state
            .effects
            .sprite_histories
            .beamos_laser_history(slot)
    }

    pub(crate) fn beamos_laser_history_mut(
        &mut self,
        slot: usize,
    ) -> NativeBeamosLaserHistoryBridgeMut<'_> {
        NativeBeamosLaserHistoryBridgeMut::new(
            &mut self.game_state.effects.sprite_histories,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn lanmola_segment_motion(&self, slot: usize) -> LanmolaSegmentMotionState {
        self.game_state
            .effects
            .sprite_histories
            .lanmola_segment_motion(slot)
    }

    pub(crate) fn lanmola_segment_motion_mut(
        &mut self,
        slot: usize,
    ) -> NativeLanmolaSegmentMotionBridgeMut<'_> {
        NativeLanmolaSegmentMotionBridgeMut::new(
            &mut self.game_state.effects.sprite_histories,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn lanmola_flat_trail_entry(&self, slot: usize) -> LanmolaFlatTrailEntry {
        lanmola_flat_trail_entry_from_ram(&self.ram, slot)
    }

    pub(crate) fn draw_scratch_position_mut(
        &mut self,
    ) -> NativeSpriteDrawWorkPositionBridgeMut<'_> {
        NativeSpriteDrawWorkPositionBridgeMut::new(
            &mut self.game_state.sprites.draw_hitbox_work,
            &mut self.ram,
        )
    }

    pub(crate) fn hitbox_scratch_offset_mut(
        &mut self,
    ) -> NativeSpriteHitboxWorkOffsetBridgeMut<'_> {
        NativeSpriteHitboxWorkOffsetBridgeMut::new(
            &mut self.game_state.sprites.draw_hitbox_work,
            &mut self.ram,
        )
    }

    pub(crate) fn set_messaging_render_buffer_word(&mut self, index: usize, value: u16) {
        self.messaging_render_buffer_mut().set_word(index, value);
    }

    pub(crate) fn set_messaging_render_buffer_word_at_byte_offset(
        &mut self,
        byte_offset: usize,
        value: u16,
    ) {
        self.messaging_render_buffer_mut()
            .set_word_at_byte_offset(byte_offset, value);
    }

    pub(crate) fn xor_messaging_render_buffer_mask(&mut self, offset: usize, mask: u8) {
        self.messaging_render_buffer_mut().xor_mask(offset, mask);
    }

    pub(crate) fn clear_messaging_render_buffer_mask(&mut self, offset: usize, mask: u8) {
        self.messaging_render_buffer_mut().clear_mask(offset, mask);
    }

    pub(crate) fn clear_messaging_render_buffer_range(&mut self, byte_count: usize) {
        self.messaging_render_buffer_mut().clear_range(byte_count);
    }

    pub(crate) fn fill_messaging_render_buffer_word_range(
        &mut self,
        start_index: usize,
        count: usize,
        value: u16,
    ) {
        self.messaging_render_buffer_mut()
            .fill_word_range(start_index, count, value);
    }

    pub(crate) fn arrghus_puff_home_position(&self, puff_slot: usize) -> BossHomePositionRead {
        // arrghus_handle_puffs writes each puff's home into the overlord slot array
        // (OVERLORD_X_LO+slot+7 .. by SNES byte reuse — the same bytes as the armos
        // x_hi/y_hi/gen2/floor home array), and that is where C reads it from. Read it
        // from RAM directly — NOT from the persisted `boss_home_positions` native, which
        // production never repopulates mid-frame (only tests drive the `_mut` bridge), so
        // it would return a stale value. Mirrors armos_knight_home_position.
        use crate::game_state::constants::{
            OVERLORD_GEN1, OVERLORD_GEN3, OVERLORD_X_LO, OVERLORD_Y_LO,
        };
        let s = puff_slot + 7;
        BossHomePositionRead::from_xy_bytes(
            self.ram[OVERLORD_X_LO + s],
            self.ram[OVERLORD_Y_LO + s],
            self.ram[OVERLORD_GEN1 + s],
            self.ram[OVERLORD_GEN3 + s],
        )
    }

    pub(crate) fn armos_knight_home_position(&self, slot: usize) -> BossHomePositionRead {
        // The armos coordinator overlord stores each knight's formation/home position in
        // the OVERLORD slot array (0xb10+: x_high, y_high, gen2, floor) by SNES byte reuse,
        // and that is where C reads it from. Read it from there — NOT from the persisted
        // `boss_home_positions` native state, which nothing populates for armos (the
        // coordinator writes the overlord slots), so it would return stale garbage.
        let slot_view = self.overlord_slot_view(slot);
        BossHomePositionRead::from_xy_bytes(
            slot_view.x_high(),
            slot_view.y_high(),
            slot_view.gen2(),
            slot_view.floor(),
        )
    }

    pub(crate) fn arrghus_puff_home_position_mut(
        &mut self,
        puff_slot: usize,
    ) -> NativeArrghusPuffHomePositionBridgeMut<'_> {
        NativeArrghusPuffHomePositionBridgeMut::new(
            &mut self.game_state.sprites.boss_home_positions,
            &mut self.ram,
            puff_slot,
        )
    }

    pub(crate) fn armos_knight_home_position_mut(
        &mut self,
        slot: usize,
    ) -> NativeArmosKnightHomePositionBridgeMut<'_> {
        NativeArmosKnightHomePositionBridgeMut::new(
            &mut self.game_state.sprites.boss_home_positions,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn enemy_damage_subclass_table_mut(
        &mut self,
    ) -> NativeEnemyDamageSubclassTableBridgeMut<'_> {
        NativeEnemyDamageSubclassTableBridgeMut::new(
            &mut self.game_state.sprites.enemy_damage_subclasses,
            &mut self.ram,
        )
    }

    /// Byte extent of the SAVELOAD_HDMA_TABLE scratch region (0x1b00..0x1cd0).
    /// This is a save-time scratch buffer: `save_snes_state` projects the live
    /// spotlight dynamic table into it so the loader can rebuild HDMA_TABLE_DYNAMIC.
    /// Outside a save it has no behavioral meaning, but the checkpoint stores
    /// whatever the projection wrote, so a resumed run's WRAM at 0x1b00 differs
    /// from a from-scratch run. Capturing/restoring the pristine bytes makes resume
    /// byte-faithful without affecting the dynamic-table reconstruction (which has
    /// already happened via restore_spotlight_hdma_from_saveload_buffer +
    /// sync_native_game_state_from_ram by the time we overwrite this region back).
    pub const SAVELOAD_HDMA_SCRATCH_LEN: usize = SpotlightHdmaState::SAVELOAD_SCRATCH_LEN;

    /// Byte extent of the live spotlight HDMA dynamic table (0x1dba0, 240 words).
    /// The C-style saveload reconstructs the native spotlight dynamic table from
    /// the LOSSY SAVELOAD_HDMA_TABLE projection (only 224 words round-trip, and the
    /// native sync re-derives entries), so a resumed run's native spotlight backing
    /// can differ from a continuous run. We capture the live table bytes and, on
    /// load, both restore them to RAM and re-sync the native model directly from
    /// them, making the spotlight backing byte-faithful.
    pub const HDMA_DYNAMIC_TABLE_LEN: usize = SpotlightHdmaState::DYNAMIC_TABLE_LEN;

    pub(crate) fn sync_native_game_state_from_ram(&mut self) {
        // The palette-provenance mirror is derived metadata that RAM cannot
        // reconstruct (load_from_ram defaults it to all-Unknown). The palette
        // shadow is only ever written through the provenance-aware bridge, so
        // the mirror stays valid across a native resync — carry it over
        // instead of poisoning it (the ZELDA3_PALETTE_PROVENANCE_CHECK gate
        // would catch any drift this assumption misses). The one exception is a
        // full-state snapshot restore, which bulk-writes the palette shadow
        // outside the bridge; `load_snes_state` reconstitutes the mirror after
        // resync to cover it.
        let palette_provenance = std::mem::take(&mut self.game_state.display.palette_provenance);
        self.game_state = GameState::load_from_ram(&self.ram);
        self.game_state.display.palette_provenance = palette_provenance;
    }

    pub(crate) fn project_native_game_state_to_ram(&mut self) {
        self.game_state.write_to_ram(&mut self.ram);
    }

    pub fn new() -> Self {
        let mut state = Self {
            // ROM $008900 establishes this descriptor during reset, before
            // the first NMI can consume low WRAM as a DMA source. The normal
            // ported initialization routine is intentionally deferred for
            // Snes9x shows that the first visible NMI reads `00 80 00` here.
            // The ROM's earlier reset descriptor ends in `$19`, but that byte
            // has already been consumed/replaced before this NMI boundary.
            ram: {
                let mut ram = vec![0; WRAM_SIZE];
                ram[0x0001] = 0x80;
                // Snes9x power-on RAM is `$55`; only these two otherwise
                // untouched first-NMI DMA windows remain observable before
                // the ROM initializes them (ROM $008b01/$008b25).
                ram[0xbd40..0xbdc0].fill(0x55);
                ram
            },
            game_state: GameState::default(),
            rom_random_replay: crate::rom_random::RomRandomReplay::default(),
            sram: vec![0; SRAM_SIZE],
            ppu: PpuState::new(),
            vram_chr_source: crate::chr_source::VramChrSourceTable::new(),
            vram_chr_preview_source: crate::chr_source::VramChrSourceTable::new(),
            animated_tile_pack: 0,
            bg3_vwf_glyph_runs: Vec::new(),
            bg3_vwf_glyph_run_dialogue_offsets: Vec::new(),
            bg3_vwf_glyph_run_dialogue_message_id: 0,
            dialogue_scroll_continuation: DialogueScrollContinuation::IDLE,
            dialogue_fast_forward_hold_pending: false,
            dialogue_vwf_completion_stops_before_scroll: false,
            dialogue_vwf_handler_completed_at_endpoint: false,
            save_menu_initialization_completed_pending: false,
            dialogue_fast_forward_hold_active: false,
            dialogue_live_message_read_position_target: None,
            game_over_text_render_calls_remaining: 0,
            game_over_text_render_call_in_flight: false,
            rom_damage_check_y_register: None,
            dialogue_vwf_handler_entry_phase: messaging::VwfHandlerEntryPhase::default(),
            dialogue_vwf_glyph_cpu_phase: messaging::VwfGlyphCpuPhase::Ready,
            published_bg3_vwf_glyph_runs: Vec::new(),
            published_bg3_vwf_glyph_run_dialogue_offsets: Vec::new(),
            published_dialogue_msg_read_pos: 0,
            published_dialogue_message_id: 0,
            dialogue_scroll_frozen_scanout: None,
            dialogue_scroll_completion_scanout: None,
            dialogue_scroll_completion_staged: None,
            dialogue_scanout_ownership: DialogueScanoutOwnership::SNAPSHOT,
            dma: DmaState::new(),
            frame_ctr_dbg: 0,
            previous_host_controller_input: 0,
            rom: Vec::new(),
            assets: None,
            gloves_color: default_gloves_color(),
            initialized: false,
            apply_links_movement_to_camera_called: false,
            ground_apress_defers_atomic_item_receipt: false,
            item_receipt_completion_live_link_dma_host: None,
            wanted_zelda_features: 0,
            state_recorder: StateRecorder::default(),
            dialogue_blk_index: 0,
            dialogue_font_blk_index: 0,
            dialogue_flags: 0,
            rom_startup_timing: false,
            original_timing_owner: OriginalTimingOwnerState::Disabled,
            original_timing_cold_start_eligible: true,
            original_timing_host_dispatch_active: false,
            original_timing_semantic_receipts: None,
            original_timing_sprite_main_return_claims_remaining: None,
            original_timing_nmi_publication_pending: false,
            original_timing_pending_nmi_update_gate: None,
            original_timing_pending_nmi_ppu_register_operands: None,
            original_timing_expected_nmi_update_gates: Vec::new(),
            original_timing_expected_nmi_ppu_register_operands: Vec::new(),
            original_timing_scheduled_nmi_accepted_at_host_return: false,
            original_timing_dungeon_exit_spotlight_entry_return_pending: false,
            original_timing_pre_dungeon_return_pending: None,
            original_timing_presented_audio: None,
            original_timing_audio_shadow_result: None,
            original_timing_bg_tilemap_shadow_result: None,
            original_timing_dialogue_text_shadow_result: None,
            original_timing_bg_scroll_shadow_result: None,
            original_timing_mode7_transform_shadow_result: None,
            original_timing_window_mask_shadow_result: None,
            active_presented_bg_scroll: None,
            active_presented_mode7_transform: None,
            active_presented_window_mask: None,
            original_timing_last_oracle_host_call: None,
            interrupted_nmi_prepare_obj_cache_vram: None,
            rom_load_partial_nmi_this_frame: false,
            game_over_iris_goal_scanout_closed_pending: false,
            intro_initialization_work_frames_pending: 0,
            intro_initialization_reset_obj_control_pending: false,
            rom_reset_frame_delay: 0,
            intro_memory_darken_frame_delay: 0,
            save_quit_reset_hold: false,
            save_quit_reset_state_published: false,
            save_quit_reset_writes_applied: false,
            pending_module09_frame_advance: None,
            pending_overworld_sprite_reload_slots: None,
            pending_selected_game_entrance: None,
            pending_overworld_sprite_activations: None,
            overworld_proximity_scan_saved_scroll: None,
            intro_poly_thread_initialization_phase: 0,
            attract_init_graphics_phase: 0,
            attract_first_story_render_delay: 0,
            game_execution_scheduler: GameExecutionScheduler::default(),
            active_dungeon_sprite_main_return: None,
            active_module09_sprite_main_return: None,
            dungeon_cached_sprite_cpu_interruption_pending: None,
            dungeon_cached_sprite_cpu_interruption_boundary: None,
            dungeon_peg_attribute_flip_pending: None,
            dungeon_state_12_caller_suffix_nmi_pending: false,
            dungeon_landing_cpu_advance_pending: None,
            dungeon_landing_spotlight_reset_prefix_scanlines: None,
            active_dungeon_landing_spotlight_reset_prefix_scanlines: None,
            dungeon_post_sprite_main_return_pending: false,
            dungeon_nmi_prepare_sprites_return_pending: false,
            dungeon_palette_cpu_advance_pending: None,
            dungeon_room_load_cpu_schedule: None,
            dungeon_submodule_cpu_schedule: None,
            module09_cpu_schedule: None,
            sprite_main_cpu_boundary: None,
            sprite_main_cpu_nmi_slices: 0,
            sprite_main_cpu_caller: SpriteMainCpuCaller::default(),
            dungeon_quadrant_cpu_continuation_active: false,
            dungeon_room_load_module_suffix_nmi_slices: 0,
            spiral_stair_return_player_control_pending: false,
            spiral_stair_return_oam_publication_host_frame: None,
            dungeon_state_13_pre_main_publication_host_frame: None,
            dungeon_state_13_recurring_main_publication_host_frame: None,
            dungeon_state_13_caller_return_publication_host_frame: None,
            dungeon_state_13_atomic_caller_return_publication_host_frame: None,
            dungeon_faded_filter_palette_completion_host_frame: None,
            dungeon_faded_filter_caller_return_publication_host_frame: None,
            dungeon_post_landing_leading_nmi_room: None,
            spiral_stair_return_player_oam_scanout: None,
            next_display_vram_generation: DisplayVramGeneration::default(),
            next_display_cgram_override: None,
            publish_live_hud_vram_on_next_capture: false,
            next_display_animated_bg_scanout_generation: None,
            next_display_bg_scroll_generation: DisplayBgScrollGeneration::default(),
            next_display_obj_scanout_generation: None,
            next_display_obj_scanout_provenance: None,
            next_display_obj_memory_generation: None,
            next_display_obj_cache_vram: None,
            next_display_interrupted_item_receipt_obj_cache: false,
            enemy_drop_item_graphics_live_extended_oam_pending: false,
            enemy_drop_item_graphics_deferred_sound_effect_2: None,
            link_obj_dma_completed_this_frame: false,
            main_loop_sprite_preparation_completed: false,
            pending_main_loop_common_suffix: None,
            next_display_spotlight_scanout: None,
            spotlight_scanout_after_active_field: None,
            interrupted_dungeon_submodule_publication: None,
            interrupted_dungeon_spotlight_build_in_flight: None,
            last_completed_interrupted_dungeon_spotlight_scanout: None,
            dungeon_landing_entry_started_after_leading_nmi: false,
            dungeon_landing_goal_display_handoff: DungeonLandingGoalDisplayHandoff::None,
            active_display_obj_generation: DisplayObjGeneration::default(),
            next_overworld_sprite_reload_entry_phase: None,
            joypad_sampled_before_main: false,
            audio_nmi_processed_before_main: false,
            audio_after_publication_ambient_nmi: None,
            dungeon_exit_spotlight_cpu_entry_envelope: None,
            overworld_spotlight_cpu_entry_envelope: None,
            dungeon_landing_goal_transition_pending: false,
            normal_dialogue_following_main_nmi_uses_host_animated_bg_operands: None,
            next_core_nmi_active_scanout_uses_host_animated_bg_operands: None,
            pending_dialogue_initialization_schedule: None,
            intro_poly_upload_delay: 0,
            display_snapshot: None,
            display_snapshot_epoch: 0,
            active_effective_dma_writes: None,
            resident_oam_dma: None,
            last_presented_oam: None,
            oam_law_pending: None,
            oam_law_visible: None,
            oam_law_entry_frame_counter: None,
            last_presented_cgram: None,
            last_presented_obj_vram: None,
            presented_history_host_frame: None,
            staged_presented_oam: None,
            staged_presented_cgram: None,
            staged_presented_obj_vram: None,
            last_presented_vram_chr_source: None,
            last_presented_vram_chr_preview_source: None,
            staged_presented_vram_chr_source: None,
            staged_presented_vram_chr_preview_source: None,
            deferred_display_snapshot: None,
            attract_map_hdma_projection_before: None,
            pre_main_graphics_dma: None,
            debug_display_publication_candidates: Vec::new(),
            debug_display_publication_context: None,
            debug_display_cgram_candidates: Vec::new(),
            pre_nmi_animated_bg_scanout: None,
            nmi_forced_blank_scanlines_pending: 0,
            legacy_nmi_forced_blank_from_scanline_pending: None,
            nmi_active_display_blanking_candidate: NmiActiveDisplayBlanking::default(),
            active_display_force_blank_event: None,
            pending_file_select_force_blank_output_scanline: None,
            last_sprite_main_timing_workload: None,
            nmi_poly_upload_deferred: 0,
            nmi_poly_upload_started: false,
            nmi_poly_deferred_upload_bypasses_latch: false,
            nmi_poly_upload_from_deferred: false,
            obj_vram_latch_generation: 0,
            cgram_upload_latch: None,
            snes9x_poly_scheduler_counter: 0,
            snes9x_hold_intro_step_this_frame: false,
            snes9x_intro_step_carry_phase_active: false,
            snes9x_intro_step_hold_alternate: false,
            last_poly_work: PolyWorkMetrics::default(),
            poly_job_in_flight: false,
            poly_dungeon_thread_startup_hold: None,
            triforce_room_scroll_this_iteration: false,
            triforce_poly_shadow_ram: None,
            poly_dungeon_frames_rendered: 0,
            poly_shadow_run: None,
            poly_shadow_master: 0,
            poly_shadow_hosts: 0,
            poly_completed_upload: None,
            poly_next_host_swap_master: None,
            poly_next_host_nmi_state: None,
            poly_receipt_gates_prev: None,
            poly_receipt_gates_cur: None,
            poly_dungeon_activation_host: 0,
            original_timing_carried_suffix_completion_pending: false,
            original_timing_sprite_main_return_claim_scope_site: None,
            original_timing_host_iteration_uninterrupted: false,
            poly_dungeon_current_frame_slices: 0,
            poly_job_hold_frames: 0,
            intro_title_fade_poly_phase: 0,
            intro_title_fade_defer_suffix_this_frame: false,
            intro_title_fade_suffix_pending: false,
            intro_bg_fade_carry_frames: 0,
            intro_bg_fade_poly_phase: 0,
            intro_bg_fade_defer_suffix_this_frame: false,
            intro_bg_fade_suffix_pending: false,
            intro_zelda_fade_transition_pending: false,
            intro_poly_thread_teardown_pending: false,
            ending_coords: sprite::PrepOamCoordsRet::default(),
            intro_poly_vram_history: Vec::new(),
            audio: audio::AudioState::default(),
            emu_memory_ptr: None,
            emu_runframe: None,
            emu_syncall: None,
        };
        state.initialize();
        state.sync_native_game_state_from_ram();
        state.assert_native_frame_state_matches_ram();
        state.assert_native_world_location_state_matches_ram();
        state.assert_native_display_state_matches_ram();
        state.assert_native_save_progress_matches_ram();
        state
    }

    /// `zelda_initialize` allocates the runtime devices and resets DMA/PPU.
    pub fn initialize(&mut self) {
        self.zelda_initialize();
    }

    pub fn reset(&mut self, preserve_sram: bool) {
        self.zelda_reset(preserve_sram);
    }

    pub fn zelda_initialize(&mut self) {
        self.dma.reset();
        self.ppu.reset();
        self.initialized = true;
    }

    pub fn zelda_reset(&mut self, preserve_sram: bool) {
        self.frame_ctr_dbg = 0;
        self.original_timing_owner = if self.rom_startup_timing {
            OriginalTimingOwnerState::PendingColdStart
        } else {
            OriginalTimingOwnerState::Disabled
        };
        self.original_timing_cold_start_eligible = true;
        self.original_timing_host_dispatch_active = false;
        self.original_timing_semantic_receipts = None;
        self.original_timing_sprite_main_return_claims_remaining = None;
        self.original_timing_nmi_publication_pending = false;
        self.original_timing_pending_nmi_update_gate = None;
        self.original_timing_pending_nmi_ppu_register_operands = None;
        self.original_timing_expected_nmi_update_gates.clear();
        self.original_timing_expected_nmi_ppu_register_operands
            .clear();
        self.original_timing_scheduled_nmi_accepted_at_host_return = false;
        self.original_timing_dungeon_exit_spotlight_entry_return_pending = false;
        self.original_timing_pre_dungeon_return_pending = None;
        self.original_timing_presented_audio = None;
        self.original_timing_audio_shadow_result = None;
        self.original_timing_bg_tilemap_shadow_result = None;
        self.original_timing_dialogue_text_shadow_result = None;
        self.original_timing_bg_scroll_shadow_result = None;
        self.original_timing_mode7_transform_shadow_result = None;
        self.original_timing_window_mask_shadow_result = None;
        self.original_timing_last_oracle_host_call = None;
        self.interrupted_nmi_prepare_obj_cache_vram = None;
        self.previous_host_controller_input = 0;
        self.dma.reset();
        self.ppu.reset();
        self.bg3_vwf_glyph_runs.clear();
        self.bg3_vwf_glyph_run_dialogue_offsets.clear();
        self.ram.fill(0);
        if !preserve_sram {
            self.sram.fill(0);
        }
        self.zelda_restore_music_after_load_locked(true);
        self.initialized = true;
        self.apply_links_movement_to_camera_called = false;
        self.intro_initialization_work_frames_pending = 0;
        self.intro_initialization_reset_obj_control_pending = false;
        self.rom_reset_frame_delay = if self.rom_startup_timing {
            configured_rom_reset_frame_delay()
        } else {
            0
        };
        self.intro_memory_darken_frame_delay = 0;
        self.save_quit_reset_hold = false;
        self.save_quit_reset_state_published = false;
        self.save_quit_reset_writes_applied = false;
        self.intro_poly_thread_initialization_phase = 0;
        self.attract_init_graphics_phase = 0;
        self.attract_first_story_render_delay = 0;
        self.game_execution_scheduler.reset();
        self.dungeon_submodule_cpu_schedule = None;
        self.dungeon_post_sprite_main_return_pending = false;
        self.dungeon_nmi_prepare_sprites_return_pending = false;
        self.dungeon_quadrant_cpu_continuation_active = false;
        self.joypad_sampled_before_main = false;
        self.audio_nmi_processed_before_main = false;
        self.audio_after_publication_ambient_nmi = None;
        self.main_loop_sprite_preparation_completed = false;
        self.pending_main_loop_common_suffix = None;
        self.dungeon_landing_goal_transition_pending = false;
        self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
        self.active_dungeon_landing_spotlight_reset_prefix_scanlines = None;
        self.normal_dialogue_following_main_nmi_uses_host_animated_bg_operands = None;
        self.next_core_nmi_active_scanout_uses_host_animated_bg_operands = None;
        self.pending_dialogue_initialization_schedule = None;
        self.dialogue_fast_forward_hold_pending = false;
        self.dialogue_fast_forward_hold_active = false;
        self.dialogue_live_message_read_position_target = None;
        self.game_over_text_render_calls_remaining = 0;
        self.game_over_text_render_call_in_flight = false;
        self.nmi_poly_upload_deferred = 0;
        self.nmi_poly_upload_started = false;
        self.nmi_poly_deferred_upload_bypasses_latch = false;
        self.nmi_poly_upload_from_deferred = false;
        self.obj_vram_latch_generation = 0;
        self.poly_completed_upload = None;
        self.snes9x_poly_scheduler_counter = 0;
        self.snes9x_hold_intro_step_this_frame = false;
        self.snes9x_intro_step_carry_phase_active = false;
        self.snes9x_intro_step_hold_alternate = false;
        self.poly_job_in_flight = false;
        self.poly_job_hold_frames = 0;
        self.intro_title_fade_poly_phase = 0;
        self.intro_title_fade_defer_suffix_this_frame = false;
        self.intro_title_fade_suffix_pending = false;
        self.intro_bg_fade_carry_frames = 0;
        self.intro_bg_fade_poly_phase = 0;
        self.intro_bg_fade_defer_suffix_this_frame = false;
        self.intro_bg_fade_suffix_pending = false;
        self.intro_zelda_fade_transition_pending = false;
        self.intro_poly_thread_teardown_pending = false;
        self.intro_poly_vram_history.clear();
        self.sync_overworld_map16_state_from_ram();
        self.display_snapshot = None;
        self.active_effective_dma_writes = None;
        self.resident_oam_dma = None;
        self.last_presented_oam = None;
        self.oam_law_pending = None;
        self.oam_law_visible = None;
        self.last_presented_cgram = None;
        self.last_presented_obj_vram = None;
        self.presented_history_host_frame = None;
        self.staged_presented_oam = None;
        self.staged_presented_cgram = None;
        self.staged_presented_obj_vram = None;
        self.last_presented_vram_chr_source = None;
        self.last_presented_vram_chr_preview_source = None;
        self.staged_presented_vram_chr_source = None;
        self.staged_presented_vram_chr_preview_source = None;
        self.deferred_display_snapshot = None;
        self.emu_synchronize_whole_state();
    }

    pub fn set_rom_startup_timing(&mut self, enabled: bool) {
        if enabled && self.rom_startup_timing {
            return;
        }
        self.rom_startup_timing = enabled;
        if !enabled {
            if self.frame_ctr_dbg != 0 {
                self.original_timing_cold_start_eligible = false;
            }
            self.original_timing_owner = OriginalTimingOwnerState::Disabled;
            self.original_timing_semantic_receipts = None;
            self.original_timing_dungeon_exit_spotlight_entry_return_pending = false;
            self.original_timing_pre_dungeon_return_pending = None;
            self.original_timing_presented_audio = None;
            self.original_timing_audio_shadow_result = None;
            self.original_timing_bg_tilemap_shadow_result = None;
            self.original_timing_dialogue_text_shadow_result = None;
            self.original_timing_bg_scroll_shadow_result = None;
            self.original_timing_mode7_transform_shadow_result = None;
            self.original_timing_window_mask_shadow_result = None;
            self.original_timing_last_oracle_host_call = None;
            self.zelda_set_rom_startup_audio_phase(false);
            self.rom_reset_frame_delay = 0;
            self.intro_initialization_work_frames_pending = 0;
            self.intro_initialization_reset_obj_control_pending = false;
            self.intro_memory_darken_frame_delay = 0;
            self.save_quit_reset_hold = false;
            self.save_quit_reset_state_published = false;
            self.save_quit_reset_writes_applied = false;
            self.dialogue_vwf_handler_completed_at_endpoint = false;
            self.pending_module09_frame_advance = None;
            self.pending_overworld_sprite_reload_slots = None;
            self.pending_selected_game_entrance = None;
            self.pending_overworld_sprite_activations = None;
            self.intro_poly_thread_initialization_phase = 0;
            self.attract_init_graphics_phase = 0;
            self.attract_first_story_render_delay = 0;
            self.game_execution_scheduler.reset();
            self.dungeon_room_load_cpu_schedule = None;
            self.dungeon_submodule_cpu_schedule = None;
            self.module09_cpu_schedule = None;
            self.sprite_main_cpu_boundary = None;
            self.sprite_main_cpu_nmi_slices = 0;
            self.sprite_main_cpu_caller = SpriteMainCpuCaller::default();
            self.original_timing_sprite_main_return_claims_remaining = None;
            self.dungeon_quadrant_cpu_continuation_active = false;
            self.dungeon_room_load_module_suffix_nmi_slices = 0;
            self.joypad_sampled_before_main = false;
            self.audio_nmi_processed_before_main = false;
            self.audio_after_publication_ambient_nmi = None;
            self.pending_main_loop_common_suffix = None;
            self.dungeon_landing_goal_transition_pending = false;
            self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
            self.active_dungeon_landing_spotlight_reset_prefix_scanlines = None;
            self.normal_dialogue_following_main_nmi_uses_host_animated_bg_operands = None;
            self.next_core_nmi_active_scanout_uses_host_animated_bg_operands = None;
            self.pending_dialogue_initialization_schedule = None;
            self.nmi_poly_upload_deferred = 0;
            self.nmi_poly_upload_started = false;
            self.nmi_poly_deferred_upload_bypasses_latch = false;
            self.nmi_poly_upload_from_deferred = false;
            self.obj_vram_latch_generation = 0;
            self.snes9x_poly_scheduler_counter = 0;
            self.snes9x_hold_intro_step_this_frame = false;
            self.snes9x_intro_step_carry_phase_active = false;
            self.snes9x_intro_step_hold_alternate = false;
            self.poly_job_in_flight = false;
            self.poly_job_hold_frames = 0;
            self.intro_title_fade_poly_phase = 0;
            self.intro_title_fade_defer_suffix_this_frame = false;
            self.intro_title_fade_suffix_pending = false;
            self.intro_bg_fade_carry_frames = 0;
            self.intro_bg_fade_poly_phase = 0;
            self.intro_bg_fade_defer_suffix_this_frame = false;
            self.intro_bg_fade_suffix_pending = false;
            self.intro_zelda_fade_transition_pending = false;
            self.intro_poly_thread_teardown_pending = false;
            self.intro_poly_vram_history.clear();
            self.display_snapshot = None;
            self.active_effective_dma_writes = None;
            self.resident_oam_dma = None;
            self.last_presented_oam = None;
            self.oam_law_pending = None;
            self.oam_law_visible = None;
            self.last_presented_cgram = None;
            self.last_presented_obj_vram = None;
            self.presented_history_host_frame = None;
            self.staged_presented_oam = None;
            self.staged_presented_cgram = None;
            self.staged_presented_obj_vram = None;
            self.last_presented_vram_chr_source = None;
            self.last_presented_vram_chr_preview_source = None;
            self.staged_presented_vram_chr_source = None;
            self.staged_presented_vram_chr_preview_source = None;
            self.deferred_display_snapshot = None;
            self.dungeon_landing_goal_display_handoff = DungeonLandingGoalDisplayHandoff::None;
        } else if self.original_timing_cold_start_eligible && self.frame_ctr_dbg == 0 {
            self.original_timing_owner = OriginalTimingOwnerState::PendingColdStart;
            self.zelda_set_rom_startup_audio_phase(true);
            if !self.game_state.display.has_animated_tile_data_source() {
                self.rom_reset_frame_delay = configured_rom_reset_frame_delay();
            }
        } else {
            self.original_timing_cold_start_eligible = false;
            self.original_timing_owner = OriginalTimingOwnerState::Unavailable(
                OriginalTimingUnavailableReason::ProgressedState,
            );
        }
    }

    pub(super) fn rom_startup_timing(&self) -> bool {
        self.rom_startup_timing
    }

    fn take_authoritative_module09_caller_phase(
        &mut self,
        shadow_phase: Option<ModuleCpuPhase>,
    ) -> ModuleCpuPhase {
        let shadow_phase = shadow_phase.expect("Module09/$20 caller suffix omitted its CPU phase");
        let Some(receipt) = (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live))
            .then(|| self.take_original_timing_main_loop_interruption_any())
            .flatten()
        else {
            // Authority migrates one semantic domain at a time. A host call
            // with no typed LinkOam/SpritePreparation receipt has not exposed
            // that phase through the replaceable interface, so the native
            // timing owner remains responsible for this boundary.
            return shadow_phase;
        };
        let authoritative_phase = module_cpu_phase_from_main_loop_interruption(receipt)
            .unwrap_or_else(|| panic!("Module09 cannot consume Module0F Link-position progress"));
        assert_eq!(
            shadow_phase, authoritative_phase,
            "native Module09 CPU shadow disagreed with the live semantic interruption receipt",
        );
        authoritative_phase
    }

    pub(super) fn begin_selected_game_load(&mut self) {
        let destination = self.selected_game_load_destination();
        self.enable_force_blank();
        self.game_execution_scheduler
            .schedule_selected_game_load(destination);
        // The ROM starts the heavy save-file load on this frame; its NMI is
        // PARTIAL (no Main_PrepSpritesForNmi — Snes9x holds 0xc00d here while
        // rust's game loop otherwise decrements once more on this entry frame,
        // the single event that left the BG-tile animation phase one step ahead
        // for the rest of the route, surfacing at frame 14661).
        self.rom_load_partial_nmi_this_frame = true;
    }

    fn preflight_continued_peg_attribute_flip(
        &self,
    ) -> Option<crate::DungeonPegAttributeFlipProgressReceipt> {
        let progress = (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.game_execution_scheduler.current_work()
                == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)
            && self.dungeon_peg_attribute_flip_pending.is_some())
        .then(|| self.original_timing_dungeon_peg_attribute_flip_progress())
        .flatten();
        if let Some(progress) = progress {
            assert_eq!(self.game_state.frame.main_module, 7);
            match self
                .dungeon_peg_attribute_flip_pending
                .expect("continued peg-attribute cursor lost its saved caller")
                .caller
            {
                DungeonPegAttributeFlipCaller::UpdatePegs => {
                    assert_eq!(self.game_state.frame.submodule, 0x16);
                    assert_eq!(self.game_state.frame.subsubmodule, 0x10);
                }
                DungeonPegAttributeFlipCaller::SupertileTransition { .. } => {
                    assert_eq!(self.game_state.frame.submodule, 2);
                }
                DungeonPegAttributeFlipCaller::SpiralStairs => {
                    assert_eq!(self.game_state.frame.submodule, 0x0e);
                }
                DungeonPegAttributeFlipCaller::WarpPad => {
                    assert_eq!(self.game_state.frame.submodule, 0x15);
                }
                DungeonPegAttributeFlipCaller::FallingTransition => {
                    assert_eq!(self.game_state.frame.submodule, 7);
                }
                DungeonPegAttributeFlipCaller::FatInterRoomStairs => {
                    assert_eq!(self.game_state.frame.submodule, 6);
                }
                DungeonPegAttributeFlipCaller::StraightInterroomStairs => {
                    assert!(matches!(self.game_state.frame.submodule, 0x11..=0x13));
                }
            }
            let semantic = self
                .original_timing_semantic_receipts
                .as_ref()
                .expect("continued peg-attribute cursor lost its host authority")
                .semantic();
            let progress_index = semantic
                .iter()
                .position(|receipt| {
                    *receipt
                        == OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress)
                })
                .expect("continued peg-attribute cursor disappeared during preflight");
            if progress.boundary == OriginalTimingBoundary::NmiAccepted {
                assert_eq!(
                    semantic.get(progress_index + 1),
                    Some(&OriginalTimingSemanticReceipt::NmiAccepted(
                        NmiUpdateGate::LatchHeld,
                    )),
                    "a continued peg-attribute cursor must immediately precede the held NMI which exposes it",
                );
            }
        }
        progress
    }

    fn pre_main_caller_continuation_is(&self, continuation: PreMainCallerContinuation) -> bool {
        self.game_execution_scheduler
            .pre_main_caller_continuation_is(continuation)
    }

    fn finish_pre_main_caller_continuation(&mut self, expected: PreMainCallerContinuation) {
        self.game_execution_scheduler
            .finish_pre_main_caller_continuation(expected);
    }

    pub fn set_rom(&mut self, rom: &[u8]) {
        self.rom = strip_copier_header(rom).to_vec();
    }

    pub fn set_assets(&mut self, assets: &[u8]) -> Result<(), String> {
        let parsed = AssetPack::parse(assets)?;
        let driver_clock_assets = match (
            parsed.asset_by_name(SPC_DRIVER_TIMING_ASSET_NAME),
            parsed.asset(0),
        ) {
            (Some(driver), Some(intro_bank)) => Some((driver, intro_bank)),
            _ => None,
        };
        if let Some((driver, intro_bank)) = driver_clock_assets {
            self.initialize_spc_driver_clock(driver, intro_bank)?;
            if self.rom_startup_timing {
                self.configure_spc_driver_clock_for_rom_bootstrap();
            }
        } else {
            self.clear_spc_driver_clock();
        }
        self.assets = Some(parsed);
        self.gloves_color = default_gloves_color();
        Ok(())
    }
    /// `zelda_run_frame_internal`.
    ///
    /// The actual module routing, poly loop, and NMI handler are intentionally
    /// skeletal. Future ports should land behind this entry point so the
    /// lockstep oracle starts validating them immediately.
    pub fn run_frame_internal(&mut self, input: u16, run_what: u8) {
        let owns_original_timing_dispatch = self.begin_original_timing_host_dispatch(input);
        self.run_frame_internal_after_original_timing(input, run_what);
        self.finish_original_timing_host_dispatch(owns_original_timing_dispatch);
    }

    pub fn zelda_run_frame_internal(&mut self, input: u16, run_what: u8) {
        self.run_frame_internal(input, run_what);
    }

    pub fn zelda_setup_emu_callbacks(
        &mut self,
        emu_ram: Option<Vec<u8>>,
        func: Option<ZeldaRunFrameFunc>,
        sync_all: Option<ZeldaSyncAllFunc>,
    ) {
        self.emu_memory_ptr = emu_ram;
        self.emu_runframe = func;
        self.emu_syncall = sync_all;
    }

    fn emu_sync_memory_region(&mut self, offset: usize, n: usize) {
        debug_assert!(offset < WRAM_SIZE);
        debug_assert!(offset + n <= WRAM_SIZE);
        let bytes = self.compatibility_ram_range(offset, n).to_vec();
        if let Some(emu_memory_ptr) = self.emu_memory_ptr.as_mut() {
            if emu_memory_ptr.len() < WRAM_SIZE {
                emu_memory_ptr.resize(WRAM_SIZE, 0);
            }
            emu_memory_ptr[offset..offset + n].copy_from_slice(&bytes);
        }
    }

    fn zelda_ppu_write(&mut self, adr: u32, val: u8) {
        debug_assert!((0x2100..=0x213f).contains(&adr));
        self.ppu.write(adr as u8, val);
    }

    fn zelda_ppu_write_word(&mut self, adr: u32, val: u16) {
        self.zelda_ppu_write(adr, val as u8);
        self.zelda_ppu_write(adr + 1, (val >> 8) as u8);
    }

    fn ram_bytes(&self, offset: usize, len: usize) -> Vec<u8> {
        Self::ram_bytes_from(&self.ram, offset, len)
    }

    fn ram_bytes_from(ram: &[u8], offset: usize, len: usize) -> Vec<u8> {
        ram.get(offset..offset + len)
            .map_or_else(Vec::new, |bytes| bytes.to_vec())
    }

    fn u16_table_bytes(table: &[u16], byte_offset: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(table.len() * 2);
        for &value in table {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
            .get(byte_offset..)
            .map_or_else(Vec::new, |s| s.to_vec())
    }

    /// Simulate 224 HDMA scanlines and capture (fixed_color_r, fixed_color_g, fixed_color_b)
    /// per scanline.  Used for GPU color math parity diagnostics.
    pub fn ppu_scanline_fixed_color(&mut self) -> Box<[(u8, u8, u8); 224]> {
        for i in 0..8 {
            self.dma.channel[i].hdma_active = self.game_state.display.is_hdma_channel_enabled(i);
        }
        let saved_cgram = self.ppu.cgram.clone();
        let saved_cgram_pointer = self.ppu.cgram_pointer;
        let saved_cgram_second_write = self.ppu.cgram_second_write;
        let saved_cgram_buffer = self.ppu.cgram_buffer;
        let saved_forced_blank = self.ppu.forced_blank;
        let saved_brightness = self.ppu.brightness;
        let saved_screen_enabled = self.ppu.screen_enabled;
        let saved_window1_left = self.ppu.window1_left;
        let saved_window1_right = self.ppu.window1_right;
        let saved_window2_left = self.ppu.window2_left;
        let saved_window2_right = self.ppu.window2_right;
        let saved_fcr = self.ppu.fixed_color_r;
        let saved_fcg = self.ppu.fixed_color_g;
        let saved_fcb = self.ppu.fixed_color_b;

        let channels: [_; 8] = std::array::from_fn(|i| self.dma.channel[i]);
        let mut hdma: [SimpleHdma; 8] = Default::default();
        for i in 0..8 {
            self.simple_hdma_init(&mut hdma[i], &channels[i]);
        }

        let mut result = Box::new([(0u8, 0u8, 0u8); 224]);
        for entry in result.iter_mut() {
            for i in 0..8 {
                self.simple_hdma_do_line(&mut hdma[i]);
            }
            *entry = (
                self.ppu.fixed_color_r,
                self.ppu.fixed_color_g,
                self.ppu.fixed_color_b,
            );
        }

        self.ppu.cgram = saved_cgram;
        self.ppu.cgram_pointer = saved_cgram_pointer;
        self.ppu.cgram_second_write = saved_cgram_second_write;
        self.ppu.cgram_buffer = saved_cgram_buffer;
        self.ppu.forced_blank = saved_forced_blank;
        self.ppu.brightness = saved_brightness;
        self.ppu.screen_enabled = saved_screen_enabled;
        self.ppu.window1_left = saved_window1_left;
        self.ppu.window1_right = saved_window1_right;
        self.ppu.window2_left = saved_window2_left;
        self.ppu.window2_right = saved_window2_right;
        self.ppu.fixed_color_r = saved_fcr;
        self.ppu.fixed_color_g = saved_fcg;
        self.ppu.fixed_color_b = saved_fcb;
        result
    }

    fn configure_ppu_side_space(&mut self) {
        self.sync_native_game_state_from_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        let mut extra_left = 0u16;
        let mut extra_right = 0u16;
        let mut extra_bottom = 0u16;
        let frame = self.game_state.frame;
        let mut module = frame.main_module;
        if module == 14 {
            module = self.game_state.frame.saved_module_for_menu;
        }

        if module == 9 {
            if frame.main_module == 14 && frame.submodule == 7 && self.overworld_map_state() >= 4 {
                extra_left = PPU_SIDE_SPACE_LIMIT;
                extra_right = PPU_SIDE_SPACE_LIMIT;
                extra_bottom = 16;
            } else {
                let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
                let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
                extra_left = bg2x.wrapping_sub(self.game_state.world.scroll.scroll_x_start());
                extra_right = self
                    .game_state
                    .world
                    .scroll
                    .scroll_x_end()
                    .wrapping_sub(bg2x);
                extra_bottom = self
                    .game_state
                    .world
                    .scroll
                    .scroll_y_end()
                    .wrapping_sub(bg2y);
            }
        } else if module == 7 {
            if !(self.game_state.dungeon.torch.dungeon_dark_with_lantern()
                && self.game_state.display.sub_screen_layers != 0)
            {
                let qm = (self.game_state.world.transient.quadrant_fullsize_x() >> 1) as usize;
                let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
                extra_left = bg2x.saturating_sub(self.game_state.world.room_bounds.x_bound(qm));
                extra_right = self
                    .game_state
                    .world
                    .room_bounds
                    .x_bound(qm + 2)
                    .saturating_sub(bg2x);
            }
            let qy = (self.game_state.world.transient.quadrant_fullsize_y() >> 1) as usize;
            let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
            extra_bottom = self
                .game_state
                .world
                .room_bounds
                .y_bound(qy + 2)
                .saturating_sub(bg2y);
        } else if module == 20 || module == 0 || module == 1 {
            extra_left = PPU_SIDE_SPACE_LIMIT;
            extra_right = PPU_SIDE_SPACE_LIMIT;
            extra_bottom = 16;
        }

        self.ppu.extra_left_cur = extra_left.min(PPU_SIDE_SPACE_LIMIT) as u8;
        self.ppu.extra_right_cur = extra_right.min(PPU_SIDE_SPACE_LIMIT) as u8;
        self.ppu.extra_bottom_cur = extra_bottom.min(16) as u8;
    }

    pub fn byte_array_append_vl(arr: &mut ByteArray, mut v: u32) {
        while v >= 255 {
            ByteArray_AppendByte(arr, 255);
            v -= 255;
        }
        ByteArray_AppendByte(arr, v as u8);
    }

    pub fn state_recorder_read_vl(data: &[u8], replay_pos: &mut usize) -> u32 {
        let mut value = 0u32;
        loop {
            assert!(*replay_pos < data.len());
            let byte = data[*replay_pos];
            *replay_pos += 1;
            value = value.wrapping_add(byte as u32);
            if byte != 255 {
                return value;
            }
        }
    }

    pub fn load_func(ctx: &mut LoadFuncState<'_>, data: &mut [u8]) {
        debug_assert!(ctx.remaining() >= data.len());
        let end = ctx.pos + data.len();
        data.copy_from_slice(&ctx.p[ctx.pos..end]);
        ctx.pos = end;
    }

    fn load_snes_state(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        self.internal_save_load(func);
        // Full C-format state loads back replay base snapshots, inline replay
        // snapshots, and ordinary save-state restores. None contains the
        // persistent original CPU/APU timing machine, so a pending cold seed
        // cannot survive this boundary even when the restored host frame is 0.
        self.invalidate_original_timing_after_checkpoint();
        self.restore_spotlight_hdma_from_saveload_buffer();
        self.zelda_restore_music_after_load_locked(false);
        self.sync_native_game_state_from_ram();
        // `internal_save_load` bulk-restores the whole WRAM (including the
        // palette buffers) from a full-state snapshot, bypassing the
        // provenance-aware palette bridge. `sync_native_game_state_from_ram`
        // carries the old mirror forward on the assumption that the palette
        // shadow only ever changes through that bridge — false here, so the
        // carried mirror is stale. A snapshot restore is a full-state reload
        // point (like power-on): the restored shadow is authoritative and has
        // no asset-derivation path, so reconstitute the mirror to mirror it.
        self.reconstitute_palette_mirror_from_shadow();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.sync_overworld_map16_state_from_ram();
        self.emu_synchronize_whole_state();
    }

    /// Seed this state from a foreign emulator's memory image (the pinned
    /// Snes9x oracle at a route boundary): WRAM, VRAM, SRAM, CGRAM and OAM
    /// replace ours, the native game-state mirrors are rebuilt from WRAM, the
    /// modern audio engine restarts on the WRAM music-control song, and the
    /// live original-timing machine restarts cold at `host_frame`.
    ///
    /// This is the oracle-seeded segment start: it lets a route segment run
    /// against the oracle independently of every earlier segment's Rust
    /// behavior. It is development evidence, never parity authority — the
    /// seeded APU, PPU register mirror and presentation provenance are not
    /// the oracle's exact state.
    pub fn seed_from_snes9x_oracle_memory(
        &mut self,
        wram: &[u8],
        vram: &[u8],
        sram: &[u8],
        cgram: &[u16],
        oam: &[u8],
        host_frame: u32,
    ) -> Result<(), String> {
        if wram.len() != self.ram.len() {
            return Err(format!(
                "seed WRAM has {} bytes, expected {}",
                wram.len(),
                self.ram.len()
            ));
        }
        if vram.len() != 0x1_0000 {
            return Err(format!(
                "seed VRAM has {} bytes, expected 65536",
                vram.len()
            ));
        }
        if cgram.len() != 0x100 {
            return Err(format!(
                "seed CGRAM has {} colors, expected 256",
                cgram.len()
            ));
        }
        if oam.len() != 0x220 {
            return Err(format!("seed OAM has {} bytes, expected 544", oam.len()));
        }
        if sram.is_empty() || sram.len() > self.sram.len() {
            return Err(format!(
                "seed SRAM has {} bytes, expected 1..={}",
                sram.len(),
                self.sram.len()
            ));
        }
        self.ram.copy_from_slice(wram);
        self.sram[..sram.len()].copy_from_slice(sram);
        self.ppu.vram = vram
            .chunks_exact(2)
            .map(|w| u16::from_le_bytes([w[0], w[1]]))
            .collect();
        self.ppu.cgram = cgram.to_vec();
        self.ppu.oam = oam
            .chunks_exact(2)
            .map(|w| u16::from_le_bytes([w[0], w[1]]))
            .collect();
        // Drop every boot-only/transient live-timing field a fresh state
        // carries (reset delay, SPC startup phase, intro thread phases, ...),
        // then re-enable live timing as a progressed (non-cold) state.
        self.set_rom_startup_timing(false);
        self.frame_ctr_dbg = host_frame;
        self.state_recorder.replay_frame_counter = host_frame;
        self.set_rom_startup_timing(true);
        // The spotlight HDMA dynamic table lives in WRAM ($1B00 scratch is a
        // C save-time projection which a foreign WRAM image never holds), so
        // the native table is rebuilt from WRAM below; no saveload copy.
        self.seed_saved_music_ports_from_wram(wram);
        self.zelda_restore_music_after_load_locked(false);
        self.sync_native_game_state_from_ram();
        self.reconstitute_palette_mirror_from_shadow();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.sync_overworld_map16_state_from_ram();
        self.emu_synchronize_whole_state();
        // Re-arm the live original-timing owner cold at this host: the first
        // installed receipts belong to host `host_frame`, nothing is carried.
        self.restore_original_timing_resume_checkpoint(crate::OriginalTimingResumeCheckpoint {
            schema: crate::OriginalTimingResumeCheckpoint::SCHEMA,
            last_consumed_host_call: Some(u64::from(host_frame).saturating_sub(1)),
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            dungeon_exit_spotlight_entry_return_pending: false,
            pre_dungeon_return_pending: None,
            item_receipt_live_link_dma_host: None,
        })
        .map_err(|error| format!("seeded state could not re-arm live timing: {error:?}"))?;
        Ok(())
    }

    pub fn state_recorder_init(sr: &mut StateRecorder) {
        *sr = StateRecorder::default();
    }

    pub fn state_recorder_record_cmd(sr: &mut StateRecorder, cmd: u8) {
        let frames = sr.frames_since_last;
        sr.frames_since_last = 0;
        let x = if cmd < 0xc0 { 0xf } else { 0x1 };
        ByteArray_AppendByte(
            &mut sr.log,
            cmd | if frames < x { frames as u8 } else { x as u8 },
        );
        if frames >= x {
            Self::byte_array_append_vl(&mut sr.log, frames - x);
        }
    }

    pub fn state_recorder_record(sr: &mut StateRecorder, inputs: u16) {
        let diff = inputs ^ sr.last_inputs;
        if diff != 0 {
            sr.last_inputs = inputs;
            for i in 0..12 {
                if (diff >> i) & 1 != 0 {
                    Self::state_recorder_record_cmd(sr, (i << 4) as u8);
                }
            }
        }
        sr.frames_since_last = sr.frames_since_last.wrapping_add(1);
        sr.total_frames = sr.total_frames.wrapping_add(1);
    }

    pub fn state_recorder_record_patch_byte(
        sr: &mut StateRecorder,
        addr: u32,
        value: &[u8],
        num: usize,
    ) {
        assert!(addr < 0x20000);
        assert!(num <= value.len());
        let lq = (num.saturating_sub(1)).min(3);
        Self::state_recorder_record_cmd(
            sr,
            0xc0 | (if addr & 0x10000 != 0 { 2 } else { 0 }) | ((lq as u8) << 2),
        );
        if lq == 3 {
            Self::byte_array_append_vl(&mut sr.log, (num - 1 - 3) as u32);
        }
        ByteArray_AppendByte(&mut sr.log, (addr >> 8) as u8);
        ByteArray_AppendByte(&mut sr.log, addr as u8);
        for &byte in value.iter().take(num) {
            ByteArray_AppendByte(&mut sr.log, byte);
        }
    }

    pub fn state_recorder_clear_key_log(&mut self, sr: &mut StateRecorder) {
        sr.base_snapshot.data.clear();
        let mut save = SaveLoadFunc::Save(&mut sr.base_snapshot);
        self.save_snes_state(&mut save);

        let old_log = std::mem::take(&mut sr.log);
        let old_frames_since_last = sr.frames_since_last;
        sr.frames_since_last = 0;
        if sr.last_inputs != 0 {
            for i in 0..12 {
                if (sr.last_inputs >> i) & 1 != 0 {
                    Self::state_recorder_record_cmd(sr, (i << 4) as u8);
                }
            }
        }
        if sr.replay_mode {
            if sr.replay_next_cmd_at != u32::MAX {
                sr.replay_next_cmd_at = sr.replay_next_cmd_at.wrapping_sub(old_frames_since_last);
                sr.frames_since_last = sr.replay_next_cmd_at;
                sr.replay_pos_last_complete = sr.log.size() as u32;
                Self::state_recorder_record_cmd(sr, sr.replay_cmd);
                let old_replay_pos = sr.replay_pos as usize;
                sr.replay_pos = sr.log.size() as u32;
                ByteArray_AppendData(&mut sr.log, &old_log.data[old_replay_pos..]);
            }
            sr.total_frames = sr.total_frames.wrapping_sub(sr.replay_frame_counter);
            sr.replay_frame_counter = 0;
        } else {
            sr.total_frames = 0;
        }
        sr.frames_since_last = 0;
    }

    pub fn read_from_file<R: Read>(f: &mut R, data: &mut [u8]) {
        f.read_exact(data).expect("fread failed");
    }

    pub fn state_recorder_load<R: Read>(
        &mut self,
        sr: &mut StateRecorder,
        f: &mut R,
        replay_mode: bool,
    ) {
        let mut hdr_bytes = [0u8; 32];
        Self::read_from_file(f, &mut hdr_bytes);
        let mut hdr = [0u32; 8];
        for i in 0..8 {
            hdr[i] = u32::from_le_bytes([
                hdr_bytes[i * 4],
                hdr_bytes[i * 4 + 1],
                hdr_bytes[i * 4 + 2],
                hdr_bytes[i * 4 + 3],
            ]);
        }
        assert_eq!(hdr[0], 1);

        sr.total_frames = hdr[1];
        sr.log.data.resize(hdr[2] as usize, 0);
        Self::read_from_file(f, &mut sr.log.data);
        sr.last_inputs = hdr[3] as u16;
        sr.frames_since_last = hdr[4];

        sr.base_snapshot
            .data
            .resize(if hdr[5] & 1 != 0 { hdr[6] as usize } else { 0 }, 0);
        Self::read_from_file(f, &mut sr.base_snapshot.data);

        sr.replay_next_cmd_at = 0;
        sr.replay_mode = replay_mode;
        if replay_mode {
            sr.frames_since_last = 0;
            sr.last_inputs = 0;
            sr.replay_pos = 0;
            sr.replay_pos_last_complete = 0;
            sr.replay_frame_counter = 0;
            if !sr.base_snapshot.data.is_empty() {
                let mut state = LoadFuncState::new(&sr.base_snapshot.data);
                let mut load = SaveLoadFunc::Load(&mut state);
                self.load_snes_state(&mut load);
                assert_eq!(state.remaining(), 0);
            } else {
                self.zelda_reset(false);
            }
        } else {
            sr.replay_pos = hdr[5] >> 1;
            sr.replay_pos_last_complete = sr.replay_pos;
            sr.replay_frame_counter = hdr[7];
            sr.replay_mode = sr.replay_frame_counter != 0;

            let mut arr = vec![0; hdr[6] as usize];
            Self::read_from_file(f, &mut arr);
            let mut state = LoadFuncState::new(&arr);
            let mut load = SaveLoadFunc::Load(&mut state);
            self.load_snes_state(&mut load);
            assert_eq!(state.remaining(), 0);
        }
    }

    pub fn input_state_read_from_file(&self) -> i32 {
        0
    }

    /// Match Snes9x libretro's opposing-direction handling.
    ///
    /// The core reports buttons in libretro ID order: Up, Down, Left, Right.
    /// With `Settings.UpAndDown` disabled, each pressed direction first clears
    /// both directions on its axis and then sets itself. Consequently Down wins
    /// Up+Down and Right wins Left+Right. Preserve every non-direction button.
    fn sanitize_frame_inputs(inputs: i32) -> u16 {
        sanitize_original_timing_input(inputs as u16)
    }

    pub fn zelda_run_frame(&mut self, inputs: i32) -> bool {
        self.zelda_run_frame_with_replay_input_override(inputs, None)
    }

    pub fn zelda_set_language(&mut self, language: Option<&str>) {
        let mut found = [0u8, 0, 0];
        if let Some(language) = language {
            let language_bytes = config_value_bytes(language);
            for i in 0.. {
                let Some(map) = self.asset_memblk(96, i) else {
                    eprintln!("Unable to find language '{}'", language);
                    break;
                };
                let name = find_index_in_memblk(map, 0);
                if name.ptr == language_bytes {
                    let conf = find_index_in_memblk(map, 1);
                    if conf.ptr.len() >= 3 {
                        found.copy_from_slice(&conf.ptr[..3]);
                    }
                    break;
                }
            }
        }
        self.dialogue_blk_index = found[0] as usize;
        self.dialogue_font_blk_index = found[1] as usize;
        self.dialogue_flags = found[2];
    }

    pub fn state_recoder_multi_patch_init(mp: &mut StateRecoderMultiPatch) {
        mp.count = 0;
        mp.addr = 0;
    }

    pub fn state_recoder_multi_patch_commit(
        sr: &mut StateRecorder,
        mp: &mut StateRecoderMultiPatch,
    ) {
        if mp.count != 0 {
            Self::state_recorder_record_patch_byte(sr, mp.addr, &mp.vals, mp.count as usize);
        }
    }

    pub fn state_recoder_multi_patch_patch(
        &mut self,
        sr: &mut StateRecorder,
        mp: &mut StateRecoderMultiPatch,
        addr: u32,
        value: u8,
    ) {
        if mp.count >= 256 || addr != mp.addr.wrapping_add(mp.count) {
            Self::state_recoder_multi_patch_commit(sr, mp);
            mp.addr = addr;
            mp.count = 0;
        }
        mp.vals[mp.count as usize] = value;
        mp.count += 1;
        self.set_compatibility_ram_byte(addr as usize, value);
        self.emu_sync_memory_region(addr as usize, 1);
    }

    pub fn patch_command(&mut self, c: char) {
        let mut state_recorder = std::mem::take(&mut self.state_recorder);
        let mut mp = StateRecoderMultiPatch::default();
        Self::state_recoder_multi_patch_init(&mut mp);
        match c {
            'w' => {
                self.state_recoder_multi_patch_patch(
                    &mut state_recorder,
                    &mut mp,
                    wram_patch_addr(LINK_HEARTS_FILLER),
                    80,
                );
                self.state_recoder_multi_patch_patch(
                    &mut state_recorder,
                    &mut mp,
                    wram_patch_addr(LINK_MAGIC_FILLER),
                    80,
                );
            }
            'W' => {
                self.state_recoder_multi_patch_patch(
                    &mut state_recorder,
                    &mut mp,
                    wram_patch_addr(LINK_BOMB_FILLER),
                    10,
                );
                self.state_recoder_multi_patch_patch(
                    &mut state_recorder,
                    &mut mp,
                    wram_patch_addr(LINK_ARROW_REFILL_COUNTER),
                    10,
                );
                let rupees = self
                    .game_state
                    .inventory
                    .player_resources
                    .rupees_goal()
                    .wrapping_add(100);
                self.state_recoder_multi_patch_patch(
                    &mut state_recorder,
                    &mut mp,
                    wram_patch_addr(LINK_RUPEES_GOAL),
                    rupees as u8,
                );
                self.state_recoder_multi_patch_patch(
                    &mut state_recorder,
                    &mut mp,
                    wram_patch_addr(LINK_RUPEES_GOAL + 1),
                    (rupees >> 8) as u8,
                );
            }
            'k' => self.state_recorder_clear_key_log(&mut state_recorder),
            'o' => self.state_recoder_multi_patch_patch(
                &mut state_recorder,
                &mut mp,
                wram_patch_addr(LINK_NUM_KEYS),
                1,
            ),
            'l' => Self::state_recorder_stop_replay(&mut state_recorder),
            'E' => self.state_recoder_multi_patch_patch(
                &mut state_recorder,
                &mut mp,
                wram_patch_addr(CHEAT_WALK_THROUGH_WALLS),
                self.game_state
                    .player
                    .follower_link
                    .cheat_walk_through_walls()
                    ^ 1,
            ),
            _ => {}
        }
        Self::state_recoder_multi_patch_commit(&mut state_recorder, &mut mp);
        self.state_recorder = state_recorder;
    }

    fn zelda_initialization_code(&mut self) {
        self.sound_load_intro_song_bank();
        self.startup_initialize_memory();
        self.finish_rom_bootstrap_initialization();
    }

    fn finish_rom_bootstrap_initialization(&mut self) {
        self.set_animated_tile_data_source_address(0xa680);
        self.sync_native_game_state_from_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
    }

    fn startup_initialize_memory(&mut self) {
        const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_0: usize = 0x0000;
        const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_1: usize = 0x0001;
        const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_2: usize = 0x0002;

        SystemWorkArea::clear_startup_low_memory(&mut self.ram);
        // The reset code at ROM $008900 initially writes `00 80 19`, but the
        // Snes9x DMA trace proves that the first visible NMI reads `00 80 00`.
        // This port reaches this setup after reset execution, so retain only
        // the bytes that remain live at that NMI boundary.
        self.ram[FIRST_BOOT_NMI_DMA_SOURCE_BYTE_0] = 0x00;
        self.ram[FIRST_BOOT_NMI_DMA_SOURCE_BYTE_1] = 0x80;
        // WRAM palette buffers are zero at power-on: seed the provenance mirror
        // so words the game never explicitly writes (transparent color-0 slots)
        // read as a known constant 0 rather than Unknown.
        self.initialize_palette_mirror_from_zeroed_buffers();
        self.set_main_color_constant(0, 0);
        self.clear_selected_save_slot();

        for offset in [0x03e5, 0x08e5, 0x0de5] {
            if read_le_u16(&self.sram, offset) != 0x55aa {
                write_le_u16(&mut self.sram, offset, 0);
            }
        }

        self.set_screen_brightness(0x80);
        self.increment_cgram_update_flag();
        self.sync_native_game_state_from_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
    }

    fn zelda_run_game_loop(&mut self) {
        let authoritative_main_loop_progress = self.take_original_timing_main_loop_progress();
        self.zelda_run_game_loop_with_progress(authoritative_main_loop_progress);
    }

    fn zelda_run_game_loop_with_progress(
        &mut self,
        authoritative_main_loop_progress: Option<crate::MainLoopProgress>,
    ) {
        let mut dialogue_text_dma = None;
        self.zelda_run_game_loop_with_progress_and_dialogue_text_dma(
            authoritative_main_loop_progress,
            &mut dialogue_text_dma,
            None,
            None,
        );
        assert!(
            dialogue_text_dma.is_none(),
            "a BG3 text-DMA publication escaped its immediate dialogue staging boundary",
        );
    }

    fn zelda_run_game_loop_body(
        &mut self,
        authoritative_main_loop_progress: Option<crate::MainLoopProgress>,
    ) {
        let mut dialogue_text_dma = None;
        self.zelda_run_game_loop_body_with_dialogue_text_dma(
            authoritative_main_loop_progress,
            &mut dialogue_text_dma,
            None,
            None,
        );
        assert!(
            dialogue_text_dma.is_none(),
            "a directly executed main-loop body lost its BG3 text-DMA publication evidence",
        );
    }

    fn complete_zelda_run_game_loop_after_module_routing(&mut self) {
        self.dialogue_live_message_read_position_target = None;
        self.replay_trace_ram_watch("game-loop-after-module");
        // A vblank can interrupt the 65816 inside the VWF loop. Keep the
        // main-thread continuation separate from the ROM's $0710 NMI gate:
        // mid-glyph $0710 is still zero, while a completed handler sets it to 2.
        self.dialogue_fast_forward_hold_active =
            std::mem::take(&mut self.dialogue_fast_forward_hold_pending);
    }

    fn complete_pending_main_loop_common_suffix_after_module_return(&mut self) {
        let Some(continuation) = self.pending_main_loop_common_suffix.take() else {
            return;
        };
        // The typed source return supersedes the atomic runtime's one-host
        // partial-NMI marker. Keeping it beyond this C call would suppress an
        // unrelated later ZeldaRunGameLoop suffix.
        self.rom_load_partial_nmi_this_frame = false;
        // A completed shared suffix proves the ROM returned to its main
        // wait: a VWF fast-forward hold cannot survive that return, and a
        // stale one would suppress the next fresh iteration's own suffix
        // (route host 154795, the post-game-over dialogue).
        self.dialogue_fast_forward_hold_active = false;
        match continuation {
            MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch => {
                assert!(
                    !self.main_loop_sprite_preparation_completed,
                    "pending ZeldaRunGameLoop common suffix would repeat NMI_PrepareSprites",
                );
                self.nmi_prepare_sprites_for_main_loop();
                self.clear_nmi_update_latch();
            }
            MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
                next_group_start,
            } => {
                assert!(
                    !self.main_loop_sprite_preparation_completed,
                    "resumed ZeldaRunGameLoop common suffix would repeat NMI_PrepareSprites",
                );
                self.nmi_prepare_sprites_resume_after_extended_oam_packing(next_group_start);
                self.main_loop_sprite_preparation_completed = true;
                self.clear_nmi_update_latch();
            }
        }
    }

    fn complete_pending_main_loop_common_suffix_from_source_return(&mut self) -> bool {
        if self.pending_main_loop_common_suffix.is_none()
            || !self.take_original_timing_main_loop_iteration_returned_to_wait()
        {
            return false;
        }
        self.complete_pending_main_loop_common_suffix_after_module_return();
        true
    }

    fn finish_recoil_landing(&mut self) {
        if self.game_state.player.follower_link.lower_level_state() == 2 {
            self.follower_link_state_mut().set_lower_level_state(0);
        }
        if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            self.dungeon_handle_layer_change();
        }
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_position_mode();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().clear_actual_velocity_xy();
    }

    fn finish_ground_movement_tail(&mut self, clear_vel_after: bool) {
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        if clear_vel_after {
            self.finish_ground_movement_clear_vel_tail();
            return;
        }
        self.finish_ground_movement_camera_tail();
    }

    fn finish_ground_movement_clear_vel_tail(&mut self) {
        self.follower_link_state_mut().clear_movement_velocity();
        self.finish_ground_movement_camera_tail();
    }

    fn read_predefined_tile_words(&self, src: u16, count: usize) -> Vec<u16> {
        let start = (src >> 1) as usize;
        (0..count).map(|i| self.asset_u16(69, start + i)).collect()
    }

    fn apply_opened_chest_tiles(&mut self, pos: u16, loc: u16, src: &[u16]) -> u16 {
        let attr = if loc < 0x8000 { 0x27 } else { 0x00 };
        let positions = [pos, pos + 64, pos + 1, pos + 65];
        for (i, &tile_pos) in positions.iter().enumerate() {
            // C writes `dung_bg2[tile_pos]` flat; a chest at tile_pos >= 0x1000 spills into
            // the contiguous BG1 span. set_bg2_tile drops that OOB index, leaving the opened
            // chest's spilled tile undrawn — route through the spill-aware path (as
            // Dungeon_Store2x2 / set_spiral_stair_wall_priority do).
            self.dungeon_room_tilemaps_mut().set_room_tilemap_word(
                crate::game_state::constants::DUNG_BG2,
                tile_pos,
                src[i],
            );
            self.dungeon_bg2_attributes_mut()
                .set_bg2_attr(tile_pos as usize, attr);
        }

        let dst = self.game_state.display.current_vram_upload_data_address();
        for (i, &tile_pos) in positions.iter().enumerate() {
            let base = dst + i * 6;
            let addr = self.Dungeon_MapVramAddr(tile_pos);
            self.write_vram_upload_absolute_word(base, addr);
            self.write_vram_upload_absolute_word(base + 2, 0x0100);
            self.write_vram_upload_absolute_word(base + 4, src[i]);
        }
        self.write_vram_upload_absolute_word(dst + 24, 0xffff);
        self.advance_vram_upload_cursor_by(24);
        self.set_bg_vram_load_mode(1);
        self.Dungeon_FlagRoomData_Quadrants();
        if self.game_state.system_signals.sound_effect_2() == 0 {
            self.set_sound_effect_2(14);
        }
        loc & 0x7fff
    }

    fn set_backdrop_color_black(&mut self) {
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
    }

    fn write_intro_x(&mut self, k: usize, value: i16) {
        self.intro_actor_mut(k).set_x(value);
    }

    fn write_intro_y(&mut self, k: usize, value: i16) {
        self.intro_actor_mut(k).set_y(value);
    }

    fn rom_byte_snes(&self, addr: u32) -> Option<u8> {
        if addr & 0x8000 == 0 {
            return None;
        }
        let offset = (((addr >> 16) & 0x7f) as usize) * 0x8000 + (addr as usize & 0x7fff);
        self.rom.get(offset).copied()
    }

    fn rom_word_snes(&self, addr: u32) -> Option<u16> {
        Some(self.rom_byte_snes(addr)? as u16 | ((self.rom_byte_snes(addr + 1)? as u16) << 8))
    }

    fn rom_or_asset_word_snes(&self, addr: u32) -> Option<u16> {
        self.rom_word_snes(addr)
            .or_else(|| self.palette_asset_word_snes(addr))
    }

    fn rom_bytes_snes(&self, mut addr: u32, len: usize) -> Option<Vec<u8>> {
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(self.rom_byte_snes(addr)?);
            addr = next_snes_addr(addr);
        }
        Some(bytes)
    }

    fn asset_memblk(&self, asset: usize, index: usize) -> Option<MemBlk<'_>> {
        let asset = self.assets.as_ref()?.asset(asset)?;
        Some(find_index_in_memblk(MemBlk { ptr: asset }, index))
    }

    fn asset_raw(&self, asset: usize) -> Option<&[u8]> {
        self.assets.as_ref()?.asset(asset)
    }

    fn asset_u8(&self, asset: usize, index: usize) -> u8 {
        self.asset_raw(asset)
            .and_then(|data| data.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn asset_u16(&self, asset: usize, index: usize) -> u16 {
        self.asset_raw(asset)
            .map(|data| read_word_from_slice(data, index * 2))
            .unwrap_or(0)
    }

    fn clear_intro_wram_block_columns(&mut self, start_offset: u16, stop_offset: u16) -> u16 {
        let next_offset = SystemWorkArea::clear_intro_wram_block_columns(
            &mut self.ram,
            start_offset,
            stop_offset,
        );
        self.sync_native_game_state_from_ram();
        next_offset
    }

    #[cfg(test)]
    fn debug_compatibility_ram_u32(&self, offset: usize) -> u32 {
        let bytes = self.compatibility_ram_range(offset, 4);
        u32::from(bytes[0])
            | (u32::from(bytes[1]) << 8)
            | (u32::from(bytes[2]) << 16)
            | (u32::from(bytes[3]) << 24)
    }
}

impl Default for ZeldaState {
    fn default() -> Self {
        Self::new()
    }
}

fn strip_copier_header(rom: &[u8]) -> &[u8] {
    if rom.len() & 0xfffff == 0x200 {
        &rom[0x200..]
    } else {
        rom
    }
}

fn next_snes_addr(addr: u32) -> u32 {
    let next = addr.wrapping_add(1);
    if next & 0x8000 == 0 {
        next.wrapping_add(0x8000)
    } else {
        next
    }
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| "asset offset overflow".to_string())?;
    let word = bytes
        .get(offset..end)
        .ok_or_else(|| "asset header truncated".to_string())?;
    Ok(u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
}

fn main_tileset(index: usize) -> [u8; 8] {
    match index {
        0 => [0, 1, 16, 6, 14, 31, 24, 15],
        1 => [0, 1, 16, 8, 14, 34, 27, 15],
        2 => [0, 1, 16, 6, 14, 31, 24, 15],
        3 => [0, 1, 19, 7, 14, 35, 28, 15],
        35 => [22, 57, 29, 23, 64, 65, 57, 30],
        _ => [0; 8],
    }
}

fn aux_tileset(index: usize) -> [u8; 4] {
    match index {
        0 => [6, 0, 31, 24],
        1 => [8, 0, 34, 27],
        2 => [6, 0, 31, 24],
        3 => [7, 0, 35, 28],
        81 => [23, 64, 65, 57],
        _ => [0; 4],
    }
}

fn sprite_tileset(index: usize) -> [u8; 4] {
    match index {
        77 => [81, 73, 19, 0],
        125 => [50, 0, 0, 8],
        126 => [93, 73, 0, 82],
        127 => [85, 73, 66, 67],
        _ => [0; 4],
    }
}

fn push_block_target_is_blocked(tile_flag: u8) -> bool {
    !matches!(
        tile_flag,
        0 | 5
            | 6
            | 7
            | 8
            | 9
            | 10
            | 12..=15
            | 28
            | 32
            | 35..=37
            | 58
            | 59
            | 64
            | 72
            | 74
            | 96
            | 97
            | 98
            | 100
    )
}

fn size_1to16(width: u8, height: u8) -> u16 {
    ((width as u16) << 2 | height as u16) + 1
}

fn size_a_to_a_plus_15(width: u8, height: u8, base: u16) -> u16 {
    ((width as u16) << 2 | height as u16) + base
}

fn size_1to15_or(width: u8, height: u8, fallback: u16) -> u16 {
    let size = (width as u16) << 2 | height as u16;
    if size == 0 {
        fallback
    } else {
        size
    }
}

fn read_word_from_slice(bytes: &[u8], offset: usize) -> u16 {
    bytes.get(offset).copied().unwrap_or(0) as u16
        | ((bytes.get(offset + 1).copied().unwrap_or(0) as u16) << 8)
}

fn upper_bitmask(index: usize) -> u16 {
    UPPER_BITMASKS[index & 0x0f]
}

fn receive_item_tab1(item: u8) -> u8 {
    RTL_RECEIVE_ITEM_OAM_EXT_SIZES
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab2(item: u8) -> i8 {
    RTL_RECEIVE_ITEM_DRAW_Y_OFFSETS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab3(item: u8) -> u8 {
    RTL_RECEIVE_ITEM_PALETTE_BITS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn memory_location_to_give_item_to(item: u8) -> usize {
    GIVE_ITEM_MEMORY_LOCATIONS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn value_to_give_item_to(item: u8) -> u8 {
    GIVE_ITEM_VALUES.get(item as usize).copied().unwrap_or(0xff)
}

fn decompress_asset(src: &[u8]) -> Vec<u8> {
    let mut dst = Vec::new();
    let mut cursor = 0usize;
    loop {
        let Some(mut cmd) = src.get(cursor).copied() else {
            return dst;
        };
        cursor += 1;
        if cmd == 0xff {
            return dst;
        }

        let len = if cmd & 0xe0 != 0xe0 {
            let len = (cmd & 0x1f) as usize + 1;
            cmd &= 0xe0;
            len
        } else {
            let Some(next) = src.get(cursor).copied() else {
                return dst;
            };
            cursor += 1;
            let len = next as usize + (((cmd & 3) as usize) << 8) + 1;
            cmd = (cmd << 3) & 0xe0;
            len
        };

        if cmd == 0 {
            for _ in 0..len {
                let Some(value) = src.get(cursor).copied() else {
                    return dst;
                };
                cursor += 1;
                dst.push(value);
            }
        } else if cmd & 0x80 != 0 {
            let Some(lo) = src.get(cursor).copied() else {
                return dst;
            };
            let Some(hi) = src.get(cursor + 1).copied() else {
                return dst;
            };
            cursor += 2;
            let mut offset = lo as usize | ((hi as usize) << 8);
            for _ in 0..len {
                let value = dst.get(offset).copied().unwrap_or(0);
                dst.push(value);
                offset += 1;
            }
        } else if cmd & 0x40 == 0 {
            let Some(value) = src.get(cursor).copied() else {
                return dst;
            };
            cursor += 1;
            dst.extend(std::iter::repeat(value).take(len));
        } else if cmd & 0x20 == 0 {
            let Some(lo) = src.get(cursor).copied() else {
                return dst;
            };
            let Some(hi) = src.get(cursor + 1).copied() else {
                return dst;
            };
            cursor += 2;
            for i in 0..len {
                dst.push(if i & 1 == 0 { lo } else { hi });
            }
        } else {
            let Some(mut value) = src.get(cursor).copied() else {
                return dst;
            };
            cursor += 1;
            for _ in 0..len {
                dst.push(value);
                value = value.wrapping_add(1);
            }
        }
    }
}

// Topical split of this module (see zelda_rtl/*.rs).
mod rtl_scroll;
mod rtl_diagnostics;
mod rtl_checkpoint;
mod rtl_palette;
mod rtl_save;
mod rtl_poly;
mod rtl_actors;
mod rtl_audio;
mod rtl_dungeon;
mod rtl_overworld;
mod rtl_vram;
mod rtl_dialogue;
mod rtl_nmi;
mod rtl_cpu_schedule;
mod rtl_spotlight;
mod rtl_oam_obj;
mod rtl_publication;
mod rtl_original_timing;
mod rtl_types_display;
pub use rtl_types_display::*;
mod rtl_types_cpu_plans;
pub(crate) use rtl_types_cpu_plans::*;
mod rtl_types_original_timing;
pub use rtl_types_original_timing::*;
mod rtl_types_spotlight;
pub(crate) use rtl_types_spotlight::*;

#[cfg(test)]
mod original_timing_receipt_validation_tests {
    use super::*;

    fn install_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state
    }

    fn assert_install_rejected_without_mutation(
        semantic: Vec<OriginalTimingSemanticReceipt>,
        expected: OriginalTimingReceiptInstallError,
    ) {
        let mut state = install_state();
        let owner = state.original_timing_owner();
        assert_eq!(
            state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                0, 0, semantic,
            )),
            Err(expected),
        );
        assert_eq!(state.original_timing_owner(), owner);
        assert!(state.original_timing_semantic_receipts.is_none());
        assert_eq!(state.last_consumed_original_timing_host_call(), None);
        assert_eq!(
            state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                0,
                0,
                Vec::new(),
            )),
            Ok(()),
            "a rejected install must not reserve the host-call slot",
        );
    }

    fn assert_install_accepted(receipt: OriginalTimingSemanticReceipt) {
        let mut state = install_state();
        assert_eq!(
            state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                0,
                0,
                vec![receipt],
            )),
            Ok(()),
        );
    }

    #[test]
    fn sprite_main_interruption_install_validates_every_source_bounded_field() {
        use crate::MainLoopInterruption as I;

        for interruption in [
            I::SpriteMainAfterSlot(16),
            I::SpriteMainAfterActiveCuccoX {
                slot: 16,
                helper_ordinal: 0,
            },
            I::SpriteMainAfterActiveCuccoYSubpixel {
                slot: 16,
                helper_ordinal: 0,
            },
            I::SpriteMainAfterCuccoFleeMovement {
                slot: 16,
                helper_ordinal: 0,
            },
            I::SpriteMainAfterCuccoSubtypeIncrements {
                slot: 16,
                helper_ordinal: 0,
                completed: 1,
            },
            I::SpriteMainAfterCuccoGraphicsPublication {
                slot: 16,
                helper_ordinal: 0,
            },
            I::SpriteMainBigKeyDropGraphicsStarted(16),
            I::SpriteMainKingZoraFlippersGraphicsStarted(16),
            I::SpriteMainHappinessPondRupeeGraphicsStarted(16),
            I::SpriteMainCatfishMedallionGraphicsStarted(16),
            I::SpriteMainItemReceiptGraphicsStarted(16),
            I::SpriteMainAfterCuccoSubtypeIncrements {
                slot: 15,
                helper_ordinal: 0,
                completed: 0,
            },
            I::SpriteMainAfterCuccoSubtypeIncrements {
                slot: 15,
                helper_ordinal: 0,
                completed: 6,
            },
        ] {
            assert_install_rejected_without_mutation(
                vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    interruption,
                )],
                OriginalTimingReceiptInstallError::InvalidMainLoopInterruption,
            );
        }

        for interruption in [
            I::SpriteMainAfterSlot(15),
            I::SpriteMainAfterActiveCuccoX {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            I::SpriteMainAfterActiveCuccoYSubpixel {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            I::SpriteMainAfterCuccoFleeMovement {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            I::SpriteMainAfterCuccoGraphicsPublication {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            I::SpriteMainBigKeyDropGraphicsStarted(15),
            I::SpriteMainKingZoraFlippersGraphicsStarted(15),
            I::SpriteMainHappinessPondRupeeGraphicsStarted(15),
            I::SpriteMainCatfishMedallionGraphicsStarted(15),
            I::SpriteMainItemReceiptGraphicsStarted(15),
        ] {
            assert_install_accepted(OriginalTimingSemanticReceipt::MainLoopInterrupted(
                interruption,
            ));
        }
        for completed in 1..=5 {
            assert_install_accepted(OriginalTimingSemanticReceipt::MainLoopInterrupted(
                I::SpriteMainAfterCuccoSubtypeIncrements {
                    slot: 15,
                    helper_ordinal: u8::MAX,
                    completed,
                },
            ));
        }
    }

    #[test]
    fn extended_oam_interruption_install_requires_its_nmi_and_source_caller() {
        let interruption = OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                next_group_start: 4,
            },
        );
        for semantic in [
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                interruption.clone(),
            ],
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                interruption.clone(),
            ],
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                interruption.clone(),
            ],
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                interruption.clone(),
            ],
        ] {
            assert_install_rejected_without_mutation(
                semantic,
                OriginalTimingReceiptInstallError::InvalidMainLoopInterruption,
            );
        }

        for next_group_start in [6, 29, 32] {
            assert_install_rejected_without_mutation(
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                            next_group_start,
                        },
                    ),
                ],
                OriginalTimingReceiptInstallError::InvalidMainLoopInterruption,
            );
        }

        for next_group_start in [0, 28] {
            let mut boundary = install_state();
            assert_eq!(
                boundary.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                    0,
                    0,
                    vec![
                        OriginalTimingSemanticReceipt::MainLoopProgress(
                            crate::MainLoopProgress::IterationStarted,
                        ),
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld,),
                        OriginalTimingSemanticReceipt::MainLoopInterrupted(
                            crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                                next_group_start,
                            },
                        ),
                    ],
                ),),
                Ok(()),
            );
        }

        let mut fresh = install_state();
        assert_eq!(
            fresh.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                    interruption.clone(),
                ],
            )),
            Ok(()),
        );

        let mut continued = install_state();
        continued.pending_main_loop_common_suffix =
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
        assert_eq!(
            continued.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                    interruption,
                ],
            )),
            Ok(()),
        );
    }

    #[test]
    fn sprite_main_progress_install_validates_every_source_bounded_field() {
        use crate::SpriteMainProgress as P;

        for progress in [
            P::AfterSlot(16),
            P::AfterActiveCuccoX {
                slot: 16,
                helper_ordinal: 0,
            },
            P::AfterActiveCuccoYSubpixel {
                slot: 16,
                helper_ordinal: 0,
            },
            P::AfterCuccoFleeMovement {
                slot: 16,
                helper_ordinal: 0,
            },
            P::AfterCuccoSubtypeIncrements {
                slot: 16,
                helper_ordinal: 0,
                completed: 1,
            },
            P::AfterCuccoGraphicsPublication {
                slot: 16,
                helper_ordinal: 0,
            },
            P::BigKeyDropGraphicsStarted(16),
            P::KingZoraFlippersGraphicsStarted(16),
            P::HappinessPondRupeeGraphicsStarted(16),
            P::CatfishMedallionGraphicsStarted(16),
            P::AfterCuccoSubtypeIncrements {
                slot: 15,
                helper_ordinal: 0,
                completed: 0,
            },
            P::AfterCuccoSubtypeIncrements {
                slot: 15,
                helper_ordinal: 0,
                completed: 6,
            },
        ] {
            assert_install_rejected_without_mutation(
                vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    progress,
                )],
                OriginalTimingReceiptInstallError::InvalidSpriteMainProgress,
            );
        }

        for progress in [
            P::AfterSlot(15),
            P::AfterActiveCuccoX {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            P::AfterActiveCuccoYSubpixel {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            P::AfterCuccoFleeMovement {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            P::AfterCuccoGraphicsPublication {
                slot: 15,
                helper_ordinal: u8::MAX,
            },
            P::BigKeyDropGraphicsStarted(15),
            P::KingZoraFlippersGraphicsStarted(15),
            P::HappinessPondRupeeGraphicsStarted(15),
            P::CatfishMedallionGraphicsStarted(15),
        ] {
            assert_install_accepted(OriginalTimingSemanticReceipt::SpriteMainProgressed(
                progress,
            ));
        }
        for completed in 1..=5 {
            assert_install_accepted(OriginalTimingSemanticReceipt::SpriteMainProgressed(
                P::AfterCuccoSubtypeIncrements {
                    slot: 15,
                    helper_ordinal: u8::MAX,
                    completed,
                },
            ));
        }
    }

    #[test]
    fn duplicate_sprite_main_progress_and_excess_returns_fail_without_installing() {
        assert_install_rejected_without_mutation(
            vec![
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    crate::SpriteMainProgress::BeforeFirstSlot,
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    crate::SpriteMainProgress::BeforeFirstSlot,
                ),
            ],
            OriginalTimingReceiptInstallError::DuplicateSpriteMainProgress,
        );
        assert_install_rejected_without_mutation(
            vec![
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::SpriteMainReturned,
            ],
            OriginalTimingReceiptInstallError::DuplicateSpriteMainReturn,
        );
    }

    #[test]
    fn forwarded_main_loop_interruption_has_one_transient_owner() {
        let mut state = install_state();
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(0, 0, Vec::new()));
        state.forward_original_timing_main_loop_interruption_to_native_owner(
            crate::MainLoopInterruption::LinkOam,
            OriginalTimingBoundary::NmiAccepted,
        );
        assert!(state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic
            .is_empty());
        assert_eq!(
            state.take_forwarded_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::LinkOam,
            ),
            Some(OriginalTimingBoundary::NmiAccepted),
        );
        assert_eq!(
            state.take_forwarded_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::LinkOam,
            ),
            None,
            "a forwarded interruption is consumed exactly once",
        );

        state.forward_original_timing_main_loop_interruption_to_native_owner(
            crate::MainLoopInterruption::SpritePreparation,
            OriginalTimingBoundary::HostReturn,
        );
        assert!(state.take_original_timing_main_loop_interruption(
            crate::MainLoopInterruption::SpritePreparation,
        ));
        assert!(!state.take_original_timing_main_loop_interruption(
            crate::MainLoopInterruption::SpritePreparation,
        ));
    }

    #[test]
    fn unclaimed_forwarded_interruption_expires_with_its_host_receipt() {
        let mut state = install_state();
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_host_dispatch_active = true;
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(0, 0, Vec::new()));
        state.forward_original_timing_main_loop_interruption_to_native_owner(
            crate::MainLoopInterruption::LinkOam,
            OriginalTimingBoundary::HostReturn,
        );

        state.finish_original_timing_host_dispatch(true);

        assert!(state.original_timing_semantic_receipts.is_none());
        assert!(!state.original_timing_host_dispatch_active);
        assert_eq!(
            state.take_forwarded_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::LinkOam,
            ),
            None,
        );
    }

    #[test]
    fn timing_invalidation_clears_mode7_shadow_diagnostic() {
        fn diagnostic() -> crate::OriginalTimingMode7TransformShadowResult {
            crate::OriginalTimingMode7TransformShadowResult {
                compared_scanline_fields: 1,
                mismatched_scanline_fields: 1,
                first_mismatch: Some((0, 0)),
            }
        }

        let mut reset = install_state();
        reset.original_timing_mode7_transform_shadow_result = Some(diagnostic());
        reset.zelda_reset(true);
        assert_eq!(reset.original_timing_mode7_transform_shadow_result, None);

        let mut checkpoint = install_state();
        checkpoint.original_timing_mode7_transform_shadow_result = Some(diagnostic());
        checkpoint.invalidate_original_timing_after_checkpoint();
        assert_eq!(
            checkpoint.original_timing_mode7_transform_shadow_result,
            None,
        );

        let mut disabled = install_state();
        disabled.original_timing_mode7_transform_shadow_result = Some(diagnostic());
        disabled.set_rom_startup_timing(false);
        assert_eq!(disabled.original_timing_mode7_transform_shadow_result, None);
    }
}

#[cfg(test)]
#[path = "zelda_rtl_tests/dialogue.rs"]
mod dialogue_tests;

#[cfg(test)]
#[path = "zelda_rtl_tests/display_publication.rs"]
mod display_publication_tests;

#[cfg(test)]
#[path = "zelda_rtl_general_runtime_tests.rs"]
mod general_runtime_tests;

#[cfg(test)]
#[path = "zelda_rtl_tests/nmi.rs"]
mod nmi_tests;

#[cfg(test)]
#[path = "chr_source_tests.rs"]
mod chr_source_tests;
